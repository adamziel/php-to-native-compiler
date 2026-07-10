#!/usr/bin/env python3
"""Compose an auditable PHPT oracle ledger from run-tests.php -W output."""

from __future__ import annotations

import argparse
import csv
import hashlib
import os
import re
import stat
import sys
import tempfile
from collections import Counter
from dataclasses import dataclass
from pathlib import Path, PurePosixPath


MAX_ROWS = 100_000
MAX_RECORD_BYTES = 32 * 1024
MAX_FILE_BYTES = 64 * 1024 * 1024
SHA256 = re.compile(r"[0-9a-f]{64}\Z")
NORMALIZED_STATUSES = (
    "PASS",
    "SKIP",
    "XFAIL",
    "XLEAK",
    "FLAKY",
    "FAIL",
    "BORK",
    "WARN",
    "LEAK",
    "TIMEOUT",
)
STATUS_ALIASES = {
    "PASS": "PASS",
    "PASSED": "PASS",
    "SKIP": "SKIP",
    "SKIPPED": "SKIP",
    "XFAIL": "XFAIL",
    "XFAILED": "XFAIL",
    "XLEAK": "XLEAK",
    "XLEAKED": "XLEAK",
    "FLAKY": "FLAKY",
    "FLAKIED": "FLAKY",
    "FAIL": "FAIL",
    "FAILED": "FAIL",
    "BORK": "BORK",
    "BORKED": "BORK",
    "WARN": "WARN",
    "WARNED": "WARN",
    "LEAK": "LEAK",
    "LEAKED": "LEAK",
    "TIMEOUT": "TIMEOUT",
    "TIMEDOUT": "TIMEOUT",
    "TIMED_OUT": "TIMEOUT",
}


class LedgerError(Exception):
    pass


@dataclass(frozen=True)
class Result:
    sequence: int
    status: str
    emitted_status: str
    path: str
    redirect_parent: str


def safe_text(value: object) -> str:
    text = "".join(
        char if char.isprintable() else f"\\x{ord(char):02x}"
        for char in str(value)
    )
    return text if len(text) <= 220 else f"{text[:217]}..."


def read_bounded_lines(path: Path, limit: int = MAX_ROWS) -> list[str]:
    try:
        info = path.stat()
        if path.is_symlink() or not stat.S_ISREG(info.st_mode):
            raise LedgerError(f"not a regular file: {safe_text(path)}")
        if info.st_size > MAX_FILE_BYTES:
            raise LedgerError(f"file exceeds size limit: {safe_text(path)}")
        rows: list[str] = []
        with path.open("rb") as source:
            for line_number, raw in enumerate(source, 1):
                if line_number > limit:
                    raise LedgerError(f"too many records: {safe_text(path)}")
                if len(raw) > MAX_RECORD_BYTES:
                    raise LedgerError(
                        f"record {line_number} exceeds size limit: {safe_text(path)}"
                    )
                try:
                    line = raw.decode("utf-8")
                except UnicodeDecodeError as exc:
                    raise LedgerError(
                        f"record {line_number} is not UTF-8: {safe_text(path)}"
                    ) from exc
                rows.append(line.rstrip("\r\n"))
        return rows
    except OSError as exc:
        raise LedgerError(f"cannot read {safe_text(path)}: {safe_text(exc)}") from exc


def validate_path(value: str, context: str) -> str:
    if not value or any(char in value for char in "\t\r\n\0"):
        raise LedgerError(f"invalid path in {context}")
    path = PurePosixPath(value)
    if path.is_absolute() or value.startswith("./") or any(part in ("", ".", "..") for part in path.parts):
        raise LedgerError(f"path is not normalized in {context}: {safe_text(value)}")
    if path.suffix != ".phpt":
        raise LedgerError(f"non-PHPT path in {context}: {safe_text(value)}")
    return value


def load_inventory(path: Path) -> tuple[list[str], str]:
    rows = read_bounded_lines(path)
    if not rows:
        raise LedgerError("inventory is empty")
    inventory: list[str] = []
    seen: set[str] = set()
    previous: str | None = None
    for line_number, row in enumerate(rows, 1):
        value = validate_path(row, f"inventory line {line_number}")
        if value in seen:
            raise LedgerError(f"duplicate inventory path: {safe_text(value)}")
        if previous is not None and value <= previous:
            raise LedgerError("inventory is not bytewise sorted")
        inventory.append(value)
        seen.add(value)
        previous = value
    digest = hashlib.sha256(("\n".join(inventory) + "\n").encode()).hexdigest()
    return inventory, digest


def normalize_result_path(
    value: str, corpus_root: Path | None, context: str
) -> str:
    if value.startswith("/"):
        if corpus_root is None:
            raise LedgerError(f"absolute path without corpus root in {context}")
        try:
            relative = Path(value).resolve().relative_to(corpus_root.resolve())
        except (OSError, ValueError) as exc:
            raise LedgerError(
                f"absolute path is outside corpus in {context}: {safe_text(value)}"
            ) from exc
        return validate_path(relative.as_posix(), context)
    return validate_path(value, context)


def split_result_index(
    value: str, line_number: int, corpus_root: Path | None
) -> tuple[str, str]:
    if value.startswith("# "):
        body = value[2:]
        if ": " not in body:
            raise LedgerError(f"malformed redirect index at result line {line_number}")
        parent, child = body.split(": ", 1)
        return (
            normalize_result_path(child, corpus_root, f"result line {line_number}"),
            normalize_result_path(
                parent, corpus_root, f"redirect parent at result line {line_number}"
            ),
        )
    return normalize_result_path(value, corpus_root, f"result line {line_number}"), ""


def load_results(path: Path, inventory: set[str], corpus_root: Path | None) -> list[Result]:
    rows = read_bounded_lines(path, MAX_ROWS * 5)
    results: list[Result] = []
    direct_seen: set[str] = set()
    redirect_seen: set[tuple[str, str]] = set()
    for line_number, row in enumerate(rows, 1):
        if row.count("\t") != 1:
            raise LedgerError(f"result line {line_number} is not two-column TSV")
        emitted_status, index = row.split("\t")
        status = STATUS_ALIASES.get(emitted_status)
        if status is None:
            raise LedgerError(
                f"unsupported status at result line {line_number}: {safe_text(emitted_status)}"
            )
        path, parent = split_result_index(index, line_number, corpus_root)
        if parent:
            if parent not in inventory:
                raise LedgerError(
                    f"redirect parent is outside inventory at result line {line_number}: {safe_text(parent)}"
                )
            key = (parent, path)
            if key in redirect_seen:
                raise LedgerError(f"duplicate redirect result at line {line_number}")
            redirect_seen.add(key)
        else:
            if path not in inventory:
                raise LedgerError(
                    f"direct result is outside inventory at line {line_number}: {safe_text(path)}"
                )
            if path in direct_seen:
                raise LedgerError(f"duplicate direct result at line {line_number}")
            direct_seen.add(path)
        if corpus_root is not None:
            candidate = corpus_root.joinpath(*PurePosixPath(path).parts)
            if not candidate.is_file():
                raise LedgerError(
                    f"result path is absent from corpus at line {line_number}: {safe_text(path)}"
                )
        results.append(Result(line_number, status, emitted_status, path, parent))
    return results


def atomic_write_tsv(path: Path, header: tuple[str, ...], rows: list[tuple[object, ...]]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    descriptor, temporary = tempfile.mkstemp(prefix=f".{path.name}.", dir=path.parent)
    try:
        with os.fdopen(descriptor, "w", encoding="utf-8", newline="") as output:
            writer = csv.writer(output, delimiter="\t", lineterminator="\n")
            writer.writerow(header)
            writer.writerows(rows)
            output.flush()
            os.fsync(output.fileno())
        os.replace(temporary, path)
    except BaseException:
        try:
            os.unlink(temporary)
        except FileNotFoundError:
            pass
        raise


def atomic_write_lines(path: Path, rows: list[str]) -> None:
    atomic_write_tsv(path, ("path",), [(row,) for row in rows])


def compose(
    inventory_path: Path,
    results_path: Path,
    ledger_path: Path,
    summary_path: Path,
    unresolved_path: Path,
    corpus_root: Path | None,
    expected_count: int | None,
    expected_sha256: str | None,
) -> tuple[int, int, int]:
    inventory, inventory_sha256 = load_inventory(inventory_path)
    if expected_count is not None and len(inventory) != expected_count:
        raise LedgerError(
            f"inventory count mismatch: expected {expected_count}, found {len(inventory)}"
        )
    if expected_sha256 is not None and inventory_sha256 != expected_sha256:
        raise LedgerError("inventory SHA-256 mismatch")
    inventory_set = set(inventory)
    results = load_results(results_path, inventory_set, corpus_root)
    direct = {result.path for result in results if not result.redirect_parent}
    redirect_parents = {result.redirect_parent for result in results if result.redirect_parent}
    covered = direct | redirect_parents
    unresolved = [path for path in inventory if path not in covered]
    counts = Counter(result.status for result in results)

    atomic_write_tsv(
        ledger_path,
        ("sequence", "status", "emitted_status", "path", "redirect_parent"),
        [
            (
                result.sequence,
                result.status,
                result.emitted_status,
                result.path,
                result.redirect_parent,
            )
            for result in results
        ],
    )
    summary_rows: list[tuple[object, ...]] = [
        ("schema", 1),
        ("inventory_count", len(inventory)),
        ("inventory_sha256", inventory_sha256),
        ("result_records", len(results)),
        ("covered_inventory", len(covered)),
        ("unresolved_inventory", len(unresolved)),
        ("redirect_parents", len(redirect_parents)),
    ]
    summary_rows.extend((status, counts[status]) for status in NORMALIZED_STATUSES)
    atomic_write_tsv(summary_path, ("metric", "value"), summary_rows)
    atomic_write_lines(unresolved_path, unresolved)
    return len(results), len(covered), len(unresolved)


def self_test() -> int:
    with tempfile.TemporaryDirectory(prefix="phpt-oracle-ledger-") as temporary:
        root = Path(temporary)
        corpus = root / "corpus"
        corpus.mkdir()
        statuses = [
            ("PASSED", "PASS"),
            ("SKIPPED", "SKIP"),
            ("XFAILED", "XFAIL"),
            ("XLEAKED", "XLEAK"),
            ("FLAKY", "FLAKY"),
            ("FAILED", "FAIL"),
            ("BORKED", "BORK"),
            ("WARNED", "WARN"),
            ("LEAKED", "LEAK"),
            ("TIMEOUT", "TIMEOUT"),
        ]
        inventory_rows = [f"tests/{index:02d}.phpt" for index in range(len(statuses))]
        inventory_rows.append("tests/redirect.phpt")
        inventory = root / "inventory.txt"
        inventory.write_text("\n".join(inventory_rows) + "\n", encoding="utf-8")
        for path in inventory_rows + ["tests/redirect-child.phpt"]:
            target = corpus / path
            target.parent.mkdir(parents=True, exist_ok=True)
            target.touch()
        raw = root / "raw.tsv"
        raw_rows = [
            f"{emitted}\t{path}"
            for (emitted, _), path in zip(statuses, inventory_rows)
        ]
        redirect_child = (corpus / "tests/redirect-child.phpt").resolve()
        raw_rows.append(f"PASSED\t# tests/redirect.phpt: {redirect_child}")
        raw.write_text("\n".join(raw_rows) + "\n", encoding="utf-8")
        expected_sha = hashlib.sha256(inventory.read_bytes()).hexdigest()
        output = root / "output"
        records, covered, unresolved = compose(
            inventory,
            raw,
            output / "ledger.tsv",
            output / "summary.tsv",
            output / "unresolved.tsv",
            corpus,
            len(inventory_rows),
            expected_sha,
        )
        if (records, covered, unresolved) != (11, 11, 0):
            print("self-test failed: incorrect accounting", file=sys.stderr)
            return 1
        ledger = (output / "ledger.tsv").read_text(encoding="utf-8")
        for _, normalized in statuses:
            if f"\t{normalized}\t" not in ledger:
                print(f"self-test failed: missing {normalized}", file=sys.stderr)
                return 1
        raw.write_text("UNKNOWN\ttests/00.phpt\n", encoding="utf-8")
        try:
            compose(
                inventory,
                raw,
                output / "ledger.tsv",
                output / "summary.tsv",
                output / "unresolved.tsv",
                corpus,
                len(inventory_rows),
                expected_sha,
            )
        except LedgerError:
            pass
        else:
            print("self-test failed: unknown status accepted", file=sys.stderr)
            return 1
        raw.write_text("PASSED\ttests/00.phpt\n", encoding="utf-8")
        _, _, unresolved = compose(
            inventory,
            raw,
            output / "ledger.tsv",
            output / "summary.tsv",
            output / "unresolved.tsv",
            corpus,
            len(inventory_rows),
            expected_sha,
        )
        if unresolved != len(inventory_rows) - 1:
            print("self-test failed: incomplete ledger accounting", file=sys.stderr)
            return 1
    print("self-test: ok")
    return 0


class BoundedParser(argparse.ArgumentParser):
    def error(self, message: str) -> None:
        self.exit(2, f"{self.prog}: {safe_text(message)}\n")


def positive_count(value: str) -> int:
    if not value.isascii() or not value.isdigit():
        raise argparse.ArgumentTypeError("must be a positive integer")
    parsed = int(value)
    if parsed < 1 or parsed > MAX_ROWS:
        raise argparse.ArgumentTypeError(f"must be between 1 and {MAX_ROWS}")
    return parsed


def main() -> int:
    parser = BoundedParser(prog="phpt-oracle-ledger", description=__doc__)
    parser.add_argument("--inventory", type=Path)
    parser.add_argument("--results", type=Path)
    parser.add_argument("--ledger", type=Path)
    parser.add_argument("--summary", type=Path)
    parser.add_argument("--unresolved", type=Path)
    parser.add_argument("--corpus-root", type=Path)
    parser.add_argument("--expected-count", type=positive_count)
    parser.add_argument("--expected-sha256")
    parser.add_argument("--allow-incomplete", action="store_true")
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        if any(
            value is not None
            for value in (
                args.inventory,
                args.results,
                args.ledger,
                args.summary,
                args.unresolved,
                args.corpus_root,
                args.expected_count,
                args.expected_sha256,
            )
        ) or args.allow_incomplete:
            parser.error("--self-test cannot be combined with ledger arguments")
        return self_test()
    required = {
        "--inventory": args.inventory,
        "--results": args.results,
        "--ledger": args.ledger,
        "--summary": args.summary,
        "--unresolved": args.unresolved,
    }
    missing = [name for name, value in required.items() if value is None]
    if missing:
        parser.error(f"required arguments missing: {', '.join(missing)}")
    if args.expected_sha256 is not None and not SHA256.fullmatch(args.expected_sha256):
        parser.error("--expected-sha256 must be 64 lowercase hexadecimal characters")
    if args.corpus_root is not None and not args.corpus_root.is_dir():
        parser.error("--corpus-root must be a directory")
    try:
        records, covered, unresolved = compose(
            args.inventory,
            args.results,
            args.ledger,
            args.summary,
            args.unresolved,
            args.corpus_root,
            args.expected_count,
            args.expected_sha256,
        )
    except LedgerError as exc:
        print(f"phpt-oracle-ledger: {safe_text(exc)}", file=sys.stderr)
        return 2
    print(
        f"oracle-ledger: records={records} covered={covered} "
        f"unresolved={unresolved} ledger={safe_text(args.ledger)}"
    )
    if unresolved and not args.allow_incomplete:
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
