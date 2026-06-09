# PTN Progress

Last refresh: 2026-06-09T17:01Z
Commit: pending `ptn-cqu.47.1` branch head

## Test Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | --- |
| Source unit tests | 3 | 3 | 0 |
| Native compiled PHP snippets | 273 | 273 | 0 |
| PHPT parsed bounded log | 171 | 121 | 50 |
| PHPT Zend rows | 76 | 60 | 16 |
| PHPT ext/standard rows | 77 | 44 | 33 |
| PHPT tests/basic+func+lang | 18 | 17 | 1 |
| COW-focused tests | 12 | 6 | 6 categorized |

## Already Ported

Lexer/parser, AST, IR, C backend, boxed values, direct variables, constants,
selected string/math/type internals, length-aware string helper paths for
selected internals and scalar bitwise strings, ordered arrays and `foreach`,
numeric-string array key normalization coverage, top-level user functions with
scoped `__FUNCTION__`/`__METHOD__` magic constants and
`func_num_args()`/`func_get_arg()`/`func_get_args()` introspection, `print_r`,
selected binary-string handling, catchable `TypeError` for string offset reads,
and catchable `count()` non-array diagnostics.

## Still Needed

Copy-on-write gaps now have a dedicated PHP-oracle suite: arrays 2/2 pass,
strings 0/1, foreach 1/2, functions 2/3, nested values 1/2, references 0/2.
All non-COW work is paused unless it is required to prove COW.

## Next Focus

1. Implement shared payload refcounts and detach-on-write.
2. Prove strings, foreach snapshots, nested writes, functions, and references.
3. Keep the COW oracle counts current as cases flip from failing to passing.
