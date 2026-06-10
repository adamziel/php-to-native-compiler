# PTN Progress

Refresh: 2026-06-10T16:35Z
Measured: `ptn-q5r` rebased on `origin/master@35eb930d`; focused `print_r`
PHPT rows now pass after bounded integer `range()` support.

## Test Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native compiled PHP snippets | 369 | 369 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 200 | 154 | 46 |
| PHPT Zend rows | 76 | 68 | 8 |
| PHPT ext/standard rows | 77 | 50 | 27 |
| PHPT tests/basic+func+lang | 45 | 34 | 11 |
| PHPT other rows | 2 | 2 | 0 |
| COW contract spec tests | 7 | 7 | 0 |
| Focused COW reducer snippets | 46 | 46 | 0 |
| Recursive reference diagnostics | 4 | 4 | 0 |
| COW oracle suite | 22 | 22 | 0 |
| By-reference foreach COW oracle | 11 | 11 | 0 |
| Mutating-internal COW matrix | 14 | 14 | 0 |
| Post-merge COW gate | 25 | 25 | 0 |
| PHPT COW manifest | 29 | 28 | 1 |
| PHPT callback manifest | 2 | 2 | 0 |

## COW PHPT Buckets

`tools/phpt-cow-manifest.txt` has 29 rows: 28 passing, 1 failing.
The `array_reduce_accumulator_refcount.phpt` row now passes; the documented
remaining COW PHPT failure is `bug69068_2.phpt`.

## Already Ported

Lexer/parser, AST/IR/C backend, boxed values, variables/constants,
string/math/type internals, ordered arrays, `foreach`, source-spanned
non-array `foreach` warnings, cursors, numeric keys, payload refcounts,
array/string COW, references, by-reference params/foreach, array dimensions,
temporaries, recursive/user functions, anonymous function values for direct
dynamic calls and internal callbacks, magic constants, `func_*`, `print_r`,
binary strings, string offsets, scalar diagnostics, array literal references,
array union `+`, scalar type hints, by-reference return boundaries, `count()`,
`??`, assignment expressions, expression-level `@`, file APIs, array-path
snapshots, `array_sum()`/`strtr()`/`in_array()`, recursive array
merge/replace, `debug_zval_dump()`, dynamic lvalue-reference calls,
append/list assignment expressions, nested same-array reference lvalues,
direct-variable and offset-form `??=`, grouped reference targets,
`array_fill_keys()`, string-callable `call_user_func()`,
string-callable/null `array_map()`, `intval()` base-prefix conversion,
`array_count_values()` over integer/string values, bounded integer `range()`,
named `array_walk()` global-array rebinding, public static `Class::method`
callables, `array_reduce()` callback dispatch with debug refcounts, and
`stdClass` objects with public dynamic property reads/writes.

## Still Needed

Remaining gaps include Closure `use` captures for the focused COW PHPT row,
full class declarations/metadata, instance methods, visibility/inheritance,
static properties/magic methods, non-static method callables, unsupported
array/string internals, 64-bit operator exactness, object/destructuring
foreach diagnostics, object/property compound lvalues, scalar offset-lvalue
fatal parity, float/string `range()` forms, and broader file APIs.

## Verification

Commands: `cargo fmt`; `cargo build --bin phpc`; focused
`compile_print_r_current_boxed_values` native test; focused
`print_r_null`, `print_r_bools`, `print_r_strings`, `print_r_ints`, and
`print_r_arrays` PHPT rows.
