# 221205Z Reflection Regression Shard

Diagnostic/control-plane report for lane 19. This report does not edit compiler
or runtime code, does not run a full PHPT gate, and cannot move the public PHPT
score by itself.

## Evidence

- Candidate artifact: `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377`
- Accepted baseline artifact used by the candidate: `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67`
- Candidate public comparable score: `7197/20294`.
- Accepted public comparable score in the baseline artifact: `7873/20294`.
- Candidate `pass-regression-summary.tsv`: baseline normalized passes `7869`, current normalized passes `7196`, pass regressions `1166`.
- `DEVELOPMENT.md` was requested by the harness prompt but is absent under `/home/claude/php-to-native-compiler`.

## Exact Count

`regressions-from-latest-published-passes.txt` contains exactly 110 rows under
`php-src/ext/reflection/`.

All 110 rows are present as `PASSED` in the accepted baseline
`current-status.normalized.tsv` and `all-results.txt`. In the 221205Z candidate,
all 110 are absent from both `current-status.normalized.tsv` and
`all-results.txt`; none has a candidate row-level `FAILED`, `SKIPPED`, `BORKED`,
or `PASSED` status.

## API Clusters

Clusters inferred from row paths and file names:

| Cluster | Rows |
| --- | ---: |
| ReflectionClass / ReflectionObject | 32 |
| Legacy numbered/bug regression rows | 29 |
| ReflectionFunction / closures | 10 |
| ReflectionProperty | 10 |
| ReflectionMethod / traits | 9 |
| Reflection extension/core objects | 7 |
| ReflectionParameter / defaults | 6 |
| Type metadata / iterable / DNF | 5 |
| Attributes / ReflectionConstant | 2 |
| Enums | 0 |

## Status And Symptoms

- Regression-row status: accepted baseline `PASSED=110`; candidate
  `MISSING=110` in both `current-status.normalized.tsv` and `all-results.txt`.
- Direct candidate shard-log hits for the 110 row paths: `0`.
- Candidate did report other reflection tests: `345` total reflection rows in
  `current-status.normalized.tsv`, broken down as `PASSED=225`, `FAILED=116`,
  and `SKIPPED=4`. The reflection directory was therefore partially reached,
  but the 110 regression rows were not reported.
- Candidate shard-level symptom: `shard-03` produced only `2114` result rows and
  stopped at `ext/pdo_mysql/tests/common.phpt`; stdout shows
  `ERROR: cannot open directory: .../run-tests-harnesses/shard-03/ext/pdo/tests`.
- Candidate shard-level symptom: `shard-04` produced only `2268` result rows and
  stopped at `ext/pdo_pgsql/tests/common.phpt`; stdout shows the same missing
  `run-tests-harnesses/shard-04/ext/pdo/tests` directory pattern.
- `aggregate-warnings.tsv` says `missing_results=0`, because each shard had a
  `results.txt`; it does not detect partial shard output.
- `shard-03/run-tests.log` and `shard-04/run-tests.log` were not available in
  the artifact directory, so the stdout tail is the available diagnostic source
  for those two incomplete shards.

## Likely Buckets

- Primary bucket: harness/control-plane omission. The 110 rows are pass
  regressions because they disappeared from candidate normalized results, not
  because the candidate produced row-level reflection failures.
- Secondary bucket: redirected PDO common-test harness setup. The available
  shard stdout points to missing redirected `ext/pdo/tests` directories under
  per-shard `run-tests-harnesses`, causing partial shard result files before
  later rows could be reported.
- Unknown semantic bucket: unproven. Some omitted rows cover real reflection
  semantics such as class/object metadata, closure reflection, method
  invocation errors, property metadata, attributes, parameter defaults, and DNF
  type metadata, but this candidate artifact does not execute those 110 rows.
- Unsupported/unknown cases remain named as unknown; this report does not claim
  reflection semantic support or regressions from these rows.
- Eval and variable-variable behavior are not part of this shard and were not
  investigated.

## Representative Rows

| Row | Accepted status | Candidate status | Diagnostic note |
| --- | --- | --- | --- |
| `php-src/ext/reflection/tests/001.phpt` | `PASSED` | `MISSING` | Accepted title: `Reflection inheritance`; no candidate status/log row. |
| `php-src/ext/reflection/tests/ReflectionClass_getConstants_basic.phpt` | `PASSED` | `MISSING` | Accepted title: `ReflectionClass::getConstants()`; class constant metadata row omitted in candidate. |
| `php-src/ext/reflection/tests/ReflectionFunction_getClosureUsedVariables.phpt` | `PASSED` | `MISSING` | Accepted title: `ReflectionFunctionAbstract::getClosureUsedVariables`; closure metadata row omitted in candidate. |
| `php-src/ext/reflection/tests/ReflectionMethod_invokeArgs_error2.phpt` | `PASSED` | `MISSING` | Accepted title: `ReflectionMethod::invokeArgs() further errors`; method invocation error row omitted in candidate. |
| `php-src/ext/reflection/tests/types/dnf_types.phpt` | `PASSED` | `MISSING` | Accepted title: `Disjunctive Normal Form types in reflection`; type metadata row omitted in candidate. |

## Focused Replay Recommendations

- Do not run `run_gate.sh` unmodified for this lane; it is a full gate.
- Use `environment.txt` and `run_gate.sh` only to copy the recorded environment:
  `SOURCE_HEAD=56fe9377fb46be00db5fdd30c966fdba406dc581`,
  `PHP_SRC_PIN=f97ff597429a2fe633665a7e02d97c8077f9f90f`,
  wrapper `/home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper`,
  `PHPC_PHPT_TIMEOUT_SECONDS=55`, `PHPC_PHPT_KILL_AFTER_SECONDS=5`, and the same
  lowercase `run-tests.php` shape with `-q -n -p <wrapper> -r <small-list>
  -W <results> -s <log> --no-color --set-timeout 65 --temp-source <php-src>
  --temp-target <temp>`.
- First replay a tiny reflection-only list containing the five representative
  rows above. This separates real reflection semantics from the candidate
  omission symptom.
- Separately replay the shard abort reproducers
  `php-src/ext/pdo_mysql/tests/common.phpt` and
  `php-src/ext/pdo_pgsql/tests/common.phpt` using the same per-shard harness
  preparation, because the available candidate stdout points to redirected
  `ext/pdo/tests` directory setup as the broad omission trigger.
- For future gates, preserve generated `shard-*.tests` files in evidence and
  add an aggregate warning for short shard result counts or last-test aborts;
  `missing_results=0` is insufficient when a partial `results.txt` exists.
