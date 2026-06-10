# PTN Progress

Refresh: 2026-06-10T12:23Z
Measured: `ptn-dis` rebased on `origin/master@8f0f5216`; focused closure
callback native evidence, nested recursive reference lvalue evidence,
`array_fill_keys()` evidence, and `tools/phpt-cow-manifest.txt`.

## Test Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native compiled PHP snippets | 351 | 351 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 200 | 151 | 49 |
| PHPT Zend rows | 76 | 67 | 9 |
| PHPT ext/standard rows | 77 | 48 | 29 |
| PHPT tests/basic+func+lang | 45 | 34 | 11 |
| PHPT other rows | 2 | 2 | 0 |
| COW contract spec tests | 7 | 7 | 0 |
| Focused COW reducer snippets | 42 | 42 | 0 |
| Recursive reference diagnostics | 7 | 7 | 0 |
| COW oracle suite | 22 | 22 | 0 |
| By-reference foreach COW oracle | 11 | 11 | 0 |
| Mutating-internal COW matrix | 14 | 14 | 0 |
| Post-merge COW gate | 25 | 25 | 0 |
| PHPT COW manifest | 29 | 26 | 3 |

## COW PHPT Buckets

`tools/phpt-cow-manifest.txt` has 29 rows. Current focused evidence is 26
passing, 3 failing: assignment-aliasing 4/4, string-offsets 4/4,
array-writes-appends-unset 4/4, nested-arrays 4/4, foreach-mutation 3/4,
function-boundaries 2/4, reference-interaction 5/5. Captureless closure
callables now cover the exact `array_reduce()` callback return-by-reference row;
`array_reduce()` accumulator refcounts remain `ptn-ept`. Details live in
`docs/COW_PHPT_BLOCKERS_2026-06-09.md`.

## Already Ported

Lexer/parser, AST, IR, C backend, boxed values, variables, constants,
string/math/type internals, ordered arrays, `foreach`, cursors,
`array_values()`, `array_fill_keys()`, numeric-string keys, payload refcounts,
array/string COW, references, by-reference parameters and `foreach`, array
dimensions, temporaries, recursive/user functions, magic constants, `func_*`,
`print_r`, binary strings, string offsets, scalar offset diagnostics, array
literal reference elements, array union `+`, scalar type hints, by-reference
return alias/separation boundaries, `count()`, `??`, assignment expressions,
expression-level `@`, selected file APIs, array-path RHS snapshots,
reference-aware `array_sum()`/`strtr()`/`in_array()`, recursive array merge and
replace, `debug_zval_dump()`, dynamic lvalue-reference calls, append/list
assignment expressions for reference arrays, nested same-array reference
lvalues with recursive `var_dump()`, direct-variable `??=`, keyed array/string
offset-form `??=`, append-form `??=` diagnostics, grouped reference targets,
recursive/same-array/nested reference and class-syntax diagnostics, named
`array_reduce()` callback dispatch with by-reference returns, captureless
anonymous-function callables, closure-backed `array_reduce()` by-reference
callback returns, value fallback with PHP notice when non-reference call results
are assigned by reference, and call-result by-reference return chains.

## Still Needed

Remaining COW PHPT gaps are closure callback mutation through
`array_walk()`/`$GLOBALS`, recursive by-reference return chaining, and
`array_reduce()` accumulator refcount behavior. Broader bounded-PHPT gaps are
still full Closure objects/captures, objects, unsupported array/string
internals, 64-bit operator exactness, foreach edge diagnostics,
object/property compound lvalues, scalar offset-lvalue fatal parity, and
broader file APIs.

## Verification

Commands: `cargo fmt --check`; `cargo test`; `tools/run-native-smoke-matrix.sh`;
`tools/run-post-merge-cow-gate.sh`; `tools/run-bounded-phpt.sh
tools/phpt-cow-manifest.txt`. Broad manifest is now 151/200.
