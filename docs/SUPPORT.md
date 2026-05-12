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
- `break;` and `continue;` for the innermost currently executing `while` loop
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
- top-level class declarations registered into the runtime metadata table:
  `class Name { ... }` with property names, method names, visibility, and static
  flags for the documented subset
- minimal object instantiation with `new ClassName()` for declared classes that
  do not define `__construct` and are called without constructor arguments
- public instance property reads and direct-variable writes by static property
  name: `$object->name` and `$object->name = ...`
- `isset($object->name)` for direct public instance property operands on direct
  object variables
- short array literals: `[]`, `[value]`, and `[key => value]` for the currently
  supported expression subset
- ordered arrays with integer and string keys
- array indexed reads: `$array[$key]` for existing integer/string keyed array
  entries
- direct variable array writes: `$array[$key] = ...` and `$array[] = ...`
- `isset($array[$key])` for direct array-variable offset operands over the
  current integer/string key subset
- builtins for the documented subset: `strlen`, `isset`, `count`, `var_dump`,
  and `print_r`; `print_r` can render current minimal object values
- structured runtime errors for undefined variables, arity mismatches,
  unsupported calls, division by zero, non-numeric string arithmetic, and
  undefined functions, non-string dynamic function callees, unsupported array
  keys, undefined array keys, invalid array access, unsupported `global`
  declarations, duplicate class/member metadata, undefined classes,
  unsupported object instantiation, undefined object properties, invalid
  property targets, unsupported non-public property access, object-to-string
  conversion, invalid `break`/`continue` outside a loop, and runaway
  user-function recursion
- explicit parse diagnostics for unsupported function syntax: variadic
  parameters, variadic argument unpacking, reference parameters/returns,
  reference expressions, anonymous functions, arrow functions, named arguments,
  and `declare(strict_types=1)`
- explicit parse diagnostics for unsupported include/require syntax:
  `include`, `include_once`, `require`, and `require_once`
- explicit parse diagnostics for unsupported direct `eval(...)` syntax
- explicit parse diagnostics for unsupported namespace and top-level `use`
  declaration syntax
- explicit parse diagnostics for unsupported namespace-qualified function and
  class names such as `App\fn()` and `new App\Box()`
- explicit parse diagnostics for unsupported long `array(...)` literal syntax
- explicit parse diagnostics for unsupported `unset(...)` syntax
- explicit parse diagnostics for unsupported `foreach (...)` syntax
- explicit parse diagnostics for unsupported `for (...)` syntax
- explicit parse diagnostics for unsupported `do ... while` syntax
- explicit parse diagnostics for unsupported `switch (...)` syntax
- explicit parse diagnostics for unsupported `break`/`continue` loop-depth
  arguments
- explicit parse diagnostics for unsupported object/class syntax: nested class
  declarations, inheritance, interface implementation, typed/default/multiple
  property declarations, anonymous class expressions, method calls, dynamic
  property names, static property access, static method calls, and class
  constant access
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
- Eval: direct `eval(...)` syntax is reserved by the lexer/parser and rejected
  with a stable parse diagnostic. The planned first executable slice treats
  `eval` as a language construct with one string-valued argument, parses that
  string through an eval-fragment parser entry point that does not require a
  `<?php` opening tag, executes the resulting statements in the caller's
  current symbol table, and uses `return` inside the fragment as the expression
  result. Eval execution, non-string eval arguments, exact `ParseError` object
  semantics, diagnostics inside evaluated strings, functions/classes declared
  by evaluated code, nested eval, include/require inside eval,
  references/copy-on-write interactions, `GLOBALS`/superglobal behavior,
  namespaces/use declarations, opcache behavior, and PHP's exact warning/fatal
  recovery behavior are not implemented.
- Namespaces/imports: `namespace` declarations, top-level `use` declarations,
  and namespace-qualified function/class names such as `App\fn()` and
  `new App\Box()` are reserved by the lexer/parser and rejected with stable
  parse diagnostics. Namespace-aware name resolution, bracketed namespace
  blocks, global namespace blocks, multiple namespaces in one file, executable
  qualified and fully qualified function/class references, aliased imports,
  grouped imports, function imports, constant imports, trait `use` execution,
  autoload interaction, and namespace-aware native lowering are not
  implemented.
- Object/class model: `php_runtime` has a small metadata and object-value model
  for the first object slice. It records an ordered class table with stable
  `ClassId` handles, declared class names with case-insensitive class lookup,
  ordered property metadata with case-sensitive property lookup, ordered method
  metadata with case-insensitive method lookup, visibility flags,
  static/instance flags, object-shape derivation for instance properties,
  initialized object values, and structured duplicate class/member diagnostics.
  `phpc run` registers top-level class declarations into this metadata table.
  The accepted member subset records properties without defaults and methods
  whose parameters/bodies use the existing function parser subset. `new
  ClassName()` looks up declared classes case-insensitively, initializes
  instance properties to `null`, skips static properties, treats object values
  as truthy, and lets direct `isset($object_variable)` return true. Undefined
  classes, constructor methods, and constructor arguments fail with stable
  runtime diagnostics. Public instance property reads and direct-variable
  writes work by static property name; property names are case-sensitive, and
  writes mutate the current object value stored in that variable.
  `isset($object->name)` works for direct object-variable operands and returns
  false for `null` slots, missing property names, undefined target variables,
  and non-object target variables. Undefined properties, property access on
  non-object values, and non-public properties still fail with stable runtime
  diagnostics for normal reads/writes. Static member expressions through `::`,
  including `ClassName::$prop`, `ClassName::method()`, and `ClassName::CONST`,
  fail with stable parse diagnostics. Method dispatch, dynamic property names,
  `$this`, visibility enforcement for non-public properties, static storage,
  class constants, object handle aliasing/identity, and native object lowering
  are not implemented.
- Arrays: array values preserve insertion order and normalize string keys that
  are valid decimal integers, such as `"2"` and `"-2"`, to integer keys.
  Strings with leading zeroes, leading `+`, decimal points, exponent notation,
  or integer overflow stay string keys. Duplicate normalized keys update the
  existing slot without moving it. Keyless literal entries and `$array[] = ...`
  writes append at the next non-negative integer key. Direct variable offset
  writes update existing array variables, and writes to undefined or `null`
  variables materialize an array. Existing-key reads return the stored value.
  Direct `isset($array[$key])` checks return true for existing non-null slots
  and false for null slots, missing keys, undefined array variables, and
  non-array target variables. Missing-key reads still fail with a stable
  runtime error instead of PHP's warning-and-`null` recovery. Array truthiness,
  `count`, `print_r`, and `var_dump` are implemented for this ordered value
  model.
- Type coercion: scalar arithmetic supports `null`, booleans, integers, floats,
  and well-formed numeric strings with optional sign, decimal point, exponent,
  and surrounding ASCII whitespace. Non-numeric strings fail with a stable
  runtime error. Truthiness is implemented for current scalar values.
- Scalar comparisons: loose equality and relational operators are implemented
  for the current scalar values using PHP 8-style behavior for booleans,
  numeric strings, non-numeric strings, empty strings, `null`, integers, and
  floats. This is not PHP's full comparison matrix: strict identity operators,
  arrays, objects, resources, and edge cases around `NAN`/`INF` and
  PHP-version-specific float string precision are not covered. Object
  comparisons in `phpc run` fail with an explicit unsupported-comparison
  runtime error.
- Loop control: `break;` and `continue;` execute for the innermost currently
  executing `while` loop in `phpc run`. A `break;` or `continue;` that reaches
  top-level code or a user-function body without an enclosing active loop fails
  with a stable invalid-loop-control runtime error. Loop-depth arguments such
  as `break 2;` and `continue 2;` are rejected with stable parse diagnostics.
  Interaction with future `for`/`foreach`/`do ... while`/`switch` execution,
  `finally`/exception behavior, and native lowering are not implemented.
- Runtime errors: diagnostics have stable messages and source locations, but
  they are not PHP `Throwable` objects and there is no warning/notice recovery
  mode yet. Representative runtime errors are covered by committed `phpc run`
  CLI snapshots that record exit code, stdout, and stderr for undefined
  variables, user-function arity mismatches, unsupported scalar `count()` calls,
  unsupported array keys, undefined array keys, unresolved dynamic function
  names, non-string dynamic function callees, division by zero, non-numeric
  string arithmetic, duplicate class metadata, undefined classes,
  undefined object properties, invalid property targets, non-public property
  access, object-to-string conversion, invalid `break`/`continue` outside a
  loop, and runaway user-function recursion.
- Native codegen: LLVM IR/assembly supports only straight-line echo/assignment
  with statically lowerable scalar expressions. Arrays, array indexing, array
  assignment, `break`, `continue`, class declarations, object instantiation,
  object property reads, and object property writes are rejected with explicit
  codegen errors.
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
  documented scalar/array/object subset. `print_r` can also render the current
  minimal object values. `strlen` remains scalar-only and rejects arrays and
  objects. `count` accepts arrays only. `isset` supports direct variable
  operands, direct array offset operands such as `isset($array[$key])`, and
  direct public object-property operands such as `isset($object->name)`; it can
  safely check undefined variables, missing/null array slots, undefined array
  variables, non-array array targets, and undefined object-property targets.
  Nested array offsets, append offset operands, dynamic property names,
  non-public property operands, complex lvalues, and general expression
  operands remain unsupported. Because `isset` is modeled as a special static
  form, it is not available through dynamic function lookup. PHP's complete
  warning behavior is not implemented.
- Object/class gaps: nested and conditional class declarations, method calls,
  `$this`, constructor execution, constructor arguments, inheritance,
  interfaces, traits, abstract/final/readonly modifiers, typed properties,
  property defaults, multiple properties in one declaration, constants, static
  property storage, late static binding, magic methods, namespaces,
  autoloading, anonymous classes, attributes, reflection, dynamic properties,
  dynamic property names, non-public property access, static member execution
  through `::`, `::class`, property assignment targets other than a direct
  variable, object handle identity/aliasing, cloning, destructors,
  serialization hooks, visibility enforcement, `self`/`parent`/`static`, object
  comparisons, object-to-string conversion, object callables, and native
  lowering are unsupported.
- Scalar arithmetic gaps: leading numeric strings with trailing non-numeric
  characters, such as `"10 apples"`, are rejected instead of warning and
  continuing with the leading number. PHP's warning/notice recovery mode,
  locale-sensitive numeric parsing, and exact integer-overflow promotion rules
  are not implemented.
- Array gaps: long `array(...)` literal syntax, `unset(...)` syntax, direct
  `foreach (...)` syntax, direct `for (...)` syntax, direct `do ... while`
  syntax, and direct `switch (...)` syntax are rejected with stable parse
  diagnostics; executing long array literals, variable/offset/property removal,
  iteration, and switch/case control flow is not implemented.
  Nested indexed writes, complex assignment lvalues, nested/complex
  `isset(...)` array offset operands, `$array[]` as a read expression, string
  offset access, `for`/`foreach`/`do ... while` iteration behavior, `switch`
  case matching/fallthrough/default handling, destructuring, spread,
  references, copy-on-write containers, and object/resource keys are not
  implemented. Array
  keys are currently limited to values that evaluate to integers or strings;
  PHP's boolean, null, float, object, and resource key coercions are rejected
  with a stable runtime error.
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
- include/require execution; `include`, `include_once`, `require`, and
  `require_once` currently fail with stable parse diagnostics
- `eval` execution; direct `eval(...)` currently fails with a stable parse
  diagnostic
- namespace and top-level `use` declarations, plus namespace-qualified
  function/class names such as `App\fn()` and `new App\Box()`; these currently
  fail with stable parse diagnostics before namespace-aware name resolution or
  imports exist
- method dispatch and dynamic property names; `$object->method()` and
  `$object->$name` currently fail with stable parse diagnostics
- non-public object property access and property writes to lvalues other than a
  direct variable
- constructor execution and constructor arguments for `new ClassName()`
- unsupported class forms including nested/conditional declarations,
  inheritance, interface implementation, typed properties, property defaults,
  multiple properties in one declaration, constants, and anonymous classes
- static property access, static method calls, and class constant access through
  `::`
- variable variables; `$$name` and `${...}` are rejected with a stable lex
  diagnostic rather than executed
- `global` declarations / importing top-level variables into function scope
- default parameter values outside the documented constant-expression subset
- required parameters after default parameters
- variadic parameters and variadic argument unpacking
- reference parameters, reference returns, reference assignments, and
  by-reference calls
- long `array(...)` literals; direct syntax is rejected with a stable parse
  diagnostic before execution
- `unset(...)`; direct syntax is rejected with a stable parse diagnostic before
  variable, array offset, or object property removal exists
- `foreach (...)`; direct syntax is rejected with a stable parse diagnostic
  before array/object iteration exists
- `for (...)`; direct syntax is rejected with a stable parse diagnostic before
  C-style loops exist
- `do ... while`; direct syntax is rejected with a stable parse diagnostic
  before post-condition loops exist
- `switch (...)`; direct syntax is rejected with a stable parse diagnostic
  before switch/case control flow exists
- `break`/`continue` loop-depth arguments such as `break 2;` and `continue 2;`;
  only statement-form `break;` and `continue;` for the innermost active
  `while` loop are implemented
- dynamic callables outside the string function-name subset, including array
  callables, object/method callables, first-class callable syntax,
  `call_user_func`, and namespace/autoload-aware callable resolution
- named arguments
- `declare(strict_types=1)` and PHP type declaration enforcement
- namespace-aware name resolution, imports, aliases, grouped imports, and
  executable qualified/fully qualified function or class references
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
