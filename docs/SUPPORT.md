# Support Matrix

## Supported in `phpc run`

- PHP opening tag `<?php`
- `echo` statements with one or more comma-separated expressions
- `print` statements
- integer literals
- float literals
- single-quoted and double-quoted string literals with basic escapes
- `null`, `true`, and `false`
- magic constants `__LINE__`, evaluated from the expression token's source
  line, `__FILE__`, evaluated from the current `phpc run` input path when one
  is available, `__DIR__`, evaluated as that path's parent directory, and
  `__FUNCTION__`, evaluated as the current user-function name or an empty
  string outside a function
- static variables backed by per-scope materialized symbol tables
- direct variable removal: `unset($name)` removes static variables from the
  current scope and treats undefined names as no-ops; `unset(...)` may include
  multiple supported operands and executes them left to right
- assignment statements, plus expression-position direct static-variable
  assignment `$name = expr` and direct array-offset assignment
  `$array[$key] = expr`, and direct public object-property assignment
  `$object->property = expr` with right-to-left chained assignment result
  values over the current value model. Direct append-offset assignment
  `$array[] = expr` is supported as a standalone assignment expression with an
  assignment result value. Direct array-offset
  assignment expressions evaluate the key before the right-hand expression and
  materialize undefined or `null` target variables as arrays. Direct
  append-offset assignment expressions evaluate the right-hand expression,
  append to direct array variables, materialize undefined or `null` target
  variables as arrays, and return the appended value. Direct object-property
  assignment expressions evaluate the right-hand expression, then write
  existing declared public property slots on direct object variables.
  Append-offset chained assignment expressions, nested-offset assignment
  expressions, dynamic property names, missing property materialization,
  references/copy-on-write, and native lowering remain unsupported.
- direct static-variable compound assignment `$name += expr`,
  `$name -= expr`, `$name *= expr`, `$name /= expr`, `$name %= expr`,
  `$name .= expr`, `$name &= expr`, `$name |= expr`, `$name ^= expr`,
  `$name <<= expr`, and `$name >>= expr` over the current scalar/bitwise value
  model in statement position, expression position, and C-style `for`
  initializer/increment slots. In expressions, compound assignment returns the
  updated value.
- direct array-offset compound assignment `$array[$key] += expr`,
  `$array[$key] -= expr`, `$array[$key] *= expr`, `$array[$key] /= expr`,
  `$array[$key] %= expr`, `$array[$key] .= expr`, `$array[$key] &= expr`,
  `$array[$key] |= expr`, `$array[$key] ^= expr`, `$array[$key] <<= expr`, and
  `$array[$key] >>= expr` over existing integer/string keyed array entries in
  statement position, expression position, and C-style `for`
  initializer/increment slots. In expressions, compound assignment returns the
  updated value.
- direct public object-property compound assignment `$object->property +=
  expr`, `$object->property -= expr`, `$object->property *= expr`,
  `$object->property /= expr`, `$object->property %= expr`, and
  `$object->property .= expr` over existing declared public property slots and
  private property slots owned by the active declaring class and protected
  slots owned by the active class or an ancestor, plus bitwise/shift compound
  forms
  `$object->property &= expr`, `$object->property |= expr`,
  `$object->property ^= expr`, `$object->property <<= expr`, and
  `$object->property >>= expr`, in statement position, expression position,
  and C-style `for` initializer/increment slots. In expressions, compound
  assignment returns the updated value.
- direct array-offset pre/post increment and decrement in statement position,
  expression position, and C-style `for` initializer/increment slots:
  `++$array[$key]`, `$array[$key]++`, `--$array[$key]`, and
  `$array[$key]--` for existing integer/string keyed entries whose current
  values are integers or floats. In expressions, pre forms return the updated
  value and post forms return the previous value.
- direct public object-property pre/post increment and decrement in statement
  position, expression position, and C-style `for` initializer/increment
  slots: `++$object->property`, `$object->property++`,
  `--$object->property`, and `$object->property--` for existing declared
  public property slots, private slots owned by the active declaring class, and
  protected slots owned by the active class or an ancestor whose current values
  are integers or floats. In expressions, pre forms return the updated value and post forms
  return the previous value.
- direct static-variable pre/post increment and decrement in statement
  position, expression position, and C-style `for` initializer/increment
  slots: `++$name`, `$name++`, `--$name`, and `$name--` for existing integer
  and float variables only. In expressions, pre forms return the updated
  value and post forms return the previous value.
- arithmetic: `+`, `-`, `*`, `/` with scalar coercions for `null`, booleans,
  integers, floats, and well-formed numeric strings; modulo `%` over the
  current integer-coercion subset for `null`, booleans, integers, floats, and
  well-formed numeric strings, returning integer remainders and reporting a
  stable modulo-by-zero diagnostic
- unary `-` and `!`
- string concatenation: `.`
- loose comparisons: `==`, `!=`, `<`, `<=`, `>`, `>=` across the current
  scalar values (`null`, booleans, integers, floats, and strings)
- strict identity comparisons: `===` and `!==` across the current scalar
  values only (`null`, booleans, integers, floats, and strings)
- logical operators `&&`, `||`, `and`, `xor`, and `or` over the current value
  model: operands use PHP-style truthiness, results are booleans, `&&`, `||`,
  `and`, and `or` evaluate right operands lazily, `xor` evaluates both
  operands, `&&` binds tighter than `||`, and word operators bind lower than
  assignment with `and` tighter than `xor` and `xor` tighter than `or` in the
  current expression and statement parser subset
- bitwise operators `&`, `|`, `^`, unary `~`, and shift operators `<<`/`>>`
  over the current integer/string subset: binary integer-like operands produce
  integer results after current scalar-to-int coercion, unary `~` accepts
  integer operands, shift operators coerce both operands through the same
  scalar-to-int path and reject negative shift counts, string operands use
  bytewise PHP behavior for `&`, `|`, `^`, and `~` when the resulting runtime
  string remains valid UTF-8, and bitwise precedence is additive before
  shifts, then concatenation, comparisons/equality before `&`, then `^`, then
  `|`, then `&&` and `||`. Direct static-variable, direct array-offset, and
  supported direct object-property compound assignments support `&=`, `|=`,
  `^=`, `<<=`, and `>>=` through the same runtime helper semantics.
- full ternary conditional expressions `$condition ? $if_true : $if_false`
  and short ternary expressions `$value ?: $fallback` over the current
  expression/value subset, including truthiness-based condition selection,
  lazy branch/fallback evaluation, condition-value reuse for short ternary,
  parenthesized nested ternaries, mixes with `??`, and assignment-expression
  branches over the documented direct-target subset
- `if` / `elseif` / `else`
- `while`
- `for (initializer; condition; increment)` loops where each header slot is
  optional and each initializer/increment slot contains at most one expression
  or assignment from the current assignment subset, including direct
  static-variable compound assignment, direct array-offset compound
  assignment, and direct static-variable increment/decrement
- `do ... while` loops with a block or single-statement body and a
  post-condition expression
- `switch ($value) { case ...: ... default: ... }` and alternate
  `switch ($value): case ...: ... default: ... endswitch;` statements over the
  current scalar loose-comparison subset, including `case`, `default`, `:` or
  `;` case/default separators, fallthrough, and `break;` to exit the switch
- `foreach ($array as $value)` and `foreach ($array as $key => $value)` over
  ordered arrays
- `break;` for the innermost currently executing `while`, `for`,
  `do ... while`, `foreach`, or `switch`; `continue;` for the innermost
  currently executing loop
- function declarations with optional trailing commas in parameter lists
- positional function calls with optional trailing commas in argument lists
- dynamic function calls through string-valued expressions that resolve to the
  documented callable builtin subset or user-defined functions, with optional
  trailing commas in argument lists
- trailing default parameter values for user functions over the documented
  constant-expression subset, including bare references to previously defined
  unqualified constants and the current built-in global constant slice when an
  omitted argument is bound
- recursive user-function calls up to a fixed 128-frame user-function call-depth
  guard
- `return`
- isolated local scopes for user-function calls; parameters and function-local
  assignments can shadow global names without mutating them
- top-level class declarations registered into the runtime metadata table:
  `class Name { ... }` and `class Child extends Parent { ... }` with
  single-parent metadata, property names, method names, visibility, and static
  flags for the documented subset
- object instantiation with `new ClassName(...)` for declared classes. Classes
  without `__construct` are supported only with no constructor arguments.
  Declared or inherited public instance `__construct` methods execute with
  scoped `$this`, positional arguments, and the current default-parameter
  subset.
- public instance property reads and direct-variable writes by static property
  name, including inherited public property slots:
  `$object->name` and `$object->name = ...`. Plain reads and direct writes for
  private property slots owned by the active declaring class and protected
  property slots owned by the active class or an ancestor are also supported,
  including inherited parent-declared protected slots on child objects.
- public, same-class private, and protected same-class/child instance method
  calls by static method name:
  `$object->method(...)` evaluates the object receiver, checks a declared
  or inherited instance method case-insensitively, evaluates positional
  arguments left-to-right, executes the method body in a fresh local scope, and
  binds `$this` to the current object handle so `$this->property` reads/writes
  share the caller-visible object slots. Private methods are callable only
  while executing a method on the same declaring class. Protected methods are
  callable while executing a method on the same declaring class or a child
  class. Current method calls reuse the existing user-function
  parameter/default/return subset.
- explicit parent method calls by static method name:
  `parent::method(...)` and `parent::__construct(...)` are supported in active
  instance method/constructor context when the current class has a parent and
  the resolved parent-chain method is public or protected under the current
  visibility rules. The call reuses the current `$this` object, evaluates
  positional arguments left-to-right, and executes the resolved parent method
  body with the declaring parent class as the active method context.
- explicit self method calls by static method name:
  `self::method(...)` is supported in active instance method/constructor
  context when the resolved current-class or inherited method is a non-static
  public/protected/private instance method visible under the current rules.
  The call reuses the current `$this` object, evaluates positional arguments
  left-to-right, and executes the resolved method body with the declaring class
  as the active method context.
- `isset($object->name)` for direct public instance property operands on direct
  object variables, plus private property operands owned by the active
  declaring class and protected property operands owned by the active class or
  an ancestor
- exact uppercase built-in global constants `CASE_LOWER`, `CASE_UPPER`,
  `ARRAY_FILTER_USE_KEY`, `ARRAY_FILTER_USE_BOTH`, `SORT_REGULAR`,
  `SORT_NUMERIC`, and `SORT_STRING`, which evaluate to integers `0`, `1`,
  `2`, `1`, `0`, `1`, and `2`
- runtime-defined constants through `define($name, $value)` and
  `constant($name)` over the current unqualified string-name and scalar/array
  value subset; `defined($name)` reports whether a supported unqualified name
  exists in the current built-in/runtime-defined constant table; string-valued
  dynamic calls to `define`, `constant`, and `defined` use the same path
- bare reads of runtime-defined unqualified constants over the same current
  name/value subset; array constant values are cloned on lookup
- top-level single and grouped `const NAME = value;` declarations for
  unqualified constant names whose values use the current constant-expression
  subset: `null`, booleans, integers, floats, strings, short and long arrays
  with supported keys, unary expressions, binary expressions over those values,
  and bare references to previously defined unqualified constants or the
  current built-in `CASE_*`, `ARRAY_FILTER_*`, `SORT_REGULAR`,
  `SORT_NUMERIC`, and `SORT_STRING` constants
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
- `empty($name)`, `empty($array[$key])`, and
  `empty($object->publicProperty)` for direct variables, direct array-variable
  offset operands, and direct object-variable public-property operands over
  the current value model
- null coalescing `??` for direct static variables, direct array-variable
  offset operands, direct object-variable public-property operands, and
  private object-property operands owned by the active declaring class and
  protected operands owned by the active class or an ancestor over the current
  value model; undefined
  variables, missing
  array keys, missing supported object properties, non-array/non-object
  targets, null variables, null array values, and null supported object
  property values evaluate the fallback, while falsey non-null values such as
  `false`, `0`, `""`, and `"0"` are returned without evaluating the fallback
- null coalescing assignment `$name ??= expr`, `$array[$key] ??= expr`, and
  `$object->property ??= expr` for direct static variables, direct
  array-variable offset operands, direct object-variable public-property
  operands, and private object-property operands owned by the active declaring
  class plus protected operands owned by the active class or an ancestor, in
  statement position and parenthesized
  expression position; undefined and `null` variables, undefined/null arrays,
  missing array keys, null array values, and null supported object property
  values evaluate and store the right-hand expression, while existing
  non-null values are preserved without evaluating the right-hand expression.
  Expression forms return the assigned or existing value. Undefined object
  targets, non-object property targets, and missing property names fail with
  stable runtime diagnostics instead of materializing objects or dynamic
  properties
- builtins for the documented subset: `strlen`, `isset`, `empty`, `count`,
  `define`, `constant`, `defined`,
  `array_key_exists`, `array_key_first`, `array_key_last`, `array_is_list`,
  `array_values`, `array_keys`, `array_reverse`, `array_slice`, `array_chunk`,
  `array_pad`, `array_merge`, `array_replace`, `array_combine`,
  `array_intersect_key`,
  `array_diff_key`, `array_diff`, `array_intersect`, `array_unique`,
  `array_flip`, `array_change_key_case`, `array_column`, `array_fill_keys`, `array_count_values`, `array_sum`,
  `array_product`, `array_reduce`, `array_filter`, `array_map`, `in_array`,
  `array_search`, `gettype`, `is_null`, `is_bool`, `is_int`, `is_integer`,
  `is_long`, `is_float`, `is_double`, `is_string`, `is_array`, `is_scalar`,
  `is_numeric`, `is_countable`, `is_iterable`, `is_callable`,
  `function_exists`, `get_class`, `is_object`,
  `get_debug_type`,
  `class_exists`, `interface_exists`, `trait_exists`, `enum_exists`,
  `property_exists`, `method_exists`, `is_a`, `get_class_methods`, `get_class_vars`,
  `get_object_vars`, `get_mangled_object_vars`, `is_subclass_of`, `get_parent_class`,
  `get_declared_classes`, `get_declared_interfaces`, `get_declared_traits`,
  `spl_object_id`, `spl_object_hash`, `var_dump`, and `print_r`;
  `gettype` returns PHP legacy type names for the current value model
  (`NULL`, `boolean`, `integer`, `double`, `string`, `array`, and `object`);
  `is_null`, `is_bool`, `is_int`/`is_integer`/`is_long`,
  `is_float`/`is_double`, `is_string`, `is_array`, and `is_scalar` inspect
  the current boxed value variant without coercion. `is_numeric` returns true
  for integers, floats, and well-formed numeric strings using the same current
  numeric-string subset as scalar arithmetic. `is_countable` and `is_iterable`
  return true for arrays and false for the current scalar/null/object values.
  `is_callable($value)` supports the current string function-name subset: it
  returns true for names that resolve to current user functions or documented
  callable builtins, and false for missing names or non-string values.
  `is_callable($value, $syntax_only)` accepts boolean syntax-only flags; for
  string values, `true` reports callable syntax without resolving the name,
  while `false` uses the current function lookup path. Scalar non-string
  values return false. Syntax-only array callable checks accept only the
  current two-element `[class-or-object, method]` shape with integer keys `0`
  and `1`, where the first value is a string class name or current object and
  the second value is a string method name; this shape check does not resolve
  classes or methods. Normal array callable resolution checks the same
  two-element shape against current declared method metadata: object receivers
  are true for public declared methods, and class-string receivers are true for
  public static declared methods. Array callable dynamic invocation,
  callable-name output, object `__invoke` callables, private/protected
  caller-context method callability, first-class callable syntax,
  namespace/autoload behavior, exact native `TypeError` behavior, native
  lowering, and the environment-specific legacy `is_real` alias are not
  implemented.
  `function_exists($name)` checks string names against the current runtime
  function table, including current user functions and documented callable
  builtins, and rejects non-string names in the current subset.
  `get_class` returns the declared class name for current minimal object
  values, `is_object` reports whether a value is one of those current object
  values, `get_debug_type` returns scalar/array type names or the current
  object's declared class name, `class_exists` checks the current declared
  class metadata by string name without autoloading, `interface_exists`
  accepts string names and returns false for the current no-interface metadata
  model without autoloading, `trait_exists` accepts string names and returns
  false for the current no-trait metadata model without autoloading,
  `enum_exists` accepts string names and returns false for the current no-enum
  metadata model without autoloading,
  `property_exists` checks
  case-sensitive declared and inherited property metadata for current object values or
  string class names, `method_exists` checks case-insensitive declared and
  inherited method metadata for current object values or string class names,
  `get_class_methods` returns public declared and inherited method names in
  child-to-parent declaration order for current object values or declared
  string class names, `get_class_vars` returns public declared and inherited
  property names in child-to-parent declaration order with `null` values for
  declared string class names, `get_object_vars` returns public exact and
  inherited instance property names with their current values in
  parent-to-child slot order for current object values, `get_mangled_object_vars`
  returns inherited and exact-class public/protected/private instance slots
  with PHP-style mangled keys for current object values,
  `is_a` checks
  exact class identity and single-parent ancestor relationships over current
  object values or string class names when `allow_string` is true,
  `is_subclass_of` walks the current single-parent metadata chain after
  validating the supported object/string and class-name argument boundary,
  `get_parent_class` returns the immediate parent class name for supported
  object/declared-string inputs with parent metadata and false otherwise,
  `get_declared_classes` returns a zero-indexed array of classes
  declared in the current program in declaration order,
  `get_declared_interfaces` returns an empty zero-indexed array because
  interface declarations and internal interface metadata are not represented,
  `get_declared_traits` returns an empty zero-indexed array because trait
  declarations and internal trait metadata are not represented,
  and `print_r` can render current minimal object values
- structured runtime errors for undefined variables, arity mismatches,
  unsupported calls, division by zero, modulo by zero, non-numeric string
  arithmetic, and
  undefined functions, non-string dynamic function callees, unsupported
  `constant`/`defined` names and non-string `constant`/`defined` name
  arguments, duplicate constants, unsupported `define()` names, values, and
  legacy flags,
  unsupported array keys, undefined
  array keys, invalid array access including non-array
  `unset($array[$key])` targets, unsupported complex
  `empty` operands, non-array `array_key_first`/`array_key_last` operands,
  non-array `array_is_list` operands, non-array `array_reverse` operands,
  non-bool `array_reverse` preserve-key
  flag values, non-array `array_slice` operands, non-int `array_slice`
  offsets, non-int/non-null `array_slice` lengths, non-bool `array_slice`
  preserve-key flag values, non-array `array_chunk` operands,
  non-int/non-positive `array_chunk` lengths, non-bool `array_chunk`
  preserve-key flag values, non-array `array_pad` operands, non-int
  `array_pad` lengths, oversized `array_pad` padding requests, non-array
  `array_merge` operands, non-array `array_replace` operands including
  variadic replacement operands, non-array `array_combine` operands,
  `array_combine` length mismatches, unsupported lossy or non-finite float
  `array_combine` key values, unsupported non-null/bool/int/string/float
  `array_combine` key values, non-array `array_intersect_key` operands,
  non-array variadic `array_intersect_key` operands, non-array
  `array_diff_key` operands, non-array variadic `array_diff_key` operands,
  non-array `array_diff` operands, non-array variadic `array_diff` operands,
  unsupported non-scalar `array_diff` value comparisons,
  non-array `array_intersect` operands, non-array variadic
  `array_intersect` operands, unsupported non-scalar `array_intersect` value
  comparisons,
  non-array `array_unique` operands, unsupported non-scalar
  `array_unique` value comparisons, unsupported `array_unique` sort flags,
  non-array `array_flip` operands, unsupported non-int/string
  `array_flip` values, non-array `array_fill_keys` operands, unsupported
  lossy or non-finite float `array_fill_keys` key values, unsupported
  non-null/bool/int/string/float `array_fill_keys` key values, non-array
  `array_count_values` operands,
  unsupported non-int/string
  `array_count_values` values, non-array `array_sum` operands, unsupported
  non-numeric/non-scalar `array_sum` values, non-array `array_product`
  operands, unsupported non-numeric/non-scalar `array_product` values,
  non-array `array_reduce` operands, non-string or unresolved `array_reduce`
  callbacks, non-array `array_filter` operands, non-string non-null
  `array_filter` callbacks, invalid `array_filter` mode flags, non-array
  `array_map` operands, non-string or unresolved
  `array_map` callbacks,
  non-array variadic `array_map` operands,
  non-array `in_array`/`array_search` haystacks,
  non-bool `in_array`/`array_search` strict-mode flag values, unsupported
  non-scalar `array_keys` search-value comparisons, non-bool `array_keys`
  strict-mode flag values, unsupported non-scalar `in_array`/`array_search`
  comparisons, bitwise non-numeric mixed string operands, bitwise
  non-UTF-8 string results, unsupported unary bitwise-not operands, negative
  shift counts, bitwise array/object operands, duplicate constants, undefined
  constants, unsupported
  `global` declarations, duplicate class/member metadata, undefined classes,
  unsupported object instantiation, undefined object properties, invalid
  property targets, unsupported non-public property access, non-object
  `get_class` operands, unsupported `property_exists` object/class or
  property arguments, unsupported `method_exists` object/class or method
  arguments, unsupported `is_a` class-name or allow-string arguments,
  non-object `get_object_vars` operands, non-object
  `get_mangled_object_vars` operands,
  unsupported `get_parent_class` object/class arguments,
  unsupported `get_called_class()` calls before method/static class context
  exists,
  object-to-string conversion,
  unsupported strict identity array/object operands, invalid `foreach`
  iterables, invalid `break`/`continue` outside a loop, unsupported `continue;`
  inside `switch`, and runaway user-function recursion
- explicit parse diagnostics for unsupported function syntax: variadic
  parameters, variadic argument unpacking, reference parameters/returns,
  reference expressions, parameter type declarations, return type declarations,
  static local variable declarations inside functions, anonymous functions,
  arrow functions, named arguments, first-class callable syntax such as
  `strlen(...)` and `$callback(...)`, and `declare(strict_types=1)`
- explicit parse diagnostics for unsupported magic constants such as
  `__CLASS__`, `__TRAIT__`, `__METHOD__`, and `__NAMESPACE__`
- explicit parse diagnostics for unsupported include/require syntax:
  `include`, `include_once`, `require`, and `require_once`
- explicit parse diagnostics for unsupported direct `eval(...)` syntax
- explicit parse diagnostics for unsupported namespace and top-level `use`
  declaration syntax
- explicit parse diagnostics for unsupported namespace-qualified function and
  class names such as `App\fn()` and `new App\Box()`
- explicit parse diagnostics for unsupported magic class names in `new`
  expressions such as `new self()`, `new parent()`, and `new static()`
- explicit parse diagnostics for unsupported nested, namespace-aware, or
  dynamic-value `const` declarations
- stable runtime diagnostics for unsupported bare global constants outside the
  current built-in/runtime-defined slice, such as `PHP_VERSION`
- explicit parse diagnostics for unsupported array spread/reference elements
- explicit parse diagnostics for unsupported array/list destructuring
  assignment targets such as `[$name] = $array` and `list($name) = $array`
- explicit parse diagnostics for unsupported `unset(...)` forms outside the
  current direct-variable and direct array-offset statement subset
- explicit parse diagnostics for unsupported `unset($object->property)` before
  object property uninitialization semantics exist
- explicit parse diagnostics for unsupported `foreach` by-reference iteration,
  destructuring loop targets, and expression-position `foreach`
- explicit parse diagnostics for unsupported expression-position `for` and
  comma-separated `for` header expression lists
- explicit parse diagnostics for unsupported expression-position `do ... while`
- explicit parse diagnostics for unsupported alternate `if`/`elseif`/`else`
  colon/`endif` syntax
- explicit parse diagnostics for unsupported expression-position `switch` and
  malformed alternate colon/`endswitch` switch bodies
- explicit parse diagnostics for unsupported `break`/`continue` loop-depth
  arguments
- explicit parse diagnostics for unsupported exception syntax: `throw`,
  `try`, `catch`, and `finally`
- explicit parse diagnostics for unsupported generator `yield` and
  `yield from` expressions
- explicit parse diagnostics for unsupported PHP 8 `match` expressions
- explicit parse diagnostics for unsupported `goto` statements and labels
- explicit parse diagnostics for unsupported exponentiation syntax: `**` and
  `**=`
- explicit parse diagnostics for unsupported unparenthesized nested ternary
  expressions
- explicit parse diagnostics for unsupported assignment-expression forms
  outside direct static-variable `$name = expr`, including append-offset
  chained assignments and complex/nested targets
- explicit parse diagnostics for unsupported compound assignment targets
  outside direct static variables, direct array offsets, and direct object
  properties
- explicit parse diagnostics for unsupported increment/decrement targets
  outside direct static variables, direct array offsets, and direct object
  properties, plus chained increment/decrement expressions
- explicit parse diagnostics for unsupported chained coalescing and
  non-variable null coalescing assignment forms
- explicit parse diagnostics for unsupported object/class syntax: nested class
  declarations, broader inheritance forms beyond declared single-parent
  `extends`, interface declarations and implementation, trait declarations,
  trait use inside classes, enum declarations,
  `abstract`/`final`/`readonly` class
  modifiers, `abstract`/`final`/`readonly` class member modifiers,
  typed property declarations, property default values, multiple property
  declarations, class constant declarations,
  unsupported `clone` expressions, unsupported
  `instanceof` expressions, unsupported `ClassName::class` expressions,
  unsupported magic static receivers such as `self::` and `static::`,
  unsupported parent static property/constant access, anonymous class
  expressions, dynamic property names, static property access, static method
  calls, and class constant access
- explicit lex diagnostics for unsupported variable-variable syntax such as
  `$$name` and `${...}`
- explicit lex diagnostics for unsupported PHP attribute syntax beginning with
  `#[...]`; ordinary `#` comments, including `# [` with whitespace before the
  bracket, remain comments

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
- Null coalescing: `phpc run` supports an executable `??` slice where the left
  operand is a direct static variable, direct array-variable offset, or direct
  object-variable public property. The left operand uses PHP-style isset
  semantics for the current value model: undefined variables, missing array
  keys, missing public properties, null variables, null array values, null
  public property values, non-array array-offset targets, and non-object
  property targets use the fallback, while falsey non-null values are returned
  as-is and the fallback expression is not evaluated. `phpc run` also supports
  direct-variable `$name ??= expr`, direct array-offset `$array[$key] ??=
  expr`, and direct public object-property `$object->property ??= expr`
  statements. These statement forms evaluate the right-hand expression only
  when the target variable, array slot, or public property slot is undefined,
  missing, or null. Direct array-offset `??=` materializes undefined/null
  target variables as arrays; existing non-array targets fail with the current
  stable invalid-array-access diagnostic. Direct object-property `??=` writes
  only existing declared public properties on existing object values; missing
  properties, undefined target variables, and non-object target variables fail
  with stable diagnostics. Complex or nested `??` left operands, append-offset
  `??=` targets, dynamic property names, non-public visibility context, magic
  methods, unparenthesized chained coalescing, references/copy-on-write, exact
  native error objects, and native lowering remain unsupported.
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
  whose parameters/bodies use the existing function parser subset, including
  optional trailing commas after the final real parameter. `new
  ClassName(...)` looks up declared classes case-insensitively, initializes
  instance properties to `null`, skips static properties, treats object values
  as truthy, and lets direct `isset($object_variable)` return true. Public
  or inherited public instance `__construct` methods execute after object
  allocation with `$this` bound to the new object handle. Explicit
  `parent::__construct(...)` and `parent::method(...)` calls execute in active
  instance method/constructor context against the current single-parent chain.
  `self::method(...)` calls execute in active instance method/constructor
  context against the current class and inherited method chain.
  Protected constructors are callable from same-class or child-class method
  context through ordinary `new ClassName(...)` expressions.
  Undefined classes, constructor arguments for classes without constructors,
  private constructors without same-class construction context, protected
  constructors outside same-class/child-class construction context, top-level
  parent calls, parent calls in classes without parents, and static parent
  methods fail with stable runtime diagnostics. Public instance property reads
  and direct-variable writes work by static property name; property names are case-sensitive, and
  writes mutate the current object value stored in that variable.
  `isset($object->name)` works for direct object-variable operands and returns
  false for `null` slots, missing property names, undefined target variables,
  and non-object target variables. `get_class($object)` returns the declared
  class name stored on the current minimal object value and is also available
  through string-valued dynamic function calls. Undefined properties, property
  access on non-object values, non-public properties, and non-object
  `get_class` arguments still fail with stable runtime diagnostics.
  `is_object($value)` returns true for current minimal object values and false
  for scalars and arrays, and is available through string-valued dynamic
  function calls. `get_debug_type($value)` returns current scalar/array type
  names (`null`, `bool`, `int`, `float`, `string`, `array`) and the declared
  class name for current minimal object values, and is available through
  string-valued dynamic function calls. `class_exists($name)` and
  `class_exists($name, $autoload)` accept string class names, perform
  case-insensitive lookup against classes declared in the current parsed
  program, accept current bool-like scalar autoload flags, and are available
  through string-valued dynamic function calls. The autoload flag does not
  trigger autoloading in the current subset.
  `interface_exists($name)` and `interface_exists($name, $autoload)` accept
  string interface names, return false for all supported calls because
  interface metadata is not represented yet, and are available through
  string-valued dynamic function calls. The autoload flag accepts current
  bool-like scalar values and does not trigger autoloading.
  `trait_exists($name)` and `trait_exists($name, $autoload)` accept string
  trait names, return false for all supported calls because trait metadata is
  not represented yet, and are available through string-valued dynamic function
  calls. The autoload flag accepts current bool-like scalar values and does not
  trigger autoloading.
  `enum_exists($name)` and `enum_exists($name, $autoload)` accept string enum
  names, return false for all supported calls because enum metadata is not
  represented yet, and are available through string-valued dynamic function
  calls. The autoload flag accepts current bool-like scalar values and does not
  trigger autoloading.
  `property_exists($object_or_class, $property)` accepts a current object value
  or string class name and a string property name. It checks the current
  declared and inherited property metadata with case-sensitive property names,
  reports public/protected/private and static properties on the exact class as
  existing, reports inherited public/protected/static properties as existing,
  keeps inherited private properties invisible, returns false for missing
  properties or missing string class names, and is available through
  string-valued dynamic function calls.
  `method_exists($object_or_class, $method)` accepts a current object value or
  string class name and a string method name. It checks the current declared
  method metadata with case-insensitive method names, reports
  public/protected/private and static methods as existing, returns false for
  missing methods or missing string class names, and is available through
  string-valued dynamic function calls.
  `get_class_methods($object_or_class)` accepts a current object value or a
  declared string class name and returns a zero-indexed array of public method
  names in declaration order, including public static methods. It is available
  through string-valued dynamic function calls.
  `get_class_vars($class_name)` accepts declared string class names and returns
  an array of public declared and inherited properties in child-to-parent
  declaration order, including public static properties, with `null` values
  because property defaults are not implemented. It is available through
  string-valued dynamic function calls.
  `get_object_vars($object)` accepts current object values and returns an array
  of public exact and inherited instance property names in parent-to-child slot
  order with their current slot values. Protected/private slots and static
  properties are not included. It is available through string-valued dynamic
  function calls.
  Direct `empty($object->name)` accepts direct object-variable public-property
  operands, returns true for falsey public property slots, missing properties,
  undefined target variables, and non-object target variables, and uses a
  stable unsupported-property diagnostic for non-public properties.
  `get_mangled_object_vars($object)` accepts current object values and returns
  public, protected, and private instance slots in declaration order. Public
  property keys are emitted as the declared name, protected property keys are
  emitted as `\0*\0name`, and private property keys are emitted with the
  declaring class name as `\0ClassName\0name`; static properties are omitted.
  Dynamic properties, property defaults, trait/interface properties, and
  non-public visibility-context behavior beyond the current declaring-class
  method context are not represented yet. It is available through
  string-valued dynamic function calls.
  `is_a($object_or_class, $class_name)` accepts current object values and
  checks exact class identity or a single-parent ancestor relationship against
  the current declared class metadata using case-insensitive class-name lookup.
  `is_a($object_or_class, $class_name, true)` also accepts a string first
  argument and checks the same relationship. A false or omitted `allow_string`
  flag makes string first arguments return false. Missing source or target
  class names return false, and string-valued dynamic calls to `is_a` use the
  same path.
  `is_subclass_of($object_or_class, $class_name[, $allow_string])` accepts the
  current object/string first-argument subset and string class names, considers
  two-argument string first arguments and three-argument string first arguments
  only when `allow_string` is true, returns false for exact-class,
  missing-class, and no-parent cases, and is available through string-valued
  dynamic calls.
  `get_parent_class($object_or_class)` accepts current object values or
  declared string class names, returns the immediate parent class name when
  one is recorded and false otherwise, and is available through string-valued
  dynamic calls.
  `get_declared_classes()` returns a zero-indexed array of classes declared in
  the current parsed program in declaration order and is available through
  string-valued dynamic calls.
  `get_declared_interfaces()` returns an empty zero-indexed array because
  interface declarations and internal interface metadata are not represented
  yet, and is available through string-valued dynamic calls.
  `get_declared_traits()` returns an empty zero-indexed array because trait
  declarations and internal trait metadata are not represented yet, and is
  available through string-valued dynamic calls.
  Static member expressions through `::`,
  including `ClassName::$prop`, `ClassName::method()`, and `ClassName::CONST`,
  fail with stable parse diagnostics. `clone $object` expressions fail with a
  stable parse diagnostic before object handle copying or `__clone` dispatch is
  implemented. `$object instanceof ClassName` expressions fail with a stable
  parse diagnostic before class/interface relationship checks exist.
  `ClassName::class` expressions fail with a stable parse diagnostic before
  class-name constant resolution exists. `static::$prop`,
  `static::method(...)`, `static::CONST`, and `static::class` fail with
  distinct stable parse diagnostics before late-static-binding resolution,
  static storage, static dispatch, or class constants exist.
  `parent::method(...)` and `self::method(...)` calls are the supported magic
  receiver slices; self/parent static property access, self/parent class
  constants, and `self::class`/`parent::class` fail with stable parse
  diagnostics.
  Public, same-class private, and protected same-class/child instance method
  dispatch supports static method names, inherited method lookup, and scoped
  `$this` binding. Dynamic method names, dynamic property names, non-public
  property/constructor visibility context, static storage, class constants,
  shallow/deep clone property copying, `__clone`, property override
  compatibility, broader
  `parent::`/`self::`/`static::`, broader inheritance/interface relationship checks,
  namespace/autoload-aware class resolution, aliases and imports for class
  names, built-in/internal/extension class entries for `get_declared_classes`,
  declared/built-in/internal interface entries for `get_declared_interfaces`,
  declared/built-in/internal trait entries for `get_declared_traits`,
  anonymous classes, exact native class/interface/trait ordering, exact PHP
  `Error` objects, and native object lowering are not implemented.
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
  $array)` checks existing slots without filtering out `null` values for
  integer/string keys, plus `null` keys coerced to the empty-string key,
  boolean keys coerced to integer `0`/`1`, and integral finite float keys
  coerced to integers. Lossy and non-finite float key coercions remain
  unsupported. It is also available through
  string-valued dynamic function calls. `array_key_first($array)` returns the
  first inserted integer or string key as an `int` or `string`, and
  `array_key_last($array)` returns the last
  inserted integer or string key. Both return `null` for an empty array and are
  available through string-valued dynamic function calls. `array_is_list($array)`
  returns true for empty arrays and arrays whose entries are ordered with exact
  integer keys `0..n-1`; numeric string keys such as `"0"` participate through
  the current array-key normalization, while string keys such as `"01"`, gaps,
  negative keys, and out-of-order integer keys return false. It is also
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
  `array_slice($array, $offset)` accepts integer offsets, returns entries from
  that insertion-order offset to the end, supports negative offsets counted
  back from the end, reindexes integer-keyed entries from zero, preserves
  string keys, and is available through string-valued dynamic function calls.
  `array_slice($array, $offset, $length)` accepts integer lengths, including
  positive lengths, zero, and negative lengths counted back from the end of the
  input array, while using the same default integer-key reindexing and
  string-key preservation. `array_slice($array, $offset, null)` treats the
  null length as a to-end slice. `array_slice($array, $offset, $length, true)`
  and `array_slice($array, $offset, null, true)` preserve integer and string
  keys; boolean `false` uses the default integer-key reindexing path.
  `array_chunk($array, $length)` accepts arrays and positive integer lengths,
  splits values in insertion order into nested arrays of that size, reindexes
  every inner chunk from integer key zero, returns an empty array for empty
  input arrays, and is available through string-valued dynamic function calls.
  `array_chunk($array, $length, true)` preserves original integer and string
  keys inside each chunk; boolean `false` uses the default chunk-key
  reindexing path. `array_pad($array, $length, $value)` accepts arrays and
  integer lengths, returns an unchanged copy when `abs($length)` is not larger
  than the input size, right-pads for positive lengths, left-pads for negative
  lengths, preserves string keys, and reindexes integer-keyed input entries
  from zero when padding is needed. It is also available through string-valued
  dynamic function calls.
  `array_merge()` returns an empty array. `array_merge($array, ...)` accepts
  zero or more array operands, processes them left to right in insertion order,
  appends and reindexes integer-keyed entries from zero, preserves string keys,
  and overwrites duplicate string-key values with later values without moving
  the original string-key slot. It is also available through string-valued
  dynamic function calls. `array_replace($array, ...$replacements)` accepts one
  or more arrays, starts with a clone of the first array, applies replacement
  arrays left to right, overwrites matching integer or string keys without
  moving existing slots, appends new replacement keys in replacement insertion
  order, preserves integer and string keys, and is available through
  string-valued dynamic function calls.
  `array_combine($keys, $values)` accepts two arrays
  with the same number of entries, reads key values and value values in
  insertion-order lockstep, maps null and false key values to the empty string
  key, maps true key values through the string `"1"` key normalization path,
  uses integer and integral finite float key values directly as integer result
  keys, normalizes string key values through the current PHP-style decimal
  string key rules, stores cloned values from the second array, and overwrites
  duplicate result keys with later pairs
  without moving the first result-key position. It is also available through
  string-valued dynamic function calls.
  `array_intersect_key($array, ...$arrays)` accepts two or more arrays, returns
  entries from the first array whose integer/string keys are present in every
  subsequent array, preserves the first array's keys, values, and insertion
  order, and is also available through string-valued dynamic function calls.
  `array_diff_key($array, ...$arrays)` accepts two or more arrays, returns
  entries from the first array whose integer/string keys are absent from every
  subsequent array, preserves the first array's keys, values, and insertion
  order, and is also available through string-valued dynamic function calls.
  `array_diff($array, ...$arrays)` accepts two or more arrays, compares
  current scalar values through their PHP string forms, returns entries from
  the first array whose scalar comparison value is absent from every subsequent
  array, preserves the first array's keys, values, insertion order, and
  append-index behavior, and is also available through string-valued dynamic
  function calls.
  `array_intersect($array, ...$arrays)` accepts two or more arrays, compares
  current scalar values through their PHP string forms, returns entries from
  the first array whose scalar comparison value is present in every subsequent
  array, preserves the first array's keys, values, insertion order, and
  append-index behavior, and is also available through string-valued dynamic
  function calls.
  `array_unique($array)` and `array_unique($array, SORT_STRING)` compare
  current scalar values through their PHP string forms,
  `array_unique($array, SORT_REGULAR)` compares current scalar values through
  the same loose scalar comparison rules used by the interpreter, and
  `array_unique($array, SORT_NUMERIC)` compares values through the current
  scalar numeric-coercion subset. All supported modes keep the first matching
  entry, preserve kept integer/string keys and insertion order, use kept
  integer keys for later append behavior, and are also available through
  string-valued dynamic function calls.
  `array_flip($array)` accepts arrays, converts
  integer and string array values into result keys using the current array-key
  normalization rules, writes each original integer/string key as the result
  value, overwrites duplicate flipped keys with later values without moving the
  first flipped-key slot, and is available through string-valued dynamic
  function calls. `array_fill_keys($keys, $value)` accepts an array of
  null/boolean/integer/string/integral-finite-float key values, creates a new
  ordered array using those values as normalized result keys, stores the
  supplied value in each result slot, and overwrites duplicate result keys with
  later entries without moving the first key position. It is also available
  through string-valued dynamic function calls. `array_count_values($array)` accepts arrays whose values are integers
  or strings, counts values in insertion order using the current array-key
  normalization rules for string values, stores integer counts as result
  values, and is available through string-valued dynamic function calls.
  `array_sum($array)` accepts arrays whose values are `null`, booleans,
  integers, floats, or well-formed numeric strings under the current scalar
  numeric-coercion rules, accumulates as an integer until a float input or
  integer overflow promotes the result to float, returns integer zero for an
  empty array, and is available through string-valued dynamic function calls.
  `array_product($array)` accepts the same current numeric scalar value subset,
  multiplies values in insertion order, accumulates as an integer until a float
  input or integer overflow promotes the result to float, returns integer one
  for an empty array, and is available through string-valued dynamic function
  calls.
  `array_reduce($array, $callback)` and `array_reduce($array, $callback,
  $initial)` accept arrays and callbacks that evaluate to string function names
  resolving to current user functions or callable builtins, invoke the callback
  once per value in insertion order with the accumulator and current value,
  start the accumulator at `null` when no initial value is supplied, return the
  supplied initial value for empty arrays when present, and are available
  through string-valued dynamic calls to `array_reduce`.
  `array_filter($array)` without a callback, `array_filter($array, null)`,
  and `array_filter($array, null, $mode)` with integer mode flags `0`, `1`,
  or `2`, finite integral float mode flags, integral numeric string mode flags
  that trim and parse to `0`, `1`, or `2`, or boolean mode flags accept arrays
  only, remove values that are falsey under the current PHP-shaped truthiness
  rules, preserve the original integer/string keys and insertion order of kept
  entries, and are available through string-valued dynamic function calls.
  `array_filter($array, $callback)` accepts callbacks that evaluate to string
  function names resolving to current user functions or callable builtins,
  invokes the callback once per value in insertion order with the value as the
  only argument, preserves keys whose callback result is truthy, accepts
  explicit integer mode flag `0`, finite integral float mode flag `0.0`,
  integral numeric string mode flag `"0"`, and boolean mode flag `false` for
  the same value-only callback path, and is also available through
  string-valued dynamic calls to `array_filter`.
  `array_filter($array, $callback, 2)` plus finite integral float and integral
  numeric string mode values that parse to `2` invoke the same string-valued
  callback subset once per entry with the current integer or string key as the
  only argument, preserving keys whose callback result is truthy.
  `array_filter($array, $callback, 1)` and `array_filter($array, $callback,
  true)`, plus finite integral float and integral numeric string mode values
  that parse to `1`, invoke that callback subset once per entry with the value
  and then the current integer or string key as arguments, preserving keys
  whose callback result is truthy.
  `array_map(null, $array)` returns an identity copy of one input array while
  preserving integer/string keys and insertion order. `array_map(null,
  $array, ...)` with two or more input arrays returns a reindexed array of
  tuple arrays, zipping values from each input in insertion order up to the
  longest input and padding missing values with `null`.
  `array_map($callback, $array, ...)` accepts callbacks that evaluate to string
  function names resolving to current user functions or callable builtins. The
  one-array string-callback form invokes the callback once per value in
  insertion order with the value as the only argument and preserves the
  original integer/string keys. Multi-array string-callback forms invoke the
  callback with one value from each input array in insertion-order lockstep up
  to the longest input, supply `null` for missing values from shorter arrays,
  and return mapped values reindexed with integer keys starting at zero. These
  forms are available through string-valued dynamic calls to `array_map`.
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
  `array_key_first`, `array_key_last`, `array_is_list`, `array_values`,
  `array_keys`, `array_reverse`, `array_slice`, `array_chunk`, `array_pad`,
  `array_merge`, `array_replace`, `array_combine`, `array_intersect_key`,
  `array_diff_key`,
  `array_diff`, `array_intersect`, `array_unique`, `array_flip`,
  `array_fill_keys`, `array_count_values`, `array_sum`, `array_product`,
  `array_reduce` in the current string-callback form with optional initial
  values,
  `array_filter` in the current no-callback, null-callback, value-only
  string-callback, key-only string-callback, and value/key string-callback
  forms, including explicit integer mode flags `0`, `1`, and `2`,
  integer-string mode values that trim and parse to those integers, plus
  boolean mode flags `false` and `true`,
  `array_map` in the current one-array null-callback identity form, variadic
  null-callback zip form, and one-array and variadic string-callback forms,
  `in_array`, `array_search`, both current `foreach` array forms, direct
  array-offset `unset`, multiple supported `unset(...)` operands, `print_r`,
  and `var_dump` are implemented for this ordered value model.
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
  `continue 2;` are rejected with stable parse diagnostics. Exception syntax
  is rejected separately at parse time, and native lowering is not implemented.
- Switch: statement-form brace `switch` and alternate
  `switch (...): ... endswitch;` execute in `phpc run` over the current scalar
  loose-comparison subset. The switch expression is evaluated once, case
  expressions are evaluated in source order until the first loose `==` match,
  `default` is used only when no case matches, and execution falls through
  later labels until a `break;`, `return`, or the end of the switch body.
  Both `:` and `;` are accepted as `case`/`default` separators. Arrays,
  objects, resources, expression-form switch, malformed alternate switch
  bodies, `continue;` inside switch, and native lowering are not implemented.
- Runtime errors: diagnostics have stable messages and source locations, but
  they are not PHP `Throwable` objects and there is no warning/notice recovery
  mode yet. Representative runtime errors are covered by committed `phpc run`
  CLI snapshots that record exit code, stdout, and stderr for undefined
  variables, user-function arity mismatches, unsupported scalar `count()` calls,
  duplicate `define()` constant definitions, unsupported `define()` names,
  unsupported `define()` values, unsupported `define()` legacy flags,
  unsupported `defined()` names and non-string name arguments,
  unsupported array keys,
  undefined array keys, invalid `array_key_exists` keys, non-array
  `array_key_exists` operands, non-array `array_key_first` or
  `array_key_last` operands, non-array `array_is_list` operands,
  non-array `array_values` operands, non-array
  `array_keys` operands, unsupported `array_keys` search-value comparisons,
  non-bool `array_keys` strict-mode flag values,
  non-array `array_reverse` operands, non-bool
  `array_reverse` preserve-key flag values, non-array `array_slice`
  operands, non-int `array_slice` offsets, non-int/non-null `array_slice`
  lengths, non-bool `array_slice` preserve-key flag values, non-array
  `array_chunk` operands, non-int/non-positive `array_chunk` lengths,
  non-bool `array_chunk` preserve-key flag values, non-array `array_pad`
  operands, non-int `array_pad` lengths, oversized `array_pad` padding
  requests, non-array `array_merge` operands, non-array `array_replace`
  operands including variadic replacement operands, non-array
  `array_combine` operands, `array_combine` length mismatches, unsupported
  lossy or non-finite float `array_combine` key values, unsupported
  non-null/bool/int/string/float `array_combine` key values, non-array
  `array_intersect_key` operands,
  non-array variadic `array_intersect_key` operands, non-array
  `array_diff_key` operands, non-array variadic `array_diff_key` operands,
  non-array `array_diff` operands, non-array variadic `array_diff` operands,
  unsupported non-scalar `array_diff` value comparisons,
  non-array `array_intersect` operands, non-array variadic
  `array_intersect` operands, unsupported non-scalar `array_intersect` value
  comparisons,
  non-array `array_unique` operands, unsupported non-scalar
  `array_unique` value comparisons, unsupported `array_unique` sort flags,
  non-array `array_flip` operands, unsupported non-int/string
  `array_flip` values, non-array `array_fill_keys` operands, unsupported
  lossy or non-finite float `array_fill_keys` key values, unsupported
  non-null/bool/int/string/float `array_fill_keys` key values, non-array
  `array_count_values` operands,
  unsupported non-int/string
  `array_count_values` values, non-array `array_sum` operands, unsupported
  non-numeric/non-scalar `array_sum` values, non-array `array_product`
  operands, unsupported non-numeric/non-scalar `array_product` values,
  non-array `array_reduce` operands, non-string and unresolved `array_reduce`
  callbacks, non-array `array_filter` operands, non-string non-null
  `array_filter` callbacks, invalid `array_filter` mode flags,
  non-array `array_map` operands, non-string and unresolved `array_map`
  callbacks, non-array variadic `array_map` operands, non-array `in_array` operands,
  non-array `array_search` operands, non-array `foreach` iterables, non-bool
  `in_array`/`array_search` strict-mode flag values, and array-value
  comparisons for `in_array`/`array_search`,
  unsupported complex `empty` operands, non-array `unset($array[$key])`
  targets, unresolved dynamic function callees, duplicate constants, undefined
  constants, division by zero, non-numeric string arithmetic, duplicate class
  metadata, undefined classes, undefined object properties, invalid property
  targets, non-public property access, non-object `get_class` operands,
  non-string `class_exists` names, null/array/object `class_exists` autoload
  flags, non-string `interface_exists` names, null/array/object
  `interface_exists` autoload flags, non-string `trait_exists` names,
  null/array/object `trait_exists` autoload flags, non-string `enum_exists`
  names, null/array/object `enum_exists` autoload flags,
  non-bool `is_callable` syntax-only flags,
  non-string `function_exists` names,
  non-string `is_a` class names, non-bool `is_a` allow_string flags,
  non-object/non-string `is_subclass_of` first arguments, non-string
  `is_subclass_of` class names, non-bool `is_subclass_of` allow_string flags,
  non-object/non-string `get_parent_class` arguments and missing
  `get_parent_class` string classes, non-object/non-string
  `get_class_methods` arguments and missing `get_class_methods` string
  classes, non-string `get_class_vars` arguments and missing
  `get_class_vars` string classes, non-object `get_object_vars` arguments,
  non-object `get_mangled_object_vars` arguments,
  extra `get_declared_interfaces` or `get_declared_traits` arguments,
  unsupported `get_called_class()` calls before method/static class context
  exists, non-object `spl_object_id` operands, non-object `spl_object_hash`
  operands,
  object-to-string conversion, invalid `break`/`continue` outside a loop,
  unsupported `continue;` inside `switch`, and runaway user-function recursion.
- Native codegen: LLVM IR/assembly supports only straight-line echo/assignment
  for the current statically lowerable scalar subset: literal `null`,
  booleans, integers, floats, and strings; direct static-variable assignments
  from those values; later direct static-variable assignments that overwrite
  earlier lowerable scalar values in the same straight-line lowering pass;
  direct reads of previously assigned static variables; direct `isset($name)`
  checks over the current static-variable map; and `echo`/`print`.
  Native echo conversion is limited to this static scalar path: `null` and
  `false` emit nothing, `true` emits `1`, integers use `%lld`, floats use
  `%g`, and strings are emitted through generated static string constants.
  Native binary arithmetic currently lowers `+`, `-`, and `*` when both
  operands are already same-type lowerable floats, or when both operands are
  lowerable integers and the integer result is statically proven not to
  overflow, in the same straight-line subset. Finite same-type float `+`, `-`,
  and `*` results remain bounded and tracked for later strict-identity
  folding when every possible result is proven. It lowers integer `%` only
  when the divisor is a statically known positive integer in that subset, and
  statically known modulo results remain tracked for later checked integer
  arithmetic. Tracked integer expression operands and integer literal operands
  for `$x % 1` fold to zero, and bounded tracked integer expression operands
  whose possible values all produce the same remainder for a positive literal
  divisor fold to that remainder. Integer modulo by one also folds after both
  operands lower when the dividend is intentionally untracked, such as an
  overflow-sensitive shift result; other modulo cases still require a
  statically known positive divisor and keep the documented runtime-check
  boundary. Identical tracked integer expression operands and identical
  integer literal operands for `-` fold to zero without a redundant native
  subtraction, and identical tracked finite float expression operands and
  identical finite float literals for `-` fold to `0.0` without a redundant
  native subtraction. Identical integer subtraction also folds after both
  operands lower when the value is intentionally untracked, such as
  overflow-sensitive shift results; other non-identity arithmetic with such
  values still rejects because exact overflow tracking is unavailable. Tracked integer expression operands and integer literal
  operands for `$x + 0`, `0 + $x`, and `$x - 0` reuse the existing value, and
  tracked integer expression operands and integer literal operands for
  `$x * 1` and `1 * $x` also reuse the existing value. Tracked integer
  expression operands and integer literal operands for `$x * 0` and `0 * $x`
  fold to zero. The `+ 0`, `- 0`, `* 1`, and `* 0` identity or annihilator
  forms also fold after both operands lower when the other integer operand is
  intentionally untracked, such as overflow-sensitive shift results;
  non-identity arithmetic with such values still rejects because exact
  overflow tracking is unavailable. Tracked finite float expression operands
  and finite float literals for nonzero `$x + 0.0`, `0.0 + $x`, and `$x - 0.0`, and for
  `$x * 1.0` and `1.0 * $x`, reuse the existing expression. Single-result
  statically known nonzero finite `0.0 - $x` folds to the known negated float
  literal. Tracked finite positive float expression operands and finite
  positive float literals for `$x * 0.0` and `0.0 * $x` fold to positive
  `0.0`. Single-result statically known nonzero finite `$x * -1.0` and
  `-1.0 * $x` fold to the known negated float literal. Possible signed zero,
  negative, and non-finite float identity/subtraction or multiplication-by-zero
  cases, and signed-zero-sensitive multiplication by `-1.0`, stay emitted or
  rejected rather than being folded.
  Mixed int/float arithmetic, PHP numeric coercions, `/`, dynamic or non-positive modulo divisors,
  division/modulo zero checks, modulo coercions, negative-divisor and min-int
  modulo edge cases, modulo results
  that are not statically known enough for later checked arithmetic, integer
  overflow promotion, float overflow/INF/NAN result tracking, references/copy-on-write
  behavior, and exact native error objects remain unsupported. Mixed int/float
  `+`, `-`, and `*` operands are rejected with a
  mixed-numeric-specific diagnostic until generated code has PHP numeric
  promotion and exact result typing. Boolean, null, and string operands in
  `+`, `-`, and `*` are rejected with a scalar-coercion-specific diagnostic
  until generated code has PHP numeric coercion and string numeric parsing.
  Overflow-sensitive or not-statically-proven integer `+`, `-`, and `*` cases
  are rejected with an integer-overflow-specific diagnostic until generated
  code has PHP integer overflow promotion and runtime checks. Native `/` is
  rejected with a division-specific codegen diagnostic until generated code has
  PHP division semantics, runtime zero checks, and no misleading integer
  truncation. Dynamic, zero, or non-positive
  integer modulo divisors are rejected with a modulo-specific codegen
  diagnostic until native runtime checks exist; the remaining arithmetic gaps
  are rejected with a specific
  codegen diagnostic. Native reads of variables that were not statically
  assigned earlier in the same straight-line lowerer are rejected with a
  specific codegen diagnostic until generated code has native symbol-table storage,
  undefined-variable diagnostics, references/copy-on-write behavior, and exact
  native error objects. Native string concatenation `.` currently lowers when
  both operands are already lowerable strings in the same straight-line subset,
  including ternary operands that prove one static string result; the result is
  folded into a generated static string constant. Empty-string concatenation
  identity also folds for already-lowerable string operands, including
  untracked string pointer expressions: `$text . ""` and `"" . $text` reuse
  `$text` without runtime string allocation. PHP scalar-to-string conversion
  for concatenation, non-empty ambiguous string expressions, arrays, objects,
  resources, runtime string allocation, references/copy-on-write behavior, and
  exact native error objects remain unsupported and are rejected with a
  specific codegen diagnostic. Native comparison lowering currently accepts
  same-type `null`, boolean, integer, finite float, known ASCII nonnumeric
  NUL-free string loose/ordering comparisons, and identical string-pointer
  self-comparisons for `==`, `!=`, `<`, `<=`, `>`, and `>=`, and strict
  identity `===`/`!==` for already lowerable `null`, integers, booleans,
  floats, and strings in the same straight-line subset.
  Static same-type scalar
  identity folds at compile time, bounded integer, float, string, and boolean
  identity fold when all possible `===`/`!==` outcomes are proven identical.
  Identical lowerable dynamic scalar operands fold for integers, booleans,
  already-lowerable string pointers, and finite tracked floats, so `$x === $x`
  and `$x !== $x` avoid runtime comparisons in those safe scalar cases.
  Identical lowerable integer operands also fold for loose/ordering
  comparisons, including intentionally untracked integer expressions such as
  overflow-sensitive shift results: `$x == $x`, `$x <= $x`, and `$x >= $x`
  fold true, while `$x != $x`, `$x < $x`, and `$x > $x` fold false.
  Dynamic boolean expression operands compared with boolean literals fold for
  `$flag === true`, `true === $flag`, `$flag !== false`, and `false !== $flag`
  by reusing the original native boolean expression, and inverse forms such as
  `$flag === false`, `false === $flag`, `$flag !== true`, and `true !== $flag`
  use the native boolean inversion path.
  Dynamic boolean expression operands compared loosely with boolean literals
  fold for `$flag == true`, `true == $flag`, `$flag != false`, and
  `false != $flag` by reusing the native boolean expression, while inverse
  forms such as `$flag == false`, `false == $flag`, `$flag != true`, and
  `true != $flag` use the native boolean inversion path.
  Dynamic boolean expression operands ordered against boolean literals also
  fold within boolean semantics, reusing the expression, inverting it, or
  folding to a static boolean for cases such as `$flag > false`,
  `$flag < true`, `$flag <= true`, and `true >= $flag`.
  Same-type integer and finite-float loose/ordering comparisons whose tracked
  possible operands prove one result fold to a static boolean. Literal-only
  comparisons still fold, while ambiguous tracked finite-float comparisons
  stay emitted as native comparisons.
  Boolean expression comparisons whose tracked possible operands prove one
  loose/ordering result also fold to that static boolean without emitting a
  redundant native boolean comparison. Identical native boolean expression
  operands also fold for loose/ordering comparisons, including ambiguous
  boolean expressions: `$flag == $flag`, `$flag <= $flag`, and `$flag >=
  $flag` fold true, while `$flag != $flag`, `$flag < $flag`, and `$flag >
  $flag` fold false. Other ambiguous boolean expression comparisons stay
  emitted. Identical native string pointer operands also fold for
  loose/ordering comparisons, including untracked string pointer expressions
  whose possible value set exceeds the current small tracker: `$text ==
  $text`, `$text <= $text`, and `$text >= $text` fold true, while `$text !=
  $text`, `$text < $text`, and `$text > $text` fold false. Non-identical
  unknown string comparisons stay rejected.
  Statically known integer strict-identity comparison results remain tracked
  for later boolean scalar lowering even when the comparison itself stays
  emitted as `icmp`. Same-type ambiguous dynamic integer, boolean, float, and
  already-lowerable string pointer identity lower through native comparisons
  and PHP-shaped boolean echo output, and already lowerable mixed scalar
  operands with different PHP scalar types fold without emitting runtime
  comparison calls. Ambiguous dynamic string identity uses `strcmp` for string
  pointers produced by the current native string ternary subset. Known ASCII
  nonnumeric string loose/ordering comparisons fold to a static boolean when
  every possible safe string outcome matches; ambiguous safe string
  loose/ordering comparisons lower through `strcmp`. Statically known boolean,
  integer, and finite-float loose/ordering comparison results remain tracked
  for later boolean scalar lowering even when the comparison itself stays
  emitted as `icmp`/`fcmp`; ambiguous bounded boolean, finite-float, or string
  loose/ordering comparison results remain dynamic and untracked.
  Ambiguous bounded integer, float, string, or boolean identity, broader
  value-correlation proofs across related expressions such as `$x` and `!$x`,
  numeric-looking, non-identical unknown, non-ASCII, or NUL-containing string loose/ordering comparisons,
  mixed null or other mixed-type comparisons, untracked or
  non-finite float comparisons, dynamic null identity beyond static/type-only folds, PHP
  truthiness conversion for loose logical operands, array/object comparisons,
  non-lowerable float sources, dynamic string allocation beyond the static
  straight-line subset, PHP comparison coercions, and non-scalar comparison
  diagnostics remain unsupported and are rejected with a specific
  codegen diagnostic.
  Native unary lowering currently accepts unary minus on already lowerable
  integers or floats and logical not on already lowerable booleans or native
  boolean expression results, on `null`, or on known integers, finite floats,
  and strings whose possible values all have the same PHP truthiness, in the same
  straight-line subset.
  Dynamic boolean double logical-not expressions such as `!!$flag` reuse the
  original native boolean expression instead of emitting redundant inversions.
  Double logical-not over known scalar operands such as integers, finite floats,
  strings, and `null` folds through the same known-truthiness subset without
  emitting boolean operations.
  Native lowering folds logical not over single-result statically known native
  boolean expression operands to the known boolean result in LLVM IR and in the
  C assembly fallback when the C boolean expression has a tracked result.
  Known numeric logical-not folds to a static boolean for zero and nonzero
  known integer/finite-float operands when all possible values have the same
  truthiness. Known string logical-not folds to a static boolean for `""`,
  `"0"`, and known-truthy string operands when all possible string values have
  the same truthiness. Null logical-not folds to `true` without claiming
  broader null truthiness beyond the documented logical binary folding subset.
  Integer
  unary-minus results remain statically tracked for later checked integer
  arithmetic when all bounded possible negation results are proven not to
  overflow; single-result statically known integer operands fold to the known
  negated result without a redundant native unary-minus operation. Finite
  float unary-minus results remain tracked for later
  strict-identity folding when every possible negation result is proven;
  single-result statically known nonzero finite float operands fold to the
  known negated result without a redundant native unary-minus operation.
  Boolean, string, null, array, and object unary-minus operands, PHP numeric
  coercion, ambiguous numeric or string logical-not truthiness, untracked
  numeric/string logical-not expressions, non-finite float logical-not
  truthiness, null truthiness outside logical-not, other truthiness
  conversion, unary integer overflow behavior, float overflow/INF/NAN result tracking,
  references/copy-on-write side-effect behavior, and exact native error objects
  remain unsupported and are rejected with a specific codegen diagnostic.
  Native logical operators `&&`, `||`, `and`, `xor`, and `or` lower only when
  both operands are already lowerable booleans or native boolean expression
  results, or when both already-lowerable scalar operands have one statically
  known PHP truthiness result, in the same straight-line subset. Static boolean
  pairs fold at compile time, and static boolean identity and annihilator edges
  such as `true || $flag`, `false && $flag`, `$flag && true`, and `$flag xor
  false` preserve the proven boolean result for later scalar lowering.
  Identical native boolean expression operands for `&&`/`and` and `||`/`or`
  reuse the existing expression without a redundant native boolean operation,
  and identical native boolean expression operands for `xor` fold to `false`.
  Native boolean expression operations whose tracked possible operands prove
  one result fold to that static boolean without a redundant native boolean
  operation. Known scalar logical operands whose null, integer, finite-float, or
  string truthiness is unambiguous fold to a static boolean result without
  emitting a native boolean operation. Statically decisive known-left
  `&&`/`and` and `||`/`or` short-circuit cases such as `false && rhs` and
  `true || rhs` lower without lowering the skipped right-hand operand. Other
  dynamic boolean expressions lower to native boolean operations with PHP-shaped
  boolean echo output. Cases that require general PHP truthiness conversion,
  dynamic short-circuiting, `xor` right-hand skipping, selected/evaluated
  unsupported right-hand operands, ambiguous scalar truthiness, untracked scalar
  logical operands, non-finite float truthiness, null coalescing, arrays,
  objects,
  references/copy-on-write behavior, exact native error objects,
  linking/execution, or broader native lowering are rejected with a
  specific codegen diagnostic. Native bitwise lowering accepts binary `&`,
  `|`, and `^`, plus unary `~`, only when operands are already lowerable
  integers in the same straight-line subset. Bounded statically known integer
  bitwise and unary bitwise-not results remain tracked for later checked
  integer arithmetic. Single-result statically known integer operands for
  unary `~` fold to the known bitwise-not result without a redundant native
  bitwise-not operation. Double unary bitwise-not `~~$x` over an
  already-lowerable integer operand reuses `$x`, including intentionally
  untracked integer expressions such as overflow-sensitive shift results.
  Identical tracked integer expression operands and
  identical integer literal operands for `&` and `|` reuse the existing value,
  and identical tracked integer expression operands and identical integer
  literal operands for `^` fold to zero. Identical integer operands also fold
  after both operands lower when the value is intentionally untracked, such as
  overflow-sensitive shift results: `$x & $x` and `$x | $x` reuse `$x`, while
  `$x ^ $x` folds to zero. Tracked integer expression operands
  and integer literal operands for `$x & -1` and `-1 & $x`, and for
  `$x | 0`, `0 | $x`, `$x ^ 0`, and `0 ^ $x`, reuse the existing value.
  Tracked integer expression operands and integer literal operands for
  `$x & 0` and `0 & $x` fold to zero. Tracked integer expression operands and
  integer literal operands for `$x | -1` and `-1 | $x` fold to `-1` after both
  operands lower. Single-known integer operands for `$x ^ -1` and `-1 ^ $x`
  fold to the known bitwise-not result. The `& 0`, `& -1`, `| 0`, and `^ 0`
  identity or annihilator forms also fold after both operands lower when the
  other integer operand is intentionally untracked, such as overflow-sensitive
  shift results. Tracked single-result integer expression
  bitwise operations with exactly one tracked expression operand and one
  literal operand for `&`, `|`, and `^` fold to the known integer literal,
  while literal-only integer bitwise operations and tracked-expression plus
  tracked-expression bitwise operations stay emitted. Native shift lowering accepts `<<`
  and `>>` only for already lowerable integer left operands with statically
  known shift counts from 0 through 63; right shifts use arithmetic shift for
  signed integer results. Tracked integer expression operands and integer
  literal operands for `$x << 0` and `$x >> 0` reuse the existing value.
  Those shift-by-zero identities also fold after both operands lower when the
  left integer operand is intentionally untracked, such as an overflow-sensitive
  shift result. Tracked single-result integer expression shifts with static safe
  nonzero counts fold to the known integer literal, while literal-only shifts
  and non-single tracked integer shifts stay emitted.
  Bounded statically known safe shift results remain tracked for later checked
  integer arithmetic; overflow-sensitive left-shift result sets
  remain unknown so later arithmetic rejects them instead of implying PHP
  overflow semantics. Dynamic shift counts, negative or large counts, PHP
  bytewise string bitwise behavior, scalar-to-int coercion for non-integer
  operands, arrays, objects,
  references/copy-on-write behavior, exact native error objects,
  linking/execution, and broader native lowering are rejected with a specific
  codegen diagnostic. Native ternary lowering
  accepts full ternary `condition ? if_true : if_false` only when the condition
  is already a lowerable boolean or native boolean expression and both branch
  values are already lowerable integers, booleans, floats, strings, or both
  branches are `null` in the same straight-line subset, or when the condition
  is a statically known boolean and both branch values are already lowerable
  scalar values, or when the condition and both branches are the same direct
  variable whose current value is already lowerable. Dynamic mixed-type branch values are rejected until native
  tagged values exist. Dynamic non-null ternaries emit LLVM `select` or the
  corresponding C conditional expression, identical static string branches fold
  to that string without a pointer select, identical boolean expression
  branches fold to the reused expression without a redundant boolean select,
  identical tracked integer expression branches and identical integer literal
  branches fold to the reused value without a redundant integer select, and
  identical integer branches also fold after both branches lower when the
  integer value is intentionally untracked, such as an overflow-sensitive shift
  result. Identical tracked float expression branches and identical float literal
  branches fold to the reused value without a redundant float select, and
  identical float branches also fold after both branches lower when the value
  is intentionally untracked, such as a non-finite overflowing float
  multiplication. Identical direct-variable full ternaries such as `$value ?
  $value : $value` reuse the direct variable value without proving truthiness
  when all three operands are the same already-lowerable direct variable,
  including untracked integer, non-finite float-producing, and string pointer
  expressions, boolean expressions, and null values.
  dynamic boolean literal branches fold without a boolean select for
  `$flag ? true : false`, `$flag ? false : true`, `$flag ? true : true`, and
  `$flag ? false : false`, dynamic `null`/`null` ternaries fold to `null`, and
  static boolean ternaries fold to the selected branch value. Dynamic integer,
  finite-float, and boolean ternaries whose possible branch values collapse to
  a single known result fold to that scalar without a redundant select;
  ambiguous same-type ternaries stay emitted. Full ternary conditions with null
  or with single-known integer, finite-float, or known-string truthiness fold to
  the selected already-lowerable branch without lowering the unselected branch;
  null selects the false branch. Dynamic boolean full ternaries still require
  both branches to lower before selection. Ambiguous integer, float, or string
  conditions, untracked string conditions, non-finite float result tracking,
  and non-finite float conditions remain rejected, and dynamic branch skipping
  for unsupported or side-effecting branches remains unsupported. Dynamic integer ternaries and later
  checked integer arithmetic track up to four statically known possible
  values; combinations with more possible results remain unsupported. Native
  short ternary `?:` accepts lowerable boolean conditions in the same
  straight-line subset; dynamic boolean forms require a lowerable boolean
  fallback, static-false forms return any already-lowerable scalar fallback,
  and static-true forms fold to `true` without lowering the fallback.
  Single-known integer conditions also fold through integer truthiness: proven
  nonzero integer conditions reuse the integer result, and proven zero integer
  conditions use the fallback. Single-known finite float conditions fold
  through float truthiness the same way, with proven nonzero finite floats
  reusing the float result and proven zero floats using the fallback. Known
  string conditions fold through PHP string truthiness when all possible
  values have the same truthiness: non-empty strings except `"0"` reuse the
  string result, while `""` and `"0"` use the fallback. Identical direct
  boolean-, integer-, float-, and string-variable short ternaries such as
  `$flag ?: $flag`, `$value ?: $value`, and `$text ?: $text` also reuse
  already-lowerable expressions without proving broader truthiness, including
  boolean expressions, untracked integer expressions, untracked non-finite
  float-producing expressions, and untracked string pointer expressions. Null short ternaries use the fallback for `null ?:
  fallback`, including direct null-variable fallback forms such as
  `$value ?: $value`; broader null truthiness in logical binaries or null coalescing
  remains unsupported. Cases
  that require general PHP truthiness, lazy branch evaluation to skip
  unsupported or side-effecting branches, ambiguous string truthiness,
  non-identical untracked integer, float, or string expressions, non-finite float truthiness, other non-boolean
  truthiness, null coalescing `??`, null-aware variable/array-offset/object lookup, arrays, objects,
  references/copy-on-write behavior, exact native error objects,
  linking/execution, or broader native lowering are rejected with a specific
  codegen diagnostic. Native lowering statically folds direct `gettype`,
  `is_null`, `is_bool`, `is_int`/`is_integer`/`is_long`,
  `is_float`/`is_double`, `is_string`, `is_array`, `is_scalar`, and
  `is_numeric` calls only when their single argument is already in the
  straight-line native scalar/null subset. Native `is_numeric` also folds
  literal and tracked string values only when the current numeric-string
  grammar proves the result statically. Selected-`clang` assembly snapshots
  validate that the deterministic folded LLVM IR for these existing
  `is_numeric`, `is_countable`, `is_iterable`, `is_object`, and
  `get_debug_type` slices is handed to the chosen backend through stdin
  without widening production lowering behavior.
  Direct `is_countable` and `is_iterable` and `is_object` calls fold to
  `false` for already-lowerable
  scalar/null/string operands only, and direct scalar/null/string
  `get_debug_type` calls fold to the current runtime type-name strings.
  Direct `class_exists`, `interface_exists`, `trait_exists`, and
  `enum_exists` calls with already-lowerable string names and optional
  already-lowerable boolean autoload flags fold to `false` in native output
  because native lowering still rejects class/interface/trait/enum
  declarations and has no autoload or native class table.
  Direct `property_exists` and `method_exists` calls with already-lowerable
  string class names and already-lowerable string member names also fold to
  `false` for the same no-native-class-table boundary.
  Direct `is_a` and `is_subclass_of` calls with already-lowerable string
  object/class names, already-lowerable string target class names, and optional
  already-lowerable boolean `allow_string` flags fold to `false` without
  claiming inheritance or native class-table support.
  Direct `is_callable($value)` calls fold in native output when `$value` is an
  already-lowerable string value with a uniform known lookup result in the
  current documented builtin table, or when `$value` is an already-lowerable
  non-string scalar/null value, which folds to `false`. Direct
  `is_callable($value, $syntax_only)` calls also fold when `$value` is an
  already-lowerable string or non-string scalar/null value and `$syntax_only`
  is an already-lowerable boolean: true syntax-only flags return true for
  string values without name lookup, non-string scalar/null values return
  false, and false flags use the same documented builtin lookup as the
  one-argument form.
  Direct `function_exists($name)` calls fold in native output when `$name` is
  an already-lowerable string value with a uniform known answer in the current
  documented builtin table: documented callable builtins, including
  `array_change_key_case`, `array_column`, `array_is_list`,
  `array_count_values`, `array_sum`, `array_product`, `array_reduce`, and
  `array_filter`, fold to `true`, and missing names fold to `false`.
  Direct calls to array builtins such as `array_change_key_case(...)`,
  `array_column(...)`, `array_sum(...)`, `array_product(...)`, and
  callback-driven forms such as `array_reduce(...)` and `array_filter(...)`
  still reject under the native array-lowering boundary. Assembly snapshots
  also validate that the deterministic folded IR for this existing slice reaches the fallback backend
  without widening production lowering behavior.
  Direct `strlen($value)` calls fold in native output when `$value` is an
  already-lowerable known string operand, including tracked string expressions
  whose possible values have one uniform byte length. A selected-`clang`
  assembly snapshot validates that the deterministic folded LLVM IR for this
  existing slice is handed to the chosen backend through stdin without
  widening production lowering behavior.
  Direct `defined($name)` calls fold in native output when `$name` is an
  already-lowerable known string operand whose possible values are supported
  unqualified constant names with a uniform answer against the current exact
  built-in constant table. Exact `CASE_LOWER`, `CASE_UPPER`,
  `ARRAY_FILTER_USE_BOTH`, `ARRAY_FILTER_USE_KEY`, `SORT_REGULAR`,
  `SORT_NUMERIC`, and `SORT_STRING` names fold to true; other supported
  unqualified names fold to false. The Milestone 569 and 573 snapshots cover
  the `SORT_REGULAR` and `SORT_NUMERIC` additions without broadening native
  constant values, runtime-defined constant lookup, dynamic calls, arrays,
  objects, or exact native PHP error behavior. A
  selected-`clang` assembly snapshot validates that the deterministic folded
  LLVM IR for the `SORT_REGULAR`, `SORT_NUMERIC`, and `SORT_STRING` slices is
  handed to the chosen backend through stdin without widening production
  lowering behavior. A broader selected-`clang` snapshot validates the same
  stdin handoff for the current exact `CASE_*`, `ARRAY_FILTER_*`, and
  `SORT_STRING` built-in constant answer table.
  Direct `isset($name)` over direct variables folds from the current
  straight-line static-variable map: missing or statically `null` variables
  fold to false, and statically assigned non-null lowerable values fold to
  true. A selected-`clang` assembly snapshot validates that the deterministic
  folded LLVM IR for this existing slice is handed to the chosen backend
  through stdin without widening production lowering behavior.
  Direct `empty($name)` over direct variables folds from the same map: missing
  variables and statically falsey lowerable scalar/null values (`null`,
  `false`, `0`, `0.0`, `""`, and `"0"`) fold to true, and statically truthy
  lowerable scalar values fold to false. A selected-`clang` assembly snapshot
  validates that the deterministic folded LLVM IR for this existing slice is
  handed to the chosen backend through stdin without widening production
  lowering behavior.
  Array/object operands remain rejected until native array/object lowering
  exists. Dynamic calls, wrong arity, non-string `function_exists` names,
  non-string `strlen` operands and exact string-coercion diagnostics,
  non-bool `is_callable` syntax-only flags, callable-name output parameters,
  array/object/method callables,
  user-defined functions in native output, namespace/import/autoload-aware
  lookup, extension-loaded functions outside the documented builtin table,
  general callable builtin dispatch, runtime call lookup, stack-frame layout,
  arity/type diagnostics, unsupported `defined(...)` names, dynamic
  string-call dispatch, and exact native error objects remain unsupported.
  Native user-function declarations
  and return statements are rejected before function-body lowering with a
  specific codegen diagnostic until generated code has function symbol tables,
  stack-frame layout, default parameter binding, recursion guards,
  return-value flow, and exact native error behavior.
  Native built-in constant values, runtime-defined constants, bare constant
  reads, top-level `const` declarations, `define()`/`constant()`, and
  unsupported `defined(...)` forms are rejected before operand or argument
  lowering with a specific codegen diagnostic until generated code has native
  constant tables, source-order definitions, namespace-aware lookup, and
  exact native error objects.
  Native class declarations, inheritance metadata, object instantiation,
  constructor dispatch, public property reads/writes, instance method calls, and object metadata
  builtins beyond scalar/null/string `is_object`,
  scalar/null/string `get_debug_type`, and direct string-name metadata-exists
  false folding, including string/string `property_exists` and
  `method_exists`, and string/string relationship false folding for `is_a` and
  `is_subclass_of`, are rejected before body, operand, or argument lowering
  with a specific codegen diagnostic until generated code has native object
  layout, handles, visibility, method dispatch, class metadata tables,
  inheritance, autoload interaction, and exact native error objects.
  Native arrays, array literals, array indexing, array assignment, `foreach`
  array iteration, array offset unset, and array builtin function calls are
  rejected before body, operand, argument, or callback lowering with a specific
  codegen diagnostic until generated code has native array storage layout, key
  normalization, copy-on-write containers, references, callback dispatch, and
  exact native error objects.
  Native `if`/`elseif`/`else`, `while`, `for`, `do ... while`, `switch`,
  `break`, and `continue` are rejected before condition, body, case, or
  loop-control lowering with a specific codegen diagnostic until generated
  code has PHP truthiness, branch layout, loop control flow, switch
  fallthrough, references/copy-on-write side-effect behavior, and exact native
  error objects.
  Native compound assignment, null coalescing assignment,
  increment/decrement, assignment expressions, direct variable unset, and
  multiple-operand unset are rejected before operand or mutation-target
  lowering with a specific codegen diagnostic until generated code has
  read-modify-write ordering, null-aware mutation, unset symbol-table effects,
  references/copy-on-write, and exact native error objects.
- Assembly emission: uses LLVM tools when available, with a temporary `cc -S`
  C fallback for the same narrow lowerable subset. CLI coverage for
  `phpc compile --emit-asm` records a normalized success summary for the current
  scalar echo/assignment fixture instead of exact assembly text, because
  emitted assembly varies by platform and backend. A separate CLI snapshot runs
  an unsupported array program with backend tools removed from `PATH`, proving
  array lowering rejects before assembly backend discovery. Another CLI
  snapshot runs a lowerable scalar program with backend tools removed from
  `PATH`, proving the stable missing-backend diagnostic when `clang`, `llc`,
  and `cc` are unavailable. A further CLI snapshot runs a lowerable scalar
  program with a PATH exposing only `cc`, proving the documented `cc -S`
  fallback path with normalized assembly-shape checks. Another snapshot uses a
  deterministic fake `clang` that passes backend discovery and exits nonzero
  after accepting generated LLVM IR, proving the stable selected-backend
  failure diagnostic shape without committing real toolchain stderr. A
  selected-`llc` snapshot hides `clang` and `cc` while exposing only a
  deterministic fake `llc`, proving the documented LLVM backend selection order
  with normalized assembly-shape checks. A selected-`llc` failure snapshot uses
  a deterministic fake `llc` that passes discovery and exits nonzero after
  accepting generated LLVM IR, proving the stable `llc failed to emit
  assembly` diagnostic shape without committing real toolchain stderr. A
  C fallback failure snapshot exposes only a deterministic fake `cc` that
  passes discovery and exits nonzero after accepting generated C fallback
  source, proving the stable `cc failed to emit assembly` diagnostic shape
  without committing real toolchain stderr. A discovery-edge snapshot exposes a
  deterministic fake `clang` whose `--version` probe fails while a fake `llc`
  probe succeeds, proving failed backend discovery probes are treated as
  unavailable and skipped before fallback selection. A discovery-exhaustion
  snapshot exposes fake `clang`, `llc`, and `cc` commands whose `--version`
  probes all fail, proving the same stable missing-backend diagnostic is
  reported when command names exist but no candidate passes discovery. An
  empty-stderr selected-backend snapshot exposes a deterministic fake `clang`
  that passes discovery and exits nonzero without stderr after accepting
  generated LLVM IR, proving the stable `backend exited without stderr`
  diagnostic detail. An empty-stdout selected-backend snapshot exposes a
  deterministic fake `clang` that passes discovery and exits successfully
  without assembly stdout, proving the stable `clang emitted empty assembly
  output` diagnostic instead of accepting an empty assembly artifact. A
  success-with-stderr selected-backend snapshot exposes a deterministic fake
  `clang` that emits assembly stdout, writes stderr diagnostics, and exits
  successfully, proving `phpc` returns the assembly and does not surface
  backend stderr on successful emission. Additional success-with-stderr
  fallback snapshots expose deterministic fake `llc` and `cc` tools, proving
  the same behavior after LLVM backend fallback selection and after the `cc -S`
  C fallback selection. Additional empty-stderr fallback failure snapshots
  expose deterministic fake `llc` and `cc` tools that exit nonzero without
  diagnostics, proving the same stable `backend exited without stderr` detail
  after fallback selection. Additional empty-stdout fallback success snapshots
  expose deterministic fake `llc` and `cc` tools that exit successfully without
  assembly text, proving the same stable empty-output diagnostic after fallback
  selection. Additional whitespace-only fallback success snapshots expose
  deterministic fake `llc` and `cc` tools that exit successfully with only
  whitespace assembly stdout, proving the same stable
  whitespace-only-output diagnostic after fallback selection. A selected
  backend whitespace-only success snapshot exposes deterministic fake `clang`
  with the same whitespace-only stdout behavior, proving that diagnostic before
  fallback selection too. A selected backend whitespace-with-stderr success
  snapshot exposes deterministic fake `clang` that exits successfully with
  whitespace-only stdout and stderr diagnostics, proving stdout validation
  wins and successful-backend stderr is not surfaced on invalid successful
  output. A selected backend whitespace-with-stderr precedence snapshot exposes
  the same invalid successful `clang` output while `llc` and `cc` are also
  available, proving fallback recovery is not attempted after invalid selected
  backend output. A selected backend empty-stdout-with-stderr precedence
  snapshot exposes invalid successful `clang` output with no assembly stdout
  and stderr diagnostics while `llc` and `cc` are also available, proving
  stdout validation wins and fallback recovery is still not attempted. An
  `llc` whitespace-with-stderr precedence snapshot exposes
  invalid successful `llc` output while the `cc -S` fallback is also
  available and `clang` is unavailable, proving fallback recovery is not
  attempted after invalid selected `llc` output. An `llc` empty-stdout
  precedence snapshot exposes the same no-recovery boundary when selected
  `llc` exits successfully without assembly stdout while `cc` is available.
  An `llc` empty-stdout-with-stderr precedence snapshot covers the same
  boundary when selected `llc` writes stderr diagnostics but emits no assembly
  stdout while `cc` is available. Additional whitespace-with-stderr fallback snapshots expose
  deterministic fake `llc` and `cc` tools with the same invalid
  successful-output behavior, proving stdout validation wins and successful
  backend stderr is not surfaced after fallback selection too.
  Selected-backend stdin handoff for representative generated LLVM IR markers
  is covered with a deterministic fake `clang`, fallback stdin handoff for
  representative generated LLVM IR and generated C markers is covered with
  deterministic fake `llc` and `cc` tools, and selected/fallback backend
  argument vectors are covered with deterministic fake `clang`, `llc`, and
  `cc` tools. Backend discovery probe argument vectors are covered with
  deterministic fake `clang`, `llc`, and `cc` tools that require an exact
  single-argument `--version` probe before selected or fallback assembly
  emission proceeds. Successful discovery probes that write stdout and stderr
  diagnostics are covered with deterministic fake `clang`, `llc`, and `cc`
  tools, proving probe output is ignored when selected or fallback assembly
  emission later succeeds. Failed discovery probes that write stdout and
  stderr diagnostics are also covered with deterministic fake `clang`, `llc`,
  and `cc` tools, proving failed-probe output is ignored before fallback
  selection and before the stable missing-backend diagnostic when every
  candidate probe fails. Discovery probe start-failure snapshots use
  deterministic fake `clang`, `llc`, and `cc` command names that exist on
  `PATH` but cannot be started for `--version`, proving probe start failures
  are treated as unavailable before fallback selection and before the stable
  missing-backend diagnostic when every candidate probe cannot start.
  Discovery probe permission-denied snapshots use deterministic fake `clang`,
  `llc`, and `cc` command names that exist on `PATH` but are not executable
  for `--version`, proving permission-denied probe starts are treated as
  unavailable before fallback selection and before the stable missing-backend
  diagnostic when every candidate probe is non-executable. A
  selected-backend start-failure snapshot uses a
  deterministic fake `clang` that passes discovery and then rewrites itself to
  use a missing interpreter before actual assembly emission, proving the
  stable `failed to start clang for assembly emission` diagnostic for that
  race-like command-start boundary. A selected-backend permission-denied
  emission snapshot uses a deterministic fake `clang` that passes discovery
  and then removes its own execute permission before actual assembly emission,
  proving the same stable selected-backend start diagnostic for
  permission-denied starts after discovery. Fallback start-failure snapshots use
  deterministic fake `llc` and `cc` tools with the same behavior, proving the
  stable `failed to start llc for assembly emission` and `failed to start cc
  for assembly emission` diagnostics after fallback selection. Fallback
  permission-denied emission snapshots use deterministic fake `llc` and `cc`
  tools that pass discovery and then remove their own execute permission
  before actual assembly emission, proving the same stable fallback backend
  start diagnostics for permission-denied starts after discovery and proving a
  selected `llc` permission-denied start is reported without falling through
  to the `cc -S` C fallback. A mixed scalar output snapshot uses a lowerable
  fixture with both `echo` and `print`, plus a deterministic fake `clang`, to
  prove the current static scalar `printf` assembly path accepts mixed output
  statements without claiming runtime-backed output conversion. A matching
  C fallback mixed-output snapshot hides LLVM assembly tools and uses a
  deterministic fake `cc` that validates generated C fallback source markers
  for the same static scalar `echo`/`print` boundary. A
  backend-precedence snapshot exposes deterministic fake `clang`, `llc`, and
  `cc` commands together and proves successful `clang` emission is selected
  before fallback tools when all candidates are available. A
  fallback-precedence snapshot hides `clang` while exposing deterministic fake
  `llc` and `cc` commands together, proving successful `llc` emission is
  selected before the `cc -S` C fallback when both fallback candidates are
  available. A selected-backend failure-precedence snapshot exposes
  deterministic fake `clang`, `llc`, and `cc` commands together, makes selected
  `clang` fail emission, and proves the selected-backend failure is reported
  without silently falling through to fallback tools. A fallback
  failure-precedence snapshot hides `clang` while exposing deterministic fake
  `llc` and `cc` commands together, makes selected `llc` fail emission, and
  proves the `llc` failure is reported without silently falling through to the
  `cc -S` C fallback. An empty-stderr fallback failure-precedence snapshot
  covers the same `clang`-unavailable boundary when selected `llc` exits
  nonzero without diagnostics, proving the stable empty-stderr `llc`
  diagnostic is reported without `cc -S` fallback recovery. An empty-stderr
  selected-backend failure-precedence snapshot exposes deterministic fake
  `clang`, `llc`, and `cc` commands together, makes selected `clang` exit
  nonzero without diagnostics, and proves the stable empty-stderr `clang`
  diagnostic is reported without falling through to fallback tools. A
  selected-backend start-failure-precedence snapshot exposes deterministic fake
  `clang`, `llc`, and `cc` commands together, makes selected `clang` pass
  discovery and then fail to start for assembly emission, and proves the
  stable selected-backend start diagnostic is reported without falling through
  to fallback tools. A selected-backend empty-stdout-with-stderr precedence
  snapshot exposes deterministic fake `clang`, `llc`, and `cc` commands
  together, makes selected `clang` exit successfully with no assembly stdout
  and stderr diagnostics, and proves the stable empty-output diagnostic is
  reported without falling through to fallback tools or surfacing
  successful-backend stderr. A fallback start-failure-precedence snapshot hides
  `clang` while exposing deterministic fake `llc` and `cc` commands together,
  makes selected `llc` pass discovery and then fail to start for assembly
  emission, and proves the stable `llc` start diagnostic is reported without
  falling through to the `cc -S` C fallback. Bundled toolchains, assembly linking/execution, full
  backend-specific IR/C validation for every backend and every lowered
  construct, full backend-specific command-line compatibility,
  backend-specific discovery semantics for every tool, backend-specific failed
  probe output/start-failure/permission-denied semantics, broader backend race-condition recovery beyond
  command-start diagnostics, backend-specific stdout/stderr guarantees,
  backend-specific assembly text, PHP zvals, native symbol-table storage,
  references/copy-on-write, exact native error objects, and broader native
  lowering remain unsupported.
- Function calls: user-defined positional calls are supported in `phpc run`,
  including optional trailing commas in argument lists.
  Dynamic function calls are supported only when the callee expression evaluates
  to a string that case-insensitively resolves to a user-defined function or to
  one of the documented callable builtins: `strlen`, `count`,
  `array_key_exists`, `array_key_first`, `array_key_last`, `array_is_list`,
  `array_values`, `array_keys`, `array_reverse`, `array_slice`, `array_chunk`,
  `array_pad`, `array_merge`, `array_replace`, `array_combine`, `define`,
  `constant`, `defined`,
  `array_intersect_key`, `array_diff_key`, `array_diff`, `array_intersect`,
  `array_unique`, `array_flip`, `array_fill_keys`, `array_count_values`,
  `array_sum`, `array_product`, `array_reduce`, `array_filter`, `array_map`,
  `in_array`, `array_search`, `gettype`, `is_null`, `is_bool`, `is_int`,
  `is_integer`, `is_long`, `is_float`, `is_double`, `is_string`, `is_array`,
  `is_scalar`, `is_numeric`, `is_countable`, `is_iterable`, `is_callable`,
  `function_exists`, `get_class`,
  `is_object`, `get_debug_type`,
  `class_exists`, `interface_exists`, `trait_exists`, `enum_exists`,
  `property_exists`, `method_exists`, `get_class_methods`, `get_class_vars`,
  `get_object_vars`, `get_mangled_object_vars`,
  `is_a`, `is_subclass_of`, `get_parent_class`, `get_declared_classes`,
  `get_declared_interfaces`, `get_declared_traits`, `get_called_class`,
  `spl_object_id`, `spl_object_hash`, `var_dump`, or `print_r`.
  The `define`, `constant`, and `defined` names resolve through the documented
  runtime constant path. Unresolved names fail with a stable undefined-function
  runtime error, and non-string callees fail with a stable unsupported-call
  runtime error. Required parameters, optional trailing commas after the final
  real parameter, and trailing default parameter values are supported.
  Defaults may use the current constant-expression subset: `null`, booleans, integers,
  floats, strings, short and long arrays with supported keys, unary
  expressions, binary expressions over those values, and bare references to
  unqualified constants that are defined in the current runtime constant table
  before the omitted argument is bound. The exact uppercase built-in
  `ARRAY_FILTER_USE_KEY` and `ARRAY_FILTER_USE_BOTH` constants are also
  accepted in default expressions. String-valued dynamic calls accept the same
  optional trailing comma syntax after the final positional argument. Omitted
  arguments bind to their defaults;
  missing constant references fail with a stable undefined-constant runtime
  diagnostic; calls outside the supported required-to-total arity range
  fail with a stable arity diagnostic. Each user-function call gets a fresh
  local scope. Parameters and local assignments shadow global variables without
  mutating them, and functions do not import top-level variables implicitly.
  `global` declarations parse but fail with a stable runtime error because
  global scope imports are not implemented. Recursive user-function calls are
  supported until the fixed 128-frame user-function call-depth guard is reached.
  That guard is a project-specific runtime diagnostic, not PHP's native stack or
  memory exhaustion behavior; it is not configurable and does not produce stack
  traces. Forward constant references at omitted-argument binding time,
  namespace-aware constants, class constants, dynamic defaults,
  references/copy-on-write behavior, and native lowering for defaults are not
  implemented. Native lowering for user-function declarations and returns is
  explicitly rejected until function symbol tables, stack-frame layout, default
  parameter binding, recursion guards, return-value flow, and exact native
  error behavior exist. Non-constant defaults such as variables, calls,
  dynamic calls, and indexed reads are rejected by the parser. Required
  parameters after default parameters are also rejected instead of modeling
  PHP's deprecation and implicit-required behavior. Empty parameter slots such
  as `function f(,)` remain rejected. Variadic parameters and argument unpacking,
  reference parameters/returns, reference expressions, anonymous functions,
  arrow functions, named arguments, first-class callable syntax such as
  `strlen(...)` and `$callback(...)`, empty call arguments, and
  `declare(strict_types=1)` are rejected with stable parse diagnostics.
  Parameter type declarations and return type
  declarations also fail with stable parse diagnostics before any type
  enforcement can run. Static local variable declarations inside functions
  also fail with a stable parse diagnostic before function-local static storage
  exists. The `__LINE__` magic constant evaluates to the source line of the
  expression token in ordinary expressions, default parameter values, and
  top-level `const` declarations. The `__FILE__` magic constant evaluates to
  the current `phpc run` input path string when one is available, including
  ordinary expressions, default parameter values, and top-level `const`
  declarations; path-less library execution currently evaluates it as an empty
  string. The `__DIR__` magic constant evaluates to the current `phpc run`
  input path's parent directory, uses `.` when that path has no parent
  directory, and evaluates to an empty string for path-less library execution.
  The `__FUNCTION__` magic constant evaluates to the current user-function
  name in ordinary expressions and default parameter values, and to an empty
  string outside a function. `__METHOD__` fails with a stable parse diagnostic
  tied to the current missing method-dispatch boundary. `__CLASS__` fails with
  a stable parse diagnostic tied to the current missing class-context tracking
  boundary. `__TRAIT__` fails with a stable parse diagnostic tied to the
  current missing trait declaration/use and trait-context tracking boundary.
  `__NAMESPACE__` fails with a stable parse diagnostic tied to the current
  missing namespace-aware name-resolution boundary. Nullable, union, and intersection
  types, `mixed`, `void`/`never`, class/interface type names, coercive versus
  strict typing, variance, static local initialization expressions,
  per-function persistence, recursion/reentrancy behavior, canonical absolute
  `__FILE__`/`__DIR__` paths matching PHP exactly, eval/include source mapping,
  method/class magic constant context, namespace and trait magic constants,
  closure function-name context, magic constant native lowering, array callables, object/method
  callables, first-class callable
  syntax, `call_user_func`, namespace-qualified callable resolution, autoload
  interaction, and native lowering for type declarations are unsupported.
- Builtins: `strlen`, `isset`, `empty`, `count`, `define`, `constant`,
  `defined`, `array_key_exists`, `array_key_first`, `array_key_last`,
  `array_is_list`, `array_values`, `array_keys`, `array_reverse`,
  `array_slice`, `array_chunk`, `array_pad`, `array_merge`, `array_replace`,
  `array_combine`, `array_intersect_key`, `array_diff_key`, `array_diff`,
  `array_intersect`, `array_unique`, `array_flip`, `array_fill_keys`,
  `array_count_values`, `array_sum`, `array_product`, `array_reduce`,
  `array_filter`, `array_map`, `in_array`, `array_search`, `gettype`,
  `is_null`, `is_bool`, `is_int`, `is_integer`, `is_long`, `is_float`,
  `is_double`, `is_string`, `is_array`, `is_scalar`, `is_numeric`,
  `is_countable`, `is_iterable`, `is_callable`, `function_exists`,
  `get_class`, `is_object`, `get_debug_type`,
  `class_exists`, `interface_exists`,
  `trait_exists`, `enum_exists`, `property_exists`, `method_exists`,
  `get_class_methods`, `is_a`, `is_subclass_of`, `get_class_vars`,
  `get_object_vars`, `get_mangled_object_vars`, `get_parent_class`,
  `get_declared_classes`, `get_declared_interfaces`, `get_declared_traits`,
  `spl_object_id`, `spl_object_hash`, `var_dump`, and `print_r`
  cover the documented scalar/array/object subset. `get_called_class` is
  recognized only as the explicit unsupported method/static class context
  boundary described below. `spl_object_id` returns the current object's stable
  process-local handle id for object inputs. `spl_object_hash` returns a stable
  32-character current-subset hash derived from that handle id; exact system PHP
  hash formatting and handle reuse after destruction are not claimed.
  `gettype($value)` returns PHP legacy type names for the current boxed value
  model, and `is_null`, `is_bool`, `is_int`/`is_integer`/`is_long`,
  `is_float`/`is_double`, `is_string`, `is_array`, and `is_scalar` report the
  current value category without coercion. `is_numeric` returns true for
  integers, floats, and well-formed numeric strings using the same current
  numeric-string subset as scalar arithmetic. `is_countable` and `is_iterable`
  return true for arrays and false for the current scalar/null/object values.
  `is_callable($value)` supports the current string function-name subset: it
  returns true for names that resolve to current user functions or documented
  callable builtins, and false for missing names or non-string values.
  `is_callable($value, $syntax_only)` accepts boolean syntax-only flags; for
  string values, `true` reports callable syntax without resolving the name,
  while `false` uses the current function lookup path. Syntax-only array
  callable checks accept only the current two-element `[class-or-object,
  method]` shape with integer keys `0` and `1`, where the first value is a
  string class name or current object and the second value is a string method
  name; this shape check does not resolve classes or methods. Normal array
  callable resolution checks the same two-element shape against current
  declared method metadata: object receivers are true for public declared
  methods, and class-string receivers are true for public static declared
  methods. Scalar non-string values return false. Native lowering folds only direct calls whose value
  argument is an already-lowerable string or non-string scalar/null value and
  whose optional syntax-only flag is an already-lowerable boolean; true
  syntax-only flags return true for string values, non-string scalar/null
  values return false, while false or omitted flags use the documented native
  builtin lookup table for strings. Additional callable forms, the
  callable-name output parameter,
  environment-specific legacy aliases such as `is_real`,
  extension/resource-aware type checks, `Countable`
  object/interface semantics, and `Traversable`/generator object semantics are
  not implemented.
  `function_exists($name)` checks string names against the current runtime
  function table, including current user functions and documented callable
  builtins. Native lowering folds only direct calls whose name argument is an
  already-lowerable string with a uniform known result in the documented
  builtin table; native user-defined function tables, dynamic calls,
  namespace/autoload-aware lookup, extension-loaded functions beyond documented
  builtins, non-string name coercion, and exact native
  `TypeError`/deprecation behavior are not implemented.
  `get_class($object)` returns the declared class name for current minimal
  object values and rejects non-object arguments. `is_object($value)` returns
  true only for current minimal object values and false for scalars and arrays.
  `get_debug_type($value)` returns current scalar/array type names and the
  declared class name for current minimal object values. `class_exists($name)`
  and `class_exists($name, $autoload)` accept string class names, return whether
  the current parsed program declared that class, and accept current bool-like
  scalar autoload flags without triggering autoloading. `null`, arrays,
  objects, references, and exact PHP deprecation/`TypeError` behavior remain
  unsupported for that flag.
  `interface_exists($name)` and `interface_exists($name, $autoload)` accept
  string interface names and return false for all supported calls because
  interface metadata is not represented yet; the autoload flag accepts current
  bool-like scalar values and does not trigger autoloading.
  `trait_exists($name)` and `trait_exists($name, $autoload)` accept string
  trait names and return false for all supported calls because trait metadata
  is not represented yet; the autoload flag accepts current bool-like scalar
  values and does not trigger autoloading.
  `enum_exists($name)` and `enum_exists($name, $autoload)` accept string enum
  names and return false for all supported calls because enum metadata is not
  represented yet; the autoload flag accepts current bool-like scalar values
  and does not trigger autoloading.
  `property_exists($object_or_class, $property)` checks declared and inherited
  property metadata for current object values or string class names with
  case-sensitive property names. `method_exists($object_or_class, $method)` checks declared and inherited
  method metadata for current object values or string class names with
  case-insensitive method names. `get_class_methods($object_or_class)` returns
  a zero-indexed array of public declared method names for current object
  values or declared string class names. `get_class_vars($class_name)` returns
  public declared and inherited property names with `null` values for declared
  string class names. `get_object_vars($object)` returns public exact and
  inherited instance property names with their current values for current
  object values.
  `get_mangled_object_vars($object)` returns public, protected, and private
  instance slots with PHP-style mangled keys for current object values.
  `empty($object->name)`
  checks falsey public slots and treats missing properties, undefined target
  variables, and non-object target variables as empty in the current
  direct-object-variable subset.
  `is_a($object_or_class, $class_name[, $allow_string])` checks exact class
  identity and single-parent ancestor relationships over current object
  values, and over string class names only when `allow_string` is true.
  `is_subclass_of($object_or_class, $class_name[, $allow_string])` validates
  current object/string relationship-check arguments and walks the current
  single-parent metadata chain.
  `get_parent_class($object_or_class)` accepts current object values or
  declared string class names and returns the immediate parent class name when
  one is recorded, otherwise false.
  `get_declared_classes()` returns a zero-indexed array containing only the
  current parsed program's declared class names in declaration order.
  `get_declared_interfaces()` returns an empty zero-indexed array because
  interface declarations and internal interface metadata are not represented.
  `get_declared_traits()` returns an empty zero-indexed array because trait
  declarations and internal trait metadata are not represented.
  `get_called_class()` is recognized as a zero-argument callable boundary, but
  direct and string-valued dynamic calls fail with a stable unsupported-call
  diagnostic until method/static class context exists.
  `spl_object_id($object)` accepts current object values and returns the
  process-local object handle id; non-object arguments fail with a stable
  type-boundary diagnostic.
  `spl_object_hash($object)` accepts current object values and returns a stable
  current-subset handle hash; non-object arguments fail with a stable
  type-boundary diagnostic.
  `print_r` can also render the current minimal object values. `strlen` remains
  scalar-only and rejects arrays and objects. `count` accepts arrays only.
  `array_key_exists($key, $array)` accepts integer
  and string keys over the current ordered array value model, plus `null`
  keys as the empty-string key, boolean keys as integer `0`/`1`, and integral
  finite float keys as integers. It returns
  true for existing keys even when the stored value is `null`, returns false
  for missing keys, rejects non-array second arguments, and rejects unsupported
  key values such as lossy or non-finite floats, arrays, objects, and future
  resources instead of applying PHP's full key coercions and
  warning/deprecation behavior.
  `array_key_first($array)` and
  `array_key_last($array)` accept arrays only, return the first or last
  inserted integer or string key as an `int` or `string`, return `null` for
  empty arrays, and are also available through string-valued dynamic function
  calls. `array_is_list($array)` accepts arrays only, returns true for empty
  arrays and entries whose keys are exactly ordered integer keys `0..n-1`, and
  returns false for gaps, negative keys, string keys, and out-of-order integer
  keys. Numeric string keys that normalize to integer keys use the current
  array-key normalization before the list check. It is also available through
  string-valued dynamic function calls. Exact native `TypeError` objects,
  references, copy-on-write containers, and native lowering are not
  implemented. `array_values($array)` accepts arrays
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
  `array_slice($array, $offset)` accepts arrays and integer offsets, returns
  entries from that insertion-order offset to the end, supports negative
  offsets counted back from the end, reindexes integer-keyed entries from zero,
  preserves string keys, and is available through string-valued dynamic
  function calls. `array_slice($array, $offset, $length)` also accepts integer
  lengths, with positive lengths limiting the number of returned entries, zero
  returning an empty array, and negative lengths excluding entries from the end
  of the input array. `array_slice($array, $offset, null)` treats the null
  length as a to-end slice. `array_slice($array, $offset, $length, true)` and
  `array_slice($array, $offset, null, true)` preserve integer and string keys,
  while boolean `false` uses the default integer-key reindexing path. Non-bool
  preserve-key coercion, non-int offset coercion, non-int/non-null length
  coercion, references, copy-on-write containers, object handle identity
  preservation, resource values, exact native `TypeError` objects, and native
  lowering are not implemented.
  `array_chunk($array, $length)` accepts arrays and positive integer lengths,
  splits entries in insertion order, reindexes each inner chunk from integer
  key zero regardless of original integer or string keys, returns an empty
  array for empty input arrays, and is available through string-valued dynamic
  function calls. `array_chunk($array, $length, true)` preserves original
  integer and string keys inside each chunk, and boolean `false` uses the
  default chunk-key reindexing path. Non-bool preserve-key coercion, non-int
  length coercion, non-positive length native `ValueError` objects,
  reference/copy-on-write behavior, object handle identity preservation,
  resource values, exact native `TypeError` objects, and native lowering are
  not implemented.
  `array_pad($array, $length, $value)` accepts arrays and integer lengths. When
  `abs($length)` is not larger than the input size it returns a cloned array
  with the original key shape and append index. Positive lengths right-pad and
  negative lengths left-pad to the requested size, preserving string keys while
  reindexing integer-keyed input entries from zero when padding is needed.
  Padding values are cloned into each new slot. Requests that would insert more
  than 1,048,576 padding entries fail with a stable project diagnostic instead
  of allocating unbounded memory. Non-int length coercion, exact native
  `ValueError`/`TypeError` objects, references, copy-on-write behavior, object
  handle identity preservation, resource values, and native lowering are not
  implemented. `array_pad` is also available through string-valued dynamic
  function calls.
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
  `array_replace($array, ...$replacements)` accepts one or more arrays, clones
  the first array, and inserts replacement entries by normalized integer or
  string key from each replacement array left to right. Existing keys are
  overwritten in place without moving their slots, new replacement keys are
  appended in replacement insertion order, integer keys are preserved rather
  than reindexed, and later append behavior follows the highest non-negative
  integer key seen in the result. It is also available through string-valued
  dynamic function calls. Non-array operands and length mismatches fail with
  stable diagnostics. References, copy-on-write containers, object handle
  identity preservation for object values, resource values, exact native
  `TypeError`/`ValueError` objects, and native lowering are not implemented.
  `array_combine($keys, $values)` accepts two array operands with equal entry
  counts, reads both arrays in insertion order, converts integer and string
  values from the first array into result keys using the current key
  normalization rules, maps null and false key values to the empty string key,
  maps true key values through the string `"1"` key normalization path,
  converts integral finite float key values into integer result keys, and
  stores cloned values from the second array. Duplicate result keys are
  overwritten by later pairs without moving the first result-key position.
  Empty key/value arrays return an empty array. Non-array operands, length
  mismatches, and unsupported key values fail with stable project diagnostics.
  Lossy finite floats, non-finite floats, array, object, future resource, and
  reference key-value coercions, exact native `ValueError`/`TypeError`
  objects, references, copy-on-write containers, object handle identity
  preservation for object values, resource values, and native lowering are not
  implemented.
  `array_combine` is also available through string-valued dynamic function
  calls.
  `array_intersect_key($array, ...$arrays)` accepts two or more array operands,
  checks integer/string keys using the current normalized array-key model, and
  returns a new ordered array containing entries from the first array whose keys
  exist in every subsequent array. The first array's key shape, values, and
  insertion order are preserved, and the source arrays are not mutated.
  Non-array operands, including variadic operands, fail with stable project
  diagnostics naming the offending positional argument. References,
  copy-on-write containers, object handle identity preservation for object
  values, resource values, exact native `TypeError` objects, and native lowering
  are not implemented. `array_intersect_key` is also available through
  string-valued dynamic function calls.
  `array_diff_key($array, ...$arrays)` accepts two or more array operands,
  checks integer/string keys using the current normalized array-key model, and
  returns a new ordered array containing entries from the first array whose
  keys do not exist in any subsequent array. The first array's key shape,
  values, and insertion order are preserved, and the source arrays are not
  mutated. Non-array operands, including variadic operands, fail with stable
  project diagnostics naming the offending positional argument. References,
  copy-on-write containers, object handle identity preservation for object
  values, resource values, exact native `TypeError` objects, and native
  lowering are not implemented. `array_diff_key` is also available through
  string-valued dynamic function calls.
  `array_diff($array, ...$arrays)` accepts two or more array operands, compares
  current scalar values by their PHP string forms, and returns a new ordered
  array containing entries from the first array whose scalar comparison value
  is absent from every subsequent array. The first array's key shape, values,
  insertion order, and append-index behavior are preserved, and the source
  arrays are not mutated. Non-array operands, including variadic operands, and
  non-scalar values such as arrays or objects fail with stable project
  diagnostics.
  References, copy-on-write containers, object/resource values, exact native
  `TypeError` objects, PHP warning-and-string-conversion behavior for
  non-scalar values, and native lowering are not implemented. `array_diff` is
  also available through string-valued dynamic function calls.
  `array_intersect($array, ...$arrays)` accepts two or more array operands,
  compares current scalar values by their PHP string forms, and returns a new
  ordered array containing entries from the first array whose scalar comparison
  value is present in every subsequent array. The first array's key shape,
  values, insertion order, and append-index behavior are preserved, and the
  source arrays are not mutated. Non-array operands, including variadic
  operands, fail with stable project diagnostics naming the offending
  positional argument. Non-scalar values such as arrays or objects fail with
  stable project diagnostics. References, copy-on-write containers,
  object/resource values, exact native `TypeError` objects, PHP
  warning-and-string-conversion behavior for non-scalar values, and native
  lowering are not implemented. `array_intersect` is also available through
  string-valued dynamic function calls.
  `array_unique($array)` accepts one array operand,
  `array_unique($array, SORT_STRING)` accepts the same array operand with the
  current exact uppercase built-in `SORT_STRING` constant or integer value
  `2`, `array_unique($array, SORT_REGULAR)` accepts the current exact
  uppercase built-in `SORT_REGULAR` constant or integer value `0`, and
  `array_unique($array, SORT_NUMERIC)` accepts the current exact uppercase
  built-in `SORT_NUMERIC` constant or integer value `1`. The default and
  `SORT_STRING` forms compare current scalar values by their PHP string forms;
  the `SORT_REGULAR` form compares current scalar values with the
  interpreter's current loose scalar equality rules; and the `SORT_NUMERIC`
  form compares values after the same current scalar numeric coercion used by
  `array_sum` and `array_product`. All supported forms return a new ordered
  array containing the first entry for each distinct comparison value. Kept
  entries preserve their original integer/string keys and insertion order,
  dropped duplicate entries do not affect later append behavior, and the
  source array is not mutated. Non-array operands, non-scalar/non-numeric
  values such as arrays, objects, or non-numeric strings in numeric mode, and
  sort flags outside the supported set fail with stable project diagnostics.
  References, copy-on-write containers, object/resource values, exact native
  `TypeError` objects, PHP warning-and-string-conversion behavior for arrays
  and objects, sort modes other than `SORT_REGULAR`/`SORT_NUMERIC`/
  `SORT_STRING`, exact native array/object `SORT_REGULAR` comparisons, PHP
  warning recovery for non-numeric values in numeric mode, and native lowering
  are not implemented. `array_unique` is also available through string-valued
  dynamic function calls.
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
  `array_change_key_case($array)` and
  `array_change_key_case($array, CASE_LOWER)` return a new ordered array with
  ASCII string keys lowercased and integer keys preserved.
  `array_change_key_case($array, CASE_UPPER)` and
  `array_change_key_case($array, $case)` with any nonzero integer case flag
  uppercase ASCII string keys.
  Duplicate converted keys are overwritten by later source entries without
  moving the first converted-key position, the source array is not mutated, and
  the builtin is available through string-valued dynamic function calls.
  Case flags must be integers in the current subset; integer `0` lowercases
  and any nonzero integer uppercases. Non-int case values still fail with a
  stable project diagnostic. Unicode/locale-aware casing, scalar flag
  coercions, references/copy-on-write, exact native warning/`TypeError`
  behavior, and native lowering are not implemented.
  `array_column($rows, $column_key)` accepts an array first argument and an
  int, string, or null column key. Array rows use the current int/string key
  normalization rules, public object rows use exact public property names for
  string column keys, missing columns are skipped, null values are preserved,
  scalar rows are skipped, and extracted values are reindexed from integer key
  zero. A null column key returns each row value reindexed in insertion order.
  `array_column($rows, $column_key, $index_key)` accepts an int, string, or
  null index key and uses null, boolean, integer, string, or integral finite
  float row values as result keys. Missing index fields append using the
  current array append cursor, duplicate result keys overwrite the previous
  value without moving that key's insertion position, and null index keys keep
  the reindexed behavior. The builtin is also available through string-valued
  dynamic function calls.
  Non-array first arguments, column or index keys other than int/string/null,
  lossy or non-finite float index values, array/object/resource index values,
  magic `__get`, `ArrayAccess`, exact visibility-context behavior for
  non-public properties,
  references/copy-on-write, exact native `TypeError`/warning behavior,
  resource values, and native lowering are not implemented.
  `array_fill_keys($keys, $value)` accepts arrays only for the first argument,
  maps null and false key values to the empty string key, maps true key values
  through the string `"1"` key normalization path, uses integer and integral
  finite float key values directly as integer result keys, normalizes string
  key values through the current PHP-style decimal string key rules, and
  stores the supplied value in every result slot using the current cloned
  `Value` model. Duplicate result keys are overwritten by later key entries
  without moving the first result-key position. Lossy finite floats,
  non-finite floats, arrays, objects, and future resources fail with a stable
  project diagnostic instead of PHP's warning-and-skip behavior. References,
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
  `array_sum($array)` accepts arrays only, treats `null` and `false` as zero,
  `true` as one, integers and floats as themselves, and well-formed numeric
  strings through the current numeric-string parser. Pure integer inputs return
  an integer result unless checked integer addition overflows, at which point
  the result is promoted to float; any float-valued input or float numeric
  string also produces a float result. Empty arrays return integer zero.
  Non-array operands, non-numeric strings, arrays, objects, and future
  resources inside the input fail with stable project diagnostics instead of
  PHP's warning/recovery behavior. References, copy-on-write containers, exact
  native `TypeError` objects, object/resource value recovery, PHP warning
  recovery, and native lowering are not implemented. `array_sum` is also
  available through string-valued dynamic function calls.
  `array_product($array)` accepts arrays only, treats `null` and `false` as
  zero, `true` as one, integers and floats as themselves, and well-formed
  numeric strings through the current numeric-string parser. Pure integer
  inputs return an integer result unless checked integer multiplication
  overflows, at which point the result is promoted to float; any float-valued
  input or float numeric string also produces a float result. Empty arrays
  return integer one. Non-array operands, non-numeric strings, arrays, objects,
  and future resources inside the input fail with stable project diagnostics
  instead of PHP's warning/recovery behavior. References, copy-on-write
  containers, exact native `TypeError` objects, object/resource value recovery,
  PHP warning recovery, and native lowering are not implemented.
  `array_product` is also available through string-valued dynamic function
  calls.
  `array_reduce($array, $callback)` and `array_reduce($array, $callback,
  $initial)` accept arrays only and callback expressions that evaluate to
  string function names resolving to current user functions or callable
  builtins. They invoke the callback with `($carry, $value)` for each source
  value in insertion order, return the final callback result, start with a
  `null` accumulator when no initial value is supplied, return `null` for empty
  arrays without an initial value, and return the supplied initial value for
  empty arrays when present. `array_reduce` is available when called through a
  string-valued dynamic function name. Non-array operands, non-string callback
  values, and unresolved callback names fail with stable diagnostics.
  Array/object callables, closures, first-class callables, method calls,
  references, copy-on-write containers, exact native `TypeError` objects,
  object handle identity preservation, resource values, and native lowering
  are not implemented.
  `array_filter($array)` without a callback, `array_filter($array, null)`,
  and `array_filter($array, null, $mode)` with integer mode flags `0`, `1`,
  or `2`, finite integral float mode flags, integral numeric string mode flags
  that trim and parse to `0`, `1`, or `2`, or boolean mode flags accept arrays
  only, remove `null`, `false`, zero
  integers and floats, empty strings, string `"0"`, and empty arrays using the current
  `Value::is_truthy` rules, preserve the original integer/string keys and
  insertion order of kept entries, and are available through string-valued
  dynamic function calls.
  `array_filter($array, $callback)` accepts callback expressions that evaluate
  to string function names resolving to current user functions or callable
  builtins, invokes the callback with the value only, keeps entries whose
  callback result is truthy, preserves original keys and insertion order,
  accepts explicit integer mode flag `0`, finite integral float mode flag
  `0.0`, integral numeric string mode flag `"0"`, and boolean mode flag
  `false` for the same value-only callback path, and is available when
  `array_filter` itself is called through a string-valued dynamic function
  name. `array_filter($array, $callback, 2)` plus finite integral float and
  integral numeric string modes that parse to `2` invoke that same
  string-valued callback subset with each entry's current integer or string
  key as the only argument and preserve original keys for entries whose
  callback result is truthy. `array_filter($array, $callback, 1)`,
  `array_filter($array, $callback, true)`, and finite integral float or
  integral numeric string modes that parse to `1` invoke the same string-valued
  callback subset with the value and then the current integer or string key as
  arguments, preserving original keys for entries whose callback result is
  truthy. Non-string
  non-null callback values
  fail with a stable diagnostic, and unresolved callback names fail with the
  current undefined-function diagnostic. Exact uppercase
  `ARRAY_FILTER_USE_KEY` and `ARRAY_FILTER_USE_BOTH` constants may be used as
  the mode argument and evaluate to the same current integer mode values as PHP.
  `constant("ARRAY_FILTER_USE_KEY")` and
  `constant("ARRAY_FILTER_USE_BOTH")` resolve to the same integer values and
  may also be used as mode expressions. `define($name, $value)` accepts
  unqualified string names matching the current identifier subset and stores
  `null`, booleans, integers, floats, strings, and arrays containing only
  supported constant values. `constant($name)` and bare reads of
  runtime-defined unqualified constants return a cloned value from the
  runtime-defined table or from the exact built-in `ARRAY_FILTER_*` and
  `SORT_*` slice.
  `defined($name)` returns true for supported unqualified names present in that
  current table and false for supported unqualified names that are missing.
  Top-level single and grouped `const NAME = value;` declarations accept
  unqualified names and the current constant-expression subset (`null`,
  booleans, integers, floats, strings, arrays, unary expressions, and binary
  expressions over those values, plus bare references to previously defined
  unqualified constants and the current exact built-in `ARRAY_FILTER_*` and
  `SORT_*` constants). Grouped declarations execute left to right, so
  references to
  earlier declarators in the same group work and duplicate diagnostics point to
  the later duplicate declarator in the current group. Duplicate definitions,
  redefinition of the built-in constants, forward or otherwise undefined const
  declaration references, non-string or unsupported names, unsupported
  object-containing values, unknown `constant(...)` names, non-string or
  unsupported `defined(...)` names, unknown bare constants, and the legacy
  third `define(...)` flag fail with stable diagnostics. Magic constants are
  rejected by the parser before runtime constant lookup. Constant names that
  are lexed as language keywords or literals cannot be read bare, and
  case-insensitive legacy constants, namespace-qualified constants, extension
  constants, nested declarations, dynamic declaration values, class constants
  through
  `constant(...)`/unsupported `defined(...)` forms,
  references/copy-on-write behavior, and broader native lowering are not
  implemented.
  Array/object callables, closures, first-class callables, method calls,
  integer mode flags outside `0`, `1`, and `2`, non-int/non-bool mode
  coercions such as string `"0"`, references,
  copy-on-write containers, exact native `TypeError` objects, object handle
  identity preservation, resource values, and native lowering are not
  implemented.
  `array_map(null, $array)` returns an identity copy of one input array while
  preserving original integer/string keys and insertion order. `array_map(null,
  $array, ...)` with two or more input arrays returns a reindexed array whose
  entries are tuple arrays containing the input values at each insertion-order
  position, padding missing values from shorter arrays with `null`.
  `array_map($callback, $array, ...)` accepts callback expressions that
  evaluate to string function names resolving to current user functions or
  callable builtins. The one-array string-callback form invokes the callback
  with the value only and preserves original integer/string keys and insertion
  order. Multi-array string-callback forms invoke the callback with one value
  from each input array in insertion-order lockstep, follow PHP's longest array
  behavior by supplying `null` for missing values from shorter arrays, and
  reindex mapped values from integer key zero.
  Non-string callback values fail with a stable diagnostic, unresolved callback
  names fail with the current undefined-function diagnostic, and non-array
  input arrays fail with stable diagnostics. Array/object callables, closures,
  first-class callables, method calls, references, copy-on-write containers,
  exact native `TypeError` objects, object handle identity preservation,
  resource values, and native lowering are not implemented.
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
  and direct public object-property operands such as `isset($object->name)`.
  In active method context, direct private operands owned by the active
  declaring class and protected operands owned by the active class or an
  ancestor are also supported.
  `isset` can safely check undefined
  variables, missing/null array slots, undefined array variables, non-array
  array targets, and undefined object-property targets. Nested array offsets,
  append offset operands, dynamic property names, non-public property operands
  outside the current private/protected visibility context, complex lvalues,
  and general expression operands remain unsupported. `empty`
  supports one direct variable operand, one direct array offset operand such
  as `empty($array[$key])`, or one direct public object-property operand such
  as `empty($object->name)`. In active method context, direct private operands
  owned by the active declaring class and protected operands owned by the
  active class or an ancestor are also supported;
  undefined variables, missing array keys,
  undefined array targets, non-array array targets, missing object properties,
  undefined object targets, and non-object property targets are treated as
  empty, and existing values use the current PHP truthiness rules. Nested array
  offsets, dynamic property names, non-public property visibility context
  outside the current private/protected method context, append offset operands, complex lvalues,
  general expression operands, magic methods, and unsupported array-key
  coercions remain unsupported.
  `array_key_first`, `array_key_last`, `array_is_list`, `array_values`,
  `array_keys`, `array_reverse`, `array_slice`, `array_chunk`, `array_pad`,
  `array_merge`, `array_replace`, `array_combine`, `array_intersect_key`,
  `array_diff_key`, `array_diff`, `array_intersect`, `array_unique`, `array_flip`,
  `array_fill_keys`, `array_count_values`, `array_sum`, `array_product`,
  `array_reduce`, `array_filter`, `array_map`, `in_array`, `array_search`, and
  both current `foreach` array forms follow the current by-value model; PHP
  references, copy-on-write containers, object handle identity preservation,
  resource values, array, object, resource, or reference search values for
  `array_keys`, non-bool `array_keys` strict-flag coercion, non-bool
  `array_reverse` preserve-key flag coercion, non-bool `array_slice`
  preserve-key flag coercion, non-int offset coercion, non-int/non-null length
  coercion, non-bool `array_chunk` preserve-key flag coercion,
  non-int/non-positive length coercion, non-int `array_pad` length coercion,
  oversized `array_pad` native `ValueError` objects, exact native
  `ValueError`/`TypeError` objects, `array_merge` reference/copy-on-write
  behavior, `array_replace` reference/copy-on-write behavior,
  `array_combine` lossy or non-finite float key-value coercions,
  `array_combine` array/object/resource key values, `array_intersect_key` and
  `array_diff_key` exact native `TypeError` objects and
  reference/copy-on-write behavior, `array_diff` and `array_intersect`
  non-scalar value comparison behavior, `array_unique` sort flags outside
  `SORT_REGULAR`/`SORT_NUMERIC`/`SORT_STRING`, `array_unique` non-scalar
  value comparison
  behavior, exact native
  `TypeError` objects, and native lowering, `array_flip`
  warning-and-skip behavior
  for unsupported source values, and `array_fill_keys` warning/stringification
  behavior for unsupported key values, `array_count_values` warning-and-skip
  behavior for unsupported values, `array_sum` PHP warning recovery for
  unsupported values, `array_product` PHP warning recovery for unsupported
  values, `array_reduce` callback forms outside the current
  string function-name subset, and `array_filter` callback forms outside the
  current null-callback, value-only string function-name, key-only string
  function-name, and value/key string function-name modes, plus `array_filter`
  mode coercions outside the current int/bool/finite-integral-float/integral
  numeric string subset, and `array_map`
  callback forms outside current null-callback and string-valued function-name
  forms are not implemented.
  Because `isset` and `empty` are modeled as special static forms, they are not
  available through dynamic function lookup. PHP's complete warning behavior is
  not implemented.
- Object/class gaps: nested and conditional class declarations, constructor
  behavior beyond public/inherited public instance `__construct` and explicit
  parent calls,
  property override compatibility,
  broader `parent::`/`self::`/`static::`, broader inheritance rules,
  interface declarations, `implements` clauses, interface constants,
  interface method signatures, interface inheritance, namespace-aware
  interfaces, trait declarations, trait use inside classes,
  trait methods/properties/constants, trait conflict resolution, aliases,
  visibility changes, namespace-aware traits,
  enum declarations, enum cases, backed enum values, enum methods, enum
  interface implementations, namespace-aware enums,
  `abstract`/`final`/`readonly` class modifiers,
  `abstract`/`final`/`readonly` class member modifiers, abstract methods, final
  methods, readonly properties, typed property storage and enforcement,
  property initialization rules, inheritance interactions, property defaults,
  multiple properties in one declaration, per-property defaults in
  multi-property declarations, class constant declarations, constants, static
  property storage, late static binding, magic methods, namespaces,
  autoloading, anonymous classes, attributes, reflection, dynamic properties,
  dynamic property names, dynamic method names, protected method visibility outside
  same-class/child method contexts, non-public property access outside the
  current private/protected method context, broader
  constructor visibility context, static member
  execution through `::`, `::class` class-name constant resolution, property assignment
  targets other than a direct variable, dynamic properties created outside
  declarations, autoload side effects from property introspection,
  object handle identity/aliasing,
  cloning, destructors, serialization hooks, visibility enforcement,
  `self`/`parent`/`static`, object comparisons, `instanceof` relationship
  checks, object-to-string conversion, object callables, and native lowering
  are unsupported.
- Constructor boundary: public instance `__construct` methods, including
  inherited public constructors and explicit public/protected
  `parent::__construct(...)` calls from instance context, execute in
  `phpc run` with scoped `$this`. Protected constructors are callable from
  same-class or child-class method context through `new ClassName(...)`.
  Constructor arguments for classes without a constructor, private
  constructors without same-class construction context, protected constructors
  outside same-class/child-class construction context, static constructors,
  constructor promotion, explicit parent calls outside active child instance
  context, named arguments, references/copy-on-write, exact PHP
  `Error`/`TypeError` object behavior, and native lowering remain unsupported.
- Scalar arithmetic gaps: leading numeric strings with trailing non-numeric
  characters, such as `"10 apples"`, are rejected instead of warning and
  continuing with the leading number. PHP's warning/notice recovery mode,
  locale-sensitive numeric parsing, and exact integer-overflow promotion rules
  are not implemented. Native arithmetic lowers same-type integer or same-type
  float operands for `+`, `-`, and `*`, plus integer `%` when the divisor is a
  statically known positive integer. Integer modulo by one also folds after
  both operands lower when the dividend is intentionally untracked, such as an
  overflow-sensitive shift result; other modulo cases still require a
  statically known positive divisor and keep the documented runtime-check
  boundary. Identical tracked integer expression
  operands and identical integer literal operands for `-` fold to zero without
  a redundant native subtraction; identical tracked finite float expression
  operands and identical finite float literals for `-` fold to `0.0` without a
  redundant native subtraction; identical integer subtraction also folds after
  both operands lower when the value is intentionally untracked, such as
  overflow-sensitive shift results, while other non-identity arithmetic with
  such values still rejects because exact overflow tracking is unavailable;
  tracked integer expression operands and integer
  literal operands for `$x + 0`, `0 + $x`, `$x - 0`, `$x * 1`, and `1 * $x`
  reuse the existing value; tracked integer expression operands and integer
  literal operands for `$x * 0` and `0 * $x` fold to zero; integer identity or
  annihilator forms `+ 0`, `- 0`, `* 1`, and `* 0` also fold after both
  operands lower when the other integer operand is intentionally untracked,
  such as overflow-sensitive shift results, while non-identity arithmetic with
  such values still rejects because exact overflow tracking is unavailable;
  tracked finite float expression operands and finite float literals for nonzero
  `$x + 0.0`, `0.0 + $x`, `$x - 0.0`, `$x * 1.0`, and `1.0 * $x` reuse the
  existing expression; single-result statically known nonzero finite
  `0.0 - $x` folds to the known negated float literal, while signed-zero and
  non-finite float identity/subtraction cases stay emitted or rejected;
  tracked finite positive float expression operands and finite positive float
  literals for `$x * 0.0` and `0.0 * $x` fold to positive `0.0`, while
  negative and signed-zero-sensitive multiplication-by-zero cases stay emitted;
  single-result statically known nonzero finite `$x * -1.0` and `-1.0 * $x`
  fold to the known negated float literal, while signed-zero-sensitive
  multiplication by `-1.0` stays emitted;
  well-formed numeric strings, scalar
  coercions, mixed int/float arithmetic, `/`, dynamic or non-positive modulo
  divisors, division/modulo zero checks, modulo coercions, and PHP overflow
  behavior that `phpc run` can execute remain unsupported.
- Scalar comparison gaps: strict identity is implemented only for the current
  scalar values. Strict identity for arrays, objects, resources, references,
  and object handle identity is not implemented. LLVM IR/assembly emission
  lowers only documented same-type `null`, boolean, integer, finite-float,
  known ASCII nonnumeric NUL-free string loose/ordering comparisons, identical
  string-pointer self-comparisons, and the documented strict-identity scalar
  subset; other loose, ordering, untracked/non-finite float, non-identical
  unknown string, numeric-looking string, NUL-containing string, or mixed-type
  comparisons are rejected instead of lowering partial PHP comparison
  semantics.
  Array/object strict identity operands fail with stable unsupported-comparison
  runtime diagnostics. Float identity currently follows Rust/PHP-style `f64`
  equality for representable literals and does not claim broader `NAN`/`INF`
  precision edge-case coverage.
- Array gaps: array spread elements, array reference elements, and
  `list(...)`/`[...]` destructuring assignment targets are rejected with
  stable parse diagnostics. `unset(...)` forms outside direct variables and
  direct array-offset operands, comma-separated `for` header expression lists,
  expression-form `do ... while`, expression-form `switch`, malformed
  alternate switch bodies, and exponentiation syntax `**`/`**=` are rejected
  with stable parse diagnostics; object property removal, append-offset unset,
  and nested/complex unset operands are not implemented.
  Nested indexed writes, complex assignment lvalues, nested/complex
  `isset(...)` and `empty(...)` array offset operands, native
  `isset($array[$key])` lowering, `$array[]` as a read expression, string
  offset access, by-reference `foreach`, object iteration, destructuring loop
  targets, array destructuring assignments with keyed, nested, reference,
  skipped-slot, or by-value unpacking semantics, references, copy-on-write
  containers, and
  object/resource keys are not implemented. The current `foreach` array forms
  snapshot array entries at loop start and do not claim PHP's full
  mutation/aliasing behavior while the iterated array is modified. Array keys
  are currently limited to values that
  evaluate to integers or strings; PHP's boolean, null, float, object, and
  resource key coercions are rejected with a stable runtime error.
  Writes to existing non-array scalar variables other than `null` are rejected
  instead of following PHP's full automatic conversion behavior. Negative-key
  auto-index behavior is not claimed beyond the current non-negative allocator.
  Native array lowering is not implemented; `phpc compile --emit-ir` and
  `--emit-asm` reject array literals, offset reads/writes, `foreach`, array
  offset `unset`, array offset `isset`, and array builtins before claiming any
  generated array
  storage, key normalization, callback dispatch, references, copy-on-write, or
  exact native error behavior.

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
- dynamic method names and dynamic property names; `$object->$name` and
  `$object->$method()` currently fail with stable parse diagnostics
- private instance method dispatch outside same-class method context,
  protected instance method dispatch outside same-class/child method context,
  static methods called through object receivers, and `$this` outside instance
  method execution currently fail with stable runtime diagnostics
- non-public object property access and property writes to lvalues other than a
  direct variable
- constructor arguments for classes without a declared constructor, non-public
  constructors, and static constructors currently fail with stable runtime
  diagnostics
- unsupported class forms including nested/conditional declarations, broader
  inheritance rules beyond the current single-parent metadata chain, interface
  declarations and `implements` clauses, interface constants, interface method
  signatures, interface inheritance, trait
  declarations, enum declarations, enum cases/backing values/methods/interface
  implementation,
  typed property storage/enforcement, property defaults, multiple properties in
  one declaration, per-property defaults in multi-property declarations,
  class constant declarations, constants, parent static properties/constants,
  and anonymous classes
- static property access, static method calls, class constant access,
  class-name constant access, unsupported self/parent static
  property/constant/class-name forms, and `static::` through `::`
- variable variables; `$$name` and `${...}` are rejected with a stable lex
  diagnostic rather than executed
- `global` declarations / importing top-level variables into function scope
- default parameter values outside the documented constant-expression and
  unqualified constant-reference subset
- required parameters after default parameters
- variadic parameters and variadic argument unpacking
- reference parameters, reference returns, reference assignments, and
  by-reference calls
- parameter type declarations and return type declarations, including
  nullable, union, intersection, `mixed`, `void`/`never`, class/interface
  names, coercive versus strict typing, variance, and native lowering
- static local variable declarations inside functions, including
  initialization expressions, per-function persistence, references,
  recursion/reentrancy behavior, and native lowering
- magic constants other than `__LINE__`, `__FILE__`, `__DIR__`, and
  `__FUNCTION__`, such as `__CLASS__`, `__TRAIT__`, `__METHOD__`, and
  `__NAMESPACE__`; `__METHOD__` specifically fails with a stable parse
  diagnostic because method-context magic constant tracking is not
  implemented, and `__CLASS__` specifically fails because class-context
  tracking is not implemented. `__TRAIT__` specifically fails because trait
  declarations, trait use, and trait-context tracking are not implemented,
  and `__NAMESPACE__` specifically fails because namespace-aware name
  resolution is not implemented.
  `__FUNCTION__` is limited to user-function context and the top-level empty
  string behavior; closure context is not implemented. `__FILE__` currently
  reports the `phpc run` input path string, and `__DIR__` derives from that
  same path string; neither is guaranteed to match PHP's canonical absolute
  filename or directory in all entry paths. Native lowering rejects executable
  magic constants `__LINE__`, `__FILE__`, `__DIR__`, and `__FUNCTION__` with a
  specific codegen diagnostic until source mapping, path canonicalization, and
  function-context lowering exist.
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
- expression-form `switch`, malformed alternate colon/`endswitch` switch
  bodies, and `continue;` behavior inside switch
- `break`/`continue` loop-depth arguments such as `break 2;` and `continue 2;`;
  only statement-form `break;` for the innermost active `while`, supported
  `for`, supported `do ... while`, supported array `foreach`, or supported
  `switch`, and `continue;` for the innermost active loop are implemented
- native lowering for `if`/`elseif`/`else`, `while`, `for`, `do ... while`,
  `switch`, `break`, and `continue`; generated code currently rejects those
  forms before lowering conditions, bodies, cases, or loop-control flow
- exception execution; `throw`, throw expressions, `try` blocks, `catch`
  clauses, and `finally` blocks currently fail with stable parse diagnostics
  before `Throwable`, `Exception`, custom exception classes, stack unwinding,
  `finally` execution, stack traces, exact native error objects, or native
  lowering exist
- PHP 8 `match` expressions currently fail with a stable parse diagnostic
  before expression-form branching exists. Strict arm matching, default arms,
  exhaustiveness errors, thrown expressions inside arms, value evaluation
  order, exact native error objects, and native lowering are not implemented.
- `goto` statements and labels currently fail with a stable parse diagnostic
  before jump-target resolution, cross-scope jump validation, `finally`
  interaction, or native lowering exists.
- heredoc/nowdoc string syntax currently fails with a stable lex diagnostic
  before multiline string tokenization, interpolation, indentation stripping,
  exact label parsing, runtime string construction, or native string lowering
  exists.
- Full ternary conditional expressions `$condition ? $if_true : $if_false`
  execute over the current expression/value subset with truthiness-based
  condition selection and lazy branch evaluation. Short ternary expressions
  `$value ?: $fallback` evaluate the condition once, return that original
  condition value when truthy, and lazily evaluate the fallback only for falsey
  condition values. Parenthesized nested ternary expressions are supported.
  Current executable coverage also pins `??` precedence in ternary conditions
  and branches, and lazy selected-branch behavior when full and short ternaries
  contain direct assignment, compound-assignment, and null coalescing
  assignment expressions. Unparenthesized nested ternaries, thrown expressions
  inside arms, references, copy-on-write aliasing, exact native error objects,
  and native lowering are not implemented.
- Assignment expressions are limited to direct static variables as
  `$name = expr`, direct array offsets as `$array[$key] = expr`, direct public
  object properties as `$object->property = expr`, direct append offsets as
  `$array[] = expr`, and null coalescing assignment expressions
  `($name ??= expr)`, `($array[$key] ??= expr)`, and
  `($object->property ??= expr)`. They write the active scope's static
  variable, current ordered array offset, appended array slot, or existing
  declared public property slot and return the assigned or existing value.
  Direct static-variable, direct array-offset, and direct public
  object-property assignment expressions can be chained with right-to-left
  result semantics, so `$left = $right = expr`, `$left = $array[$key] = expr`,
  and `$left = $object->property = expr` assign the inner target first and
  then store that result in the outer target. The chained right-hand value may
  also be a direct compound assignment such as `$left = ($right += expr)` or a
  direct null coalescing assignment such as `$left = ($right ??= expr)`, reusing
  the inner assignment expression result. Direct array-offset assignment
  expressions evaluate the key before the right-hand expression, materialize
  undefined or `null` target variables as arrays, and reject existing
  non-array targets with a stable runtime diagnostic. Direct
  append-offset assignment expressions evaluate the right-hand expression,
  append to direct array variables, materialize undefined or `null` target
  variables as arrays, and reject existing non-array targets with a stable
  runtime diagnostic; append offsets are not supported inside chained
  assignment expressions.
  Direct object-property assignment expressions evaluate the right-hand expression
  before validating/writing the direct object-variable target, reject
  undefined or non-object targets and missing/non-public properties with stable
  runtime diagnostics, and do not materialize missing properties. Direct
  null coalescing assignment expressions use the same lazy evaluation and
  materialization behavior as the supported statement forms. The supported
  assignment-expression values are executable in ordinary expression positions
  covered by the current parser, including function-call arguments, array
  literal keys and values, `if`/`while`/`for` conditions, and builtin
  arguments; native codegen still rejects assignment expressions explicitly, and
  enclosing unsupported constructs may reject before lowering nested
  assignment values. Nested
  append/offset assignment expressions, append-offset chained assignment
  expressions, dynamic property names, append-offset `??=` targets, reference
  assignment, copy-on-write container aliasing, exact native error objects, and
  native lowering are not implemented.
- Compound assignment is limited to direct static variables, direct
  array-variable offsets, direct public object properties, private properties
  in active declaring-class method context, and protected properties owned by
  the active class or an ancestor over the current scalar/object value model.
  The
  read-modify-write operation reuses the existing PHP-shaped scalar arithmetic,
  modulo, bitwise/shift, and string concatenation helpers, so undefined
  left-hand variables, missing array keys, missing object properties, non-array
  targets, non-object property targets, non-public properties outside the
  current private/protected visibility context, division by
  zero, modulo by zero, non-numeric strings, arrays, and objects as operand
  values fail through existing stable runtime diagnostics.
  Statement forms, expression forms such as `($name += expr)` and
  `($array[$key] += expr)` and `($object->property += expr)`, and single
  C-style `for` initializer/increment actions are supported for those direct
  targets; expression forms return the updated value. Append offsets, nested
  offsets/properties, dynamic property names, non-public visibility context,
  references/copy-on-write, PHP warning recovery, exact native error objects,
  and native lowering are not implemented.
- Pre/post increment and decrement is limited to direct static variables,
  direct array offsets, and direct public object properties whose current
  values are integers or floats, either as standalone statements,
  expressions, or single C-style `for` initializer/increment actions.
  Expression pre forms return the updated value and expression post forms
  return the previous value. Strings, arrays/objects as current values,
  undefined variables, missing array keys, non-array offset targets, append
  offsets, nested offsets/properties, dynamic property names, non-public
  visibility context, missing-property materialization, references,
  copy-on-write, exact native warning/error behavior, PHP string increment
  semantics, broader coercion recovery, and native lowering are not
  implemented.
- Null coalescing is limited to direct static variables, direct array-variable
  offsets, and direct object-variable public properties on the left side, plus
  direct-variable `$name ??= expr`, direct array-offset `$array[$key] ??=
  expr`, and direct public object-property `$object->property ??= expr`
  statements and parenthesized expression forms. `??=` expression forms return
  the assigned fallback or existing non-null value. Object-property `??=`
  writes only existing declared public properties on existing object values;
  missing properties, undefined target variables, and non-object target
  variables fail with stable diagnostics.
  Complex or nested `??` left operands, append-offset `??=` targets, dynamic
  property names, non-public visibility context, magic methods,
  unparenthesized chained coalescing, precedence interactions beyond the
  current single-operator expression slice, references/copy-on-write, exact
  native error objects, and native lowering are not implemented.
- Native lowering for conditional expressions is intentionally partial. LLVM
  IR/assembly emission lowers full ternary expressions only when the condition
  is already a lowerable boolean or native boolean expression and both branch
  values are already lowerable integers, booleans, floats, strings, or both
  branches are `null` in the same straight-line subset, or when the condition
  is a statically known boolean and both branch values are already lowerable
  scalar values, or when the condition and both branches are the same direct
  variable whose current value is already lowerable. Dynamic mixed-type branch values are rejected until native
  tagged values exist. It emits `select` or a C conditional expression for
  dynamic non-null boolean conditions, folds identical static string branches
  to that string without a pointer select, folds identical tracked numeric
  expression branches and identical numeric literal branches without a numeric
  select, folds identical integer branches after both branches lower even
  when the integer value is intentionally untracked, such as an
  overflow-sensitive shift result, folds identical direct-variable full
  ternaries such as `$value ? $value : $value` without proving truthiness when
  all three operands are the same already-lowerable direct variable, including
  untracked integer, non-finite float-producing, and string pointer
  expressions, boolean expressions, and null values, and folds identical float branches after
  both branches lower even when the value is intentionally untracked, such as
  a non-finite overflowing float multiplication. It folds boolean literal branches such as `$flag ? true : false` and
  `$flag ? false : true` without a boolean select, folds dynamic
  `null`/`null` ternaries to `null`, folds static boolean conditions to the selected branch
  value, folds dynamic integer, finite-float, and boolean ternaries whose
  possible branch values collapse to a single known result without a redundant
  select, folds full ternary conditions with null or single-known integer,
  finite-float, or known-string truthiness to the selected already-lowerable
  branch without lowering the unselected branch, including direct
  null-variable conditions that select the false branch without lowering
  unsupported true-branch calls, and lowers
  short ternary `?:` for lowerable boolean conditions when
  dynamic boolean forms have lowerable boolean fallbacks. Static-false short
  ternaries return any already-lowerable scalar fallback, and static-true short
  ternaries fold to `true` without lowering the fallback. Single-known integer
  conditions also fold through integer truthiness, reusing proven nonzero
  integer results and using the fallback for proven zero integer results.
  Single-known finite float conditions fold through float truthiness the same
  way, reusing proven nonzero finite float results and using the fallback for
  proven zero float results. Known string conditions fold through PHP string
  truthiness when all possible values have the same truthiness, reusing
  truthy string results and using the fallback for `""`/`"0"` string results.
  Identical direct boolean-, integer-, float-, and string-variable short
  ternaries such as `$flag ?: $flag`, `$value ?: $value`, and `$text ?: $text`
  also reuse already-lowerable expressions without proving broader truthiness,
  including boolean expressions, untracked integer expressions, untracked
  non-finite float-producing expressions, and untracked string pointer
  expressions. Null short ternaries
  use the fallback for `null ?: fallback`, including direct null-variable
  fallback forms such as `$value ?: $value`. It rejects
  broader null truthiness in logical binaries or null coalescing, plus general
  PHP truthiness, lazy branch evaluation for
  unsupported or side-effecting branches, ambiguous string truthiness,
  non-identical untracked integer, float, or string expressions, non-finite float truthiness, other non-boolean
  truthiness, arrays, objects, references/copy-on-write behavior, and exact
  native error objects.
- Native lowering for unary operators is intentionally partial. LLVM
  IR/assembly emission lowers unary minus only for operands that are already
  lowerable integers or floats and logical not only for operands that are
  already lowerable booleans or native boolean expression results, `null`, or
  known integers, finite floats, and strings whose possible values all have the
  same PHP truthiness, in the same straight-line subset. Dynamic boolean double
  logical-not expressions such as `!!$flag` reuse the original native boolean
  expression instead of emitting redundant inversions. Double logical-not over
  known scalar operands such as integers, finite floats, strings, and `null`
  folds through the same known-truthiness subset without emitting boolean
  operations. Native lowering folds
  logical not over single-result statically known native boolean expression
  operands to the known boolean result in LLVM IR and in the C assembly
  fallback when the C boolean expression has a tracked result. Known numeric
  logical-not folds to a static boolean for zero and nonzero known
  integer/finite-float operands when all possible values have the same
  truthiness. Known string logical-not folds to a static boolean for `""`,
  `"0"`, and known-truthy string operands when all possible string values have
  the same truthiness. Null logical-not folds to `true` without claiming
  broader null truthiness beyond the documented logical binary folding subset. Integer unary-minus
  results remain statically tracked
  for later checked integer arithmetic when all bounded possible negation
  results are proven not to overflow; single-result statically known integer
  operands fold to the known negated result without a redundant native
  unary-minus operation. Finite float unary-minus results remain tracked for
  later strict-identity folding when every possible negation result is proven;
  single-result statically known nonzero finite float operands fold to the
  known negated result without a redundant native unary-minus operation. It
  rejects boolean, string, null, array, and object
  unary-minus operands, so generated code does not imply PHP
  numeric coercion. It rejects ambiguous numeric or string logical-not
  truthiness, untracked numeric/string logical-not expressions, non-finite
  float logical-not truthiness, null truthiness outside logical-not, other
  truthiness conversion, unary integer overflow behavior,
  references/copy-on-write behavior, or exact native error objects.
- Native lowering for comparison operators is intentionally partial. LLVM
  IR/assembly emission lowers same-type `null`, boolean, integer,
  finite-float, known ASCII nonnumeric NUL-free string loose/ordering
  comparisons, and identical string-pointer self-comparisons for `==`, `!=`,
  `<`, `<=`, `>`, and `>=`, plus strict identity `===` and `!==` when both
  operands are already lowerable `null`, integers, booleans, floats, or
  strings in the same straight-line subset.
  Static
  same-type scalar identity
  folds at compile time, bounded integer, float, string, and boolean identity
  fold when all possible `===`/`!==` outcomes are proven identical. Identical
  lowerable dynamic scalar operands fold for integers, booleans,
  already-lowerable string pointers, and finite tracked floats, so `$x === $x`
  and `$x !== $x` avoid runtime comparisons in those safe scalar cases.
  Identical lowerable integer operands also fold for loose/ordering
  comparisons, including intentionally untracked integer expressions such as
  overflow-sensitive shift results: `$x == $x`, `$x <= $x`, and `$x >= $x`
  fold true, while `$x != $x`, `$x < $x`, and `$x > $x` fold false.
  Dynamic boolean expression operands compared with boolean literals fold for
  `$flag === true`, `true === $flag`, `$flag !== false`, and `false !== $flag`
  by reusing the original native boolean expression, and inverse forms such as
  `$flag === false`, `false === $flag`, `$flag !== true`, and `true !== $flag`
  use the native boolean inversion path.
  Dynamic boolean expression operands compared loosely with boolean literals
  fold for `$flag == true`, `true == $flag`, `$flag != false`, and
  `false != $flag` by reusing the native boolean expression, while inverse
  forms such as `$flag == false`, `false == $flag`, `$flag != true`, and
  `true != $flag` use the native boolean inversion path.
  Dynamic boolean expression operands ordered against boolean literals also
  fold within boolean semantics, reusing the expression, inverting it, or
  folding to a static boolean for cases such as `$flag > false`,
  `$flag < true`, `$flag <= true`, and `true >= $flag`.
  Same-type integer and finite-float loose/ordering comparisons whose tracked
  possible operands prove one result fold to a static boolean. Literal-only
  comparisons still fold, while ambiguous tracked finite-float comparisons
  stay emitted as native comparisons.
  Boolean expression comparisons whose tracked possible operands prove one
  loose/ordering result also fold to that static boolean without emitting a
  redundant native boolean comparison. Identical native boolean expression
  operands also fold for loose/ordering comparisons, including ambiguous
  boolean expressions: `$flag == $flag`, `$flag <= $flag`, and `$flag >=
  $flag` fold true, while `$flag != $flag`, `$flag < $flag`, and `$flag >
  $flag` fold false. Other ambiguous boolean expression comparisons stay
  emitted. Identical native string pointer operands also fold for
  loose/ordering comparisons, including untracked string pointer expressions
  whose possible value set exceeds the current small tracker: `$text ==
  $text`, `$text <= $text`, and `$text >= $text` fold true, while `$text !=
  $text`, `$text < $text`, and `$text > $text` fold false. Non-identical
  unknown string comparisons stay rejected.
  Statically known integer strict-identity comparison results remain tracked
  for later boolean scalar lowering even when the comparison itself stays
  emitted as `icmp`. Same-type ambiguous dynamic integer, boolean, float, and
  already-lowerable string pointer identity lower through native comparisons
  and PHP-shaped boolean echo output, and already lowerable mixed scalar
  operands with different PHP scalar types fold without runtime comparison
  calls. Ambiguous dynamic string identity uses `strcmp` for string pointers
  produced by the current native string ternary subset. Known ASCII nonnumeric
  string loose/ordering comparisons fold to a static boolean when every
  possible safe string outcome matches; ambiguous safe string loose/ordering
  comparisons lower through `strcmp`. Statically known boolean, integer, and
  finite-float loose/ordering comparison results remain tracked for later
  boolean scalar lowering even when the comparison itself stays emitted as
  `icmp`/`fcmp`; ambiguous bounded boolean, finite-float, or string
  loose/ordering comparison results remain dynamic and untracked.
  It rejects ambiguous bounded integer, float, string, or boolean identity, broader
  value-correlation proofs across related expressions such as `$x` and `!$x`,
  numeric-looking, non-identical unknown, non-ASCII, or NUL-containing string loose/ordering comparisons,
  mixed null or other mixed-type comparisons, untracked or
  non-finite float comparisons, dynamic null identity beyond static/type-only folds, PHP truthiness
  conversion for loose logical operands, arrays, objects,
  non-lowerable float sources, and dynamic string allocation beyond the static
  straight-line subset, so generated code does not imply PHP comparison
  coercions, non-scalar comparison behavior, references/copy-on-write behavior,
  or exact native error objects.
- Native lowering for binary arithmetic operators is intentionally partial.
  LLVM IR/assembly emission lowers only `+`, `-`, and `*` for operands that
  are already same-type lowerable floats or same-type lowerable integers whose
  result is statically proven not to overflow in the same straight-line subset,
  plus integer `%` when the divisor is a statically known positive integer.
  Statically known modulo results remain tracked for later checked integer
  arithmetic, and tracked integer expression operands or integer literal
  operands for `$x % 1` fold to zero. Bounded tracked integer expression
  operands whose possible values all produce the same remainder for a positive
  literal divisor fold to that remainder. Tracked integer expression
  arithmetic for `+`, `-`, and `*` folds to the known integer literal after
  checked overflow analysis when tracked possible integer operands prove one
  result, while literal-only integer arithmetic and ambiguous
  tracked-expression plus tracked-expression integer arithmetic stay emitted.
  Tracked finite nonzero float
  expression operands and finite nonzero float literals for `$x + 0.0`,
  `0.0 + $x`, and `$x - 0.0` reuse the existing expression, while possible
  signed-zero float identities stay emitted. Single-result statically known
  nonzero finite `0.0 - $x` folds to the known negated float literal, while
  possible signed-zero left-zero subtraction stays emitted. Tracked finite
  positive float expression operands and finite positive float literals for
  `$x * 0.0` and `0.0 * $x` fold to positive `0.0`, while negative and
  signed-zero-sensitive multiplication-by-zero cases stay emitted.
  Single-result statically known nonzero finite `$x * -1.0` and `-1.0 * $x`
  fold to the known negated float literal, while signed-zero-sensitive
  multiplication by `-1.0` stays emitted. Tracked finite nonzero float
  expression arithmetic for `+`, `-`, and `*` folds to the known float literal
  when tracked possible finite-float operands prove one nonzero result, while
  literal-only float arithmetic and zero-result arithmetic stay emitted. It
  rejects mixed int/float arithmetic, strings, booleans, nulls,
  arrays, objects, `/`, overflow-sensitive or not-statically-proven integer
  arithmetic, dynamic or non-positive modulo divisors, modulo results that are
  not statically known enough for later checked arithmetic, and modulo cases
  that need PHP coercion or runtime checks, so generated code does not imply
  PHP numeric coercion, dynamic division/modulo zero checks, modulo coercions,
  negative-divisor/min-int modulo edge behavior, integer overflow promotion,
  references/copy-on-write behavior, or exact native error objects. Mixed
  int/float `+`, `-`, and `*` operands use a
  mixed-numeric-specific codegen diagnostic until generated code has PHP
  numeric promotion and exact result typing. Boolean, null, and string operands
  in `+`, `-`, and `*` use a scalar-coercion-specific codegen diagnostic until
  generated code has PHP numeric coercion and string numeric parsing.
  Overflow-sensitive or not-statically-proven integer `+`, `-`, and `*` cases
  use an integer-overflow-specific codegen diagnostic until generated code has
  PHP integer overflow promotion and runtime checks. Native `/` uses a
  division-specific codegen diagnostic until generated code has PHP division
  semantics, runtime zero checks, and no misleading integer truncation.
  Dynamic, zero, or non-positive integer modulo
  divisors use a modulo-specific codegen diagnostic until native runtime checks
  exist.
- Native lowering for string concatenation is intentionally partial. LLVM
  IR/assembly emission lowers `.` only when both operands are already
  lowerable strings in the same straight-line subset, including ternary
  operands that prove one static string result, folding the result into a
  generated static string constant. Empty-string concatenation identity also
  folds for already-lowerable string operands, including untracked string
  pointer expressions: `$text . ""` and `"" . $text` reuse `$text` without
  runtime string allocation. It rejects scalar-to-string conversion, non-empty
  ambiguous string expressions, arrays, objects, resources, runtime string
  allocation, references/copy-on-write behavior, and exact native error objects.
- Logical operators are limited to `&&`, `||`, `and`, `xor`, and `or` over the
  current interpreter truthiness rules. `&&`, `||`, `and`, and `or`
  short-circuit, `xor` evaluates both operands, all return booleans, and
  fixture coverage exercises symbolic precedence plus word-operator precedence
  around direct assignment expressions. Native LLVM IR/assembly lowering
  accepts operands that are already lowerable booleans or native boolean
  expression results, plus already-lowerable scalar operands whose possible
  values all have one known PHP truthiness result, in the same straight-line
  subset. Static boolean pairs fold, and static boolean identity and
  annihilator edges preserve proven boolean results for later scalar lowering
  without claiming broader short-circuit support. Identical native boolean
  expression operands for `&&`/`and` and `||`/`or` reuse the existing
  expression without a redundant native boolean operation, and identical native
  boolean expression operands for `xor` fold to `false`. Native boolean
  expression operations whose tracked possible operands prove one result fold
  to that static boolean without a redundant native boolean operation. Known
  scalar logical operands whose null, integer, finite-float, or string truthiness is
  unambiguous fold to a static boolean result without emitting a native boolean
  operation. Statically decisive known-left `&&`/`and` and `||`/`or`
  short-circuit cases such as `false && rhs` and `true || rhs` lower without
  lowering the skipped right-hand operand. Other dynamic boolean expressions
  lower to native boolean operations with PHP-shaped boolean echo output. Native
  lowering still rejects general PHP truthiness conversion, dynamic
  short-circuiting, `xor` right-hand skipping, selected/evaluated unsupported
  right-hand operands, ambiguous scalar truthiness, untracked scalar logical
  operands, non-finite float truthiness, null coalescing, arrays, objects,
  references/copy-on-write side effects, exact native error objects,
  linking/execution, and broader native lowering.
- Bitwise operators are limited to `&`, `|`, `^`, unary `~`, and shift
  operators `<<`/`>>` over the current integer/string subset. Mixed binary
  operands and shift operands use the current
  scalar-to-int coercion path; string operands use bytewise operations but
  still store results in the runtime's UTF-8 `String` value, so arbitrary
  binary outputs that are not valid UTF-8 fail with a stable runtime
  diagnostic. Unary `~` currently accepts integers and string operands whose
  bytewise-not result remains valid UTF-8; boolean, null, float, array, and
  object operands are rejected with stable runtime diagnostics instead of exact
  native `TypeError` objects. Shift operators return zero for left shifts with
  counts at least the native integer width and sign-fill right shifts for
  large counts; negative shift counts fail with a stable project diagnostic.
  Non-numeric mixed strings fail instead of modeling PHP's exact native
  `TypeError` object, arrays/objects are rejected for binary bitwise and shift
  operators, append-offset/nested bitwise compound-assignment targets, PHP
  warning/deprecation recovery for float-to-int precision loss,
  references/copy-on-write side effects, exact native error objects, and broad
  native lowering are not implemented. LLVM IR/assembly emission lowers only
  the already-lowerable integer subset for binary `&`, `|`, `^`, unary `~`,
  and shifts with statically known counts from 0 through 63. Bounded
  statically known integer bitwise and unary bitwise-not results remain tracked
  for later checked integer arithmetic. Single-result statically known integer
  operands for unary `~` fold to the known bitwise-not result without a
  redundant native bitwise-not operation. Double unary bitwise-not `~~$x` over
  an already-lowerable integer operand reuses `$x`, including intentionally
  untracked integer expressions such as overflow-sensitive shift results.
  Identical tracked integer expression
  operands and identical integer literal operands for `&` and `|` reuse the
  existing value, and identical tracked integer expression operands and
  identical integer literal operands for `^` fold to zero. Identical integer
  operands also fold after both operands lower when the value is intentionally
  untracked, such as overflow-sensitive shift results: `$x & $x` and `$x | $x`
  reuse `$x`, while `$x ^ $x` folds to zero. Tracked integer
  expression operands and integer literal operands for `$x & -1` and
  `-1 & $x`, and for `$x | 0`, `0 | $x`, `$x ^ 0`, and `0 ^ $x`, reuse the
  existing value. Tracked integer expression operands and integer literal
  operands for `$x & 0` and `0 & $x` fold to zero. Tracked integer expression
  operands and integer literal operands for `$x | -1` and `-1 | $x` fold to
  `-1` after both operands lower. Single-known integer operands for `$x ^ -1`
  and `-1 ^ $x` fold to the known bitwise-not result. The `& 0`, `& -1`,
  `| 0`, and `^ 0` identity or annihilator forms also fold after both operands
  lower when the other integer operand is intentionally untracked, such as
  overflow-sensitive shift results. Tracked integer expression
  bitwise operations for `&`, `|`, and `^` fold to the known integer literal
  when tracked possible integer operands prove one result, while literal-only
  integer bitwise operations and ambiguous tracked-expression plus
  tracked-expression bitwise operations stay emitted.
  Bounded statically known
  safe shift results remain tracked for later checked integer arithmetic, and
  tracked integer expression operands and integer literal operands for
  `$x << 0` and `$x >> 0` reuse the existing value. Those shift-by-zero
  identities also fold after both operands lower when the left integer operand
  is intentionally untracked, such as an overflow-sensitive shift result.
  Tracked single-result integer expression shifts with literal counts or
  tracked integer expression counts that prove one safe count fold to the known
  integer literal, while literal-only shifts and non-single tracked integer
  shifts stay emitted, and overflow-sensitive left-shift result sets remain
  unknown. It rejects
  ambiguous dynamic shift counts, negative or large counts, string bitwise operands,
  scalar-to-int coercion for non-integer operands, arrays, and
  objects so generated code does not imply partial PHP bytewise string,
  coercion, overflow, or complete shift-count semantics.
- dynamic callables outside the string function-name subset, including array
  callables, object/method callables, first-class callable syntax,
  `call_user_func`, and namespace/autoload-aware callable resolution
- `array_key_exists` lossy or non-finite float key coercion and PHP
  warning/deprecation behavior, array/object/resource/reference keys, exact
  native `TypeError` objects, reference/copy-on-write behavior, and native
  lowering
- `array_key_first`/`array_key_last`/`array_is_list` exact native `TypeError`
  objects, reference/copy-on-write container behavior, and native lowering
- `array_keys` filtering over array, object, resource, or reference search
  values or array values, plus non-bool strict-flag coercion
- `in_array` and `array_search` strict-mode searches involving
  array/object/resource/reference values, non-bool strict-flag coercion, and
  array/object needle or haystack-value comparisons for the current
  array-search builtins
- `array_reverse` non-bool `preserve_keys` coercion, reference/copy-on-write
  behavior, object handle identity preservation, resource values, and native
  lowering
- `array_slice` non-bool `preserve_keys` coercion, non-int offset coercion,
  non-int/non-null length coercion, reference/copy-on-write behavior, object
  handle identity preservation, resource values, exact native `TypeError`
  objects, and native lowering
- `array_chunk` non-bool `preserve_keys` coercion, non-int/non-positive length
  coercion, exact native `ValueError`/`TypeError` objects,
  reference/copy-on-write behavior, object handle identity preservation,
  resource values, and native lowering
- `array_pad` non-int length coercion, exact native `ValueError`/`TypeError`
  objects, reference/copy-on-write behavior, object handle identity
  preservation, resource values, and native lowering
- `array_merge` reference/copy-on-write behavior, object handle identity
  preservation, resource values, exact native `TypeError` objects, and native
  lowering
- `array_replace` reference/copy-on-write behavior, object handle identity
  preservation for object values, resource values, exact native `TypeError`
  objects, and native lowering
- `array_combine` lossy or non-finite float, array, object, resource, and
  reference key-value coercions, length mismatch native `ValueError` objects,
  non-array native `TypeError` objects, reference/copy-on-write behavior, object handle
  identity preservation for object values, resource values, and native
  lowering
- `array_intersect_key` exact native `TypeError` objects,
  reference/copy-on-write behavior, object handle identity preservation for
  object values, resource values, and native lowering
- `array_diff_key` exact native `TypeError` objects, reference/copy-on-write
  behavior, object handle identity preservation for object values, resource
  values, and native lowering
- `array_diff` non-scalar value comparisons, exact native `TypeError` objects,
  PHP warning-and-string-conversion behavior for arrays and objects,
  reference/copy-on-write behavior, object/resource values, and native lowering
- `array_intersect` non-scalar value comparisons, exact native `TypeError`
  objects, PHP warning-and-string-conversion behavior for arrays and objects,
  reference/copy-on-write behavior, object/resource values, and native lowering
- `array_unique` sort flags outside `SORT_REGULAR`/`SORT_NUMERIC`/
  `SORT_STRING`, non-scalar value comparisons, numeric-mode PHP warning
  recovery for non-numeric values, exact native `TypeError` objects, PHP
  warning-and-string-conversion behavior for arrays and objects,
  reference/copy-on-write behavior, object/resource values, and native
  lowering
- `array_flip` warning-and-skip behavior for unsupported source values,
  reference/copy-on-write behavior, exact native warning/`TypeError` objects,
  resource values, and native lowering
- `array_change_key_case` Unicode/locale-aware casing, non-int case coercions,
  reference/copy-on-write behavior, exact native warning/`TypeError` objects,
  resource keys, and native lowering
- `array_column` first-argument coercions outside arrays, column or index keys
  outside int/string/null, lossy or non-finite float index values,
  array/object/resource index values, magic `__get`, `ArrayAccess`, exact
  visibility-context behavior for non-public properties,
  reference/copy-on-write behavior, exact native `TypeError`/warning objects,
  resource values, and native lowering
- `array_fill_keys` lossy or non-finite float stringification,
  warning-and-skip behavior for unsupported key values, array/object/resource
  key values, reference/copy-on-write behavior, object handle identity for
  object fill values, exact native warning/`TypeError` objects, resource
  values, and native lowering
- `array_count_values` warning-and-skip behavior for unsupported values,
  reference/copy-on-write behavior, exact native warning/`TypeError` objects,
  resource values, and native lowering
- `array_sum` PHP warning recovery for non-numeric strings and unsupported
  value types, object/resource values, reference/copy-on-write behavior, exact
  native `TypeError` objects, and native lowering
- `array_product` PHP warning recovery for non-numeric strings and unsupported
  value types, object/resource values, reference/copy-on-write behavior, exact
  native `TypeError` objects, and native lowering
- `array_reduce` array/object callables, closures, first-class callables,
  method calls, reference/copy-on-write behavior, object handle
  identity preservation, resource values, exact native `TypeError` objects, and
  native lowering
- `array_filter` callbacks outside `null` and string-valued
  user-function/callable-builtin names, integer mode flags outside `0`, `1`,
  and `2`, non-finite or non-integral float mode values, string mode coercions
  outside the current trimmed integral numeric string subset, lossy mode
  coercions such as `2.5` and `"2.5"`,
  reference/copy-on-write behavior, object handle identity
  preservation, resource values, exact native `TypeError` objects, and native
  lowering
- `array_map` array/object callables, closures, first-class callables, method
  calls, reference/copy-on-write behavior, object handle identity preservation,
  resource values, exact native `TypeError` objects, and native lowering
- `method_exists` method dispatch beyond current declared/inherited lookup,
  traits, interfaces, aliases/imports, namespace-aware names, autoloading, visibility behavior
  beyond metadata reporting, exact native `TypeError` objects, object operands,
  and native lowering beyond direct string/string false folding
- `get_class_methods` inheritance beyond current single-parent chain, traits,
  interfaces, aliases/imports, namespace-aware names, autoloading, non-public/context-sensitive visibility
  listing, exact native ordering and `TypeError` behavior, and native lowering
- `get_class_vars` property defaults, inheritance, traits, interfaces,
  aliases/imports, namespace-aware names, autoloading,
  non-public/context-sensitive visibility listing, exact native ordering and
  `TypeError` behavior, and native lowering
- `get_object_vars` dynamic properties, visibility context for non-public
  properties, traits, interfaces, aliases/imports,
  namespace-aware names, references/copy-on-write, exact native ordering and
  `TypeError` behavior, and native lowering
- `get_mangled_object_vars` dynamic properties, property defaults, traits,
  interfaces, aliases/imports, namespace-aware names,
  non-public/context-sensitive visibility behavior beyond the current
  declaring-class slot ownership, references/copy-on-write, exact native
  ordering and `TypeError` behavior, and native lowering
- `property_exists` native true results, native declared property tables,
  object operands, built-in/internal/extension classes, autoloading,
  namespaces/import aliases, exact native `TypeError` behavior, and native
  lowering beyond direct string/string false folding
- `empty($object->name)` dynamic property names, non-public visibility
  context, complex lvalues, magic `__isset`/`__get` behavior,
  references/copy-on-write, exact native error behavior, and native lowering
- `unset($object->name)` property uninitialization, typed/uninitialized
  property behavior, dynamic property names, non-public visibility context,
  magic `__unset` behavior, references/copy-on-write, exact native error
  behavior, and native lowering
- `is_a` inheritance beyond current single-parent class chain, interfaces,
  traits, aliases/imports, namespace-aware names, autoloading, exact native `TypeError` behavior, object handle
  identity beyond current class ids, object operands, and native lowering
  beyond direct string/string false folding
- `is_subclass_of` inheritance beyond current single-parent class chain,
  interfaces, traits, aliases/imports, namespace-aware names, autoloading, exact native `TypeError` behavior, object
  operands, and native lowering beyond direct string/string false folding
- `get_parent_class` inheritance lookup beyond immediate declared parents,
  interfaces, aliases/imports, namespace-aware names, autoloading, default `$this` behavior, exact native
  `TypeError` behavior, and native lowering
- `get_called_class` method/static class context, late static binding,
  inheritance, aliases/imports, namespace-aware names, exact native `Error`
  behavior, and native lowering
- `spl_object_id` handle reuse after destruction, clone semantics, destructors,
  references/copy-on-write behavior, exact native `TypeError` behavior, and
  native lowering
- `spl_object_hash` exact system PHP hash formatting, handle reuse after
  destruction, clone semantics, destructors, references/copy-on-write behavior,
  exact native `TypeError` behavior, and native lowering
- `class_exists` native true results, native declared class tables,
  built-in/internal/extension class entries, autoloading, namespaces/import
  aliases, exact native `TypeError` behavior, and native lowering beyond
  direct string-name false folding
- `interface_exists` declared interface metadata, built-in/internal interface
  entries, autoloading, namespaces/import aliases, exact native `TypeError`
  behavior, and native lowering beyond direct string-name false folding
- `trait_exists` declared trait metadata, built-in/internal trait entries,
  autoloading, namespaces/import aliases, exact native `TypeError` behavior,
  and native lowering beyond direct string-name false folding
- `enum_exists` declared enum metadata, built-in/internal enum entries,
  autoloading, namespaces/import aliases, exact native `TypeError` behavior,
  and native lowering beyond direct string-name false folding
- `get_declared_interfaces` declared interface metadata, built-in/internal
  interface entries, autoloading, namespaces/import aliases, exact native
  ordering, and native lowering
- `get_declared_traits` declared trait metadata, built-in/internal trait
  entries, autoloading, namespaces/import aliases, exact native ordering, and
  native lowering
- named arguments
- `declare(strict_types=1)` and PHP type declaration enforcement
- bare global constant resolution outside exact uppercase
  `ARRAY_FILTER_USE_KEY`, `ARRAY_FILTER_USE_BOTH`, `SORT_REGULAR`,
  `SORT_NUMERIC`, `SORT_STRING`, and runtime-defined unqualified constants in
  the current name/value subset;
  unsupported
  `define(...)` names or values, case-insensitive legacy constants, extension
  constants, namespace-qualified constants, nested `const` declarations,
  dynamic declaration values, `constant()`/`defined()` lookup
  for class constants, names lexed as language keywords or literals for bare
  reads, magic constants other than `__LINE__`, `__FILE__`, `__DIR__`, and
  `__FUNCTION__`,
  reference/copy-on-write behavior for constant values, and native lowering
  remain unsupported
- namespace-aware name resolution, imports, aliases, grouped imports, and
  executable qualified/fully qualified function or class references
- closures and arrow functions
- configurable recursion/call-stack limits matching PHP deployments
- exception objects and exception handling beyond the current parse boundary
- traits/interfaces/enums
- generator functions, generator objects, `yield`, `yield from`, key/value
  yields, by-reference yields, `send`/`throw`/`return` generator semantics,
  and native lowering
- executable attribute declarations and reflection metadata beyond the current
  lex boundary
- `is_callable` callable-name output parameter, array/object callable dynamic
  invocation, object `__invoke` callables, private/protected caller-context
  method callability, inherited/trait/interface method lookup, first-class
  callable syntax, namespace/autoload-aware resolution, exact native
  `TypeError` behavior, and native lowering beyond direct known string
  builtin/missing-name folding with optional known boolean syntax-only flags
  and direct non-string scalar/null false folding
- `function_exists` non-string name coercion, namespace/autoload-aware lookup,
  extension-loaded functions beyond documented builtins, exact native
  `TypeError`/deprecation behavior, and native lowering beyond direct known
  string builtin/missing-name folding
- PHP standard library beyond documented builtins
- `empty(...)` operands outside direct variables, direct array offsets, and
  direct public object-property operands, including nested offsets, dynamic
  property names, non-public property visibility context, append offsets,
  complex lvalues, magic methods, and general expressions
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
