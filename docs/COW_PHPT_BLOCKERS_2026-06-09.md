# COW PHPT Blockers: 2026-06-10 Refresh

Evidence base: `ptn-ept` rebased on `origin/master@df737c70`, refreshed at
2026-06-10T14:44Z. The branch adds generic `array_reduce()` accumulator
ownership/debug refcount behavior through the callable-value dispatcher and
raises the focused COW manifest to 28/29.

## Focused COW Counts

| Bucket | Total | Pass | Fail |
| --- | ---: | ---: | ---: |
| assignment-aliasing | 4 | 4 | 0 |
| string-offsets | 4 | 4 | 0 |
| array-writes-appends-unset | 4 | 4 | 0 |
| nested-arrays | 4 | 4 | 0 |
| foreach-mutation | 4 | 3 | 1 |
| function-boundaries | 4 | 4 | 0 |
| reference-interaction | 5 | 5 | 0 |
| **Total** | **29** | **28** | **1** |

## Remaining Blocker Rows

| Bucket | PHPT row | Current generic blocker |
| --- | --- | --- |
| foreach-mutation | `ext/standard/tests/array/array_walk/bug69068_2.phpt` | Anonymous Closure/callable `use` syntax; named `array_walk()` `$GLOBALS` swap has native reducer coverage. |

## Verification

- `cargo fmt --check`; `cargo build --bin phpc`.
- `cargo test`: source unit 3/3, compile_native 367/367, COW reducer 4/4;
  `/tmp` filled during `cow_oracle`, then `TMPDIR=target/tmp cargo test --test
  cow_oracle -- --nocapture` passed 22/22.
- `TMPDIR=target/tmp cargo test --test cow_payload_contract --test
  foreach_by_ref_cow -- --nocapture`: 7/7 and 11/11.
- focused PHPT `array_reduce_accumulator_refcount.phpt`: pass.
- `tools/run-bounded-phpt.sh tools/phpt-cow-manifest.txt`: 28/29 on
  2026-06-10T14:43Z; remaining failure is the row listed above.
- Prior `tools/run-phpt-manifest.sh tools/phpt-manifest-200.txt`: 152/200;
  Zend 68/76, ext/standard 48/77, tests/basic+func+lang 34/45, other 2/2.
