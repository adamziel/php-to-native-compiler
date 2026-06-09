# PTN Progress

Last refresh: 2026-06-09T19:04Z
Commit: pending `ptn-cqu.47.12` rebased after `ptn-cqu.47.11`

## Test Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | --- |
| Source unit tests | 3 | 3 | 0 |
| Native compiled PHP snippets | 303 | 303 | 0 |
| PHPT parsed bounded log | 171 | 121 | 50 |
| PHPT Zend rows | 76 | 60 | 16 |
| PHPT ext/standard rows | 77 | 44 | 33 |
| PHPT tests/basic+func+lang | 18 | 17 | 1 |
| COW contract spec tests | 5 | 5 | 0 |
| COW-focused native tests | 6 | 6 | 0 |
| Focused COW reducer snippets | 16 | 16 | 0 |
| PHPT COW manifest | 29 | 2 | 27 failed, 0 skipped, 0 warned |
| Focused PHPT foreach COW row | 1 | 1 | by-reference foreach remains unsupported |

## COW PHPT Buckets

`tools/phpt-cow-manifest.txt` at 2026-06-09T17:30Z: assignment-aliasing
0/4, string-offsets 2/4, array-writes-appends-unset 0/4, nested-arrays 0/4,
foreach-mutation 0/4, function-boundaries 0/4, reference-interaction 0/5.
Focused local foreach by-value PHPT row: 1/1. Native COW reducer matrix: 16
pass, 0 fail.

## Already Ported

Lexer/parser, AST, IR, C backend, boxed values, direct variables, constants,
selected string/math/type internals, length-aware string helper paths for
selected internals and scalar bitwise strings, ordered arrays and `foreach`,
array cursor internals on direct variable ordered arrays, `array_values()` over
ordered arrays, numeric-string array key normalization coverage, array payload
refcounts with detach-on-write for named mutations and cursor-mutating internals,
by-value `foreach` COW snapshots for append/unset and alias mutation visibility,
generated C ABI share/drop handling for returns, temporaries, and call argument
slots, top-level user functions with scoped `__FUNCTION__`/`__METHOD__` magic
constants plus `func_num_args()`/`func_get_arg()`/`func_get_args()`
introspection, `print_r`, selected binary-string handling, string offset read
diagnostics, direct-variable string offset writes with append/unset/assign-op
Error boundaries, catchable `count()` non-array diagnostics, and
expression-form `??` reads over direct variables, arrays, and string offsets
using quiet lookup semantics, shared array/string payload COW with path detach
for nested array assignment, append, and unset, and focused native COW reducers
for assignment aliasing, array writes, nested copies, foreach by-value mutation,
function boundaries, cursor helpers, `array_shift`, string offsets, and string
compound assignment.

## Still Needed

Broader COW coverage for strings, references, function boundary edges,
by-reference foreach, string/nested dynamic paths, and dynamic edges. Nested
array path detach is present for assignment, append, and unset. All non-COW work
is paused unless it is required to prove COW. Assignment-form null coalescing
`??=` remains unsupported.

## Next Focus

1. Extend the COW suite into strings, references, and function boundaries.
2. Carry array COW through additional by-reference/dynamic call paths.
3. Prove by-reference foreach paths remain blocked or become implemented.
4. Keep this dashboard numeric and under 500 words.
