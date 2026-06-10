# COW PHPT Blockers: 2026-06-10 Refresh

Evidence base: `ptn-j8p` rebased on `origin/master@1cbcfe170`, refreshed at
2026-06-10T13:23Z. The focused COW status is 27/29 after recursive
array-literal work covers `assign_by_val_function_by_ref_return_value.phpt`,
named `array_walk()` covers callback-visible `$GLOBALS` swaps for plain
callbacks, and no-capture anonymous callback values cover
`array_reduce_return_by_ref.phpt`.

## Focused COW Counts

| Bucket | Total | Pass | Fail |
| --- | ---: | ---: | ---: |
| assignment-aliasing | 4 | 4 | 0 |
| string-offsets | 4 | 4 | 0 |
| array-writes-appends-unset | 4 | 4 | 0 |
| nested-arrays | 4 | 4 | 0 |
| foreach-mutation | 4 | 3 | 1 |
| function-boundaries | 4 | 3 | 1 |
| reference-interaction | 5 | 5 | 0 |
| **Total** | **29** | **27** | **2** |

## Remaining Blocker Rows

| Bucket | PHPT row | Current generic blocker |
| --- | --- | --- |
| foreach-mutation | `ext/standard/tests/array/array_walk/bug69068_2.phpt` | Anonymous Closure/callable `use` syntax; named `array_walk()` `$GLOBALS` swap has native reducer coverage. |
| function-boundaries | `ext/standard/tests/array/array_reduce_accumulator_refcount.phpt` | `array_reduce()` callback/refcount behavior. |

## Verification

- `cargo fmt --check`; `cargo check`; `cargo test`: source unit 3/3,
  compile_native 359/359, COW reducer 4/4, COW oracle 1/1, payload contract
  7/7, foreach-by-ref oracle 1/1.
- `cargo test anonymous`: 3/3 anonymous callback native reducer cases.
- `tools/run-native-smoke-matrix.sh`: 6/6.
- `tools/run-post-merge-cow-gate.sh`: 25/25, split as 15 oracle, 3 notice,
  and 7 diagnostic cases.
- focused PHPT `array_reduce_return_by_ref.phpt`: pass.
- focused PHPT `array_reduce_accumulator_refcount.phpt`: fails high refcounts.
- `tools/run-bounded-phpt.sh tools/phpt-cow-manifest.txt`: 27/29 on
  2026-06-10T13:25Z; remaining failures are the two blocker rows listed above.
- Prior `tools/run-phpt-manifest.sh tools/phpt-manifest-200.txt`: 152/200;
  Zend 68/76, ext/standard 48/77, tests/basic+func+lang 34/45, other 2/2.
