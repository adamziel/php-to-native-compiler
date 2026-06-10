# COW PHPT Blockers: 2026-06-10 Refresh

Evidence base: `ptn-2p4`, refreshed at 2026-06-10T11:48Z. The focused COW
manifest is 25/29 after nested same-array reference lvalues and recursive
`var_dump()` behavior close `Zend/tests/bug35163.phpt`.

## Focused COW Counts

| Bucket | Total | Pass | Fail |
| --- | ---: | ---: | ---: |
| assignment-aliasing | 4 | 4 | 0 |
| string-offsets | 4 | 4 | 0 |
| array-writes-appends-unset | 4 | 4 | 0 |
| nested-arrays | 4 | 4 | 0 |
| foreach-mutation | 4 | 3 | 1 |
| function-boundaries | 4 | 1 | 3 |
| reference-interaction | 5 | 5 | 0 |
| **Total** | **29** | **25** | **4** |

## Remaining Blocker Rows

| Bucket | PHPT row | Current generic blocker |
| --- | --- | --- |
| foreach-mutation | `ext/standard/tests/array/array_walk/bug69068_2.phpt` | Closure callback mutation with `$GLOBALS` swap. |
| function-boundaries | `Zend/tests/assign_by_val_function_by_ref_return_value.phpt` | Recursive self-reference diagnostics now block the PHPT row earlier; call-result value fallback is covered by native reducers. |
| function-boundaries | `ext/standard/tests/array/array_reduce_accumulator_refcount.phpt` | `array_reduce()` callback/refcount behavior. |
| function-boundaries | `ext/standard/tests/array/array_reduce_return_by_ref.phpt` | Callback return by reference through `array_reduce()`. |

## Verification

- `cargo fmt --check`; `cargo test`: 3 source unit, 346 native snippets, COW
  contract 7/7, focused reducer cases 41/41, COW oracle 22/22,
  foreach-by-ref oracle 11/11, recursive diagnostics 7/7.
- `tools/run-native-smoke-matrix.sh`: 6/6.
- `tools/run-post-merge-cow-gate.sh`: 25/25, split as 14 oracle, 1 notice,
  and 10 diagnostic cases.
- `tools/run-bounded-phpt.sh tools/phpt-cow-manifest.txt`: 25/29.
- `tools/run-phpt-manifest.sh tools/phpt-manifest-200.txt`: 150/200; Zend
  67/76, ext/standard 47/77, tests/basic+func+lang 34/45, other 2/2; broad
  manifest does not include `Zend/tests/bug35163.phpt`.
