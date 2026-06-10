# PTN Progress

Refresh: 2026-06-10T14:20Z
Measured: `ptn-nvs` retry rebased on `origin/master@be9cc3570`; focused
`array_walk()` closure global-swap COW evidence plus current callback, static
callable, recursive directory API, and bounded `stdClass` property evidence.

## Test Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native compiled PHP snippets | 365 | 365 | 0 |
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
| PHPT COW manifest | 29 | 27 | 2 |
| PHPT callback manifest | 2 | 2 | 0 |

## COW PHPT Buckets

`tools/phpt-cow-manifest.txt` has 29 rows: 27 passing, 2 failing.
Buckets: assignment-aliasing 4/4, string-offsets 4/4,
array-writes-appends-unset 4/4, nested-arrays 4/4, foreach-mutation 4/4,
function-boundaries 2/4, reference-interaction 5/5. `array_walk()` supports
string/named callables and anonymous closures with `use` captures that swap
the walked global through `$GLOBALS`.

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
reference-aware internals, closure values/captures, `array_walk()` closure
callbacks with `$GLOBALS` swaps, recursive array merge/replace,
`debug_zval_dump()`, dynamic lvalue-reference calls, append/list assignment
expressions for reference arrays, nested same-array reference lvalues,
`??=`, grouped reference targets, named/static `array_reduce()` callbacks,
string-callable and null-callback `array_map()`, public static methods as
`Class::method` callables, `new stdClass` boxed objects with public dynamic
property reads/writes shared through aliases, non-reference call-result
by-reference fallback notices, `array_fill_keys()`, and string-callable
`call_user_func()`.

## Still Needed

Remaining COW PHPT gaps are closure-backed `array_reduce()` accumulator
refcount and by-reference return rows. Broader bounded-PHPT gaps are full
class declarations/metadata, instance methods, visibility/inheritance/static
properties/magic methods, non-static method callable values, unsupported
array/string internals, 64-bit operator exactness, foreach diagnostics,
object/property compound lvalues, scalar offset-lvalue fatal parity, and file
APIs beyond the current local filesystem subset.

## Verification

Commands: `cargo fmt --check`; `cargo test`; `tools/run-native-smoke-matrix.sh`;
`tools/run-post-merge-cow-gate.sh`; `tools/run-bounded-phpt.sh
tools/phpt-cow-manifest.txt`; callback manifest.
