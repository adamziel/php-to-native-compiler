# PTN Progress

Refresh: 2026-06-11T13:58Z
Measured: `ptn-m0e0` rebased after current `origin/master`; array
null-offset diagnostics, Closure captures, expression-form `print`, static
properties, inherited methods, property `??=`, loose object `switch`, nested
string-offset unset, braced interpolation, branch-condition assignment, named
arguments, scalar offsets, object `foreach`, class metadata, method
callables/magic constants, cslashes, scalar variable variables, include
returns, array/object type predicates, `define()` legacy flag parity, shared
`error_reporting()` filtering, dirname edges, high-byte string escapes,
declared-function `continue`/`switch` warnings, `str_repeat()`,
`array_fill()` including integer-key overflow parity, `array_flip()`,
`array_change_key_case()`, braced-interpolation alternative-offset parse
errors, `chr()` out-of-range deprecations with suppression coverage, integer
`range()` including default and descending-step coverage, and COW evidence are
integrated. `array_fill_basic.phpt` still fails at the separate heredoc parser
boundary.

## Test Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native compiled PHP snippets | 403 | 403 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 200 | 166 | 34 |
| PHPT Zend rows | 76 | 69 | 7 |
| PHPT ext/standard rows | 77 | 61 | 16 |
| PHPT tests/basic+func+lang | 45 | 34 | 11 |
| PHPT other rows | 2 | 2 | 0 |
| PHPT COW manifest | 29 | 29 | 0 |
| COW reducer/oracle/gate suites | 150 | 150 | 0 |
| PHPT callback manifest | 3 | 3 | 0 |

## Already Ported

Lexer/parser, AST/IR/C backend, boxed values, variables/constants, strings,
math/type internals, ordered arrays, `foreach`, COW/references/by-reference
params, dimensions, recursive/user functions, Closure captures, magic
constants, `func_*`, `print_r`, scalar diagnostics, array union, scalar type
hints, by-reference returns, variadics/default parameters, `count()`, `??`,
assignment expressions, named arguments, expression-form `print`, `@`, file
predicates, selected array/string internals including `array_fill()`,
`array_flip()`, `array_change_key_case()`, `array_count_values()`,
`str_repeat()`, and integer `range()`, lvalue-reference calls, list/append
assignment, scalar variable variables, method callables,
`array_reduce()`/`array_walk()`, `stdClass`, class/object metadata, `$this`,
object `foreach`, scalar array-lvalue fatals, static properties, property
`??=`, inherited public methods, include/require return propagation,
diagnostic filtering, and the focused PHPT rows listed above.

## Still Needed

Constructors, declared properties, non-public visibility, interfaces/traits,
broader inheritance, magic methods, property compounds beyond `??=`,
static-property compound/null-coalescing lvalues, destructors, exceptions,
reflection, unsupported internals, 64-bit operator exactness, destructuring
`foreach`, heredoc/nowdoc, remaining scalar offset-lvalue parity,
dynamic-variable array-offset lvalues, dynamic include/include_once behavior,
broader file APIs, and `chr()` float-to-int precision diagnostics.

## Verification

Commands: focused native/parser/phpc reducers for recent slices, including
`compile_array_fill_to_native_binary`, `compile_array_flip_to_native_binary`,
and `compile_array_change_key_case_to_native_binary`;
exact `array_null_offset_deprecation.phpt`, `array_count_values.phpt`,
`array_fill.phpt`, `array_flip_basic.phpt`, `array_change_key_case.phpt`,
`alternative_offset_syntax_in_encaps_string.phpt`, `chr_out_of_range.phpt`,
`ord_basic.phpt`, declared-function continue/switch rows, and
`print_r_ints.phpt`; focused `chr` and `range()` native reducers;
`cargo fmt --check`; `cargo build --bin phpc`; `cargo test`;
`tools/run-bounded-phpt.sh tools/phpt-cow-manifest.txt`
(20260611T135658Z, 29/29);
`tools/run-bounded-phpt.sh tools/phpt-bounded-manifest.txt`
(20260611T135037Z, 166/200);
`tools/run-post-merge-cow-gate.sh`.
