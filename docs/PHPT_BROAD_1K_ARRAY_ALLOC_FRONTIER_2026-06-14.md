# PHPT Broad 1k Array Allocation Frontier: 2026-06-14

Issue: `ptn-yvgh`

This slice refreshed the broad 1k baseline, fixed a generic array allocation
guard for `array_fill()`, and refreshed the current array set/callback frontier
counts. The implementation deliberately avoids row-specific output shaping:
`array_fill()` now validates impossible counts before materializing entries and
emits the same PHP allocation-failure class of diagnostic used by the upstream
row.

## Broad 1k Baseline

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only
```

Generated manifest:
`.runtime/ptn-yvgh-rebased-baseline/20260614T022409Z/phpt-baseline-1000.txt`

Corpus revision: `8c63ec400ce8e07c57a8d9499317b96a8beafb8b`

Classifier result:

| Bucket | Rows |
| --- | ---: |
| Selected | 1,000 |
| Runnable | 409 |
| Excluded | 591 |
| `unsupported-language` | 351 |
| `unsupported-class-metadata` | 95 |
| `unsupported-request-input-ini` | 28 |
| `unsupported-extension` | 20 |
| `unsupported-diagnostics-runtime` | 18 |
| `unsupported-assertion-ini` | 17 |
| `unsupported-resource-limit-ini` | 15 |
| `sapi-behavior` | 13 |
| `unsupported-assertion-runtime` | 9 |
| Other classifier buckets | 25 |

Artifacts:

- `.runtime/phpt-progress/classification-20260614T022409Z.tsv`
- `.runtime/phpt-progress/runnable-20260614T022409Z.txt`
- `.runtime/phpt-progress/manifest-20260614T022409Z.txt`

## Implementation

`array_fill()` now has a bounded allocation preflight:

- negative counts still throw `ValueError`.
- counts greater than `INT32_MAX` throw
  `array_fill(): Argument #2 ($count) is too large`.
- counts above PTN's bounded array allocation limit emit a fatal
  `Possible integer overflow in memory allocation` diagnostic before any entry
  allocation. This keeps huge-array PHPT rows from leaving long-lived native
  processes while preserving the PHP observable failure mode.

Runtime changes:

- `src/backend/runtime/core_values.c`: shared `PTN_ARRAY_MAX_ALLOC_ENTRIES`.
- `src/backend/runtime/diagnostics.c`: generic memory-allocation fatal helper.
- `src/backend/runtime/internals_internal_functions.c`: `array_fill()`
  preflight.

Native coverage:

```sh
cargo test --test compile_native compile_array_fill -- --nocapture
```

Result: 3 passed.

## Focused PHPT Evidence

### Already-Covered `array_chunk()`

Focused manifest:
`.runtime/ptn-yvgh-array-chunk-manifest.txt`

Command:

```sh
tools/run-bounded-phpt.sh .runtime/ptn-yvgh-array-chunk-manifest.txt
```

Result:

| Focused set | Selected | Runnable | Passed | Failed |
| --- | ---: | ---: | ---: | ---: |
| `array_chunk*` | 32 | 32 | 32 | 0 |

Log: `.runtime/phpt-progress/run-20260614T015316Z-manifest.log`

### Allocation Guard

The rebased focused fill/pad manifest includes
`ext/standard/tests/array/array_fill_error2.phpt`, which current `ptn-7xxw`
classification excludes as `unsupported-resource-limit`:
`.runtime/ptn-yvgh-rebased-array-fill-pad-manifest.txt`

Command:

```sh
tools/run-bounded-phpt.sh .runtime/ptn-yvgh-rebased-array-fill-pad-manifest.txt
```

Result:

| Focused set | Selected | Runnable | Excluded | Passed | Failed |
| --- | ---: | ---: | ---: | ---: | ---: |
| `array_fill_error2.phpt` | 1 | 0 | 1 | 0 | 0 |

Native coverage above exercises the runtime allocation guard directly. The PHPT
harness no longer executes this row after the resource-limit classifier update.

Log: `.runtime/phpt-progress/run-20260614T030726Z-manifest.log`

### Fill/Pad Family

Focused manifest:
`.runtime/ptn-yvgh-rebased-array-fill-pad-manifest.txt`

Command:

```sh
tools/run-bounded-phpt.sh .runtime/ptn-yvgh-rebased-array-fill-pad-manifest.txt
```

Result:

| Focused set | Selected | Runnable | Excluded | Passed | Failed |
| --- | ---: | ---: | ---: | ---: | ---: |
| `array_(fill|pad)*` | 12 | 11 | 1 | 11 | 0 |

Log: `.runtime/phpt-progress/run-20260614T030726Z-manifest.log`

## Current Set/Callback Frontier

Focused manifest:
`.runtime/ptn-yvgh-rebased-array-set-callback-frontier.txt`

Command:

```sh
tools/run-bounded-phpt.sh .runtime/ptn-yvgh-rebased-array-set-callback-frontier.txt
```

Result:

| Focused set | Selected | Runnable | Passed | Failed |
| --- | ---: | ---: | ---: | ---: |
| `array_(diff|intersect|udiff|uintersect|map|filter|reduce|all|any|find)*` | 106 | 106 | 86 | 20 |

Log: `.runtime/phpt-progress/run-20260614T023151Z-manifest.log`

The 20 remaining failures are not one credible 25-row implementation slice:

| Rows | Blocker | Representative rows |
| ---: | --- | --- |
| 3 | Nested-array stringification in set operations | `array_diff_variation9.phpt`, `array_intersect_assoc_variation9.phpt`, `array_intersect_variation9.phpt` |
| 2 | `array_filter()` mode constants and key/value callback argument shape | `array_filter_invalid_mode.phpt`, `array_filter_variation10.phpt` |
| 7 | `array_map()` diagnostics, object-callable diagnostics, references, zipping, binary strings, and callback result semantics | `array_map_error.phpt`, `array_map_object2.phpt`, `array_map_variation10.phpt`, `array_map_variation12.phpt`, `array_map_variation2.phpt`, `array_map_variation7.phpt`, `array_map_variation9.phpt` |
| 1 | `array_reduce()` accumulator/initial-value variation | `array_reduce_variation1.phpt` |
| 7 | User-comparator set-operation ordering, duplicate matching, and comparator-result semantics | `array_udiff_assoc_variation.phpt`, `array_udiff_assoc_variation5.phpt`, `array_udiff_uassoc_variation6.phpt`, `array_udiff_variation5.phpt`, `array_uintersect_assoc_basic2.phpt`, `array_uintersect_assoc_variation5.phpt`, `array_uintersect_uassoc_variation6.phpt` |

The direct implementation covers one known broad-run blocker,
`ext/standard/tests/array/array_fill_error2.phpt`, at the runtime layer. After
the `ptn-7xxw` classifier update, bounded PHPT runs still classify that row as
resource-limited instead of executing it. The remaining 20-row frontier is split
across at least five runtime surfaces, so it is recorded here as the blocker map
rather than claimed as a single high-yield implementation slice.
