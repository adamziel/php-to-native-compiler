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
  coalescing forms
- scalar arithmetic, concatenation, comparisons, logical operators, bitwise
  operators, shifts, ternaries, and increments/decrements over documented value
  boundaries
- `if`, loops, `switch`, `break`, `continue`, `foreach`, and user functions with
  local scopes, defaults, returns, dynamic string-valued calls, and recursion
  guarded by a fixed depth limit
- ordered arrays with integer/string keys, array literals, indexed reads/writes,
  append writes, offset removal, and array iteration
- top-level constants, selected built-in constants, runtime-defined constants,
  and executable magic constants documented in the support matrix
- a minimal object/class slice: class metadata, `new ClassName(...)` with
  public and inherited public instance `__construct`, public instance
  property reads/writes, inherited instance property slots with
  declaring-class ownership, private same-declaring-class and protected
  same-class/child property reads/writes, `isset`/`empty`, read-modify-write,
  and null-coalescing forms, compatible public/protected inherited property
  redeclarations sharing one runtime slot,
  public and same-class private instance method calls, inherited public method
  calls, protected same-class/child method calls, explicit `parent::method()`
  and `parent::__construct()` calls in instance context, narrow
  `self::method()` calls in instance context, narrow `ClassName::class`,
  `self::class`, and `parent::class` resolution, narrow class constants
  through `ClassName::CONST`, `self::CONST`, and `parent::CONST`,
  narrow static properties through `ClassName::$prop`, `self::$prop`, and
  `parent::$prop` with direct reads/writes, compound assignment,
  pre/post increment/decrement, `isset`/`empty`, `??`, and `??=`,
  single-parent metadata, object `isset` and `empty`, and selected metadata
  builtins
- a documented builtin subset for strings, arrays, constants, type checks,
  callability checks, object/class metadata, and debug-style output

The runtime still names unsupported zones explicitly. Examples include
references, copy-on-write, namespaces/imports, includes/requires, eval,
generators, closures, typed declarations, interfaces, traits, enums,
constructor behavior beyond public/inherited public instance `__construct`
and explicit parent calls, broader `self::`, all `static::` execution and late
static binding, visibility enforcement beyond the current public and
same-declaring-class private-property, protected-property, protected-method,
constructor, and class-constant slice, typed/default property compatibility,
typed or multi-declarator class constants, static methods, `static::`
late-bound property access, dynamic method/property names, resources, and
native extension integration.

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

Native lowering rejects arrays, objects, user functions, broad control flow,
mutation forms that require symbol-table effects, dynamic calls, runtime
constant tables, PHP-wide coercions, references, copy-on-write, linking, and
execution until those semantics exist in generated code.

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
