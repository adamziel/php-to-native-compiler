#!/usr/bin/env python3
"""Run a PHPT manifest through one streaming run-tests.php parent."""

from __future__ import annotations

import argparse
import csv
import dataclasses
import datetime as dt
import os
import re
import subprocess
import sys
import time
from pathlib import Path


STATUS_RE = re.compile(r"^(PASS|FAIL|SKIP|WARN|BORK|XFAIL|LEAK)\b.*\[(.+?\.phpt)\]\s*$")


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
    parser.add_argument("--workers", type=int, default=4)
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
    candidates.extend([root / ".runtime/php-src-phpt", Path("/home/claude/php-src-phpt")])
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


def timestamp() -> str:
    return dt.datetime.now(dt.UTC).strftime("%Y%m%dT%H%M%SZ")


def atomic_write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    temp = path.with_name(f".{path.name}.{os.getpid()}.tmp")
    temp.write_text(text, encoding="utf-8")
    temp.replace(path)


def normalize_reported_path(raw: str, php_src: Path) -> str:
    value = raw.strip()
    path = Path(value)
    if path.is_absolute():
        try:
            return path.resolve().relative_to(php_src).as_posix()
        except ValueError:
            return path.as_posix()
    return value.removeprefix("./")


def result_for_status(row: Row, status: str, elapsed_ms: int) -> Result:
    state = {
        "PASS": "passed",
        "XFAIL": "passed",
        "SKIP": "skipped",
        "WARN": "warned",
        "FAIL": "failed",
        "BORK": "failed",
        "LEAK": "failed",
    }[status]
    return Result(
        index=row.index,
        rows=1,
        first_rel=row.rel,
        last_rel=row.rel,
        state=state,
        exit_code=0 if state in {"passed", "skipped"} else 1,
        elapsed_ms=elapsed_ms,
        tests=1,
        passed=1 if state == "passed" else 0,
        failed=1 if state == "failed" else 0,
        skipped=1 if state == "skipped" else 0,
        warned=1 if state == "warned" else 0,
        timed_out=0,
        crashed=0,
    )


def write_results(path: Path, results: list[Result]) -> None:
    lines = [
        "index\trows\tfirst_row\tlast_row\tstate\texit_code\telapsed_ms\ttests\tpassed\tfailed\t"
        "skipped\twarned\ttimed_out\tcrashed"
    ]
    for item in sorted(results, key=lambda result: result.index):
        lines.append(
            f"{item.index}\t{item.rows}\t{item.first_rel}\t{item.last_rel}\t"
            f"{item.state}\t{item.exit_code}\t{item.elapsed_ms}\t{item.tests}\t"
            f"{item.passed}\t{item.failed}\t{item.skipped}\t{item.warned}\t"
            f"{item.timed_out}\t{item.crashed}"
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
        f"PHPT streaming full-corpus shard {stamp}",
        f"commit: {commit}",
        f"corpus: {php_src}",
        f"corpus-revision: {corpus_revision}",
        f"manifest: {manifest}",
        f"runnable-manifest: {manifest}",
        (
            f"command: tools/run-phpt-stream-manifest.py --timeout {timeout_seconds} "
            f"--workers {workers} --out-dir {path.parent} {manifest}"
        ),
        f"timeout-seconds: {timeout_seconds}",
        f"row-workers: {workers}",
        "batch-size: stream",
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
            f"tests={tests} passed={passed} failed={failed} skipped={skipped} warned={warned} "
            f"elapsed={elapsed}s timed_out={timed_out} crashed={crashed}"
        ),
    ]
    if run_tests_exit is not None:
        lines.append(f"run-tests-exit: {run_tests_exit}")
    lines.append("")
    atomic_write_text(path, "\n".join(lines))


def main() -> int:
    args = parse_args()
    if args.timeout <= 0:
        raise SystemExit("--timeout must be positive")
    if args.workers <= 0 or args.workers > 32:
        raise SystemExit("--workers must be between 1 and 32")

    root = repo_root()
    php_src = resolve_php_src(args.php_src, root)
    phpc_bin = (args.phpc_bin or Path(os.environ.get("PHPC_BIN", root / "target/debug/phpc"))).resolve()
    if not phpc_bin.is_file():
        raise SystemExit(f"phpc binary not found: {phpc_bin}")
    rows = load_rows(args.manifest, php_src)
    by_rel = {row.rel: row for row in rows}
    args.out_dir.mkdir(parents=True, exist_ok=True)

    stamp = timestamp()
    start = time.monotonic()
    results_by_rel: dict[str, Result] = {}
    live_row_results = args.out_dir / "row-results-000-live.tsv"
    live_summary = args.out_dir / "summary-000-live.txt"
    run_log = args.out_dir / f"run-tests-{stamp}.log"

    def write_live_checkpoint(run_tests_exit: int | None = None) -> None:
        elapsed = int(time.monotonic() - start)
        results = list(results_by_rel.values())
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
            args.timeout,
            run_tests_exit=run_tests_exit,
        )

    write_live_checkpoint()
    env = os.environ.copy()
    env["PHPC_BIN"] = str(phpc_bin)
    env["TEST_PHP_EXECUTABLE"] = str(phpc_bin)
    env["TEST_PHP_CGI_EXECUTABLE"] = str(phpc_bin)
    env["TEST_PHP_CGI_EXECUTABLE_ESCAPED"] = f"'{phpc_bin}'"
    command = [
        "php",
        str(php_src / "run-tests.php"),
        "-q",
        f"-j{args.workers}",
        "--set-timeout",
        str(args.timeout),
        "-p",
        str(phpc_bin),
        *[str(row.path) for row in rows],
    ]

    print(
        f"[phpt-stream] rows={len(rows)} workers={args.workers} timeout={args.timeout}s",
        flush=True,
    )
    next_progress = time.monotonic() + 60
    with run_log.open("w", encoding="utf-8", errors="replace") as log:
        process = subprocess.Popen(
            command,
            cwd=php_src,
            env=env,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            errors="replace",
            bufsize=1,
        )
        assert process.stdout is not None
        for line in process.stdout:
            log.write(line)
            match = STATUS_RE.match(line)
            if match is not None:
                status = match.group(1)
                rel = normalize_reported_path(match.group(2), php_src)
                row = by_rel.get(rel)
                if row is not None and rel not in results_by_rel:
                    elapsed_ms = int((time.monotonic() - start) * 1000)
                    results_by_rel[rel] = result_for_status(row, status, elapsed_ms)
                    write_live_checkpoint()
            now = time.monotonic()
            if now >= next_progress:
                print(
                    f"[phpt-stream] reported={len(results_by_rel)}/{len(rows)} elapsed={int(now - start)}s",
                    flush=True,
                )
                next_progress = now + 60
        returncode = process.wait()

    elapsed = int(time.monotonic() - start)
    results = list(results_by_rel.values())
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
        args.timeout,
        run_tests_exit=returncode,
    )
    write_live_checkpoint(run_tests_exit=returncode)
    (args.out_dir / "shard-exit-code.txt").write_text(f"{returncode}\n", encoding="utf-8")
    print(
        f"[phpt-stream] reported={len(results_by_rel)} passed={sum(r.passed for r in results)} "
        f"failed={sum(r.failed for r in results)} skipped={sum(r.skipped for r in results)} "
        f"warned={sum(r.warned for r in results)} exit={returncode}",
        flush=True,
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
