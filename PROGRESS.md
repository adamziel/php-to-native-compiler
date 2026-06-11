# PTN Progress

Refresh: 2026-06-11T22:08Z
Measured: `ptn-na3m` rebased on `origin/master` at `06f1dcced`.

Recent RC slices cover array/key canonicalization, foreach targets, catchable
arithmetic/assertion boundaries, public `__call`/`__toString`, precision-driven
float output, scalar `var_dump()` spelling, inline HTML, string-offset
diagnostics, minimal resources, `array_key_exists()` null/resource-key parity,
PHP-style float exponent spelling, direct `ksort()`/`shuffle()`,
variable-root array-path cursor mutation, one-argument `array_pop()`/
`array_shift()`, literal-array defaults, `pow()`, `array_merge()`, bounded
`highlight_string()` output buffers, `phpc -d error_reporting=N`, and
`call_user_func_array()` through shared callable dispatch. `ptn-na3m` routes
bitwise integer-conversion diagnostics for `&`, `|`, `^`, and `~` through
runtime error-reporting filters, including float precision and out-of-range
float warnings; all four 64-bit bitwise basiclong PHPT rows pass.

Exact `strlen.phpt` gets object `__toString()` length right; it still fails on
ordering/source-path parity for the `${str}` deprecation versus the undefined
`$strS` warning.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 477 | 477 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 200 | 194 | 6 |
| PHPT Zend rows | 76 | 76 | 0 |
| PHPT ext/standard rows | 77 | 72 | 5 |
| PHPT tests/basic+func+lang | 45 | 44 | 1 |
| PHPT other rows | 2 | 2 | 0 |
| PHPT COW manifest | 29 | 29 | 0 |
| Post-merge COW gate | 25 | 25 | 0 |

## RC Surface

Parser/IR/C backend, boxed values, variables/constants, strings, scalar
operators, ordered arrays, `foreach`, branch/loop/switch, compile-time includes,
selected internals, COW/reference slices, top-level functions, call-frame
introspection, scalar type hints including literal-array defaults, bounded
closures/callables, `stdClass`, public class/object shells, public static
properties, public property writes/`??=`, inherited methods/constructors,
public `__call`, public `__toString`, `is_callable()`, assertion errors,
heredoc/nowdoc literals, string interpolation slices, variable-root array-path
cursor/single-pop/shift mutation, `pow()`, `array_merge()`,
`call_user_func_array()`, CLI `error_reporting` ini wiring, bounded
`highlight_string()`/empty output-buffer reads, error-reporting-aware bitwise
integer conversion diagnostics, and direct array mutators through `shuffle`.

## Remaining Bounded Failures

- `ptn-lrty.3`, `ptn-xery`, and `ptn-k95f`: 2 broad array-internal rows remain:
  `007` and `008`. `001`, `005`, `array_column()`, and `array_key_exists()`
  variants are covered.
- `ptn-lrty.4`, `ptn-loyg`, and `ptn-qm7v`: 3 string/output rows remain. `004`
  reaches array-element inc/dec; `005` is covered; `006` and `strlen`
  diagnostic parity remain.
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

Follow-ups remain broad visibility/inheritance, typed/non-public/promoted
properties, interfaces/traits, namespaces, reflection, remaining magic methods,
first-class callables, destructors, broader resources/exceptions, dynamic
includes, heredoc interpolation, full unsupported-internal coverage, remaining
scalar offset-lvalues, assertion configuration, non-direct-variable or
non-numeric inc/dec, and broader foreach destructuring/reference targets.
