#!/usr/bin/env python3
"""Report the active PHPT partial-run scoreboard without mixing data sources."""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from collections import Counter
from dataclasses import asdict, dataclass, field
from pathlib import Path
from typing import Iterable


ANSI_RE = re.compile(r"\x1b\[[0-9;]*m")
FAIL_RE = re.compile(r"\b(FAIL|BORK|WARN)\b.*\[(.+?\.phpt)\]")
STATUS_RE = re.compile(
    r"^(PASS|FAIL|SKIP|XFAIL|BORK|WARN|LEAK|XLEAK|REDIRECT)\s+.*?(?:\[(.+?\.phpt)\])?\s*$"
)
TEST_RE = re.compile(r"^TEST\s+\d+/\d+\s+\[(.+?\.phpt)\]")
KEY_VALUE_RE = re.compile(r"([A-Za-z0-9_-]+)=([^ ]+)")
SHARD_RE = re.compile(r"(?:^|[/_-])shard[-_/]?(\d+)(?:\D|$)")
SUMMARY_STAMP_RE = re.compile(r"summary-(\d{8}T\d{6}Z-\d+)\.txt$")


@dataclass
class Counts:
    selected: int = 0
    runnable: int = 0
    excluded: int = 0
    executed: int = 0
    passed: int = 0
    failed: int = 0
    skipped: int = 0
    warned: int = 0


@dataclass
class SourceReport:
    name: str
    freshness: str
    commit: str
    classifier_mode: str
    source_path: str
    counts: Counts
    corpus_revision: str = ""
    run_path: str = ""
    top_exclusion_categories: list[tuple[str, int]] = field(default_factory=list)
    top_failing_path_clusters: list[tuple[str, int]] = field(default_factory=list)
    notes: list[str] = field(default_factory=list)


@dataclass
class ShardSummary:
    path: Path
    shard: int | None = None
    commit: str = ""
    corpus_revision: str = ""
    classification_enabled: str = ""
    counts: Counts = field(default_factory=Counts)
    run_logs: list[Path] = field(default_factory=list)
    exclusions_by_category: Counter[str] = field(default_factory=Counter)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Summarize the active local PHPT partial run used for scheduling. "
            "Fresh artifact and rolling aggregate views are opt-in."
        )
    )
    parser.add_argument(
        "sources",
        nargs="*",
        type=Path,
        help="Auto-detected fresh artifact roots or rolling dashboard paths.",
    )
    parser.add_argument(
        "--fresh",
        action="append",
        type=Path,
        default=[],
        help="Fresh CI full-corpus artifact root, summary.json, or shard summary.",
    )
    parser.add_argument(
        "--rolling",
        action="append",
        type=Path,
        default=[],
        help="Rolling dashboard state dir, latest.tsv, or partial summary/status file.",
    )
    parser.add_argument(
        "--include-rolling-summary",
        action="store_true",
        help="Also print the rolling aggregate. Default is active partial run only.",
    )
    parser.add_argument(
        "--top",
        type=int,
        default=10,
        help="Number of exclusion/failure clusters to display.",
    )
    parser.add_argument(
        "--json-out",
        type=Path,
        help="Optional path for machine-readable JSON output.",
    )
    parser.add_argument(
        "--format",
        choices=("markdown", "json"),
        default="markdown",
        help="Output format printed to stdout.",
    )
    return parser.parse_args()


def parse_key_values(line: str) -> dict[str, str]:
    return {match.group(1): match.group(2) for match in KEY_VALUE_RE.finditer(line)}


def int_value(values: dict[str, str], key: str) -> int:
    value = values.get(key, "0")
    return int(value) if value.isdigit() else 0


def infer_shard(path: Path) -> int | None:
    for part in reversed(path.parts):
        match = SHARD_RE.search(part)
        if match:
            return int(match.group(1))
    return None


def latest_by_shard(paths: Iterable[Path]) -> list[Path]:
    by_shard: dict[int, Path] = {}
    unassigned: list[Path] = []
    for path in paths:
        shard = infer_shard(path)
        if shard is None:
            unassigned.append(path)
            continue
        previous = by_shard.get(shard)
        if previous is None or str(path) > str(previous):
            by_shard[shard] = path
    return [by_shard[index] for index in sorted(by_shard)] + sorted(unassigned)


def read_key_value_tsv(path: Path) -> dict[str, str]:
    values: dict[str, str] = {}
    for raw_line in path.read_text(errors="replace").splitlines():
        parts = raw_line.split("\t", 1)
        if len(parts) == 2:
            values[parts[0]] = parts[1]
    return values


def current_repo_head(path: Path) -> str:
    repo = path if path.is_dir() else path.parent
    try:
        result = subprocess.run(
            ["git", "-C", str(repo), "rev-parse", "--short=12", "HEAD"],
            check=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
            text=True,
        )
    except (OSError, subprocess.CalledProcessError):
        return ""
    return result.stdout.strip()


def commit_matches_head(commit: str, head: str) -> bool:
    if not commit or not head:
        return True
    return commit == head or commit.startswith(head) or head.startswith(commit)


def normalize_row(row: str) -> str:
    value = ANSI_RE.sub("", row).replace("\\", "/")
    for marker in (".runtime/php-src-phpt/", "php-src-phpt/"):
        if marker in value:
            return value.split(marker, 1)[1]
    return value.removeprefix("./")


def row_subsystem(row: str) -> str:
    parts = Path(normalize_row(row)).parts
    if not parts:
        return "unknown"
    if parts[0] == "Zend":
        return "/".join(parts[: min(3, len(parts))])
    if len(parts) >= 4 and parts[0] == "ext":
        ext = parts[1]
        if ext == "standard" and len(parts) >= 4:
            return f"ext/standard/{parts[3]}"
        return f"ext/{ext}"
    if parts[0] == "tests":
        return "/".join(parts[: min(3, len(parts))])
    return parts[0]


def failing_clusters_from_log(path: Path) -> Counter[str]:
    clusters: Counter[str] = Counter()
    if not path.is_file():
        return clusters
    seen: set[tuple[str, str]] = set()
    for raw_line in path.read_text(errors="replace").splitlines():
        for part in raw_line.replace("\r", "\n").split("\n"):
            line = ANSI_RE.sub("", part).strip()
            match = FAIL_RE.search(line)
            if not match:
                continue
            status, row = match.group(1), normalize_row(match.group(2))
            key = (status, row)
            if key in seen:
                continue
            seen.add(key)
            clusters[row_subsystem(row)] += 1
    return clusters


def failing_clusters_from_status_tsv(path: Path) -> Counter[str]:
    clusters: Counter[str] = Counter()
    if not path.is_file():
        return clusters
    for raw_line in path.read_text(errors="replace").splitlines():
        parts = raw_line.split("\t", 1)
        if len(parts) != 2:
            continue
        status, row = parts
        if status in {"FAIL", "BORK", "WARN"}:
            clusters[row_subsystem(row)] += 1
    return clusters


def statuses_from_run_log(path: Path) -> dict[str, str]:
    statuses: dict[str, str] = {}
    if not path.is_file():
        return statuses

    current = ""
    for raw_line in path.read_text(errors="replace").splitlines():
        for part in raw_line.replace("\r", "\n").split("\n"):
            line = ANSI_RE.sub("", part).strip()
            test_match = TEST_RE.search(line)
            if test_match:
                current = normalize_row(test_match.group(1))
                continue

            status_match = STATUS_RE.search(line)
            if status_match:
                status = status_match.group(1)
                row = normalize_row(status_match.group(2) or current)
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


def active_run_statuses(run_dir: Path) -> dict[str, str]:
    statuses: dict[str, str] = {}
    if not run_dir.is_dir():
        return statuses

    candidates: list[Path] = []
    candidates.extend(sorted((run_dir / "shards").glob("shard-*/run.log")))
    candidates.extend(sorted((run_dir / "shards").glob("shard-*/batches/batch-*")))
    candidates.sort(key=lambda item: (item.stat().st_mtime, str(item)))
    for path in candidates:
        if path.is_file():
            statuses.update(statuses_from_run_log(path))
    return statuses


def report_from_active_run(
    run_dir: Path,
    latest_values: dict[str, str],
    summary_values: dict[str, str],
    top: int,
) -> SourceReport | None:
    if not run_dir.is_dir():
        return None

    statuses = active_run_statuses(run_dir)
    if not statuses:
        return None

    status_counts = Counter(statuses.values())
    failures: Counter[str] = Counter()
    for row, status in statuses.items():
        if status in {"FAIL", "BORK", "WARN", "LEAK", "XLEAK"}:
            failures[row_subsystem(row)] += 1

    selected = int(latest_values.get("selected") or summary_values.get("corpus") or 0)
    commit = (
        latest_values.get("active_source_commit")
        or latest_values.get("source_commit")
        or summary_values.get("source_commit", "")
    )
    counts = Counts(
        selected=selected,
        runnable=selected,
        excluded=0,
        executed=len(statuses),
        passed=status_counts["PASS"],
        failed=(
            status_counts["FAIL"]
            + status_counts["BORK"]
            + status_counts["LEAK"]
            + status_counts["XLEAK"]
        ),
        skipped=status_counts["SKIP"] + status_counts["XFAIL"],
        warned=status_counts["WARN"],
    )

    notes = [
        "partial live view; do not compare as a completed corpus result",
        f"unknown rows: {max(selected - len(statuses), 0)}",
    ]
    repo_head = current_repo_head(Path.cwd())
    if repo_head and commit and not commit_matches_head(commit, repo_head):
        notes.append(
            f"stale vs current HEAD {repo_head}; use as scheduling input, "
            "then verify suspected fixes before counting them retired"
        )

    return SourceReport(
        name=f"{run_dir.parent.parent.name}-active",
        freshness="live-active-run-partial",
        commit=commit,
        corpus_revision=summary_values.get("php_src_revision", ""),
        classifier_mode="active run logs; incomplete until all shards finish",
        source_path=str(run_dir),
        run_path=str(run_dir),
        counts=counts,
        top_exclusion_categories=[],
        top_failing_path_clusters=failures.most_common(top),
        notes=notes,
    )


def resolve_artifact_sibling(summary_path: Path, recorded_path: str) -> Path:
    recorded = Path(recorded_path)
    if recorded.is_file():
        return recorded
    sibling = summary_path.parent / recorded.name
    return sibling if sibling.is_file() else recorded


def parse_shard_summary(path: Path) -> ShardSummary:
    summary = ShardSummary(path=path, shard=infer_shard(path))
    for raw_line in path.read_text(errors="replace").splitlines():
        line = raw_line.strip()
        if line.startswith("commit: "):
            summary.commit = line.removeprefix("commit: ").strip()
        elif line.startswith("corpus-revision: "):
            summary.corpus_revision = line.removeprefix("corpus-revision: ").strip()
        elif line.startswith("classification: "):
            values = parse_key_values(line)
            summary.classification_enabled = values.get("enabled", "")
            summary.counts.selected = int_value(values, "selected")
            summary.counts.runnable = int_value(values, "runnable")
            summary.counts.excluded = int_value(values, "excluded")
        elif line.startswith("classification.") and ": rows=" in line:
            category = line.split(":", 1)[0].removeprefix("classification.")
            values = parse_key_values(line)
            summary.exclusions_by_category[category] += int_value(values, "rows")
        elif line.startswith("bucket: "):
            values = parse_key_values(line)
            log_value = values.get("log", "")
            if log_value:
                summary.run_logs.append(resolve_artifact_sibling(path, log_value))
        elif line.startswith("result: "):
            values = parse_key_values(line)
            summary.counts.selected = int_value(values, "selected")
            summary.counts.runnable = int_value(values, "runnable")
            summary.counts.excluded = int_value(values, "excluded")
            summary.counts.executed = int_value(values, "tests")
            summary.counts.passed = int_value(values, "passed")
            summary.counts.failed = int_value(values, "failed")
            summary.counts.skipped = int_value(values, "skipped")
            summary.counts.warned = int_value(values, "warned")

    if not summary.exclusions_by_category:
        for excluded_tsv in path.parent.glob("excluded-*.tsv"):
            for raw_line in excluded_tsv.read_text(errors="replace").splitlines():
                parts = raw_line.split("\t", 2)
                if len(parts) >= 2:
                    summary.exclusions_by_category[parts[1]] += 1
    return summary


def find_fresh_summaries(path: Path) -> list[Path]:
    if path.is_file() and SUMMARY_STAMP_RE.match(path.name):
        return [path]
    if not path.is_dir():
        return []
    return latest_by_shard(path.rglob("summary-*.txt"))


def report_from_composed_json(path: Path, top: int) -> SourceReport:
    payload = json.loads(path.read_text())
    totals = payload.get("totals", {})
    commits = payload.get("commits", {})
    revisions = payload.get("corpus_revisions", {})
    exclusions = Counter(payload.get("exclusion_categories", {}))
    failures = Counter(payload.get("failure_clusters", {}))
    commit = "mixed"
    if len(commits) == 1:
        commit = next(iter(commits))
    elif not commits:
        commit = ""
    revision = ""
    if len(revisions) == 1:
        revision = next(iter(revisions))
    report = SourceReport(
        name=path.parent.name or path.name,
        freshness="fresh-ci-corpus",
        commit=commit,
        corpus_revision=revision,
        classifier_mode="composed-summary",
        source_path=str(path),
        counts=Counts(
            selected=int(totals.get("selected", 0)),
            runnable=int(totals.get("runnable", 0)),
            excluded=int(totals.get("excluded", 0)),
            executed=int(totals.get("tests", 0)),
            passed=int(totals.get("passed", 0)),
            failed=int(totals.get("failed", 0)),
            skipped=int(totals.get("skipped", 0)),
            warned=int(totals.get("warned", 0)),
        ),
        top_exclusion_categories=exclusions.most_common(top),
        top_failing_path_clusters=failures.most_common(top),
    )
    if not exclusions:
        report.notes.append("composed summary has no exclusion category detail")
    if not failures:
        report.notes.append("composed summary has no failing path cluster detail")
    return report


def report_from_fresh(path: Path, top: int) -> SourceReport:
    if path.is_file() and path.name == "summary.json":
        return report_from_composed_json(path, top)
    if path.is_dir() and (path / "summary.json").is_file():
        summaries = list(path.rglob("summary-*.txt"))
        if not summaries:
            return report_from_composed_json(path / "summary.json", top)

    summaries = [parse_shard_summary(item) for item in find_fresh_summaries(path)]
    counts = Counts()
    commits: Counter[str] = Counter()
    revisions: Counter[str] = Counter()
    classifier_modes: Counter[str] = Counter()
    exclusions: Counter[str] = Counter()
    failures: Counter[str] = Counter()

    for summary in summaries:
        counts.selected += summary.counts.selected
        counts.runnable += summary.counts.runnable
        counts.excluded += summary.counts.excluded
        counts.executed += summary.counts.executed
        counts.passed += summary.counts.passed
        counts.failed += summary.counts.failed
        counts.skipped += summary.counts.skipped
        counts.warned += summary.counts.warned
        if summary.commit:
            commits[summary.commit] += 1
        if summary.corpus_revision:
            revisions[summary.corpus_revision] += 1
        classifier_modes[summary.classification_enabled or "unknown"] += 1
        exclusions.update(summary.exclusions_by_category)
        for log in summary.run_logs:
            failures.update(failing_clusters_from_log(log))

    commit = "mixed"
    if len(commits) == 1:
        commit = next(iter(commits))
    elif not commits:
        commit = ""
    revision = ""
    if len(revisions) == 1:
        revision = next(iter(revisions))
    classifier_mode = ", ".join(
        f"enabled={mode} ({count} shard(s))"
        for mode, count in sorted(classifier_modes.items())
    )
    if any(category.startswith("harness-") for category in exclusions):
        classifier_mode += "; harness programs classified"

    report = SourceReport(
        name=path.name or str(path),
        freshness="fresh-ci-corpus",
        commit=commit,
        corpus_revision=revision,
        classifier_mode=classifier_mode or "unknown",
        source_path=str(path),
        counts=counts,
        top_exclusion_categories=exclusions.most_common(top),
        top_failing_path_clusters=failures.most_common(top),
    )
    if not summaries:
        report.notes.append("no shard summary-*.txt files found")
    if counts.failed and not failures:
        report.notes.append("failed count is present, but no local run logs were found")
    return report


def newest_partial_summary(root: Path) -> Path | None:
    summaries = sorted(root.glob("partial-summary-*.txt"))
    return summaries[-1] if summaries else None


def rolling_summary_values(path: Path) -> dict[str, str]:
    values: dict[str, str] = {}
    for raw_line in path.read_text(errors="replace").splitlines():
        line = raw_line.strip()
        if line.startswith("source: "):
            values["source_commit"] = line.removeprefix("source: ").strip()
        elif line.startswith("php-src revision: "):
            values["php_src_revision"] = line.removeprefix("php-src revision: ").strip()
        elif line.startswith("runs-dir: "):
            values["runs_dir"] = line.removeprefix("runs-dir: ").strip()
        elif line.startswith("dedup-statuses: "):
            values["dedup_statuses"] = line.removeprefix("dedup-statuses: ").strip()
        elif line.startswith("statused: "):
            values.update(parse_key_values(line))
    return values


def reports_from_rolling(
    path: Path, top: int, include_rolling_summary: bool = False
) -> list[SourceReport]:
    source_path = path
    latest_values: dict[str, str] = {}
    summary_path: Path | None = None
    status_path: Path | None = None

    if path.is_dir():
        latest = path / "latest.tsv"
        if latest.is_file():
            latest_values = read_key_value_tsv(latest)
            source_path = latest
        summary_value = latest_values.get("summary", "")
        summary_path = Path(summary_value) if summary_value else newest_partial_summary(path)
    elif path.name == "latest.tsv":
        latest_values = read_key_value_tsv(path)
        summary_value = latest_values.get("summary", "")
        summary_path = Path(summary_value) if summary_value else None
    elif path.name.startswith("partial-summary-"):
        summary_path = path
    elif path.name.startswith("partial-statuses-"):
        status_path = path

    summary_values: dict[str, str] = {}
    if summary_path is not None and summary_path.is_file():
        summary_values = rolling_summary_values(summary_path)
        status_value = summary_values.get("dedup_statuses", "")
        if status_value:
            status_path = Path(status_value)
    if status_path is None and path.is_dir():
        statuses = sorted(path.glob("partial-statuses-*.tsv"))
        status_path = statuses[-1] if statuses else None

    selected = int(latest_values.get("selected") or summary_values.get("corpus") or 0)
    runnable = int(latest_values.get("runnable") or summary_values.get("tests") or selected)
    executed = int(latest_values.get("tests") or summary_values.get("tests") or 0)
    counts = Counts(
        selected=selected,
        runnable=runnable,
        excluded=int(latest_values.get("excluded", 0)),
        executed=executed,
        passed=int(latest_values.get("passed") or summary_values.get("passed") or 0),
        failed=int(latest_values.get("failed") or summary_values.get("failed") or 0),
        skipped=int(latest_values.get("skipped") or summary_values.get("skipped") or 0),
        warned=int(latest_values.get("warned") or summary_values.get("warned") or 0),
    )

    commit = latest_values.get("source_commit") or summary_values.get("source_commit", "")
    corpus_revision = summary_values.get("php_src_revision", "")
    run_path = latest_values.get("active_run") or summary_values.get("runs_dir", "")
    failures = failing_clusters_from_status_tsv(status_path) if status_path else Counter()

    notes: list[str] = []
    if status_path is None:
        notes.append("no partial-statuses TSV found for failing path clusters")
    repo_head = current_repo_head(Path.cwd())
    if repo_head and commit and not commit_matches_head(commit, repo_head):
        notes.append(
            f"stale vs current HEAD {repo_head}; use as scheduling input, "
            "then verify suspected fixes before counting them retired"
        )
    if latest_values.get("active_source_commit"):
        notes.append(f"active source commit: {latest_values['active_source_commit']}")
        active_commit = latest_values["active_source_commit"]
        if repo_head and not commit_matches_head(active_commit, repo_head):
            notes.append(f"active run stale vs current HEAD {repo_head}")
    if latest_values.get("active_tests"):
        notes.append(f"active run observed tests: {latest_values['active_tests']}")

    rolling_report = SourceReport(
        name=path.name or str(path),
        freshness="rolling-dashboard",
        commit=commit,
        corpus_revision=corpus_revision,
        classifier_mode="rolling status feed; exclusions not classified",
        source_path=str(source_path),
        run_path=run_path,
        counts=counts,
        top_exclusion_categories=[],
        top_failing_path_clusters=failures.most_common(top),
        notes=notes,
    )

    reports: list[SourceReport] = []

    active_run_value = latest_values.get("active_run", "")
    if active_run_value:
        active_report = report_from_active_run(
            Path(active_run_value), latest_values, summary_values, top
        )
        if active_report is not None:
            reports.append(active_report)
    if include_rolling_summary or not reports:
        reports.insert(0, rolling_report)
    return reports


def detect_source_kind(path: Path) -> str:
    if path.is_dir():
        if (path / "latest.tsv").is_file() or newest_partial_summary(path) is not None:
            return "rolling"
        if any(path.rglob("summary-*.txt")) or (path / "summary.json").is_file():
            return "fresh"
    if path.is_file():
        if path.name == "latest.tsv" or path.name.startswith("partial-summary-"):
            return "rolling"
        if path.name.startswith("partial-statuses-"):
            return "rolling"
        if path.name == "summary.json" or SUMMARY_STAMP_RE.match(path.name):
            return "fresh"
    raise ValueError(f"cannot auto-detect PHPT scoreboard source kind: {path}")


def render_list(rows: list[tuple[str, int]]) -> str:
    if not rows:
        return "none"
    return ", ".join(f"{name}={count}" for name, count in rows)


def render_markdown(reports: list[SourceReport]) -> str:
    lines = [
        "# PHPT Active Partial Scoreboard",
        "",
        "| Source | State | Commit | Classifier mode | Selected | Runnable | Excluded | Executed | Pass | Fail | Skip | Warn | Source path |",
        "| --- | --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |",
    ]
    for report in reports:
        counts = report.counts
        lines.append(
            f"| {report.name} | {report.freshness} | `{report.commit}` | "
            f"{report.classifier_mode} | {counts.selected} | {counts.runnable} | "
            f"{counts.excluded} | {counts.executed} | {counts.passed} | "
            f"{counts.failed} | {counts.skipped} | {counts.warned} | "
            f"`{report.source_path}` |"
        )
    lines.append("")

    for report in reports:
        lines.extend(
            [
                f"## {report.name}",
                "",
                f"- state: {report.freshness}",
                f"- commit: `{report.commit}`",
                f"- php-src revision: `{report.corpus_revision or 'unknown'}`",
                f"- classifier mode: {report.classifier_mode}",
                f"- source artifact/run path: `{report.run_path or report.source_path}`",
                f"- top exclusion categories: {render_list(report.top_exclusion_categories)}",
                f"- top failing path clusters: {render_list(report.top_failing_path_clusters)}",
            ]
        )
        if report.notes:
            lines.append(f"- notes: {'; '.join(report.notes)}")
        lines.append("")
    return "\n".join(lines)


def report_to_json(report: SourceReport) -> dict[str, object]:
    payload = asdict(report)
    payload["top_exclusion_categories"] = [
        {"category": name, "count": count}
        for name, count in report.top_exclusion_categories
    ]
    payload["top_failing_path_clusters"] = [
        {"cluster": name, "count": count}
        for name, count in report.top_failing_path_clusters
    ]
    return payload


def main() -> int:
    args = parse_args()
    requested: list[tuple[str, Path]] = []
    requested.extend(("fresh", path) for path in args.fresh)
    requested.extend(("rolling", path) for path in args.rolling)
    requested.extend(("auto", path) for path in args.sources)
    if not requested:
        default_rolling = Path("/home/claude/.local/state/ptn-full-phpt-dashboard-loop")
        if default_rolling.exists():
            requested.append(("rolling", default_rolling))
        else:
            print("no sources supplied and default rolling dashboard state was not found", file=sys.stderr)
            return 2

    reports: list[SourceReport] = []
    for kind, raw_path in requested:
        path = raw_path.expanduser()
        if kind == "auto":
            kind = detect_source_kind(path)
        if kind == "fresh":
            reports.append(report_from_fresh(path, args.top))
        elif kind == "rolling":
            reports.extend(
                reports_from_rolling(path, args.top, args.include_rolling_summary)
            )
        else:
            raise AssertionError(kind)

    json_payload = {"sources": [report_to_json(report) for report in reports]}
    if args.json_out:
        args.json_out.write_text(json.dumps(json_payload, indent=2) + "\n")

    if args.format == "json":
        print(json.dumps(json_payload, indent=2))
    else:
        print(render_markdown(reports))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
