# Focused Replay: Standard Scalar/Misc Regression Rows

| Field | Value |
| --- | --- |
| Title | Focused replay: standard scalar/misc regression rows |
| Owner | developer-413 |
| Lane | 86, `Focused replay: standard scalar/misc regression rows` |
| Mode | read-only |
| Created | 2026-06-05T09:52:35+00:00 |
| Branch/worktree | `work/developer-413` / `/home/claude/php-to-native-compiler/.harness/worktrees/developer-413` |
| Source edits | none |
| Full gate run | no |
| Public score movement | none |

## 2026-06-07 Revalidation Addendum

Current owner `developer-111` was assigned lane 86 after `developer-105`
ended without an accepted report. I preserved the existing integrated report
artifact and rechecked whether the original evidence was still available for a
focused accepted-vs-candidate replay.

Current filesystem state:

| Evidence | Current status |
| --- | --- |
| Integrated lane86 report artifact | present at `.harness/reports/focused-replay-standard-scalar-misc-dev117.md` |
| Candidate run root under `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377` | absent |
| Accepted run root under `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67` | absent |
| Historical accepted release binary under `/tmp/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67/cargo-target/release/phpc` | absent |
| Historical candidate release binary under `/tmp/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/cargo-target/release/phpc` | absent |
| Pinned php-src checkout | present at `/home/claude/php-src-phpt`, commit `f97ff597429a2fe633665a7e02d97c8077f9f90f` |
| PHPT wrapper | present and executable at `/home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper` |

A shallow search under `/home/claude` found no relocated
`regressions-from-latest-published-passes.txt`,
`current-status.normalized.tsv`, or `all-results.txt` paths matching the
`221205Z` or `135138Z` run ids. Because both the historical result artifacts
and the historical binaries are absent, this revalidation could not recompute
the 142-row accounting table or run an accepted-vs-candidate focused replay.

The current deterministic conclusion remains the same as the original report:
no standard scalar/misc semantic repair should start from this lane until the
historical run roots are restored or approved replacement binaries are rebuilt
for the accepted and candidate commits. The only current lane86 delta is this
evidence-availability addendum; no compiler/runtime source files were edited
and no full PHPT gate was run.

## Evidence Inputs

| Evidence | Path or value |
| --- | --- |
| Candidate gate directory | `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377` |
| Accepted baseline directory | `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67` |
| Candidate regression list | `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/regressions-from-latest-published-passes.txt` |
| Candidate status file | `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/current-status.normalized.tsv` |
| Candidate aggregate results | `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/all-results.txt` |
| Accepted status file | `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67/current-status.normalized.tsv` |
| PHPT source checkout | `/home/claude/php-src-phpt` at php-src pin `f97ff597429a2fe633665a7e02d97c8077f9f90f` |
| Wrapper | `/home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper` |
| Existing shard evidence | `/home/claude/php-to-native-compiler/.harness/reports/221205Z-standard-scalar-misc.md` |
| Replay cookbook | `/home/claude/php-to-native-compiler/.harness/reports/focused-replay-cookbook.md` |
| Harness database | `/home/claude/php-to-native-compiler/.harness/harness.sqlite3` |

The strict accepted-vs-candidate replay step is blocked because the historical
accepted and candidate release binaries are no longer present:

| Historical binary | Exists | Executable |
| --- | --- | --- |
| `/tmp/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67/cargo-target/release/phpc` | no | no |
| `/tmp/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/cargo-target/release/phpc` | no | no |

No replacement full gate or release rebuild was run in this lane.

## Scope Definition

| Scope item | Value |
| --- | --- |
| Included prefixes | `php-src/ext/standard/tests/math/`, `php-src/ext/standard/tests/general_functions/`, `php-src/ext/standard/tests/serialize/`, `php-src/ext/standard/tests/url/`, `php-src/ext/standard/tests/class_object/`, `php-src/ext/standard/tests/assert/`, `php-src/ext/standard/tests/crypt/`, `php-src/ext/standard/tests/time/`, `php-src/ext/standard/tests/versioning/`, `php-src/ext/standard/tests/misc/` |
| Excluded prefixes | none inside the included prefixes |
| Regression source | Candidate `regressions-from-latest-published-passes.txt` |
| Selection rule | Rows under `php-src/ext/standard/tests/` whose immediate test subdirectory is one of `math`, `general_functions`, `serialize`, `url`, `class_object`, `assert`, `crypt`, `time`, `versioning`, or `misc` |
| Late-priority exclusions | eval and variable-variable rows are absent from this scope |
| Owned rows | 142 |

## Accounting Summary

| Candidate artifact bucket | Rows | Definition |
| --- | ---: | --- |
| `ABSENT` from candidate status/results | 142 | No candidate status row and no normalized aggregate result row. |
| `FAILED` | 0 | Candidate status is a concrete non-PASS failure. |
| `BORKED` | 0 | Candidate status is a concrete PHPT setup/SKIPIF/parsing problem. |
| `SKIPPED` | 0 | Candidate status is skipped and no longer contributes a pass. |
| `PASSED` but still listed | 0 | Candidate still passed despite appearing in the regression list. |
| Conflicting duplicate statuses | 0 | Same PHPT path appears with more than one candidate status inside this owned set. |
| Total owned regressions | 142 | Selected rows from the regression list. |

All 142 owned rows were `PASSED` in the accepted baseline and `ABSENT` from the
candidate status/results artifacts. This points first to a gate completeness or
result-coverage problem, not to a proven standard-library semantic regression.

Directory split:

| Subdirectory | Rows |
| --- | ---: |
| `math` | 53 |
| `general_functions` | 44 |
| `serialize` | 14 |
| `class_object` | 12 |
| `url` | 10 |
| `assert` | 6 |
| `crypt` | 1 |
| `time` | 1 |
| `versioning` | 1 |
| `misc` | 0 |
| **Total** | **142** |

## Representative Rows

| Row | PHPT title | Accepted status | Candidate status | Artifact source | SKIPIF | Bucket | Why selected | Likely owner | Next action |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `php-src/ext/standard/tests/math/round_RoundingMode.phpt` | `round(): Test RoundingMode enum.` | `PASSED` | `ABSENT` | accepted/candidate normalized status join | none | Missing candidate result/status | RoundingMode and one of the `round*` rows. | harness/replay | Rebuild historical candidate binary or run focused replay with an integrator-approved substitute. |
| `php-src/ext/standard/tests/math/acos_basic.phpt` | `Test return type and value for expected input acos()` | `PASSED` | `ABSENT` | accepted/candidate normalized status join | none | Missing candidate result/status | Simple math builtin baseline. | harness/replay | Include in first focused replay if math rows need a no-SKIPIF smoke. |
| `php-src/ext/standard/tests/general_functions/var_dump_arrays.phpt` | `Test var_dump() function with arrays` | `PASSED` | `ABSENT` | accepted/candidate normalized status join | none | Missing candidate result/status | Representative `var_dump()` formatting row. | harness/replay | Replay after candidate binary is restored. |
| `php-src/ext/standard/tests/general_functions/is_callable_variation1.phpt` | `Test is_callable() function : usage variations - undefined functions` | `PASSED` | `ABSENT` | accepted/candidate normalized status join | none | Missing candidate result/status | Representative predicate/callability row. | harness/replay | Replay after candidate binary is restored. |
| `php-src/ext/standard/tests/serialize/unserialize_allowed_classes_option_invalid_array.phpt` | `Test unserialize() with array allowed_classes and nonsensical values` | `PASSED` | `ABSENT` | accepted/candidate normalized status join | none | Missing candidate result/status | Representative `unserialize()` options row. | harness/replay | Replay after candidate binary is restored. |
| `php-src/ext/standard/tests/serialize/serialize_globals_var_refs.phpt` | `Reference IDs should be correctly generated when $GLOBALS is serialized` | `PASSED` | `ABSENT` | accepted/candidate normalized status join | none | Missing candidate result/status | Serialization/reference interaction row. | harness/replay | Replay after candidate binary is restored. |
| `php-src/ext/standard/tests/url/parse_url_basic_004.phpt` | `Test parse_url() function: Parse a load of URLs without specifying PHP_URL_PORT as the URL component` | `PASSED` | `ABSENT` | accepted/candidate normalized status join | none | Missing candidate result/status | Representative URL parsing row. | harness/replay | Replay after candidate binary is restored. |
| `php-src/ext/standard/tests/assert/assert_basic2.phpt` | `assert() - basic - correct call back values before and after assert.` | `PASSED` | `ABSENT` | accepted/candidate normalized status join | none | Missing candidate result/status | Representative assertion callback row. | harness/replay | Replay after candidate binary is restored. |

The one-row `crypt`, `time`, and `versioning` buckets are also absent-status
rows and can be appended to a second replay list if the first eight rows
confirm a coverage/control-plane issue rather than semantic failures.

## Symptom Buckets

| Bucket | Rows | Evidence | Representative rows | Interpretation |
| --- | ---: | --- | --- | --- |
| Missing candidate result/status | 142 | Present in the accepted PASS baseline and candidate regression list, absent from candidate normalized status and aggregate results. | `round_RoundingMode.phpt`, `var_dump_arrays.phpt`, `serialize_globals_var_refs.phpt`, `parse_url_basic_004.phpt` | Control-plane or shard-completeness investigation comes before semantic repair. |
| Concrete runtime/compiler failure | 0 | No owned row has a candidate `FAILED` status. | none | No standard scalar/misc semantic repair should be started from this report alone. |
| SKIPIF/wrapper/environment | 0 | No owned row has a candidate `BORKED` status. | none | Wrapper/SKIPIF constants are not the observed mechanism for this shard. |
| Duplicate/normalization conflict | 0 | No conflicting duplicate status was observed inside this owned set. | none | Not the local dominant issue. |
| Pending replay | 142 | Historical accepted/candidate binaries are absent under `/tmp`. | all selected rows | Rebuild or restore binaries before claiming replay reproduction. |

## Replay Results

| Row | Accepted replay | Candidate replay | Classification | Evidence path |
| --- | --- | --- | --- | --- |
| `php-src/ext/standard/tests/math/round_RoundingMode.phpt` | not run | not run | pending, historical binaries missing | this report and candidate/accepted status artifacts |
| `php-src/ext/standard/tests/math/acos_basic.phpt` | not run | not run | pending, historical binaries missing | this report and candidate/accepted status artifacts |
| `php-src/ext/standard/tests/general_functions/var_dump_arrays.phpt` | not run | not run | pending, historical binaries missing | this report and candidate/accepted status artifacts |
| `php-src/ext/standard/tests/general_functions/is_callable_variation1.phpt` | not run | not run | pending, historical binaries missing | this report and candidate/accepted status artifacts |
| `php-src/ext/standard/tests/serialize/unserialize_allowed_classes_option_invalid_array.phpt` | not run | not run | pending, historical binaries missing | this report and candidate/accepted status artifacts |
| `php-src/ext/standard/tests/serialize/serialize_globals_var_refs.phpt` | not run | not run | pending, historical binaries missing | this report and candidate/accepted status artifacts |
| `php-src/ext/standard/tests/url/parse_url_basic_004.phpt` | not run | not run | pending, historical binaries missing | this report and candidate/accepted status artifacts |
| `php-src/ext/standard/tests/assert/assert_basic2.phpt` | not run | not run | pending, historical binaries missing | this report and candidate/accepted status artifacts |

## Commands Used

Artifact inspection:

```sh
sed -n '1,260p' .harness/reports/221205Z-standard-scalar-misc.md
sed -n '1,260p' .harness/reports/focused-replay-cookbook.md
sed -n '1,260p' .harness/reports/regression-shard-report-schema.md
sed -n '1,260p' .harness/reports/221205Z-pass-regression-manifest.md
find /home/claude/php-to-native-compiler/.harness -maxdepth 4 -type f -path '*/reports/*' -print
```

SQLite status/event update, using Python `sqlite3` because the SQLite MCP tools
and local `sqlite3` binary were unavailable in this session:

```sh
python3 - <<'PY'
import sqlite3, datetime, json
path='/home/claude/php-to-native-compiler/.harness/harness.sqlite3'
now=datetime.datetime.now(datetime.timezone.utc).replace(microsecond=0).isoformat()
con=sqlite3.connect(path)
cur=con.cursor()
cur.execute("update agents set current_status=?, last_seen_at=?, notes=? where name=?", (
    'in_progress: lane86 standard scalar/misc focused replay report',
    now,
    'Accepted lane86 report-only assignment; DEVELOPMENT.md absent in checkout; SQLite MCP unavailable so using Python sqlite3; locating 221205Z artifacts and writing .harness/reports focused replay report.',
    'developer-413',
))
cur.execute("insert into events(ts,type,message,agent_name,payload_json) values(?,?,?,?,?)", (
    now,
    'memory_record_event',
    'developer-413 accepted lane86 focused replay/report assignment for standard scalar/misc regression rows',
    'developer-413',
    json.dumps({'lane_id':86,'artifact':'.harness/reports/focused-replay-standard-scalar-misc-dev117.md','mcp_unavailable':True}),
))
con.commit()
print(now)
PY
```

Fresh owned-row accounting and title/SKIPIF extraction:

```sh
python3 - <<'PY'
from pathlib import Path
from collections import Counter
ACC=Path('/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67')
CAND=Path('/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377')
PHP_SRC=Path('/home/claude/php-src-phpt')
target={'math','general_functions','serialize','url','class_object','assert','crypt','time','versioning','misc'}
regs=(CAND/'regressions-from-latest-published-passes.txt').read_text().splitlines()
rows=[r for r in regs if r.startswith('php-src/ext/standard/tests/') and len(r.split('/'))>=5 and r.split('/')[4] in target]

def status_map(root):
    out={}
    for line in (root/'current-status.normalized.tsv').read_text().splitlines():
        parts=line.split('\t')
        if len(parts)>=2:
            out[parts[1]]=parts[0]
    return out

acc=status_map(ACC)
cand=status_map(CAND)
all_results=set()
for line in (CAND/'all-results.txt').read_text(errors='replace').splitlines():
    if 'php-src/' in line:
        idx=line.find('php-src/')
        all_results.add(line[idx:].split()[0])
counts=Counter(r.split('/')[4] for r in rows)
buckets=Counter()
for r in rows:
    if r in cand:
        buckets[cand[r]] += 1
    elif r in all_results:
        buckets['RESULTS_ONLY'] += 1
    else:
        buckets['ABSENT'] += 1
print('owned_rows', len(rows))
print('counts_by_dir', dict(sorted(counts.items())))
print('buckets', dict(sorted(buckets.items())))
print('accepted_non_pass', [r for r in rows if acc.get(r) != 'PASSED'][:10])
PY
```

Historical replay binary check:

```sh
python3 - <<'PY'
from pathlib import Path
for p in [
 Path('/tmp/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67/cargo-target/release/phpc'),
 Path('/tmp/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/cargo-target/release/phpc')
]:
    print(p, 'exists=', p.exists(), 'executable=', p.exists() and p.stat().st_mode & 0o111 != 0)
PY
```

No focused replay or full gate command was run. Focused replay requires restored
or rebuilt historical accepted/candidate binaries. A full gate is outside this
lane's scope.

Developer-111 revalidation commands:

```sh
python3 - <<'PY'
from pathlib import Path
for label, p in [
    ('accepted_bin', Path('/tmp/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67/cargo-target/release/phpc')),
    ('candidate_bin', Path('/tmp/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/cargo-target/release/phpc')),
    ('candidate_regressions', Path('/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/regressions-from-latest-published-passes.txt')),
    ('candidate_status', Path('/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/current-status.normalized.tsv')),
    ('candidate_all_results', Path('/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/all-results.txt')),
    ('accepted_status', Path('/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67/current-status.normalized.tsv')),
    ('php_src', Path('/home/claude/php-src-phpt')),
    ('wrapper', Path('/home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper')),
]:
    print(f'{label}\t{p}\texists={p.exists()}\texecutable={p.exists() and p.is_file() and (p.stat().st_mode & 0o111)!=0}')
PY
```

```sh
find /home/claude -maxdepth 5 \
  \( -name 'regressions-from-latest-published-passes.txt' \
     -o -name 'current-status.normalized.tsv' \
     -o -name 'all-results.txt' \) 2>/dev/null |
  rg '221205|135138|phpt-full-current-score'
```

```sh
git -C /home/claude/php-src-phpt rev-parse HEAD
test -x /home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper
```

## SQLite Status

The expected harness SQLite MCP tools were not exposed to this session, and the
local `sqlite3` command was not installed. I used Python's standard `sqlite3`
module as the deterministic fallback to update `agents`, record the lane claim
event, and later mark completion.

## Artifact Manifest

| Artifact | Purpose | Created by | Hash/check |
| --- | --- | --- | --- |
| `.harness/reports/focused-replay-standard-scalar-misc-dev117.md` | Main lane86 focused replay/report artifact | developer-413 | `git diff --check -- .harness/reports/focused-replay-standard-scalar-misc-dev117.md` passed |

## Proposed Next Action

| Priority | Action | Owner type | Preconditions | Stop condition | Expected artifact |
| ---: | --- | --- | --- | --- | --- |
| 1 | Restore or rebuild release `phpc` binaries for accepted commit `0b917f67a37d9ca9779d77f87173b628431c2425` and candidate commit `56fe9377fb46be00db5fdd30c966fdba406dc581`, then run focused `run-tests.php` replay for the eight selected no-SKIPIF rows only. | Developer or Integrator | Integrator approval for rebuild cost or restored historical run roots under `/tmp`. | Accepted replay passes; candidate replay either produces concrete row statuses or confirms candidate result-coverage loss. | Focused replay logs under `/tmp/phpt-focused-replay-lane86-*` and an updated report section. |
| 2 | If candidate replay rows still vanish or cannot produce status rows, patch gate completeness/accounting to fail when expected PHPT paths are absent from candidate normalized status/results. | Developer, control-plane lane | Replay or artifact evidence confirming absent rows are a coverage issue. | A focused harness/control-plane test proves absent expected rows block publication before score comparison. | Control-plane patch, tests, and report update. |
| 3 | Start semantic repair only for rows that focused replay converts from `ABSENT` to concrete `FAILED`/`BORKED`. | Developer, semantic lane | Row-level failing stdout/run-tests evidence. | One narrow source fix with PHP comparison proof for that row. | Compiler/runtime patch plus focused PHPT evidence. |
