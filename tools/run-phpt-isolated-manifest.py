#!/usr/bin/env python3
"""Run every PHPT manifest row in an isolated run-tests.php process.

This runner is meant for full-corpus scoring where completeness matters more
than stopping on the first crash. Rows are run in small batches for throughput.
If a batch times out, exits by signal, or does not produce a complete run-tests
summary, the batch is split and retried until the crashing unit is reduced to a
single PHPT row. A single-row timeout/crash is recorded as a failed row, then the
runner continues.
"""

from __future__ import annotations

import argparse
import concurrent.futures
import dataclasses
import datetime as dt
import os
import re
import signal
import subprocess
import sys
import time
from pathlib import Path


COUNT_RE = {
    "tests": re.compile(r"^\s*Number of tests\s*:\s*(\d+)\b", re.MULTILINE),
    "skipped": re.compile(r"^\s*Tests skipped\s*:\s*(\d+)\b", re.MULTILINE),
    "warned": re.compile(r"^\s*Tests warned\s*:\s*(\d+)\b", re.MULTILINE),
    "failed": re.compile(r"^\s*Tests failed\s*:\s*(\d+)\b", re.MULTILINE),
    "passed": re.compile(r"^\s*Tests passed\s*:\s*(\d+)\b", re.MULTILINE),
}

PROCESS_TIMEOUT_GRACE_SECONDS = 60


@dataclasses.dataclass(frozen=True)
class Row:
    index: int
    rel: str
    path: Path


@dataclasses.dataclass(frozen=True)
class Result:
    index: int
    rows: int
    first_rel: str
    last_rel: str
    state: str
    exit_code: int | str
    elapsed_ms: int
    tests: int
    passed: int
    failed: int
    skipped: int
    warned: int
    timed_out: int
    crashed: int


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("manifest", type=Path)
    parser.add_argument("--out-dir", type=Path, required=True)
    parser.add_argument("--timeout", type=int, default=120)
    parser.add_argument("--workers", type=int, default=2)
    parser.add_argument("--batch-size", type=int, default=24)
    parser.add_argument("--php-src", type=Path, default=None)
    parser.add_argument("--phpc-bin", type=Path, default=None)
    return parser.parse_args()


def repo_root() -> Path:
    return Path(__file__).resolve().parents[1]


def resolve_php_src(arg: Path | None, root: Path) -> Path:
    candidates: list[Path] = []
    if arg is not None:
        candidates.append(arg)
    if os.environ.get("PHP_SRC_PHPT"):
        candidates.append(Path(os.environ["PHP_SRC_PHPT"]))
    candidates.extend(
        [
            root / ".runtime/php-src-phpt",
            Path("/home/claude/php-src-phpt"),
        ]
    )
    for candidate in candidates:
        if (candidate / "run-tests.php").is_file():
            return candidate.resolve()
    raise SystemExit("could not resolve php-src PHPT corpus; set PHP_SRC_PHPT")


def git_output(args: list[str], cwd: Path) -> str:
    try:
        return subprocess.run(
            ["git", "-C", str(cwd), *args],
            check=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
            text=True,
            timeout=30,
        ).stdout.strip()
    except (OSError, subprocess.SubprocessError):
        return ""


def trim_manifest_row(line: str) -> str:
    return line.split("#", 1)[0].strip()


def load_rows(manifest: Path, php_src: Path) -> list[Row]:
    rows: list[Row] = []
    for raw in manifest.read_text(encoding="utf-8").splitlines():
        value = trim_manifest_row(raw)
        if not value:
            continue
        path = Path(value)
        if path.is_absolute():
            abs_path = path
            try:
                rel = abs_path.resolve().relative_to(php_src).as_posix()
            except ValueError:
                rel = abs_path.as_posix()
        else:
            rel = value
            abs_path = php_src / value
        if not abs_path.is_file():
            raise SystemExit(f"PHPT row not found: {abs_path}")
        rows.append(Row(len(rows), rel, abs_path))
    if not rows:
        raise SystemExit(f"manifest contains no PHPT rows: {manifest}")
    return rows


def parse_counts(output: str) -> dict[str, int] | None:
    values: dict[str, int] = {}
    for key, pattern in COUNT_RE.items():
        match = pattern.search(output)
        if match is None:
            return None
        values[key] = int(match.group(1))
    return values


def classify_result(
    returncode: int,
    timed_out: bool,
    counts: dict[str, int] | None,
    row_count: int,
) -> str:
    if timed_out:
        return "timeout"
    if returncode < 0:
        return f"signal-{abs(returncode)}"
    if counts is None:
        return "no-summary"
    if counts["tests"] != row_count:
        return "partial-summary"
    if counts["failed"]:
        return "failed"
    if counts["warned"]:
        return "warned"
    if counts["skipped"]:
        return "skipped"
    if counts["passed"]:
        return "passed"
    return "unknown"


def batch_process_timeout(test_timeout: int, row_count: int) -> int:
    return test_timeout * row_count + PROCESS_TIMEOUT_GRACE_SECONDS


def run_batch_once(rows: list[Row], php_src: Path, phpc_bin: Path, timeout: int) -> Result:
    env = os.environ.copy()
    env["PHPC_BIN"] = str(phpc_bin)
    env["TEST_PHP_EXECUTABLE"] = str(phpc_bin)
    env["TEST_PHP_CGI_EXECUTABLE"] = str(phpc_bin)
    env["TEST_PHP_CGI_EXECUTABLE_ESCAPED"] = f"'{phpc_bin}'"
    command = [
        "php",
        str(php_src / "run-tests.php"),
        "-q",
        "--set-timeout",
        str(timeout),
        "-p",
        str(phpc_bin),
        *[str(row.path) for row in rows],
    ]

    start = time.monotonic()
    timed_out = False
    try:
        process = subprocess.Popen(
            command,
            cwd=php_src,
            env=env,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            start_new_session=True,
        )
        try:
            raw_output, _ = process.communicate(
                timeout=batch_process_timeout(timeout, len(rows))
            )
        except subprocess.TimeoutExpired:
            timed_out = True
            os.killpg(process.pid, signal.SIGTERM)
            try:
                raw_output, _ = process.communicate(timeout=15)
            except subprocess.TimeoutExpired:
                os.killpg(process.pid, signal.SIGKILL)
                raw_output, _ = process.communicate()
        returncode = process.returncode
    except OSError:
        raw_output = b""
        returncode = 127

    elapsed_ms = int((time.monotonic() - start) * 1000)
    output = raw_output.decode("utf-8", errors="replace")
    counts = parse_counts(output)
    state = classify_result(returncode, timed_out, counts, len(rows))
    if counts is None or counts["tests"] != len(rows):
        counts = {
            "tests": len(rows),
            "passed": 0,
            "failed": len(rows),
            "skipped": 0,
            "warned": 0,
        }
    crashed = int(timed_out or returncode < 0 or state in {"no-summary", "unknown"})
    if state == "partial-summary":
        crashed = 1
    exit_value: int | str = "timeout" if timed_out else returncode
    return Result(
        index=rows[0].index,
        rows=len(rows),
        first_rel=rows[0].rel,
        last_rel=rows[-1].rel,
        state=state,
        exit_code=exit_value,
        elapsed_ms=elapsed_ms,
        tests=counts["tests"],
        passed=counts["passed"],
        failed=counts["failed"],
        skipped=counts["skipped"],
        warned=counts["warned"],
        timed_out=int(timed_out),
        crashed=crashed,
    )


def run_batch(rows: list[Row], php_src: Path, phpc_bin: Path, timeout: int) -> list[Result]:
    result = run_batch_once(rows, php_src, phpc_bin, timeout)
    incomplete = result.state in {
        "timeout",
        "no-summary",
        "partial-summary",
        "unknown",
    } or result.state.startswith("signal-")
    if incomplete and len(rows) > 1:
        midpoint = len(rows) // 2
        return (
            run_batch(rows[:midpoint], php_src, phpc_bin, timeout) +
            run_batch(rows[midpoint:], php_src, phpc_bin, timeout)
        )
    return [result]


def chunk_rows(rows: list[Row], size: int) -> list[list[Row]]:
    return [rows[index:index + size] for index in range(0, len(rows), size)]


def timestamp() -> str:
    return dt.datetime.now(dt.UTC).strftime("%Y%m%dT%H%M%SZ")


def atomic_write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    temp = path.with_name(f".{path.name}.{os.getpid()}.tmp")
    temp.write_text(text, encoding="utf-8")
    temp.replace(path)


def write_results(path: Path, results: list[Result]) -> None:
    lines = [
        "index\trows\tfirst_row\tlast_row\tstate\texit_code\telapsed_ms\ttests\tpassed\tfailed\t"
        "skipped\twarned\ttimed_out\tcrashed"
    ]
    for item in sorted(results, key=lambda result: result.index):
        lines.append(
            f"{item.index}\t{item.rows}\t{item.first_rel}\t{item.last_rel}\t"
            f"{item.state}\t{item.exit_code}\t"
            f"{item.elapsed_ms}\t{item.tests}\t{item.passed}\t{item.failed}\t"
            f"{item.skipped}\t{item.warned}\t{item.timed_out}\t{item.crashed}"
        )
    atomic_write_text(path, "\n".join(lines) + "\n")


def write_summary(
    path: Path,
    stamp: str,
    root: Path,
    php_src: Path,
    manifest: Path,
    row_results: Path,
    results: list[Result],
    elapsed: int,
    workers: int,
    batch_size: int,
    timeout_seconds: int,
    run_tests_exit: int | None,
) -> None:
    commit = git_output(["rev-parse", "--short=12", "HEAD"], root)
    corpus_revision = git_output(["rev-parse", "HEAD"], php_src)
    selected = sum(item.rows for item in results)
    tests = sum(item.tests for item in results)
    passed = sum(item.passed for item in results)
    failed = sum(item.failed for item in results)
    skipped = sum(item.skipped for item in results)
    warned = sum(item.warned for item in results)
    timed_out = sum(item.timed_out for item in results)
    crashed = sum(item.crashed for item in results)
    lines = [
        f"PHPT isolated full-corpus shard {stamp}",
        f"commit: {commit}",
        f"corpus: {php_src}",
        f"corpus-revision: {corpus_revision}",
        f"manifest: {manifest}",
        f"runnable-manifest: {manifest}",
        (
            f"command: tools/run-phpt-isolated-manifest.py --timeout {timeout_seconds} "
            f"--workers {workers} --batch-size {batch_size} --out-dir {path.parent} {manifest}"
        ),
        f"timeout-seconds: {timeout_seconds}",
        f"row-workers: {workers}",
        f"batch-size: {batch_size}",
        f"checkpoint-complete: {'yes' if run_tests_exit is not None else 'no'}",
        f"count: {selected} selected PHPT rows; {selected} runnable; 0 excluded by classification in 1 buckets",
        "classification: enabled=0 selected={0} runnable={0} excluded=0".format(selected),
        f"classification-files: all={manifest} runnable={manifest} classification= excluded=",
        (
            f"bucket: manifest selected={selected} runnable={selected} tests={tests} "
            f"passed={passed} failed={failed} skipped={skipped} warned={warned} "
            f"elapsed={elapsed}s run-tests-exit={run_tests_exit if run_tests_exit is not None else 'incomplete'} "
            f"log={row_results}"
        ),
        (
            f"result: buckets=1 selected={selected} runnable={selected} excluded=0 "
            f"tests={tests} passed={passed} failed={failed} skipped={skipped} "
            f"warned={warned} elapsed={elapsed}s timed_out={timed_out} crashed={crashed}"
        ),
    ]
    if run_tests_exit is not None:
        lines.append(f"run-tests-exit: {run_tests_exit}")
    lines.append("")
    atomic_write_text(path, "\n".join(lines))


def chunk_exception_result(chunk: list[Row], exc: Exception) -> Result:
    return Result(
        index=chunk[0].index,
        rows=len(chunk),
        first_rel=chunk[0].rel,
        last_rel=chunk[-1].rel,
        state=f"runner-error-{type(exc).__name__}",
        exit_code=type(exc).__name__,
        elapsed_ms=0,
        tests=len(chunk),
        passed=0,
        failed=len(chunk),
        skipped=0,
        warned=0,
        timed_out=0,
        crashed=1,
    )


def main() -> int:
    args = parse_args()
    if args.timeout <= 0:
        raise SystemExit("--timeout must be positive")
    if args.workers <= 0 or args.workers > 16:
        raise SystemExit("--workers must be between 1 and 16")
    if args.batch_size <= 0 or args.batch_size > 256:
        raise SystemExit("--batch-size must be between 1 and 256")

    root = repo_root()
    php_src = resolve_php_src(args.php_src, root)
    phpc_bin = (args.phpc_bin or Path(os.environ.get("PHPC_BIN", root / "target/debug/phpc"))).resolve()
    if not phpc_bin.is_file():
        raise SystemExit(f"phpc binary not found: {phpc_bin}")
    rows = load_rows(args.manifest, php_src)
    args.out_dir.mkdir(parents=True, exist_ok=True)

    stamp = timestamp()
    print(
        f"[phpt-isolated] rows={len(rows)} workers={args.workers} "
        f"batch_size={args.batch_size} timeout={args.timeout}s",
        flush=True,
    )
    start = time.monotonic()
    results: list[Result] = []
    live_row_results = args.out_dir / "row-results-000-live.tsv"
    live_summary = args.out_dir / "summary-000-live.txt"

    def write_live_checkpoint() -> None:
        elapsed = int(time.monotonic() - start)
        write_results(live_row_results, results)
        write_summary(
            live_summary,
            stamp,
            root,
            php_src,
            args.manifest.resolve(),
            live_row_results.resolve(),
            results,
            elapsed,
            args.workers,
            args.batch_size,
            args.timeout,
            run_tests_exit=None,
        )

    write_live_checkpoint()
    next_progress = time.monotonic() + 60
    completed_rows = 0
    chunks = chunk_rows(rows, args.batch_size)
    with concurrent.futures.ThreadPoolExecutor(max_workers=args.workers) as executor:
        futures = {
            executor.submit(run_batch, chunk, php_src, phpc_bin, args.timeout): chunk
            for chunk in chunks
        }
        for future in concurrent.futures.as_completed(futures):
            try:
                batch_results = future.result()
            except Exception as exc:
                batch_results = [chunk_exception_result(futures[future], exc)]
            results.extend(batch_results)
            completed_rows += sum(item.rows for item in batch_results)
            write_live_checkpoint()
            now = time.monotonic()
            if now >= next_progress:
                print(
                    f"[phpt-isolated] completed={completed_rows}/{len(rows)} elapsed={int(now - start)}s",
                    flush=True,
                )
                next_progress = now + 60

    elapsed = int(time.monotonic() - start)
    row_results = args.out_dir / f"row-results-{stamp}.tsv"
    summary = args.out_dir / f"summary-{stamp}.txt"
    write_results(row_results, results)
    write_summary(
        summary,
        stamp,
        root,
        php_src,
        args.manifest.resolve(),
        row_results.resolve(),
        results,
        elapsed,
        args.workers,
        args.batch_size,
        args.timeout,
        run_tests_exit=0,
    )
    write_summary(
        live_summary,
        stamp,
        root,
        php_src,
        args.manifest.resolve(),
        live_row_results.resolve(),
        results,
        elapsed,
        args.workers,
        args.batch_size,
        args.timeout,
        run_tests_exit=0,
    )
    (args.out_dir / "shard-exit-code.txt").write_text("0\n", encoding="utf-8")
    selected = sum(item.rows for item in results)
    passed = sum(item.passed for item in results)
    failed = sum(item.failed for item in results)
    skipped = sum(item.skipped for item in results)
    warned = sum(item.warned for item in results)
    timed_out = sum(item.timed_out for item in results)
    crashed = sum(item.crashed for item in results)
    print(
        "[phpt-isolated] "
        f"tests={selected} passed={passed} failed={failed} "
        f"skipped={skipped} warned={warned} timed_out={timed_out} crashed={crashed}",
        flush=True,
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
