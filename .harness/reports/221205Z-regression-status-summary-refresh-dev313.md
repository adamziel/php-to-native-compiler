# 221205Z Regression Status Summary Refresh

Lane: 130, developer-313

Generated: 2026-06-05T08:57Z

Scope: read-only artifact refresh. No compiler/runtime source edits were made.
No full PHPT gate was run. This report does not move the public score.

## Result

The blocked 221205Z gate still has exactly `1166` latest-public PASS
regressions.

Coarse status bucket recomputation from the authoritative candidate artifacts:

| Bucket | Rows | Meaning |
| --- | ---: | --- |
| `ABSENT` | 1136 | Present in accepted normalized PASS baseline, absent from candidate normalized PASS set, and absent from candidate `current-status.normalized.tsv`/`all-results.txt`. |
| `FAILED` | 27 | Regression row has direct candidate `FAILED` status in both `current-status.normalized.tsv` and `all-results.txt`. |
| `BORKED` | 3 | Regression row has direct candidate `BORKED` status in both `current-status.normalized.tsv` and `all-results.txt`. |
| Total | 1166 | Matches `regressions-from-latest-published-passes.txt` and `pass-regression-summary.tsv`. |

No regression row is still present in `current-passes.normalized.txt`.
There are no regression rows present only in `all-results.txt` while absent
from `current-status.normalized.tsv`, or vice versa.

The accepted public score remains `7873 / 20294 = 38.79%` at
`0b917f67a37d9ca9779d77f87173b628431c2425`. The blocked 221205Z candidate
remains `7197 / 20294 = 35.46%` at
`56fe9377fb46be00db5fdd30c966fdba406dc581`; this is not publishable because
the PASS-regression count is nonzero.

## Evidence Paths

Candidate gate directory:

`/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377`

Accepted baseline gate directory:

`/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67`

Primary candidate files used:

- `regressions-from-latest-published-passes.txt`
- `current-status.normalized.tsv`
- `all-results.txt`
- `current-passes.normalized.txt`
- `pass-regression-summary.tsv`
- `counts.tsv`
- `public-comparable-score.tsv`
- `current-score-gate-preflight.tsv`
- `shard-*/stdout.log`
- `aggregate-warnings.tsv`
- `run_gate.sh`

Primary accepted files used:

- `public-comparable-score.tsv`
- `current-score-gate-preflight.tsv`

SQLite confirmation:

- `/home/claude/php-to-native-compiler/.harness/harness.sqlite3`
- `goals.id=1`
- `metric_samples.id in (1,2,3,4,5,6)`
- `work_lanes.id=130`

## Recomputed Counts

Low-CPU artifact reads only:

```sh
wc -l \
  "$CAND/regressions-from-latest-published-passes.txt" \
  "$CAND/current-status.normalized.tsv" \
  "$CAND/all-results.txt" \
  "$CAND/counts.tsv" \
  "$CAND/public-comparable-score.tsv"

sed -n '1,80p' "$CAND/pass-regression-summary.tsv"
sed -n '1,80p' "$CAND/counts.tsv"
sed -n '1,80p' "$CAND/public-comparable-score.tsv"
rg -n 'ERROR: cannot open directory|Report saved to:' "$CAND"/shard-*/stdout.log
```

Observed line/count artifacts:

| Artifact | Value |
| --- | ---: |
| Candidate regression rows | 1166 |
| Candidate `current-status.normalized.tsv` rows | 18940 |
| Candidate `all-results.txt` rows | 18949 |
| Candidate raw passes from `counts.tsv` | 7197 |
| Candidate public score from `public-comparable-score.tsv` | `7197 / 20294 = 35.46%` |
| Accepted public score from `public-comparable-score.tsv` | `7873 / 20294 = 38.79%` |

The independent path/status join normalized absolute `/tmp/.../php-src/...`
paths to `php-src/...` keys and produced:

```text
regression_rows 1166 unique 1166
direct_current_rows 30 Counter({'FAILED': 27, 'BORKED': 3})
direct_all_results_rows 30 Counter({'FAILED': 27, 'BORKED': 3})
absent_from_current_status 1136
absent_from_all_results 1136
present_current_not_all [] 0
present_all_not_current [] 0
regressions_still_current_passes 0
coarse_absent_failed_borked {'ABSENT': 1136, 'FAILED': 27, 'BORKED': 3}
```

## Direct Non-PASS Rows

The 27 direct `FAILED` rows are:

- `php-src/ext/bcmath/tests/number/properties_unset.phpt`
- `php-src/ext/bcmath/tests/number/properties_write_error.phpt`
- `php-src/ext/date/tests/DatePeriod_modify_readonly_property.phpt`
- `php-src/ext/date/tests/DatePeriod_properties2.phpt`
- `php-src/ext/opcache/tests/opt/sccp_037.phpt`
- `php-src/ext/standard/tests/directory/DirectoryClass_readonly_handle.phpt`
- `php-src/ext/standard/tests/directory/DirectoryClass_readonly_path.phpt`
- `php-src/ext/xmlreader/tests/014.phpt`
- `php-src/tests/classes/ctor_dtor.phpt`
- `php-src/tests/classes/destructor_and_echo.phpt`
- `php-src/tests/classes/factory_and_singleton_002.phpt`
- `php-src/tests/classes/iterators_002.phpt`
- `php-src/Zend/tests/assert/expect_008.phpt`
- `php-src/Zend/tests/assert/expect_011.phpt`
- `php-src/Zend/tests/attributes/deprecated/property_readonly_001.phpt`
- `php-src/Zend/tests/attributes/deprecated/property_readonly_002.phpt`
- `php-src/Zend/tests/attributes/override/properties_08.phpt`
- `php-src/Zend/tests/bug73989.phpt`
- `php-src/Zend/tests/gc/bug63635.phpt`
- `php-src/Zend/tests/property_hooks/gh19548_002.phpt`
- `php-src/Zend/tests/property_hooks/gh19548.phpt`
- `php-src/Zend/tests/readonly_classes/readonly_class_property1.phpt`
- `php-src/Zend/tests/readonly_classes/readonly_class_property2.phpt`
- `php-src/Zend/tests/readonly_props/readonly_trait_mismatch.phpt`
- `php-src/Zend/tests/serialize/bug76502.phpt`
- `php-src/Zend/tests/type_declarations/variance/internal_parent/unresolvable_inheritance_check_param.phpt`
- `php-src/Zend/tests/uncaught_exception_error_supression.phpt`

The 3 direct `BORKED` rows are:

- `php-src/ext/intl/tests/rangeformatter/rangeformatter_icu63_compatibility.phpt`
- `php-src/ext/openssl/tests/openssl_libctx_without_zts_argon.phpt`
- `php-src/ext/pcre/tests/grep2.phpt`

## Absent-Row Refinement

The coarse `ABSENT=1136` bucket should not be interpreted as 1136 semantic
failures. Reconstructing the 6-shard assignment from `run_gate.sh` and the
tracked php-src pin `f97ff597429a2fe633665a7e02d97c8077f9f90f` gives:

| Reconstructed bucket | Rows |
| --- | ---: |
| Absent rows assigned to aborted shards `03`/`04` | 506 |
| Absent rows assigned to other shards | 630 |
| Total coarse `ABSENT` rows | 1136 |

The per-shard absent counts are:

| Shard | Coarse absent rows |
| --- | ---: |
| `01` | 74 |
| `02` | 71 |
| `03` | 199 |
| `04` | 307 |
| `05` | 297 |
| `06` | 188 |

Shard stdout evidence:

- `shard-03/stdout.log` has
  `ERROR: cannot open directory: .../run-tests-harnesses/shard-03/ext/pdo/tests`
  and does not have `Report saved to:`.
- `shard-04/stdout.log` has
  `ERROR: cannot open directory: .../run-tests-harnesses/shard-04/ext/pdo/tests`
  and does not have `Report saved to:`.
- Shards `01`, `02`, `05`, and `06` reached `Report saved to:`.
- `aggregate-warnings.tsv` says `missing_results	0`, which only proves
  result files existed; it does not prove every expected PHPT path received a
  per-test status row.

## Status Decision

Do not move the public score. The candidate remains
`FINAL / BLOCKED-PASS-REGRESSIONS` until the `1166` rows are repaired,
replayed cleanly, or auditor-adjudicated under the policy in `goals.id=1`.

Near-term deterministic follow-up remains split:

- handle the shard directory/control-plane abort before treating all absent
  rows as product regressions;
- replay representative direct `FAILED`/`BORKED` rows for repair or
  environment-adjudication lanes;
- keep eval and variable-variable rows late-priority, per the goal policy.
