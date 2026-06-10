# COW PHPT Blockers: 2026-06-10 Refresh

Evidence base: `ptn-nvs` on `origin/master@df737c70`, refreshed at
2026-06-10T14:49Z. The focused COW status is 28/29 after closure `use`
captures cover `array_walk()` callback-visible `$GLOBALS` swaps in
`bug69068_2.phpt`.

## Focused COW Counts

| Bucket | Total | Pass | Fail |
| --- | ---: | ---: | ---: |
| assignment-aliasing | 4 | 4 | 0 |
| string-offsets | 4 | 4 | 0 |
| array-writes-appends-unset | 4 | 4 | 0 |
| nested-arrays | 4 | 4 | 0 |
| foreach-mutation | 4 | 4 | 0 |
| function-boundaries | 4 | 3 | 1 |
| reference-interaction | 5 | 5 | 0 |
| **Total** | **29** | **28** | **1** |

## Remaining Blocker Rows

| Bucket | PHPT row | Current generic blocker |
| --- | --- | --- |
| function-boundaries | `ext/standard/tests/array/array_reduce_accumulator_refcount.phpt` | `array_reduce()` callback/refcount behavior. |

## Verification

- `cargo check`: pass.
- `cargo test compile_array_walk_closure_globals_swap_cow_to_native_binary
  --test compile_native`: pass.
- `cargo test anonymous --test compile_native`: 3/3 pass.
- `cargo test array_walk --test compile_native`: 2/2 pass.
- focused PHPT `bug69068_2.phpt`: pass.
- focused PHPT `array_reduce_accumulator_refcount.phpt`: fails high refcounts.
- `tools/run-bounded-phpt.sh tools/phpt-cow-manifest.txt`: 28/29 on
  2026-06-10T14:49Z; remaining failure is the blocker row listed above.
- Prior `tools/run-phpt-manifest.sh tools/phpt-manifest-200.txt`: 152/200;
  Zend 68/76, ext/standard 48/77, tests/basic+func+lang 34/45, other 2/2.
