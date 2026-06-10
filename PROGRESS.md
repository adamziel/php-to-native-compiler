# PTN Progress

Refresh: 2026-06-11T00:40Z
Measured: `ptn-l9c` rebased on current `origin/master` after prior queue
merges; expression-form `print`, declared static-property reducers, inherited
public instance-method reducers, public property `??=`, nested string-offset
unset exception parity, addslashes/stripslashes, braced interpolation, scalar
offset-lvalue reducers, object `foreach`, declared class metadata,
object-method callable dispatch, method-scope magic constants, cslashes, and
callback/COW/directory evidence.

## Test Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native compiled PHP snippets | 382 | 382 | 0 |
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
double-quoted ASCII octal/hex escapes, direct/braced variable and array-offset
interpolation, string/math/type internals, ordered arrays, `foreach`,
non-array `foreach` warnings, array/string COW, references, by-reference
params/foreach, array dimensions, temporaries, recursive/user functions,
anonymous function values, magic constants, `func_*`, `print_r`, scalar
diagnostics, array union `+`, scalar type hints, by-reference returns,
`count()`, `??`, assignment expressions, expression-form `print`,
expression-level `@`, directory/file predicates, array-path snapshots,
array/string internals including `addcslashes()`/`stripcslashes()`,
`addslashes()`/`stripslashes()`, dynamic lvalue-reference calls, list/append
assignment expressions, static method callable values, `array_reduce()`
callback dispatch, `array_count_values()`, `new stdClass` dynamic properties,
declared class method metadata, `new DeclaredClass()` object shells, direct
declared method calls, `$this` binding, object/static method callable arrays,
object `foreach` over public dynamic properties with live additions and
by-reference binding, scalar array-lvalue write/reference fatals for
non-convertible scalars with false-to-array deprecation, declared class/method
metadata through `class_exists()` and `method_exists()`, callable-only object
method dispatch through internal callbacks, method-scope
`__FUNCTION__`/`__METHOD__`/`__CLASS__`, nested string-offset unset errors,
public property `??=` with quiet lookup, lazy RHS evaluation, PHP receiver
re-evaluation order, inherited public instance methods, and static properties
with constant defaults, direct/self read-write persistence, and undeclared
static-property `Error` diagnostics.

## Still Needed

The remaining focused COW PHPT gap is Closure `use` captures for the
`array_walk()`/`$GLOBALS` row. Broader bounded-PHPT gaps are constructors,
declared properties, non-public visibility/interfaces/traits, broader
inheritance, magic methods, property compound operators beyond `??=`,
static-property compound/null-coalescing lvalues, destructors, exceptions,
broader magic constants for traits/namespaces/includes/eval, reflection,
unsupported array/string internals, 64-bit operator exactness, destructuring
`foreach` diagnostics/semantics, remaining string sub-path scalar
offset-lvalue parity, and broader file APIs.

## Verification

Commands: focused `print` parser/native reducers; `cargo check`; `cargo test
static_property --test compile_native`; `cargo fmt --check`; `cargo build
--bin phpc`; focused class/method, property `??=`, addslashes, cslashes,
object `foreach`, scalar offset, and declared class metadata native
tests/reducers; `cargo test callable`; focused catchable exception-message
tests; focused `cargo test interpolation`; exact `add-and-stripcslashes.phpt`;
`cargo test`. PHPT runners resolve php-src via `PHP_SRC_PHPT`,
`/home/claude/php-src-phpt`, or `.runtime/php-src-phpt`.
