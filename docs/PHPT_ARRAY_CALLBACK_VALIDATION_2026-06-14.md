# PHPT Array Callback Validation: 2026-06-14

Issue: `ptn-x6x5`

This slice starts from the broad 1k PHPT tier and narrows the standard-array
callback/set-operation frontier. It implements a generic callback-validation
step for internal helpers rather than shaping any individual PHPT output.

## Broad 1k Baseline

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only --out-dir .runtime/ptn-x6x5-baseline-before
```

Generated manifest:
`.runtime/ptn-x6x5-baseline-before/20260614T004103Z/phpt-baseline-1000.txt`

Corpus revision: `8c63ec400ce8e07c57a8d9499317b96a8beafb8b`

Classifier output:

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

Artifacts:

- `.runtime/phpt-progress/classification-20260614T004103Z.tsv`
- `.runtime/phpt-progress/runnable-20260614T004103Z.txt`
- `.runtime/phpt-progress/summary-20260614T004103Z.txt`

## Implementation

The runtime now validates callback operands before internal helper iteration in
the shared callback path:

- nullable callback diagnostics for `array_map()` and `array_filter()` use the
  PHP-style "valid callback or null" message.
- `array_reduce()` validates its callback with the same catchable `TypeError`
  helper used by `array_walk()`.
- user-comparator set helpers validate value/key comparator operands before
  treating variadic operands as array arguments, matching PHP's callback-first
  diagnostic order for these helpers.
- variadic comparator diagnostics omit synthetic parameter names, matching the
  observed PHP shape for `array_diff_ukey()` and `array_u*()` helpers.

Native coverage:

```sh
cargo test --test compile_native compile_array_callback_validation_to_native_binary -- --nocapture
```

Result: passed.

## Focused PHPT Evidence

Focused manifest:
`.runtime/ptn-x6x5-array-callback-validation-after.txt`

Generated from the broad runnable manifest with:

```sh
awk '/^ext\/standard\/tests\/array\/array_(map|filter|reduce|diff_u|intersect_u|udiff|uintersect)/ {print}' \
  .runtime/phpt-progress/runnable-20260614T004103Z.txt \
  > .runtime/ptn-x6x5-array-callback-validation-after.txt
tools/run-bounded-phpt.sh .runtime/ptn-x6x5-array-callback-validation-after.txt
```

Final after-run log:
`.runtime/phpt-progress/run-20260614T010408Z-manifest.log`

| Focused set | Selected | Runnable | Passed | Failed |
| --- | ---: | ---: | ---: | ---: |
| Array callback/set validation frontier | 65 | 65 | 46 | 19 |

Rows from prior blocker maps that now pass include:

```text
ext/standard/tests/array/array_diff_ukey_variation10.phpt
ext/standard/tests/array/array_intersect_ukey_variation8.phpt
ext/standard/tests/array/array_map_variation15.phpt
```

This is below the 25-row target for one implementation slice, so the remaining
work is kept as a blocker map rather than claimed as a broad frontier move.

## Remaining Focused Blockers

The 19 remaining focused failures split across multiple generic runtime
surfaces:

| Rows | Blocker | Representative rows |
| ---: | --- | --- |
| 1 | Catchable arity/array-argument diagnostics for set helpers | `array_diff_uassoc_error.phpt` |
| 2 | `array_filter()` mode and key/value callback argument shape | `array_filter_invalid_mode.phpt`, `array_filter_variation10.phpt` |
| 2 | `array_map()` error/object callable diagnostics | `array_map_error.phpt`, `array_map_object2.phpt` |
| 6 | `array_map()` callback result, reference, zip/null, and binary string semantics | `array_map_variation10.phpt`, `array_map_variation12.phpt`, `array_map_variation14.phpt`, `array_map_variation2.phpt`, `array_map_variation7.phpt`, `array_map_variation9.phpt` |
| 1 | `array_reduce()` accumulator/initial-value variation | `array_reduce_variation1.phpt` |
| 7 | User-comparator set-operation matching, duplicate/order, and comparator-result semantics | `array_udiff_assoc_variation.phpt`, `array_udiff_assoc_variation5.phpt`, `array_udiff_uassoc_variation6.phpt`, `array_udiff_variation5.phpt`, `array_uintersect_assoc_basic2.phpt`, `array_uintersect_assoc_variation5.phpt`, `array_uintersect_uassoc_variation6.phpt` |

Next credible splits:

1. Make array-helper argument validation catchable for array/arity failures
   before extending focused set-operation runs.
2. Split `array_map()` into null/zipping, references, binary-safe strings, and
   object-callable diagnostics instead of treating it as one helper bug.
3. Implement comparator ordering/duplicate semantics for `array_udiff*()` and
   `array_uintersect*()` through the shared custom set-operation path.
