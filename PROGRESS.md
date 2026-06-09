# PTN Progress

Refresh: 2026-06-10T00:12Z
Measured branches: `ptn-4yt.9` then `ptn-4yt.3` rebased on `master`;
`ptn-4yt.2` adds array-path RHS snapshots and `array_merge_recursive()`.

## Test Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 4 | 4 | 0 |
| Native compiled PHP snippets | 322 | 322 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 200 | 146 | 54 |
| PHPT Zend rows | 76 | 63 | 13 |
| PHPT ext/standard rows | 77 | 48 | 29 |
| PHPT tests/basic+func+lang | 45 | 33 | 12 |
| PHPT other rows | 2 | 2 | 0 |
| COW contract spec tests | 7 | 7 | 0 |
| COW-focused native tests | 15 | 15 | 0 |
| Focused COW reducer snippets | 30 | 30 | 0 |
| COW oracle suite | 22 | 22 | 0 |
| By-reference foreach COW oracle | 11 | 11 | 0 |
| Post-merge COW gate | 15 | 15 | 0 |
| COW/reference-focused native tests | 12 | 12 | 0 |
| Mutating-internal COW matrix | 14 | 14 | 0 |
| PHPT COW manifest | 29 | 19 | 10 |
| Focused PHPT foreach COW row | 1 | 1 | 0 |

## COW PHPT Buckets

`tools/phpt-cow-manifest.txt` still has 29 rows. Reference-interaction improved
from 0/5 on `origin/master` to 4/5 on `ptn-4yt.9`; total COW manifest evidence
is now 19 passing, 10 failing. Bucket pass counts: assignment-aliasing 4/4,
string-offsets 4/4, array-writes-appends-unset 4/4, nested-arrays 0/4,
foreach-mutation 2/4, function-boundaries 1/4, reference-interaction 4/5.
The full bounded runner still stops at `Zend/tests/bug38469.phpt`; that row is
counted failing. Remaining blockers are tracked in
`docs/COW_PHPT_BLOCKERS_2026-06-09.md`. Native COW reducers are 30/30,
by-reference foreach oracle is 11/11, mutating-internal matrix is 14/14 plus
six unsupported target diagnostics, and the post-merge COW gate is 15/15.

## Already Ported

Lexer/parser, AST, IR, C backend, boxed values, variables, constants,
string/math/type internals, ordered arrays, `foreach`, array cursors,
`array_values()`, numeric-string keys, payload refcounts, array/string COW,
references, by-reference parameters and `foreach`, array dimensions,
temporaries, recursive/user functions, magic constants, `func_*`, `print_r`,
binary strings, string offsets, corpus-style scalar offset diagnostics, array
literal reference elements, array union `+`, scalar type hints, typed
by-reference return separation, `count()`, `??`, COW gates/oracles, assignment
expressions, expression-level `@` suppression, `file_put_contents()`,
`sha1_file()`, `unlink()` byte-file slices, array-path RHS snapshots,
reference-aware `array_sum()`/`strtr()`, `array_merge_recursive()`,
`debug_zval_dump()` reference inspection, and string-valued dynamic calls with
lvalue reference arguments through the fallback dispatcher.

## Still Needed

More PHPT COW rows, reference-aware internals, call-result references, broader
by-reference returns, nested reference lvalues, recursive reference diagnostics,
closure/callback call surfaces, and assignment-form `??=`. File API coverage
remains narrow.

## Next Focus

1. Prove remaining function and reference boundaries.
2. Carry COW through remaining callback/internal call paths.
3. Keep dashboard cells numeric and status files under 500 words.
