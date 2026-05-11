# Support Matrix

## Supported in `phpc run`

- PHP opening tag `<?php`
- `echo` statements with one or more comma-separated expressions
- `print` statements
- integer literals
- float literals
- single-quoted and double-quoted string literals with basic escapes
- `null`, `true`, and `false`
- variables
- assignment statements
- arithmetic: `+`, `-`, `*`, `/`
- unary `-` and `!`
- string concatenation: `.`
- comparisons: `==`, `!=`, `<`, `<=`, `>`, `>=`
- `if` / `else`
- `while`
- function declarations
- positional function calls
- `return`
- scalar builtins: `strlen`, `isset`, `var_dump`, and `print_r`
- structured runtime errors for undefined variables, arity mismatches,
  unsupported calls, and division by zero

## Partially Supported

- Type coercion: implemented for scalar arithmetic and truthiness only.
- Equality: scalar equality is implemented; PHP's full comparison matrix is not.
- Runtime errors: diagnostics have stable messages and source locations, but
  they are not PHP `Throwable` objects and there is no warning/notice recovery
  mode yet.
- Native codegen: LLVM IR/assembly supports only straight-line echo/assignment
  with statically lowerable scalar expressions.
- Assembly emission: uses LLVM tools when available, with a temporary `cc -S`
  C fallback for the same narrow lowerable subset.
- Function calls: user-defined positional calls are supported in `phpc run` with
  exact arity only. Default values and variadics are not implemented.
- Builtins: `strlen`, `isset`, `var_dump`, and `print_r` are implemented only
  for current scalar values. `isset` supports direct variable operands and can
  safely check undefined variables; complex lvalues and expression operands are
  unsupported. Array/object formatting and PHP's complete warning behavior are
  not implemented.

## Test Support

- `phpc test [fixture-dir]` validates fixture programs against committed
  `.stdout`, `.stderr`, and `.exit` files.
- `phpc test --compare-php [fixture-dir]` also runs each fixture with system
  `php`, when available, and compares stdout, stderr, and exit code against
  `phpc run` behavior. If `php` is not installed, the comparison is skipped and
  committed fixture expectations still run.
- System PHP comparison is a Milestone 2 test aid for supported `phpc run`
  fixtures only. It does not normalize PHP-version-specific diagnostics, INI
  settings, loaded extensions, locale, line ending differences, or unsupported
  dynamic PHP features.
- A fixture can opt out of system PHP comparison with a sibling `.phpc-only`
  marker file when the committed `phpc` behavior intentionally differs from
  system PHP, such as stable project-specific runtime diagnostics.

## Unsupported

- arrays
- references
- objects/classes
- includes/requires
- variable variables
- dynamic function calls
- `eval`
- namespaces
- closures
- exceptions
- traits/interfaces
- generators
- attributes
- PHP standard library beyond documented scalar builtins
- Zend extension loading
- WordPress compatibility
- PHP's warning-and-continue behavior for undefined variables; plain reads fail
  with a runtime error in the current subset, while `isset($name)` remains the
  supported presence check
- PHP `Throwable`/`Error` objects, stack traces, recoverable warnings, notices,
  and user error handlers

Unsupported code should fail with an explicit parse, runtime, or codegen error.
