# Regression Shard Report Schema

Owner: developer-130
Lane: 32
Mode: read-only M0 QA/report schema; no compiler/runtime source edits

## Purpose

Use this schema for 221205Z regression shard, replay, and triage reports so an
integrator can merge evidence deterministically. The schema is designed for the
blocked candidate gate:

`/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377`

It must preserve the key distinction already seen in the evidence: most
latest-public PASS regressions are absent from candidate status/results, while
a much smaller set has concrete `FAILED` or `BORKED` rows. Do not treat absent
rows as proven semantic failures until focused replay or a repaired gate
produces row-level evidence.

## Required Header

Every shard report must begin with this metadata block:

| Field | Required value |
| --- | --- |
| `Title` | Human-readable report title naming the shard or replay area. |
| `Owner` | Agent name, for example `developer-130`. |
| `Lane` | Harness `work_lanes.id` plus title. |
| `Mode` | One of `read-only`, `focused replay`, `control-plane patch`, or `semantic repair`. |
| `Created` | UTC timestamp. |
| `Branch/worktree` | Git branch and worktree used. |
| `Source edits` | `none`, or a precise list of changed files if the lane explicitly allowed edits. |
| `Full gate run` | `no` unless a manager/integrator explicitly assigned a full PHPT gate. |
| `Public score movement` | Always `none` for M0/M1 reports and focused replay artifacts. |

Reports that omit source-edit status, full-gate status, or score-movement
status are not integration-ready.

## Evidence Inputs

Each report must list absolute paths for every evidence source it reads. Include
at least these rows when applicable:

| Evidence | Path or value |
| --- | --- |
| Candidate gate directory | `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377` |
| Accepted baseline directory | `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67` |
| Candidate regression list | `regressions-from-latest-published-passes.txt` |
| Candidate status file | `current-status.normalized.tsv` |
| Candidate aggregate results | `all-results.txt` |
| Candidate pass set | `current-passes.normalized.txt` |
| Accepted pass set | `current-passes.normalized.txt` from the accepted baseline directory |
| Gate script | `run_gate.sh` |
| PHPT source checkout | `/home/claude/php-src-phpt` at `f97ff597429a2fe633665a7e02d97c8077f9f90f` |
| Wrapper | `/home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper` |
| Harness database | `/home/claude/php-to-native-compiler/.harness/harness.sqlite3`, if inspected or updated |

If a historical binary, run root, shard list, or per-shard log is missing, name
the missing artifact explicitly and explain how that limits the conclusion.

## Scope Definition

Every shard report must define its row ownership before counting results.

Use this table:

| Scope item | Value |
| --- | --- |
| Included prefixes | Exact PHPT prefixes, for example `php-src/ext/standard/tests/array/`. |
| Excluded prefixes | Exact excluded prefixes, if any. |
| Regression source | Usually `regressions-from-latest-published-passes.txt`. |
| Selection rule | Text filter, path prefix, manifest list, or replay row file. |
| Late-priority exclusions | State whether eval or variable-variable rows were excluded or present. |
| Owned rows | Count after applying the selection rule. |

Do not use broad labels such as "standard library" without listing the exact
prefixes or row file.

## Accounting Summary

Every report must include a normalized accounting table. Use `ABSENT` only when
the row is present in the accepted PASS regression set but has no candidate row
in the inspected status/result artifact.

| Candidate artifact bucket | Rows | Definition |
| --- | ---: | --- |
| `ABSENT` from candidate status/results |  | No candidate status row and no normalized aggregate result row. |
| `FAILED` |  | Candidate status is a concrete non-PASS failure. |
| `BORKED` |  | Candidate status is a concrete PHPT setup/SKIPIF/parsing problem. |
| `SKIPPED` |  | Candidate status is skipped and no longer contributes a pass. |
| `PASSED` but still listed |  | Should normally be zero; investigate duplicates/normalization if nonzero. |
| Conflicting duplicate statuses |  | Same PHPT path appears with more than one candidate status. |
| Total owned regressions |  | Must equal the selected row count. |

Add a short paragraph interpreting the dominant bucket. For the 221205Z
candidate, a report should say explicitly whether the evidence points first to
missing-result/control-plane investigation or to a concrete semantic repair.

## Representative Rows

Each report must include representative rows that an integrator or follow-up
worker can replay without rereading the whole artifact set.

| Row | PHPT title | Accepted status | Candidate status | Artifact source | SKIPIF | Bucket | Why selected | Likely owner | Next action |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `php-src/.../*.phpt` | Title from `--TEST--` | `PASSED` | `ABSENT`/`FAILED`/`BORKED` | status/results/stdout path | `none` or reason | control-plane/semantic/env | One sentence | runtime/compiler/harness/replay | Focused command or repair lane |

Minimum row requirements:

- For a replay selector, include 3 to 8 rows.
- Include at least one concrete `FAILED` or `BORKED` row when the owned scope has
  one.
- Include absent rows separately from failed rows.
- Prefer no-`SKIPIF` rows for first focused replay unless the lane is about
  SKIPIF/environment behavior.
- Avoid eval and variable-variable rows unless the manager explicitly assigns
  that late-priority area.

## Symptom Buckets

Group rows by observed mechanism, not just by directory.

| Bucket | Rows | Evidence | Representative rows | Interpretation |
| --- | ---: | --- | --- | --- |
| Missing candidate result/status |  | Accepted pass set minus candidate status/results |  | Control-plane or shard-completeness first. |
| Concrete runtime/compiler failure |  | Candidate `FAILED` with stdout/run-tests diff |  | Semantic repair candidate. |
| SKIPIF/wrapper/environment |  | Candidate `BORKED`, missing constants, missing files, wrapper setup |  | Harness/wrapper repair or adjudication. |
| Duplicate/normalization conflict |  | Same path has multiple statuses or duplicate pass lines |  | Aggregator/accounting repair. |
| Unsupported documented boundary |  | Existing docs name unsupported behavior and PHPT now exercises it |  | Requires support decision, not a silent regression claim. |
| Pending replay |  | Historical binary/run root missing or row absent from artifacts |  | Rebuild/restore binary or rerun focused sample. |

The "likely bucket" in a report must be one of these labels or a stricter
sub-bucket that maps back to one of them.

## Commands Used

Reports must include exact commands in fenced shell blocks. For each command,
state whether it was:

- artifact inspection only,
- SQLite status/event update,
- focused replay,
- focused unit test,
- control-plane test,
- or full gate.

For replay commands, include the environment variables that affect execution:
`PHPC_BIN`, `TEST_PHP_EXECUTABLE`, `PHPT_SYSTEM_PHP`,
`PHPC_PHPT_TIMEOUT_SECONDS`, `PHPC_PHPT_KILL_AFTER_SECONDS`, `TMPDIR`,
`TEST_PHP_SRCDIR`, and the `run-tests.php` command line.

If a command is intentionally not run, include the reason and the missing
precondition. Example: "not run because the historical candidate release
binary under `/tmp/.../cargo-target/release/phpc` is no longer present."

## Replay Results

Focused replay reports must write a result table even when replay is blocked.

| Row | Accepted replay | Candidate replay | Classification | Evidence path |
| --- | --- | --- | --- | --- |
| `php-src/.../*.phpt` | `not run`/`PASS`/`FAIL`/`BORK` | `not run`/`PASS`/`FAIL`/`BORK` | semantic/control-plane/env/pending | Absolute log/result path |

Rules:

- Focused replay cannot move the public score.
- A focused candidate `PASS` only clears that row for the replay context; it
  does not prove shard completeness.
- A candidate `FAIL` must be paired with the failing stdout/run-tests excerpt
  path, not just a summary claim.
- If accepted replay fails, stop and classify the sample as replay-environment
  invalid until fixed.

## Artifact Manifest

Reports must list their own outputs.

| Artifact | Purpose | Created by | Hash/check |
| --- | --- | --- | --- |
| `.harness/reports/<name>.md` | Main report | Agent/lane | `git status` or `sha256sum`, if outside git |
| `/tmp/.../*.tests` | Optional replay row list | Command block | `wc -l` and path list |
| `/tmp/.../results.txt` | Optional replay result | `run-tests.php` | status summary |
| `/tmp/.../stdout.log` | Optional replay log | `run-tests.php` | inspected lines |

If the artifact is central harness state outside the worker git worktree, say so
explicitly so integrators do not look for a product-source diff.

## SQLite Status

When the harness MCP memory tools are unavailable, agents may use Python's
standard `sqlite3` module as the deterministic fallback. A report should note
that fallback only when it was used.

Minimum DB events/status for owned lanes:

- update the agent row with current status and lane notes,
- mark the claimed lane `in_progress` with branch/worktree,
- insert a `developer_progress` event when claiming,
- insert a completion event with report path when finished,
- mark the lane `completed` only after the report artifact exists.

Do not mark another worker's lane complete unless the manager explicitly
assigned reconciliation work.

## Proposed Next Action

End each report with one deterministic next action table:

| Priority | Action | Owner type | Preconditions | Stop condition | Expected artifact |
| ---: | --- | --- | --- | --- | --- |
| 1 | Focused replay / harness patch / semantic repair | Developer/Integrator/Manager | Exact required inputs | Exact pass/fail condition | Report, commit, or gate artifact |

The next action must be scoped. Avoid vague actions such as "fix arrays" or
"improve compatibility." Prefer "run accepted-vs-candidate replay for these
eight row paths after rebuilding candidate `phpc` at commit X" or "patch shard
harness directory copy so `ext/pdo/tests` exists for shard-03/04 dry-run."

## Integration-Ready Checklist

A report is integration-ready only if all items below are true:

- Header names owner, lane, branch/worktree, source-edit status, full-gate
  status, and score movement.
- Evidence inputs use absolute paths.
- Scope prefixes or row file are exact.
- Counts reconcile to the selected row set.
- `ABSENT`, `FAILED`, and `BORKED` rows are not collapsed into one bucket.
- Representative rows include candidate and accepted status.
- Commands are exact and reproducible.
- Replay results or blocked replay preconditions are explicit.
- Proposed next action is deterministic and scoped.
- Eval and variable-variable rows are either absent from scope or named as
  late-priority exceptions.
- The report does not claim PHP support or public-score movement without an
  assigned full gate and passing evidence.

## Minimal Template

````markdown
# <Shard or Replay Report Title>

Owner: <agent>
Lane: <id and title>
Mode: <read-only/focused replay/control-plane patch/semantic repair>
Created: <UTC timestamp>
Branch/worktree: <branch> / <absolute worktree>
Source edits: <none or files>
Full gate run: no
Public score movement: none

## Evidence Inputs

| Evidence | Path or value |
| --- | --- |
| Candidate gate directory | ... |
| Accepted baseline directory | ... |
| Regression list | ... |
| Candidate status/results | ... |
| PHPT source checkout | ... |

## Scope

| Scope item | Value |
| --- | --- |
| Included prefixes | ... |
| Excluded prefixes | ... |
| Owned rows | ... |

## Accounting

| Candidate artifact bucket | Rows |
| --- | ---: |
| ABSENT | ... |
| FAILED | ... |
| BORKED | ... |
| SKIPPED | ... |
| PASSED but still listed | ... |
| Total owned regressions | ... |

## Representative Rows

| Row | PHPT title | Accepted status | Candidate status | Bucket | Why selected | Next action |
| --- | --- | --- | --- | --- | --- | --- |
| ... | ... | ... | ... | ... | ... | ... |

## Commands Used

```sh
...
```

## Replay Results

| Row | Accepted replay | Candidate replay | Classification | Evidence path |
| --- | --- | --- | --- | --- |
| ... | ... | ... | ... | ... |

## Proposed Next Action

| Priority | Action | Owner type | Preconditions | Stop condition | Expected artifact |
| ---: | --- | --- | --- | --- | --- |
| 1 | ... | ... | ... | ... | ... |
````
