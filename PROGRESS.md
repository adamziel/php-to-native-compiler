# PTN Progress

Refresh: 2026-06-10T13:20Z
Measured: `ptn-ept` on `origin/master@1cbcfe1`; focused
`array_reduce()` accumulator evidence plus current recursive `mkdir()`,
named `array_walk()`, recursive literal, and string-callable gates.

## Test Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native PHP snippets | 355 | 355 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 200 | 152 | 48 |
| PHPT Zend rows | 76 | 68 | 8 |
| PHPT ext/standard rows | 77 | 48 | 29 |
| PHPT tests/basic+func+lang | 45 | 34 | 11 |
| PHPT other rows | 2 | 2 | 0 |
| COW contract spec tests | 7 | 7 | 0 |
| Focused COW reducer snippets | 46 | 46 | 0 |
| Recursive reference diagnostics | 4 | 4 | 0 |
| COW oracle suite | 22 | 22 | 0 |
| By-reference foreach COW oracle | 11 | 11 | 0 |
| Mutating-internal COW matrix | 14 | 14 | 0 |
| Post-merge COW gate | 25 | 25 | 0 |
| PHPT COW manifest | 29 | 26 | 3 |
| PHPT callback manifest | 1 | 1 | 0 |

## COW PHPT Buckets

`tools/phpt-cow-manifest.txt` has 29 rows: 26 passing, 3 failing.
`bug35163.phpt` and `assign_by_val_function_by_ref_return_value.phpt` pass.
Named `array_walk()` callbacks observe `$GLOBALS` swaps, and named
`array_reduce()` callbacks preserve by-reference returns plus accumulator debug
refcounts. Exact closure-backed rows remain blocked by Closure/callable values
(`ptn-dis`).

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
lvalues, direct-variable and keyed offset-form `??=`, append-form `??=`
diagnostics, grouped reference targets, recursive array literal cleanup, named
`array_reduce()` callback dispatch with by-reference returns and accumulator
debug refcounts, non-reference call-result by-reference fallback notices,
call-result by-reference return chains, `array_fill_keys()`, string-callable
`call_user_func()` dispatch, top-level `$GLOBALS[...]` callback writes, and
named `array_walk()` callbacks that rebind the walked global array.

## Still Needed

Remaining COW PHPT gaps are Closure/callable `use` syntax for exact
`array_walk()`/`$GLOBALS` and `array_reduce()` rows, closure-backed callback
by-reference returns, and broader recursive by-reference return edges. Broader
bounded-PHPT gaps are objects, unsupported array/string internals, 64-bit
operator exactness, foreach diagnostics, object/property compound lvalues,
scalar offset-lvalue fatal parity, and file APIs beyond the current subset.

## Verification

Commands: `cargo fmt --check`; `cargo test`; `tools/run-native-smoke-matrix.sh`;
`tools/run-post-merge-cow-gate.sh`; `tools/run-bounded-phpt.sh
tools/phpt-cow-manifest.txt`; callback manifest.
