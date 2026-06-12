# PTN Progress

Refresh: 2026-06-12T01:12Z
Measured: `ptn-9dn7` on `origin/master` at `9bf0f4051`.

Recent RC slices cover array/key canonicalization, foreach targets, catchable
arithmetic/assertion boundaries, public `__call`/`__toString`, scalar
`var_dump()`, inline HTML, string-offset diagnostics, boxed streams,
`array_key_exists()` parity, PHP-style float output, `ksort()`/`shuffle()`,
array cursor/pop/shift mutation, literal-array defaults, `pow()`,
`array_merge()`, `call_user_func_array()`, `phpc -d error_reporting=N`,
filtered bitwise diagnostics, legacy `${var}` deprecations, scalar/array
`var_export()`, `array_diff*()`/`array_intersect*()`/`array_udiff*()`,
bounded `highlight_string()`/`highlight_file()`, statement-form array-offset
`++`/`--`, `join()`/`implode()`, scalar `sprintf()`, and simple
dynamic-variable array/string-offset assignments.

Recent PHPT movers: `ptn-dcyl` exact `strings/006`, `ptn-e3zm` focused
`array_udiff*()`, `ptn-bhp6` exact `strings/004`, `ptn-e3ha`
`tests/lang/024` via `${expr}[key] = value`, and this slice's broader
dynamic-root reducer covering nested names, appends, string offsets, and
target-before-RHS evaluation order.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 488 | 488 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 200 | 199 | 1 |
| PHPT Zend rows | 76 | 76 | 0 |
| PHPT ext/standard rows | 77 | 76 | 1 |
| PHPT tests/basic+func+lang | 45 | 45 | 0 |
| PHPT other rows | 2 | 2 | 0 |
| PHPT COW manifest | 29 | 29 | 0 |
| Post-merge COW gate | 25 | 25 | 0 |
| PHPT callback manifest | 2 | 2 | 0 |

## RC Surface

Parser/IR/C backend, boxed values, variables/constants, strings, scalar
operators, ordered arrays, `foreach`, branch/loop/switch, compile-time
includes, selected internals, COW/reference slices, user functions,
call-frame introspection, scalar type hints with literal-array defaults,
bounded closures/callables, `stdClass`, public class/object shells, public
properties, public constructors, `is_callable()`, assertions,
heredoc/nowdoc, interpolation, streams, `pow()`, `array_merge()`,
`join()`/`implode()`, scalar `sprintf()`, `call_user_func_array()`,
CLI/error-reporting wiring, highlight output paths, scalar/array
`var_export()`, direct array mutators, set operations, statement array-offset
inc/dec, and simple dynamic-variable array/string-offset writes.

## Remaining Bounded Failures

- `ext/standard/tests/array/007.phpt`: covered diff/intersect/udiff reducers
  still stop at unsupported non-public class members in the full row.

## Verification

This branch ran `cargo fmt --check`, focused
`cargo test dynamic_variable_array_dimension`, exact `tests/lang/024.phpt`
green, bounded PHPT 199/200 with only `array/007` failing, and COW PHPT
29/29. Prior `ptn-e3ha` full-suite evidence remains the broad Rust baseline.

Follow-ups remain visibility/inheritance, typed/promoted properties,
interfaces/traits, namespaces, reflection, remaining magic methods,
first-class callables, destructors, dynamic includes, unsupported internals,
scalar offset-lvalues, assertion configuration, non-direct-variable or
expression inc/dec, and broader foreach destructuring/reference targets.
