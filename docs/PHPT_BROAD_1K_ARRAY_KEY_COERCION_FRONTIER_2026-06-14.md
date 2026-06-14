# PHPT Broad 1k Array Key/Coercion Frontier: 2026-06-14

Issue: `ptn-i1p9`

This slice uses the broad PHPT baseline tooling on the 1k tier and maps the
standard-array key/coercion lookup frontier. It is a blocker map rather than an
implementation patch: the remaining failures span dynamic variable reads,
binary-safe string keys, resource helpers, loose comparison, key coercion
diagnostics, and warning formatting. No single credible semantic change explains
25 broad rows without crossing those runtime boundaries.

## Broad 1k Baseline

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only --out-dir .runtime/ptn-i1p9-baseline
```

Generated manifest:
`.runtime/ptn-i1p9-baseline/20260614T040449Z/phpt-baseline-1000.txt`

Corpus revision: `8c63ec400ce8e07c57a8d9499317b96a8beafb8b`

Classifier artifact:
`.runtime/phpt-progress/summary-20260614T040449Z.txt`

The broad run selected 1,000 rows, kept 429 runnable, and excluded 571:

| Classification | Rows |
| --- | ---: |
| `runnable` | 429 |
| `unsupported-language` | 281 |
| `unsupported-class-metadata` | 144 |
| `unsupported-request-input-ini` | 28 |
| `unsupported-extension` | 20 |
| `unsupported-diagnostics-runtime` | 18 |
| `unsupported-assertion-ini` | 17 |
| `unsupported-resource-limit-ini` | 15 |
| `sapi-behavior` | 13 |
| `unsupported-assertion-runtime` | 9 |
| `unsupported-diagnostics-ini` | 5 |
| `harness-cleanup` | 4 |
| `process-boundary` | 3 |
| `unsupported-function-disable-ini` | 2 |
| `unsupported-host-path-ini` | 2 |
| `unsupported-opcache-ini` | 2 |
| `unsupported-scalar-format-ini` | 2 |
| `skipif-precondition` | 2 |
| `environment-assumption` | 1 |
| `external-service` | 1 |
| `unsupported-internal` | 1 |
| `unsupported-resource-limit` | 1 |

Runnable rows by broad bucket:

| Bucket | Runnable |
| --- | ---: |
| `ext/standard/tests` | 296 |
| `Zend/tests` | 117 |
| `tests/basic` | 16 |

## Focused Manifest

Focused manifest:
`.runtime/ptn-i1p9-array-key-coercion-manifest.txt`

Generated from the broad classification with:

```sh
awk '$1 ~ /^ext\/standard\/tests\/array\/array_(change_key_case|combine|count_values|flip|key_exists|key_first|key_last|keys|search)/ {print $1}' \
  .runtime/phpt-progress/classification-20260614T040449Z.tsv \
  > .runtime/ptn-i1p9-array-key-coercion-manifest.txt
```

Focused run:

```sh
timeout 900s tools/run-bounded-phpt.sh .runtime/ptn-i1p9-array-key-coercion-manifest.txt
```

Focused artifacts:

- `.runtime/phpt-progress/classification-20260614T041127Z.tsv`
- `.runtime/phpt-progress/excluded-20260614T041127Z.tsv`
- `.runtime/phpt-progress/run-20260614T041127Z-manifest.log`
- `.runtime/phpt-progress/summary-20260614T041127Z.txt`

Focused result:

| Selected | Runnable | Excluded | Passed | Failed |
| ---: | ---: | ---: | ---: | ---: |
| 49 | 44 | 5 | 34 | 10 |

## Family Counts

| Family | Selected | Runnable | Passed | Failed | Excluded |
| --- | ---: | ---: | ---: | ---: | ---: |
| `array_change_key_case()` | 7 | 7 | 6 | 1 | 0 |
| `array_combine()` | 7 | 5 | 3 | 2 | 2 |
| `array_count_values()` | 3 | 2 | 1 | 1 | 1 |
| `array_flip()` | 6 | 5 | 3 | 2 | 1 |
| `array_key_exists()` | 9 | 8 | 7 | 1 | 1 |
| `array_key_first()` | 2 | 2 | 2 | 0 | 0 |
| `array_key_last()` | 2 | 2 | 2 | 0 | 0 |
| `array_keys()` | 7 | 7 | 6 | 1 | 0 |
| `array_search()` / `array_search1` | 6 | 6 | 4 | 2 | 0 |

Excluded rows:

| Category | Rows | Files |
| --- | ---: | --- |
| `unsupported-class-metadata` | 4 | `array_combine_variation4.phpt`, `array_combine_variation5.phpt`, `array_flip_variation4.phpt`, `array_key_exists_variation1.phpt` |
| `harness-cleanup` | 1 | `array_count_values_variation.phpt` |

## Remaining Runnable Failures

| Blocker | Rows | Evidence |
| --- | ---: | --- |
| Binary-safe string key storage/display and double-quoted/heredoc escape byte parity | 3 | `array_change_key_case_variation8.phpt`, `array_combine_variation3.phpt`, `array_flip_variation3.phpt` |
| Dynamic variable-variable reads for local symbols | 1 | `array_combine.phpt` uses `$$letter` to read `$a` through `$p`; PTN currently emits an undefined-variable path before dumping the combined arrays. |
| Warning formatting cadence for invalid flip/count values | 2 | `array_count_values2.phpt`, `array_flip.phpt` differ on blank-line placement and warning source formatting around skipped invalid entries. |
| Float-to-int key coercion deprecation diagnostics | 1 | `array_key_exists_variation3.phpt` expects precision-loss deprecations before successful lookups. |
| Directory resources in array helpers | 2 | `array_keys_variation_005.phpt`, `array_search_variation4.phpt` fail at missing `opendir()`/`closedir()` before resource comparison can run. |
| Object/resource loose comparison and type-name diagnostics | 1 | `array_search_variation3.phpt` reaches the helper but reports generic `object given` instead of the concrete class name, with remaining loose haystack comparison parity. |

## Next Implementation Split

1. Land binary-safe array-key storage and dumping for embedded NUL/control
   bytes, sharing the same path used by `array_flip()`, `array_combine()`, and
   key case conversion.
2. Finish double-quoted and heredoc escape byte parity for control escapes that
   currently remain literal in string-key rows.
3. Revisit scalar variable-variable reads in local scopes; this explains the
   broad `array_combine.phpt` failure and should be handled through the dynamic
   root model, not an array helper special case.
4. Add directory resource helpers (`opendir()`, `closedir()`) or classify them
   until directory resources join the modeled resource surface.
5. Centralize array-helper warning emission so invalid count/flip values and
   float key coercions use PHP-compatible deprecation/warning cadence.

## Verification

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only --out-dir .runtime/ptn-i1p9-baseline
timeout 900s tools/run-bounded-phpt.sh .runtime/ptn-i1p9-array-key-coercion-manifest.txt
```

This branch records a blocker map only; it intentionally does not claim newly
passing rows or newly classified unsupported rows.
