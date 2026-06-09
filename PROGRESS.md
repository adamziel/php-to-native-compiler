# PTN Progress

Last refresh: 2026-06-09T18:48Z
Measured base: `ptn-cqu.47.16` rebased after `ptn-cqu.47.19`

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
| COW oracle suite | 22 | 21 | 1 |
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
Native COW reducers: 26/26, including dynamic temporary, call-result, and
read-slot cases 10/10 against PHP. Oracle coverage: arrays 2/2, strings 1/1,
foreach 2/2, functions 3/3, nested values 2/2, references 1/2,
array element references 10/10.
Mutating-internal matrix: 12/12 plus five unsupported target diagnostics.
Contract stress balances 12 nested cycles with 0 live payloads.

## Already Ported

Lexer/parser, AST, IR, C backend, boxed values, variables, constants,
string/math/type internals, ordered arrays, `foreach`, array cursors,
`array_values()`, numeric-string keys, payload refcounts, by-value snapshots,
generated C ABI share/drop handling, shared array/string COW, nested path
detach, reference aliases, element references, by-reference parameters,
by-value parameter and function-boundary splits, recursive/user functions,
magic constants, `func_*`, `print_r`, binary strings, string offset reads and
writes, `count()` diagnostics, `??`, array assignment/compound/append/unset
COW, focused native/oracle COW reducers, array-element reference oracle
coverage, unsupported mutating-internal diagnostics, and payload lifetime
debug counters.

## Still Needed

Broader COW for references, by-reference `foreach`, more PHPT COW rows,
by-reference returns, nested reference lvalues, recursive reference
diagnostics, dynamic calls, and assignment-form `??=`.

## Next Focus

1. Prove references and remaining function boundaries.
2. Carry COW through by-reference and dynamic call paths.
3. Keep dashboard cells numeric and status files under 500 words.
