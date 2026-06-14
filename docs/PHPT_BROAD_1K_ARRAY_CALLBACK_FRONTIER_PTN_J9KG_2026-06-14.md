# PHPT Broad 1k Array Callback Frontier: ptn-j9kg

Issue: `ptn-j9kg`

This slice refreshed the broad 1k PHPT classifier on `origin/master` and
selected the standard-array callback/set-operation frontier. No compiler
behavior is changed here: the current 66-row frontier still splits across
several runtime semantics, so this is a blocker map with focused row counts
rather than an implementation claim.

## Broad 1k Baseline

Source state:

- PTN: `80da9cd3a587`
- php-src PHPT corpus: `8c63ec400ce8e07c57a8d9499317b96a8beafb8b`

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-j9kg-baseline-before
```

Generated broad manifest:

```text
.runtime/ptn-j9kg-baseline-before/20260614T032211Z/phpt-baseline-1000.txt
```

Classification artifacts:

```text
.runtime/phpt-progress/classification-20260614T032211Z.tsv
.runtime/phpt-progress/runnable-20260614T032211Z.txt
.runtime/phpt-progress/summary-20260614T032211Z.txt
```

Result:

| Measurement | Selected | Runnable | Excluded |
| --- | ---: | ---: | ---: |
| broad 1k classify-only | 1,000 | 430 | 570 |

Top excluded buckets:

| Bucket | Rows |
| --- | ---: |
| `unsupported-language` | 281 |
| `unsupported-class-metadata` | 144 |
| `unsupported-request-input-ini` | 28 |
| `unsupported-extension` | 20 |
| `unsupported-diagnostics-runtime` | 17 |
| `unsupported-assertion-ini` | 17 |
| `unsupported-resource-limit-ini` | 15 |
| `sapi-behavior` | 13 |

The broad runnable set contains 296 `ext/standard/tests/array` rows. The
largest callback-adjacent slice is the 66-row manifest committed as
`tools/phpt-array-callback-validation-manifest.txt`.

## Focused Evidence

Focused manifest generation:

```sh
awk '/^ext\/standard\/tests\/array\/array_(map|filter|reduce|diff_u|intersect_u|udiff|uintersect)/ {print}' \
  .runtime/phpt-progress/runnable-20260614T032211Z.txt \
  > .runtime/ptn-j9kg-array-callback-frontier.txt
tools/run-bounded-phpt.sh .runtime/ptn-j9kg-array-callback-frontier.txt
```

Equivalent committed manifest:

```sh
tools/run-bounded-phpt.sh tools/phpt-array-callback-validation-manifest.txt
```

Run artifact:

```text
.runtime/phpt-progress/run-20260614T032740Z-manifest.log
```

Focused result:

| Focused set | Selected | Runnable | Passed | Failed |
| --- | ---: | ---: | ---: | ---: |
| Array callback/set-operation frontier | 66 | 66 | 49 | 17 |

Relative to the prior `ptn-x6x5` callback map, the current broad slice adds
`array_reduce_return_by_ref.phpt` and improves from 46/65 to 49/66. The
remaining 17 failures are still mixed across runtime subsystems, so no single
small generic patch is credible for the full 66-row frontier.

## Remaining Blockers

| Rows | Blocker | Representative rows |
| ---: | --- | --- |
| 2 | `array_filter()` mode validation and key/value callback argument shape | `array_filter_invalid_mode.phpt`, `array_filter_variation10.phpt` |
| 2 | `array_map()` invalid callback/object-callable diagnostics | `array_map_error.phpt`, `array_map_object2.phpt` |
| 5 | `array_map()` callback return, reference, zipping, and binary string parity | `array_map_variation10.phpt`, `array_map_variation12.phpt`, `array_map_variation2.phpt`, `array_map_variation7.phpt`, `array_map_variation9.phpt` |
| 1 | `array_reduce()` accumulator and initial-value parity | `array_reduce_variation1.phpt` |
| 7 | User-comparator set-operation matching, callback precedence, ordering, and duplicate handling | `array_udiff_assoc_variation.phpt`, `array_udiff_assoc_variation5.phpt`, `array_udiff_uassoc_variation6.phpt`, `array_udiff_variation5.phpt`, `array_uintersect_assoc_basic2.phpt`, `array_uintersect_assoc_variation5.phpt`, `array_uintersect_uassoc_variation6.phpt` |

## Next Credible Splits

1. Fix `array_filter()` mode/key callback dispatch through the shared internal
   callback path, then re-run this manifest.
2. Split `array_map()` into diagnostic/object-callable, by-reference result,
   multi-array zip, and binary-safe string sub-slices.
3. Implement comparator ordering and duplicate semantics for `array_udiff*()`
   and `array_uintersect*()` through the custom set-operation helper.

## Verification

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-j9kg-baseline-before
tools/run-bounded-phpt.sh .runtime/ptn-j9kg-array-callback-frontier.txt
```

The focused run exits non-zero because 17 known blocker rows still fail; the
row counts above are the deliverable for this blocker-map slice.
