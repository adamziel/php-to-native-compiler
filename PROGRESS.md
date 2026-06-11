# PTN Progress

Refresh: 2026-06-11T23:08Z
Measured: `ptn-ndqj` rebased on `origin/master` at `b62428b21`.

Recent RC slices cover array/key canonicalization, foreach targets, catchable
arithmetic/assertion boundaries, public `__call`/`__toString`, precision-driven
float output, scalar `var_dump()` spelling, inline HTML, string-offset
diagnostics, minimal resources, `array_key_exists()` null/resource-key parity,
PHP-style float exponent spelling, direct `ksort()`/`shuffle()`,
variable-root array-path cursor mutation, one-argument `array_pop()`/
`array_shift()`, literal-array defaults, `pow()`, `array_merge()`, bounded
`highlight_string()` output buffers, `phpc -d error_reporting=N`,
`call_user_func_array()` through shared callable dispatch, and filtered bitwise
integer-conversion diagnostics. `ptn-snsk` emits legacy `${var}` string
interpolation deprecations before runtime execution and routes undefined
variable warnings through stdout formatting, so exact `strlen.phpt` passes.
`ptn-pjah` extends minimal resources into boxed stream resources: `fopen()`
keeps streams open, `fclose()` closes them, `is_resource()` is false after
close, and `gettype()` reports `resource (closed)`. `ptn-ndqj` adds
scalar/array `var_export()` plus ordered-array `array_diff()`,
`array_diff_assoc()`, `array_intersect()`, and `array_intersect_assoc()`
through shared stringified-value comparison helpers; exact
`ext/standard/tests/array/008.phpt` now passes.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 478 | 478 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 200 | 196 | 4 |
| PHPT Zend rows | 76 | 76 | 0 |
| PHPT ext/standard rows | 77 | 73 | 4 |
| PHPT tests/basic+func+lang | 45 | 44 | 1 |
| PHPT other rows | 2 | 2 | 0 |
| PHPT COW manifest | 29 | 29 | 0 |
| Post-merge COW gate | 25 | 25 | 0 |

## RC Surface

Parser/IR/C backend, boxed values, variables/constants, strings, scalar
operators, ordered arrays, `foreach`, branch/loop/switch, compile-time
includes, selected internals, COW/reference slices, top-level functions,
call-frame introspection, scalar type hints including literal-array defaults,
bounded closures/callables, `stdClass`, public class/object shells, public
properties/static properties, public constructors, public `__call`/
`__toString`, `is_callable()`, assertion errors, heredoc/nowdoc,
interpolation slices, stream resources, `pow()`, `array_merge()`,
`call_user_func_array()`, CLI `error_reporting` ini wiring, bounded
`highlight_string()`/empty output-buffer reads, error-reporting-aware bitwise
integer conversion diagnostics, scalar/array `var_export()`, and direct array
mutators and set operations including `shuffle`, `array_diff*()`, and
`array_intersect*()`.

## Remaining Bounded Failures

- `ext/standard/tests/array/007.phpt`: first diff reducers are covered, but the
  full row still reaches non-public class members and later `array_udiff*()`
  callback variants. `001`, `005`, `008`, `array_column()`, and
  `array_key_exists()` variants are covered.
- `ptn-lrty.4`, `ptn-loyg`, and `ptn-qm7v`: 2 string/output rows remain. `004`
  now reaches array-element inc/dec after sort/shuffle support; `005` and
  `strlen` are covered; `006` still needs highlight-file/output-buffer support.
- `ptn-lrty.6` plus `ptn-r52`: `tests/lang/024.phpt` remains at the
  dynamic-variable array-offset lvalue blocker.

## Verification

Evidence: exact `array_key_exists*` PHPT rows pass 3/3 for `ptn-lrty.3`; exact
`add_basiclong_64bit.phpt` passes for `ptn-icd9`; `strings/004.phpt` advances
for `ptn-loyg`; exact `array/005.phpt` passes for `ptn-xery`; exact
`array/001.phpt` passes for `ptn-k95f`; `strings/005.phpt` passes for
`ptn-qm7v`; exact `call_user_func_array_variation_001.phpt` passes for
`ptn-z8jv`; exact bitwise PHPT rows `bitwiseAnd_basiclong_64bit`,
`bitwiseNot_basiclong_64bit`, `bitwiseOr_basiclong_64bit`, and
`bitwiseXor_basiclong_64bit` pass 4/4 for `ptn-na3m`. Bounded PHPT
`summary-20260611T214758Z.txt` is 194/200. Focused `ptn-na3m` coverage includes
bitwise diagnostic filtering, out-of-range `~`, and existing bitwise native
rows.
`ptn-snsk` post-rebase evidence includes exact
`ext/standard/tests/strings/strlen.phpt` passing 1/1, bounded PHPT
`run-20260611T223244Z-manifest.log` at 195/200, COW PHPT
`run-20260611T222948Z-*` at 29/29, full `cargo test`, and
`cargo fmt --check`.
`ptn-pjah` pre-rebase evidence on `6cdf19c77` includes `cargo fmt --check`,
`git diff --check HEAD~1..HEAD`, full `cargo test` with native/compiler
478/478, native smoke 6/6, post-merge COW gate 25/25, and bounded PHPT
`manifest-20260611T223047Z.txt` at 194/200 before the `strlen` row landed.
It keeps `array_key_exists_variation1.phpt`, `strings/005.phpt`,
`call_user_func_array_variation_001.phpt`, and all 4 bitwise basiclong rows
passing. `ptn-pjah` adds focused native stream-resource coverage.
`ptn-ndqj` pre-rebase evidence on `6cdf19c77` includes full `cargo test`,
bounded PHPT `run-20260611T223638Z.log` at 195/200, exact
`ext/standard/tests/array/008.phpt` passing, exact `array/007.phpt` still
failing at non-public class members, and post-merge COW gate 15 oracle,
3 notice, and 7 diagnostic cases passing.

Follow-ups remain broad visibility/inheritance, typed/non-public/promoted
properties, interfaces/traits, namespaces, reflection, remaining magic
methods, first-class callables, destructors, broader resources/exceptions,
dynamic includes, heredoc interpolation, full unsupported-internal coverage,
remaining scalar offset-lvalues, assertion configuration, non-direct-variable
or non-numeric inc/dec, and broader foreach destructuring/reference targets.
