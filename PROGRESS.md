# PTN Progress

Refresh: 2026-06-10T14:13Z
Measured: `ptn-d0w` rebased on `origin/master@51e3314`; preserves anonymous
callback closures, static callables, `stdClass` properties, and improves
bounded 64-bit unary bitwise conversion evidence.

## Test Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native compiled PHP snippets | 368 | 368 | 0 |
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
Anonymous callbacks now cover the closure callback row. Remaining documented
failures are `bug69068_2.phpt` and `array_reduce_accumulator_refcount.phpt`.

## Already Ported

Lexer/parser, AST/IR/C backend, boxed values, variables/constants,
string/math/type internals, ordered arrays, `foreach`, cursors, numeric keys,
payload refcounts, array/string COW, references, by-reference params/foreach,
array dimensions, temporaries, recursive/user functions, anonymous function
values for direct dynamic calls and internal callbacks, magic constants,
`func_*`, `print_r`, binary strings, string offsets, scalar diagnostics, array
literal references, array union `+`, scalar type hints, by-reference return
boundaries, `count()`, `??`, assignment expressions, expression-level `@`, file
APIs including recursive `mkdir()` plus directory predicates, array-path
snapshots, `array_sum()`/`strtr()`/`in_array()`, recursive array merge/replace,
`debug_zval_dump()`, dynamic lvalue-reference calls, append/list assignment
expressions, nested same-array reference lvalues, direct-variable and
offset-form `??=`, grouped reference targets, `array_fill_keys()`,
string-callable `call_user_func()`, string-callable/null `array_map()`, named
`array_walk()` global-array rebinding, public static methods registered as
`Class::method` callables, `new stdClass` dynamic properties, and integer-only
operator conversions that distinguish non-representable float warnings from
in-range precision-loss deprecations with PHP-style uppercase float exponents.

## Still Needed

Remaining COW PHPT gaps are Closure `use` captures and `array_reduce()`
accumulator/refcount behavior. Broader bounded-PHPT gaps are full class
declarations/metadata, instance methods, visibility/inheritance/static
properties/magic methods, non-static method callable values, unsupported
array/string internals, remaining 64-bit binary bitwise diagnostic suppression,
foreach diagnostics, object/property compound lvalues, scalar offset-lvalue
fatal parity, and broader file APIs.

## Verification

Commands: `cargo fmt --check`; focused native 64-bit bitwise conversion and
integer precision tests; `cargo test`; `cargo build --bin phpc`; focused PHPT
`bitwiseNot_basiclong_64bit.phpt`; native smoke matrix; post-merge COW gate.
