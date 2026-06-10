# COW PHPT Blockers: 2026-06-10 Refresh

Evidence base: `ptn-ept` on `origin/master@1c445f8`, refreshed at
2026-06-10T12:45Z. The focused COW manifest is 26/29 after recursive
array-literal support and named `array_reduce()` accumulator debug refcount
coverage.

## Focused COW Counts

| Bucket | Total | Pass | Fail |
| --- | ---: | ---: | ---: |
| assignment-aliasing | 4 | 4 | 0 |
| string-offsets | 4 | 4 | 0 |
| array-writes-appends-unset | 4 | 4 | 0 |
| nested-arrays | 4 | 4 | 0 |
| foreach-mutation | 4 | 3 | 1 |
| function-boundaries | 4 | 2 | 2 |
| reference-interaction | 5 | 5 | 0 |
| **Total** | **29** | **26** | **3** |

## Remaining Blocker Rows

| Bucket | PHPT row | Current generic blocker |
| --- | --- | --- |
| foreach-mutation | `ext/standard/tests/array/array_walk/bug69068_2.phpt` | Closure callback mutation with `$GLOBALS` swap. |
| function-boundaries | `ext/standard/tests/array/array_reduce_accumulator_refcount.phpt` | Exact row still needs Closure/callable support on this branch; named callback accumulator refcounts are covered. |
| function-boundaries | `ext/standard/tests/array/array_reduce_return_by_ref.phpt` | Exact row still needs Closure/callable support on this branch; named callback by-reference return is covered. |

## Verification

- `cargo fmt --check`; `cargo test`: 3 source unit, 353 native snippets, COW
  contract 7/7, focused reducer cases 45/45, COW oracle 22/22,
  foreach-by-ref oracle 11/11, recursive diagnostics 4/4.
- `tools/run-native-smoke-matrix.sh`: 6/6.
- `tools/run-post-merge-cow-gate.sh`: 25/25, split as 15 oracle, 3 notice,
  and 7 diagnostic cases.
- `tools/run-bounded-phpt.sh tools/phpt-cow-manifest.txt`: 26/29.
