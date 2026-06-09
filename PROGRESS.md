# PTN Progress

Last refresh: 2026-06-09T18:59Z
Measured base: `ptn-cqu.47.17` rebased after `ptn-0xy`

## Test Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 4 | 4 | 0 |
| Native compiled PHP snippets | 310 | 310 | 0 |
| PHPT bounded manifest | 200 | 145 | 55 |
| PHPT Zend rows | 76 | 63 | 13 |
| PHPT ext/standard rows | 77 | 47 | 30 |
| PHPT tests/basic+func+lang | 45 | 33 | 12 |
| PHPT other rows | 2 | 2 | 0 |
| COW contract spec tests | 7 | 7 | 0 |
| COW-focused native tests | 12 | 12 | 0 |
| Focused COW reducer snippets | 26 | 26 | 0 |
| COW oracle suite | 22 | 22 | 0 |
| By-reference foreach COW oracle | 9 | 9 | 0 |
| COW/reference-focused native tests | 10 | 10 | 0 |
| Mutating-internal COW matrix | 12 | 12 | 0 |
| PHPT COW manifest | 29 | 7 | 22 |
| Focused PHPT foreach COW row | 1 | 1 | 0 |

## COW PHPT Buckets

`tools/phpt-cow-manifest.txt` at 2026-06-09T18:30Z:
assignment-aliasing 4/3/1, string-offsets 4/2/2,
array-writes-appends-unset 4/2/2, nested-arrays 4/0/4,
foreach-mutation 4/0/4, function-boundaries 4/0/4,
reference-interaction 5/0/5. The 22 failing rows are bucketed in
`docs/COW_PHPT_BLOCKERS_2026-06-09.md`; `Zend/tests/bug38469.phpt` is counted
as fail because native recursive output exhausts `run-tests.php` diff memory.
Native COW reducers: 26/26, including dynamic temporary/call-result/read-slot
cases 10/10. Oracle coverage: arrays 2/2, strings 1/1, foreach 2/2,
functions 3/3, nested values 2/2, references 2/2, array element references
10/10. By-reference foreach oracle: 9/9. Mutating-internal matrix: 12/12 plus
five unsupported target diagnostics. Contract stress balances 12 nested cycles
with 0 live payloads.

## Already Ported

Lexer/parser, AST, IR, C backend, boxed values, variables, constants,
string/math/type internals, ordered arrays, `foreach`, array cursors,
`array_values()`, numeric-string keys, payload refcounts, by-value snapshots,
generated C ABI share/drop handling, shared array/string COW, nested path
detach, reference aliases, element references, by-reference parameters,
by-reference `foreach` over variables, array dimensions, and temporaries with
live appends and source COW detach, by-value parameter/function-boundary
splits, recursive/user functions, magic constants, `func_*` and bounds
diagnostics, `print_r`, binary strings, string offset reads/writes, `count()`
diagnostics, `??`, array assignment/compound/append/unset COW, native/oracle
COW reducers, array-element reference oracle coverage, unsupported
mutating-internal diagnostics, PHPT COW blocker buckets, and payload lifetime
debug counters.

## Still Needed

More PHPT COW rows, reference-aware internals, by-reference returns, nested
reference lvalues, recursive reference diagnostics, dynamic calls, and
assignment-form `??=`.

## Next Focus

1. Prove remaining function and reference boundaries.
2. Carry COW through dynamic call paths and internals.
3. Keep dashboard cells numeric and status files under 500 words.
