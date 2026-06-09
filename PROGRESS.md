# PTN Progress

Last refresh: 2026-06-09T18:32Z
Measured base: `ptn-cqu.47.10` rebased after `ptn-cqu.47.3`

## Test Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 4 | 4 | 0 |
| Native compiled PHP snippets | 309 | 309 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT parsed bounded log | 171 | 121 | 50 |
| PHPT Zend rows | 76 | 60 | 16 |
| PHPT ext/standard rows | 77 | 44 | 33 |
| PHPT tests/basic+func+lang | 18 | 17 | 1 |
| COW contract spec tests | 5 | 5 | 0 |
| COW-focused native tests | 12 | 12 | 0 |
| Focused COW reducer snippets | 16 | 16 | 0 |
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
Native COW reducer matrix: 16 pass, 0 fail. Dedicated oracle
`tests/cow_oracle.rs`: arrays 2/2, strings 1/1, foreach 2/2, functions 3/3,
nested values 2/2, references 0/2. Mutating-internal matrix: 12 pass,
0 fail plus five unsupported target diagnostics.

## Already Ported

Lexer/parser, AST, IR, C backend, boxed values, direct variables, constants,
selected string/math/type internals, ordered arrays and `foreach`, array cursor
internals plus `array_pop()`/`array_push()`/`array_shift()` on direct variable
arrays, `array_values()`, numeric-string array key normalization, array payload
refcounts with detach-on-write, by-value `foreach` COW snapshots, generated C
ABI share/drop handling, shared array/string payload COW with nested path
detach, direct variable reference aliases, array element references,
by-reference user parameters, by-value parameter/reference split behavior,
by-value function-boundary COW across arguments, locals, returns, recursion,
and extra arguments, top-level user functions with scoped magic constants and
`func_*` introspection, `print_r`, selected binary-string handling, string
offset read diagnostics, direct-variable string offset writes, refcounted
string payloads with assignment/function sharing and detach-on-write offset
mutation, catchable `count()` non-array diagnostics, expression-form `??`
reads, nested array path detach for assignment, compound assignment, append,
and unset, focused native COW reducers, COW oracle coverage, and five explicit
unsupported diagnostics for non-variable mutating internals, plus COW debug
counters/assertions for string and array payload alloc/free/share/drop/detach
evidence.

## Still Needed

Broader COW for references, by-reference foreach, broader PHPT COW rows, and
dynamic edges such as by-reference returns, nested reference lvalues, recursive
reference diagnostics, and dynamic calls. Assignment-form `??=` remains
unsupported.

## Next Focus

1. Prove references and remaining function boundaries.
2. Carry COW through by-reference and dynamic call paths.
3. Keep dashboard cells numeric and every status file under 500 words.
