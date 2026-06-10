# PTN Progress

Refresh: 2026-06-10T12:45Z
Measured: `ptn-ept` on `origin/master@1c445f8`; focused `array_reduce()`
accumulator evidence plus prior recursive array-literal, string-callable,
nested-reference, and `array_fill_keys()` gates.

## Test Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native compiled PHP snippets | 353 | 353 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 200 | 152 | 48 |
| PHPT Zend rows | 76 | 68 | 8 |
| PHPT ext/standard rows | 77 | 48 | 29 |
| PHPT tests/basic+func+lang | 45 | 34 | 11 |
| PHPT other rows | 2 | 2 | 0 |
| COW contract spec tests | 7 | 7 | 0 |
| Focused COW reducer snippets | 45 | 45 | 0 |
| Recursive reference diagnostics | 4 | 4 | 0 |
| COW oracle suite | 22 | 22 | 0 |
| By-reference foreach COW oracle | 11 | 11 | 0 |
| Mutating-internal COW matrix | 14 | 14 | 0 |
| Post-merge COW gate | 25 | 25 | 0 |
| PHPT COW manifest | 29 | 26 | 3 |
| PHPT callback manifest | 1 | 1 | 0 |

## COW PHPT Buckets

`tools/phpt-cow-manifest.txt` has 29 rows: 26 passing, 3 failing.
Buckets: assignment-aliasing 4/4, string-offsets 4/4,
array-writes-appends-unset 4/4, nested-arrays 4/4, foreach-mutation 3/4,
function-boundaries 2/4, reference-interaction 5/5. Named `array_reduce()`
callbacks preserve by-reference returns and callback-visible accumulator
refcounts; exact closure-backed PHPT rows remain blocked by Closure/callable
values (`ptn-dis`).

## Already Ported

Lexer/parser, AST, IR, C backend, boxed values, variables, constants,
string/math/type internals, ordered arrays, `foreach`, cursors,
`array_values()`, numeric-string keys, payload refcounts, array/string COW,
references, by-reference parameters and `foreach`, array dimensions,
temporaries, recursive/user functions, magic constants, `func_*`, `print_r`,
binary strings, string offsets, scalar offset diagnostics, array literal
reference elements, array union `+`, scalar type hints, by-reference return
alias/separation boundaries, `count()`, `??`, assignment expressions,
expression-level `@`, selected file APIs, array-path RHS snapshots,
reference-aware `array_sum()`/`strtr()`/`in_array()`, recursive array merge and
replace, `debug_zval_dump()`, dynamic lvalue-reference calls, append/list
assignment expressions for reference arrays, nested same-array reference
lvalues, direct-variable and keyed offset-form `??=`, append-form `??=`
diagnostics, grouped reference targets, recursive array literal cleanup, named
`array_reduce()` callback dispatch with by-reference returns and accumulator
debug refcounts, non-reference call-result by-reference fallback notices,
call-result by-reference return chains, `array_fill_keys()`, and
string-callable `call_user_func()` dispatch.

## Still Needed

Remaining COW PHPT gaps are closure callback mutation through
`array_walk()`/`$GLOBALS`, closure-backed callback by-reference returns,
closure-backed `array_reduce()` accumulator rows, and broader recursive
by-reference return edges. Broader bounded-PHPT gaps are objects, unsupported
array/string internals, 64-bit operator exactness, foreach diagnostics,
object/property compound lvalues, scalar offset-lvalue fatal parity, and file
APIs.

## Verification

Commands: `cargo fmt --check`; `cargo test`; `tools/run-native-smoke-matrix.sh`;
`tools/run-post-merge-cow-gate.sh`; `tools/run-bounded-phpt.sh
tools/phpt-cow-manifest.txt`; callback manifest.
