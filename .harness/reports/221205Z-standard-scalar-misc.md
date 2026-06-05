# 221205Z Standard Scalar/Misc Regression Shard

Lane: 17, developer-83

Scope: read-only M0 shard for `php-src/ext/standard/tests/{math,general_functions,serialize,url,class_object,assert,crypt,time,versioning,misc}` regressions from the blocked `221205Z` public PHPT gate. No compiler/runtime source edits were made.

## Evidence

- Candidate gate:
  `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377`
- Regression list:
  `regressions-from-latest-published-passes.txt`
- Candidate result/status artifacts:
  `all-results.txt`, `current-status.normalized.tsv`, `shard-*/results.txt`,
  `shard-*/run-tests.log`, `aggregate-warnings.tsv`, `shard-exit-codes.tsv`
- PHPT source checkout for title/section inspection:
  `/home/claude/php-src-phpt`

The overall blocked gate remains `7197 / 20294` with `1166` latest-public PASS regressions. This shard accounts for `142` of those regression rows.

## Counts

| Subdirectory | Regression rows |
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

Status coverage for these `142` rows:

| Candidate artifact view | Count |
| --- | ---: |
| Absent from `current-status.normalized.tsv` | 142 |
| Absent from normalized `all-results.txt` paths | 142 |
| Present as `FAILED`, `BORKED`, `SKIPPED`, or `PASSED` | 0 |

This is the dominant symptom for the shard: the rows are in the accepted baseline and in `regressions-from-latest-published-passes.txt`, but the candidate status/result artifacts do not contain direct per-row outcomes for them. Across all `1166` regressions, `1136` are absent from the candidate result/status artifacts, so this shard appears to be part of the broader missing-result regression shape rather than a directly observed per-test semantic failure in the candidate artifacts.

`aggregate-warnings.tsv` says `missing_results	0`, and all six shard exit rows are `1`, so the artifact set itself does not flag these absent rows as missing expected results. Treat this as an evidence-integrity/replay target: the rows should be replayed directly before assigning semantic root causes.

## Feature Clusters

Top filename-derived clusters inside this shard:

| Cluster | Rows | Notes |
| --- | ---: | --- |
| `serialize` bug/edge cases | 10 | Serialization edge cases, object/incomplete class handling, reference/global cases, malformed payloads. |
| `general_functions: var*` | 8 | `var_dump()`, `var_export()` output and formatting cases. |
| `general_functions: is*` | 7 | Type/callability/resource predicates. |
| `assert` | 6 | Assertion callbacks, return values, and closure assertions. |
| `math: round*` | 6 | Rounding-mode and large-exponent edge cases. |
| `general_functions: print_r*` | 4 | Array/scalar `print_r()` output cases. |
| `url: parse_url*` | 4 | URL component parsing cases. |
| `math: number_format*` | 3 | Numeric formatting/coercion cases. |
| `serialize: unserialize*` | 3 | `unserialize()` option/malformed-data cases. |

Smaller math clusters include trigonometric/hyperbolic functions, base conversion, integer-base string conversion, constants, float predicates, exponent/log functions, and power/division edge cases. Smaller class/object clusters cover `get_class_methods()`, `get_object_vars()`, `get_declared_*()`, `get_parent_class()`, `interface_exists()`, `is_subclass_of()`, and `method_exists()`.

## Representative Replay Rows

Use these as a focused accepted-vs-candidate replay sample. They cover the high-count clusters and the one-off directories without pulling in the full gate:

| Row | Why this row |
| --- | --- |
| `php-src/ext/standard/tests/math/round_RoundingMode.phpt` | RoundingMode enum and one of the six `round*` regressions. |
| `php-src/ext/standard/tests/math/acos_basic.phpt` | Simple math builtin baseline for return type/value behavior. |
| `php-src/ext/standard/tests/general_functions/var_dump_arrays.phpt` | Representative `var_dump()` output formatting cluster. |
| `php-src/ext/standard/tests/general_functions/is_callable_variation1.phpt` | Representative predicate/callability cluster. |
| `php-src/ext/standard/tests/general_functions/ob_get_flush_basic.phpt` | Output-buffering general-functions edge case. |
| `php-src/ext/standard/tests/serialize/unserialize_allowed_classes_option_invalid_array.phpt` | Representative `unserialize()` options cluster. |
| `php-src/ext/standard/tests/serialize/serialize_globals_var_refs.phpt` | Serialization/reference interaction row. |
| `php-src/ext/standard/tests/url/parse_url_basic_004.phpt` | Representative URL parsing cluster. |
| `php-src/ext/standard/tests/class_object/get_class_methods_variation_001.phpt` | Representative class/object metadata row. |
| `php-src/ext/standard/tests/assert/assert_basic2.phpt` | Representative assertion callback row. |
| `php-src/ext/standard/tests/crypt/bcrypt_invalid_algorithm.phpt` | One-off `crypt` row. |
| `php-src/ext/standard/tests/time/001.phpt` | One-off `time` row (`microtime()` test). |
| `php-src/ext/standard/tests/versioning/phpversion.phpt` | One-off versioning row. |

Recommended first replay set for low cost: `round_RoundingMode.phpt`, `var_dump_arrays.phpt`, `unserialize_allowed_classes_option_invalid_array.phpt`, `parse_url_basic_004.phpt`, and `assert_basic2.phpt`. If those reproduce as absent-result artifacts rather than normal failures, replay should expand by status/mechanism rather than by standard-library feature.

## Commands Run

```sh
python3 - <<'PY'
from pathlib import Path
from collections import Counter
root = Path('/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377')
regs = (root / 'regressions-from-latest-published-passes.txt').read_text().splitlines()
target = {'math','general_functions','serialize','url','class_object','assert','crypt','time','versioning','misc'}
rows = [r for r in regs if r.startswith('php-src/ext/standard/tests/') and len(r.split('/')) >= 5 and r.split('/')[4] in target]
print(len(rows))
print(Counter(r.split('/')[4] for r in rows))
PY
```

```sh
rg -n 'assert_basic2|floatval\.phpt|acos_basic\.phpt|parse_url_basic_004|phpversion\.phpt|bcrypt_invalid_algorithm|serialize/002' \
  /home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377 \
  -g '*.txt' -g '*.log' -g '*.tsv'
```

```sh
tail -n 25 \
  /home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/shard-*/results.txt
```

```sh
cat /home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/shard-exit-codes.tsv
cat /home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/aggregate-warnings.tsv
```

## Next Action

Do not start a scalar/misc implementation lane from this report alone. First replay a small representative sample against the accepted and candidate binaries. The important question is whether these rows truly fail semantically, or whether they disappeared from candidate result normalization/artifact coverage while still being counted as latest-public PASS regressions.
