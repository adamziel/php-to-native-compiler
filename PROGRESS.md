# PTN Progress

Refresh: 2026-06-10T23:14Z
Measured: `ptn-ock` rebased after `ptn-4n1` on current `origin/master`;
addslashes/stripslashes native PHPT-shape coverage, cslashes PHPT shape,
double-quoted ASCII octal/hex escapes, declared class/object method dispatch,
object `foreach` reducers, and callback/COW/directory evidence.

## Test Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native compiled PHP snippets | 375 | 375 | 0 |
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
| PHPT callback manifest | 3 | 3 | 0 |

## Already Ported

Lexer/parser, AST/IR/C backend, boxed values, variables/constants,
double-quoted ASCII octal/hex escapes, string/math/type internals, ordered
arrays, `foreach`, source-spanned non-array `foreach` warnings,
array/string COW, references, by-reference params/foreach, array dimensions,
temporaries, recursive/user functions, anonymous function values, magic
constants, `func_*`, `print_r`, scalar diagnostics, array union `+`, scalar
type hints, by-reference returns, `count()`, `??`, assignment expressions,
expression-level `@`, directory/file predicates, array-path snapshots,
selected array/string internals including `addcslashes()`/`stripcslashes()`,
`addslashes()`/`stripslashes()`,
dynamic lvalue-reference calls, list/append assignment expressions,
static method callable values, `array_reduce()` callback dispatch,
`array_count_values()`, `new stdClass` dynamic properties, declared class
method metadata, `new DeclaredClass()` object shells, direct declared method
calls, `$this` binding, object/static method callable arrays, and object
`foreach` over public dynamic properties including live additions and
by-reference value binding.

## Still Needed

The remaining focused COW PHPT gap is Closure `use` captures for the
`array_walk()`/`$GLOBALS` row. Broader bounded-PHPT gaps are constructors,
declared properties, visibility/inheritance/interfaces/traits, static
properties, magic methods, object/property compound lvalues, destructors,
exceptions, reflection, unsupported array/string internals, 64-bit operator
exactness, destructuring `foreach` diagnostics/semantics, scalar offset-lvalue
fatal parity, and broader file APIs.

## Verification

Commands: `cargo fmt --check`; `cargo build --bin phpc`; focused addslashes,
cslashes, and object `foreach` native tests/reducers; `cargo test`. Exact
addslashes PHPT rows are blocked locally by missing
`/home/claude/php-src-phpt/run-tests.php` (`ptn-jmq`).
