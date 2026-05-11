# Support Matrix

## Supported in `phpc run`

- PHP opening tag `<?php`
- `echo` statements with one or more comma-separated expressions
- `print` statements
- integer literals
- float literals
- single-quoted and double-quoted string literals with basic escapes
- `null`, `true`, and `false`
- static variables backed by per-scope materialized symbol tables
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
- dynamic function calls through string-valued expressions that resolve to the
  documented callable builtin subset or user-defined functions
- trailing default parameter values for user functions over the documented
  constant-expression subset
- recursive user-function calls up to a fixed 128-frame user-function call-depth
  guard
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
  undefined functions, non-string dynamic function callees, unsupported array
  keys, undefined array keys, invalid array access, unsupported `global`
  declarations, and runaway user-function recursion
- explicit parse diagnostics for unsupported function syntax: variadic
  parameters, variadic argument unpacking, reference parameters/returns,
  reference expressions, anonymous functions, arrow functions, named arguments,
  and `declare(strict_types=1)`
- explicit parse diagnostics for unsupported include/require syntax:
  `include`, `include_once`, `require`, and `require_once`
- explicit lex diagnostics for unsupported variable-variable syntax such as
  `$$name` and `${...}`

## Partially Supported

- Variable storage: top-level code and each user-function call use materialized
  symbol tables keyed by variable name. Current static variable reads, writes,
  `isset($name)`, parameter binding, default-parameter evaluation, and direct
  array writes route through that symbol table path. Runtime lookup by a value
  computed from PHP code is not implemented yet, so variable variables still do
  not execute.
- Include/require: `include`, `include_once`, `require`, and `require_once`
  are reserved by the lexer/parser and rejected with stable parse diagnostics.
  The planned first executable slice resolves string paths relative to the
  including file, executes included files in caller scope, and tracks `_once`
  files by canonical absolute path when possible. Execution, include-path
  lookup, current-working-directory fallback, stream wrappers, URL includes,
  `phar://`, opcache behavior, autoload interaction, and PHP's exact
  warning-vs-fatal recovery behavior are not implemented.
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
  unsupported array keys, undefined array keys, unresolved dynamic function
  names, non-string dynamic function callees, division by zero, non-numeric
  string arithmetic, and runaway user-function recursion.
- Native codegen: LLVM IR/assembly supports only straight-line echo/assignment
  with statically lowerable scalar expressions. Arrays, array indexing, and
  array assignment are rejected with explicit codegen errors.
- Assembly emission: uses LLVM tools when available, with a temporary `cc -S`
  C fallback for the same narrow lowerable subset.
- Function calls: user-defined positional calls are supported in `phpc run`.
  Dynamic function calls are supported only when the callee expression evaluates
  to a string that case-insensitively resolves to a user-defined function or to
  one of the documented callable builtins: `strlen`, `count`, `var_dump`, or
  `print_r`. Unresolved names fail with a stable undefined-function runtime
  error, and non-string callees fail with a stable unsupported-call runtime
  error. Required parameters and trailing default parameter values are
  supported. Defaults may use the current constant-expression subset: `null`,
  booleans, integers, floats, strings, short arrays with supported keys, unary
  expressions, and binary expressions over those values. Omitted arguments bind
  to their defaults; calls outside the supported required-to-total arity range
  fail with a stable arity diagnostic. Each user-function call gets a fresh
  local scope. Parameters and local assignments shadow global variables without
  mutating them, and functions do not import top-level variables implicitly.
  `global` declarations parse but fail with a stable runtime error because
  global scope imports are not implemented. Recursive user-function calls are
  supported until the fixed 128-frame user-function call-depth guard is reached.
  That guard is a project-specific runtime diagnostic, not PHP's native stack or
  memory exhaustion behavior; it is not configurable and does not produce stack
  traces. Non-constant defaults such as variables, calls, dynamic calls, and
  indexed reads are rejected by the parser. Required parameters after default
  parameters are also rejected instead of modeling PHP's deprecation and
  implicit-required behavior. Variadic parameters and argument unpacking,
  reference parameters/returns, reference expressions, anonymous functions,
  arrow functions, named arguments, and `declare(strict_types=1)` are rejected
  with stable parse diagnostics. The project does not implement any runtime
  semantics for those features yet. Parameter type declarations, return type
  declarations, nullable/union/intersection types, static locals, magic function
  constants, array callables, object/method callables, first-class callable
  syntax, `call_user_func`, namespace-qualified callable resolution, and
  autoload interaction are also unsupported.
- Builtins: `strlen`, `isset`, `count`, `var_dump`, and `print_r` cover the
  documented scalar/array subset only. `strlen` remains scalar-only and rejects
  arrays. `count` accepts arrays only. `isset` supports direct variable
  operands and can safely check undefined variables; array offsets, complex
  lvalues, and expression operands are unsupported. Because `isset` is modeled
  as a special static form, it is not available through dynamic function
  lookup. Object formatting and PHP's complete warning behavior are not
  implemented.
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
- include/require execution; `include`, `include_once`, `require`, and
  `require_once` currently fail with stable parse diagnostics
- variable variables; `$$name` and `${...}` are rejected with a stable lex
  diagnostic rather than executed
- `global` declarations / importing top-level variables into function scope
- default parameter values outside the documented constant-expression subset
- required parameters after default parameters
- variadic parameters and variadic argument unpacking
- reference parameters, reference returns, reference assignments, and
  by-reference calls
- dynamic callables outside the string function-name subset, including array
  callables, object/method callables, first-class callable syntax,
  `call_user_func`, and namespace/autoload-aware callable resolution
- named arguments
- `declare(strict_types=1)` and PHP type declaration enforcement
- `eval`
- namespaces
- closures and arrow functions
- configurable recursion/call-stack limits matching PHP deployments
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
