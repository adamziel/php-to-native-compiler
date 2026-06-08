# Focused Replay: Reflection Regression Rows

| Field | Value |
| --- | --- |
| Title | Focused replay: reflection regression rows |
| Owner | developer-109 |
| Lane | worklanes#83, Focused replay: reflection regression rows |
| Mode | read-only focused replay/report |
| Created | 2026-06-07T18:17:48Z |
| Branch/worktree | `work/developer-109` / `/home/claude/php-to-native-compiler/.harness/worktrees/developer-109` |
| Source edits | none; report-only artifact under `.harness/reports/` |
| Full gate run | no |
| Public score movement | none |

## Evidence Inputs

| Evidence | Path or value |
| --- | --- |
| Reflection shard report | `/home/claude/php-to-native-compiler/.harness/reports/221205Z-reflection.md` and this worktree copy `.harness/reports/221205Z-reflection.md` |
| PASS-regression manifest | `.harness/reports/221205Z-pass-regression-manifest.md` |
| Late-priority overlap report | `.harness/reports/221205Z-late-priority-overlap.md` |
| Replay cookbook | `.harness/reports/focused-replay-cookbook.md` |
| Binary availability recheck | `.harness/reports/phpt-binary-wrapper-availability-recheck-dev305.md` |
| Report schema | `.harness/reports/regression-shard-report-schema.md` |
| Accepted baseline directory named by reports | `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67` |
| Candidate gate directory named by reports | `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377` |
| PHPT source checkout | `/home/claude/php-src-phpt` at `f97ff597429a2fe633665a7e02d97c8077f9f90f` |
| Wrapper | `/home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper` |
| Harness database | `/home/claude/php-to-native-compiler/.harness/harness.sqlite3` |

The historical `state/logs/phpt-full-current-score-*` evidence directories are
not present in this live filesystem anymore, so direct artifact joins against
`regressions-from-latest-published-passes.txt`, `current-status.normalized.tsv`,
and `all-results.txt` cannot be recomputed in this lane. Counts and
accepted/candidate status buckets below come from the saved, committed
`221205Z-reflection.md` and `221205Z-pass-regression-manifest.md` reports, which
were produced from those artifacts while they were available.

## Scope

| Scope item | Value |
| --- | --- |
| Included prefixes | `php-src/ext/reflection/` |
| Excluded prefixes | none within the prefix for accounting; replay sample excludes late-priority rows |
| Regression source | Saved `221205Z-reflection.md` over the candidate `regressions-from-latest-published-passes.txt` |
| Selection rule | Eight representative reflection rows across class, function, method, parameter, property, and DNF type metadata |
| Late-priority exclusions | `php-src/ext/reflection/tests/bug64936.phpt` is tagged `eval` in `.harness/reports/221205Z-late-priority-overlap.md` and is excluded from this first replay sample |
| Owned rows | 110 latest-public PASS regressions under `php-src/ext/reflection/` |

## Accounting Summary

| Candidate artifact bucket | Rows | Definition |
| --- | ---: | --- |
| `ABSENT` from candidate status/results | 110 | Accepted PASS rows with no candidate row in saved `current-status.normalized.tsv` or normalized `all-results.txt` |
| `FAILED` | 0 | Candidate status is a concrete non-PASS failure |
| `BORKED` | 0 | Candidate status is a PHPT setup/SKIPIF/parsing problem |
| `SKIPPED` | 0 | Candidate status is skipped |
| `PASSED` but still listed | 0 | Candidate status is pass despite regression listing |
| Conflicting duplicate statuses | 0 | Same selected PHPT path has more than one candidate status |
| Total owned regressions | 110 | All rows under `php-src/ext/reflection/` in the saved regression shard |

The reflection shard is therefore a control-plane/result-coverage symptom first,
not a proven reflection semantic regression. The saved shard report says all
110 rows were accepted `PASSED` and all 110 were absent from candidate status
and aggregate result artifacts. It also records that the candidate did execute
other reflection rows, so the problem is not a total reflection-directory skip.

## Representative Rows

| Row | PHPT title | Accepted status | Candidate status | Artifact source | SKIPIF | Bucket | Why selected | Likely owner | Next action |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `php-src/ext/reflection/tests/001.phpt` | Reflection inheritance | `PASSED` | `ABSENT` | `221205Z-reflection.md` representative row | none | control-plane/pending replay | Legacy inheritance metadata row | replay/harness first | Replay after authoritative binaries are restored |
| `php-src/ext/reflection/tests/ReflectionClass_getConstants_basic.phpt` | ReflectionClass::getConstants() | `PASSED` | `ABSENT` | `221205Z-reflection.md` and backlog precheck | none | control-plane/pending replay | Class constant metadata | replay/harness first | Replay after authoritative binaries are restored |
| `php-src/ext/reflection/tests/ReflectionClass_getProperties_001.phpt` | ReflectionClass::getProperties() | `PASSED` | `ABSENT` | reflection backlog precheck plus all-reflection absence report | none | control-plane/pending replay | Class property enumeration | replay/harness first | Replay after authoritative binaries are restored |
| `php-src/ext/reflection/tests/ReflectionFunction_getClosureUsedVariables.phpt` | ReflectionFunctionAbstract::getClosureUsedVariables | `PASSED` | `ABSENT` | `221205Z-reflection.md` representative row | none | control-plane/pending replay | Closure/function metadata | replay/harness first | Replay after authoritative binaries are restored |
| `php-src/ext/reflection/tests/ReflectionMethod_constructor_basic.phpt` | ReflectionMethod::isConstructor() | `PASSED` | `ABSENT` | reflection backlog precheck plus all-reflection absence report | none | control-plane/pending replay | Method metadata | replay/harness first | Replay after authoritative binaries are restored |
| `php-src/ext/reflection/tests/ReflectionParameter_001.phpt` | ReflectionParameter class - getNames() method. | `PASSED` | `ABSENT` | reflection backlog precheck plus all-reflection absence report | none | control-plane/pending replay | Parameter metadata/default cluster | replay/harness first | Replay after authoritative binaries are restored |
| `php-src/ext/reflection/tests/ReflectionProperty_getModifiers.001.phpt` | ReflectionProperty::getModifiers() | `PASSED` | `ABSENT` | reflection backlog precheck plus all-reflection absence report | none | control-plane/pending replay | Property modifier metadata | replay/harness first | Replay after authoritative binaries are restored |
| `php-src/ext/reflection/tests/types/dnf_types.phpt` | Disjunctive Normal Form types in reflection | `PASSED` | `ABSENT` | `221205Z-reflection.md` representative row | none | control-plane/pending replay | DNF type metadata | replay/harness first | Replay after authoritative binaries are restored |

Live source scan confirmed all eight selected PHPT files exist under
`/home/claude/php-src-phpt` and none contains the planning-compatible `eval` or
variable-variable markers used by the late-priority overlap reports.

## Focused Replay Results

| Row | Accepted replay | Candidate replay | Classification | Evidence path |
| --- | --- | --- | --- |
| `php-src/ext/reflection/tests/001.phpt` | not run | not run | pending replay; candidate row absent | Missing historical accepted/candidate `PHPC_BIN` paths |
| `php-src/ext/reflection/tests/ReflectionClass_getConstants_basic.phpt` | not run | not run | pending replay; candidate row absent | Missing historical accepted/candidate `PHPC_BIN` paths |
| `php-src/ext/reflection/tests/ReflectionClass_getProperties_001.phpt` | not run | not run | pending replay; candidate row absent | Missing historical accepted/candidate `PHPC_BIN` paths |
| `php-src/ext/reflection/tests/ReflectionFunction_getClosureUsedVariables.phpt` | not run | not run | pending replay; candidate row absent | Missing historical accepted/candidate `PHPC_BIN` paths |
| `php-src/ext/reflection/tests/ReflectionMethod_constructor_basic.phpt` | not run | not run | pending replay; candidate row absent | Missing historical accepted/candidate `PHPC_BIN` paths |
| `php-src/ext/reflection/tests/ReflectionParameter_001.phpt` | not run | not run | pending replay; candidate row absent | Missing historical accepted/candidate `PHPC_BIN` paths |
| `php-src/ext/reflection/tests/ReflectionProperty_getModifiers.001.phpt` | not run | not run | pending replay; candidate row absent | Missing historical accepted/candidate `PHPC_BIN` paths |
| `php-src/ext/reflection/tests/types/dnf_types.phpt` | not run | not run | pending replay; candidate row absent | Missing historical accepted/candidate `PHPC_BIN` paths |

Focused `run-tests.php` replay was not executed. The wrapper and pinned php-src
checkout are present, but the historical accepted and candidate release
binaries are both absent:

```text
missing /tmp/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67/cargo-target/release/phpc
missing /tmp/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/cargo-target/release/phpc
present executable /home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper
present executable /home/claude/php-src-phpt/run-tests.php
```

Running unpinned scratch binaries would not measure accepted-vs-candidate
behavior for the blocked 221205Z gate, so this lane leaves replay unavailable
instead of producing misleading results.

## Commands Used

Session documents and lane/context reads:

```sh
sed -n '1,240p' AGENTS.md
sed -n '1,260p' docs/PROGRESS.md
sed -n '1,260p' docs/ARCHITECTURE.md
sed -n '1,260p' docs/SUPPORT.md
sed -n '1,260p' README.md
sed -n '1,260p' docs/LOOP_MEMORY.md
sed -n '1,260p' /home/claude/php-to-native-compiler/DEVELOPMENT.md
```

Evidence and schema reads:

```sh
sed -n '1,260p' .harness/reports/221205Z-reflection.md
sed -n '1,220p' .harness/reports/221205Z-late-priority-overlap.md
sed -n '1,220p' .harness/reports/phpt-binary-wrapper-availability-recheck-dev305.md
sed -n '1,220p' .harness/reports/regression-shard-report-schema.md
sed -n '1,220p' .harness/reports/221205Z-pass-regression-manifest.md
```

Live evidence-root and replay-prerequisite checks:

```sh
ls -la /home/claude/supervised-php-compiler/state/logs
find /home/claude/supervised-php-compiler/state/logs -maxdepth 2 -type f -name 'regressions-from-latest-published-passes.txt' -print
find /home/claude/supervised-php-compiler/state -maxdepth 4 -type d -name 'phpt-full-current-score-20260604T221205Z*' -print
git -C /home/claude/php-src-phpt rev-parse HEAD
test -x /home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper
test -x /home/claude/php-src-phpt/run-tests.php
test -x /tmp/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67/cargo-target/release/phpc
test -x /tmp/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/cargo-target/release/phpc
```

Selected-row title/source-marker check:

```sh
python3 - <<'PY'
from pathlib import Path
PHP=Path('/home/claude/php-src-phpt')
rows=[
'php-src/ext/reflection/tests/001.phpt',
'php-src/ext/reflection/tests/ReflectionClass_getConstants_basic.phpt',
'php-src/ext/reflection/tests/ReflectionClass_getProperties_001.phpt',
'php-src/ext/reflection/tests/ReflectionFunction_getClosureUsedVariables.phpt',
'php-src/ext/reflection/tests/ReflectionMethod_constructor_basic.phpt',
'php-src/ext/reflection/tests/ReflectionParameter_001.phpt',
'php-src/ext/reflection/tests/ReflectionProperty_getModifiers.001.phpt',
'php-src/ext/reflection/tests/types/dnf_types.phpt',
]
for row in rows:
    p=PHP / row.removeprefix('php-src/')
    text=p.read_text(errors='replace')
    lower=text.lower()
    late=[]
    if 'eval(' in lower or 'eval (' in lower:
        late.append('eval')
    if '$$' in text or '${$' in text:
        late.append('variable-variable')
    print(row, p.exists(), '+'.join(late) if late else 'none')
PY
```

SQLite/MCP actions:

```text
memory_query inspected agents, worklanes, events, messages, test_runs, test_results, and issues schemas/rows.
memory_update_agent set developer-109 status to working on lane 83.
memory_record_event recorded lane 83 start and scope.
```

## Artifact Manifest

| Artifact | Purpose | Created by | Hash/check |
| --- | --- | --- | --- |
| `.harness/reports/focused-replay-reflection-dev110.md` | Main lane 83 focused reflection replay/report artifact | developer-109 | validate with `git diff --check -- .harness/reports/focused-replay-reflection-dev110.md` |

No `/tmp` replay row file, `results.txt`, stdout log, stderr log, or
`run-tests.log` was created because focused replay was blocked before execution
by missing authoritative binaries.

## Proposed Next Action

| Priority | Action | Owner type | Preconditions | Stop condition | Expected artifact |
| ---: | --- | --- | --- | --- | --- |
| 1 | Restore or rebuild durable accepted and candidate release `phpc` binaries for commits `0b917f67a37d9ca9779d77f87173b628431c2425` and `56fe9377fb46be00db5fdd30c966fdba406dc581`, then run the eight-row reflection replay selector above | Developer or Integrator | Durable `PHPC_BIN` pair with manifest, existing wrapper, existing php-src checkout | Accepted replay passes and candidate replay emits row-level statuses for all eight selected rows | Focused replay directory with row list, results, stdout/stderr/run-tests logs, binary manifest, and updated report |
| 2 | If the candidate replay emits all eight rows, replay the full 110-row reflection regression prefix in a lane-local evidence directory | Developer or Integrator | Priority 1 completed with valid accepted replay | All 110 reflection rows have candidate row-level statuses | Full reflection focused replay artifact, still with no public score movement |
| 3 | Repair the shard completeness/control-plane issue before public gate retry | Harness/control-plane Developer | Confirmed missing or pruned expected row/status artifacts and historical `run-tests-harnesses` abort evidence | A managed dry-run/focused gate preserves expected row lists and detects partial shard result output | Harness patch/test artifact followed only by an explicitly assigned full current-score gate |

## Integration-Ready Checklist

- [x] Report includes owner, lane, mode, created timestamp, branch/worktree,
  source-edit status, full-gate status, and score-movement status.
- [x] Report lists absolute evidence paths and names missing replay
  prerequisites.
- [x] Report defines exact reflection prefix scope and row count.
- [x] Report separates absent candidate rows from concrete semantic failures.
- [x] Report includes eight representative non-late reflection rows.
- [x] Report states focused replay was not run and names the missing
  authoritative binaries.
- [x] Report includes exact commands used and MCP status/event actions.
- [x] Report names deterministic next actions and expected artifacts.
