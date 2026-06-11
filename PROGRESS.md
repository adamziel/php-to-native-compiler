# PTN Progress

Refresh: 2026-06-11T22:58Z
Measured here on `ptn-k14b` rebased on `origin/master` at `b62428b21`
plus local slice changes.

Recent RC slices cover parser/IR/C backend, boxed values, ordered arrays,
strings, scalar operators, selected internals, COW/reference slices, user
functions, call-frame introspection, scalar type hints, bounded callables,
public class/object shells, public properties/static properties, public
constructors, public `__call`/`__toString`, `is_callable()`, assertion errors,
heredoc/nowdoc literals, interpolation diagnostic ordering, stream resources,
variable-root array-path mutation, `pow()`, `array_merge()`,
`call_user_func_array()`, CLI `error_reporting`, bounded
`highlight_string()`/empty output-buffer reads, bitwise integer-conversion
diagnostic filtering, and direct array mutators through `shuffle`.

`ptn-pjah` extends minimal resources into boxed stream resources: `fopen()`
keeps streams open, `fclose()` closes them, `is_resource()` is false after
close, and `gettype()` reports `resource (closed)`.

`ptn-k14b` adds scalar/array `var_export()` plus non-callback `array_diff()`,
`array_diff_assoc()`, `array_intersect()`, and `array_intersect_assoc()` through
the shared ordered-array runtime. Array set comparisons use PHP's stringified
value comparison and preserve keys from the first array.

## Current Evidence

| Signal | Latest result |
| --- | --- |
| Rust native/compiler suite | `cargo test` passes; `compile_native` is 480/480 |
| Focused native reducer | `compile_var_export_and_array_set_internals_to_native_binary` passes |
| Exact PHPT array frontier | `array/008.phpt` passes; `array/007.phpt` reaches the later non-public class-member blocker |
| PHPT COW manifest | 29/29 on `6cdf19c77` before this slice |
| Bounded PHPT full manifest | latest full run before `ptn-k14b` was 195/200; full post-change rerun pending |

## Remaining Bounded Work

- Array frontier: `array/007.phpt` still fails when the row reaches private
  class members and callback-based `array_udiff*` variants. The scalar/array
  `array_diff*` section is now covered, and exact `array/008.phpt` passes.
- String/output frontier: `strings/004.phpt` and `strings/006.phpt` remain
  around array-element inc/dec and `highlight_file()`/output-buffer support;
  `strlen.phpt` is covered on the rebased base.
- Language/control frontier: `tests/lang/024.phpt` remains at the
  dynamic-variable array-offset lvalue blocker.

Follow-ups remain broad visibility/inheritance, typed/non-public/promoted
properties, interfaces/traits, namespaces, reflection, remaining magic methods,
first-class callables, destructors, broader resources/exceptions, dynamic
includes, heredoc interpolation, unsupported internals, scalar offset-lvalues,
assertion configuration, non-direct-variable or non-numeric inc/dec, and broader
foreach destructuring/reference targets.
