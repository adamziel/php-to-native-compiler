#!/usr/bin/env python3
"""Record unreported PHPT extension-suite rows as explicit quarantined skips."""

from __future__ import annotations

import argparse
from pathlib import Path


HEADER = (
    "index\trows\tfirst_row\tlast_row\tstate\texit_code\telapsed_ms\ttests\tpassed\tfailed\t"
    "skipped\twarned\ttimed_out\tcrashed"
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--run-dir", type=Path, required=True)
    parser.add_argument("--extension", action="append", required=True)
    parser.add_argument("--reason", default="quarantined after repeated non-reporting recovery")
    return parser.parse_args()


def atomic_write(path: Path, content: str) -> None:
    temporary = path.with_name(f".{path.name}.tmp")
    temporary.write_text(content, encoding="utf-8")
    temporary.replace(path)


def manifest_rows(path: Path) -> list[str]:
    rows: list[str] = []
    for raw in path.read_text(encoding="utf-8").splitlines():
        row = raw.split("#", 1)[0].strip()
        if row:
            rows.append(row)
    return rows


def load_results(path: Path) -> dict[str, list[str]]:
    results: dict[str, list[str]] = {}
    if not path.is_file():
        return results
    for raw in path.read_text(encoding="utf-8").splitlines()[1:]:
        values = raw.split("\t")
        if len(values) == 14:
            results[values[2]] = values
    return results


def shard_directories(run_dir: Path) -> list[Path]:
    plan = run_dir / "shards" / "plan.tsv"
    directories: list[Path] = []
    for raw in plan.read_text(encoding="utf-8").splitlines()[1:]:
        values = raw.split("\t")
        if len(values) != 3:
            continue
        directories.append((run_dir / values[2]).parent)
    return directories


def main() -> int:
    args = parse_args()
    run_dir = args.run_dir.resolve()
    extensions = tuple(sorted({name.strip().lower() for name in args.extension if name.strip()}))
    if not extensions:
        raise SystemExit("at least one non-empty --extension is required")
    ledger_path = run_dir / "quarantined-tests.tsv"
    ledger_lines = ["extension\trow\treason"]
    known_ledger_rows: set[str] = set()
    if ledger_path.is_file():
        ledger_lines = ledger_path.read_text(encoding="utf-8").splitlines()
        known_ledger_rows = {line.split("\t", 2)[1] for line in ledger_lines[1:] if "\t" in line}

    totals = {extension: 0 for extension in extensions}
    for shard_dir in shard_directories(run_dir):
        rows = manifest_rows(shard_dir / "manifest.txt")
        results_path = shard_dir / "row-results.tsv"
        results = load_results(results_path)
        added = 0
        for index, row in enumerate(rows):
            extension = next(
                (name for name in extensions if row.startswith(f"ext/{name}/tests/")), None
            )
            if extension is None or row in results:
                continue
            results[row] = [
                str(index),
                "1",
                row,
                row,
                "skipped",
                "0",
                "0",
                "1",
                "0",
                "0",
                "1",
                "0",
                "0",
                "0",
            ]
            if row not in known_ledger_rows:
                ledger_lines.append(f"{extension}\t{row}\t{args.reason}")
                known_ledger_rows.add(row)
            totals[extension] += 1
            added += 1
        if added:
            ordered = sorted(results.values(), key=lambda values: int(values[0]))
            atomic_write(results_path, HEADER + "\n" + "\n".join("\t".join(row) for row in ordered) + "\n")

    atomic_write(ledger_path, "\n".join(ledger_lines) + "\n")
    for extension in extensions:
        print(f"extension={extension} quarantined={totals[extension]}")
    print(f"total_quarantined={sum(totals.values())}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
