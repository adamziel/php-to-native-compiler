# PTN Progress

Refresh: 2026-06-10T13:58Z
Measured: `ptn-dis` rebased on `origin/master@be9cc3570`; focused closure,
static callable, object/property, `array_map()`, directory API, and COW
evidence.

## Test Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 4 | 4 | 0 |
| Native compiled PHP snippets | 366 | 366 | 0 |
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
| PHPT callback manifest | 2 | 2 | 0 |

## COW PHPT Buckets

`tools/phpt-cow-manifest.txt` has 29 rows: 26 passing, 3 failing.
`bug35163.phpt` and `assign_by_val_function_by_ref_return_value.phpt` pass.
Captureless closure callables cover the exact `array_reduce()` callback
return-by-reference row. Named `array_walk()` callbacks observe `$GLOBALS`
swaps; Closure/callable `use` rows remain blocked by full Closure captures.

## Already Ported

Lexer/parser, AST/IR/C backend, boxed values, variables/constants,
string/math/type internals, ordered arrays, `foreach`, cursors, numeric keys,
payload refcounts, array/string COW, references, by-reference params/foreach,
array dimensions, temporaries, recursive/user functions, captureless anonymous
functions, magic constants, `func_*`, `print_r`, binary strings, string
offsets, scalar diagnostics, array literal references, array union `+`, scalar
type hints, by-reference return boundaries, `count()`, `??`, assignment
expressions, expression-level `@`, recursive directory APIs, array-path
snapshots, `array_sum()`/`strtr()`/`in_array()`, recursive array merge/replace,
`debug_zval_dump()`, dynamic lvalue-reference calls, append/list assignment,
nested same-array reference lvalues, direct-variable and offset-form `??=`,
grouped reference targets, `array_fill_keys()`, string-callable
`call_user_func()`, string-callable/null `array_map()`, named and
captureless-closure `array_reduce()` callbacks with by-reference returns, named
`array_walk()` global-array rebinding, public static methods registered as
`Class::method` callables, and `new stdClass` boxed objects with public dynamic
property reads/writes shared through object aliases.

## Still Needed

Remaining COW PHPT gaps are Closure/callable `use` syntax, full Closure
capture/object callback semantics, `array_reduce()` accumulator refcount
behavior, and broader recursive by-reference return edges. Broader bounded-PHPT
gaps are full class declarations/metadata, instance methods,
visibility/inheritance/static properties/magic methods, non-static method
callable values, unsupported array/string internals, 64-bit operator exactness,
foreach diagnostics, object/property compound lvalues, scalar offset-lvalue
fatal parity, and broader file APIs.

## Verification

Commands: `cargo fmt --check`; `cargo build --bin phpc`; focused
callback/object native tests; COW reducers; focused PHPT rows;
`tools/run-native-smoke-matrix.sh`; `tools/run-post-merge-cow-gate.sh`.
