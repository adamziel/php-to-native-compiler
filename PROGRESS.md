# PTN Progress

Last refresh: 2026-06-09T17:52Z
Commit: pending `ptn-cqu.47.1` rebased after `ptn-cqu.47.11`

## Test Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | --- |
| Source unit tests | 3 | 3 | 0 |
| Native compiled PHP snippets | 287 | 287 | 0 |
| PHPT parsed bounded log | 171 | 121 | 50 |
| PHPT Zend rows | 76 | 60 | 16 |
| PHPT ext/standard rows | 77 | 44 | 33 |
| PHPT tests/basic+func+lang | 18 | 17 | 1 |
| COW contract spec tests | 5 | 5 | 0 |
| COW-focused native tests | 5 | 5 | 0 |
| COW oracle suite | 12 | 10 | references 2 compile blockers |
| PHPT COW manifest | 29 | 2 | 27 failed, 0 skipped, 0 warned |
| Focused PHPT foreach COW row | 1 | 1 | by-reference foreach remains unsupported |

## COW PHPT Buckets

`tools/phpt-cow-manifest.txt` at 2026-06-09T17:30Z: assignment-aliasing
0/4, string-offsets 2/4, array-writes-appends-unset 0/4, nested-arrays 0/4,
foreach-mutation 0/4, function-boundaries 0/4, reference-interaction 0/5.
Focused local foreach by-value PHPT row: 1/1. Dedicated oracle
`tests/cow_oracle.rs`: arrays 2/2, strings 1/1, foreach 2/2, functions 3/3,
nested values 2/2, references 0/2.

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
using quiet lookup semantics, and shared array/string payload COW with path
detach for nested array assignment, append, and unset.

## Still Needed

Broader copy-on-write coverage for strings, references, function boundaries,
by-reference foreach, and dynamic edges. Nested array path detach is present for
assignment, append, and unset. Oracle blockers are reference assignment syntax
and by-reference foreach compilation/runtime support. All non-COW work is
paused unless required to prove COW. Assignment-form null coalescing `??=`
remains unsupported.

## Next Focus

1. Add reference parsing/compiler/runtime support for COW blockers.
2. Carry array COW through by-reference and dynamic call paths.
3. Extend string, nested-value, and function-boundary oracle coverage.
4. Keep this dashboard numeric and under 500 words.
