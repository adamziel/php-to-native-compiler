#!/usr/bin/env python3
"""Supervise a local full PHPT run with live checkpoints.

This runner is intended to run inside a detached tmux session. It keeps one
run-tests.php parent per chunk for throughput, records every reported row, and
retries only rows that were not reported. If a chunk makes no progress, it
falls back to a single-row run so one bad row can be marked crashed and the run
can continue.
"""

from __future__ import annotations

import argparse
import dataclasses
import datetime as dt
import os
import re
import select
import signal
import subprocess
import sys
import time
from pathlib import Path


STATUS_RE = re.compile(r"^(PASS|FAIL|SKIP|WARN|BORK|XFAIL|LEAK)\b.*\[(.+?\.phpt)\]\s*$")
ANSI_RE = re.compile(r"\x1b\[[0-9;]*[A-Za-z]")


@dataclasses.dataclass(frozen=True)
class Row:
    index: int
    rel: str
    path: Path


@dataclasses.dataclass(frozen=True)
class Result:
    index: int
    rows: int
    first_row: str
    last_row: str
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
    parser.add_argument("--out-dir", type=Path, required=True)
    parser.add_argument("--manifest", type=Path)
    parser.add_argument("--php-src", type=Path)
    parser.add_argument("--phpc-bin", type=Path, required=True)
    parser.add_argument("--timeout", type=int, default=60)
    parser.add_argument("--jobs", type=int, default=8)
    parser.add_argument("--stream-batch-size", type=int, default=500)
    parser.add_argument("--stall-timeout", type=int, default=600)
    parser.add_argument("--single-wall-timeout", type=int, default=300)
    parser.add_argument(
        "--recover-one-row",
        action="store_true",
        help="run and checkpoint only the first unreported row, then exit",
    )
    return parser.parse_args()


def repo_root() -> Path:
    return Path(__file__).resolve().parents[1]


def timestamp() -> str:
    return dt.datetime.now(dt.UTC).strftime("%Y-%m-%dT%H:%M:%SZ")


def resolve_php_src(root: Path, arg: Path | None) -> Path:
    candidates: list[Path] = []
    if arg is not None:
        candidates.append(arg)
    if os.environ.get("PHP_SRC_PHPT"):
        candidates.append(Path(os.environ["PHP_SRC_PHPT"]))
    candidates.extend([root / ".runtime/php-src-phpt", Path("/home/claude/php-src-phpt")])
    for candidate in candidates:
        if (candidate / "run-tests.php").is_file():
            return candidate.resolve()
    raise SystemExit("could not resolve php-src checkout with run-tests.php")


def trim_manifest_row(line: str) -> str:
    return line.split("#", 1)[0].strip()


def load_manifest(path: Path, php_src: Path) -> list[Row]:
    rows: list[Row] = []
    for raw in path.read_text(encoding="utf-8").splitlines():
        value = trim_manifest_row(raw)
        if not value:
            continue
        row_path = Path(value)
        if row_path.is_absolute():
            abs_path = row_path
            try:
                rel = abs_path.resolve().relative_to(php_src).as_posix()
            except ValueError:
                rel = abs_path.as_posix()
        else:
            rel = value
            abs_path = php_src / value
        if abs_path.is_file():
            rows.append(Row(len(rows), rel, abs_path))
    if not rows:
        raise SystemExit(f"manifest contains no PHPT rows: {path}")
    return rows


def load_full_corpus(php_src: Path) -> list[Row]:
    rels: list[str] = []
    for path in php_src.rglob("*.phpt"):
        if ".git" in path.parts:
            continue
        rels.append(path.relative_to(php_src).as_posix())
    rels.sort()
    return [Row(index, rel, php_src / rel) for index, rel in enumerate(rels)]


def normalize_reported_path(raw: str, php_src: Path) -> str:
    value = raw.strip().removeprefix("./")
    path = Path(value)
    if path.is_absolute():
        try:
            return path.resolve().relative_to(php_src).as_posix()
        except ValueError:
            return path.as_posix()
    return value


def clean_run_tests_line(line: str) -> str:
    return ANSI_RE.sub("", line).strip()


def atomic_write(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp = path.with_name(f".{path.name}.{os.getpid()}.tmp")
    tmp.write_text(text, encoding="utf-8")
    tmp.replace(path)


def result_from_status(row: Row, status: str, elapsed_ms: int) -> Result:
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
        first_row=row.rel,
        last_row=row.rel,
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


def crashed_result(row: Row, elapsed_ms: int, exit_code: int | str) -> Result:
    return Result(
        index=row.index,
        rows=1,
        first_row=row.rel,
        last_row=row.rel,
        state="crashed",
        exit_code=exit_code,
        elapsed_ms=elapsed_ms,
        tests=1,
        passed=0,
        failed=1,
        skipped=0,
        warned=0,
        timed_out=0,
        crashed=1,
    )


def write_results(path: Path, results: dict[str, Result]) -> None:
    lines = [
        "index\trows\tfirst_row\tlast_row\tstate\texit_code\telapsed_ms\ttests\tpassed\tfailed\t"
        "skipped\twarned\ttimed_out\tcrashed"
    ]
    for result in sorted(results.values(), key=lambda item: item.index):
        lines.append(
            f"{result.index}\t{result.rows}\t{result.first_row}\t{result.last_row}\t"
            f"{result.state}\t{result.exit_code}\t{result.elapsed_ms}\t{result.tests}\t"
            f"{result.passed}\t{result.failed}\t{result.skipped}\t{result.warned}\t"
            f"{result.timed_out}\t{result.crashed}"
        )
    atomic_write(path, "\n".join(lines) + "\n")


def counts(results: dict[str, Result]) -> dict[str, int]:
    return {
        "reported": len(results),
        "passed": sum(item.passed for item in results.values()),
        "failed": sum(item.failed for item in results.values()),
        "skipped": sum(item.skipped for item in results.values()),
        "warned": sum(item.warned for item in results.values()),
        "crashed": sum(item.crashed for item in results.values()),
    }


def write_status(
    path: Path,
    *,
    state: str,
    total: int,
    results: dict[str, Result],
    mode: str,
    current: str,
    last_row: str,
    last_state: str,
    started_at: str,
    run_dir: Path,
) -> None:
    values = counts(results)
    values["remaining"] = total - values["reported"]
    lines = [
        f"state\t{state}",
        f"started_at_utc\t{started_at}",
        f"updated_at_utc\t{timestamp()}",
        f"run_dir\t{run_dir}",
        f"total\t{total}",
        f"reported\t{values['reported']}",
        f"remaining\t{values['remaining']}",
        f"passed\t{values['passed']}",
        f"failed\t{values['failed']}",
        f"skipped\t{values['skipped']}",
        f"warned\t{values['warned']}",
        f"crashed\t{values['crashed']}",
        f"mode\t{mode}",
        f"current\t{current}",
        f"last_row\t{last_row}",
        f"last_state\t{last_state}",
    ]
    atomic_write(path, "\n".join(lines) + "\n")


def load_existing_results(path: Path, rows_by_rel: dict[str, Row]) -> dict[str, Result]:
    if not path.is_file():
        return {}
    results: dict[str, Result] = {}
    for raw in path.read_text(encoding="utf-8").splitlines()[1:]:
        parts = raw.split("\t")
        if len(parts) != 14:
            continue
        rel = parts[2]
        if rel not in rows_by_rel:
            continue
        results[rel] = Result(
            index=int(parts[0]),
            rows=int(parts[1]),
            first_row=parts[2],
            last_row=parts[3],
            state=parts[4],
            exit_code=parts[5],
            elapsed_ms=int(parts[6]),
            tests=int(parts[7]),
            passed=int(parts[8]),
            failed=int(parts[9]),
            skipped=int(parts[10]),
            warned=int(parts[11]),
            timed_out=int(parts[12]),
            crashed=int(parts[13]),
        )
    return results


def kill_process_group(process: subprocess.Popen[str]) -> None:
    try:
        os.killpg(process.pid, signal.SIGTERM)
        process.wait(timeout=15)
    except Exception:
        try:
            os.killpg(process.pid, signal.SIGKILL)
        except Exception:
            pass
        try:
            process.wait(timeout=15)
        except Exception:
            pass


def run_stream_chunk(
    *,
    rows: list[Row],
    rows_by_rel: dict[str, Row],
    results: dict[str, Result],
    php_src: Path,
    phpc_bin: Path,
    timeout: int,
    jobs: int,
    stall_timeout: int,
    log_path: Path,
    status_path: Path,
    results_path: Path,
    started_at: str,
    run_dir: Path,
) -> int:
    before = len(results)
    command = [
        "php",
        str(php_src / "run-tests.php"),
        "-q",
        f"-j{jobs}",
        "--set-timeout",
        str(timeout),
        "-p",
        str(phpc_bin),
        *[str(row.path) for row in rows],
    ]
    env = os.environ.copy()
    env["PHPC_BIN"] = str(phpc_bin)
    env["TEST_PHP_EXECUTABLE"] = str(phpc_bin)
    env["TEST_PHP_CGI_EXECUTABLE"] = str(phpc_bin)
    env["TEST_PHP_CGI_EXECUTABLE_ESCAPED"] = f"'{phpc_bin}'"
    last_progress = time.monotonic()
    process = subprocess.Popen(
        command,
        cwd=php_src,
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        errors="replace",
        bufsize=1,
        start_new_session=True,
    )
    assert process.stdout is not None
    current = rows[0].rel if rows else ""
    with log_path.open("a", encoding="utf-8", errors="replace") as log:
        log.write(f"\n===== stream chunk start {timestamp()} rows={len(rows)} first={current} =====\n")
        while process.poll() is None:
            readable, _, _ = select.select([process.stdout], [], [], 1)
            if readable:
                line = process.stdout.readline()
                if line:
                    log.write(line)
                    match = STATUS_RE.match(clean_run_tests_line(line))
                    if match is not None:
                        rel = normalize_reported_path(match.group(2), php_src)
                        row = rows_by_rel.get(rel)
                        if row is not None and rel not in results:
                            results[rel] = result_from_status(
                                row,
                                match.group(1),
                                int((time.monotonic() - last_progress) * 1000),
                            )
                            last_progress = time.monotonic()
                            current = rel
                            write_results(results_path, results)
                            write_status(
                                status_path,
                                state="running",
                                total=len(rows_by_rel),
                                results=results,
                                mode="stream",
                                current=current,
                                last_row=rel,
                                last_state=results[rel].state,
                                started_at=started_at,
                                run_dir=run_dir,
                            )
            if time.monotonic() - last_progress > stall_timeout:
                log.write(f"\n===== stream chunk stalled {timestamp()} =====\n")
                kill_process_group(process)
                break
        for line in process.stdout:
            log.write(line)
            match = STATUS_RE.match(clean_run_tests_line(line))
            if match is not None:
                rel = normalize_reported_path(match.group(2), php_src)
                row = rows_by_rel.get(rel)
                if row is not None and rel not in results:
                    results[rel] = result_from_status(
                        row,
                        match.group(1),
                        int((time.monotonic() - last_progress) * 1000),
                    )
                    last_progress = time.monotonic()
                    write_results(results_path, results)
        exit_code = process.wait()
        log.write(f"\n===== stream chunk exit {exit_code} {timestamp()} =====\n")
    return len(results) - before


def run_single_fallback(
    *,
    row: Row,
    rows_by_rel: dict[str, Row],
    results: dict[str, Result],
    php_src: Path,
    phpc_bin: Path,
    timeout: int,
    wall_timeout: int,
    log_path: Path,
    status_path: Path,
    results_path: Path,
    started_at: str,
    run_dir: Path,
) -> None:
    command = [
        "php",
        str(php_src / "run-tests.php"),
        "-q",
        "--set-timeout",
        str(timeout),
        "-p",
        str(phpc_bin),
        str(row.path),
    ]
    env = os.environ.copy()
    env["PHPC_BIN"] = str(phpc_bin)
    env["TEST_PHP_EXECUTABLE"] = str(phpc_bin)
    env["TEST_PHP_CGI_EXECUTABLE"] = str(phpc_bin)
    env["TEST_PHP_CGI_EXECUTABLE_ESCAPED"] = f"'{phpc_bin}'"
    write_status(
        status_path,
        state="running",
        total=len(rows_by_rel),
        results=results,
        mode="single",
        current=row.rel,
        last_row=row.rel,
        last_state="running",
        started_at=started_at,
        run_dir=run_dir,
    )
    start = time.monotonic()
    process = subprocess.Popen(
        command,
        cwd=php_src,
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        errors="replace",
        bufsize=1,
        start_new_session=True,
    )
    matched_status: str | None = None
    with log_path.open("a", encoding="utf-8", errors="replace") as log:
        log.write(f"\n===== single fallback start {timestamp()} row={row.rel} =====\n")
        try:
            output, _ = process.communicate(timeout=wall_timeout)
        except subprocess.TimeoutExpired:
            log.write(f"\n===== single fallback wall-timeout {timestamp()} row={row.rel} =====\n")
            kill_process_group(process)
            output = ""
        log.write(output)
        for line in output.splitlines():
            match = STATUS_RE.match(clean_run_tests_line(line))
            if match is not None:
                rel = normalize_reported_path(match.group(2), php_src)
                if rel == row.rel:
                    matched_status = match.group(1)
        exit_code = process.poll()
        log.write(f"\n===== single fallback exit {exit_code} {timestamp()} row={row.rel} =====\n")
    elapsed_ms = int((time.monotonic() - start) * 1000)
    if matched_status is not None:
        results[row.rel] = result_from_status(row, matched_status, elapsed_ms)
    else:
        results[row.rel] = crashed_result(row, elapsed_ms, exit_code if exit_code is not None else "timeout")
    write_results(results_path, results)
    write_status(
        status_path,
        state="running",
        total=len(rows_by_rel),
        results=results,
        mode="single",
        current=row.rel,
        last_row=row.rel,
        last_state=results[row.rel].state,
        started_at=started_at,
        run_dir=run_dir,
    )


def main() -> int:
    args = parse_args()
    root = repo_root()
    php_src = resolve_php_src(root, args.php_src)
    phpc_bin = args.phpc_bin.resolve()
    if not phpc_bin.is_file():
        raise SystemExit(f"phpc binary not found: {phpc_bin}")
    args.out_dir.mkdir(parents=True, exist_ok=True)

    rows = load_manifest(args.manifest, php_src) if args.manifest else load_full_corpus(php_src)
    rows_by_rel = {row.rel: row for row in rows}
    manifest_path = args.out_dir / "manifest.txt"
    if not manifest_path.exists():
        atomic_write(
            manifest_path,
            "\n".join(
                [
                    "# Generated by tools/run-phpt-local-stream-supervisor.py",
                    f"# corpus: {php_src}",
                    f"# total: {len(rows)}",
                    "",
                    *[row.rel for row in rows],
                ]
            )
            + "\n",
        )

    status_path = args.out_dir / "status.tsv"
    results_path = args.out_dir / "row-results.tsv"
    log_path = args.out_dir / "run-tests.log"
    started_at = timestamp()
    results = load_existing_results(results_path, rows_by_rel)
    write_results(results_path, results)

    if args.recover_one_row:
        remaining = [row for row in rows if row.rel not in results]
        if not remaining:
            write_status(
                status_path,
                state="complete",
                total=len(rows),
                results=results,
                mode="done",
                current="",
                last_row="",
                last_state="",
                started_at=started_at,
                run_dir=args.out_dir.resolve(),
            )
            return 0
        run_single_fallback(
            row=remaining[0],
            rows_by_rel=rows_by_rel,
            results=results,
            php_src=php_src,
            phpc_bin=phpc_bin,
            timeout=args.timeout,
            wall_timeout=args.single_wall_timeout,
            log_path=log_path,
            status_path=status_path,
            results_path=results_path,
            started_at=started_at,
            run_dir=args.out_dir.resolve(),
        )
        return 0

    while len(results) < len(rows):
        remaining = [row for row in rows if row.rel not in results]
        chunk = remaining[: args.stream_batch_size]
        write_status(
            status_path,
            state="running",
            total=len(rows),
            results=results,
            mode="stream",
            current=chunk[0].rel if chunk else "",
            last_row="",
            last_state="",
            started_at=started_at,
            run_dir=args.out_dir.resolve(),
        )
        run_stream_chunk(
            rows=chunk,
            rows_by_rel=rows_by_rel,
            results=results,
            php_src=php_src,
            phpc_bin=phpc_bin,
            timeout=args.timeout,
            jobs=args.jobs,
            stall_timeout=args.stall_timeout,
            log_path=log_path,
            status_path=status_path,
            results_path=results_path,
            started_at=started_at,
            run_dir=args.out_dir.resolve(),
        )
        unreported_chunk_rows = [row for row in chunk if row.rel not in results]
        if unreported_chunk_rows:
            run_single_fallback(
                row=unreported_chunk_rows[0],
                rows_by_rel=rows_by_rel,
                results=results,
                php_src=php_src,
                phpc_bin=phpc_bin,
                timeout=args.timeout,
                wall_timeout=args.single_wall_timeout,
                log_path=log_path,
                status_path=status_path,
                results_path=results_path,
                started_at=started_at,
                run_dir=args.out_dir.resolve(),
            )

    write_status(
        status_path,
        state="complete",
        total=len(rows),
        results=results,
        mode="done",
        current="",
        last_row="",
        last_state="",
        started_at=started_at,
        run_dir=args.out_dir.resolve(),
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
