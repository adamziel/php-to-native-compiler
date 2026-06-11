# PTN Progress

Refresh: 2026-06-11T20:31Z
Measured: `ptn-lrty.3` rebased on `origin/master` at `636306708`.

Recent RC slices cover array/key canonicalization, foreach assignment targets,
catchable arithmetic/assertion boundaries, public `__call`, public
`__toString` string conversion, precision-driven float output, scalar
`var_dump()` spelling, inline HTML, and string-offset diagnostics. This slice
adds minimal resource values plus `fopen()`, `fclose()`, `is_resource()`, and
`array_key_exists()` parity for `null` deprecation and resource-key integer
casting.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 464 | 464 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 200 | 186 | 14 |
| PHPT Zend rows | 76 | 76 | 0 |
| PHPT ext/standard rows | 77 | 69 | 8 |
| PHPT tests/basic+func+lang | 45 | 39 | 6 |
| PHPT other rows | 2 | 2 | 0 |
| PHPT COW manifest | 29 | 29 | 0 |
| Post-merge COW gate | 25 | 25 | 0 |
| PHPT callback manifest | 2 | 2 | 0 |

## Remaining Bounded Failures

- `ptn-lrty.3`: 4 broad array-internal rows remain: `001`, `005`, `007`, and
  `008`. `array_column()` and `array_key_exists()` variants are covered.
- `ptn-lrty.4`: 4 string/output rows remain: `004`, `005`, `006`, and
  `strlen` diagnostic-order/source-path parity.
- `ptn-lrty.6` plus `ptn-r52`: `tests/lang/024.phpt` remains at the
  dynamic-variable array-offset lvalue blocker.
- `ptn-lrty.5`: 5 64-bit operator rows remain after object/array add
  diagnostics and `add_variationStr` are covered.

## Verification

Evidence: exact target PHPT rows
`array_key_exists.phpt`, `array_key_exists_variation1.phpt`, and
`array_key_exists_null_deprecation.phpt` pass 3/3. Bounded PHPT
`summary-20260611T201728Z.txt` is 186/200. COW PHPT
`summary-20260611T202912Z.txt` is 29/29. Callback PHPT
`summary-20260611T203116Z.txt` is 2/2. `cargo fmt --check`,
`cargo build --bin phpc`, `cargo test` with 464 native/compiler cases plus COW
tails, native smoke 6/6, and post-merge COW 25/25 all pass.

Follow-ups remain broad visibility/inheritance, typed/non-public/promoted
properties, interfaces/traits, namespaces, reflection, remaining magic methods,
first-class callables, destructors, broader resources/exceptions, dynamic
includes, heredoc interpolation, full unsupported-internal coverage, exact
64-bit operator parity, scalar offset-lvalues, assertion configuration, and
broader foreach destructuring/reference targets.
