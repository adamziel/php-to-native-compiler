# COW PHPT Blockers: 2026-06-10 Refresh

Evidence base: `origin/master` `e7d39874`, refreshed at
2026-06-10T10:46Z. Command:
`tools/run-bounded-phpt.sh tools/phpt-cow-manifest.txt`; summary:
`.runtime/phpt-progress/summary-20260610T104451Z.txt`.

## Focused COW Counts

| Bucket | Total | Pass | Fail |
| --- | ---: | ---: | ---: |
| assignment-aliasing | 4 | 4 | 0 |
| string-offsets | 4 | 4 | 0 |
| array-writes-appends-unset | 4 | 4 | 0 |
| nested-arrays | 4 | 3 | 1 |
| foreach-mutation | 4 | 3 | 1 |
| function-boundaries | 4 | 1 | 3 |
| reference-interaction | 5 | 5 | 0 |
| **Total** | **29** | **24** | **5** |

`ptn-4yt` and all nine children are closed. Newly reflected from latest master:
`Zend/tests/bug38469.phpt`,
`ext/standard/tests/array/array_merge_recursive_basic1.phpt`,
`ext/standard/tests/array/array_merge_replace_recursive_refs.phpt`, and
`Zend/tests/foreach/foreach_by_ref_repacking_insert.phpt`.

## Remaining Blocker Rows

| Bucket | PHPT row | Current generic blocker |
| --- | --- | --- |
| nested-arrays | `Zend/tests/bug35163.phpt` | Nested recursive reference lvalues and dumps. |
| foreach-mutation | `ext/standard/tests/array/array_walk/bug69068_2.phpt` | Closure callback mutation with `$GLOBALS` swap. |
| function-boundaries | `Zend/tests/assign_by_val_function_by_ref_return_value.phpt` | Assignment by reference from call result. |
| function-boundaries | `ext/standard/tests/array/array_reduce_accumulator_refcount.phpt` | `array_reduce()` callback/refcount behavior. |
| function-boundaries | `ext/standard/tests/array/array_reduce_return_by_ref.phpt` | Callback return by reference through `array_reduce()`. |

## Verification

- `cargo test`: 3 source unit, 344 native compiled snippets, COW contract 7/7,
  focused reducer cases 34/34, COW oracle 22/22, foreach-by-ref oracle 11/11,
  recursive reference diagnostics 9/9.
- `tools/run-native-smoke-matrix.sh`: 6/6.
- `tools/run-post-merge-cow-gate.sh`: 25/25.
- `tools/run-bounded-phpt.sh`: 150/200; Zend 67/76, ext/standard 47/77,
  tests/basic+func+lang 34/45, other 2/2. Summary:
  `.runtime/phpt-progress/summary-20260610T100242Z.txt`.
