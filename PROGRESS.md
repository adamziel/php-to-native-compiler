# PTN Progress

Refresh: 2026-06-10T13:31Z
Measured: `ptn-zo6` rebased on `origin/master@3c11d09`; recursive `mkdir()`,
string-callable `array_map()`, and static callable evidence.

## Test Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native compiled PHP snippets | 360 | 360 | 0 |
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
Named `array_walk()` callbacks observe `$GLOBALS` swaps. Closure-backed
callback rows remain blocked by Closure/callable values (`ptn-dis`).

## Already Ported

Lexer/parser, AST/IR/C backend, boxed values, variables/constants,
string/math/type internals, ordered arrays, `foreach`, cursors, numeric keys,
payload refcounts, array/string COW, references, by-reference params/foreach,
array dimensions, temporaries, recursive/user functions, magic constants,
`func_*`, `print_r`, binary strings, string offsets, scalar diagnostics, array
literal references, array union `+`, scalar type hints, by-reference return
boundaries, `count()`, `??`, assignment expressions, expression-level `@`, file
APIs including recursive `mkdir()` plus directory predicates, array-path
snapshots, `array_sum()`/`strtr()`/`in_array()`, recursive array merge/replace,
`debug_zval_dump()`, dynamic lvalue-reference calls, append/list assignment
expressions, nested same-array reference lvalues, direct-variable and
offset-form `??=`, grouped reference targets, `array_fill_keys()`,
string-callable `call_user_func()`, string-callable/null `array_map()`, named
`array_walk()` global-array rebinding, and public static methods registered as
`Class::method` callables for dynamic calls and internals.

## Still Needed

Remaining COW PHPT gaps are Closure/callable `use` syntax, closure-backed
callback by-reference returns, `array_reduce()` refcount behavior, and broader
recursive by-reference return edges. Broader bounded-PHPT gaps are objects,
non-static method callable values, unsupported array/string internals, 64-bit
operator exactness, foreach diagnostics, object/property compound lvalues,
scalar offset-lvalue fatal parity, and broader file APIs.

## Verification

Commands: `cargo fmt --check`; focused static/string callable and `array_walk()`
native tests; `cargo test`; `cargo build --bin phpc`; focused callback PHPT
rows; `tools/run-native-smoke-matrix.sh`; `tools/run-post-merge-cow-gate.sh`.
