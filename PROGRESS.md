# PTN Progress

Refresh: 2026-06-11T20:02Z
Measured: `ptn-xy5f` rebased on `origin/master` at `dac95008d`.
Recent slices cover `array_column()`, `array_filter()` mode `ValueError`s,
foreach boolean diagnostics and assignment-target bindings, shared
deprecation separators, public `__construct` dispatch, catchable arithmetic
`TypeError`s with concrete object/exception/closure class names, inline HTML
output, shared string-internal argument `TypeError`s, string-offset assign-op
diagnostics, public `__call` object-callable fallback, `is_callable()` subset
validation, `phpc -d precision=N` float stringification, scalar `var_dump()`
float spelling, `assert()` with catchable `AssertionError`,
compiler-generated direct-call text, explicit descriptions, and
PHP-compatible empty dynamic-call fallback, plus public declared
`__toString()` for runtime string conversions.

Exact `strlen.phpt` now gets the object `__toString()` length right; it still
fails on ordering/source-path parity for the `${str}` deprecation versus the
undefined `$strS` warning.

## RC Surface

The release-candidate path covers parser/IR/C backend, boxed values,
variables/constants, strings, scalar operators, ordered arrays, `foreach`,
branch/loop/switch control flow, compile-time-resolved includes, selected
standard internals, COW/reference slices, top-level functions, call-frame
introspection, scalar type hints, bounded closures/callables, `stdClass`,
public class/object shells, direct public static properties, public property
writes/`??=`, inherited public methods, public constructor dispatch,
diagnostic filtering, catchable arithmetic/string-internal/assertion
boundaries, inline HTML output, string-offset assign-op diagnostics, foreach
assignment targets, public `__call` fallback for object calls/callables,
`is_callable()` subset validation, public `__toString()` string conversion,
scalar float stringification/`var_dump()` spelling, plain heredoc/nowdoc
literals, and string interpolation slices.

## Test Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 464 | 464 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 200 | 185 | 15 |
| PHPT Zend rows | 76 | 76 | 0 |
| PHPT ext/standard rows | 77 | 68 | 9 |
| PHPT tests/basic+func+lang | 45 | 39 | 6 |
| PHPT other rows | 2 | 2 | 0 |
| PHPT COW manifest | 29 | 29 | 0 |
| Post-merge COW gate | 25 | 25 | 0 |
| PHPT callback manifest | 2 | 2 | 0 |

## Remaining Bounded Failures

- Array internals: `001`, `005`, `007`, `008`, and
  `array_key_exists_variation1`.
- String/output: `004`, `005`, `006`, and the remaining `strlen`
  diagnostic-order/source-path parity.
- Control/lang: `tests/lang/024.phpt` reaches the dynamic-variable
  array-offset lvalue blocker after inline HTML.
- Numeric/operators: `add_basiclong_64bit` and the remaining 64-bit bitwise
  rows after `zend-pow-assign`, `offset_assign`, `add_variationStr`, and
  object/array add diagnostics.

## Post-RC Architecture

Follow-ups: full visibility/inheritance, typed/non-public/promoted properties,
interfaces/traits, namespaces, class constants, reflection, remaining magic
methods beyond the current public `__call` and `__toString` slices,
first-class callable syntax, old-style constructors, destructors, broader
static properties, object destructuring/`Traversable`, property
compound/static lvalues, exceptions/resources, dynamic include/include_once,
heredoc interpolation/flexible indentation, unsupported internals, exact
64-bit operator/diagnostic parity, scalar offset-lvalue parity, assertion
configuration side effects, broader foreach destructuring/reference targets,
and non-direct-variable or non-numeric inc/dec.

## Verification

Evidence after the `ptn-9d28` rebase includes bounded PHPT
`manifest-20260611T192948Z.txt` (183/200), exact add-row
`manifest-20260611T194807Z.txt` (2/2), exact
`Zend/tests/ast/zend-pow-assign.phpt` in `manifest-20260611T194054Z.txt`
(1/1), COW PHPT `manifest-20260611T194117Z.txt` (29/29),
`compile_assert_internal_to_native_binary`, full `cargo test` with native
target 463/463 plus reducer/oracle tails, native smoke matrix (6/6), and
post-merge COW gate (25/25). Post-`ptn-ttud` baseline exact `strlen.phpt`
`run-20260611T194435Z-manifest.log` failed on object `__toString()` and
diagnostic ordering; `ptn-xy5f` exact rerun
`run-20260611T195220Z-manifest.log` removes the object diff and leaves only
diagnostic ordering. Focused native coverage includes
`compile_object_to_string_conversion_to_native_binary`,
`compile_scalar_echo_keeps_direct_output_path_to_native_binary`,
`compile_print_expression_contexts_to_native_binary`,
`compile_strlen_expression_to_native_binary`,
`compile_string_internals_reject_non_string_arrays_to_native_binary`, and
`compile_string_internals_use_direct_string_operand_fast_paths_to_native_binary`.
`ptn-xy5f` COW PHPT `run-20260611T195415Z-*` passes 29/29.
