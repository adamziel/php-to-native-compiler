# 221205Z standard array regression shard

## Scope

This is diagnostic/control-plane work only. It does not edit compiler,
runtime, source, support docs, or php-src files, does not run a full PHPT gate,
and cannot move the accepted public PHPT score.

Artifacts inspected:

- `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/regressions-from-latest-published-passes.txt`
- `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/current-status.normalized.tsv`
- `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/all-results.txt`
- `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/shard-*/{results.txt,stdout.log,stderr.log,run-tests.log,exit.tsv}`
- `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/{counts.tsv,public-comparable-score.tsv,pass-regression-summary.tsv,environment.txt,run_gate.sh,evidence-files.sha256}`

`DEVELOPMENT.md` was requested by the harness prompt but is not present under
`/home/claude/php-to-native-compiler`.

## Exact count

The candidate-vs-accepted regression artifact contains 1,166 latest-published
PASS regressions. Filtering it to `php-src/ext/standard/tests/array/` gives
exactly 249 rows.

All 249 rows are accepted/latest-published PASS rows by construction of
`regressions-from-latest-published-passes.txt`.

## Candidate status breakdown

| Source | Candidate status | Count |
| --- | ---: | ---: |
| `current-status.normalized.tsv` join | ABSENT | 249 |
| `current-status.normalized.tsv` join | FAILED/BORKED/SKIPPED/PASSED | 0 |
| `all-results.txt` join | ABSENT | 249 |
| `all-results.txt` join | FAILED/BORKED/SKIPPED/PASSED | 0 |

No row-level PHPT diff, failure, or BORK diagnostic is available for these 249
paths in the current candidate artifacts. They are regressions because the
accepted/latest-published pass set contains them and the candidate result set
does not.

For contrast, `all-results.txt` does contain other standard-array rows:

| Candidate standard-array rows seen in `all-results.txt` | Count |
| --- | ---: |
| PASSED | 522 |
| FAILED | 45 |
| SKIPPED | 4 |
| Total seen | 571 |

Those 571 seen rows are not the 249 regression rows in this shard. The observed
regression symptom for the assigned shard is therefore coverage loss/absence,
not known per-test output mismatch.

The candidate aggregate for the whole gate is still blocked at 7,197 public
comparable passes out of 20,294 (`public-comparable-score.tsv`), with aggregate
counts 7,197 passed, 8,851 failed, 2,222 skipped, 669 borked, 8 xfailed, and 2
warned (`counts.tsv`).

## Path-family breakdown

| Regression subpath under `ext/standard/tests/array` | Count |
| --- | ---: |
| root directory | 175 |
| `sort/` | 49 |
| `array_walk/` | 14 |
| `range/` | 5 |
| `in_array/` | 4 |
| `gh16649/` | 2 |

Top recurring filename families:

| Family | Count |
| --- | ---: |
| `bug*` regression tests | 27 |
| `array_chunk*` | 11 |
| `array_walk*` | 10 |
| `array_map*` | 7 |
| `array_key*` | 6 |
| `extract*` | 6 |
| `array_multisort*` | 6 |
| `array_unshift*` | 5 |
| `range*` | 5 |
| `asort*` | 5 |
| `sort*` | 5 |

## Shard-log symptoms

Every parallel shard recorded exit code 1:

| Shard | Result rows | Result summary | `run-tests.log` | Exit symptom |
| --- | ---: | --- | --- | --- |
| 01 | 3,630 | 1,408 PASS, 1,643 FAIL, 464 SKIP, 112 BORK, 2 XFAIL, 1 WARN | present | rc 1; stderr has `autoconf: command not found` and missing `php-src/libtool` |
| 02 | 3,630 | 1,357 PASS, 1,685 FAIL, 453 SKIP, 133 BORK, 2 XFAIL | present | rc 1; stderr has `autoconf: command not found` and missing `php-src/libtool` |
| 03 | 2,114 | 798 PASS, 1,043 FAIL, 192 SKIP, 80 BORK, 1 XFAIL | absent | rc 1; stdout stops at `ext/pdo_mysql/tests/common.phpt` with missing copied harness directory `run-tests-harnesses/shard-03/ext/pdo/tests` |
| 04 | 2,268 | 794 PASS, 1,191 FAIL, 191 SKIP, 91 BORK, 1 XFAIL | absent | rc 1; stdout stops at `ext/pdo_pgsql/tests/common.phpt` with missing copied harness directory `run-tests-harnesses/shard-04/ext/pdo/tests` |
| 05 | 3,630 | 1,397 PASS, 1,649 FAIL, 452 SKIP, 132 BORK | present | rc 1; stderr has `autoconf: command not found` and missing `php-src/libtool` |
| 06 | 3,630 | 1,407 PASS, 1,633 FAIL, 467 SKIP, 121 BORK, 1 XFAIL, 1 WARN | present | rc 1; stderr has `autoconf: command not found` and missing `php-src/libtool` |

Likely failure modes from the preserved artifacts:

- Primary observed symptom for this shard: all 249 accepted-pass standard-array
  rows are absent from both candidate status artifacts. There is no evidence
  here that these specific rows failed due to compiler/runtime behavior.
- Shards 03 and 04 aborted before reaching later `ext/standard` coverage. The
  durable stdout logs show PHP's `run-tests.php` trying to open
  `run-tests-harnesses/shard-03/ext/pdo/tests` and
  `run-tests-harnesses/shard-04/ext/pdo/tests`, then stopping with
  `ERROR: cannot open directory`. `run_gate.sh` copies `run-tests.php` into a
  per-shard harness, so redirected common-test directory lookups can break when
  the copied harness does not include the expected source-tree directories.
- The temporary run root and `shard-*.tests` input lists are no longer present,
  so exact intended shard ownership for each missing standard-array row cannot
  be confirmed from durable evidence. The count pattern is consistent with
  aborted shard coverage: 571 standard-array rows reached the aggregate, while
  249 standard-array accepted-pass rows did not.
- The four shards with full `run-tests.log` files still ended rc 1 and reported
  setup/environment noise, including `autoconf: command not found` and missing
  `php-src/libtool`.
- Many BORK rows outside this shard are invalid SKIPIF/setup failures around
  missing extension functions/constants/classes such as curl/gd/intl, `PDOTest`,
  `Phar`, and `ZipArchive`. They add aggregate noise but do not provide
  diagnostics for the 249 absent standard-array rows.

## Representative rows

| Row | Accepted/latest-published status | Candidate status | Diagnostic note |
| --- | --- | --- | --- |
| `php-src/ext/standard/tests/array/006.phpt` | PASS | ABSENT | Missing from both `current-status.normalized.tsv` and `all-results.txt`; no row-level shard diagnostic. |
| `php-src/ext/standard/tests/array/array_chunk_variation12.phpt` | PASS | ABSENT | Representative of the 11 `array_chunk*` missing rows; no candidate FAIL/BORK output exists. |
| `php-src/ext/standard/tests/array/array_walk/array_walk_recursive_basic2.phpt` | PASS | ABSENT | Representative of `array_walk/` coverage loss; no row-level PHPT diff is preserved. |
| `php-src/ext/standard/tests/array/range/range_step_errors.phpt` | PASS | ABSENT | Representative of `range/`; should be replayed before treating as runtime behavior. |
| `php-src/ext/standard/tests/array/sort/array_multisort_basic1.phpt` | PASS | ABSENT | Representative of the 49 `sort/` rows; absent status is consistent with aborted shard coverage. |

## Focused replay recommendations

Do not infer a compiler/runtime regression from this shard alone. The durable
evidence says the 249 assigned rows need replay because they are absent, not
because they produced mismatched output.

Recommended control-plane sequence:

1. Build a focused replay list directly from the durable regression artifact:

   ```sh
   rg '^php-src/ext/standard/tests/array/' /home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/regressions-from-latest-published-passes.txt
   ```

2. First replay the five representative rows above under the existing PHPT
   wrapper command shape recorded in `environment.txt`, after fixing or avoiding
   the copied-harness directory issue that stopped shards 03 and 04.

3. If the representatives execute, replay by subpath group in this order:
   `sort/` (49), root directory rows (175), `array_walk/` (14), `range/` (5),
   `in_array/` (4), and `gh16649/` (2). This keeps the next pass narrow and
   turns absent rows into real PASS/FAIL/BORK evidence.

4. Preserve the focused replay input list as an evidence artifact next time.
   The current durable evidence does not include `shard-*.tests`, which prevents
   exact per-row shard attribution after the temporary run root is removed.

5. Do not run another full PHPT gate just to classify these rows. A focused
   replay of the 249 paths is the deterministic next step; only after those
   paths have row-level outcomes should compiler/runtime work be assigned.

## Non-goals confirmed

- No eval or variable-variable work was performed.
- No compiler, runtime, test fixture, php-src, or project support document was
  edited.
- No full PHPT gate was run.
- This report is diagnostic/control-plane only and cannot improve the public
  PHPT score by itself.
