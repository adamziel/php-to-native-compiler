#!/usr/bin/env python3
"""Fail-closed validation for run-full-corpus-crash-probe.sh artifacts."""

from __future__ import annotations

import argparse
import hashlib
import os
import re
import select
import shutil
import stat
import subprocess
import sys
import tempfile
import time
from dataclasses import dataclass
from pathlib import Path


MAX_COUNT = (1 << 63) - 1
MAX_BYTES = 8 * 1024 * 1024
MAX_ERRORS = 20
MAX_INVENTORY_ROWS = 100_000
MAX_INVENTORY_ENTRIES = 500_000
INVENTORY_TIMEOUT_SECONDS = 60
MAX_STATUS_RECORDS = 200
MAX_MANIFEST_RECORDS = MAX_INVENTORY_ROWS + 100
MAX_RECORD_BYTES = 16 * 1024
MAX_LOG_BYTES = 64 * 1024 * 1024
LOG_TAIL_BYTES = 128 * 1024
INTEGER = re.compile(r"(?:0|[1-9][0-9]*)\Z")
REVISION = re.compile(r"[0-9a-f]{40}(?:[0-9a-f]{24})?\Z")
SHA256 = re.compile(r"[0-9a-f]{64}\Z")
PATROL_SUMMARY = re.compile(
    r"^\[ptn-patrol\] tests=(?P<tests>0|[1-9][0-9]*) "
    r"passed=(?P<passed>0|[1-9][0-9]*) "
    r"failed=(?P<failed>0|[1-9][0-9]*) "
    r"skipped=(?P<skipped>0|[1-9][0-9]*) "
    r"warned=(?P<warned>0|[1-9][0-9]*)(?: .*)?$"
)


class Errors:
    def __init__(self, limit: int) -> None:
        self.limit, self.items, self.more = limit, [], 0

    def add(self, message: str) -> None:
        if len(self.items) < self.limit:
            self.items.append(message)
        else:
            self.more += 1

    def output(self) -> list[str]:
        return self.items + ([f"{self.more} additional errors suppressed"] if self.more else [])


@dataclass(frozen=True)
class Manifest:
    corpus: str
    revision: str
    total: int
    index: int
    count: int
    rows: tuple[str, ...]


def short(value: object) -> str:
    text = "".join(
        char if char.isprintable() else f"\\x{ord(char):02x}"
        for char in str(value)
    )
    return text if len(text) <= 220 else f"{text[:217]}..."


def read_lines(path: Path, errors: Errors, record_limit: int) -> list[str] | None:
    try:
        info = path.stat()
        if path.is_symlink() or not stat.S_ISREG(info.st_mode) or info.st_size > MAX_BYTES:
            raise OSError("not a bounded regular file")
        lines: list[str] = []
        with path.open("rb") as source:
            for line_no, raw in enumerate(source, 1):
                if line_no > record_limit:
                    raise OSError("too many records")
                if len(raw) > MAX_RECORD_BYTES:
                    raise OSError("oversized record")
                line = raw.decode("utf-8")
                if line.endswith("\n"):
                    line = line[:-1]
                if line.endswith("\r"):
                    line = line[:-1]
                lines.append(line)
        return lines
    except OSError as exc:
        errors.add(f"cannot read {short(path)}: {short(exc)}")
        return None
    except UnicodeDecodeError as exc:
        errors.add(f"cannot read {short(path)}: {exc.__class__.__name__}")
        return None


def number(value: str | None, context: str, errors: Errors) -> int | None:
    if value is None or not INTEGER.fullmatch(value):
        errors.add(f"malformed integer for {context}")
        return None
    result = int(value)
    if result > MAX_COUNT:
        errors.add(f"integer overflow for {context}")
        return None
    return result


def tsv(path: Path, errors: Errors) -> dict[str, list[str]] | None:
    lines = read_lines(path, errors, MAX_STATUS_RECORDS)
    if lines is None:
        return None
    result: dict[str, list[str]] = {}
    for line_no, line in enumerate(lines, 1):
        if line.count("\t") != 1:
            errors.add(f"malformed TSV at {short(path)} line {line_no}")
            continue
        key, value = line.split("\t", 1)
        if not key or not value or key.strip() != key:
            errors.add(f"malformed TSV field at {short(path)} line {line_no}")
            continue
        result.setdefault(key, []).append(value)
    return result


def run_log_metrics(path: Path, expected_digest: str, errors: Errors) -> dict[str, int] | None:
    try:
        info = path.stat()
        if path.is_symlink() or not stat.S_ISREG(info.st_mode) or info.st_size > MAX_LOG_BYTES:
            raise OSError("not a bounded regular log")
        digest = hashlib.sha256()
        with path.open("rb") as source:
            while chunk := source.read(64 * 1024):
                digest.update(chunk)
        if digest.hexdigest() != expected_digest:
            errors.add(f"run log digest does not match status: {short(path)}")
            return None
        with path.open("rb") as source:
            source.seek(max(0, info.st_size - LOG_TAIL_BYTES))
            tail = source.read(LOG_TAIL_BYTES)
        for raw_line in reversed(tail.rsplit(b"\n", 300)):
            line = raw_line.decode("utf-8", errors="replace").strip()
            match = PATROL_SUMMARY.fullmatch(line)
            if match is not None:
                parsed = {
                    key: number(value, f"run log {path.name} {key}", errors)
                    for key, value in match.groupdict().items()
                }
                return parsed if all(value is not None for value in parsed.values()) else None
        errors.add(f"run log has no final parseable PHPT summary: {short(path)}")
        return None
    except OSError as exc:
        errors.add(f"cannot read run log {short(path)}: {short(exc)}")
        return None


def one(values: dict[str, list[str]], key: str, context: str, errors: Errors) -> str | None:
    result = values.get(key, [])
    if len(result) != 1:
        errors.add(f"{context} must contain exactly one {key}")
        return None
    return result[0]


def manifest(path: Path, errors: Errors) -> Manifest | None:
    lines = read_lines(path, errors, MAX_MANIFEST_RECORDS)
    if lines is None:
        return None
    wanted = {"corpus", "corpus-revision", "total-rows", "shard-index", "shard-count"}
    headers: dict[str, str] = {}
    rows: list[str] = []
    generated = False
    for line_no, line in enumerate(lines, 1):
        if not line.strip():
            continue
        if line == "# Generated by tools/phpt-full-corpus-shard.sh":
            generated = True
            continue
        if line.startswith("# "):
            key, marker, value = line[2:].partition(": ")
            if not marker or key not in wanted or not value or key in headers:
                errors.add(f"malformed manifest header at {short(path)} line {line_no}")
            else:
                headers[key] = value
            continue
        row = Path(line)
        if (
            line[0].isspace()
            or line != line.strip()
            or not line.endswith(".phpt")
            or row.is_absolute()
            or "." in row.parts
            or ".." in row.parts
        ):
            errors.add(f"malformed manifest row at {short(path)} line {line_no}")
        else:
            rows.append(line)
            if len(rows) > MAX_INVENTORY_ROWS:
                errors.add(f"manifest rows exceed bounded limit in {short(path)}")
                return None
    missing = wanted - headers.keys()
    if not generated:
        errors.add(f"missing generator header in {short(path)}")
    for key in sorted(missing):
        errors.add(f"missing {key} header in {short(path)}")
    if missing or not generated:
        return None
    total = number(headers["total-rows"], f"{path.name} total-rows", errors)
    index = number(headers["shard-index"], f"{path.name} shard-index", errors)
    count = number(headers["shard-count"], f"{path.name} shard-count", errors)
    if total is None or index is None or count is None:
        return None
    if not Path(headers["corpus"]).is_absolute() or not REVISION.fullmatch(headers["corpus-revision"]):
        errors.add(f"invalid corpus header in {short(path)}")
        return None
    if count == 0 or count > 10_000 or index >= count or len(rows) != len(set(rows)):
        errors.add(f"invalid shard metadata or duplicate rows in {short(path)}")
        return None
    return Manifest(headers["corpus"], headers["corpus-revision"], total, index, count, tuple(rows))


def git_head(corpus: Path, errors: Errors) -> str | None:
    try:
        result = subprocess.run(
            ["git", "-C", str(corpus), "rev-parse", "--verify", "HEAD"],
            stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True, timeout=30,
        )
    except (OSError, subprocess.TimeoutExpired) as exc:
        errors.add(f"cannot read corpus revision: {exc.__class__.__name__}")
        return None
    revision = result.stdout.strip()
    if result.returncode or not REVISION.fullmatch(revision):
        errors.add(f"corpus is not a readable Git checkout: {short(corpus)}")
        return None
    return revision


def git_clean(corpus: Path, errors: Errors) -> bool:
    try:
        process = subprocess.Popen(
            ["git", "-C", str(corpus), "status", "--porcelain=v1", "--untracked-files=all", "--ignore-submodules=none", "-z"],
            stdout=subprocess.PIPE, stderr=subprocess.DEVNULL,
        )
        assert process.stdout is not None
        ready, _, _ = select.select([process.stdout], [], [], INVENTORY_TIMEOUT_SECONDS)
        if not ready:
            process.kill()
            process.wait(timeout=10)
            errors.add("corpus cleanliness check timed out")
            return False
        dirty = bool(process.stdout.read(1))
        process.stdout.close()
        if dirty:
            process.terminate()
        code = process.wait(timeout=60)
    except subprocess.TimeoutExpired as exc:
        process.kill()
        process.wait(timeout=10)
        errors.add(f"cannot inspect corpus cleanliness: {exc.__class__.__name__}")
        return False
    except OSError as exc:
        errors.add(f"cannot inspect corpus cleanliness: {exc.__class__.__name__}")
        return False
    if dirty:
        errors.add(f"corpus checkout is dirty: {short(corpus)}")
    elif code:
        errors.add(f"cannot inspect corpus cleanliness: {short(corpus)}")
    return not dirty and code == 0


def inventory(corpus: Path, errors: Errors) -> list[str] | None:
    if corpus.is_symlink() or not corpus.is_dir():
        errors.add(f"corpus path is not a real directory: {short(corpus)}")
        return None
    rows: list[str] = []
    entries = 0
    deadline = time.monotonic() + INVENTORY_TIMEOUT_SECONDS
    pending = [corpus]
    try:
        while pending:
            if time.monotonic() > deadline:
                errors.add("corpus inventory exceeds bounded traversal limits")
                return None
            directory = pending.pop()
            with os.scandir(directory) as items:
                for item in items:
                    entries += 1
                    if entries > MAX_INVENTORY_ENTRIES or time.monotonic() > deadline:
                        errors.add("corpus inventory exceeds bounded traversal limits")
                        return None
                    if item.name == ".git" and item.is_dir(follow_symlinks=False):
                        continue
                    if item.is_dir(follow_symlinks=False):
                        pending.append(Path(item.path))
                    elif item.name.endswith(".phpt") and item.is_file(follow_symlinks=False):
                        rows.append(Path(item.path).relative_to(corpus).as_posix())
                        if len(rows) > MAX_INVENTORY_ROWS:
                            errors.add("corpus inventory exceeds bounded row limit")
                            return None
    except OSError as exc:
        errors.add(f"cannot enumerate corpus: {exc.__class__.__name__}")
        return None
    rows.sort()
    if not rows or len(rows) != len(set(rows)):
        errors.add("invalid corpus inventory")
        return None
    return rows


def metrics(status: dict[str, list[str]], context: str, errors: Errors) -> dict[str, int] | None:
    result: dict[str, int] = {}
    for key in ("selected", "tests", "passed", "failed", "skipped", "warned"):
        parsed = number(one(status, key, context, errors), f"{context} {key}", errors)
        if parsed is not None:
            result[key] = parsed
    return result if len(result) == 6 else None


def verify(
    run_dir: Path,
    limit: int,
    expected_source_commit: str,
    expected_revision: str,
    expected_count: int,
) -> tuple[Errors, int, int]:
    errors = Errors(limit)
    if run_dir.is_symlink() or not run_dir.is_dir():
        errors.add(f"run directory is not a real directory: {short(run_dir)}")
        return errors, 0, 0
    root = tsv(run_dir / "status.tsv", errors)
    if root is None:
        return errors, 0, 0
    if one(root, "source_commit", "run status", errors) != expected_source_commit:
        errors.add("run status source commit does not match expected compiler source")
    if one(root, "corpus_revision", "run status", errors) != expected_revision:
        errors.add("run status corpus revision does not match expected revision")
    if number(one(root, "corpus_count", "run status", errors), "run status corpus_count", errors) != expected_count:
        errors.add("run status corpus count does not match expected count")
    shards = number(one(root, "shards", "run status", errors), "run status shards", errors)
    if one(root, "state", "run status", errors) != "finished":
        errors.add("run status is not terminal finished")
    if shards is None or shards == 0 or shards > 10_000:
        if shards is not None:
            errors.add("run status shards is outside supported range")
        return errors, 0, 0
    if shards > expected_count:
        errors.add("run status shard count exceeds expected corpus row count")
        return errors, shards, 0
    shard_root = run_dir / "shards"
    if shard_root.is_symlink() or not shard_root.is_dir():
        errors.add(f"missing shards directory: {short(shard_root)}")
        return errors, 0, 0
    expected = {f"shard-{index:02d}" for index in range(shards)}
    try:
        actual: set[str] = set()
        with os.scandir(shard_root) as items:
            for item in items:
                if len(actual) >= shards + MAX_ERRORS:
                    errors.add("shards directory exceeds bounded entry limit")
                    return errors, shards, 0
                actual.add(item.name)
    except OSError as exc:
        errors.add(f"cannot inspect shards directory: {exc.__class__.__name__}")
        return errors, 0, 0
    for name in sorted(expected - actual):
        errors.add(f"missing shard directory: {name}")
    for name in sorted(actual - expected):
        errors.add(f"unexpected shard directory: {short(name)}")

    manifests: list[Manifest] = []
    row_owner: dict[str, int] = {}
    selected = 0
    for index in range(shards):
        directory = shard_root / f"shard-{index:02d}"
        if directory.is_symlink():
            errors.add(f"shard directory is a symlink: {directory.name}")
            continue
        item = manifest(directory / "manifest.txt", errors)
        status = tsv(directory / "status.tsv", errors)
        if item is not None:
            if len(item.rows) > expected_count - selected:
                errors.add(f"manifest rows exceed expected corpus count at shard {index}")
                continue
            manifests.append(item)
            selected += len(item.rows)
            if item.index != index or item.count != shards:
                errors.add(f"manifest shard metadata differs for shard {index}")
            for row in item.rows:
                if row in row_owner:
                    errors.add(f"row appears in more than one manifest: {short(row)}")
                row_owner[row] = index
        if status is None:
            continue
        label = f"shard {index} status"
        shard = number(one(status, "shard", label, errors), f"{label} shard", errors)
        exit_code = number(one(status, "exit", label, errors), f"{label} exit", errors)
        log_digest = one(status, "log_sha256", label, errors)
        if shard != index:
            errors.add(f"{label} has wrong shard index")
        if status.get("state") != ["running", "passed"]:
            errors.add(f"{label} must transition exactly from running to passed")
        if exit_code != 0:
            errors.add(f"{label} exit is not zero")
        values = metrics(status, label, errors)
        if log_digest is None or not SHA256.fullmatch(log_digest):
            errors.add(f"{label} has no valid run log digest")
        else:
            log_values = run_log_metrics(directory / "run.log", log_digest, errors)
            if values is not None and log_values is not None:
                for key in ("tests", "passed", "failed", "skipped", "warned"):
                    if values[key] != log_values[key]:
                        errors.add(f"{label} run log {key} differs from status")
        if values is not None and item is not None:
            count = len(item.rows)
            if values["selected"] != count:
                errors.add(f"{label} selected does not equal manifest rows")
            if values["tests"] != count or values["passed"] != count:
                errors.add(f"{label} tests/passed do not equal selected")
            if any(values[key] for key in ("failed", "skipped", "warned")):
                errors.add(f"{label} has nonzero failed/skipped/warned metrics")

    if len(manifests) != shards:
        return errors, shards, selected
    first = manifests[0]
    if any((item.corpus, item.revision, item.total) != (first.corpus, first.revision, first.total) for item in manifests):
        errors.add("manifest headers disagree about corpus path, revision, or total rows")
        return errors, shards, selected
    if first.revision != expected_revision:
        errors.add("manifest corpus revision does not match expected revision")
    if first.total != expected_count:
        errors.add("manifest total-rows does not match expected corpus count")
    corpus = Path(first.corpus)
    run_tests = corpus / "run-tests.php"
    if run_tests.is_symlink() or not run_tests.is_file():
        errors.add("recorded corpus is not a php-src checkout with run-tests.php")
        return errors, shards, selected
    head = git_head(corpus, errors)
    if head is not None and (head != first.revision or head != expected_revision):
        errors.add("recorded corpus revision does not match expected checkout HEAD")
    if head is not None:
        git_clean(corpus, errors)
    rows = inventory(corpus, errors)
    if rows is None:
        return errors, shards, selected
    if first.total != len(rows) or expected_count != len(rows):
        errors.add("manifest total-rows does not equal clean corpus inventory")
    if set(row_owner) != set(rows):
        errors.add(f"manifest union differs from corpus inventory (missing={len(set(rows) - set(row_owner))} extra={len(set(row_owner) - set(rows))})")
    for position, row in enumerate(rows):
        if row_owner.get(row) != position % shards:
            errors.add(f"manifest partition is not deterministic at inventory row {position}")
            break
    if selected != len(rows):
        errors.add("sum of manifest rows does not equal clean corpus inventory")
    return errors, shards, selected


def git(args: list[str], cwd: Path) -> str:
    return subprocess.run(["git", "-C", str(cwd), *args], check=True, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True).stdout.strip()


def fixture(run_dir: Path, corpus: Path, revision: str) -> None:
    rows = ["Zend/tests/a.phpt", "ext/soap/tests/b.phpt", "tests/c.phpt"]
    (run_dir / "shards").mkdir(parents=True)
    (run_dir / "status.tsv").write_text(
        "\n".join([
            "state\tfinished", f"source_commit\t{revision}", f"corpus_revision\t{revision}",
            "corpus_count\t3", "shards\t2", "",
        ]),
        encoding="utf-8",
    )
    for index in range(2):
        selected = [row for position, row in enumerate(rows) if position % 2 == index]
        directory = run_dir / "shards" / f"shard-{index:02d}"
        directory.mkdir()
        (directory / "manifest.txt").write_text("\n".join([
            "# Generated by tools/phpt-full-corpus-shard.sh", f"# corpus: {corpus}",
            f"# corpus-revision: {revision}", "# total-rows: 3", f"# shard-index: {index}",
            "# shard-count: 2", "", *selected, "",
        ]), encoding="utf-8")
        count = len(selected)
        log = directory / "run.log"
        log.write_text(
            f"[ptn-patrol] tests={count} passed={count} failed=0 skipped=0 warned=0 run_tests_time=0s\n",
            encoding="utf-8",
        )
        digest = hashlib.sha256(log.read_bytes()).hexdigest()
        (directory / "status.tsv").write_text("\n".join([
            "state\trunning", f"shard\t{index}", f"selected\t{count}", "exit\t0", "state\tpassed",
            f"tests\t{count}", f"passed\t{count}", "failed\t0", "skipped\t0", "warned\t0",
            f"log_sha256\t{digest}", "",
        ]), encoding="utf-8")


def self_test() -> int:
    if shutil.which("git") is None:
        print("self-test failed: git is required", file=sys.stderr)
        return 1
    with tempfile.TemporaryDirectory(prefix="verify-full-phpt-run-") as temp:
        root, corpus = Path(temp), Path(temp) / "corpus"
        for row in ("Zend/tests/a.phpt", "ext/soap/tests/b.phpt", "tests/c.phpt"):
            path = corpus / row
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("--TEST--\nfixture\n", encoding="utf-8")
        (corpus / "run-tests.php").write_text("<?php\n", encoding="utf-8")
        git(["init", "-q"], corpus)
        git(["add", "."], corpus)
        git(["-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid", "commit", "-qm", "fixture"], corpus)
        revision = git(["rev-parse", "HEAD"], corpus)
        run = root / "run"

        def reset() -> None:
            shutil.rmtree(run, ignore_errors=True)
            fixture(run, corpus, revision)

        def check(
            expected: str,
            expected_source_commit: str = revision,
            expected_revision: str = revision,
            expected_count: int = 3,
        ) -> bool:
            return any(
                expected in item
                for item in verify(
                    run,
                    MAX_ERRORS,
                    expected_source_commit,
                    expected_revision,
                    expected_count,
                )[0].items
            )

        reset()
        initial_errors = verify(run, MAX_ERRORS, revision, revision, 3)[0].items
        if initial_errors:
            print(f"self-test failed: valid fixture rejected: {'; '.join(initial_errors)}", file=sys.stderr)
            return 1
        (corpus / "tests/c.phpt").write_text("--TEST--\ndirty\n", encoding="utf-8")
        if not check("dirty"):
            print("self-test failed: dirty corpus accepted", file=sys.stderr)
            return 1
        git(["checkout", "--", "tests/c.phpt"], corpus)
        reset()
        if not check("expected revision", expected_revision="0" * 40):
            print("self-test failed: provenance mismatch accepted", file=sys.stderr)
            return 1
        reset()
        if not check("expected corpus count", expected_count=4):
            print("self-test failed: expected count mismatch accepted", file=sys.stderr)
            return 1
        reset()
        (corpus / "run-tests.php").unlink()
        if not check("run-tests.php"):
            print("self-test failed: non-php-src corpus accepted", file=sys.stderr)
            return 1
        git(["checkout", "--", "run-tests.php"], corpus)
        reset()
        global MAX_INVENTORY_ENTRIES
        prior_inventory_limit = MAX_INVENTORY_ENTRIES
        MAX_INVENTORY_ENTRIES = 1
        try:
            if not check("bounded traversal"):
                print("self-test failed: traversal limit accepted", file=sys.stderr)
                return 1
        finally:
            MAX_INVENTORY_ENTRIES = prior_inventory_limit
        reset()
        global MAX_INVENTORY_ROWS
        prior_row_limit = MAX_INVENTORY_ROWS
        MAX_INVENTORY_ROWS = 1
        try:
            if not check("manifest rows exceed bounded limit"):
                print("self-test failed: oversized manifest accepted", file=sys.stderr)
                return 1
        finally:
            MAX_INVENTORY_ROWS = prior_row_limit
        reset()
        global MAX_MANIFEST_RECORDS
        prior_manifest_limit = MAX_MANIFEST_RECORDS
        MAX_MANIFEST_RECORDS = 1
        try:
            if not check("too many records"):
                print("self-test failed: manifest record limit accepted", file=sys.stderr)
                return 1
        finally:
            MAX_MANIFEST_RECORDS = prior_manifest_limit
        reset()
        status = run / "shards/shard-00/status.tsv"
        status.write_text(
            status.read_text(encoding="utf-8").replace("state\trunning", "state\tfailed"),
            encoding="utf-8",
        )
        if not check("transition exactly"):
            print("self-test failed: invalid state transition accepted", file=sys.stderr)
            return 1
        reset()
        log = run / "shards/shard-00/run.log"
        log.write_text("tampered\n", encoding="utf-8")
        if not check("digest"):
            print("self-test failed: tampered run log accepted", file=sys.stderr)
            return 1
        reset()
        log = run / "shards/shard-00/run.log"
        log.write_text(
            "[ptn-patrol] tests=0 passed=0 failed=0 skipped=0 warned=0 run_tests_time=0s\n",
            encoding="utf-8",
        )
        status = run / "shards/shard-00/status.tsv"
        status.write_text(
            re.sub(
                r"log_sha256\t[0-9a-f]{64}",
                f"log_sha256\t{hashlib.sha256(log.read_bytes()).hexdigest()}",
                status.read_text(encoding="utf-8"),
            ),
            encoding="utf-8",
        )
        if not check("run log tests differs"):
            print("self-test failed: mismatched run log metrics accepted", file=sys.stderr)
            return 1
        reset()
        duplicate_manifest = run / "shards/shard-01/manifest.txt"
        duplicate_manifest.write_text(
            duplicate_manifest.read_text(encoding="utf-8").replace(
                "ext/soap/tests/b.phpt", "Zend/tests/a.phpt"
            ),
            encoding="utf-8",
        )
        if not check("more than one manifest"):
            print("self-test failed: duplicate manifest row accepted", file=sys.stderr)
            return 1
        reset()
        truncated_manifest = run / "shards/shard-00/manifest.txt"
        truncated_manifest.write_text(
            truncated_manifest.read_text(encoding="utf-8").replace("Zend/tests/a.phpt\n", ""),
            encoding="utf-8",
        )
        if not check("selected does not equal"):
            print("self-test failed: truncated manifest accepted", file=sys.stderr)
            return 1
        reset()
        status = run / "shards/shard-00/status.tsv"
        for old, new, expected in (("passed\t2", "passed\t3", "tests/passed"), ("passed\t3", f"passed\t{MAX_COUNT + 1}", "overflow"), (f"passed\t{MAX_COUNT + 1}", "passed\t02", "malformed integer")):
            status.write_text(status.read_text(encoding="utf-8").replace(old, new), encoding="utf-8")
            if not check(expected):
                print(f"self-test failed: {expected} was accepted", file=sys.stderr)
                return 1
        hostile = root / "bad\nline-1\nline-2"
        command = [
            sys.executable,
            str(Path(__file__)),
            "--expected-revision",
            revision,
            "--expected-source-commit",
            revision,
            "--expected-count",
            "3",
            str(hostile),
        ]
        result = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, timeout=20)
        if result.returncode == 0 or len(result.stderr.splitlines()) != 1 or "\\x0a" not in result.stderr:
            print("self-test failed: hostile path output was not bounded", file=sys.stderr)
            return 1
        result = subprocess.run(
            [sys.executable, str(Path(__file__)), "--max-errors", "x" * 1000, "--self-test"],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            timeout=20,
        )
        if result.returncode != 2 or len(result.stderr) > 300 or len(result.stderr.splitlines()) != 1:
            print("self-test failed: parser error output was not bounded", file=sys.stderr)
            return 1
    print("self-test: ok")
    return 0


class BoundedParser(argparse.ArgumentParser):
    def error(self, message: str) -> None:
        self.exit(2, f"{self.prog}: {short(message)}\n")


def main() -> int:
    parser = BoundedParser(prog="verify-full-phpt-run", description=__doc__, add_help=True)
    parser.add_argument("run_dir", nargs="?", type=Path)
    parser.add_argument("--self-test", action="store_true")
    parser.add_argument("--expected-source-commit")
    parser.add_argument("--expected-revision")
    parser.add_argument("--expected-count")
    parser.add_argument("--max-errors", default=str(MAX_ERRORS))
    args = parser.parse_args()
    parser_errors = Errors(1)
    max_errors = number(args.max_errors, "max-errors", parser_errors)
    if max_errors is None or max_errors < 1 or max_errors > MAX_ERRORS:
        parser.error(f"--max-errors must be between 1 and {MAX_ERRORS}")
    if args.self_test:
        if (
            args.run_dir is not None
            or args.expected_source_commit is not None
            or args.expected_revision is not None
            or args.expected_count is not None
        ):
            parser.error("--self-test cannot be combined with RUN_DIR or expected corpus arguments")
        return self_test()
    if (
        args.run_dir is None
        or args.expected_source_commit is None
        or args.expected_revision is None
        or args.expected_count is None
    ):
        parser.error("RUN_DIR and expected source/corpus revision/count arguments are required")
    if not REVISION.fullmatch(args.expected_source_commit):
        parser.error("--expected-source-commit must be a full hexadecimal Git revision")
    if not REVISION.fullmatch(args.expected_revision):
        parser.error("--expected-revision must be a full hexadecimal Git revision")
    expected_count = number(args.expected_count, "expected-count", parser_errors)
    if expected_count is None or expected_count < 1 or expected_count > MAX_INVENTORY_ROWS:
        parser.error(f"--expected-count must be between 1 and {MAX_INVENTORY_ROWS}")
    errors, shards, selected = verify(
        args.run_dir,
        max_errors,
        args.expected_source_commit,
        args.expected_revision,
        expected_count,
    )
    if errors.items:
        for message in errors.output():
            print(f"verify-full-phpt-run: {message}", file=sys.stderr)
        return 1
    print(f"full-phpt-run-ok: shards={shards} selected={selected} run_dir={short(args.run_dir)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
