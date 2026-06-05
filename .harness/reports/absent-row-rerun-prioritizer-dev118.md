# Absent-Row Rerun Prioritizer

| Field | Value |
| --- | --- |
| Title | Absent-row rerun prioritizer for post-abort gate recovery |
| Owner | developer-414 |
| Lane | work_lanes#87, `Absent-row rerun prioritizer for post-abort gate recovery` |
| Mode | read-only report |
| Created | 2026-06-05T09:59Z |
| Branch/worktree | `work/developer-414`, `/home/claude/php-to-native-compiler/.harness/worktrees/developer-414` |
| Source edits | none; report artifact only |
| Full gate run | no |
| Public score movement | none; accepted score remains `7873 / 20294 = 38.79%` |

## Decision

The 221205Z candidate remains blocked. Its absent rows should be treated as
incomplete gate evidence, not as proven semantic PHP failures. The immediate
post-abort recovery order is:

1. Repair/prove the shard harness directory-layout issue for the redirected PDO
   rows that aborted shards 03 and 04.
2. Rerun/recover shard 04 absent rows first (`307`), then shard 03 (`199`).
3. Rerun/recover shard 05 (`297`) next, because it has nearly the same absent
   volume as shard 04 despite reaching `Report saved to:`.
4. Rerun/recover shard 06 (`188`), then shards 01 (`74`) and 02 (`71`).
5. Only after every selected absent row has a candidate status should semantic
   repair lanes be assigned from the resulting `FAILED`/`BORKED` rows.

## Evidence Inputs

| Evidence | Path or value |
| --- | --- |
| Candidate gate directory | `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377` |
| Accepted baseline directory | `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67` |
| Candidate regression list | `regressions-from-latest-published-passes.txt` |
| Candidate status file | `current-status.normalized.tsv` |
| Candidate aggregate results | `all-results.txt` |
| Gate script | `run_gate.sh` |
| PHPT source checkout | `/home/claude/php-src-phpt` at `f97ff597429a2fe633665a7e02d97c8077f9f90f` |
| Harness database | `/home/claude/php-to-native-compiler/.harness/harness.sqlite3` |

The candidate artifact did not archive `all-tests.txt`, `sharded-tests.txt`, or
`shard-XX.tests`; only `serialized-openbasedir.tests` is present. Shard
membership below is therefore reconstructed from the pinned php-src checkout and
the archived `run_gate.sh` round-robin rule:

```sh
find "$PHP_SRC" -path "$PHP_SRC/.git" -prune -o -type f -name '*.phpt' -print | sort > "$RUN_ROOT/all-tests.txt"
find "$PHP_SRC/tests/security" -type f -name 'open_basedir_*.phpt' -print | sort > "$RUN_ROOT/serialized-openbasedir.tests"
awk 'NR==FNR{skip[$0]=1; next} !($0 in skip)' "$RUN_ROOT/serialized-openbasedir.tests" "$RUN_ROOT/all-tests.txt" > "$RUN_ROOT/sharded-tests.txt"
awk -v shards="$SHARDS" -v root="$RUN_ROOT" '{ idx=((NR-1) % shards) + 1; file=sprintf("%s/shard-%02d.tests", root, idx); print > file }' "$RUN_ROOT/sharded-tests.txt"
```

## Accounting Summary

| Candidate artifact bucket | Rows | Definition |
| --- | ---: | --- |
| `ABSENT` from candidate status/results | 1136 | Accepted PASS regression rows with no candidate status row and no aggregate result row. |
| `FAILED` | 27 | Concrete candidate failure rows. |
| `BORKED` | 3 | Concrete candidate setup/SKIPIF rows. |
| `PASSED` but still listed | 0 | No regression row remains in candidate normalized passes. |
| Total latest-public PASS regressions | 1166 | Matches `pass-regression-summary.tsv`. |

Absent rows dominate `97.4%` of the 1166 regressions. They must be recovered by
control-plane rerun/completeness work before being used as product bug evidence.

## Absent Rows By Shard

| Reconstructed shard | Absent rows | Shard terminal evidence | Recovery classification | Rerun priority |
| --- | ---: | --- | --- | ---: |
| shard-04 | 307 | `ERROR: cannot open directory: .../run-tests-harnesses/shard-04/ext/pdo/tests`; no `run-tests.log` | known directory-abort/control-plane absence | 1 |
| shard-03 | 199 | `ERROR: cannot open directory: .../run-tests-harnesses/shard-03/ext/pdo/tests`; no `run-tests.log` | known directory-abort/control-plane absence | 2 |
| shard-05 | 297 | reached `Report saved to:` | aggregate/status completeness gap; not yet semantic | 3 |
| shard-06 | 188 | reached `Report saved to:` | aggregate/status completeness gap; not yet semantic | 4 |
| shard-01 | 74 | reached `Report saved to:` | aggregate/status completeness gap; not yet semantic | 5 |
| shard-02 | 71 | reached `Report saved to:` | aggregate/status completeness gap; not yet semantic | 6 |

The known abort shards account for `506` absent rows. The other `630` absent
rows still require expected-path reconciliation because result files can exist
while individual PHPT statuses are missing.

## Directory Priority

These are exact parent-directory counts among the 1136 absent rows. They are
useful rerun selectors, but they are not semantic failure counts.

| Directory | Absent rows | Representative rows |
| --- | ---: | --- |
| `php-src/ext/standard/tests/strings` | 197 | `005.phpt` shard-03; `006.phpt` shard-04; `addcslashes_001.phpt` shard-03 |
| `php-src/ext/standard/tests/array` | 175 | `006.phpt` shard-04; `007.phpt` shard-05; `array_change_key_case.phpt` shard-05 |
| `php-src/ext/standard/tests/file` | 160 | `005_error.phpt` shard-04; `005_variation2.phpt` shard-01; `006_variation1.phpt` shard-04 |
| `php-src/ext/reflection/tests` | 104 | `001.phpt` shard-03; `007.phpt` shard-03; `014.phpt` shard-04 |
| `php-src/ext/spl/tests` | 73 | `bug28822.phpt` shard-04; `bug31185.phpt` shard-05; `bug33136.phpt` shard-04 |
| `php-src/ext/standard/tests/math` | 53 | `acos_basic.phpt`; `acos_basiclong_64bit.phpt`; `asin_basic.phpt` |
| `php-src/ext/standard/tests/array/sort` | 49 | `array_multisort_basic1.phpt`; `array_multisort_natural_case.phpt` |
| `php-src/ext/standard/tests/general_functions` | 44 | `001.phpt`; `002.phpt`; `008.phpt` |
| `php-src/ext/spl/tests/ArrayObject` | 29 | `array_001.phpt` shard-04; `array_006.phpt` shard-03 |
| `php-src/ext/spl/tests/SplFileObject` | 19 | `SplFileObject_fgetcsv_basic.phpt`; `SplFileObject_fputcsv_variation1.phpt` |

Cluster totals by extension area: `ext/standard=792`, `ext/spl=137`,
`ext/reflection=110`, `ext/uri=41`, `ext/posix=16`, `ext/tokenizer=14`,
`ext/xmlreader=8`, `ext/session=7`, `ext/random=4`, `sapi=3`, and one each
for `ext/phar`, `ext/sodium`, `ext/zip`, and `ext/zlib`.

## Minimal Rerun Ordering

| Priority | Rerun selector | Rows recovered | Preconditions | Stop condition | Expected artifact |
| ---: | --- | ---: | --- | --- | --- |
| 0 | Redirect smoke: `php-src/ext/pdo_mysql/tests/common.phpt` and `php-src/ext/pdo_pgsql/tests/common.phpt` | control-plane proof, not scored rows | Shard harness either invokes php-src `run-tests.php` in place or mirrors root dirs such as `ext/` under each shard harness. | No `run-tests-harnesses/.../ext/pdo/tests` abort, and both rows produce logs/results. | Focused smoke report/logs. |
| 1 | All absent rows reconstructed for shard-04 | 307 | Priority 0 passed; candidate binary and wrapper available. | Every selected row has a candidate status/result row. | shard-04 absent-row result manifest. |
| 2 | All absent rows reconstructed for shard-03 | 199 | Priority 0 passed; candidate binary and wrapper available. | Every selected row has a candidate status/result row. | shard-03 absent-row result manifest. |
| 3 | All absent rows reconstructed for shard-05 | 297 | Expected-path completeness checker active; do not rely on `missing_results=0`. | Every selected row has a candidate status/result row. | shard-05 absent-row result manifest. |
| 4 | All absent rows reconstructed for shard-06 | 188 | Same as priority 3. | Every selected row has a candidate status/result row. | shard-06 absent-row result manifest. |
| 5 | All absent rows reconstructed for shard-01 | 74 | Same as priority 3. | Every selected row has a candidate status/result row. | shard-01 absent-row result manifest. |
| 6 | All absent rows reconstructed for shard-02 | 71 | Same as priority 3. | Every selected row has a candidate status/result row. | shard-02 absent-row result manifest. |
| 7 | Directory-focused semantic triage from newly concrete rows | TBD | Priorities 1-6 have produced candidate statuses. | Direct `FAILED`/`BORKED` rows are assigned to semantic/env lanes with logs. | Repair-lane proposals with row evidence. |

## Representative Rows

| Row | Accepted status | Candidate status | Bucket | Why selected | Next action |
| --- | --- | --- | --- | --- | --- |
| `php-src/ext/standard/tests/strings/006.phpt` | `PASSED` | `ABSENT` | directory-abort/control-plane | high-volume strings directory on shard-04 | rerun with shard-04 absent selector after redirect smoke passes |
| `php-src/ext/standard/tests/file/005_error.phpt` | `PASSED` | `ABSENT` | directory-abort/control-plane | high-volume file directory on shard-04 | rerun with shard-04 absent selector |
| `php-src/ext/reflection/tests/001.phpt` | `PASSED` | `ABSENT` | directory-abort/control-plane | high-volume reflection directory on shard-03 | rerun with shard-03 absent selector |
| `php-src/ext/standard/tests/array/007.phpt` | `PASSED` | `ABSENT` | aggregate/status completeness | shard-05 reached `Report saved to:` but row is missing | rerun with shard-05 absent selector and expected-path check |
| `php-src/ext/standard/tests/array/array_change_key_case_flag_error.phpt` | `PASSED` | `ABSENT` | aggregate/status completeness | shard-06 reached `Report saved to:` but row is missing | rerun with shard-06 absent selector and expected-path check |
| `php-src/ext/spl/tests/ArrayObject/array_001.phpt` | `PASSED` | `ABSENT` | directory-abort/control-plane | SPL ArrayObject coverage, shard-04 | rerun with shard-04 absent selector |
| `php-src/ext/posix/tests/001.phpt` | `PASSED` | `ABSENT` | directory-abort/control-plane | small extension cluster on shard-03 | rerun with shard-03 absent selector |

Eval and variable-variable rows were not used for prioritization.

## Commands Used

Artifact inspection only:

```sh
sed -n '1,240p' .harness/reports/221205Z-pass-regression-manifest.md
sed -n '1,220p' .harness/reports/221205Z-shard-abort-root-cause.md
sed -n '1,220p' .harness/reports/221205Z-regression-status-summary-refresh-dev313.md
wc -l "$CAND/regressions-from-latest-published-passes.txt" "$CAND/current-status.normalized.tsv" "$CAND/all-results.txt"
sed -n '1,80p' "$CAND/pass-regression-summary.tsv"
sed -n '1,80p' "$CAND/shard-exit-codes.tsv"
find "$CAND" -maxdepth 2 -name 'shard-*.tests' -o -name 'all-tests.txt' -o -name 'sharded-tests.txt' -o -name 'serialized-openbasedir.tests'
git -C /home/claude/php-src-phpt rev-parse HEAD
```

SQLite status/event updates used Python's standard `sqlite3` module because the
MCP memory tools and local `sqlite3` CLI were unavailable in this session.

The Python artifact parser recomputed:

```text
overall {'regressions': 1166, 'absent': 1136, 'failed': 27, 'borked': 3}
absent_by_shard: 01=74, 02=71, 03=199, 04=307, 05=297, 06=188
top parent dirs: strings=197, array=175, file=160, reflection=104, spl=73
```

No focused replay, source build, cargo test, or full PHPT gate was run.

## Artifact Manifest

| Artifact | Purpose | Created by | Check |
| --- | --- | --- | --- |
| `.harness/reports/absent-row-rerun-prioritizer-dev118.md` | Main lane 87 report | developer-414 | `git diff --check -- .harness/reports/absent-row-rerun-prioritizer-dev118.md` |

## Integration-Ready Checklist

- [x] Exact candidate and accepted evidence paths named.
- [x] Source edits, full-gate status, and public-score status stated.
- [x] `ABSENT` rows separated from concrete `FAILED`/`BORKED` rows.
- [x] Directory-abort absent rows separated from other aggregate/status absence.
- [x] Rerun priority list has preconditions, stop conditions, and expected artifacts.
- [x] Eval and variable-variable rows kept out of near-term priority.
