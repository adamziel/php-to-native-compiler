#!/usr/bin/env python3
"""Classify full-suite test results against a pinned known-failure baseline."""

from __future__ import annotations

import argparse
import json
import sqlite3
import sys
from datetime import datetime, timezone
from pathlib import Path


KNOWN_FAILURE_OWNER_CARD = "1808"
KNOWN_FAILURE_REMOVAL_CONDITION = (
    "Remove or update only after card 1808 is reviewed/integrated or a new "
    "scheduler-approved quarantine baseline is recorded."
)
KNOWN_FAILURE_BASELINE = (
    "tests::native_closure_invoke_helpers_bridge_call_arguments_to_call_results",
    "tests::native_magic_method_lookup_rejects_malformed_signature_metadata_before_fallback",
)

MODE_FULL_GREEN = "full_green"
MODE_QUARANTINED = "quarantined_known_red_no_regressions"
MODE_FAILED = "failed_new_regressions"


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Classify a harness test_runs row as full green, exact known-red "
            "quarantine, or failed/new-regressions."
        )
    )
    parser.add_argument(
        "--db",
        default=".harness/harness.sqlite3",
        help="Harness SQLite database path.",
    )
    parser.add_argument(
        "--run-id",
        type=int,
        help="Specific test_runs.id to classify. Defaults to latest matching command.",
    )
    parser.add_argument(
        "--command",
        default="tools/run-tests.sh",
        help="test_runs.command to use when --run-id is omitted.",
    )
    parser.add_argument(
        "--no-write-metadata",
        action="store_true",
        help="Do not update the metadata table.",
    )
    return parser.parse_args(argv)


def connect(path: str) -> sqlite3.Connection:
    conn = sqlite3.connect(path)
    conn.row_factory = sqlite3.Row
    return conn


def load_run(conn: sqlite3.Connection, run_id: int | None, command: str) -> sqlite3.Row:
    if run_id is not None:
        row = conn.execute(
            "SELECT id, command, status FROM test_runs WHERE id = ?",
            (run_id,),
        ).fetchone()
    else:
        row = conn.execute(
            """
            SELECT id, command, status
            FROM test_runs
            WHERE command = ?
            ORDER BY id DESC
            LIMIT 1
            """,
            (command,),
        ).fetchone()
    if row is None:
        selector = f"id={run_id}" if run_id is not None else f"command={command!r}"
        raise SystemExit(f"known-failure gate: no test run found for {selector}")
    return row


def load_failures(conn: sqlite3.Connection, run_id: int) -> list[str]:
    rows = conn.execute(
        """
        SELECT DISTINCT nodeid
        FROM test_results
        WHERE run_id = ? AND status != 'passed'
        ORDER BY nodeid
        """,
        (run_id,),
    ).fetchall()
    return [row["nodeid"] for row in rows]


def classify(run_status: str, failures: list[str]) -> dict[str, object]:
    observed = sorted(set(failures))
    baseline = sorted(KNOWN_FAILURE_BASELINE)

    if run_status == "passed" and not observed:
        return {
            "mode": MODE_FULL_GREEN,
            "ok": True,
            "metric_eligible": True,
            "reason": "Full tools/run-tests.sh gate passed; metric sampling may proceed.",
            "failures": observed,
            "missing_baseline_failures": [],
            "unexpected_failures": [],
        }

    if observed == baseline:
        return {
            "mode": MODE_QUARANTINED,
            "ok": True,
            "metric_eligible": False,
            "reason": (
                "Observed failures exactly match the card 1808 known-failure "
                "baseline; no accepted PHPT metric progress may be recorded."
            ),
            "failures": observed,
            "missing_baseline_failures": [],
            "unexpected_failures": [],
        }

    baseline_set = set(baseline)
    observed_set = set(observed)
    missing = sorted(baseline_set - observed_set)
    unexpected = sorted(observed_set - baseline_set)
    reason_parts: list[str] = []
    if missing:
        reason_parts.append(
            "Baseline failures absent without baseline update: " + ", ".join(missing)
        )
    if unexpected:
        reason_parts.append("New or different failures: " + ", ".join(unexpected))
    if not reason_parts:
        reason_parts.append(
            f"Test run status is {run_status!r}, but no explicit failure set was recorded."
        )

    return {
        "mode": MODE_FAILED,
        "ok": False,
        "metric_eligible": False,
        "reason": "; ".join(reason_parts),
        "failures": observed,
        "missing_baseline_failures": missing,
        "unexpected_failures": unexpected,
    }


def write_metadata(conn: sqlite3.Connection, run: sqlite3.Row, result: dict[str, object]) -> None:
    conn.execute(
        """
        CREATE TABLE IF NOT EXISTS metadata (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )
        """
    )
    now = datetime.now(timezone.utc).isoformat()
    values = {
        "test_gate_mode": str(result["mode"]),
        "test_gate_reason": str(result["reason"]),
        "test_gate_run_id": str(run["id"]),
        "test_gate_command": str(run["command"]),
        "test_gate_failure_count": str(len(result["failures"])),
        "test_gate_failures_json": json.dumps(result["failures"], sort_keys=True),
        "test_gate_metric_eligible": "1" if result["metric_eligible"] else "0",
        "test_gate_known_failure_owner_card": KNOWN_FAILURE_OWNER_CARD,
        "test_gate_known_failures_json": json.dumps(sorted(KNOWN_FAILURE_BASELINE)),
        "test_gate_known_failure_removal_condition": KNOWN_FAILURE_REMOVAL_CONDITION,
    }
    for key, value in values.items():
        conn.execute(
            """
            INSERT INTO metadata(key, value, updated_at)
            VALUES (?, ?, ?)
            ON CONFLICT(key) DO UPDATE SET
                value = excluded.value,
                updated_at = excluded.updated_at
            """,
            (key, value, now),
        )
    conn.commit()


def main(argv: list[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    db_path = Path(args.db)
    with connect(str(db_path)) as conn:
        run = load_run(conn, args.run_id, args.command)
        result = classify(run["status"], load_failures(conn, run["id"]))
        result.update(
            {
                "run_id": run["id"],
                "command": run["command"],
                "run_status": run["status"],
                "known_failure_owner_card": KNOWN_FAILURE_OWNER_CARD,
                "known_failure_baseline": sorted(KNOWN_FAILURE_BASELINE),
                "known_failure_removal_condition": KNOWN_FAILURE_REMOVAL_CONDITION,
            }
        )
        if not args.no_write_metadata:
            write_metadata(conn, run, result)

    print(json.dumps(result, sort_keys=True))
    return 0 if result["ok"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
