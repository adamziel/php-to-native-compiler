# PTN Progress

Refresh: 2026-06-10T16:32Z
Measured: prior bounded dashboard from `ptn-wk3` on
`origin/master@d2d51779`, plus focused `ptn-r52` variable-variable checks on
this branch.

## Test Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 4 | 4 | 0 |
| Native compiled PHP snippets | 370 | 370 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 200 | 153 | 47 |
| PHPT Zend rows | 76 | 68 | 8 |
| PHPT ext/standard rows | 77 | 49 | 28 |
| PHPT tests/basic+func+lang | 45 | 34 | 11 |
| PHPT other rows | 2 | 2 | 0 |
| COW contract spec tests | 7 | 7 | 0 |
| Focused COW reducer snippets | 46 | 46 | 0 |
| Recursive reference diagnostics | 4 | 4 | 0 |
| COW oracle suite | 22 | 22 | 0 |
| By-reference foreach COW oracle | 11 | 11 | 0 |
| Mutating-internal COW matrix | 14 | 14 | 0 |
| Post-merge COW gate | 25 | 25 | 0 |
| PHPT COW manifest | 29 | 28 | 1 |
| PHPT callback manifest | 2 | 2 | 0 |

## COW PHPT Buckets

`tools/phpt-cow-manifest.txt` has 29 rows: 28 passing, 1 failing.
The documented remaining COW PHPT failure is `bug69068_2.phpt`.

## Already Ported

Lexer/parser, AST/IR/C backend, boxed values, variables/constants,
string/math/type internals, ordered arrays, `foreach`, numeric keys,
payload refcounts, array/string COW, references, by-reference
params/foreach, array dimensions, temporaries, recursive/user functions,
anonymous functions for direct dynamic calls and internal callbacks, magic
constants, `func_*`, `print_r`, binary strings, string offsets, scalar
diagnostics, array literal references, array union `+`, scalar type hints,
by-reference return boundaries, `count()`, `??`, `??=`, assignment
expressions, expression-level `@`, file/directory APIs, array-path snapshots,
selected array/string internals, `debug_zval_dump()`, dynamic lvalue-reference
calls, append/list assignment expressions, grouped reference targets,
string/static callables, `array_reduce()` callback dispatch, `new stdClass`
dynamic properties, `array_count_values()`, and variable-variable scalar
reads/writes plus dynamic-root array-dimension assignment.

## Still Needed

The remaining focused COW PHPT gap is Closure `use` captures for the
`array_walk()`/`$GLOBALS` row. Broader bounded-PHPT gaps are full class
metadata, instance methods, visibility/inheritance/static properties/magic
methods, non-static method callable values, unsupported internals, 64-bit
operator exactness, object/destructuring foreach diagnostics,
object/property compound lvalues, variable-variable reference/compound forms,
scalar offset-lvalue fatal parity, and broader file APIs.

## Verification

Commands: `cargo fmt --check`; `cargo build --bin phpc`; `cargo test`;
`cargo test variable_variables`; `tools/diff-native-output.sh --snippet ...`;
`tests/lang/024.phpt` remains blocked on inline HTML on this base.
