# PTN Progress

Refresh: 2026-06-10T13:35Z
Measured: `ptn-1f5` rebased on `origin/master@1cbcfe1`; recursive `mkdir()`,
public-static class callables, string/static-callable `array_map()`, and named
`array_walk()` evidence.

## Test Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native compiled PHP snippets | 358 | 358 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 200 | 152 | 48 |
| PHPT Zend rows | 76 | 68 | 8 |
| PHPT ext/standard rows | 77 | 48 | 29 |
| PHPT tests/basic+func+lang | 45 | 34 | 11 |
| PHPT other rows | 2 | 2 | 0 |
| COW contract spec tests | 7 | 7 | 0 |
| Focused COW reducer snippets | 45 | 45 | 0 |
| Recursive reference diagnostics | 4 | 4 | 0 |
| COW oracle suite | 22 | 22 | 0 |
| By-reference foreach COW oracle | 11 | 11 | 0 |
| Mutating-internal COW matrix | 14 | 14 | 0 |
| Post-merge COW gate | 25 | 25 | 0 |
| PHPT COW manifest | 29 | 26 | 3 |
| PHPT callback manifest | 2 | 2 | 0 |

## COW PHPT Buckets

`tools/phpt-cow-manifest.txt` has 29 rows: 26 passing, 3 failing.
`bug35163.phpt` and `assign_by_val_function_by_ref_return_value.phpt` pass.
Named `array_walk()` callbacks observe `$GLOBALS` swaps of the walked variable.
Exact closure-backed callback rows remain blocked by Closure/callable values
(`ptn-dis`). Details: `docs/COW_PHPT_BLOCKERS_2026-06-09.md`.

## Already Ported

Lexer/parser, AST, IR, C backend, boxed values, variables, constants,
string/math/type internals, ordered arrays, `foreach`, cursors,
`array_values()`, numeric-string keys, payload refcounts, array/string COW,
references, by-reference parameters and `foreach`, array dimensions,
temporaries, recursive/user functions, magic constants, `func_*`, `print_r`,
binary strings, string offsets, scalar diagnostics, array literal references,
array union `+`, scalar type hints, by-reference return boundaries, `count()`,
`??`, assignment expressions, expression-level `@`, selected file APIs
including recursive `mkdir()`, `rmdir()`, `is_dir()`, `is_file()`, and
`file_exists()`, array-path snapshots, `array_sum()`/`strtr()`/`in_array()`,
recursive array merge/replace, `debug_zval_dump()`, dynamic lvalue-reference
calls, append/list assignment expressions, nested same-array reference lvalues,
direct-variable and offset-form `??=`, grouped reference targets,
`array_fill_keys()`, string-callable `call_user_func()`, public static methods
as `Class::method` callables, string/static-callable and null-callback
`array_map()`, and named `array_walk()` callbacks that rebind the walked global
array.

## Still Needed

Remaining COW PHPT gaps are Closure/callable `use` syntax for the exact
`array_walk()`/`$GLOBALS` row, closure-backed callback by-reference returns,
`array_reduce()` callback/refcount behavior, and broader recursive
by-reference return edges. Broader bounded-PHPT gaps are objects, full class
metadata, visibility checks, unsupported array/string internals, 64-bit
operator exactness, foreach diagnostics, object/property compound lvalues,
scalar offset-lvalue fatal parity, and broader file APIs beyond the current
filesystem subset.

## Verification

Commands: `cargo fmt --check`; `cargo test`; focused callback PHPT evidence.
`array_map_object1.phpt` remains blocked by unsupported class members and
visibility/object-model semantics before full row parity.
