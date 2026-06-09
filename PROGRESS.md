# PTN Progress

Last refresh: 2026-06-09T18:43Z
Commit: branch `ptn-cqu.44`

## Test Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | --- |
| Source unit tests | 3 | 3 | 0 |
| Native compiled PHP snippets | 268 | 268 | 0 |
| PHPT parsed bounded log | 171 | 121 | 50 |
| PHPT Zend rows | 76 | 60 plus 2 cursor telemetry rows | 16 |
| PHPT ext/standard rows | 77 | 44 | 33 |
| PHPT tests/basic+func+lang | 18 | 17 | 1 |
| COW-focused tests | 0 | 0 | 1 full suite needed |

## Already Ported

Lexer/parser, AST, IR, C backend, boxed values, direct variables, constants,
selected string/math/type internals, ordered arrays, variable-root array cursor
internals, `foreach`, top-level user functions with scoped `__FUNCTION__`/
`__METHOD__` magic constants, `print_r`, selected binary-string handling, and
catchable `TypeError` for string offset reads.

## Latest Slice

Bounded ordered-array cursor internals:

- `end()` and `prev()` are registered with `current()`, `key()`, `reset()`, and
  `next()`.
- Variable-root `reset($array)`, `end($array)`, `next($array)`, and
  `prev($array)` update the ordered-array cursor and return cloned selected
  values or `false` at invalid cursor positions.
- Non-variable mutating cursor arguments stop at an explicit unsupported
  by-reference boundary; scalar variable arguments emit a clear array type
  diagnostic.
- Ordered-array insertion and unset preserve the modeled cursor across common
  mutation cases.
- `cargo test` passes with 273 tests.
- Focused public PHPT telemetry through `phpc` passes
  `Zend/tests/bug41372.phpt` and `Zend/tests/gh13178_4.phpt`.

## Still Needed

Copy-on-write for arrays, strings, variables, function calls, foreach, nested
containers, and references. Object cursor internals and full PHP by-reference
temporary/notice behavior for mutating cursor calls also remain unsupported.
All non-COW work is paused unless it is required to prove COW.

## Next Focus

1. Build a dedicated COW correctness suite.
2. Implement shared payload refcounts and detach-on-write.
3. Prove arrays, strings, nested values, foreach, and function boundaries.
4. Keep this dashboard numeric and under 500 words.
