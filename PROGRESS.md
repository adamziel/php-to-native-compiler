# PTN Progress

Refresh: 2026-06-10T15:30Z
Measured: `ptn-9s5` rebased on `origin/master@01da7022`; `intval()` string
conversion, `array_reduce()` refcounts, nested string offset-lvalue fatals,
and non-array `foreach` source-path diagnostics.

## Test Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native compiled PHP snippets | 370 | 370 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 200 | 152 | 48 |
| PHPT Zend rows | 76 | 68 | 8 |
| PHPT ext/standard rows | 77 | 48 | 29 |
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
The `array_reduce_accumulator_refcount.phpt` row now passes; the documented
remaining COW PHPT failure is `bug69068_2.phpt`.

## Already Ported

Lexer/parser, AST/IR/C backend, boxed values, variables/constants,
string/math/type internals, ordered arrays, `foreach`, source-spanned
non-array `foreach` warnings, cursors, numeric keys, payload refcounts,
array/string COW, references, by-reference params/foreach, array dimensions,
temporaries, recursive/user functions, anonymous function values for direct
dynamic calls and internal callbacks, magic constants, `func_*`, `print_r`,
binary strings, string offsets, nested string offset-lvalue fatals, scalar
diagnostics, array literal references, array union `+`, scalar type hints,
by-reference return boundaries, `count()`, `??`, assignment expressions,
expression-level `@`, file APIs including recursive `mkdir()` plus directory
predicates, array-path snapshots,
`array_sum()`/`strtr()`/`in_array()`, recursive array merge/replace,
`debug_zval_dump()`, dynamic lvalue-reference calls, append/list assignment
expressions, nested same-array reference lvalues, direct-variable and
offset-form `??=`, grouped reference targets, `array_fill_keys()`,
string-callable `call_user_func()`, string-callable/null `array_map()`,
`intval()` base-prefix and range-saturating string conversion, named
`array_walk()` global-array rebinding, public static methods registered as
`Class::method` callables, generic `array_reduce()` callback dispatch with
accumulator debug refcounts, and `new stdClass` boxed objects with public
dynamic property reads/writes shared through object aliases.

## Still Needed

The remaining focused COW PHPT gap is Closure `use` captures for the
`array_walk()`/`$GLOBALS` row. Broader bounded-PHPT gaps are full class
declarations/metadata, instance methods, visibility/inheritance/static
properties/magic methods, non-static method callable values, unsupported
array/string internals, 64-bit operator exactness, object/destructuring
foreach diagnostics, object/property compound lvalues, scalar offset-lvalue
remaining edges, and broader file APIs.

## Verification

Commands: `cargo fmt --check`; `cargo build --bin phpc`; `cargo test`
under repo-local `TMPDIR`; focused nested string offset-lvalue native test;
focused `Zend/tests/offset_assign.phpt`; `tools/run-native-smoke-matrix.sh`;
`tools/run-post-merge-cow-gate.sh`.
