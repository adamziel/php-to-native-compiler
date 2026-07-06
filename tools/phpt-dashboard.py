#!/usr/bin/env python3
"""Build a live PHPT campaign dashboard from bounded-run artifacts."""

from __future__ import annotations

import argparse
import hashlib
import os
import re
from dataclasses import dataclass, field
from pathlib import Path
from typing import Iterable


KEY_VALUE_RE = re.compile(r"([A-Za-z0-9_-]+)=([^ ]+)")
FAIL_RE = re.compile(r"^(FAIL|BORK|WARN)\s+.*\[(.+?\.phpt)\]\s*$")
SUMMARY_STAMP_RE = re.compile(r"summary-(\d{8}T\d{6}Z-\d+)\.txt$")


@dataclass
class Summary:
    path: Path
    lane: str
    lane_root: Path
    stamp: str
    mtime: float
    commit: str = ""
    manifest: Path | None = None
    runnable_manifest: Path | None = None
    classification_tsv: Path | None = None
    excluded_tsv: Path | None = None
    result: dict[str, str] = field(default_factory=dict)
    buckets: list[dict[str, str]] = field(default_factory=list)
    exclusions_by_category: dict[str, int] = field(default_factory=dict)
    selected_hash: str = ""


@dataclass(frozen=True)
class Failure:
    row: str
    status: str
    summary: Summary
    log: Path


@dataclass(frozen=True)
class Exclusion:
    row: str
    category: str
    reason: str
    summary: Summary


@dataclass(frozen=True)
class RowStatus:
    row: str
    state: str
    summary: Summary
    status: str = ""
    category: str = ""
    reason: str = ""
    log: Path | None = None


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Summarize PHPT bounded-run failures and exclusions."
    )
    parser.add_argument(
        "--root",
        action="append",
        type=Path,
        help="Lane/repo root to scan. Defaults to integration plus ptn-active-lanes.",
    )
    parser.add_argument(
        "--out-dir",
        type=Path,
        default=Path(".runtime/phpt-dashboard"),
        help="Output directory for latest.md and TSV files.",
    )
    parser.add_argument(
        "--all-runs",
        action="store_true",
        help="Do not deduplicate repeated runs with identical selected rows.",
    )
    return parser.parse_args()


def default_roots(cwd: Path) -> list[Path]:
    roots = [cwd]
    active_lanes = Path("/home/claude/ptn-active-lanes")
    if active_lanes.is_dir():
        roots.extend(sorted(path for path in active_lanes.iterdir() if path.is_dir()))
    return roots


def lane_name(root: Path) -> str:
    if root.name == "integration":
        return "integration"
    return root.name


def parse_key_values(line: str) -> dict[str, str]:
    return {match.group(1): match.group(2) for match in KEY_VALUE_RE.finditer(line)}


def hash_file(path: Path | None) -> str:
    if path is None or not path.is_file():
        return ""
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def parse_summary(path: Path, root: Path) -> Summary | None:
    stamp_match = SUMMARY_STAMP_RE.search(path.name)
    if not stamp_match:
        return None
    summary = Summary(
        path=path,
        lane=lane_name(root),
        lane_root=root,
        stamp=stamp_match.group(1),
        mtime=path.stat().st_mtime,
    )

    for raw_line in path.read_text(errors="replace").splitlines():
        line = raw_line.strip()
        if line.startswith("commit: "):
            summary.commit = line.removeprefix("commit: ").strip()
        elif line.startswith("manifest: "):
            summary.manifest = Path(line.removeprefix("manifest: ").strip())
        elif line.startswith("runnable-manifest: "):
            summary.runnable_manifest = Path(
                line.removeprefix("runnable-manifest: ").strip()
            )
        elif line.startswith("classification-files: "):
            values = parse_key_values(line)
            if "classification" in values:
                summary.classification_tsv = Path(values["classification"])
            if "excluded" in values:
                summary.excluded_tsv = Path(values["excluded"])
        elif line.startswith("classification.") and ": rows=" in line:
            category = line.split(":", 1)[0].removeprefix("classification.")
            values = parse_key_values(line)
            if "rows" in values:
                summary.exclusions_by_category[category] = int(values["rows"])
        elif line.startswith("bucket: "):
            values = parse_key_values(line)
            values["bucket"] = line.split()[1]
            summary.buckets.append(values)
        elif line.startswith("result: "):
            summary.result = parse_key_values(line)

    summary.selected_hash = hash_file(summary.manifest)
    return summary


def iter_summaries(roots: Iterable[Path]) -> list[Summary]:
    summaries: list[Summary] = []
    for root in roots:
        runtime_dir = root / ".runtime"
        if not runtime_dir.is_dir():
            continue
        for path in sorted(runtime_dir.glob("**/summary-*.txt")):
            summary = parse_summary(path, root)
            if summary is not None and summary.result:
                summaries.append(summary)
    return summaries


def dedupe_summaries(summaries: list[Summary]) -> list[Summary]:
    by_hash: dict[str, Summary] = {}
    no_hash: list[Summary] = []
    for summary in summaries:
        if not summary.selected_hash:
            no_hash.append(summary)
            continue
        previous = by_hash.get(summary.selected_hash)
        if previous is None or summary.mtime > previous.mtime:
            by_hash[summary.selected_hash] = summary
    return sorted([*by_hash.values(), *no_hash], key=lambda item: item.mtime, reverse=True)


def row_subsystem(row: str) -> str:
    parts = Path(row).parts
    if "Zend" in parts:
        index = parts.index("Zend")
        return "/".join(parts[index : min(index + 3, len(parts))])
    if "tests" in parts and "ext" in parts:
        ext_index = parts.index("ext")
        if ext_index + 1 < len(parts):
            ext = parts[ext_index + 1]
            if ext == "standard" and ext_index + 3 < len(parts):
                return f"ext/standard/{parts[ext_index + 3]}"
            return f"ext/{ext}"
    if "tests" in parts:
        test_index = parts.index("tests")
        return "/".join(parts[test_index : min(test_index + 3, len(parts))])
    return parts[0] if parts else "unknown"


def normalize_row(row: str) -> str:
    php_src = "/home/claude/php-src-phpt/"
    if row.startswith(php_src):
        return row.removeprefix(php_src)
    return row


def row_family(row: str) -> str:
    name = Path(row).stem
    name = re.sub(r"_[0-9]+$", "", name)
    name = re.sub(r"[0-9]+$", "", name)
    name = re.sub(r"_(variation|basic|bug)$", "", name)
    return name or "unknown"


def cluster_for_failure(row: str) -> str:
    return f"fail:{row_subsystem(row)}:{row_family(row)}"


def cluster_for_exclusion(exclusion: Exclusion) -> str:
    return f"exclude:{exclusion.category}:{row_subsystem(exclusion.row)}"


def extract_failures(summary: Summary) -> list[Failure]:
    failures: list[Failure] = []
    seen: set[tuple[str, str, Path]] = set()
    for bucket in summary.buckets:
        log_value = bucket.get("log", "")
        if not log_value:
            continue
        log = Path(log_value)
        if not log.is_file():
            continue
        for raw_line in log.read_text(errors="replace").splitlines():
            line = raw_line.replace("\r", "\n").split("\n")[-1].strip()
            match = FAIL_RE.match(line)
            if not match:
                continue
            status, row = match.group(1), match.group(2)
            key = (status, row, log)
            if key in seen:
                continue
            seen.add(key)
            failures.append(
                Failure(row=normalize_row(row), status=status, summary=summary, log=log)
            )
    return failures


def extract_exclusions(summary: Summary) -> list[Exclusion]:
    path = summary.excluded_tsv
    if path is None or not path.is_file():
        return []
    exclusions: list[Exclusion] = []
    for raw_line in path.read_text(errors="replace").splitlines():
        parts = raw_line.split("\t", 2)
        if len(parts) == 3:
            row, category, reason = parts
        elif len(parts) == 2:
            row, category = parts
            reason = ""
        else:
            continue
        exclusions.append(
            Exclusion(
                row=normalize_row(row),
                category=category,
                reason=reason,
                summary=summary,
            )
        )
    return exclusions


def extract_row_statuses(summary: Summary) -> list[RowStatus]:
    path = summary.classification_tsv
    if path is None or not path.is_file():
        return []

    failures_by_row = {
        normalize_row(failure.row): failure for failure in extract_failures(summary)
    }
    ran_tests = int_result(summary, "tests") > 0
    statuses: list[RowStatus] = []
    for raw_line in path.read_text(errors="replace").splitlines():
        parts = raw_line.split("\t", 2)
        if len(parts) == 3:
            row, category, reason = parts
        elif len(parts) == 2:
            row, category = parts
            reason = ""
        else:
            continue
        row = normalize_row(row)
        failure = failures_by_row.get(row)
        if failure is not None:
            statuses.append(
                RowStatus(
                    row=row,
                    state="failed",
                    summary=summary,
                    status=failure.status,
                    log=failure.log,
                )
            )
        elif category == "runnable":
            statuses.append(
                RowStatus(
                    row=row,
                    state="passed" if ran_tests else "runnable-unrun",
                    summary=summary,
                )
            )
        else:
            statuses.append(
                RowStatus(
                    row=row,
                    state="excluded",
                    summary=summary,
                    category=category,
                    reason=reason,
                )
            )
    return statuses


def current_row_statuses(summaries: list[Summary]) -> dict[str, RowStatus]:
    latest: dict[str, RowStatus] = {}
    for summary in sorted(summaries, key=lambda item: item.mtime):
        for status in extract_row_statuses(summary):
            latest[status.row] = status
    return latest


def active_runs() -> list[str]:
    root = Path(os.environ.get("PTN_DETACHED_CHECK_ROOT", ".runtime/detached-checks"))
    if not root.is_dir():
        return []

    runs: list[tuple[float, str]] = []
    for status_path in root.glob("*/status.tsv"):
        values: dict[str, str] = {}
        try:
            for line in status_path.read_text(encoding="utf-8").splitlines():
                key, sep, value = line.partition("\t")
                if sep:
                    values[key] = value
        except OSError:
            continue
        if values.get("state") != "running":
            continue
        try:
            mtime = status_path.stat().st_mtime
        except OSError:
            mtime = 0.0
        started = values.get("started_at_utc", "")
        runs.append((mtime, f"{status_path.parent.name}\tstarted={started}"))
    return [line for _, line in sorted(runs, reverse=True)]


def write_tsv(path: Path, header: list[str], rows: Iterable[Iterable[object]]) -> None:
    with path.open("w", encoding="utf-8") as handle:
        handle.write("\t".join(header) + "\n")
        for row in rows:
            handle.write("\t".join(str(item) for item in row) + "\n")


def int_result(summary: Summary, key: str) -> int:
    try:
        return int(summary.result.get(key, "0"))
    except ValueError:
        return 0


def build_markdown(
    summaries: list[Summary],
    row_statuses: dict[str, RowStatus],
    clusters: dict[str, dict[str, object]],
    active: list[str],
) -> str:
    states: dict[str, int] = {}
    for status in row_statuses.values():
        states[status.state] = states.get(status.state, 0) + 1
    failures = [
        status for status in row_statuses.values() if status.state == "failed"
    ]
    exclusions = [
        status for status in row_statuses.values() if status.state == "excluded"
    ]

    lines = [
        "# PHPT Campaign Dashboard",
        "",
        "Data source: latest completed bounded-run artifact per selected row.",
        "",
        "## Current Row Status",
        "",
        "| Metric | Count |",
        "| --- | ---: |",
        f"| Rows with observed latest status | {len(row_statuses)} |",
        f"| Passed runnable rows | {states.get('passed', 0)} |",
        f"| Failing runnable rows | {states.get('failed', 0)} |",
        f"| Classified exclusions | {states.get('excluded', 0)} |",
        f"| Runnable but not executed | {states.get('runnable-unrun', 0)} |",
        f"| Distinct completed row sets scanned | {len(summaries)} |",
        "",
    ]

    lines.extend(["## Active Runs", ""])
    if active:
        lines.extend(f"- `{line}`" for line in active[:20])
    else:
        lines.append("- None detected.")
    lines.append("")

    lines.extend(
        [
            "## Failure Clusters",
            "",
            "| Cluster | Rows | Lanes |",
            "| --- | ---: | --- |",
        ]
    )
    for cluster, data in sorted(
        clusters.items(), key=lambda item: (-int(item[1]["count"]), item[0])
    )[:40]:
        lanes = ", ".join(sorted(data["lanes"]))
        lines.append(f"| `{cluster}` | {data['count']} | {lanes} |")
    if not clusters:
        lines.append("| none | 0 | |")
    lines.append("")

    lines.extend(["## Current Failures", ""])
    if failures:
        lines.extend(["| Row | Status | Lane | Log |", "| --- | --- | --- | --- |"])
        for failure in sorted(failures, key=lambda item: item.row)[:80]:
            lines.append(
                f"| `{failure.row}` | {failure.status} | {failure.summary.lane} | "
                f"`{failure.log}` |"
            )
    else:
        lines.append("No failing runnable rows in completed row sets.")
    lines.append("")

    by_category: dict[str, int] = {}
    for exclusion in exclusions:
        by_category[exclusion.category] = by_category.get(exclusion.category, 0) + 1
    lines.extend(["## Exclusions By Category", ""])
    if by_category:
        lines.extend(["| Category | Rows |", "| --- | ---: |"])
        for category, count in sorted(by_category.items(), key=lambda item: (-item[1], item[0])):
            lines.append(f"| `{category}` | {count} |")
    else:
        lines.append("No exclusions in completed row sets.")
    lines.append("")

    lines.extend(
        [
            "## Recent Completed Row Sets",
            "",
            "| Lane | Commit | Selected | Runnable | Passed | Failed | Excluded | Summary |",
            "| --- | --- | ---: | ---: | ---: | ---: | ---: | --- |",
        ]
    )
    for summary in summaries[:40]:
        lines.append(
            f"| {summary.lane} | `{summary.commit}` | {int_result(summary, 'selected')} | "
            f"{int_result(summary, 'runnable')} | {int_result(summary, 'passed')} | "
            f"{int_result(summary, 'failed')} | {int_result(summary, 'excluded')} | "
            f"`{summary.path}` |"
        )
    return "\n".join(lines) + "\n"


def main() -> int:
    args = parse_args()
    cwd = Path.cwd()
    roots = args.root if args.root else default_roots(cwd)
    roots = [root.resolve() for root in roots if root.exists()]

    summaries = iter_summaries(roots)
    if not args.all_runs:
        summaries = dedupe_summaries(summaries)
    else:
        summaries = sorted(summaries, key=lambda item: item.mtime, reverse=True)

    row_statuses = current_row_statuses(summaries)
    failures = [
        status for status in row_statuses.values() if status.state == "failed"
    ]
    exclusions = [
        status for status in row_statuses.values() if status.state == "excluded"
    ]

    clusters: dict[str, dict[str, object]] = {}
    for failure in failures:
        key = cluster_for_failure(failure.row)
        item = clusters.setdefault(key, {"count": 0, "lanes": set()})
        item["count"] = int(item["count"]) + 1
        item["lanes"].add(failure.summary.lane)
    for exclusion in exclusions:
        key = f"exclude:{exclusion.category}:{row_subsystem(exclusion.row)}"
        item = clusters.setdefault(key, {"count": 0, "lanes": set()})
        item["count"] = int(item["count"]) + 1
        item["lanes"].add(exclusion.summary.lane)

    out_dir = args.out_dir
    if not out_dir.is_absolute():
        out_dir = cwd / out_dir
    out_dir.mkdir(parents=True, exist_ok=True)

    write_tsv(
        out_dir / "summaries.tsv",
        ["lane", "commit", "selected", "runnable", "passed", "failed", "excluded", "summary"],
        (
            [
                summary.lane,
                summary.commit,
                int_result(summary, "selected"),
                int_result(summary, "runnable"),
                int_result(summary, "passed"),
                int_result(summary, "failed"),
                int_result(summary, "excluded"),
                summary.path,
            ]
            for summary in summaries
        ),
    )
    write_tsv(
        out_dir / "failures.tsv",
        ["cluster", "row", "status", "lane", "commit", "summary", "log"],
        (
            [
                cluster_for_failure(failure.row),
                failure.row,
                failure.status,
                failure.summary.lane,
                failure.summary.commit,
                failure.summary.path,
                failure.log or "",
            ]
            for failure in failures
        ),
    )
    write_tsv(
        out_dir / "exclusions.tsv",
        ["cluster", "row", "category", "lane", "commit", "reason", "summary"],
        (
            [
                f"exclude:{exclusion.category}:{row_subsystem(exclusion.row)}",
                exclusion.row,
                exclusion.category,
                exclusion.summary.lane,
                exclusion.summary.commit,
                exclusion.reason,
                exclusion.summary.path,
            ]
            for exclusion in exclusions
        ),
    )
    write_tsv(
        out_dir / "clusters.tsv",
        ["cluster", "rows", "lanes"],
        (
            [cluster, data["count"], ",".join(sorted(data["lanes"]))]
            for cluster, data in sorted(
                clusters.items(), key=lambda item: (-int(item[1]["count"]), item[0])
            )
        ),
    )

    markdown = build_markdown(
        summaries=summaries,
        row_statuses=row_statuses,
        clusters=clusters,
        active=active_runs(),
    )
    (out_dir / "latest.md").write_text(markdown, encoding="utf-8")

    print(out_dir / "latest.md")
    print(out_dir / "failures.tsv")
    print(out_dir / "exclusions.tsv")
    print(out_dir / "clusters.tsv")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
