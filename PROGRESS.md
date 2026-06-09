# PTN Progress

Last refresh: 2026-06-09T17:34Z
Commit: pending `ptn-cqu.47.1` COW oracle suite

## Test Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | --- |
| Source unit tests | 3 | 3 | 0 |
| Native compiled PHP snippets | 284 | 284 | 0 |
| PHPT parsed bounded log | 171 | 121 | 50 |
| PHPT Zend rows | 76 | 60 | 16 |
| PHPT ext/standard rows | 77 | 44 | 33 |
| PHPT tests/basic+func+lang | 18 | 17 | 1 |
| COW contract spec tests | 5 | 5 | Runtime COW implementation still needed |
| COW-focused tests | 2 | 2 | strings, nested write targets, references |
| COW oracle suite | 13 | 9 | arrays 2/2, strings 2/2, foreach 1/2, functions 3/3, nested values 1/2, references 0/2 |

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
boundaries, catchable `count()` non-array diagnostics, and expression-form `??`
reads over direct variables, arrays, and string offsets using quiet lookup
semantics.

## Still Needed

Copy-on-write for nested write targets, references, foreach source mutation
snapshots, and broader dynamic edges. COW oracle gaps are categorized as one
foreach runtime mismatch, one nested direct-write compile blocker, and two
reference compile blockers. Assignment-form null coalescing `??=` remains
unsupported.

## Next Focus

1. Fix foreach source mutation snapshots against the COW oracle.
2. Carry array COW through additional by-reference/dynamic call paths.
3. Add nested direct write targets and reference-aware COW semantics.
4. Keep this dashboard numeric and under 500 words.
