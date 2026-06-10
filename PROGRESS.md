# PTN Progress

Refresh: 2026-06-10T10:18Z
Measured: `polecat/117/ptn-hto@mq7v7ud2` after `ptn-4dv`/`ptn-c7b`; native
boundary tests and post-merge COW gate.

## Test Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 4 | 4 | 0 |
| Native compiled PHP snippets | 344 | 344 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 200 | 146 | 54 |
| PHPT Zend rows | 76 | 63 | 13 |
| PHPT ext/standard rows | 77 | 48 | 29 |
| PHPT tests/basic+func+lang | 45 | 33 | 12 |
| PHPT other rows | 2 | 2 | 0 |
| COW contract spec tests | 7 | 7 | 0 |
| COW-focused native tests | 16 | 16 | 0 |
| Focused COW reducer snippets | 34 | 34 | 0 |
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
5 failing. Bucket pass counts: assignment-aliasing 4/4, string-offsets 4/4,
array-writes-appends-unset 4/4, nested-arrays 3/4, foreach-mutation 3/4,
function-boundaries 1/4, reference-interaction 5/5. Native COW reducers are
34/34, recursive reference diagnostics are 9/9, mutating-internal matrix is
14/14 plus six unsupported diagnostics, and the post-merge COW gate is 25/25.

## Already Ported

Lexer/parser, AST, IR, C backend, boxed values, variables, constants,
string/math/type internals, ordered arrays, `foreach`, cursors,
`array_values()`, numeric-string keys, payload refcounts, array/string COW,
references, by-reference parameters and `foreach`, array dimensions,
temporaries, recursive/user functions, magic constants, `func_*`, `print_r`,
binary strings, string offsets, corpus-style scalar offset diagnostics, array
literal reference elements, array union `+`, scalar type hints, by-reference
return alias/separation boundaries, `count()`, `??`, COW gates/oracles,
assignment expressions, expression-level `@` suppression, `file_put_contents()`,
`sha1_file()`, `unlink()` byte-file slices, array-path RHS snapshots,
reference-aware `array_sum()`/`strtr()`/`in_array()`, `array_merge_recursive()`/
`array_replace_recursive()`, `debug_zval_dump()`, dynamic lvalue-reference
calls through fallback dispatch, append/list assignment expressions for
reference arrays, direct-variable assignment-form `??=`, six offset-form `??=`
diagnostics, grouped reference targets, and recursive/same-array/nested
reference and class-syntax diagnostics.

## Still Needed

More PHPT COW rows, reference-aware internals, recursive/call-result
by-reference return chaining, closure/callback by-reference returns, nested
reference lvalues, offset-form `??=` runtime support, and broader file APIs.

## Next Focus

1. Prove remaining function and reference boundaries.
2. Carry COW through remaining callback/internal call paths.
3. Keep status files under 500 words.
