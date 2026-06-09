# PTN Progress

Refresh: 2026-06-09T23:23Z
Measured branch: `ptn-4yt.9` reference-interaction update on `origin/master`

## Test Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 4 | 4 | 0 |
| Native compiled PHP snippets | 314 | 314 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 200 | 146 | 54 |
| PHPT Zend rows | 76 | 63 | 13 |
| PHPT ext/standard rows | 77 | 48 | 29 |
| PHPT tests/basic+func+lang | 45 | 33 | 12 |
| PHPT other rows | 2 | 2 | 0 |
| COW contract spec tests | 7 | 7 | 0 |
| COW-focused native tests | 13 | 13 | 0 |
| Focused COW reducer snippets | 28 | 28 | 0 |
| COW oracle suite | 22 | 22 | 0 |
| By-reference foreach COW oracle | 11 | 11 | 0 |
| Post-merge COW gate | 14 | 14 | 0 |
| COW/reference-focused native tests | 12 | 12 | 0 |
| Mutating-internal COW matrix | 14 | 14 | 0 |
| PHPT COW manifest | 29 | 14 | 15 |
| Focused PHPT foreach COW row | 1 | 1 | 0 |

## COW PHPT Buckets

`tools/phpt-cow-manifest.txt` still has 29 rows. Before evidence
2026-06-09T20:32Z: 9 passing, 20 failing. Refresh at 2026-06-09T23:23Z on
`ptn-4yt.9`: 14 passing, 15 failing. Bucket pass counts:
assignment-aliasing 4/4, string-offsets 2/4, array-writes-appends-unset 3/4,
nested-arrays 0/4, foreach-mutation 1/4, function-boundaries 0/4,
reference-interaction 4/5. The full bounded runner still stops at
`Zend/tests/bug38469.phpt`; the remaining rows were rerun bucket-wise and that
row is counted failing. Blocker beads are linked in
`docs/COW_PHPT_BLOCKERS_2026-06-09.md`; `ptn-6vg` tracks the remaining
append/list assignment expression row. Native COW reducers are 28/28,
by-reference foreach oracle is 11/11, mutating-internal matrix is 14/14 plus
six unsupported target diagnostics, and the post-merge COW gate is 14/14.

## Already Ported

Lexer/parser, AST, IR, C backend, boxed values, variables, constants,
string/math/type internals, ordered arrays, `foreach`, array cursors,
`array_values()`, numeric-string keys, payload refcounts, array/string COW,
references, by-reference parameters and `foreach`, array dimensions,
temporaries, recursive/user functions, magic constants, `func_*`, `print_r`,
binary strings, string offsets, corpus-style scalar offset diagnostics,
`count()`, `??`, COW gates/oracles, assignment expressions,
expression-level `@` suppression,
`file_put_contents()`, `sha1_file()`, `unlink()` byte-file slices, reference
array literals, reference-aware `array_sum()`/`strtr()`, and
`debug_zval_dump()` reference inspection.

## Still Needed

More PHPT COW rows, reference-aware internals, by-reference returns, nested
reference lvalues, append/list assignment expressions, recursive reference
diagnostics, dynamic calls, and assignment-form `??=`. File API coverage remains
intentionally narrow.

## Next Focus

1. Prove remaining function and reference boundaries.
2. Carry COW through dynamic call paths and internals.
3. Keep dashboard cells numeric and status files under 500 words.
