# PTN Progress

Last refresh: 2026-06-09T18:02Z
Measured base: `ptn-cqu.47.1` rebased after `ptn-cqu.47.12`

## Test Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native compiled PHP snippets | 303 | 303 | 0 |
| PHPT parsed bounded log | 171 | 121 | 50 |
| PHPT Zend rows | 76 | 60 | 16 |
| PHPT ext/standard rows | 77 | 44 | 33 |
| PHPT tests/basic+func+lang | 18 | 17 | 1 |
| COW contract spec tests | 5 | 5 | 0 |
| COW-focused native tests | 6 | 6 | 0 |
| Focused COW reducer snippets | 16 | 16 | 0 |
| COW oracle suite | 12 | 10 | 2 |
| PHPT COW manifest | 29 | 2 | 27 |
| Focused PHPT foreach COW row | 1 | 1 | 0 |

## COW PHPT Buckets

`tools/phpt-cow-manifest.txt` at 2026-06-09T17:30Z:
assignment-aliasing 0/4, string-offsets 2/4,
array-writes-appends-unset 0/4, nested-arrays 0/4,
foreach-mutation 0/4, function-boundaries 0/4,
reference-interaction 0/5. Focused local foreach by-value PHPT row: 1/1.
Native COW reducer matrix: 16 pass, 0 fail. Dedicated oracle
`tests/cow_oracle.rs`: arrays 2/2, strings 1/1, foreach 2/2, functions 3/3,
nested values 2/2, references 0/2.

## Already Ported

Lexer/parser, AST, IR, C backend, boxed values, direct variables, constants,
selected string/math/type internals, ordered arrays and `foreach`, array cursor
internals, `array_values()`, numeric-string array key normalization, array
payload refcounts with detach-on-write, by-value `foreach` COW snapshots,
generated C ABI share/drop handling, top-level user functions with scoped magic
constants and `func_*` introspection, `print_r`, selected binary-string
handling, string offset read diagnostics, direct-variable string offset writes,
catchable `count()` non-array diagnostics, expression-form `??` reads, nested
array path detach for assignment, append, and unset, focused native COW
reducers, and COW oracle coverage.

## Still Needed

Broader COW coverage for strings, references, function boundaries,
by-reference foreach, and dynamic edges. Oracle blockers are reference
assignment syntax and by-reference foreach support. All non-COW work is paused
unless it directly proves COW correctness or COW evidence. Assignment-form
`??=` remains unsupported.

## Next Focus

1. Add reference parsing/compiler/runtime support for COW blockers.
2. Carry array COW through by-reference and dynamic call paths.
3. Extend string, nested-value, and function-boundary oracle coverage.
4. Keep dashboard cells numeric and every status file under 500 words.
