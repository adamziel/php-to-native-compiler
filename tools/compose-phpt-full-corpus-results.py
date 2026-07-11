#!/usr/bin/env python3
"""Compose 15-way full-corpus PHPT shard artifacts into one status number."""

from __future__ import annotations

import argparse
import json
import re
import sys
from collections import Counter
from dataclasses import dataclass, field
from pathlib import Path


ANSI_RE = re.compile(r"\x1b\[[0-9;]*m")
FAIL_RE = re.compile(r"\b(FAIL|BORK|WARN)\b.*\[(.+?\.phpt)\]")
KEY_VALUE_RE = re.compile(r"([A-Za-z0-9_-]+)=([^ ]+)")
SHARD_RE = re.compile(r"(?:^|[/_-])shard[-_/]?(\d+)(?:\D|$)")


@dataclass
class ShardSummary:
    shard: int | None
    path: str
    commit: str = ""
    corpus: str = ""
    corpus_revision: str = ""
    selected: int = 0
    runnable: int = 0
    excluded: int = 0
    tests: int = 0
    passed: int = 0
    failed: int = 0
    skipped: int = 0
    warned: int = 0
    timed_out: int = 0
    crashed: int = 0
    run_tests_exit: int | None = None
    exclusions_by_category: Counter[str] = field(default_factory=Counter)
    run_logs: list[str] = field(default_factory=list)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Compose full-corpus PHPT shard summaries."
    )
    parser.add_argument(
        "roots",
        nargs="+",
        type=Path,
        help="Artifact directories containing summary-*.txt files.",
    )
    parser.add_argument(
        "--expected-shards",
        type=int,
        default=15,
        help="Expected number of zero-based shards.",
    )
    parser.add_argument(
        "--out-dir",
        type=Path,
        default=Path(".runtime/full-phpt-composed"),
        help="Directory for summary.md and summary.json.",
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


def resolve_artifact_sibling(summary_path: Path, recorded_path: str) -> Path:
    recorded = Path(recorded_path)
    if recorded.is_file():
        return recorded
    sibling = summary_path.parent / recorded.name
    return sibling if sibling.is_file() else recorded


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


def parse_summary(path: Path) -> ShardSummary:
    summary = ShardSummary(shard=infer_shard(path), path=str(path))
    for raw_line in path.read_text(errors="replace").splitlines():
        line = raw_line.strip()
        if line.startswith("commit: "):
            summary.commit = line.removeprefix("commit: ").strip()
        elif line.startswith("corpus: "):
            summary.corpus = line.removeprefix("corpus: ").strip()
        elif line.startswith("corpus-revision: "):
            summary.corpus_revision = line.removeprefix("corpus-revision: ").strip()
        elif line.startswith("count: "):
            match = re.search(
                r"(\d+) selected PHPT rows; (\d+) runnable; (\d+) excluded", line
            )
            if match:
                summary.selected = int(match.group(1))
                summary.runnable = int(match.group(2))
                summary.excluded = int(match.group(3))
        elif line.startswith("result: "):
            values = parse_key_values(line)
            summary.selected = int_value(values, "selected")
            summary.runnable = int_value(values, "runnable")
            summary.excluded = int_value(values, "excluded")
            summary.tests = int_value(values, "tests")
            summary.passed = int_value(values, "passed")
            summary.failed = int_value(values, "failed")
            summary.skipped = int_value(values, "skipped")
            summary.warned = int_value(values, "warned")
            summary.timed_out = int_value(values, "timed_out")
            summary.crashed = int_value(values, "crashed")
        elif line.startswith("classification.") and ": rows=" in line:
            category = line.split(":", 1)[0].removeprefix("classification.")
            values = parse_key_values(line)
            summary.exclusions_by_category[category] += int_value(values, "rows")
        elif line.startswith("bucket: "):
            values = parse_key_values(line)
            log_value = values.get("log", "")
            if log_value:
                summary.run_logs.append(str(resolve_artifact_sibling(path, log_value)))
        elif line.startswith("run-tests-exit: "):
            value = line.removeprefix("run-tests-exit: ").strip()
            if value.isdigit():
                summary.run_tests_exit = int(value)
    return summary


def percent(part: int, total: int) -> str:
    if total <= 0:
        return "n/a"
    return f"{(part / total) * 100:.2f}%"


def summary_to_dict(summary: ShardSummary) -> dict[str, object]:
    return {
        "shard": summary.shard,
        "path": summary.path,
        "commit": summary.commit,
        "corpus": summary.corpus,
        "corpus_revision": summary.corpus_revision,
        "selected": summary.selected,
        "runnable": summary.runnable,
        "excluded": summary.excluded,
        "tests": summary.tests,
        "passed": summary.passed,
        "failed": summary.failed,
        "skipped": summary.skipped,
        "warned": summary.warned,
        "timed_out": summary.timed_out,
        "crashed": summary.crashed,
        "run_tests_exit": summary.run_tests_exit,
        "exclusions_by_category": dict(summary.exclusions_by_category),
        "run_logs": summary.run_logs,
    }


def main() -> int:
    args = parse_args()
    paths: list[Path] = []
    for root in args.roots:
        paths.extend(sorted(root.rglob("summary-*.txt")))

    summaries = [parse_summary(path) for path in paths]
    by_shard: dict[int, ShardSummary] = {}
    unassigned: list[ShardSummary] = []
    for summary in summaries:
        if summary.shard is None:
            unassigned.append(summary)
            continue
        previous = by_shard.get(summary.shard)
        if previous is None or summary.path > previous.path:
            by_shard[summary.shard] = summary

    expected = set(range(args.expected_shards))
    present = set(by_shard)
    missing = sorted(expected - present)
    extra = sorted(present - expected)
    complete_summaries = [
        item for item in by_shard.values() if item.run_tests_exit is not None
    ]
    incomplete_summaries = [
        item for item in by_shard.values() if item.run_tests_exit is None
    ]
    selected = sum(item.selected for item in complete_summaries)
    runnable = sum(item.runnable for item in complete_summaries)
    excluded = sum(item.excluded for item in complete_summaries)
    tests = sum(item.tests for item in complete_summaries)
    passed = sum(item.passed for item in complete_summaries)
    failed = sum(item.failed for item in complete_summaries)
    skipped = sum(item.skipped for item in complete_summaries)
    warned = sum(item.warned for item in complete_summaries)
    timed_out = sum(item.timed_out for item in complete_summaries)
    crashed = sum(item.crashed for item in complete_summaries)
    incomplete_selected = sum(item.selected for item in incomplete_summaries)
    incomplete_runnable = sum(item.runnable for item in incomplete_summaries)
    incomplete_excluded = sum(item.excluded for item in incomplete_summaries)
    incomplete_tests = sum(item.tests for item in incomplete_summaries)
    incomplete_passed = sum(item.passed for item in incomplete_summaries)
    incomplete_failed = sum(item.failed for item in incomplete_summaries)
    incomplete_skipped = sum(item.skipped for item in incomplete_summaries)
    incomplete_warned = sum(item.warned for item in incomplete_summaries)
    incomplete_timed_out = sum(item.timed_out for item in incomplete_summaries)
    incomplete_crashed = sum(item.crashed for item in incomplete_summaries)
    checkpoint_summaries = list(by_shard.values())
    checkpoint_selected = sum(item.selected for item in checkpoint_summaries)
    checkpoint_runnable = sum(item.runnable for item in checkpoint_summaries)
    checkpoint_tests = sum(item.tests for item in checkpoint_summaries)
    checkpoint_passed = sum(item.passed for item in checkpoint_summaries)
    checkpoint_failed = sum(item.failed for item in checkpoint_summaries)
    checkpoint_skipped = sum(item.skipped for item in checkpoint_summaries)
    checkpoint_warned = sum(item.warned for item in checkpoint_summaries)
    checkpoint_timed_out = sum(item.timed_out for item in checkpoint_summaries)
    checkpoint_crashed = sum(item.crashed for item in checkpoint_summaries)
    nonzero = sum(
        1
        for item in by_shard.values()
        if item.run_tests_exit is not None and item.run_tests_exit != 0
    )
    commits = Counter(item.commit for item in by_shard.values() if item.commit)
    corpus_revisions = Counter(
        item.corpus_revision for item in by_shard.values() if item.corpus_revision
    )
    exclusion_categories: Counter[str] = Counter()
    failure_clusters: Counter[str] = Counter()
    for item in by_shard.values():
        exclusion_categories.update(item.exclusions_by_category)
        for log in item.run_logs:
            failure_clusters.update(failing_clusters_from_log(Path(log)))

    payload = {
        "expected_shards": args.expected_shards,
        "shard_summaries_found": len(present),
        "completed_shards": len(complete_summaries),
        "incomplete_shards": [item.shard for item in incomplete_summaries],
        "missing_shards": missing,
        "extra_shards": extra,
        "unassigned_summaries": [summary_to_dict(item) for item in unassigned],
        "totals": {
            "selected": selected,
            "runnable": runnable,
            "excluded": excluded,
            "tests": tests,
            "passed": passed,
            "failed": failed,
            "skipped": skipped,
            "warned": warned,
            "timed_out": timed_out,
            "crashed": crashed,
            "not_passing_corpus_rows": max(selected - passed, 0),
            "pass_rate_of_corpus": percent(passed, selected),
            "pass_rate_of_runnable": percent(passed, runnable),
            "nonzero_shard_exits": nonzero,
            "incomplete_selected": incomplete_selected,
            "incomplete_runnable": incomplete_runnable,
            "incomplete_excluded": incomplete_excluded,
            "incomplete_tests": incomplete_tests,
            "incomplete_passed": incomplete_passed,
            "incomplete_failed": incomplete_failed,
            "incomplete_skipped": incomplete_skipped,
            "incomplete_warned": incomplete_warned,
            "incomplete_timed_out": incomplete_timed_out,
            "incomplete_crashed": incomplete_crashed,
            "checkpoint_selected": checkpoint_selected,
            "checkpoint_runnable": checkpoint_runnable,
            "checkpoint_tests": checkpoint_tests,
            "checkpoint_passed": checkpoint_passed,
            "checkpoint_failed": checkpoint_failed,
            "checkpoint_skipped": checkpoint_skipped,
            "checkpoint_warned": checkpoint_warned,
            "checkpoint_timed_out": checkpoint_timed_out,
            "checkpoint_crashed": checkpoint_crashed,
            "checkpoint_pass_rate": percent(checkpoint_passed, checkpoint_selected),
        },
        "commits": dict(commits),
        "corpus_revisions": dict(corpus_revisions),
        "exclusion_categories": dict(exclusion_categories),
        "failure_clusters": dict(failure_clusters),
        "shards": [summary_to_dict(by_shard[index]) for index in sorted(by_shard)],
    }

    args.out_dir.mkdir(parents=True, exist_ok=True)
    (args.out_dir / "summary.json").write_text(json.dumps(payload, indent=2) + "\n")

    lines = [
        "# Full PHPT Corpus",
        "",
        "| Metric | Count |",
        "| --- | ---: |",
        f"| Expected shards | {args.expected_shards} |",
        f"| Shard summaries found | {len(present)} |",
        f"| Complete shard summaries | {len(complete_summaries)} |",
        f"| Incomplete shard summaries | {len(incomplete_summaries)} |",
        f"| Selected corpus rows | {selected} |",
        f"| Runnable rows | {runnable} |",
        f"| Classified exclusions | {excluded} |",
        f"| Executed tests | {tests} |",
        f"| Passed tests | {passed} |",
        f"| Failed tests | {failed} |",
        f"| Skipped tests | {skipped} |",
        f"| Warned tests | {warned} |",
        f"| Timed-out tests | {timed_out} |",
        f"| Crashed/no-summary batches | {crashed} |",
        f"| Not passing corpus rows | {max(selected - passed, 0)} |",
        f"| Pass rate of corpus | {percent(passed, selected)} |",
        f"| Pass rate of runnable rows | {percent(passed, runnable)} |",
        f"| Nonzero shard exits | {nonzero} |",
        f"| Rows in incomplete shard summaries | {incomplete_selected} |",
        f"| Runnable rows in incomplete shard summaries | {incomplete_runnable} |",
        f"| Exclusions in incomplete shard summaries | {incomplete_excluded} |",
        f"| Tests in incomplete shard checkpoints | {incomplete_tests} |",
        f"| Passed tests in incomplete shard checkpoints | {incomplete_passed} |",
        f"| Failed tests in incomplete shard checkpoints | {incomplete_failed} |",
        f"| Skipped tests in incomplete shard checkpoints | {incomplete_skipped} |",
        f"| Warned tests in incomplete shard checkpoints | {incomplete_warned} |",
        f"| Timed-out tests in incomplete shard checkpoints | {incomplete_timed_out} |",
        f"| Crashed/no-summary batches in incomplete shard checkpoints | {incomplete_crashed} |",
        f"| Checkpoint reported rows | {checkpoint_selected} |",
        f"| Checkpoint passed tests | {checkpoint_passed} |",
        f"| Checkpoint failed tests | {checkpoint_failed} |",
        f"| Checkpoint pass rate | {percent(checkpoint_passed, checkpoint_selected)} |",
        "",
    ]

    if commits:
        lines.extend(["## Commits", ""])
        lines.extend(f"- `{commit}`: {count} shard(s)" for commit, count in commits.items())
        lines.append("")
    if corpus_revisions:
        lines.extend(["## PHPT Corpus Revisions", ""])
        lines.extend(
            f"- `{revision}`: {count} shard(s)"
            for revision, count in corpus_revisions.items()
        )
        lines.append("")
    if exclusion_categories:
        lines.extend(
            [
                "## Top Exclusion Categories",
                "",
                "| Category | Rows |",
                "| --- | ---: |",
            ]
        )
        for category, count in exclusion_categories.most_common(20):
            lines.append(f"| `{category}` | {count} |")
        lines.append("")
    if failure_clusters:
        lines.extend(
            [
                "## Top Failing Path Clusters",
                "",
                "| Cluster | Rows |",
                "| --- | ---: |",
            ]
        )
        for cluster, count in failure_clusters.most_common(20):
            lines.append(f"| `{cluster}` | {count} |")
        lines.append("")
    if missing:
        lines.extend(["## Missing Shards", "", ", ".join(str(item) for item in missing), ""])
    if extra:
        lines.extend(["## Extra Shards", "", ", ".join(str(item) for item in extra), ""])
    if incomplete_summaries:
        lines.extend(
            [
                "## Incomplete Shards",
                "",
                "These shards wrote a header but did not reach `run-tests-exit`; they are usually canceled or runner-terminated and are excluded from pass-rate totals.",
                "",
                "| Shard | Selected | Passed | Failed | Skipped | Warned | Timed out | Crashed | Summary |",
                "| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |",
            ]
        )
        for item in sorted(
            incomplete_summaries,
            key=lambda summary: -1 if summary.shard is None else summary.shard,
        ):
            shard = "" if item.shard is None else str(item.shard)
            lines.append(
                f"| {shard} | {item.selected} | {item.passed} | {item.failed} | "
                f"{item.skipped} | {item.warned} | {item.timed_out} | "
                f"{item.crashed} | `{item.path}` |"
            )
        lines.append("")

    lines.extend(
        [
            "## Shards",
            "",
            "| Shard | Selected | Runnable | Excluded | Passed | Failed | Skipped | Warned | Exit | Summary |",
            "| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |",
        ]
    )
    for index in sorted(by_shard):
        item = by_shard[index]
        exit_value = "" if item.run_tests_exit is None else str(item.run_tests_exit)
        lines.append(
            f"| {index} | {item.selected} | {item.runnable} | {item.excluded} | "
            f"{item.passed} | {item.failed} | {item.skipped} | {item.warned} | "
            f"{exit_value} | `{item.path}` |"
        )
    lines.append("")

    markdown = "\n".join(lines)
    (args.out_dir / "summary.md").write_text(markdown)
    print(markdown)

    return 1 if missing or incomplete_summaries else 0


if __name__ == "__main__":
    sys.exit(main())
