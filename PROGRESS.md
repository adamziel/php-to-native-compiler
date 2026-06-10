# PTN Progress

Refresh: 2026-06-10T12:23Z
Measured: `polecat/58-mq81a2ss` rebased on `origin/master@8f0f52165`;
focused directory/status file API native+PHPT evidence.

## Test Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native compiled PHP snippets | 347 | 347 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 200 | 151 | 49 |
| PHPT Zend rows | 76 | 67 | 9 |
| PHPT ext/standard rows | 77 | 48 | 29 |
| PHPT tests/basic+func+lang | 45 | 34 | 11 |
| PHPT other rows | 2 | 2 | 0 |
| COW contract spec tests | 7 | 7 | 0 |
| Focused COW reducer snippets | 41 | 41 | 0 |
| Recursive reference diagnostics | 7 | 7 | 0 |
| COW oracle suite | 22 | 22 | 0 |
| By-reference foreach COW oracle | 11 | 11 | 0 |
| Mutating-internal COW matrix | 14 | 14 | 0 |
| Post-merge COW gate | 25 | 25 | 0 |
| PHPT COW manifest | 29 | 25 | 4 |

## COW PHPT Buckets

`tools/phpt-cow-manifest.txt` has 29 rows: 25 passing, 4 failing. Named
`array_reduce()` callbacks preserve by-reference
callback returns; the exact closure-backed PHPT row remains blocked by
Closure/callable values (`ptn-dis`). Details live in
`docs/COW_PHPT_BLOCKERS_2026-06-09.md`.

## Already Ported

Lexer/parser, AST, IR, C backend, boxed values, variables, constants,
string/math/type internals, ordered arrays, `foreach`, cursors,
`array_values()`, numeric-string keys, payload refcounts, array/string COW,
references, by-reference parameters and `foreach`, array dimensions,
temporaries, recursive/user functions, magic constants, `func_*`, `print_r`,
binary strings, string offsets, scalar offset diagnostics, array literal
reference elements, array union `+`, scalar type hints, by-reference return
alias/separation boundaries, `count()`, `??`, assignment expressions,
expression-level `@`, selected file APIs (`file_put_contents()`,
`sha1_file()`, `unlink()`, `mkdir()`, `rmdir()`, `file_exists()`,
`is_dir()`, `is_file()`, and no-op `clearstatcache()`), array-path RHS snapshots,
reference-aware `array_sum()`/`strtr()`/`in_array()`, recursive array merge and
replace, `debug_zval_dump()`, dynamic lvalue-reference calls, append/list
assignment expressions for reference arrays, nested same-array reference
lvalues with recursive `var_dump()`, direct-variable `??=`, keyed array/string
offset-form `??=`, append-form `??=` diagnostics, grouped reference targets,
recursive/same-array/nested reference and class-syntax diagnostics, named
`array_reduce()` callback dispatch with by-reference returns, value fallback
with PHP notice when non-reference call results are assigned by reference, and
call-result by-reference return chains, and `array_fill_keys()` over current
boxed arrays with scalar key coercion.

## Still Needed

Remaining COW PHPT gaps are closure callback mutation through
`array_walk()`/`$GLOBALS`, recursive by-reference return chaining,
closure-backed callback by-reference returns, and `array_reduce()`
callback/refcount behavior. Broader bounded-PHPT gaps are still objects,
unsupported array/string internals, 64-bit operator exactness, foreach edge
diagnostics, object/property compound lvalues, scalar offset-lvalue fatal
parity, recursive mkdir/stream context/stat-cache filesystem parity, and
broader file APIs.

## Verification

Commands: `cargo fmt --check`; `cargo test`; focused native `cargo test
compile_directory_status_file_apis_to_native_binary --test compile_native`;
focused PHPT `is_dir_variation1.phpt` and `mkdir-001.phpt`. Prior broad
manifest remains 150/200; smoke/COW gates unchanged.
