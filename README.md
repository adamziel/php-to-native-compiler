# PHP-to-Native Compiler

This project is an experimental PHP-to-native compiler implemented in stable
Rust. It is intentionally small and honest: implemented features are tested,
unsupported features are documented, and native code generation starts with LLVM
IR text.

The current project has two execution surfaces:

- `phpc run`, an interpreter/runtime path for the supported PHP subset.
- `phpc compile`, a narrower native-code path that emits LLVM IR or assembly for
  straight-line programs and rejects unsupported lowering with structured errors.

For exact support boundaries, read `docs/SUPPORT.md`. For design notes, read
`docs/ARCHITECTURE.md`. For the chronological proof log, read
`docs/PROGRESS.md`.

## Build

```sh
cargo build
```

## CLI

```sh
cargo run -p phpc -- run examples/hello.php
cargo run -p phpc -- compile examples/hello.php --emit-ir
cargo run -p phpc -- compile examples/hello.php --emit-asm
cargo run -p phpc -- test
```

The installed binary name is `phpc`.

### `phpc run`

`phpc run <input.php>` parses the supported PHP subset and executes it through
the Rust runtime. This is the broadest implemented path today.

The runtime is PHP-shaped rather than Rust-shaped: values are boxed, arrays keep
PHP-style integer/string keys, supported object values use runtime class
metadata, and unsupported dynamic behavior fails with stable diagnostics instead
of silently pretending to work.

### `phpc compile --emit-ir`

`phpc compile <input.php> --emit-ir` emits LLVM IR text for a smaller
straight-line subset. It currently supports scalar literals, direct scalar
variable assignment/readback, scalar `echo`/`print`, selected scalar operators,
selected folds, and a documented set of native builtin folds.

Anything outside that lowerable subset is rejected before misleading IR is
emitted. Arrays, objects, functions, general control flow, references,
copy-on-write, and broad PHP coercions remain interpreter-only or unsupported for
native lowering.

### `phpc compile --emit-asm`

`phpc compile <input.php> --emit-asm` first performs the same LLVM lowering as
`--emit-ir`. If lowering succeeds, assembly backend selection is:

1. `clang`
2. `llc`
3. `cc -S` over generated narrow-subset C, as a temporary bootstrap fallback

The C fallback keeps assembly emission usable on machines without LLVM tools. It
is not the long-term backend.

Backend behavior is covered by CLI fixtures, including backend discovery order,
missing tools, failed probes, selected-backend failures, empty or whitespace-only
assembly output, stderr handling on successful assembly, stdin handoff, and
argument validation. The tests normalize success output instead of snapshotting
platform-specific assembly text.

## Current Status

Milestone 1 is in progress. The interpreter path is intentionally ahead of the
native path; native codegen must reject unsupported programs rather than emit
incorrect native code.

### Interpreter Path

`phpc run` currently supports the documented subset of:

- literals, variables, assignment, direct `unset`, `isset`, `empty`, and null
  coalescing forms, plus bounded inline HTML output between PHP close/open tags
- scalar arithmetic, concatenation, comparisons, logical operators, bitwise
  operators, shifts, `(string)`, `(int)`, `(bool)`, `(float)`/`(double)`, and
  `(array)` casts over documented
  current value boundaries, ternaries, increments/decrements, and PHP
  error-control syntax `@expr` as a transparent runtime wrapper without
  warning/notice suppression
- `if`, loops, `switch`, `break`/`continue` including positive integer literal
  loop-depth arguments, bounded `goto`/label execution,
  `foreach`, and user functions with local scopes, bounded function-local
  `static` variables, defaults, trailing variadic parameters, returns,
  dynamic string-valued calls,
  bounded function-scope `global $name, ...;` imports for direct variables,
  bounded namespace-scoped function declarations and unqualified same-namespace
  calls with global fallback lookup,
  inert no-capture anonymous and arrow closure values,
  and recursion guarded by a fixed depth limit;
  parameter/return type syntax is accepted as metadata only, without runtime
  type enforcement
- top-level `global $name, ...;` declarations as no-op/import-compatible
  statements
- ordered arrays with integer/string keys, array literals, indexed reads/writes,
  append writes, nested direct-variable array-offset assignment expressions,
  append-at-depth assignment expressions, offset removal, array iteration, and
  simple positional statement-form `list($a, $b) = expr;` assignment over
  numeric keys
- top-level constants, namespace-scoped top-level `const` declarations in the
  current unbracketed namespace slice, selected built-in constants,
  runtime-defined constants with bounded qualified string names, simple
  interpolated runtime string names for `defined()`/`constant()`, bounded
  runtime string lookup of declared public class constants through
  `defined("ClassName::CONST")` and declared visible class constants through
  `constant("ClassName::CONST")`, and
  executable magic constants documented in the support matrix
- statement-form `throw expr;` as a bounded exception boundary: guarded throws
  can parse and be skipped, while reached throws report a stable runtime
  diagnostic without constructing exception objects or unwinding the stack
- statement-form `try`/`catch`/`finally` blocks as a bounded exception
  boundary: non-throwing try bodies execute, catch bodies are skipped without a
  thrown exception, finally bodies execute after normal try completion, and
  reached throws still report a stable runtime diagnostic before catch matching
  or unwinding exists
- narrow `require`, `require_once`, `include`, and `include_once` statement
  execution for local string paths, including constant/string-concatenated
  paths resolved relative to the current source file, included files executing
  in caller scope, and `_once` de-duplication by resolved local file
- a bounded namespace/class-name/function slice: one unbracketed named `namespace`
  declaration per file, simple top-level class `use` imports with optional
  `as` aliases, namespace-qualified class declarations, class imports for
  class-like references, `new`, `extends`, `instanceof`, static members, and
  `ClassName::class`, plus namespace-scoped function declarations and
  unqualified same-namespace calls
- declared interface metadata: top-level `interface Name {}` declarations and
  public method signatures parse, register class-like interface names, power
  `interface_exists()` and `get_declared_interfaces()`, and otherwise execute
  as declaration metadata only
- a minimal object/class slice: class metadata, `new ClassName(...)` with
  public and inherited public instance `__construct`, public instance
  property reads/writes, inherited instance property slots with
  declaring-class ownership, private same-declaring-class and protected
  same-class/child property reads/writes, `isset`/`empty`, read-modify-write,
  and null-coalescing forms, compatible public/protected inherited property
  redeclarations sharing one runtime slot,
  braced nested class declarations that register only when execution reaches
  the `class` statement,
  metadata-only built-in `Exception` and `stdClass` class seeds, including
  no-argument instantiation and user subclasses for `Exception`,
  public and same-class private instance method calls, inherited public method
  calls, protected same-class/child method calls, explicit `parent::method()`
  and `parent::__construct()` calls in instance context, narrow
  `self::method()` calls in instance context, class-method default parameters
  using `self::CONST` from the declaring method class, narrow `ClassName::class`,
  `self::class`, and `parent::class` resolution, narrow class constants
  through `ClassName::CONST`, `self::CONST`, `parent::CONST`, and late-bound
  `static::CONST` in active called-class context,
  narrow static properties through `ClassName::$prop`, `self::$prop`,
  `parent::$prop`, and late-bound `static::$prop` in active called-class
  context with direct reads/writes, compound assignment, pre/post
  increment/decrement, `isset`/`empty`, `??`, `??=`, and stable `unset(...)`
  diagnostics for PHP-forbidden static-property unset,
  dynamic static method calls through `$object::method()` and
  `$className::method()` for visible static methods,
  dynamic property-name reads/writes for existing public slots and `stdClass`
  public dynamic slots when property-name values are strings or integers,
  `clone $object` for current object values without declared `__clone`
  methods, using fresh object handles and shallow-copied property slots,
  single-parent metadata including namespaced parent names when the parent is
  already declared, object `isset` and `empty`, and selected metadata builtins,
  including declared interface metadata, declared empty-trait metadata, and
  declared unit-enum metadata
- a documented builtin subset for strings, arrays, constants, type checks,
  callability checks, bounded truthy assertions, object/class metadata, and
  debug-style output

The runtime still names unsupported zones explicitly. Examples include
references, copy-on-write, namespace forms beyond the current class-name/import,
same-namespace function, and namespace-scoped top-level constant slices,
include/require breadth beyond the current narrow local
`require`/`require_once`/`include`/`include_once` statement slice, eval,
generators, closure invocation, explicit and implicit capture binding,
callback integration, type declaration enforcement, cast
behavior outside the current `(string)`, `(int)`, `(bool)`, and
`(float)`/`(double)` slices plus the null/scalar/array `(array)` slice,
actual PHP warning/notice suppression for `@expr`,
interface inheritance/implementation enforcement, trait members and trait
composition, enum case objects/backed values/methods/interfaces,
catch matching and exception unwinding, exception objects and stack unwinding,
autoload-triggered class discovery,
array destructuring beyond simple positional statement-form `list(...)`,
constructor behavior beyond public/inherited public instance `__construct`
and explicit parent calls, broader `self::`/`static::` execution beyond the
current method, dynamic static method, class-name, class-constant, and
static-property slices,
exact PHP nested class declaration timing and fatal behavior, real
`Exception` constructor state/methods, `Throwable`, stack traces, exception
throw/catch execution,
bare namespace constant fallback reads, class-constant lookup through
`defined()`/`constant()` beyond the current declared-class/public-visibility
string-name slice, full extension constant catalogs,
complex double-quoted string interpolation such as array offsets or object
properties, heredoc/nowdoc,
visibility enforcement beyond the current public and
same-declaring-class private-property, protected-property, protected-method,
constructor, and class-constant slice, typed property compatibility and
instance property defaults,
typed or multi-declarator class constants, dynamic method names, dynamic
property creation outside `stdClass`, non-public dynamic property access,
magic property hooks, resources, and
`__clone` dispatch, clone visibility/destructor behavior, resources, and native
extension integration.

By-reference assignment syntax is accepted only as a runtime boundary for
direct variable and direct array-offset sources: guarded code can parse, but
reached `=&` assignments fail with a stable unsupported diagnostic until
reference containers and copy-on-write exist.
By-reference `foreach` value syntax is also a runtime boundary: containing code
can parse, but reached loops fail with a stable unsupported diagnostic until
aliasing, mutation ordering, and copy-on-write are implemented.

### Native Path

`phpc compile --emit-ir` and `--emit-asm` are intentionally narrower than
`phpc run`.

The current native path is focused on straight-line scalar lowering:

- scalar/null literals, direct scalar assignments, direct reads, `echo`, and
  `print`
- selected `isset` and `empty` folds over the current static variable map
- selected scalar arithmetic, bitwise, comparison, logical, unary, ternary, and
  string-concatenation forms when operands are already lowerable and the result
  semantics are proven
- selected direct builtin folds such as scalar type checks, `strlen`, selected
  callability/function-existence checks, selected metadata-existence checks, and
  selected constant-existence checks

Native lowering rejects arrays, array destructuring, objects, user functions,
closure values,
include/require, broad control flow, exception boundaries, scalar casts,
mutation forms that require symbol-table effects, dynamic calls, `assert()`,
runtime constant tables, PHP-wide coercions,
references, copy-on-write, linking, and execution until those semantics exist
in generated code.

### Tests And Fixtures

Fixture tests live under `tests/fixtures`. The test runner strips one final
newline from `.stdout` and `.stderr` fixtures so expected-output files remain
editor-friendly; use a blank final line when expected program output should
include a trailing newline.

Fixtures with a sibling `.phpc-only` marker are still tested by `phpc`, but are
skipped by optional system PHP comparison when the project intentionally reports
different diagnostics.

Use these commands while developing:

```sh
cargo test --workspace
cargo run -p phpc -- test
cargo run -p phpc -- test --compare-php
```

For the exhaustive support matrix, see `docs/SUPPORT.md`.

## Operations

Operational automation lives in `docs/OPERATIONS.md`.

- `tools/run-tests.sh` runs the full project test suite.
- `tools/checkpoint.sh "message"` runs the suite and commits all current changes
  only if tests pass.
- `tools/codex-loop.sh` runs a bounded Codex supervisor loop when
  `CODEX_RUNNER` is set.
- `tools/codex-yolo-forever.sh` runs an infinite unattended yolo loop with
  durable memory in `docs/LOOP_MEMORY.md`.
