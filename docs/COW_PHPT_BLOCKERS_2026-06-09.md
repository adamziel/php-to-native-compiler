# COW PHPT Blockers: 2026-06-10 Refresh

Evidence base: `ptn-nvs` retry rebased on `origin/master@bb82827d`,
refreshed at 2026-06-10T13:10Z. The focused COW manifest is 27/29 after
combining current recursive array-literal/function-boundary work with
anonymous `array_walk()` closure callbacks, `use` captures, and `$GLOBALS`
single-key array writes.

## Focused COW Counts

| Bucket | Total | Pass | Fail |
| --- | ---: | ---: | ---: |
| assignment-aliasing | 4 | 4 | 0 |
| string-offsets | 4 | 4 | 0 |
| array-writes-appends-unset | 4 | 4 | 0 |
| nested-arrays | 4 | 4 | 0 |
| foreach-mutation | 4 | 4 | 0 |
| function-boundaries | 4 | 2 | 2 |
| reference-interaction | 5 | 5 | 0 |
| **Total** | **29** | **27** | **2** |

## Remaining Blocker Rows

| Bucket | PHPT row | Current generic blocker |
| --- | --- | --- |
| function-boundaries | `ext/standard/tests/array/array_reduce_accumulator_refcount.phpt` | Closure-backed `array_reduce()` accumulator refcount parity. |
| function-boundaries | `ext/standard/tests/array/array_reduce_return_by_ref.phpt` | Closure-backed callback return by reference through `array_reduce()`. |

## Verification

- `cargo fmt --check`; `cargo test`: 3 source unit, 355 native snippets, COW
  contract 7/7, focused reducer cases 45/45, COW oracle 22/22,
  foreach-by-ref oracle 11/11, recursive diagnostics 4/4.
- `tools/run-native-smoke-matrix.sh`: 6/6.
- `tools/run-post-merge-cow-gate.sh`: 25/25, split as 15 oracle, 3 notice,
  and 7 diagnostic cases.
- `tools/run-bounded-phpt.sh tools/phpt-cow-manifest.txt`: 27/29.
