# PTN Progress

Refresh: 2026-06-11T12:52Z
Measured: `ptn-28xs` rebased after current `origin/master`; array null-offset
diagnostics, Closure captures, expression-form `print`, static properties,
inherited methods, property `??=`, loose object `switch`, nested string-offset
unset, braced interpolation, branch-condition assignment, named arguments,
scalar offsets, object `foreach`, class metadata, method callables/magic
constants, cslashes, scalar variable variables, include returns, array/object
type predicates, `define()` legacy flag parity, shared `error_reporting()`
filtering, dirname edges, high-byte string escapes, declared-function
`continue`/`switch` warnings, `str_repeat()`, `array_fill()`, and
braced-interpolation alternative-offset parse errors are integrated. Bounded
PHPT/COW evidence is refreshed; `array_fill_basic.phpt` still fails at the
separate heredoc parser boundary.

## Test Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native compiled PHP snippets | 398 | 398 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 200 | 159 | 41 |
| PHPT Zend rows | 76 | 69 | 7 |
| PHPT ext/standard rows | 77 | 55 | 22 |
| PHPT tests/basic+func+lang | 45 | 34 | 11 |
| PHPT other rows | 2 | 2 | 0 |
| COW contract spec tests | 7 | 7 | 0 |
| Focused COW reducer snippets | 47 | 47 | 0 |
| Recursive reference diagnostics | 4 | 4 | 0 |
| COW oracle suite | 22 | 22 | 0 |
| By-reference foreach COW oracle | 11 | 11 | 0 |
| Mutating-internal COW matrix | 14 | 14 | 0 |
| Post-merge COW gate | 25 | 25 | 0 |
| PHPT COW manifest | 29 | 29 | 0 |
| PHPT callback manifest | 3 | 3 | 0 |

## Already Ported

Lexer/parser, AST/IR/C backend, boxed values, variables/constants, strings,
math/type internals, ordered arrays, `foreach`, COW/references/by-reference
params, dimensions, recursive/user functions, Closure captures, magic
constants, `func_*`, `print_r`, scalar diagnostics, array union, scalar type
hints, by-reference returns, variadics/default parameters, `count()`, `??`,
assignment expressions, named arguments, expression-form `print`, `@`, file
predicates, selected array/string internals, lvalue-reference calls,
list/append assignment, scalar variable variables, method callables,
`array_reduce()`/`array_walk()`, `array_count_values()`, `stdClass`,
class/object metadata, `$this`, scalar array-lvalue fatals, static properties,
property `??=`, inherited public methods, include/require return propagation,
diagnostic filtering, and the focused PHPT rows listed above.

## Still Needed

Constructors, declared properties, non-public visibility, interfaces/traits,
broader inheritance, magic methods, broader property compounds,
static-property compound/null-coalescing lvalues, destructors, exceptions,
reflection, unsupported internals, 64-bit operator exactness, destructuring
`foreach`, heredoc/nowdoc, remaining string sub-path scalar offset-lvalue
parity, dynamic-variable array-offset lvalues, dynamic include/include_once
behavior, and broader file APIs.

## Verification

Commands: focused native/parser/phpc reducers for recent slices; exact
`array_null_offset_deprecation.phpt`, `array_count_values.phpt`,
`array_fill.phpt`, `alternative_offset_syntax_in_encaps_string.phpt`,
`ord_basic.phpt`, and declared-function continue/switch rows;
`cargo fmt --check`; `cargo build --bin phpc`; `cargo test`;
`tools/run-phpt-manifest.sh tools/phpt-cow-manifest.txt`;
`tools/run-phpt-manifest.sh tools/phpt-manifest-200.txt`;
`tools/run-post-merge-cow-gate.sh`.
PHPT runners resolve php-src via `PHP_SRC_PHPT`, `/home/claude/php-src-phpt`, or
`.runtime/php-src-phpt`.
