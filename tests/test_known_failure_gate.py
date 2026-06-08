import importlib.util
import json
import sqlite3
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "tools" / "known_failure_gate.py"
SPEC = importlib.util.spec_from_file_location("known_failure_gate", SCRIPT)
known_failure_gate = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(known_failure_gate)


class KnownFailureGateTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.db_path = Path(self.tmp.name) / "harness.sqlite3"
        self.conn = sqlite3.connect(self.db_path)
        self.conn.row_factory = sqlite3.Row
        self.create_schema()

    def tearDown(self):
        self.conn.close()

    def create_schema(self):
        self.conn.executescript(
            """
            CREATE TABLE test_runs (
                id INTEGER PRIMARY KEY,
                command TEXT NOT NULL,
                status TEXT NOT NULL
            );
            CREATE TABLE test_results (
                id INTEGER PRIMARY KEY,
                run_id INTEGER NOT NULL,
                nodeid TEXT NOT NULL,
                file TEXT NOT NULL DEFAULT 'cargo',
                status TEXT NOT NULL,
                message TEXT NOT NULL DEFAULT ''
            );
            CREATE TABLE metric_samples (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                value REAL NOT NULL
            );
            """
        )
        self.conn.commit()

    def insert_run(self, run_id, status, failures=()):
        self.conn.execute(
            "INSERT INTO test_runs(id, command, status) VALUES (?, ?, ?)",
            (run_id, "tools/run-tests.sh", status),
        )
        for index, nodeid in enumerate(failures, start=1):
            self.conn.execute(
                """
                INSERT INTO test_results(run_id, nodeid, file, status, message)
                VALUES (?, ?, 'cargo', 'failed', '')
                """,
                (run_id, nodeid),
            )
        self.conn.commit()

    def run_gate(self, run_id):
        completed = subprocess.run(
            [
                sys.executable,
                str(SCRIPT),
                "--db",
                str(self.db_path),
                "--run-id",
                str(run_id),
            ],
            cwd=ROOT,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )
        payload = json.loads(completed.stdout)
        return completed, payload

    def meta(self, key):
        row = self.conn.execute(
            "SELECT value FROM metadata WHERE key = ?",
            (key,),
        ).fetchone()
        return None if row is None else row["value"]

    def test_green_run_records_full_green_metric_eligible_state(self):
        self.insert_run(1, "passed")

        completed, payload = self.run_gate(1)

        self.assertEqual(completed.returncode, 0, completed.stderr)
        self.assertEqual(payload["mode"], known_failure_gate.MODE_FULL_GREEN)
        self.assertTrue(payload["metric_eligible"])
        self.assertEqual(self.meta("test_gate_mode"), known_failure_gate.MODE_FULL_GREEN)
        self.assertEqual(self.meta("test_gate_metric_eligible"), "1")

    def test_exact_known_red_quarantines_without_metric_sample(self):
        self.insert_run(2, "failed", reversed(known_failure_gate.KNOWN_FAILURE_BASELINE))

        completed, payload = self.run_gate(2)

        self.assertEqual(completed.returncode, 0, completed.stderr)
        self.assertEqual(payload["mode"], known_failure_gate.MODE_QUARANTINED)
        self.assertFalse(payload["metric_eligible"])
        self.assertEqual(payload["failures"], sorted(known_failure_gate.KNOWN_FAILURE_BASELINE))
        self.assertEqual(self.meta("test_gate_known_failure_owner_card"), "1808")
        self.assertEqual(self.meta("test_gate_metric_eligible"), "0")
        count = self.conn.execute("SELECT COUNT(*) FROM metric_samples").fetchone()[0]
        self.assertEqual(count, 0)

    def test_missing_known_failure_fails_until_baseline_updates(self):
        self.insert_run(3, "failed", [known_failure_gate.KNOWN_FAILURE_BASELINE[0]])

        completed, payload = self.run_gate(3)

        self.assertEqual(completed.returncode, 1)
        self.assertEqual(payload["mode"], known_failure_gate.MODE_FAILED)
        self.assertIn("Baseline failures absent without baseline update", payload["reason"])
        self.assertEqual(self.meta("test_gate_metric_eligible"), "0")

    def test_extra_or_different_failure_fails_as_new_regression(self):
        self.insert_run(
            4,
            "failed",
            [*known_failure_gate.KNOWN_FAILURE_BASELINE, "tests::new_unexpected_failure"],
        )

        completed, payload = self.run_gate(4)

        self.assertEqual(completed.returncode, 1)
        self.assertEqual(payload["mode"], known_failure_gate.MODE_FAILED)
        self.assertIn("New or different failures", payload["reason"])
        self.assertEqual(payload["unexpected_failures"], ["tests::new_unexpected_failure"])


if __name__ == "__main__":
    unittest.main()
