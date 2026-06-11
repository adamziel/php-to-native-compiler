# PTN Progress

Refresh: 2026-06-11T19:49Z
Measured: `ptn-9d28` rebased on `origin/master` at `75bfa2cc1`.
Recent slices cover `array_column()`, `array_filter()` mode `ValueError`s,
foreach boolean diagnostics and assignment-target bindings, shared
deprecation separators, public `__construct` dispatch, catchable arithmetic
`TypeError`s with concrete object/exception/closure class names, inline HTML
output, shared string-internal argument `TypeError`s, string-offset assign-op
diagnostics, public `__call` object-callable fallback, `is_callable()` subset
validation, `phpc -d precision=N` float stringification, scalar `var_dump()`
float spelling, and `assert()` with catchable `AssertionError`,
compiler-generated direct-call text, explicit descriptions, and
PHP-compatible empty dynamic-call fallback.

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
`is_callable()` subset validation, scalar float stringification/`var_dump()`
spelling, plain heredoc/nowdoc literals, and string interpolation slices.

## Test Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 463 | 463 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 200 | 185 | 15 |
| PHPT Zend rows | 76 | 76 | 0 |
| PHPT ext/standard rows | 77 | 68 | 9 |
| PHPT tests/basic+func+lang | 45 | 39 | 6 |
| PHPT other rows | 2 | 2 | 0 |
| PHPT COW manifest | 29 | 29 | 0 |
| Post-merge COW gate | 25 | 25 | 0 |
| PHPT callback manifest | 2 | 2 | 0 |

## Frozen Failure Clusters

- `ptn-lrty.5`: 5 numeric/operator rows remain after `zend-pow-assign`,
  `offset_assign`, `add_variationStr`, and object/array add diagnostics.
- `ptn-lrty.3`: 5 array-internal rows remain: `001`, `005`, `007`, `008`,
  and `array_key_exists_variation1`.
- `ptn-lrty.4`: 4 string/output rows remain: `004`, `005`, `006`, and
  `strlen`.
- `ptn-lrty.6` plus `ptn-r52`: 1 control-flow/lang row remains after
  `foreachLoop.003.phpt` and `foreachLoop.004.phpt`; `tests/lang/024.phpt`
  reaches the dynamic-variable array-offset lvalue blocker after inline HTML.

## Post-RC Architecture

Follow-ups: full visibility/inheritance, typed/non-public/promoted properties,
interfaces/traits, namespaces, class constants, reflection, remaining magic
methods, first-class callable syntax, old-style constructors, destructors,
broader static properties, object destructuring/`Traversable`, property
compound/static lvalues, exceptions/resources, dynamic include/include_once,
heredoc interpolation/flexible indentation, unsupported internals, exact
64-bit operator/diagnostic parity, scalar offset-lvalue parity, assertion
configuration side effects, broader foreach destructuring/reference targets,
and non-direct-variable or non-numeric inc/dec.

## Verification

Evidence after final rebase: bounded PHPT
`manifest-20260611T192948Z.txt` (183/200) plus exact add-row
`manifest-20260611T194807Z.txt` (2/2), exact
`Zend/tests/ast/zend-pow-assign.phpt` in `manifest-20260611T194054Z.txt`
(1/1), COW PHPT `manifest-20260611T194117Z.txt` (29/29),
`compile_assert_internal_to_native_binary`, `cargo fmt --check`, full
`cargo test` (native target 463/463 plus reducer/oracle tails), native smoke
matrix (6/6), and post-merge COW gate (25/25).
