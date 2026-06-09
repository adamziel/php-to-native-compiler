# PTN Progress

Last refresh: 2026-06-09T17:14Z
Commit: pending `ptn-cqu.47.2` branch head

## Test Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | --- |
| Source unit tests | 3 | 3 | 0 |
| Native compiled PHP snippets | 273 | 273 | 0 |
| PHPT parsed bounded log | 171 | 121 | 50 |
| PHPT Zend rows | 76 | 60 | 16 |
| PHPT ext/standard rows | 77 | 44 | 33 |
| PHPT tests/basic+func+lang | 18 | 17 | 1 |
| COW contract spec tests | 5 | 5 | Runtime COW implementation still needed |

## Already Ported

Lexer/parser, AST, IR, C backend, boxed values, direct variables, constants,
selected string/math/type internals, length-aware string helper paths for
selected internals and scalar bitwise strings, ordered arrays and `foreach`,
array cursor internals on direct variable ordered arrays, `array_values()` over
ordered arrays, numeric-string array key normalization coverage, top-level user
functions with scoped `__FUNCTION__`/`__METHOD__` magic constants and
`func_num_args()`/`func_get_arg()`/`func_get_args()` introspection, `print_r`,
selected binary-string handling, string offset read diagnostics,
direct-variable string offset writes with append/unset/assign-op Error
boundaries, and catchable `count()` non-array diagnostics.

## Still Needed

Runtime copy-on-write for arrays, strings, variables, function calls, foreach,
nested containers, and references. A design-only COW payload contract and
executable spec tests now pin the required ownership transitions.

## Next Focus

1. Build a dedicated COW correctness suite.
2. Implement shared payload refcounts and detach-on-write.
3. Prove arrays, strings, nested values, foreach, and function boundaries.
4. Keep this dashboard numeric and under 500 words.
