# PTN Progress

Refresh: 2026-06-10T10:56Z
Measured: `ptn-14r` rebased on `master@d2b551b31`; native tests, smoke,
post-merge COW gate, and PHPT manifests.

## Test Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native compiled PHP snippets | 344 | 344 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 200 | 150 | 50 |
| PHPT Zend rows | 76 | 67 | 9 |
| PHPT ext/standard rows | 77 | 47 | 30 |
| PHPT tests/basic+func+lang | 45 | 34 | 11 |
| PHPT other rows | 2 | 2 | 0 |
| COW contract spec tests | 7 | 7 | 0 |
| COW-focused native tests | 16 | 16 | 0 |
| Focused COW reducer snippets | 38 | 38 | 0 |
| COW oracle suite | 22 | 22 | 0 |
| By-reference foreach COW oracle | 11 | 11 | 0 |
| Post-merge COW gate | 25 | 25 | 0 |
| COW/reference-focused native tests | 12 | 12 | 0 |
| Mutating-internal COW matrix | 14 | 14 | 0 |
| PHPT COW manifest | 29 | 24 | 5 |
| Recursive reference diagnostic reducers | 9 | 9 | 0 |
| Focused PHPT foreach COW row | 1 | 1 | 0 |

## COW PHPT Buckets

`tools/phpt-cow-manifest.txt` has 29 rows. Focused evidence remains 24 passing,
5 failing: assignment-aliasing 4/4, string-offsets 4/4,
array-writes-appends-unset 4/4, nested-arrays 3/4, foreach-mutation 3/4,
function-boundaries 1/4, reference-interaction 5/5. Native COW reducers are
38/38, recursive diagnostics 9/9, mutating-internal matrix 14/14 plus six
unsupported diagnostics, and post-merge COW gate 25/25: 12 oracle, 1 notice,
and 12 diagnostic cases.

## Already Ported

Lexer/parser, AST, IR, C backend, boxed values, variables, constants,
string/math/type internals, ordered arrays, `foreach`, cursors,
`array_values()`, numeric-string keys, payload refcounts, array/string COW,
references, by-reference parameters and `foreach`, array dimensions,
temporaries, recursive/user functions, magic constants, `func_*`, `print_r`,
binary strings, string offsets, scalar offset diagnostics, array literal
reference elements, array union `+`, scalar type hints, by-reference return
alias/separation boundaries, `count()`, `??`, assignment expressions,
expression-level `@`, selected file APIs, array-path RHS snapshots,
reference-aware `array_sum()`/`strtr()`/`in_array()`, recursive array merge and
replace, `debug_zval_dump()`, dynamic lvalue-reference calls, append/list
assignment expressions for reference arrays, direct-variable `??=`, offset-form
`??=` diagnostics, grouped reference targets, recursive/same-array/nested
reference and class-syntax diagnostics, and value fallback with PHP notice when
non-reference call results are assigned by reference.

## Still Needed

Five focused PHPT COW rows remain: nested recursive reference lvalues,
`array_walk()` closure/global swaps, recursive/call-result by-reference return
chaining, `array_reduce()` callback refcounts, and callback returns by
reference. Broader gaps include offset-form `??=` runtime support,
closure/callback surfaces, objects, unsupported array/string internals, and
64-bit operator exactness.
