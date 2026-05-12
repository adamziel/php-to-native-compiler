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
  constant-expression subset, including bare references to previously defined
  unqualified constants and the current built-in global constant slice when an
  omitted argument is bound
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
- exact uppercase built-in global constants `ARRAY_FILTER_USE_KEY` and
  `ARRAY_FILTER_USE_BOTH`, which evaluate to integers `2` and `1`
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
  current built-in `ARRAY_FILTER_*` constants
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
  offset operands, and direct object-variable public-property operands over the
  current value model; undefined variables, missing array keys, missing public
  properties, non-array/non-object targets, null variables, null array values,
  and null public property values evaluate the fallback, while falsey non-null
  values such as `false`, `0`, `""`, and `"0"` are returned without evaluating
  the fallback
- null coalescing assignment `$name ??= expr` and `$array[$key] ??= expr` for
  direct static variables and direct array-variable offset operands; undefined
  and `null` variables, undefined/null arrays, missing array keys, and null
  array values evaluate and store the right-hand expression, while existing
  non-null values are preserved without evaluating the right-hand expression
- builtins for the documented subset: `strlen`, `isset`, `empty`, `count`,
  `define`, `constant`, `defined`,
  `array_key_exists`, `array_key_first`, `array_key_last`, `array_is_list`,
  `array_values`, `array_keys`, `array_reverse`, `array_slice`, `array_chunk`,
  `array_pad`, `array_merge`, `array_replace`, `array_combine`,
  `array_intersect_key`,
  `array_diff_key`, `array_diff`, `array_intersect`, `array_unique`,
  `array_flip`, `array_fill_keys`, `array_count_values`, `array_sum`,
  `array_product`, `array_reduce`, `array_filter`, `array_map`, `in_array`,
  `array_search`, `get_class`, `is_object`, `get_debug_type`,
  `class_exists`, `interface_exists`, `trait_exists`, `enum_exists`,
  `property_exists`, `method_exists`, `is_a`, `get_class_methods`, `get_class_vars`,
  `get_object_vars`, `get_mangled_object_vars`, `is_subclass_of`, `get_parent_class`,
  `get_declared_classes`, `get_declared_interfaces`, `get_declared_traits`,
  `spl_object_id`, `spl_object_hash`, `var_dump`, and `print_r`;
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
  case-sensitive declared property metadata for current object values or
  string class names, `method_exists` checks case-insensitive declared method
  metadata for current object values or string class names, `get_class_methods`
  returns public declared method names in declaration order for current object
  values or declared string class names, `get_class_vars` returns public
  declared property names in declaration order with `null` values for declared
  string class names, `get_object_vars` returns public instance property names
  with their current values in declaration order for current object values,
  `get_mangled_object_vars` currently returns the same public instance
  property slice for current object values,
  `is_a` checks
  exact class identity over current object values or string class names when
  `allow_string` is true, `is_subclass_of` returns false for the current
  no-inheritance metadata model after validating the supported object/string
  and class-name argument boundary, `get_parent_class` returns false for
  supported object/declared-string inputs because parent metadata is not
  represented, `get_declared_classes` returns a zero-indexed array of classes
  declared in the current program in declaration order,
  `get_declared_interfaces` returns an empty zero-indexed array because
  interface declarations and internal interface metadata are not represented,
  `get_declared_traits` returns an empty zero-indexed array because trait
  declarations and internal trait metadata are not represented,
  and `print_r` can render current minimal object values
- structured runtime errors for undefined variables, arity mismatches,
  unsupported calls, division by zero, non-numeric string arithmetic, and
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
  `array_combine` length mismatches, unsupported non-int/string
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
  non-int/string `array_fill_keys` key values, non-array
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
  comparisons, duplicate constants, undefined constants, unsupported
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
  arrow functions, named arguments, and `declare(strict_types=1)`
- explicit parse diagnostics for unsupported magic constants such as
  `__CLASS__`, `__TRAIT__`, `__METHOD__`, and `__NAMESPACE__`
- explicit parse diagnostics for unsupported include/require syntax:
  `include`, `include_once`, `require`, and `require_once`
- explicit parse diagnostics for unsupported direct `eval(...)` syntax
- explicit parse diagnostics for unsupported namespace and top-level `use`
  declaration syntax
- explicit parse diagnostics for unsupported namespace-qualified function and
  class names such as `App\fn()` and `new App\Box()`
- explicit parse diagnostics for unsupported nested, namespace-aware, or
  dynamic-value `const` declarations
- stable runtime diagnostics for unsupported bare global constants outside the
  current built-in/runtime-defined slice, such as `PHP_VERSION`
- explicit parse diagnostics for unsupported array spread/reference elements
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
  alternate colon/`endswitch` syntax
- explicit parse diagnostics for unsupported `break`/`continue` loop-depth
  arguments
- explicit parse diagnostics for unsupported exception syntax: `throw`,
  `try`, `catch`, and `finally`
- explicit parse diagnostics for unsupported PHP 8 `match` expressions
- explicit parse diagnostics for unsupported ternary conditional expressions
- explicit parse diagnostics for unsupported chained coalescing and
  non-variable null coalescing assignment forms
- explicit parse diagnostics for unsupported object/class syntax: nested class
  declarations, inheritance, interface declarations and implementation, trait
  declarations, trait use inside classes, enum declarations,
  `abstract`/`final`/`readonly` class
  modifiers, `abstract`/`final`/`readonly` class member modifiers,
  typed property declarations, property default values, multiple property
  declarations, class constant declarations,
  unsupported `$this` usage, unsupported `clone` expressions, unsupported
  `instanceof` expressions, unsupported `ClassName::class` expressions,
  unsupported magic static receivers such as `self::`, `parent::`, and
  `static::`, anonymous class expressions, method calls, dynamic property
  names, static property access, static method calls, and class constant access
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
- Null coalescing: `phpc run` supports an executable `??` slice where the left
  operand is a direct static variable, direct array-variable offset, or direct
  object-variable public property. The left operand uses PHP-style isset
  semantics for the current value model: undefined variables, missing array
  keys, missing public properties, null variables, null array values, null
  public property values, non-array array-offset targets, and non-object
  property targets use the fallback, while falsey non-null values are returned
  as-is and the fallback expression is not evaluated. `phpc run` also supports
  direct-variable `$name ??= expr` and direct array-offset `$array[$key] ??=
  expr` statements with lazy right-hand evaluation only when the variable or
  array slot is undefined, missing, or null. Direct array-offset `??=`
  materializes undefined/null target variables as arrays; existing non-array
  targets fail with the current stable invalid-array-access diagnostic.
  Complex or nested `??` left operands, append-offset/object-property `??=`
  targets, dynamic property names, non-public visibility context, magic
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
  program, accept only boolean autoload flags, and are available through
  string-valued dynamic function calls. The autoload flag does not trigger
  autoloading in the current subset.
  `interface_exists($name)` and `interface_exists($name, $autoload)` accept
  string interface names, return false for all supported calls because
  interface metadata is not represented yet, and are available through
  string-valued dynamic function calls. The autoload flag must be boolean and
  does not trigger autoloading.
  `trait_exists($name)` and `trait_exists($name, $autoload)` accept string
  trait names, return false for all supported calls because trait metadata is
  not represented yet, and are available through string-valued dynamic function
  calls. The autoload flag must be boolean and does not trigger autoloading.
  `enum_exists($name)` and `enum_exists($name, $autoload)` accept string enum
  names, return false for all supported calls because enum metadata is not
  represented yet, and are available through string-valued dynamic function
  calls. The autoload flag must be boolean and does not trigger autoloading.
  `property_exists($object_or_class, $property)` accepts a current object value
  or string class name and a string property name. It checks the current
  declared property metadata with case-sensitive property names, reports
  public/protected/private and static properties as existing, returns false for
  missing properties or missing string class names, and is available through
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
  an array of public declared properties in declaration order, including public
  static properties, with `null` values because property defaults are not
  implemented. It is available through string-valued dynamic function calls.
  `get_object_vars($object)` accepts current object values and returns an array
  of public instance property names in declaration order with their current
  slot values. Protected/private slots and static properties are not included.
  It is available through string-valued dynamic function calls.
  Direct `empty($object->name)` accepts direct object-variable public-property
  operands, returns true for falsey public property slots, missing properties,
  undefined target variables, and non-object target variables, and uses a
  stable unsupported-property diagnostic for non-public properties.
  `get_mangled_object_vars($object)` accepts current object values and returns
  the same public instance property slice in declaration order. Protected and
  private property-name mangling, dynamic properties, and visibility-context
  behavior are not represented yet. It is available through string-valued
  dynamic function calls.
  `is_a($object_or_class, $class_name)` accepts current object values and
  checks exact class identity against the current declared class metadata using
  case-insensitive class-name lookup. `is_a($object_or_class, $class_name,
  true)` also accepts a string first argument and checks whether both string
  names resolve to the same declared class. A false or omitted `allow_string`
  flag makes string first arguments return false. Missing source or target
  class names return false, and string-valued dynamic calls to `is_a` use the
  same path.
  `is_subclass_of($object_or_class, $class_name[, $allow_string])` accepts the
  current object/string first-argument subset and string class names, considers
  string first arguments only when `allow_string` is true, returns false for
  exact-class, missing-class, and no-parent cases because inheritance metadata
  is not represented yet, and is available through string-valued dynamic calls.
  `get_parent_class($object_or_class)` accepts current object values or
  declared string class names, returns false for all supported inputs because
  parent metadata is not represented yet, and is available through
  string-valued dynamic calls.
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
  class-name constant resolution exists. Magic static receivers such as
  `self::`, `parent::`, and `static::` fail with a stable parse diagnostic
  before class-context, parent-class, or late-static-binding resolution exists.
  Method dispatch, dynamic property names, `$this` object context binding,
  visibility enforcement for non-public properties, static storage, class
  constants, object handle aliasing/identity, shallow/deep clone property
  copying, `__clone`, inheritance/interface relationship checks,
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
  $array)` checks existing integer/string keyed slots without filtering out
  `null` values and is also available through string-valued dynamic function
  calls. `array_key_first($array)` returns the first inserted integer or string
  key as an `int` or `string`, and `array_key_last($array)` returns the last
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
  insertion-order lockstep, uses integer key values directly as result keys,
  normalizes string key values through the current PHP-style decimal string key
  rules, stores cloned values from the second array, and overwrites duplicate
  result keys with later pairs without moving the first result-key position.
  It is also available through string-valued dynamic function calls.
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
  `array_unique($array)` compares current scalar values through their PHP
  string forms, keeps the first entry for each string form, preserves kept
  integer/string keys and insertion order, uses kept integer keys for later
  append behavior, and is also available through string-valued dynamic
  function calls.
  `array_flip($array)` accepts arrays, converts
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
  and `array_filter($array, null, $mode)` with integer mode flags `0`, `1`, or
  `2` accept arrays only, remove values that are falsey under the current
  PHP-shaped truthiness rules, preserve the original integer/string keys and
  insertion order of kept entries, and are available through string-valued
  dynamic function calls.
  `array_filter($array, $callback)` accepts callbacks that evaluate to string
  function names resolving to current user functions or callable builtins,
  invokes the callback once per value in insertion order with the value as the
  only argument, preserves keys whose callback result is truthy, accepts
  explicit integer mode flag `0` for the same value-only callback path, and is
  also available through string-valued dynamic calls to `array_filter`.
  `array_filter($array, $callback, 2)` invokes the same string-valued callback
  subset once per entry with the current integer or string key as the only
  argument, preserving keys whose callback result is truthy.
  `array_filter($array, $callback, 1)` invokes that callback subset once per
  entry with the value and then the current integer or string key as
  arguments, preserving keys whose callback result is truthy.
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
  `array_combine` operands, `array_combine` length mismatches, unsupported non-int/string
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
  non-int/string `array_fill_keys` key values, non-array
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
  non-string `class_exists` names, non-bool `class_exists` autoload flags,
  non-string `interface_exists` names, non-bool `interface_exists` autoload
  flags, non-string `trait_exists` names, non-bool `trait_exists` autoload
  flags, non-string `enum_exists` names, non-bool `enum_exists` autoload
  flags,
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
  exists, unsupported `spl_object_id($object)` calls before PHP object handle
  identity exists, non-object `spl_object_id` operands, unsupported
  `spl_object_hash($object)` calls before PHP object handle hash behavior
  exists, non-object `spl_object_hash` operands,
  object-to-string conversion, invalid `break`/`continue` outside a loop,
  unsupported `continue;` inside `switch`, and runaway user-function recursion.
- Native codegen: LLVM IR/assembly supports only straight-line echo/assignment
  with statically lowerable scalar expressions. `if`/`elseif`/`else`, `while`,
  arrays, array indexing, array assignment, variable unset, array offset unset,
  multiple-operand unset, `for`, `do ... while`, `switch`, `foreach`, `break`,
  `continue`, class declarations, object instantiation, object property reads,
  object property writes, global constants, top-level `const` declarations,
  `get_class(...)`, `is_object(...)`, `get_debug_type(...)`,
  `class_exists(...)`, `interface_exists(...)`, `trait_exists(...)`,
  `enum_exists(...)`, `property_exists(...)`,
  `method_exists(...)`,
  `get_class_methods(...)`, `get_class_vars(...)`, `is_a(...)`,
  `get_object_vars(...)`, `get_mangled_object_vars(...)`,
  `is_subclass_of(...)`, `get_parent_class(...)`,
  `get_declared_classes(...)`, `get_declared_interfaces(...)`,
  `get_declared_traits(...)`, `get_called_class(...)`,
  `spl_object_id(...)`, `spl_object_hash(...)`,
  `constant(...)`, `defined(...)`, and `define(...)` constant definitions are
  rejected with explicit codegen errors.
- Assembly emission: uses LLVM tools when available, with a temporary `cc -S`
  C fallback for the same narrow lowerable subset.
- Function calls: user-defined positional calls are supported in `phpc run`.
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
  `in_array`, `array_search`, `get_class`, `is_object`, `get_debug_type`,
  `class_exists`, `interface_exists`, `trait_exists`, `enum_exists`,
  `property_exists`, `method_exists`, `get_class_methods`, `get_class_vars`,
  `get_object_vars`, `get_mangled_object_vars`,
  `is_a`, `is_subclass_of`, `get_parent_class`, `get_declared_classes`,
  `get_declared_interfaces`, `get_declared_traits`, `get_called_class`,
  `spl_object_id`, `spl_object_hash`, `var_dump`, or `print_r`.
  The `define`, `constant`, and `defined` names resolve through the documented
  runtime constant path. Unresolved names fail with a stable undefined-function
  runtime error, and non-string callees fail with a stable unsupported-call
  runtime error. Required parameters and trailing default
  parameter values are supported. Defaults may
  use the current constant-expression subset: `null`, booleans, integers,
  floats, strings, short and long arrays with supported keys, unary
  expressions, binary expressions over those values, and bare references to
  unqualified constants that are defined in the current runtime constant table
  before the omitted argument is bound. The exact uppercase built-in
  `ARRAY_FILTER_USE_KEY` and `ARRAY_FILTER_USE_BOTH` constants are also
  accepted in default expressions. Omitted arguments bind to their defaults;
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
  implemented. Non-constant defaults such as variables, calls, dynamic calls,
  and indexed reads are rejected by the parser. Required parameters after default
  parameters are also rejected instead of modeling PHP's deprecation and
  implicit-required behavior. Variadic parameters and argument unpacking,
  reference parameters/returns, reference expressions, anonymous functions,
  arrow functions, named arguments, and `declare(strict_types=1)` are rejected
  with stable parse diagnostics. Parameter type declarations and return type
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
  `array_filter`, `array_map`, `in_array`, `array_search`, `get_class`,
  `is_object`, `get_debug_type`, `class_exists`, `interface_exists`,
  `trait_exists`, `enum_exists`, `property_exists`, `method_exists`,
  `get_class_methods`, `is_a`, `is_subclass_of`, `get_class_vars`,
  `get_object_vars`, `get_mangled_object_vars`, `get_parent_class`,
  `get_declared_classes`, `get_declared_interfaces`, `get_declared_traits`,
  `spl_object_id`, `spl_object_hash`, `var_dump`, and `print_r`
  cover the documented scalar/array/object subset. `get_called_class` is
  recognized only as the explicit unsupported method/static class context
  boundary described below. `spl_object_id` is recognized only as the explicit
  unsupported object-handle identity boundary described below.
  `spl_object_hash` is recognized only as the explicit unsupported
  object-handle hash boundary described below.
  `get_class($object)` returns the declared class name for current minimal
  object values and rejects non-object arguments. `is_object($value)` returns
  true only for current minimal object values and false for scalars and arrays.
  `get_debug_type($value)` returns current scalar/array type names and the
  declared class name for current minimal object values. `class_exists($name)`
  and `class_exists($name, $autoload)` accept string class names, return whether
  the current parsed program declared that class, and accept only boolean
  autoload flags without triggering autoloading.
  `interface_exists($name)` and `interface_exists($name, $autoload)` accept
  string interface names and return false for all supported calls because
  interface metadata is not represented yet; the autoload flag must be boolean
  and does not trigger autoloading.
  `trait_exists($name)` and `trait_exists($name, $autoload)` accept string
  trait names and return false for all supported calls because trait metadata
  is not represented yet; the autoload flag must be boolean and does not
  trigger autoloading.
  `enum_exists($name)` and `enum_exists($name, $autoload)` accept string enum
  names and return false for all supported calls because enum metadata is not
  represented yet; the autoload flag must be boolean and does not trigger
  autoloading.
  `property_exists($object_or_class, $property)` checks declared property
  metadata for current object values or string class names with case-sensitive
  property names. `method_exists($object_or_class, $method)` checks declared
  method metadata for current object values or string class names with
  case-insensitive method names. `get_class_methods($object_or_class)` returns
  a zero-indexed array of public declared method names for current object
  values or declared string class names. `get_class_vars($class_name)` returns
  public declared property names with `null` values for declared string class
  names. `get_object_vars($object)` returns public instance property names
  with their current values for current object values.
  `get_mangled_object_vars($object)` currently returns that same public
  instance property slice for current object values. `empty($object->name)`
  checks falsey public slots and treats missing properties, undefined target
  variables, and non-object target variables as empty in the current
  direct-object-variable subset.
  `is_a($object_or_class, $class_name[, $allow_string])` checks exact class
  identity over current object values, and over string class names only when
  `allow_string` is true.
  `is_subclass_of($object_or_class, $class_name[, $allow_string])` validates
  current object/string relationship-check arguments and returns false for the
  current no-inheritance metadata model.
  `get_parent_class($object_or_class)` accepts current object values or
  declared string class names and returns false because parent class metadata
  is not represented.
  `get_declared_classes()` returns a zero-indexed array containing only the
  current parsed program's declared class names in declaration order.
  `get_declared_interfaces()` returns an empty zero-indexed array because
  interface declarations and internal interface metadata are not represented.
  `get_declared_traits()` returns an empty zero-indexed array because trait
  declarations and internal trait metadata are not represented.
  `get_called_class()` is recognized as a zero-argument callable boundary, but
  direct and string-valued dynamic calls fail with a stable unsupported-call
  diagnostic until method/static class context exists.
  `spl_object_id($object)` is recognized as a one-argument callable boundary,
  but object arguments fail with a stable unsupported-call diagnostic until PHP
  object handle identity exists; non-object arguments fail with a stable
  type-boundary diagnostic.
  `spl_object_hash($object)` is recognized as a one-argument callable boundary,
  but object arguments fail with a stable unsupported-call diagnostic until PHP
  object handle hash behavior is modeled on top of object identity; non-object
  arguments fail with a stable type-boundary diagnostic.
  `print_r` can also render the current minimal object values. `strlen` remains
  scalar-only and rejects arrays and objects. `count` accepts arrays only.
  `array_key_exists($key, $array)` accepts integer
  and string keys over the current ordered array value model, returns true for
  existing keys even when the stored value is `null`, returns false for missing
  keys, rejects non-array second arguments, and rejects unsupported key values
  such as booleans, `null`, floats, objects, and future resources instead of
  applying PHP's full key coercions. `array_key_first($array)` and
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
  dynamic function calls. Non-array operands, including variadic replacement
  operands, fail with stable diagnostics. References, copy-on-write
  containers, object handle identity preservation for object values, resource
  values, exact native `TypeError` objects, and native lowering are not
  implemented.
  `array_combine($keys, $values)` accepts two array operands with equal entry
  counts, reads both arrays in insertion order, converts integer and string
  values from the first array into result keys using the current key
  normalization rules, and stores cloned values from the second array.
  Duplicate result keys are overwritten by later pairs without moving the first
  result-key position. Empty key/value arrays return an empty array. Non-array
  operands, length mismatches, and unsupported key values fail with stable
  project diagnostics. Bool, null, float, array, object, future resource, and
  reference key-value coercions, exact native `ValueError`/`TypeError` objects,
  references, copy-on-write containers, object handle identity preservation for
  object values, resource values, and native lowering are not implemented.
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
  `array_unique($array)` accepts one array operand, compares current scalar
  values by their PHP string forms, and returns a new ordered array containing
  the first entry for each distinct string form. Kept entries preserve their
  original integer/string keys and insertion order, dropped duplicate entries
  do not affect later append behavior, and the source array is not mutated.
  Non-array operands, non-scalar values such as arrays or objects, and the
  optional sort-flags argument fail with stable project diagnostics.
  References, copy-on-write containers, object/resource values, exact native
  `TypeError` objects, PHP warning-and-string-conversion behavior for arrays
  and objects, sort modes other than the current default string-form behavior,
  and native lowering are not implemented. `array_unique` is also available
  through string-valued dynamic function calls.
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
  and `array_filter($array, null, $mode)` with integer mode flags `0`, `1`, or
  `2` accept arrays only, remove `null`, `false`, zero integers and floats,
  empty strings, string `"0"`, and empty arrays using the current
  `Value::is_truthy` rules, preserve the original integer/string keys and
  insertion order of kept entries, and are available through string-valued
  dynamic function calls.
  `array_filter($array, $callback)` accepts callback expressions that evaluate
  to string function names resolving to current user functions or callable
  builtins, invokes the callback with the value only, keeps entries whose
  callback result is truthy, preserves original keys and insertion order,
  accepts explicit integer mode flag `0` for the same value-only callback path,
  and is available when `array_filter` itself is called through a string-valued
  dynamic function name. `array_filter($array, $callback, 2)` invokes that
  same string-valued callback subset with each entry's current integer or
  string key as the only argument and preserves original keys for entries
  whose callback result is truthy. `array_filter($array, $callback, 1)`
  invokes the same string-valued callback subset with the value and then the
  current integer or string key as arguments, preserving original keys for
  entries whose callback result is truthy. Non-string non-null callback values
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
  runtime-defined table or from the exact built-in `ARRAY_FILTER_*` slice.
  `defined($name)` returns true for supported unqualified names present in that
  current table and false for supported unqualified names that are missing.
  Top-level single and grouped `const NAME = value;` declarations accept
  unqualified names and the current constant-expression subset (`null`,
  booleans, integers, floats, strings, arrays, unary expressions, and binary
  expressions over those values, plus bare references to previously defined
  unqualified constants and the current exact built-in `ARRAY_FILTER_*`
  constants). Grouped declarations execute left to right, so references to
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
  `constant(...)`/`defined(...)`, references/copy-on-write behavior, and native
  lowering are not implemented.
  Array/object callables, closures, first-class callables, method calls,
  integer mode flags outside `0`, `1`, and `2`, non-int mode coercions such as
  `false`, references,
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
  and direct public object-property
  operands such as `isset($object->name)`; it can safely check undefined
  variables, missing/null array slots, undefined array variables, non-array
  array targets, and undefined object-property targets. Nested array offsets,
  append offset operands, dynamic property names, non-public property operands,
  complex lvalues, and general expression operands remain unsupported. `empty`
  supports one direct variable operand, one direct array offset operand such
  as `empty($array[$key])`, or one direct public object-property operand such
  as `empty($object->name)`; undefined variables, missing array keys,
  undefined array targets, non-array array targets, missing object properties,
  undefined object targets, and non-object property targets are treated as
  empty, and existing values use the current PHP truthiness rules. Nested array
  offsets, dynamic property names, non-public property visibility context,
  append offset operands, complex lvalues, general expression operands, magic
  methods, and unsupported array-key coercions remain unsupported.
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
  `array_combine` key coercions beyond integer/string values,
  `array_combine` object/resource key values, `array_intersect_key` and
  `array_diff_key` exact native `TypeError` objects and
  reference/copy-on-write behavior, `array_diff` and `array_intersect`
  non-scalar value comparison behavior, `array_unique` sort flags,
  `array_unique` non-scalar value comparison behavior, exact native
  `TypeError` objects, and native lowering, `array_flip`
  warning-and-skip behavior
  for unsupported source values, and `array_fill_keys` warning-and-skip
  behavior for unsupported key values, `array_count_values` warning-and-skip
  behavior for unsupported values, `array_sum` PHP warning recovery for
  unsupported values, `array_product` PHP warning recovery for unsupported
  values, `array_reduce` callback forms outside the current
  string function-name subset, and `array_filter` callback forms outside the
  current null-callback, value-only string function-name, key-only string
  function-name, and value/key string function-name modes, and `array_map`
  callback forms outside current null-callback and string-valued function-name
  forms are not implemented.
  Because `isset` and `empty` are modeled as special static forms, they are not
  available through dynamic function lookup. PHP's complete warning behavior is
  not implemented.
- Object/class gaps: nested and conditional class declarations, method calls,
  `$this`, constructor execution, constructor arguments, inheritance,
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
  multi-property declarations, class constant declarations, constants, `$this`
  object context binding, static property storage, late static binding, magic
  methods, namespaces, autoloading, anonymous classes, attributes, reflection,
  dynamic properties, dynamic property names, non-public property access,
  static member execution
  through `::`, `::class` class-name constant resolution, property assignment
  targets other than a direct variable, dynamic properties created outside
  declarations, autoload side effects from property introspection,
  object handle identity/aliasing,
  cloning, destructors, serialization hooks, visibility enforcement,
  `self`/`parent`/`static`, object comparisons, `instanceof` relationship
  checks, object-to-string conversion, object callables, and native lowering
  are unsupported.
- Constructor boundary: declaring a class with `__construct` or supplying
  arguments to `new ClassName(...)` fails with stable runtime diagnostics before
  any user constructor body executes. `$this` binding, constructor property
  initialization, visibility checks, inheritance and parent constructor calls,
  promoted properties, exact PHP `Error` object behavior, and native lowering
  remain unsupported.
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
  currently fail with stable runtime diagnostics when a declared constructor is
  present or arguments are supplied
- unsupported class forms including nested/conditional declarations,
  inheritance, interface declarations and `implements` clauses, interface
  constants, interface method signatures, interface inheritance, trait
  declarations, enum declarations, enum cases/backing values/methods/interface
  implementation,
  typed property storage/enforcement, property defaults, multiple properties in
  one declaration, per-property defaults in multi-property declarations,
  class constant declarations, constants, `$this` object context binding, and
  anonymous classes
- static property access, static method calls, class constant access,
  class-name constant access, and magic static receivers such as `self::`,
  `parent::`, and `static::` through `::`
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
  diagnostic because method dispatch and method-context tracking are not
  implemented, and `__CLASS__` specifically fails because class-context
  tracking is not implemented. `__TRAIT__` specifically fails because trait
  declarations, trait use, and trait-context tracking are not implemented,
  and `__NAMESPACE__` specifically fails because namespace-aware name
  resolution is not implemented.
  `__FUNCTION__` is limited to user-function context and the top-level empty
  string behavior; closure context is not implemented. `__FILE__` currently
  reports the `phpc run` input path string, and `__DIR__` derives from that
  same path string; neither is guaranteed to match PHP's canonical absolute
  filename or directory in all entry paths.
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
- exception execution; `throw`, throw expressions, `try` blocks, `catch`
  clauses, and `finally` blocks currently fail with stable parse diagnostics
  before `Throwable`, `Exception`, custom exception classes, stack unwinding,
  `finally` execution, stack traces, exact native error objects, or native
  lowering exist
- PHP 8 `match` expressions currently fail with a stable parse diagnostic
  before expression-form branching exists. Strict arm matching, default arms,
  exhaustiveness errors, thrown expressions inside arms, value evaluation
  order, exact native error objects, and native lowering are not implemented.
- Ternary conditional expressions currently fail with a stable parse diagnostic
  before expression-form branching exists. Both full ternary
  `$condition ? $if_true : $if_false` and short ternary `$value ?: $fallback`
  forms are rejected. Condition truthiness, short-ternary value reuse,
  nesting/precedence, thrown expressions inside arms, exact native error
  objects, and native lowering are not implemented.
- Null coalescing is limited to direct static variables, direct array-variable
  offsets, and direct object-variable public properties on the left side, plus
  direct-variable `$name ??= expr` and direct array-offset `$array[$key] ??=
  expr` statements. Complex or nested `??` left operands, append-offset and
  object-property `??=` targets, dynamic property names, non-public visibility
  context, magic methods, unparenthesized chained coalescing, precedence
  interactions beyond the current single-operator expression slice,
  references/copy-on-write, exact native error objects, and native lowering are
  not implemented.
- dynamic callables outside the string function-name subset, including array
  callables, object/method callables, first-class callable syntax,
  `call_user_func`, and namespace/autoload-aware callable resolution
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
- `array_combine` key-value coercions beyond integers and strings, length
  mismatch native `ValueError` objects, non-array native `TypeError` objects,
  reference/copy-on-write behavior, object handle identity preservation for
  object values, resource values, and native lowering
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
- `array_unique` sort flags, non-scalar value comparisons, exact native
  `TypeError` objects, PHP warning-and-string-conversion behavior for arrays
  and objects, reference/copy-on-write behavior, object/resource values, and
  native lowering
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
  and `2`, non-int mode coercions such as `false`,
  reference/copy-on-write behavior, object handle identity
  preservation, resource values, exact native `TypeError` objects, and native
  lowering
- `array_map` array/object callables, closures, first-class callables, method
  calls, reference/copy-on-write behavior, object handle identity preservation,
  resource values, exact native `TypeError` objects, and native lowering
- `method_exists` method dispatch, inheritance, traits, interfaces,
  aliases/imports, namespace-aware names, autoloading, visibility behavior
  beyond metadata reporting, exact native `TypeError` objects, and native
  lowering
- `get_class_methods` inheritance, traits, interfaces, aliases/imports,
  namespace-aware names, autoloading, non-public/context-sensitive visibility
  listing, exact native ordering and `TypeError` behavior, and native lowering
- `get_class_vars` property defaults, inheritance, traits, interfaces,
  aliases/imports, namespace-aware names, autoloading,
  non-public/context-sensitive visibility listing, exact native ordering and
  `TypeError` behavior, and native lowering
- `get_object_vars` dynamic properties, visibility context for non-public
  properties, inheritance, traits, interfaces, aliases/imports,
  namespace-aware names, references/copy-on-write, exact native ordering and
  `TypeError` behavior, and native lowering
- `get_mangled_object_vars` protected/private property-name mangling, dynamic
  properties, non-public visibility context, inheritance, traits, interfaces,
  aliases/imports, namespace-aware names, references/copy-on-write, exact
  native ordering and `TypeError` behavior, and native lowering
- `empty($object->name)` dynamic property names, non-public visibility
  context, complex lvalues, magic `__isset`/`__get` behavior,
  references/copy-on-write, exact native error behavior, and native lowering
- `unset($object->name)` property uninitialization, typed/uninitialized
  property behavior, dynamic property names, non-public visibility context,
  magic `__unset` behavior, references/copy-on-write, exact native error
  behavior, and native lowering
- `is_a` inheritance, interfaces, traits, aliases/imports, namespace-aware
  names, autoloading, exact native `TypeError` behavior, object handle
  identity beyond current class ids, and native lowering
- `is_subclass_of` inheritance, interfaces, traits, aliases/imports,
  namespace-aware names, autoloading, exact native `TypeError` behavior, and
  native lowering
- `get_parent_class` inheritance lookup, interfaces, aliases/imports,
  namespace-aware names, autoloading, default `$this` behavior, exact native
  `TypeError` behavior, and native lowering
- `get_called_class` method/static class context, late static binding,
  inheritance, aliases/imports, namespace-aware names, exact native `Error`
  behavior, and native lowering
- `spl_object_id` object handle identity, handle reuse after destruction,
  clone semantics, destructors, references/copy-on-write behavior, exact native
  `TypeError` behavior, and native lowering
- `spl_object_hash` object handle hash formatting, object handle identity,
  handle reuse after destruction, clone semantics, destructors,
  references/copy-on-write behavior, exact native `TypeError` behavior, and
  native lowering
- `interface_exists` declared interface metadata, built-in/internal interface
  entries, autoloading, namespaces/import aliases, exact native `TypeError`
  behavior, and native lowering
- `trait_exists` declared trait metadata, built-in/internal trait entries,
  autoloading, namespaces/import aliases, exact native `TypeError` behavior,
  and native lowering
- `enum_exists` declared enum metadata, built-in/internal enum entries,
  autoloading, namespaces/import aliases, exact native `TypeError` behavior,
  and native lowering
- `get_declared_interfaces` declared interface metadata, built-in/internal
  interface entries, autoloading, namespaces/import aliases, exact native
  ordering, and native lowering
- `get_declared_traits` declared trait metadata, built-in/internal trait
  entries, autoloading, namespaces/import aliases, exact native ordering, and
  native lowering
- named arguments
- `declare(strict_types=1)` and PHP type declaration enforcement
- bare global constant resolution outside exact uppercase
  `ARRAY_FILTER_USE_KEY`, `ARRAY_FILTER_USE_BOTH`, and runtime-defined
  unqualified constants in the current name/value subset; unsupported
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
- generators
- attributes
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
