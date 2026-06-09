# PTN Progress

Last refresh: 2026-06-09T16:37Z
Commit: pending `ptn-cqu.45` branch

## Test Dashboard

| Format / source | Ported or tracked | Passing | Needs work |
| --- | ---: | ---: | --- |
| Rust/unit | tracked | last known green | keep green |
| Native compiled PHP snippets | tracked | last known green | expand smoke matrix |
| PHPT bounded sample | 59 | 54 | 5 failing/unsupported |
| PHPT Zend | tracked in manifest | numeric string offset passes | broaden syntax/errors |
| PHPT ext/standard | tracked in manifest | partial | array basics cluster |

## Current Notes

- `array_keys()` now supports the one-argument ordered-array path.
- Focused `ext/standard/tests/array/array_keys_basic.phpt` passes through
  `phpc`.
- The 17-row ext/standard array basic reducer improved from 3/17 to 4/17.
- Remaining reducer failures cluster around missing array transforms/sums,
  aliases/constants, and pointer-function edge behavior.

## Already Ported

Lexer/parser, AST, IR, C backend, boxed values, direct variables, constants,
selected string/math/type internals, ordered arrays and `foreach`, top-level
user functions, `print_r`, selected binary-string handling, and catchable
`TypeError` for string offset reads.

## Still Needed

References, copy-on-write, classes/objects, namespaces, includes, resources,
full exceptions/finally/throw, broad standard library coverage, richer
diagnostics, and larger PHPT manifests.

## Next Focus

1. Keep compact progress patrol alive.
2. Finish native smoke matrix.
3. Reduce ext/standard array PHPT failures to generic gaps.
4. Keep runtime/string/function lanes small and mergeable.
