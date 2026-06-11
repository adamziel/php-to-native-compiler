# PTN Progress

Refresh: 2026-06-12T00:25Z
Measured: `ptn-bhp6` rebased on `origin/master` at `9107cacdf`.

Recent RC slices cover array/key canonicalization, foreach targets, catchable
arithmetic/assertion boundaries, public `__call`/`__toString`, scalar
`var_dump()`, inline HTML, string-offset diagnostics, boxed streams,
`array_key_exists()` parity, PHP-style float output, `ksort()`/`shuffle()`,
array cursor/pop/shift mutation, literal-array defaults, `pow()`,
`array_merge()`, `call_user_func_array()`, `phpc -d error_reporting=N`,
filtered bitwise diagnostics, legacy `${var}` deprecations, scalar/array
`var_export()`, `array_diff*()`/`array_intersect*()`/`array_udiff*()`,
bounded `highlight_string()`/`highlight_file()`, and this slice's
statement-form array-offset `++`/`--`, `join()`/`implode()`, and scalar
`sprintf()`.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 486 | 486 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 200 | 198 | 2 |
| PHPT Zend rows | 76 | 76 | 0 |
| PHPT ext/standard rows | 77 | 76 | 1 |
| PHPT tests/basic+func+lang | 45 | 44 | 1 |
| PHPT other rows | 2 | 2 | 0 |
| PHPT COW manifest | 29 | 29 | 0 |
| Post-merge COW gate | 25 | 25 | 0 |
| PHPT callback manifest | 2 | 2 | 0 |

## RC Surface

Parser/IR/C backend, boxed values, variables/constants, strings, scalar
operators, ordered arrays, `foreach`, branch/loop/switch, compile-time
includes, selected internals, COW/reference slices, user functions, call-frame
introspection, scalar type hints with literal-array defaults, bounded
closures/callables, `stdClass`, public class/object shells, public properties,
public constructors, `is_callable()`, assertions, heredoc/nowdoc,
interpolation, streams, `pow()`, `array_merge()`, `join()`/`implode()`,
scalar `sprintf()`, `call_user_func_array()`, CLI/error-reporting wiring,
highlight output paths, scalar/array `var_export()`, direct array mutators,
set operations, and statement array-offset inc/dec.

## Remaining Bounded Failures

- `ext/standard/tests/array/007.phpt`: covered diff/intersect/udiff reducers
  still stop at unsupported non-public class members in the full row.
- `tests/lang/024.phpt`: dynamic-variable array-offset lvalue blocker.

## Verification

This slice keeps prior ptn-dcyl/e3zm/nveq highlight and callback coverage and
adds focused
`compile_array_offset_increment_decrement_statements_to_native_binary`,
`compile_join_and_implode_to_native_binary`, and
`compile_sprintf_scalar_formats_to_native_binary`. Branch evidence reported
exact `strings/004.phpt` green, bounded PHPT 198/200, and COW PHPT 29/29;
post-rebase refinery gates are rerun before merge.

Follow-ups remain visibility/inheritance, typed/promoted properties,
interfaces/traits, namespaces, reflection, remaining magic methods, first-class
callables, destructors, dynamic includes, unsupported internals, scalar
offset-lvalues, assertion configuration, non-direct-variable or expression
inc/dec, and broader foreach destructuring/reference targets.
