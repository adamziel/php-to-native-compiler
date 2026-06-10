# PTN Progress

Refresh: 2026-06-10T16:21Z
Measured: `ptn-886` rebased on `origin/master@8a659986`; binary bitwise
`&`, `|`, and `^` now route integer-conversion diagnostics through the modeled
`error_reporting()` mask, and scalar float string conversion uses PHP-style
uppercase exponent spelling.

## Test Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native compiled PHP snippets | 370 | 370 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 200 | 156 | 44 |
| PHPT Zend rows | 76 | 68 | 8 |
| PHPT ext/standard rows | 77 | 49 | 28 |
| PHPT tests/basic+func+lang | 45 | 37 | 8 |
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

`tools/phpt-cow-manifest.txt` has 29 rows: 28 passing, 1 failing. The remaining
documented COW PHPT failure is `bug69068_2.phpt`.

## Already Ported

Lexer/parser, AST/IR/C backend, boxed values, variables/constants,
string/math/type internals, ordered arrays, `foreach`, source-spanned
non-array `foreach` warnings, cursors, numeric keys, payload refcounts,
array/string COW, references, by-reference params/foreach, array dimensions,
temporaries, recursive/user functions, direct dynamic calls and internal
callbacks, magic constants, `func_*`, `print_r`, binary strings, string
offsets, scalar diagnostics, array literal references, array union `+`,
scalar type hints, by-reference return boundaries, `count()`, `??`,
assignment expressions, expression-level `@`, recursive `mkdir()` plus
directory predicates, array-path snapshots, `array_sum()`/`strtr()`/
`in_array()`, recursive array merge/replace, `debug_zval_dump()`,
dynamic lvalue-reference calls, append/list assignment expressions, nested
same-array reference lvalues, `??=`, grouped reference targets,
`array_fill_keys()`, string-callable `call_user_func()`, string-callable/null
`array_map()`, `intval()`, named `array_walk()` global-array rebinding, public
static `Class::method` callables, `array_reduce()` callback dispatch,
`stdClass` public-property storage, `array_count_values()`, and 64-bit binary
bitwise PHPT rows under `error_reporting(E_ERROR)`.

## Still Needed

Focused COW gap: Closure `use` captures for `array_walk()`/`$GLOBALS`.
Broader bounded-PHPT gaps include full class metadata, instance methods,
visibility/inheritance/static properties/magic methods, non-static method
callables, resources, unsupported array/string internals, object/destructuring
foreach diagnostics, object/property compound lvalues, scalar offset-lvalue
fatal parity, broader file APIs, and complete error-handler routing.

## Verification

Commands: `cargo fmt --check`; `cargo check`; `cargo build --bin phpc`;
focused binary bitwise native tests; exact PHPT rows
`bitwiseAnd_basiclong_64bit.phpt`, `bitwiseOr_basiclong_64bit.phpt`, and
`bitwiseXor_basiclong_64bit.phpt`.
