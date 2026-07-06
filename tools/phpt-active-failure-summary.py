#!/usr/bin/env python3
"""Summarize active PHPT partial-run failures from the active run logs."""

from __future__ import annotations

import argparse
import re
from collections import Counter
from pathlib import Path


DEFAULT_ROOT = Path("/home/claude/.local/state/ptn-full-phpt-dashboard-loop")
FAILISH = {"FAIL", "BORK", "WARN"}
ANSI_RE = re.compile(r"\x1b\[[0-9;]*m")
STATUS_RE = re.compile(
    r"^(PASS|FAIL|SKIP|XFAIL|BORK|WARN|LEAK|XLEAK|REDIRECT)\s+.*?(?:\[(.+?\.phpt)\])?\s*$"
)
TEST_RE = re.compile(r"^TEST\s+\d+/\d+\s+\[(.+?\.phpt)\]")


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
        help="Specific partial-statuses TSV to summarize instead of active run logs.",
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


def active_run_from_dashboard(root: Path) -> Path:
    dashboard = read_dashboard(root)
    active_run = dashboard.get("active_run", "")
    if not active_run:
        raise SystemExit(f"no active_run entry found in {root / 'latest.tsv'}")
    path = Path(active_run)
    if not path.is_dir():
        raise SystemExit(f"active_run does not exist: {path}")
    return path


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


def statuses_from_run_log(path: Path) -> dict[str, str]:
    statuses: dict[str, str] = {}
    current = ""
    for raw_line in path.read_text(errors="replace").splitlines():
        for part in raw_line.replace("\r", "\n").split("\n"):
            line = ANSI_RE.sub("", part).strip()
            test_match = TEST_RE.search(line)
            if test_match:
                current = normalize_path(test_match.group(1))
                continue

            status_match = STATUS_RE.search(line)
            if status_match:
                status = status_match.group(1)
                row = normalize_path(status_match.group(2) or current)
                if row:
                    statuses[row] = status
                if row == current:
                    current = ""
                continue

            if (
                current
                and (
                    "Allowed memory size" in line
                    or "died unexpectedly" in line
                    or "Fatal error" in line
                    or line.startswith("ERROR:")
                )
            ):
                statuses[current] = "FAIL"
                current = ""
    return statuses


def active_statuses(run_dir: Path) -> dict[str, str]:
    statuses: dict[str, str] = {}
    candidates: list[Path] = []
    candidates.extend(sorted((run_dir / "shards").glob("shard-*/run.log")))
    candidates.extend(sorted((run_dir / "shards").glob("shard-*/batches/batch-*")))
    candidates.sort(key=lambda item: (item.stat().st_mtime, str(item)))
    for path in candidates:
        if path.is_file():
            statuses.update(statuses_from_run_log(path))
    return statuses


def tsv_statuses(statuses_path: Path) -> dict[str, str]:
    statuses: dict[str, str] = {}
    for line in statuses_path.read_text(errors="replace").splitlines():
        parts = line.split("\t")
        if len(parts) >= 2:
            statuses[normalize_path(parts[1])] = parts[0]
    return statuses


def main() -> int:
    args = parse_args()
    dashboard = read_dashboard(args.root)
    source_path: Path
    source_label: str
    if args.statuses:
        source_path = args.statuses
        source_label = "partial_statuses_tsv"
        statuses = tsv_statuses(source_path)
    else:
        source_path = active_run_from_dashboard(args.root)
        source_label = "active_run_logs"
        statuses = active_statuses(source_path)

    counts: Counter[str] = Counter()
    clusters: Counter[str] = Counter()
    failish: list[tuple[str, str]] = []

    for row, status in statuses.items():
        counts[status] += 1
        if status in FAILISH:
            failish.append((status, row))
            clusters[bucket(row)] += 1

    print(f"source_kind={source_label}")
    print(f"source_path={source_path}")
    print(f"observed_tests={len(statuses)}")
    print(f"observed_passed={counts['PASS']}")
    print(f"observed_failed={counts['FAIL'] + counts['BORK']}")
    print(f"observed_skipped={counts['SKIP'] + counts['XFAIL']}")
    print(f"observed_warned={counts['WARN']}")
    print(f"observed_unknown={max(int(dashboard.get('selected', '0') or '0') - len(statuses), 0)}")
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
            print(f"dashboard_snapshot_{key}={dashboard[key]}")

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
