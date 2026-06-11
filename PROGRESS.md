# PTN Progress

Refresh: 2026-06-11T11:43Z
Measured: `ptn-hfpk` rebased after current `origin/master`; array null-offset
diagnostic routing, Closure captures, expression-form `print`, static
properties, inherited methods, property `??=`, loose object equality in
`switch`, nested string-offset unset, braced interpolation, branch-condition
assignment and named-argument reducers, scalar offset reducers, object
`foreach`, class metadata, method callables/magic constants, cslashes, scalar
variable variables, include return helpers, array/object type predicates,
runtime `define()` legacy flag parity, shared `error_reporting()` filtering,
and COW evidence.

## Test Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native compiled PHP snippets | 393 | 393 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 200 | 153 | 47 |
| PHPT Zend rows | 76 | 68 | 8 |
| PHPT ext/standard rows | 77 | 49 | 28 |
| PHPT tests/basic+func+lang | 45 | 34 | 11 |
| PHPT other rows | 2 | 2 | 0 |
| COW contract spec tests | 7 | 7 | 0 |
| Focused COW reducer snippets | 47 | 47 | 0 |
| Recursive reference diagnostics | 4 | 4 | 0 |
| COW oracle suite | 22 | 22 | 0 |
| By-reference foreach COW oracle | 11 | 11 | 0 |
| Mutating-internal COW matrix | 14 | 14 | 0 |
| Post-merge COW gate | 25 | 25 | 0 |
| PHPT COW manifest | 29 | 28 | 1 |
| PHPT callback manifest | 3 | 3 | 0 |

## Already Ported

Lexer/parser, AST/IR/C backend, boxed values, variables/constants, strings,
math/type internals, ordered arrays, `foreach`, COW/references/by-reference
params, array dimensions, recursive/user functions, Closure captures, magic
constants, `func_*`, `print_r`, scalar diagnostics, array null-offset
diagnostics routed through `@` suppression, array union `+`, scalar type hints,
by-reference returns, top-level variadic parameter packing, trailing scalar
default function parameters, `count()`, `??`, assignment expressions including
direct-variable compound branch/loop conditions, direct user-function named
arguments, expression-form `print`, expression-level `@`, directory/file
predicates, array-path snapshots, selected array/string internals,
lvalue-reference calls, list/append assignment, scalar variable-variable
reads/ordinary assignments with unsupported-name diagnostics, method callables,
`array_reduce()`/`array_walk()`, `array_count_values()`, `stdClass`, declared
class/object method metadata and calls, `$this`, object `foreach`, scalar
array-lvalue fatals with false-to-array deprecation, class/method predicates,
static properties, property `??=`, method-scope magic constants, inherited
public methods, loose object equality in braced `switch`, nested string-offset
unset errors, and compile-time-resolved statement-only `include`/`require`
return propagation, and `define()`'s legacy case-insensitive flag warning with
case-sensitive runtime constants, and `error_reporting()` mask filtering for
modeled shared warning/deprecation/notice emitters.

## Still Needed

PHPT confirmation for the `array_walk()`/`$GLOBALS` Closure row is pending.
Broader gaps are constructors, declared properties, non-public visibility,
interfaces/traits, broader inheritance, magic methods, property compounds
beyond `??=`, static-property compound/null-coalescing lvalues, destructors,
exceptions, broader magic constants, reflection, unsupported internals,
64-bit operator exactness, destructuring `foreach`, remaining string sub-path
scalar offset-lvalue parity, dynamic-variable array-offset lvalues, dynamic
include/include_once behavior, and broader file APIs.

## Verification

Commands: focused Closure/`print`/interpolation/branch-condition assignment/
named-argument/loose-object switch/addslashes/cslashes/object `foreach`/scalar
offset/callable/metadata/scalar variable-variable/focused include parser and
native reducers/array-object-predicate tests; `cargo check`;
`cargo test static_property --test compile_native`; runtime `define()` legacy
flag ordering reducer; `cargo fmt --check`; `cargo build --bin phpc`; exact
`array_null_offset_deprecation.phpt` reducer; focused `error_reporting()`
suppression reducer; exact `array_count_values.phpt` row; `cargo test`.
PHPT runners resolve php-src via `PHP_SRC_PHPT`, `/home/claude/php-src-phpt`, or
`.runtime/php-src-phpt`.
