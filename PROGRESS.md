# PTN Progress

Last refresh: 2026-06-09T16:56Z
Commit: `70e7254bcae0`
Priority: COW only. Non-COW work allowed: 0 unless required for COW.

## Test Dashboard

| Format / source | Ported | Passing | Failing | Needed |
| --- | ---: | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 | 0 |
| Native compiled PHP snippets | 273 | 273 | 0 | 0 |
| COW-adjacent native | 1 | 1 | 0 | 5 |
| PHPT bounded total | 200 | 138 | 62 | 62 |
| PHPT Zend rows | 76 | 60 | 16 | 16 |
| PHPT ext/standard rows | 77 | 46 | 31 | 31 |
| PHPT tests/* rows | 47 | 32 | 15 | 15 |

## Already Ported

Lexer/parser, AST, IR, C backend, boxed values, direct variables, constants,
selected string/math/type internals, ordered arrays and `foreach`, numeric-string
array key normalization coverage, top-level user functions with scoped
`__FUNCTION__`/`__METHOD__` magic constants, `print_r`, selected binary-string
handling, catchable `TypeError` for string offset reads, and catchable
`count()` non-array diagnostics.

## COW Blockers

5: shared payload refcounts, detach-on-write, nested container cloning,
by-value `foreach` mutation visibility, function-boundary value separation.
COW-adjacent PHPT failures: 32/200.

## Next Focus

1. Build a dedicated COW correctness suite.
2. Implement shared payload refcounts and detach-on-write.
3. Prove arrays, strings, nested values, foreach, and function boundaries.
4. Keep dashboards numeric and under 500 words.
