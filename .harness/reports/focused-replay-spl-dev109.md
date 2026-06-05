# Focused Replay: SPL Regression Rows

| Field | Value |
| --- | --- |
| Title | Focused replay: SPL regression rows |
| Owner | developer-394 |
| Lane | work_lanes#82, Focused replay: SPL regression rows |
| Mode | read-only focused replay/report |
| Created | 2026-06-05T09:52:15+00:00 |
| Branch/worktree | `work/developer-394` / `/home/claude/php-to-native-compiler/.harness/worktrees/developer-394` |
| Source edits | none; report-only artifact under `.harness/reports/` |
| Full gate run | no |
| Public score movement | none |

## Evidence Inputs

| Evidence | Path or value |
| --- | --- |
| Candidate gate directory | `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377` |
| Accepted baseline directory | `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67` |
| Candidate regression list | `regressions-from-latest-published-passes.txt` |
| Candidate status file | `current-status.normalized.tsv` |
| Candidate aggregate results | `all-results.txt` |
| Accepted status file | `current-status.normalized.tsv` |
| Accepted aggregate results | `all-results.txt` |
| Gate script | `run_gate.sh` under both accepted and candidate evidence roots |
| PHPT source checkout | `/home/claude/php-src-phpt` at `f97ff597429a2fe633665a7e02d97c8077f9f90f` |
| Wrapper | `/home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper` |
| Existing cluster report | `.harness/reports/221205Z-spl.md` |
| Replay cookbook | `.harness/reports/focused-replay-cookbook.md` |
| Binary availability report | `.harness/reports/phpt-binary-wrapper-availability-recheck-dev305.md` |
| Harness database | `/home/claude/php-to-native-compiler/.harness/harness.sqlite3` |

Historical accepted and candidate `/tmp` run roots are not durable, and both
recorded release binaries are currently missing:

- `/tmp/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67/cargo-target/release/phpc`
- `/tmp/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/cargo-target/release/phpc`

The wrapper and pinned php-src checkout are present and executable, so replay is
blocked specifically on the missing authoritative accepted/candidate `PHPC_BIN`
pair, not on wrapper or php-src availability.

## Scope

| Scope item | Value |
| --- | --- |
| Included prefixes | `php-src/ext/spl/` latest-public PASS regression rows |
| Excluded prefixes | none |
| Regression source | Candidate `regressions-from-latest-published-passes.txt` |
| Selection rule | `row.startswith("php-src/ext/spl/")` |
| Late-priority exclusions | No eval or variable-variable rows are in this SPL prefix |
| Owned rows | 137 |

## Accounting Summary

| Candidate artifact bucket | Rows | Definition |
| --- | ---: | --- |
| `ABSENT` from candidate status/results | 137 | No candidate row in `current-status.normalized.tsv` and no normalized row in `all-results.txt` |
| `FAILED` | 0 | Candidate status is a concrete non-PASS failure |
| `BORKED` | 0 | Candidate status is a PHPT setup/SKIPIF/parsing problem |
| `SKIPPED` | 0 | Candidate status is skipped |
| `PASSED` but still listed | 0 | Candidate status is pass despite regression listing |
| Conflicting duplicate statuses | 0 | Same selected PHPT path appears with more than one candidate status |
| Total owned regressions | 137 | All `php-src/ext/spl/` rows in the regression list |

All 137 selected rows were `PASSED` in the accepted baseline status artifact.
The candidate artifacts do not contain row-level status or result output for any
of them. This points first to shard completeness/control-plane investigation,
not to a proven SPL semantic regression. The existing SPL cluster report ties
the absence to the blocked candidate gate's truncated shard-03/shard-04
execution, where the saved stdout reports missing copied
`run-tests-harnesses/.../ext/pdo/tests` directories before later SPL buckets
can be observed.

## Representative Rows

| Row | PHPT title | Accepted status | Candidate status | Candidate result | Bucket | Why selected | Likely owner | Next action |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `php-src/ext/spl/tests/ArrayObject/ArrayObject_clone_other_std_props.phpt` | Clone ArrayObject using other with STD_PROP_LIST | `PASSED` | `ABSENT` | `ABSENT` | control-plane/pending replay | ArrayObject/ArrayIterator coverage | replay/harness first | Replay after authoritative binaries are restored |
| `php-src/ext/spl/tests/SplFileObject/SplFileObject_fgetcsv_basic.phpt` | SplFileObject::fgetcsv default path | `PASSED` | `ABSENT` | `ABSENT` | control-plane/pending replay | SplFileObject/file iterator coverage | replay/harness first | Replay after authoritative binaries are restored |
| `php-src/ext/spl/tests/SplObjectStorage/SplObjectStorage_seek.phpt` | SplObjectStorage::seek() basic functionality | `PASSED` | `ABSENT` | `ABSENT` | control-plane/pending replay | SplObjectStorage coverage | replay/harness first | Replay after authoritative binaries are restored |
| `php-src/ext/spl/tests/autoloading/spl_autoload_call_basic.phpt` | spl_autoload_call() function - basic test for spl_autoload_call() | `PASSED` | `ABSENT` | `ABSENT` | control-plane/pending replay | SPL autoloading coverage | replay/harness first | Replay after authoritative binaries are restored |
| `php-src/ext/spl/tests/DirectoryIterator_getBasename_basic_test.phpt` | DirectoryIterator::getBasename() - Basic Test | `PASSED` | `ABSENT` | `ABSENT` | control-plane/pending replay | Iterator/DirectoryIterator coverage | replay/harness first | Replay after authoritative binaries are restored |
| `php-src/ext/spl/tests/SplFixedArray__construct_param_array.phpt` | SplFixedArray::__construct() with array passed as integer. | `PASSED` | `ABSENT` | `ABSENT` | control-plane/pending replay | SplFixedArray coverage | replay/harness first | Replay after authoritative binaries are restored |
| `php-src/ext/spl/tests/SplDoublyLinkedList_offsetGet_param_array.phpt` | SplDoublyLinkedList::offsetGet() with 1st parameter passed as array. | `PASSED` | `ABSENT` | `ABSENT` | control-plane/pending replay | SPL data-structure coverage | replay/harness first | Replay after authoritative binaries are restored |

The selected seven rows are all present in the candidate regression list, exist
in `/home/claude/php-src-phpt`, and cover the requested ArrayObject,
SplFileObject, SplObjectStorage, autoloading, and iterator surfaces. Two
additional SPL structures are included to keep the replay selector useful if a
follow-up worker restores binaries.

## Focused Replay Results

| Row | Accepted replay | Candidate replay | Classification | Evidence path |
| --- | --- | --- | --- | --- |
| `php-src/ext/spl/tests/ArrayObject/ArrayObject_clone_other_std_props.phpt` | not run | not run | pending replay; candidate row absent | Missing historical accepted/candidate `PHPC_BIN` paths listed above |
| `php-src/ext/spl/tests/SplFileObject/SplFileObject_fgetcsv_basic.phpt` | not run | not run | pending replay; candidate row absent | Missing historical accepted/candidate `PHPC_BIN` paths listed above |
| `php-src/ext/spl/tests/SplObjectStorage/SplObjectStorage_seek.phpt` | not run | not run | pending replay; candidate row absent | Missing historical accepted/candidate `PHPC_BIN` paths listed above |
| `php-src/ext/spl/tests/autoloading/spl_autoload_call_basic.phpt` | not run | not run | pending replay; candidate row absent | Missing historical accepted/candidate `PHPC_BIN` paths listed above |
| `php-src/ext/spl/tests/DirectoryIterator_getBasename_basic_test.phpt` | not run | not run | pending replay; candidate row absent | Missing historical accepted/candidate `PHPC_BIN` paths listed above |
| `php-src/ext/spl/tests/SplFixedArray__construct_param_array.phpt` | not run | not run | pending replay; candidate row absent | Missing historical accepted/candidate `PHPC_BIN` paths listed above |
| `php-src/ext/spl/tests/SplDoublyLinkedList_offsetGet_param_array.phpt` | not run | not run | pending replay; candidate row absent | Missing historical accepted/candidate `PHPC_BIN` paths listed above |

No authoritative accepted-vs-candidate replay was possible in this lane because
the recorded release binaries no longer exist. The lane did not use unpinned
scratch binaries for adjudication.

## Commands Used

Artifact inventory, avoiding recursive `.harness/worktrees` scans:

```sh
find .harness -maxdepth 3 -type f | sort
find /home/claude/php-to-native-compiler \
  -path '/home/claude/php-to-native-compiler/.harness/worktrees' -prune \
  -o -maxdepth 4 -type f \
  \( -iname '*regression*' -o -iname '*spl*' -o -iname '*phpt*' \
     -o -iname '*manifest*' -o -iname '*replay*' \) -print
```

Evidence reports read:

```sh
sed -n '1,240p' .harness/reports/221205Z-spl.md
sed -n '1,260p' .harness/reports/221205Z-pass-regression-manifest.md
sed -n '1,260p' .harness/reports/focused-replay-cookbook.md
sed -n '1,260p' .harness/reports/phpt-binary-wrapper-availability-recheck-dev305.md
sed -n '1,220p' .harness/reports/regression-shard-report-schema.md
```

Historical binary and wrapper/php-src availability check:

```sh
python - <<'PY'
from pathlib import Path
paths = [
  '/tmp/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67/cargo-target/release/phpc',
  '/tmp/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/cargo-target/release/phpc',
  '/home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper',
  '/home/claude/php-src-phpt/run-tests.php',
]
for p in paths:
    q = Path(p)
    print(f'{p}\texists={q.exists()}\texecutable={q.exists() and q.stat().st_mode & 0o111 != 0}')
PY
git -C /home/claude/php-src-phpt rev-parse HEAD
```

Artifact join and representative row selection:

```sh
python - <<'PY'
from pathlib import Path
from collections import Counter
ACC = Path('/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67')
CAND = Path('/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377')
PHP = Path('/home/claude/php-src-phpt')

def statuses(path):
    out = {}
    for line in path.read_text(errors='replace').splitlines():
        parts = line.split('\t')
        if len(parts) >= 2:
            out[parts[1]] = parts[0]
    return out

acc = statuses(ACC / 'current-status.normalized.tsv')
cand = statuses(CAND / 'current-status.normalized.tsv')
reg = set((CAND / 'regressions-from-latest-published-passes.txt').read_text().splitlines())

allres = {}
for line in (CAND / 'all-results.txt').read_text(errors='replace').splitlines():
    st, p = line.split('\t', 1)
    idx = p.find('/php-src/')
    norm = 'php-src/' + p[idx + len('/php-src/'):] if idx >= 0 else p
    allres.setdefault(norm, set()).add(st)

spl = [row for row in reg if row.startswith('php-src/ext/spl/')]
print('spl_regressions', len(spl))
print('accepted_statuses', Counter(acc.get(row, 'MISSING') for row in spl))
print('candidate_statuses', Counter(cand.get(row, 'ABSENT') for row in spl))
print('candidate_all_results', Counter('ABSENT' if row not in allres else 'PRESENT' for row in spl))
PY
```

SQLite fallback was used because the MCP memory tools were unavailable. The
agent row, lane row, message `533`, and progress/completion events were updated
with Python's standard `sqlite3` module.

## Artifact Manifest

| Artifact | Purpose | Created by | Hash/check |
| --- | --- | --- | --- |
| `.harness/reports/focused-replay-spl-dev109.md` | Main lane 82 SPL replay/report artifact | developer-394 | tracked in Git; validate with `git diff --check -- .harness/reports/focused-replay-spl-dev109.md` |

No `/tmp` replay row file or PHPT result log was created because replay was
blocked before execution by missing historical binaries.

## Proposed Next Action

| Priority | Action | Owner type | Preconditions | Stop condition | Expected artifact |
| ---: | --- | --- | --- | --- | --- |
| 1 | Rebuild or restore authoritative accepted and candidate release `phpc` binaries, then run the seven-row SPL focused replay selector above | Developer or Integrator | Durable binaries for accepted `0b917f67a37d9ca9779d77f87173b628431c2425` and candidate `56fe9377fb46be00db5fdd30c966fdba406dc581`, plus existing wrapper and php-src checkout | Accepted replay passes; candidate replay emits row-level statuses for all seven selected rows | Focused replay result directory with row list, `results.txt`, stdout/stderr/run-tests logs, binary manifest, and updated lane report |
| 2 | If focused replay emits all seven candidate rows, replay the full 137-row SPL regression list in a lane-local evidence directory | Developer or Integrator | Priority 1 completed with valid accepted replay | All 137 SPL rows have candidate row-level statuses | Full SPL focused replay artifact, still with no public score movement |
| 3 | Fix the candidate full-gate shard directory-copy issue before any public gate retry | Harness/control-plane Developer | Confirmed missing `run-tests-harnesses/shard-03/ext/pdo/tests` and shard-04 equivalent in candidate evidence | Dry-run or focused gate no longer aborts before SPL buckets | Harness patch/test artifact, then a managed full current-score gate if assigned |

## Integration-Ready Checklist

- [x] Report includes owner, lane, mode, created timestamp, branch/worktree,
  source-edit status, full-gate status, and score-movement status.
- [x] Report lists absolute evidence paths and missing replay prerequisites.
- [x] Report defines exact SPL prefix scope and row count.
- [x] Report separates `ABSENT` candidate rows from concrete semantic failures.
- [x] Report includes 5-8 representative rows across the requested SPL surfaces.
- [x] Report states focused replay was not run and names the missing
  authoritative binaries.
- [x] Report includes exact commands used and SQLite fallback note.
- [x] Report names deterministic next actions and expected artifacts.
