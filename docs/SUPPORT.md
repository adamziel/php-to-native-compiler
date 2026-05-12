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
- direct variable removal: `unset($name)` removes static variables from the
  current scope and treats undefined names as no-ops; `unset(...)` may include
  multiple supported operands and executes them left to right
- assignment statements
- arithmetic: `+`, `-`, `*`, `/` with scalar coercions for `null`, booleans,
  integers, floats, and well-formed numeric strings
- unary `-` and `!`
- string concatenation: `.`
- loose comparisons: `==`, `!=`, `<`, `<=`, `>`, `>=` across the current
  scalar values (`null`, booleans, integers, floats, and strings)
- strict identity comparisons: `===` and `!==` across the current scalar
  values only (`null`, booleans, integers, floats, and strings)
- `if` / `elseif` / `else`
- `while`
- `for (initializer; condition; increment)` loops where each header slot is
  optional and each initializer/increment slot contains at most one expression
  or assignment from the current assignment subset
- `do ... while` loops with a block or single-statement body and a
  post-condition expression
- `switch ($value) { case ...: ... default: ... }` statements over the current
  scalar loose-comparison subset, including `case`, `default`, fallthrough, and
  `break;` to exit the switch
- `foreach ($array as $value)` and `foreach ($array as $key => $value)` over
  ordered arrays
- `break;` for the innermost currently executing `while`, `for`,
  `do ... while`, `foreach`, or `switch`; `continue;` for the innermost
  currently executing loop
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
- short array literals (`[]`, `[value]`, `[key => value]`) and long
  `array(...)` literals as an alias for that same array-literal subset
- ordered arrays with integer and string keys
- array indexed reads: `$array[$key]` for existing integer/string keyed array
  entries
- direct variable array writes: `$array[$key] = ...` and `$array[] = ...`
- direct array offset removal: `unset($array[$key])` for direct array
  variables over the current integer/string key subset; multiple supported
  `unset(...)` operands execute left to right
- `foreach ($array as $value)` and `foreach ($array as $key => $value)`
  iteration in insertion order over a snapshot of the current array entries
- `isset($array[$key])` for direct array-variable offset operands over the
  current integer/string key subset
- `empty($name)` and `empty($array[$key])` for direct variables and direct
  array-variable offset operands over the current scalar/array value model
- builtins for the documented subset: `strlen`, `isset`, `empty`, `count`,
  `array_key_exists`, `array_key_first`, `array_key_last`, `array_values`,
  `array_keys`, `array_reverse`, `array_merge`, `array_flip`,
  `array_fill_keys`, `array_count_values`, `array_filter`, `in_array`,
  `array_search`, `var_dump`, and `print_r`; `print_r` can render current
  minimal object values
- structured runtime errors for undefined variables, arity mismatches,
  unsupported calls, division by zero, non-numeric string arithmetic, and
  undefined functions, non-string dynamic function callees, unsupported array
  keys, undefined array keys, invalid array access including non-array
  `unset($array[$key])` targets, unsupported complex
  `empty` operands, non-array `array_key_first`/`array_key_last` operands,
  non-array `array_reverse` operands, non-bool `array_reverse` preserve-key
  flag values, non-array `array_merge` operands, non-array
  `array_flip` operands, unsupported non-int/string `array_flip` values,
  non-array `array_fill_keys` operands, unsupported non-int/string
  `array_fill_keys` key values, non-array `array_count_values` operands,
  unsupported non-int/string `array_count_values` values, non-array
  `array_filter` operands, unsupported `array_filter` callback/mode operands,
  non-array `in_array`/`array_search` haystacks, non-bool
  `in_array`/`array_search` strict-mode flag values, unsupported non-scalar
  `array_keys` search-value comparisons, non-bool `array_keys` strict-mode
  flag values, unsupported non-scalar `in_array`/`array_search` comparisons,
  unsupported `global` declarations,
  duplicate class/member metadata, undefined classes, unsupported object
  instantiation, undefined object properties, invalid property targets,
  unsupported non-public property access, object-to-string conversion,
  unsupported strict identity array/object operands, invalid `foreach`
  iterables, invalid `break`/`continue` outside a loop, unsupported `continue;`
  inside `switch`, and runaway user-function recursion
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
- explicit parse diagnostics for unsupported array spread/reference elements
- explicit parse diagnostics for unsupported `unset(...)` forms outside the
  current direct-variable and direct array-offset statement subset
- explicit parse diagnostics for unsupported `foreach` by-reference iteration,
  destructuring loop targets, and expression-position `foreach`
- explicit parse diagnostics for unsupported expression-position `for` and
  comma-separated `for` header expression lists
- explicit parse diagnostics for unsupported expression-position `do ... while`
- explicit parse diagnostics for unsupported alternate `if`/`elseif`/`else`
  colon/`endif` syntax
- explicit parse diagnostics for unsupported expression-position `switch` and
  alternate colon/`endswitch` syntax
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
  direct `unset($name)`, `isset($name)`, parameter binding, default-parameter
  evaluation, and direct array writes route through that symbol table path.
  Direct `unset($name)` removes the current-scope symbol and treats missing
  names as no-ops; later plain reads use the existing undefined-variable
  diagnostic. Multiple supported `unset(...)` operands run left to right.
  Runtime lookup by a value computed from PHP code is not implemented yet, so
  variable variables still do not execute.
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
  Direct `unset($array[$key])` removes matching entries from existing arrays,
  preserves the insertion order of remaining entries, does not rewind the next
  append key, treats missing keys as no-ops, and treats undefined or `null`
  target variables as no-ops. Multiple supported `unset(...)` operands execute
  left to right, including any array-offset key expressions. Existing non-array
  targets fail with a stable invalid-array-access diagnostic.
  Direct `isset($array[$key])` checks return true for existing non-null slots
  and false for null slots, missing keys, undefined array variables, and
  non-array target variables. Direct `empty($array[$key])` checks return true
  for missing keys, undefined array variables, non-array target variables, and
  existing slots whose values use the current falsey rules (`null`, `false`,
  zero, empty string, string `"0"`, and empty arrays). `array_key_exists($key,
  $array)` checks existing integer/string keyed slots without filtering out
  `null` values and is also available through string-valued dynamic function
  calls. `array_key_first($array)` returns the first inserted integer or string
  key as an `int` or `string`, and `array_key_last($array)` returns the last
  inserted integer or string key. Both return `null` for an empty array and are
  available through string-valued dynamic function calls. `array_values($array)`
  returns a new ordered array containing the original values in insertion order
  with integer keys starting at zero.
  `array_keys($array)` returns a new ordered array containing the original
  integer/string keys as values in insertion order with integer keys starting at
  zero. `array_keys($array, $search_value)` returns only keys whose values match
  the supplied current scalar `search_value` under the same loose comparison
  rules used by `in_array` and `array_search`, reindexed from zero.
  `array_keys($array, $search_value, true)` uses the current scalar strict
  identity rules, and `array_keys($array, $search_value, false)` uses the loose
  path. These forms are available through string-valued dynamic function calls.
  `array_reverse($array)` and `array_reverse($array, false)` return a new
  ordered array in reverse insertion order, reindex integer-keyed entries from
  zero, preserve string keys, and are available through string-valued dynamic
  function calls. `array_reverse($array, true)` returns a new ordered array in
  reverse insertion order while preserving both integer and string keys.
  `array_merge()` returns an empty array. `array_merge($array, ...)` accepts
  zero or more array operands, processes them left to right in insertion order,
  appends and reindexes integer-keyed entries from zero, preserves string keys,
  and overwrites duplicate string-key values with later values without moving
  the original string-key slot. It is also available through string-valued
  dynamic function calls. `array_flip($array)` accepts arrays, converts
  integer and string array values into result keys using the current array-key
  normalization rules, writes each original integer/string key as the result
  value, overwrites duplicate flipped keys with later values without moving the
  first flipped-key slot, and is available through string-valued dynamic
  function calls. `array_fill_keys($keys, $value)` accepts an array of
  integer/string key values, creates a new ordered array using those values as
  normalized result keys, stores the supplied value in each result slot, and
  overwrites duplicate result keys with later entries without moving the first
  key position. It is also available through string-valued dynamic function
  calls. `array_count_values($array)` accepts arrays whose values are integers
  or strings, counts values in insertion order using the current array-key
  normalization rules for string values, stores integer counts as result
  values, and is available through string-valued dynamic function calls.
  `array_filter($array)` without a callback accepts arrays only, removes values
  that are falsey under the current PHP-shaped truthiness rules, preserves the
  original integer/string keys and insertion order of kept entries, and is
  available through string-valued dynamic function calls.
  `in_array($needle, $array)` scans values in insertion order using the
  current loose scalar comparison rules; `in_array($needle, $array, true)` uses
  the current scalar strict identity rules, and `in_array($needle, $array,
  false)` uses the loose path. `in_array` is also available through
  string-valued dynamic function calls. `array_search($needle, $array)` uses
  the same loose scalar scan, returning the first matching integer/string key or
  `false` when no value matches; `array_search($needle, $array, true)` uses the
  current scalar strict identity rules, and `array_search($needle, $array,
  false)` uses the loose path. It is also available through string-valued
  dynamic function calls. `foreach ($array as $value)` iterates array values in
  insertion order over a snapshot of the current entries and writes the current
  value to the direct loop variable in the active scope. `foreach ($array as
  $key => $value)` additionally writes the current integer or string key as an
  `int` or `string` value to the direct key loop variable. Missing key reads
  still fail with a stable runtime error instead of PHP's
  warning-and-`null` recovery. Array truthiness, `count`, `array_key_exists`,
  `array_key_first`, `array_key_last`, `array_values`, `array_keys`,
  `array_reverse`, `array_merge`, `array_flip`, `array_fill_keys`,
  `array_count_values`, `array_filter` without a callback, `in_array`,
  `array_search`, both current `foreach` array forms, direct array-offset
  `unset`, multiple supported `unset(...)` operands, `print_r`, and `var_dump`
  are implemented for this ordered value model.
- Type coercion: scalar arithmetic supports `null`, booleans, integers, floats,
  and well-formed numeric strings with optional sign, decimal point, exponent,
  and surrounding ASCII whitespace. Non-numeric strings fail with a stable
  runtime error. Truthiness is implemented for current scalar values.
- Scalar comparisons: loose equality and relational operators are implemented
  for the current scalar values using PHP 8-style behavior for booleans,
  numeric strings, non-numeric strings, empty strings, `null`, integers, and
  floats. Strict identity operators `===` and `!==` execute for the current
  scalar values with type-and-value semantics and no numeric/string coercion.
  This is not PHP's full comparison matrix: strict identity for arrays,
  objects, resources, references, object handle identity, and edge cases around
  `NAN`/`INF` and PHP-version-specific float string precision are not covered.
  Object loose comparisons and strict identity involving array/object operands
  in `phpc run` fail with explicit unsupported-comparison runtime errors.
- Conditionals: statement-form `if` supports zero or more `elseif` clauses and
  an optional `else` clause over the current expression and truthiness subset.
  Branch bodies may be brace blocks or single statements. Alternate
  `if`/`elseif`/`else` colon/`endif` conditional syntax now fails with a
  stable parse diagnostic. Alternate conditional execution, nested alternate
  conditional parsing, mixed brace/colon conditional recovery, and native
  conditional lowering are not implemented.
- Loop control: `break;` and `continue;` execute for the innermost currently
  executing `while`, supported `for`, supported `do ... while`, or supported
  array `foreach` loop in `phpc run`; `break;` also exits the innermost
  supported `switch`. For `for` loops, `continue;` runs the increment action
  before the next condition check. For `do ... while` loops, `continue;` skips
  the rest of the body and evaluates the post-condition before the next
  iteration. A `break;` or `continue;` that reaches top-level code or a
  user-function body without an enclosing active loop fails with a stable
  invalid-loop-control runtime error. A `continue;` that reaches a `switch`
  body is rejected with a stable runtime error instead of modeling PHP's
  warning-and-break behavior. Loop-depth arguments such as `break 2;` and
  `continue 2;` are rejected with stable parse diagnostics. `finally`/exception
  behavior and native lowering are not implemented.
- Switch: statement-form brace `switch` executes in `phpc run` over the current
  scalar loose-comparison subset. The switch expression is evaluated once, case
  expressions are evaluated in source order until the first loose `==` match,
  `default` is used only when no case matches, and execution falls through
  later labels until a `break;`, `return`, or the end of the switch body.
  Arrays, objects, resources, alternate colon/`endswitch` syntax, semicolon
  case separators, `continue;` inside switch, and native lowering are not
  implemented.
- Runtime errors: diagnostics have stable messages and source locations, but
  they are not PHP `Throwable` objects and there is no warning/notice recovery
  mode yet. Representative runtime errors are covered by committed `phpc run`
  CLI snapshots that record exit code, stdout, and stderr for undefined
  variables, user-function arity mismatches, unsupported scalar `count()` calls,
  unsupported array keys, undefined array keys, invalid `array_key_exists`
  keys, non-array `array_key_exists` operands, non-array `array_key_first`
  or `array_key_last` operands, non-array `array_values` operands, non-array
  `array_keys` operands, unsupported `array_keys` search-value comparisons,
  non-bool `array_keys` strict-mode flag values,
  non-array `array_reverse` operands, non-bool
  `array_reverse` preserve-key flag values, non-array
  `array_merge` operands, non-array `array_flip` operands, unsupported
  non-int/string `array_flip` values, non-array `array_fill_keys` operands,
  unsupported non-int/string `array_fill_keys` key values, non-array
  `array_count_values` operands, unsupported non-int/string
  `array_count_values` values, non-array `array_filter` operands, unsupported
  `array_filter` callback/mode operands, non-array `in_array` operands,
  non-array `array_search` operands, non-array `foreach` iterables, non-bool
  `in_array`/`array_search` strict-mode flag values, and array-value
  comparisons for `in_array`/`array_search`,
  unsupported complex `empty` operands, non-array `unset($array[$key])`
  targets, unresolved dynamic function callees, division by zero, non-numeric
  string arithmetic, duplicate class metadata, undefined classes, undefined
  object properties, invalid property targets, non-public property access,
  object-to-string conversion, invalid `break`/`continue` outside a loop,
  unsupported `continue;` inside `switch`, and runaway user-function recursion.
- Native codegen: LLVM IR/assembly supports only straight-line echo/assignment
  with statically lowerable scalar expressions. `if`/`elseif`/`else`, `while`,
  arrays, array indexing, array assignment, variable unset, array offset unset,
  multiple-operand unset, `for`, `do ... while`, `switch`, `foreach`, `break`,
  `continue`, class declarations, object instantiation, object property reads,
  and object property writes are rejected with explicit codegen errors.
- Assembly emission: uses LLVM tools when available, with a temporary `cc -S`
  C fallback for the same narrow lowerable subset.
- Function calls: user-defined positional calls are supported in `phpc run`.
  Dynamic function calls are supported only when the callee expression evaluates
  to a string that case-insensitively resolves to a user-defined function or to
  one of the documented callable builtins: `strlen`, `count`,
  `array_key_exists`, `array_key_first`, `array_key_last`, `array_values`,
  `array_keys`, `array_reverse`, `array_merge`, `array_flip`,
  `array_fill_keys`, `array_count_values`, `array_filter`, `in_array`,
  `array_search`, `var_dump`, or `print_r`.
  Unresolved names fail with a stable undefined-function runtime error, and
  non-string callees fail with a stable unsupported-call runtime error. Required
  parameters and trailing default parameter values are supported. Defaults may
  use the current constant-expression subset: `null`, booleans, integers,
  floats, strings, short and long arrays with supported keys, unary
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
- Builtins: `strlen`, `isset`, `empty`, `count`, `array_key_exists`,
  `array_key_first`, `array_key_last`, `array_values`, `array_keys`,
  `array_reverse`, `array_merge`, `array_flip`, `array_fill_keys`,
  `array_count_values`, `array_filter`, `in_array`, `array_search`,
  `var_dump`, and `print_r` cover the documented scalar/array/object subset.
  `print_r` can also render the current minimal object values. `strlen`
  remains scalar-only and rejects arrays and objects. `count` accepts arrays
  only.
  `array_key_exists($key, $array)` accepts integer
  and string keys over the current ordered array value model, returns true for
  existing keys even when the stored value is `null`, returns false for missing
  keys, rejects non-array second arguments, and rejects unsupported key values
  such as booleans, `null`, floats, objects, and future resources instead of
  applying PHP's full key coercions. `array_key_first($array)` and
  `array_key_last($array)` accept arrays only, return the first or last
  inserted integer or string key as an `int` or `string`, return `null` for
  empty arrays, and are also available through string-valued dynamic function
  calls. `array_values($array)` accepts arrays
  only, preserves value insertion order, and returns a new ordered array
  reindexed with integer keys `0..n-1`; it is also available through
  string-valued dynamic function calls. `array_keys($array)` accepts arrays
  only, preserves insertion order, and returns a new ordered array reindexed
  with integer keys `0..n-1` whose values are the original integer/string keys.
  `array_keys($array,
  $search_value)` accepts current scalar search values, scans array values in
  insertion order with the current PHP 8-style loose scalar comparison rules,
  emits every matching integer/string key as a value, and reindexes the returned
  key array from zero. `array_keys($array, $search_value, true)` uses current
  scalar strict identity semantics, and `array_keys($array, $search_value,
  false)` uses the loose path. The third argument must evaluate to a boolean in
  the current subset. These forms are also available through string-valued
  dynamic function calls. Array/object search values or array/object values
  encountered during filtering fail with stable unsupported-call diagnostics.
  `array_reverse($array)` and `array_reverse($array, false)` accept arrays only,
  return a new array in
  reverse insertion order, reindex integer-keyed entries from zero, preserve
  string keys, and are also available through string-valued dynamic function
  calls. `array_reverse($array, true)` preserves both integer and string keys
  while reversing insertion order. The optional `preserve_keys` argument must
  evaluate to a boolean in the current subset; non-bool flag coercion,
  reference/copy-on-write behavior, object handle identity preservation,
  resource values, and native lowering are not implemented.
  `array_merge()` accepts zero arguments and returns an empty array.
  `array_merge($array, ...)` accepts any number of array operands, processes
  them left to right in insertion order, appends integer-keyed entries with new
  integer keys starting at zero, preserves string keys, and overwrites
  duplicate string keys with later values without moving the first string-key
  position. It is also available through string-valued dynamic function calls.
  Non-array operands fail with stable diagnostics naming the offending
  positional argument. References, copy-on-write containers, object handle
  identity preservation, resource values, exact native `TypeError` objects, and
  native lowering are not implemented.
  `array_flip($array)` accepts arrays only, uses integer values directly as
  result keys, normalizes string values through the current PHP-style decimal
  string key rules, and writes each original integer/string key as the result
  value. Duplicate flipped keys are overwritten by later source entries without
  moving the first flipped-key position. Unsupported source values such as
  `null`, booleans, floats, arrays, objects, and future resources fail with a
  stable project diagnostic instead of PHP's warning-and-skip behavior.
  References, copy-on-write containers, exact native warning/`TypeError`
  behavior, and native lowering are not implemented. `array_flip` is also
  available through string-valued dynamic function calls.
  `array_fill_keys($keys, $value)` accepts arrays only for the first argument,
  uses integer key values directly as result keys, normalizes string key values
  through the current PHP-style decimal string key rules, and stores the
  supplied value in every result slot using the current cloned `Value` model.
  Duplicate result keys are overwritten by later key entries without moving the
  first result-key position. Unsupported key values such as `null`, booleans,
  floats, arrays, objects, and future resources fail with a stable project
  diagnostic instead of PHP's warning-and-skip behavior. References,
  copy-on-write containers, object handle identity for object fill values,
  exact native warning/`TypeError` behavior, and native lowering are not
  implemented. `array_fill_keys` is also available through string-valued
  dynamic function calls.
  `array_count_values($array)` accepts arrays only, uses integer values
  directly as result keys, normalizes string values through the current
  PHP-style decimal string key rules, and stores integer occurrence counts as
  result values. Duplicate counted keys update the existing count without
  moving the first result-key position. Unsupported source values such as
  `null`, booleans, floats, arrays, objects, and future resources fail with a
  stable project diagnostic instead of PHP's warning-and-skip behavior.
  References, copy-on-write containers, exact native warning/`TypeError`
  behavior, resource values, and native lowering are not implemented.
  `array_count_values` is also available through string-valued dynamic
  function calls.
  `array_filter($array)` without a callback accepts arrays only, removes
  `null`, `false`, zero integers and floats, empty strings, string `"0"`, and
  empty arrays using the current `Value::is_truthy` rules, preserves the
  original integer/string keys and insertion order of kept entries, and is
  available through string-valued dynamic function calls. Callback arguments,
  mode flags such as key-only or key/value callback mode, references,
  copy-on-write containers, exact native `TypeError` objects, object handle
  identity preservation, resource values, and native lowering are not
  implemented.
  `in_array($needle, $array)` accepts an array haystack, scans values in
  insertion order, and uses the
  current PHP 8-style loose scalar comparison rules for `null`, booleans,
  integers, floats, and strings. `in_array($needle, $array, true)` uses the
  current scalar strict identity rules with no numeric/string coercion;
  `in_array($needle, $array, false)` uses the loose path. The third argument
  must evaluate to a boolean in the current subset. `in_array` rejects non-array
  haystacks and rejects array or object needles/values when encountered instead
  of modeling PHP's full non-scalar comparison behavior. `in_array` is also
  available through string-valued dynamic function calls.
  `array_search($needle, $array)` accepts an array haystack, scans values in
  insertion order, returns the first matching integer/string key as an `int` or
  `string`, and returns `false` when no value matches. The two-argument form
  uses the current loose scalar comparison rules, `array_search($needle,
  $array, true)` uses current scalar strict identity with no numeric/string
  coercion, and `array_search($needle, $array, false)` uses the loose path. The
  third argument must evaluate to a boolean in the current subset. It rejects
  non-array haystacks and rejects array or object needles/values when
  encountered. `array_search` is also available through string-valued dynamic
  function calls. `isset` supports direct variable
  operands, direct array offset operands such as `isset($array[$key])`,
  and direct public object-property
  operands such as `isset($object->name)`; it can safely check undefined
  variables, missing/null array slots, undefined array variables, non-array
  array targets, and undefined object-property targets. Nested array offsets,
  append offset operands, dynamic property names, non-public property operands,
  complex lvalues, and general expression operands remain unsupported. `empty`
  supports one direct variable operand or one direct array offset operand such
  as `empty($array[$key])`; undefined variables, missing array keys, undefined
  array targets, and non-array array targets are treated as empty, and existing
  values use the current PHP truthiness rules. Nested array offsets, object
  property operands, append offset operands, complex lvalues, general
  expression operands, and unsupported array-key coercions remain unsupported.
  `array_key_first`, `array_key_last`, `array_values`, `array_keys`,
  `array_reverse`, `array_merge`, `array_flip`, `array_fill_keys`,
  `array_count_values`, `array_filter`, `in_array`, `array_search`, and both
  current `foreach` array forms follow the current by-value model; PHP
  references, copy-on-write containers, object handle identity preservation,
  resource values, array, object, resource, or reference search values for
  `array_keys`, non-bool `array_keys` strict-flag coercion, non-bool
  `array_reverse` preserve-key flag coercion, `array_merge`
  reference/copy-on-write behavior, `array_flip` warning-and-skip behavior for
  unsupported source values, and `array_fill_keys` warning-and-skip behavior
  for unsupported key values, `array_count_values` warning-and-skip behavior
  for unsupported values, and `array_filter` callback/mode forms are not
  implemented.
  Because `isset` and `empty` are modeled as special static forms, they are not
  available through dynamic function lookup. PHP's complete warning behavior is
  not implemented.
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
- Scalar comparison gaps: strict identity is implemented only for the current
  scalar values. Strict identity for arrays, objects, resources, references,
  object handle identity, and native lowering is not implemented. Array/object
  strict identity operands fail with stable unsupported-comparison runtime
  diagnostics. Float identity currently follows Rust/PHP-style `f64` equality
  for representable literals and does not claim broader `NAN`/`INF` precision
  edge-case coverage.
- Array gaps: array spread elements and array reference elements are rejected
  with stable parse diagnostics. `unset(...)` forms outside direct variables
  and direct array-offset operands, comma-separated `for` header expression
  lists, expression-form `do ... while`, expression-form `switch`, and
  alternate switch syntax are rejected with stable parse diagnostics; object
  property removal, append-offset unset, and nested/complex unset operands are
  not implemented.
  Nested indexed writes, complex assignment lvalues, nested/complex
  `isset(...)` and `empty(...)` array offset operands, `$array[]` as a read
  expression, string offset access, by-reference `foreach`, object iteration,
  destructuring loop targets, references, copy-on-write containers, and
  object/resource keys are not implemented. The current `foreach` array forms
  snapshot array entries at loop start and do not claim PHP's full
  mutation/aliasing behavior while the iterated array is modified. Array keys
  are currently limited to values that
  evaluate to integers or strings; PHP's boolean, null, float, object, and
  resource key coercions are rejected with a stable runtime error.
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
- array literal spread elements and array literal reference elements
- `unset(...)` forms outside direct variables and direct array offsets,
  including object property removal, append-offset unset, and nested/complex
  operands; these fail with stable parse diagnostics
- by-reference `foreach`, object iteration, destructuring loop targets, and
  expression-form `foreach`
- comma-separated `for` initializer, condition, or increment expression lists;
  only zero or one expression or assignment is supported in each header slot
- expression-form `for`; `for` is only supported as a statement
- alternate colon/`endif` syntax for `if`/`elseif`/`else`; conditionals are
  limited to brace blocks or single-statement bodies
- expression-form `do ... while`; `do ... while` is only supported as a
  statement
- expression-form `switch`, alternate colon/`endswitch` switch syntax,
  semicolon case separators, and `continue;` behavior inside switch
- `break`/`continue` loop-depth arguments such as `break 2;` and `continue 2;`;
  only statement-form `break;` for the innermost active `while`, supported
  `for`, supported `do ... while`, supported array `foreach`, or supported
  `switch`, and `continue;` for the innermost active loop are implemented
- dynamic callables outside the string function-name subset, including array
  callables, object/method callables, first-class callable syntax,
  `call_user_func`, and namespace/autoload-aware callable resolution
- `array_key_first`/`array_key_last` exact native `TypeError` objects,
  reference/copy-on-write container behavior, and native lowering
- `array_keys` filtering over array, object, resource, or reference search
  values or array values, plus non-bool strict-flag coercion
- `in_array` and `array_search` strict-mode searches involving
  array/object/resource/reference values, non-bool strict-flag coercion, and
  array/object needle or haystack-value comparisons for the current
  array-search builtins
- `array_reverse` non-bool `preserve_keys` coercion, reference/copy-on-write
  behavior, object handle identity preservation, resource values, and native
  lowering
- `array_merge` reference/copy-on-write behavior, object handle identity
  preservation, resource values, exact native `TypeError` objects, and native
  lowering
- `array_flip` warning-and-skip behavior for unsupported source values,
  reference/copy-on-write behavior, exact native warning/`TypeError` objects,
  resource values, and native lowering
- `array_fill_keys` warning-and-skip behavior for unsupported key values,
  reference/copy-on-write behavior, object handle identity for object fill
  values, exact native warning/`TypeError` objects, resource values, and native
  lowering
- `array_count_values` warning-and-skip behavior for unsupported values,
  reference/copy-on-write behavior, exact native warning/`TypeError` objects,
  resource values, and native lowering
- `array_filter` callback arguments, `ARRAY_FILTER_USE_KEY` and
  `ARRAY_FILTER_USE_BOTH` callback modes, reference/copy-on-write behavior,
  object handle identity preservation, resource values, exact native
  `TypeError` objects, and native lowering
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
- `empty(...)` operands outside direct variables and direct array offsets,
  including nested offsets, object properties, append offsets, and general
  expressions
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
