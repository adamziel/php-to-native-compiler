# PHP-to-Native Compiler

This project is an experimental PHP-to-native compiler implemented in stable Rust.
It is intentionally small and honest: implemented features are tested, unsupported
features are documented, and native code generation starts with LLVM IR text.

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

`--emit-asm` prefers `clang` or `llc` for LLVM IR assembly emission. If neither
tool exists, it currently falls back to generating equivalent narrow-subset C and
running `cc -S`; this is a real assembly path, but it is documented as a
temporary bootstrap fallback rather than the long-term backend.

## Current Status

Milestone 1 is in progress. The interpreter/runtime path supports a small PHP
subset:

- `echo`
- integer, float, and string literals
- static variables and assignment through per-scope symbol tables
- `+`, `-`, `*`, `/`, `.`
- loose scalar comparisons used by control flow
- `if` / `else`
- `while`
- function declarations, positional calls with trailing default parameter
  values, recursive calls up to the documented guard, `return`, and isolated
  local scopes for user-function calls
- dynamic function calls through string-valued expressions that resolve to the
  documented callable builtin subset or user-defined functions
- top-level class declarations registered as metadata, with property and method
  metadata for the documented subset
- minimal object instantiation with `new ClassName()` for declared classes that
  do not define constructors; public instance properties can be read and
  written by static property name and checked with `isset($object->name)`
- short array literals with integer/string keys
- array indexed reads, indexed writes, and append writes for the documented
  direct-variable array subset
- builtins for the documented scalar/array/object subset: `strlen`, `isset`,
  `count`, `var_dump`, and `print_r`
- stable runtime diagnostics for the currently covered runtime errors,
  including unresolved or non-string dynamic function calls, unsupported
  `global` declarations, and runaway recursion
- stable lex/parse diagnostics for unsupported dynamic/function features
  including variable variables, include/require/eval constructs,
  namespace and `use` declarations, variadics, references, closures, named
  arguments, `declare(strict_types=1)`, object method calls, dynamic property
  names, anonymous classes, and unsupported class forms, static member access,
  and class constants

`php_runtime` also contains a tested object/class metadata registry and minimal
object values. `phpc run` can instantiate declared constructor-free classes,
read/write public instance properties by static name, and check those public
properties with `isset`, but constructors, `$this`, method dispatch, dynamic
property names, visibility enforcement for non-public properties, object handle
identity, static property storage, static method dispatch, class constants, and
native object lowering are not supported yet.

LLVM IR emission currently supports a smaller straight-line subset and rejects
unsupported programs with a structured codegen error.

Fixture tests live under `tests/fixtures`. For editor-friendly expected-output
files, the test runner strips one final newline from `.stdout` and `.stderr`
fixtures; use a blank final line when the expected program output should include
a trailing newline. Fixtures with a sibling `.phpc-only` marker are still tested
by `phpc`, but are skipped by optional system PHP comparison when the project
intentionally reports different diagnostics.

See `docs/SUPPORT.md` for the detailed support matrix.

## Operations

Operational automation lives in `docs/OPERATIONS.md`.

- `tools/run-tests.sh` runs the full project test suite.
- `tools/checkpoint.sh "message"` runs the suite and commits all current changes
  only if tests pass.
- `tools/codex-loop.sh` runs a bounded Codex supervisor loop when
  `CODEX_RUNNER` is set.
- `tools/codex-yolo-forever.sh` runs an infinite unattended yolo loop with
  durable memory in `docs/LOOP_MEMORY.md`.
