# PTN Progress

Last refresh: 2026-06-09T17:12Z
Commit: pending `ptn-cqu.47.4` branch head

## Test Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | --- |
| Source unit tests | 3 | 3 | 0 |
| Native compiled PHP snippets | 275 | 275 | 0 |
| PHPT parsed bounded log | 171 | 121 | 50 |
| PHPT Zend rows | 76 | 60 | 16 |
| PHPT ext/standard rows | 77 | 44 | 33 |
| PHPT tests/basic+func+lang | 18 | 17 | 1 |
| COW-focused tests | 2 | 2 | strings, nested write targets, references |

## Already Ported

Lexer/parser, AST, IR, C backend, boxed values, direct variables, constants,
selected string/math/type internals, length-aware string helper paths for
selected internals and scalar bitwise strings, ordered arrays and `foreach`,
array cursor internals on direct variable ordered arrays, `array_values()` over
ordered arrays, numeric-string array key normalization coverage, array payload
refcounts with detach-on-write for named mutations and cursor-mutating internals,
top-level user functions with scoped `__FUNCTION__`/`__METHOD__` magic constants
plus
`func_num_args()`/`func_get_arg()`/`func_get_args()` introspection, `print_r`,
selected binary-string handling, string offset read diagnostics,
direct-variable string offset writes with append/unset/assign-op Error
boundaries, and catchable `count()` non-array diagnostics.

## Still Needed

Copy-on-write for strings, nested write targets, references, and broader dynamic
edges. All non-COW work is paused unless it is required to prove COW.

## Next Focus

1. Extend the COW suite into strings, nested write targets, and references.
2. Carry array COW through additional by-reference/dynamic call paths.
3. Prove strings, nested values, foreach, and function boundaries.
4. Keep this dashboard numeric and under 500 words.
