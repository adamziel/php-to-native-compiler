# PTN Progress

Refresh: 2026-06-10T12:36Z
Measured: current branch rebased on `origin/master@1c445f86d`; focused
static-callable native coverage plus latest COW/callback evidence.

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
| Focused COW reducer snippets | 44 | 44 | 0 |
| Recursive reference diagnostics | 4 | 4 | 0 |
| COW oracle suite | 22 | 22 | 0 |
| By-reference foreach COW oracle | 11 | 11 | 0 |
| Mutating-internal COW matrix | 14 | 14 | 0 |
| Post-merge COW gate | 25 | 25 | 0 |
| PHPT COW manifest | 29 | 26 | 3 |
| PHPT callback manifest | 1 | 1 | 0 |

## COW PHPT Buckets

`tools/phpt-cow-manifest.txt` is 26/29: assignment-aliasing, string-offsets,
array-writes/appends/unset, nested-arrays, and reference-interaction pass;
foreach-mutation is 3/4 and function-boundaries is 2/4. Closure-backed callback
rows and `array_reduce()` refcount edges remain blocked.

## Already Ported

Lexer/parser, AST/IR/C backend, boxed values, variables/constants,
string/math/type internals, ordered arrays, `foreach`, cursors, numeric keys,
payload refcounts, array/string COW, references, by-reference params/foreach,
array dimensions, temporaries, recursive/user functions, magic constants,
`func_*`, `print_r`, binary strings, string offsets, scalar diagnostics, array
literal references, array union `+`, scalar type hints, by-reference return
boundaries, `count()`, `??`, assignment expressions, expression-level `@`, file
APIs, array-path RHS snapshots, `array_sum()`/`strtr()`/`in_array()`, recursive
array merge/replace, `debug_zval_dump()`, dynamic lvalue-reference calls,
append/list assignment expressions, nested same-array reference lvalues,
direct-variable and offset-form `??=`, grouped reference targets,
`array_fill_keys()`, string-callable `call_user_func()`, and public static
methods registered as `Class::method` callables for dynamic calls and internals.

## Still Needed

Remaining COW PHPT gaps are closure callback mutation through
`array_walk()`/`$GLOBALS`, closure-backed callback by-reference returns,
`array_reduce()` callback/refcount behavior, and broader recursive
by-reference return edges. Broader bounded-PHPT gaps are still objects,
unsupported array/string internals, 64-bit operator exactness, foreach
diagnostics, object/property compound lvalues, scalar offset-lvalue fatal
parity, and broader file APIs.

## Verification

Commands: `cargo fmt --check`; `cargo test`; `tools/run-native-smoke-matrix.sh`;
`tools/run-post-merge-cow-gate.sh`; focused callback PHPT evidence. Static
callback PHPT rows `array_map_object1`/`bug36011` are 0/2 because class-member
and non-static-method blockers are reached before callback checks.
