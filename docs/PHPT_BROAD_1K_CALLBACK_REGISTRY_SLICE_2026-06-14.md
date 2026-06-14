# PHPT Broad 1k Callback Registry Slice: 2026-06-14

Issue: `ptn-d6n9`

This slice used the broad 1k PHPT baseline tooling on the rebased
`origin/master` lineage and narrowed the broad `ext/standard/tests/array`
callback rows. The generic cause was sorted internal function registry order:
`ptn_find_internal_function()` must be able to find `array_filter()` through
its binary-search lookup. The final branch adds regression coverage for that
lookup edge and records the remaining callback blocker map.

## Broad 1k Classifier Baseline

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only
```

Artifacts:

- generated manifest:
  `.runtime/phpt-baseline/20260614T010012Z/phpt-baseline-1000.txt`
- classification:
  `.runtime/phpt-progress/classification-20260614T010012Z.tsv`
- runnable manifest:
  `.runtime/phpt-progress/runnable-20260614T010012Z.txt`

Corpus revision: `8c63ec400ce8e07c57a8d9499317b96a8beafb8b`

Classifier result:

| Bucket | Rows |
| --- | ---: |
| Selected | 1,000 |
| Runnable | 447 |
| Excluded | 553 |
| `unsupported-language` | 351 |
| `unsupported-class-metadata` | 84 |
| `unsupported-request-input-ini` | 28 |
| `unsupported-extension` | 20 |
| `unsupported-assertion-ini` | 17 |
| `unsupported-resource-limit-ini` | 15 |
| `sapi-behavior` | 13 |
| Other classifier buckets | 25 |

The runnable rows are still dominated by `ext/standard/tests/array`: 276 of
447 runnable broad rows.

## Focused Callback Evidence

Focused manifest command, using the fresh broad 1k runnable manifest:

```sh
rg '^ext/standard/tests/array/array_(map|filter|reduce|all|any|find|find_key)' \
  .runtime/phpt-progress/runnable-20260614T010012Z.txt |
  tools/run-phpt-manifest.sh -
```

Before the registry-order fix:

| Selected | Runnable | Passed | Failed | Log |
| ---: | ---: | ---: | ---: | --- |
| 38 | 38 | 17 | 21 | `.runtime/phpt-progress/run-20260614T011826Z.log` |

After the registry-order fix on the earlier base:

| Selected | Runnable | Passed | Failed | Log |
| ---: | ---: | ---: | ---: | --- |
| 38 | 38 | 24 | 14 | `.runtime/phpt-progress/run-20260614T014042Z.log` |

After the final rebase and verification run:

| Selected | Runnable | Passed | Failed | Log |
| ---: | ---: | ---: | ---: | --- |
| 38 | 38 | 28 | 10 | `.runtime/phpt-progress/run-20260614T040614Z.log` |

Registry-order movement: 7 broad callback rows newly passed on the earlier
base. The final rebased branch is 11 passes ahead of the before-fix run.

Rows unlocked by the registry-order fix:

- `ext/standard/tests/array/array_filter.phpt`
- `ext/standard/tests/array/array_filter_basic.phpt`
- `ext/standard/tests/array/array_filter_variation3.phpt`
- `ext/standard/tests/array/array_filter_variation4.phpt`
- `ext/standard/tests/array/array_filter_variation6.phpt`
- `ext/standard/tests/array/array_filter_variation7.phpt`
- `ext/standard/tests/array/array_filter_variation8.phpt`

Additional rows passing on the final rebase:

- `ext/standard/tests/array/array_filter_variation9.phpt`
- `ext/standard/tests/array/array_map_variation14.phpt`
- `ext/standard/tests/array/array_map_variation15.phpt`
- `ext/standard/tests/array/array_map_variation16.phpt`

## Generic Cause

`ptn_find_internal_function()` binary-searches the C internal function table,
which is documented as sorted by ASCII case-insensitive name. `array_filter`
was placed after `array_find_key`, so lookups for `array_filter` missed the
runtime helper even though `ptn_internal_array_filter()` was implemented.

Observable effects:

- `function_exists("array_filter")` returned false.
- `array_filter()` calls reached `ptn_call_internal()` and reported an
  undefined function through the CLI/PHPT path.
- Callback rows using ordinary `array_filter()` behavior failed before
  exercising the modeled helper.

The rebased base has `array_filter` before `array_find`; this slice extends the
internal registry lookup regression to cover the sorted-table edge.

## Remaining Callback Blockers

The remaining 10 final-base focused callback failures are not one
implementation surface:

| Rows | Blocker | Representative rows |
| ---: | --- | --- |
| 1 | Named-argument mode exception path in native CLI | `array_filter_invalid_mode.phpt` |
| 1 | `array_filter()` key/both mode callback arity diagnostics | `array_filter_variation10.phpt` |
| 4 | `array_map()` callback validation and catchable arity diagnostics | `array_map_error.phpt`, `array_map_object2.phpt`, `array_map_variation10.phpt`, `array_map_variation12.phpt` |
| 2 | `array_map()` reference and uneven-array/null-fill semantics | `array_map_variation2.phpt`, `array_map_variation7.phpt` |
| 1 | Binary-safe string callback behavior | `array_map_variation9.phpt` |
| 1 | `array_reduce()` callback arity diagnostics | `array_reduce_variation1.phpt` |

This is why the slice lands the generic registry fix plus this blocker map
rather than claiming the callback cluster is complete.

## Verification

```sh
cargo test compile_internal_function_registry_lookup_edges_to_native_binary
cargo test compile_array_filter_to_native_binary
cargo build --bin phpc
target/debug/phpc -r 'var_dump(function_exists("array_filter")); var_dump(array_filter([1,2]));'
tools/run-phpt-baseline.sh --tier 1000 --classify-only
rg '^ext/standard/tests/array/array_(map|filter|reduce|all|any|find|find_key)' \
  .runtime/phpt-progress/runnable-20260614T010012Z.txt |
  tools/run-phpt-manifest.sh -
```
