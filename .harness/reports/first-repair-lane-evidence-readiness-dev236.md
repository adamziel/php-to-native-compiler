# First Repair-Lane Evidence Readiness Review

Agent: developer-416
Lane: 117
Generated: 2026-06-05T09:55Z

Scope: read-only M0/M1 evidence review. No compiler, runtime, harness source,
php-src, or PHPT gate files were edited. No full PHPT gate was run, and this
report does not move public score.

`DEVELOPMENT.md` was requested by the harness role instructions but is absent
under `/home/claude/php-to-native-compiler` and this worktree.

## Decision

No product/compiler-runtime repair implementation lane is ready to start from
the current 221205Z evidence.

The only implementation class with enough deterministic evidence is
control-plane/gate repair: command selection, idle-alert filtering, shard
run-tests layout, and strict expected-row completeness. Lane8 and lane100 have
fresh scheduler-visible completion evidence, but lane119 is the assigned
proof-evaluator for those patches. The next PHPT-gate repair candidate is the
shard harness/completeness path from lane69 plus lane78 smoke, not a broad PHP
feature lane.

Product repair candidates remain blocked by missing accepted-vs-candidate
focused replay evidence. The dominant 221205Z symptom is still absent rows:
`1136 / 1166` latest-public PASS regressions have no candidate row in
`current-status.normalized.tsv` or `all-results.txt`. Direct `FAILED` and
`BORKED` rows are better candidates, but lane68's direct triage artifact is
still in progress and the historical accepted/candidate `PHPC_BIN` pair is
missing.

Accepted public score remains `7873 / 20294 = 38.79%`. The blocked 221205Z
candidate remains `7197 / 20294 = 35.46%` with `1166` latest-public PASS
regressions.

## Evidence Snapshot

Authoritative evidence roots:

- Accepted baseline:
  `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67`
- Blocked candidate:
  `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377`
- Harness DB:
  `/home/claude/php-to-native-compiler/.harness/harness.sqlite3`

Low-CPU recomputation in this lane:

| Check | Result |
| --- | ---: |
| Unique regression rows | `1166` |
| Candidate `ABSENT` rows | `1136` |
| Candidate `FAILED` rows | `27` |
| Candidate `BORKED` rows | `3` |
| Historical accepted `PHPC_BIN` | missing |
| Historical candidate `PHPC_BIN` | missing |
| PHPT wrapper | present and executable |
| Pinned `php-src` `run-tests.php` | present and executable |

Fresh scheduler state observed during this review:

- `work_lanes.id=8`: `completed` on `work/developer-402`; event `94637`
  records `python -m unittest discover -s .harness/tests -v` with 7 tests
  passed and `discover_test_command` returning `tools/run-tests.sh`.
- `work_lanes.id=100`: `completed` on `work/developer-402`; same event
  records focused verification for lane8/lane100 control-plane fixes.
- `work_lanes.id=119`: still owns read-only proof evaluation for lane8/lane100
  outputs, so this report does not accept or reject those patches.

## Readiness Matrix

| Candidate | Class | Readiness | Reason |
| --- | --- | --- | --- |
| Shard harness directory layout and expected-row completeness | `M1-control-plane` | Evidence-ready candidate | Lane69 identifies copied `run-tests.php` plus missing `ext/pdo/tests` layout as the shard-03/04 abort cause. Candidate artifacts also prove `aggregate-warnings.tsv` missed row-level incompleteness. Needs manager-owned source path and lane78 smoke proof before full gate restart. |
| Command selection and stale-agent idle alert filtering | `M1-control-plane` | Already reported completed, pending evaluator | DB event `94637` records focused .harness tests and command-selection proof. Lane119 is assigned to evaluate patch proof and before/after counts. |
| Rebuild durable accepted/candidate PHPT binaries | `M0/M1 replay enabler` | Ready as an enabling lane, not a repair | The wrapper and pinned php-src are available, but historical `/tmp/.../release/phpc` binaries are gone. Focused replay lanes need rebuilt durable binaries for commits `0b917f67` and `56fe9377`. |
| Direct readonly/internal property failures | `M0-direct` | Not M2-ready yet | Rows have direct `FAILED` status, but lane68 triage is still active and no final focused replay/postcheck artifact is present. Likely owner surfaces are object/property diagnostics and internal class metadata. |
| Direct SKIPIF constant `BORKED` rows | `M0-direct` | Not M2-ready yet | Rows identify missing `INTL_ICU_VERSION`, `ZEND_THREAD_SAFE`, and `PCRE_JIT_SUPPORT`, but need replay/environment classification and exact constant-surface ownership before implementation. Passing SKIPIF would not imply full intl/openssl/pcre support. |
| Standard array/string/file/SPL/reflection absent clusters | `M0-replay` | Not implementation-ready | These are mostly or entirely absent from candidate artifacts. Source repair would be guessing until focused replay proves semantic failures. |
| Runtime candidates from lanes 61/66 | Integration-only | Not first PHPT repair | They have focused `php_runtime --lib` evidence and merge prerequisites, but no zero-regression public PHPT gate and no 221205Z row repair evidence. Integration belongs to Integrator, not a new repair lane. |
| Self-selected product slices such as `str_ireplace`/`similar_text` | Quarantined | Not first repair | Some have tests/docs/CLI proof, but reports explicitly quarantine them because they were not selected from current M0/M1 regression evidence. |

## Candidate Next Lanes

These are candidates only; lane117 does not assign source work.

### 1. Gate Completeness And Shard Harness Layout

Recommended first deterministic repair class: `M1-control-plane`.

Minimum evidence already present:

- Lane69 shows shard-03 and shard-04 aborted because copied
  `run-tests.php` made PHPT `REDIRECTTEST` `__DIR__` probes resolve under
  incomplete `run-tests-harnesses/shard-*` directories.
- The aborted rows include PDO redirect probes that looked for
  `run-tests-harnesses/shard-03/ext/pdo/tests` and
  `run-tests-harnesses/shard-04/ext/pdo/tests`.
- Candidate artifacts lack `run-tests.log` for shards 03 and 04 and have
  `18949` result rows versus `21827` recorded PHPT files.
- `aggregate-warnings.tsv` reporting `missing_results=0` is insufficient
  because it only proves result files existed.

Missing before implementation completion:

- Owned source path for the full-gate script or generator. Existing reports
  identify archived `run_gate.sh`, but an implementation lane must name the
  durable source file that creates it.
- A focused lane78 smoke using `ext/pdo_mysql/tests/common.phpt` and
  `ext/pdo_pgsql/tests/common.phpt`, proving no copied-harness
  `ext/pdo/tests` abort.
- Postcheck evidence that every expected PHPT path has a normalized row, every
  shard has `run-tests.log`, and shard assignment lists are archived.
- Documentation of the gate evidence requirement. This is not a PHP language
  support claim and should not edit `docs/SUPPORT.md` as feature support.

### 2. Durable Focused Replay Binary Rebuild

Recommended enabling class: `M0/M1 replay enabler`.

Minimum evidence already present:

- The wrapper exists and is executable:
  `/home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper`.
- The pinned php-src checkout exists at
  `/home/claude/php-src-phpt`.
- Both source commits exist, but historical accepted and candidate release
  binaries under `/tmp/phpt-full-current-score-*` are gone.

Missing before replay lanes can promote product work:

- Durable rebuilds for accepted `0b917f67a37d9ca9779d77f87173b628431c2425`
  and candidate `56fe9377fb46be00db5fdd30c966fdba406dc581`.
- Manifest with source commit, build command, target directory, binary sha256,
  and one `phpc run` smoke for each binary.
- Focused accepted-vs-candidate replay results with `results.txt`,
  `run-tests.log`, `stdout.log`, `stderr.log`, row list, and exit status.

### 3. Direct Non-PASS Repair Candidates

Recommended class after lane68: `M0-direct`, possibly `M2-repair` after replay.

Candidate row families:

- Readonly/internal property diagnostics:
  `DirectoryClass_readonly_*`, `DatePeriod_*readonly*`,
  `BcMath\\Number` property rows, Zend readonly/property hook rows, and
  `xmlreader/014.phpt`.
- SKIPIF constants:
  `INTL_ICU_VERSION`, `ZEND_THREAD_SAFE`, and `PCRE_JIT_SUPPORT`.
- Lifecycle/exception/iterator rows:
  destructor ordering, assert/uncaught exception behavior, serialize error
  chains, and legacy iterator/class lifecycle rows.

Missing before any M2 product lane:

- A final direct triage artifact from lane68 with preserved failure text.
- Focused replay against rebuilt accepted/candidate binaries.
- Exact owner files and a narrow behavior target.
- Focused Rust tests, a `phpc run` or focused PHPT CLI exercise path, docs and
  `docs/PROGRESS.md` updates, and named unsupported edges.

## Explicit Non-Ready Areas

Do not open implementation from the large absent clusters yet:

- Standard arrays: `249` absent rows.
- Standard strings: `197` absent rows.
- Standard filesystem/streams/directories: mostly absent rows.
- SPL: `137` absent rows.
- Reflection: `110` absent rows.

These are support-risk and replay targets, not current implementation proof.
`eval` and variable-variable rows remain late-priority and should not drive
first-wave repair work.

## Commands And Queries Used

Required docs and current report reads:

```sh
sed -n '1,220p' AGENTS.md
sed -n '1,220p' docs/ARCHITECTURE.md
sed -n '1,220p' docs/SUPPORT.md
sed -n '1,220p' README.md
sed -n '1,220p' docs/LOOP_MEMORY.md
sed -n '1,260p' .harness/reports/first-repair-lane-proposals.md
sed -n '1,280p' .harness/reports/regression-repair-backlog-template.md
sed -n '1,260p' .harness/reports/221205Z-pass-regression-manifest.md
sed -n '1,260p' .harness/reports/221205Z-shard-abort-root-cause.md
sed -n '1,240p' .harness/reports/phpt-binary-wrapper-availability-recheck-dev305.md
```

`DEVELOPMENT.md` lookup:

```sh
rg --files /home/claude/php-to-native-compiler | rg '(^|/)DEVELOPMENT\.md$'
find /home/claude/php-to-native-compiler -name DEVELOPMENT.md -print
```

SQLite was accessed through Python's standard library because neither the
SQLite MCP wrappers nor the `sqlite3` CLI were available:

```sh
python3 - <<'PY'
import sqlite3
con = sqlite3.connect('/home/claude/php-to-native-compiler/.harness/harness.sqlite3')
con.row_factory = sqlite3.Row
for row in con.execute('select id,title,status,branch from work_lanes where id in (8,100,117) order by id'):
    print(dict(row))
PY
```

Artifact recomputation:

```sh
python3 - <<'PY'
from pathlib import Path
from collections import Counter
cand = Path('/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377')
rows = [line.strip() for line in (cand / 'regressions-from-latest-published-passes.txt').read_text().splitlines() if line.strip()]
status = {}
for line in (cand / 'current-status.normalized.tsv').read_text().splitlines():
    parts = line.split('\t')
    if len(parts) >= 2 and parts[1].startswith('php-src/'):
        status[parts[1]] = parts[0]
print(len(rows), Counter(status.get(row, 'ABSENT') for row in rows))
PY
```

## Bottom Line

Start or continue control-plane repair and proof work first. Do not start a
compiler/runtime PHP feature repair lane from the 221205Z absent clusters until
focused replay converts specific rows into semantic failures. Direct
`FAILED`/`BORKED` rows are the closest product candidates, but they still need
lane68 triage plus rebuilt accepted/candidate replay binaries before they meet
the repository's implementation standard.
