# PTN Progress

Last refresh: 2026-06-09T19:58Z
Measured base: `ptn-cqu.47.14` rebased after `ptn-cqu.47.11`

## Test Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native compiled PHP snippets | 289 | 289 | 0 |
| PHPT parsed bounded log | 171 | 121 | 50 |
| PHPT Zend rows | 76 | 60 | 16 |
| PHPT ext/standard rows | 77 | 44 | 33 |
| PHPT tests/basic+func+lang | 18 | 17 | 1 |
| COW contract spec tests | 5 | 5 | 0 |
| COW-focused native tests | 7 | 7 | 0 |
| PHPT COW manifest | 29 | 2 | 27 |
| Focused PHPT foreach COW row | 1 | 1 | 0 |

## COW PHPT Buckets

`tools/phpt-cow-manifest.txt` at 2026-06-09T17:30Z:
assignment-aliasing 0/4, string-offsets 2/4,
array-writes-appends-unset 0/4, nested-arrays 0/4,
foreach-mutation 0/4, function-boundaries 0/4,
reference-interaction 0/5. Focused local foreach by-value PHPT row: 1/1.

## Already Ported

Lexer/parser, AST, IR, C backend, boxed values, direct variables, constants,
selected string/math/type internals, ordered arrays and `foreach`, array cursor
internals, `array_values()`, numeric-string array key normalization, array and
string payload COW, by-value `foreach` COW snapshots, generated C ABI
share/drop handling, mutating-internal COW matrix coverage for array
pop/push/shift, cursor mutators, and string offset writes (13 pass, 0 fail,
3 unsupported diagnostics), temporary/nested array mutator diagnostics,
top-level user functions with scoped magic constants and `func_*`
introspection, `print_r`, selected binary-string handling, string offset read
diagnostics, direct-variable string offset writes, catchable `count()`
non-array diagnostics, expression-form `??` reads, and nested array path detach
for assignment, append, and unset.

## Still Needed

Broader COW coverage for strings, references, function boundaries,
by-reference foreach, and dynamic edges. All non-COW work is paused unless it
directly proves COW correctness or COW evidence. Assignment-form `??=` remains
unsupported.

## Next Focus

1. Prove strings, references, and function boundaries.
2. Carry COW through by-reference and dynamic call paths.
3. Keep dashboard cells numeric and every status file under 500 words.
