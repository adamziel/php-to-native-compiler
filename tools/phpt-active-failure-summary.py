#!/usr/bin/env python3
"""Summarize active PHPT dashboard failures without scanning shard logs."""

from __future__ import annotations

import argparse
from collections import Counter
from pathlib import Path


DEFAULT_ROOT = Path("/home/claude/.local/state/ptn-full-phpt-dashboard-loop")
FAILISH = {"FAIL", "BORK", "WARN"}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--root",
        type=Path,
        default=DEFAULT_ROOT,
        help="Rolling PHPT dashboard state directory.",
    )
    parser.add_argument(
        "--statuses",
        type=Path,
        help="Specific partial-statuses TSV to summarize.",
    )
    parser.add_argument(
        "--clusters",
        type=int,
        default=25,
        help="Maximum failure clusters to print.",
    )
    parser.add_argument(
        "--samples",
        type=int,
        default=50,
        help="Maximum failish rows to print.",
    )
    return parser.parse_args()


def latest_statuses(root: Path) -> Path:
    statuses = sorted(root.glob("partial-statuses-*.tsv"))
    if not statuses:
        raise SystemExit(f"no partial-statuses TSV found under {root}")
    return statuses[-1]


def read_dashboard(root: Path) -> dict[str, str]:
    latest = root / "latest.tsv"
    if not latest.exists():
        return {}
    rows: dict[str, str] = {}
    for line in latest.read_text(errors="replace").splitlines():
        parts = line.split("\t", 1)
        if len(parts) == 2:
            rows[parts[0]] = parts[1]
    return rows


def normalize_path(path: str) -> str:
    prefixes = (
        "/home/claude/php-src-phpt/",
        "/home/claude/ptn-lanes/integration/.runtime/php-src-phpt/",
        ".runtime/php-src-phpt/",
    )
    for prefix in prefixes:
        if path.startswith(prefix):
            return path[len(prefix) :]
    return path


def bucket(row: str) -> str:
    parts = row.split("/")
    if len(parts) >= 3 and parts[0] in {"Zend", "ext"}:
        return "/".join(parts[:3])
    if len(parts) >= 2:
        return "/".join(parts[:2])
    return row


def main() -> int:
    args = parse_args()
    statuses_path = args.statuses or latest_statuses(args.root)
    counts: Counter[str] = Counter()
    clusters: Counter[str] = Counter()
    failish: list[tuple[str, str]] = []

    for line in statuses_path.read_text(errors="replace").splitlines():
        parts = line.split("\t")
        if len(parts) < 2:
            continue
        status = parts[0]
        row = normalize_path(parts[1])
        counts[status] += 1
        if status in FAILISH:
            failish.append((status, row))
            clusters[bucket(row)] += 1

    dashboard = read_dashboard(args.root)
    print(f"statuses_file={statuses_path}")
    for key in (
        "refreshed_at_utc",
        "source_commit",
        "active_source_commit",
        "active_tests",
        "active_passed",
        "active_failed",
        "active_skipped",
        "active_warned",
        "active_unknown",
    ):
        if key in dashboard:
            print(f"{key}={dashboard[key]}")

    print("\nstatus_counts")
    for status, count in counts.most_common():
        print(f"{status}\t{count}")

    print(f"\nfailish_rows\t{len(failish)}")
    print("top_failish_clusters")
    for name, count in clusters.most_common(max(args.clusters, 0)):
        print(f"{count}\t{name}")

    print("\nsample_failish_rows")
    for status, row in failish[: max(args.samples, 0)]:
        print(f"{status}\t{row}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
