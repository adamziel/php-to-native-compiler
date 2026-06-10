# PTN Progress

Refresh: 2026-06-10T13:18Z
Measured: `ptn-9s5` rebased on `origin/master@1cbcfe170`; focused
`Zend/tests/offset_assign.phpt`, recursive `mkdir()`, dir predicates, and
gates.

## Test Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native compiled PHP snippets | 355 | 355 | 0 |
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
| PHPT callback manifest | 1 | 1 | 0 |

## COW PHPT Buckets

`tools/phpt-cow-manifest.txt` has 29 rows: 26 passing, 3 failing.
`bug35163.phpt` and `assign_by_val_function_by_ref_return_value.phpt` now pass.
Named `array_walk()` callbacks observe `$GLOBALS` swaps; closure-backed
callback rows remain blocked by Closure/callable values (`ptn-dis`).

## Already Ported

Lexer/parser, AST, IR, C backend, boxed values, variables, constants,
string/math/type internals, ordered arrays, `foreach`, cursors,
`array_values()`, numeric-string keys, payload refcounts, array/string COW,
references, by-reference parameters and `foreach`, array dimensions,
temporaries, recursive/user functions, magic constants, `func_*`, `print_r`,
binary strings, string offsets, scalar offset diagnostics, array literal
references, array union `+`, scalar type hints, by-reference return
alias/separation boundaries, `count()`, `??`, assignment expressions,
expression-level `@`, selected file APIs including recursive `mkdir()`,
`rmdir()`, `is_dir()`, `is_file()`, and `file_exists()`, array-path snapshots,
reference-aware `array_sum()`/`strtr()`/`in_array()`, recursive array merge and
replace, `debug_zval_dump()`, dynamic lvalue-reference calls, append/list
assignment expressions for reference arrays, nested same-array reference
lvalues with recursive `var_dump()`, direct-variable `??=`, keyed array/string
offset-form `??=`, append-form `??=` diagnostics, grouped reference targets,
named `array_reduce()` callback dispatch with by-reference returns,
non-reference call-result by-reference fallback notices, call-result
by-reference return chains, `array_fill_keys()`, string-callable
`call_user_func()` dispatch, variable-assigned recursive array literals with
slot replacement and cleanup, top-level `$GLOBALS[...]` callback writes, named
`array_walk()` callbacks that rebind the walked global array, and nested
string-offset lvalue fatals.

## Still Needed

Remaining COW PHPT gaps are Closure/callable `use` syntax for the
`array_walk()`/`$GLOBALS` row, closure-backed callback by-reference returns,
`array_reduce()` callback/refcount behavior, and broader recursive
by-reference return edges. Broader bounded-PHPT gaps are objects, unsupported
array/string internals, 64-bit operator exactness, foreach diagnostics,
object/property compound lvalues, remaining scalar offset-lvalue edges, and
broader file APIs beyond the current local filesystem subset.

## Verification

Commands: `cargo fmt --check`; `cargo test`; `tools/run-native-smoke-matrix.sh`;
`tools/run-post-merge-cow-gate.sh`; focused `Zend/tests/offset_assign.phpt`,
callback, recursive literal, `array_walk()`, `mkdir-003.phpt`.
