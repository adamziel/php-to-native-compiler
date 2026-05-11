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
- arithmetic: `+`, `-`, `*`, `/` with scalar coercions for `null`, booleans,
  integers, floats, and well-formed numeric strings
- unary `-` and `!`
- string concatenation: `.`
- comparisons: `==`, `!=`, `<`, `<=`, `>`, `>=` across the current scalar
  values (`null`, booleans, integers, floats, and strings)
- `if` / `else`
- `while`
- function declarations
- positional function calls
- `return`
- isolated local scopes for user-function calls; parameters and function-local
  assignments can shadow global names without mutating them
- short array literals: `[]`, `[value]`, and `[key => value]` for the currently
  supported expression subset
- ordered arrays with integer and string keys
- array indexed reads: `$array[$key]` for existing integer/string keyed array
  entries
- direct variable array writes: `$array[$key] = ...` and `$array[] = ...`
- builtins for the documented scalar/array subset: `strlen`, `isset`, `count`,
  `var_dump`, and `print_r`
- structured runtime errors for undefined variables, arity mismatches,
  unsupported calls, division by zero, non-numeric string arithmetic, and
  unsupported array keys, undefined array keys, invalid array access, and
  unsupported `global` declarations

## Partially Supported

- Arrays: array values preserve insertion order and normalize string keys that
  are valid decimal integers, such as `"2"` and `"-2"`, to integer keys.
  Strings with leading zeroes, leading `+`, decimal points, exponent notation,
  or integer overflow stay string keys. Duplicate normalized keys update the
  existing slot without moving it. Keyless literal entries and `$array[] = ...`
  writes append at the next non-negative integer key. Direct variable offset
  writes update existing array variables, and writes to undefined or `null`
  variables materialize an array. Existing-key reads return the stored value.
  Missing-key reads fail with a stable runtime error instead of PHP's
  warning-and-`null` recovery. Array truthiness, `count`, `print_r`, and
  `var_dump` are implemented for this ordered value model.
- Type coercion: scalar arithmetic supports `null`, booleans, integers, floats,
  and well-formed numeric strings with optional sign, decimal point, exponent,
  and surrounding ASCII whitespace. Non-numeric strings fail with a stable
  runtime error. Truthiness is implemented for current scalar values.
- Scalar comparisons: loose equality and relational operators are implemented
  for the current scalar values using PHP 8-style behavior for booleans,
  numeric strings, non-numeric strings, empty strings, `null`, integers, and
  floats. This is not PHP's full comparison matrix: strict identity operators,
  arrays, objects, resources, and edge cases around `NAN`/`INF` and
  PHP-version-specific float string precision are not covered.
- Runtime errors: diagnostics have stable messages and source locations, but
  they are not PHP `Throwable` objects and there is no warning/notice recovery
  mode yet. Representative runtime errors are covered by committed `phpc run`
  CLI snapshots that record exit code, stdout, and stderr for undefined
  variables, user-function arity mismatches, unsupported scalar `count()` calls,
  unsupported array keys, undefined array keys, division by zero, and
  non-numeric string arithmetic.
- Native codegen: LLVM IR/assembly supports only straight-line echo/assignment
  with statically lowerable scalar expressions. Arrays, array indexing, and
  array assignment are rejected with explicit codegen errors.
- Assembly emission: uses LLVM tools when available, with a temporary `cc -S`
  C fallback for the same narrow lowerable subset.
- Function calls: user-defined positional calls are supported in `phpc run` with
  exact arity only. Each call gets a fresh local scope. Parameters and local
  assignments shadow global variables without mutating them, and functions do
  not import top-level variables implicitly. `global` declarations parse but
  fail with a stable runtime error because global scope imports are not
  implemented. Default values and variadics are not implemented.
- Builtins: `strlen`, `isset`, `count`, `var_dump`, and `print_r` cover the
  documented scalar/array subset only. `strlen` remains scalar-only and rejects
  arrays. `count` accepts arrays only. `isset` supports direct variable
  operands and can safely check undefined variables; array offsets, complex
  lvalues, and expression operands are unsupported. Object formatting and PHP's
  complete warning behavior are not implemented.
- Scalar arithmetic gaps: leading numeric strings with trailing non-numeric
  characters, such as `"10 apples"`, are rejected instead of warning and
  continuing with the leading number. PHP's warning/notice recovery mode,
  locale-sensitive numeric parsing, and exact integer-overflow promotion rules
  are not implemented.
- Array gaps: nested indexed writes, complex assignment lvalues, `$array[]` as
  a read expression, string offset access, `unset`, `foreach`, long `array()`
  syntax, destructuring, spread, references, copy-on-write containers, and
  object/resource keys are not implemented. Array keys are currently limited to
  values that evaluate to integers or strings; PHP's boolean, null, float,
  object, and resource key coercions are rejected with a stable runtime error.
  Writes to existing non-array scalar variables other than `null` are rejected
  instead of following PHP's full automatic conversion behavior. Negative-key
  auto-index behavior is not claimed beyond the current non-negative allocator.

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

- nested/complex array assignment lvalues
- string offset access
- references
- objects/classes
- includes/requires
- variable variables
- `global` declarations / importing top-level variables into function scope
- dynamic function calls
- `eval`
- namespaces
- closures
- exceptions
- traits/interfaces
- generators
- attributes
- PHP standard library beyond documented builtins
- Zend extension loading
- WordPress compatibility
- PHP's warning-and-continue behavior for undefined variables; plain reads fail
  with a runtime error in the current subset, while `isset($name)` remains the
  supported presence check
- PHP `Throwable`/`Error` objects, stack traces, recoverable warnings, notices,
  and user error handlers
- Preserving partial stdout emitted before a runtime failure; the current
  runtime-error path aborts the command with a diagnostic instead of modeling
  PHP's output buffering and fatal-error behavior

Unsupported code should fail with an explicit parse, runtime, or codegen error.
