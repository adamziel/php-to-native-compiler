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

## Partially Supported

- Type coercion: implemented for scalar arithmetic and truthiness only.
- Equality: scalar equality is implemented; PHP's full comparison matrix is not.
- Native codegen: LLVM IR/assembly supports only straight-line echo/assignment
  with statically lowerable scalar expressions.
- Assembly emission: uses LLVM tools when available, with a temporary `cc -S`
  C fallback for the same narrow lowerable subset.
- Function calls: user-defined positional calls are supported in `phpc run` with
  exact arity only. Default values and variadics are not implemented.

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
- PHP standard library beyond the runtime helpers already used internally
- Zend extension loading
- WordPress compatibility

Unsupported code should fail with an explicit parse, runtime, or codegen error.
