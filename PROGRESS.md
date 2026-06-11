# PTN Progress

Refresh: 2026-06-11T15:55Z
Measured: `ptn-if91` rebased after current `origin/master`; array null-offset
diagnostics, Closure captures, expression-form `print`, static properties,
inherited methods, property `??=`, loose object `switch`, nested string-offset
unset, simple/braced/legacy string interpolation, branch-condition assignment,
named arguments, scalar offsets, direct-variable numeric pre/post inc/dec,
object `foreach`, empty statement loop bodies, class metadata, method
callables/magic constants, cslashes, scalar variable variables, include
returns, array/object type predicates, `define()` legacy flag parity, shared
`error_reporting()` filtering, dirname edges, high-byte string escapes,
declared-function `continue`/`switch` warnings, `str_repeat()`, `array_fill()`
including integer-key overflow parity, `array_flip()`,
`array_change_key_case()`, and `array_chunk()` with assoc-key/COW coverage,
`array_combine()` including reference values, `array_filter()`, public
declared properties, `array_key_exists()` TypeErrors, braced-interpolation
alternative-offset parse errors, `chr()` out-of-range deprecations, integer
`range()`, plain heredoc/nowdoc literals, and COW evidence are integrated.
`array_key_exists.phpt` now reaches only deprecation blank-line formatting
parity.

## Test Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native compiled PHP snippets | 413 | 413 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 200 | 172 | 28 |
| PHPT Zend rows | 76 | 69 | 7 |
| PHPT ext/standard rows | 77 | 66 | 11 |
| PHPT tests/basic+func+lang | 45 | 36 | 9 |
| PHPT other rows | 2 | 2 | 0 |
| PHPT COW manifest | 29 | 29 | 0 |
| COW reducer/oracle/gate suites | 150 | 150 | 0 |
| PHPT callback manifest | 3 | 3 | 0 |

## Already Ported

Lexer/parser, AST/IR/C backend, boxed values, variables/constants, strings,
plain heredoc/nowdoc literals, math/type internals, ordered arrays, `foreach`,
empty statements,
COW/references/by-reference params, dimensions, recursive/user functions,
Closure captures, magic constants, `func_*`, `print_r`, scalar diagnostics,
array union, scalar type hints, by-reference returns, variadics/default
parameters, `count()`, `??`, assignment expressions, named arguments,
expression-form `print`, direct-variable numeric inc/dec statement and
expression results, simple array-offset and legacy dollar-brace interpolation,
`@`, file predicates, selected array/string internals
including `array_fill()`, `array_flip()`, `array_change_key_case()`,
`array_chunk()`, `array_combine()`, `array_filter()`,
`array_count_values()`, `str_repeat()`, and integer `range()`,
lvalue-reference calls, list/append assignment, scalar variable variables,
method callables, `array_reduce()`/`array_walk()`, `stdClass`, class/object
metadata, `$this`, object `foreach`, scalar array-lvalue fatals, static
properties, property `??=`, inherited public methods, include/require return
propagation, diagnostic filtering, and the focused PHPT rows listed above.

## Still Needed

Constructors, typed/non-public properties, interfaces/traits, broader
inheritance, magic methods, property compounds beyond `??=`,
static-property compound/null-coalescing lvalues, destructors, exceptions,
reflection, unsupported internals, 64-bit operator exactness, destructuring
`foreach`, heredoc interpolation/flexible indentation, legacy interpolation
diagnostic ordering, remaining scalar offset-lvalue parity, dynamic-variable
array-offset lvalues, dynamic
include/include_once behavior, non-direct-variable and non-numeric inc/dec
parity, broader file APIs, and `chr()` float-to-int precision diagnostics.

## Verification

Commands: focused native/parser/phpc reducers for recent slices, including
`compile_increment_and_decrement_expression_results_to_native_binary`,
`compile_array_combine_to_native_binary`,
`compile_array_combine_preserves_reference_values_to_native_binary`,
`compile_array_filter_to_native_binary`,
`compile_array_fill_to_native_binary`, `compile_array_flip_to_native_binary`,
`compile_array_change_key_case_to_native_binary`,
`compile_array_chunk_to_native_binary`,
`compile_simple_and_legacy_interpolation_to_native_binary`,
`compile_declared_public_instance_properties_to_native_binary`,
`compile_plain_heredoc_values_to_native_binary`, and
`compile_foreach_empty_statement_body_to_native_binary`; exact
`array_combine_basic.phpt`, `array_filter_basic.phpt`,
`array_null_offset_deprecation.phpt`, `array_count_values.phpt`,
`array_fill.phpt`, `array_fill_basic.phpt`, `array_flip_basic.phpt`,
`array_change_key_case.phpt`, `array_chunk_basic1.phpt`,
`array_chunk_basic2.phpt`, `array_chunk2.phpt`,
`alternative_offset_syntax_in_encaps_string.phpt`, `chr_out_of_range.phpt`,
`ord_basic.phpt`, declared-function continue/switch rows,
`foreachLoop.001.phpt`, `print_r_ints.phpt`, and one-row
`array_key_exists.phpt`; `cargo fmt --check`; `cargo build --bin phpc`;
`cargo test`; `tools/run-bounded-phpt.sh tools/phpt-cow-manifest.txt`;
`tools/run-bounded-phpt.sh tools/phpt-bounded-manifest.txt`;
`tools/run-post-merge-cow-gate.sh`.
