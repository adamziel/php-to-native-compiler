#!/usr/bin/env python3
"""Compose 15-way full-corpus PHPT shard artifacts into one status number."""

from __future__ import annotations

import argparse
import json
import re
import sys
from collections import Counter
from dataclasses import dataclass, asdict
from pathlib import Path


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
    run_tests_exit: int | None = None


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
        elif line.startswith("run-tests-exit: "):
            value = line.removeprefix("run-tests-exit: ").strip()
            if value.isdigit():
                summary.run_tests_exit = int(value)
    return summary


def percent(part: int, total: int) -> str:
    if total <= 0:
        return "n/a"
    return f"{(part / total) * 100:.2f}%"


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
    selected = sum(item.selected for item in by_shard.values())
    runnable = sum(item.runnable for item in by_shard.values())
    excluded = sum(item.excluded for item in by_shard.values())
    tests = sum(item.tests for item in by_shard.values())
    passed = sum(item.passed for item in by_shard.values())
    failed = sum(item.failed for item in by_shard.values())
    skipped = sum(item.skipped for item in by_shard.values())
    warned = sum(item.warned for item in by_shard.values())
    nonzero = sum(
        1
        for item in by_shard.values()
        if item.run_tests_exit is not None and item.run_tests_exit != 0
    )
    commits = Counter(item.commit for item in by_shard.values() if item.commit)
    corpus_revisions = Counter(
        item.corpus_revision for item in by_shard.values() if item.corpus_revision
    )

    payload = {
        "expected_shards": args.expected_shards,
        "completed_shards": len(present),
        "missing_shards": missing,
        "extra_shards": extra,
        "unassigned_summaries": [asdict(item) for item in unassigned],
        "totals": {
            "selected": selected,
            "runnable": runnable,
            "excluded": excluded,
            "tests": tests,
            "passed": passed,
            "failed": failed,
            "skipped": skipped,
            "warned": warned,
            "not_passing_corpus_rows": max(selected - passed, 0),
            "pass_rate_of_corpus": percent(passed, selected),
            "pass_rate_of_runnable": percent(passed, runnable),
            "nonzero_shard_exits": nonzero,
        },
        "commits": dict(commits),
        "corpus_revisions": dict(corpus_revisions),
        "shards": [asdict(by_shard[index]) for index in sorted(by_shard)],
    }

    args.out_dir.mkdir(parents=True, exist_ok=True)
    (args.out_dir / "summary.json").write_text(json.dumps(payload, indent=2) + "\n")

    lines = [
        "# Full PHPT Corpus",
        "",
        "| Metric | Count |",
        "| --- | ---: |",
        f"| Expected shards | {args.expected_shards} |",
        f"| Completed shards | {len(present)} |",
        f"| Selected corpus rows | {selected} |",
        f"| Runnable rows | {runnable} |",
        f"| Classified exclusions | {excluded} |",
        f"| Executed tests | {tests} |",
        f"| Passed tests | {passed} |",
        f"| Failed tests | {failed} |",
        f"| Skipped tests | {skipped} |",
        f"| Warned tests | {warned} |",
        f"| Not passing corpus rows | {max(selected - passed, 0)} |",
        f"| Pass rate of corpus | {percent(passed, selected)} |",
        f"| Pass rate of runnable rows | {percent(passed, runnable)} |",
        f"| Nonzero shard exits | {nonzero} |",
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
    if missing:
        lines.extend(["## Missing Shards", "", ", ".join(str(item) for item in missing), ""])
    if extra:
        lines.extend(["## Extra Shards", "", ", ".join(str(item) for item in extra), ""])

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

    return 1 if missing else 0


if __name__ == "__main__":
    sys.exit(main())
