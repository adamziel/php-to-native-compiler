# PTN Progress

Last refresh: 2026-06-09T18:43Z
Measured base: `ptn-cqu.47.19` rebased after `ptn-cqu.47.18`

## Test Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 4 | 4 | 0 |
| Native compiled PHP snippets | 309 | 309 | 0 |
| PHPT parsed bounded log | 171 | 121 | 50 |
| PHPT Zend rows | 76 | 60 | 16 |
| PHPT ext/standard rows | 77 | 44 | 33 |
| PHPT tests/basic+func+lang | 18 | 17 | 1 |
| COW contract spec tests | 7 | 7 | 0 |
| COW-focused native tests | 12 | 12 | 0 |
| Focused COW reducer snippets | 26 | 26 | 0 |
| COW oracle suite | 12 | 10 | 2 |
| COW/reference-focused native tests | 10 | 10 | 0 |
| Mutating-internal COW matrix | 12 | 12 | 0 |
| PHPT COW manifest | 29 | 2 | 27 |
| Focused PHPT foreach COW row | 1 | 1 | 0 |

## COW PHPT Buckets

`tools/phpt-cow-manifest.txt` at 2026-06-09T17:30Z:
assignment-aliasing 0/4, string-offsets 2/4,
array-writes-appends-unset 0/4, nested-arrays 0/4,
foreach-mutation 0/4, function-boundaries 0/4,
reference-interaction 0/5. Focused local foreach by-value PHPT row: 1/1.
Native COW reducer matrix: 26 pass, 0 fail, including dynamic
temporary/call-result/read-slot cases 10/10 against PHP oracle. Dedicated oracle
`tests/cow_oracle.rs`: arrays 2/2, strings 1/1, foreach 2/2, functions 3/3,
nested values 2/2, references 0/2. Mutating-internal matrix: 12 pass,
0 fail plus five unsupported target diagnostics. Contract stress includes
12 nested drop cycles with 48 array detaches, 12 string detaches, 108 frees,
and 0 live payloads.

## Already Ported

Lexer/parser, AST, IR, C backend, boxed values, variables, constants,
string/math/type internals, ordered arrays and `foreach`, array cursor
internals, `array_values()`, numeric-string keys, array payload refcounts,
by-value `foreach` COW snapshots, generated C ABI share/drop handling,
shared array/string payload COW with nested path detach, reference aliases,
array element references, by-reference user parameters, by-value parameter
splits, function-boundary COW across arguments, locals, returns, recursion,
and extras, user functions with scoped magic constants and `func_*`
introspection, `print_r`, binary-string handling, string offset diagnostics
and writes, refcounted string payload detach-on-write, catchable `count()`
non-array diagnostics, expression-form `??`, nested array path detach for
assignment, compound assignment, append, and unset, focused native COW
reducers, dynamic temporary/read-slot COW oracle reducers, five unsupported
mutating-internal diagnostics, and payload lifetime debug counters.

## Still Needed

Broader COW for references, by-reference foreach, broader PHPT COW rows, and
dynamic edges such as by-reference returns, nested reference lvalues, recursive
reference diagnostics, and dynamic calls. Assignment-form `??=` remains
unsupported.

## Next Focus

1. Prove references and remaining function boundaries.
2. Carry COW through by-reference and dynamic call paths.
3. Keep dashboard cells numeric and every status file under 500 words.
