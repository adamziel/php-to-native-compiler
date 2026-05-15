# Support Matrix

## Supported in `phpc run`

- PHP opening tag `<?php`; text between `?>` and the next PHP open tag is
  treated as inline HTML output through the interpreter path, including the
  current PHP-compatible single-newline consumption immediately after `?>`
- `echo` statements with one or more comma-separated expressions
- `print` statements
- decimal and hexadecimal integer literals in the current signed 64-bit subset
- float literals
- single-quoted and double-quoted string literals with basic escapes; double
  quoted strings additionally support simple `$name` and `{$name}`
  interpolation over the current variable table
- `null`, `true`, and `false`
- magic constants `__LINE__`, evaluated from the expression token's source
  line, `__FILE__`, evaluated from the current `phpc run` input path when one
  is available, `__DIR__`, evaluated as that path's parent directory, and
  `__FUNCTION__`, evaluated as the current user-function name or an empty
  string outside a function. `__CLASS__` evaluates to the current class name in
  method context and to an empty string outside class context. `__METHOD__`
  evaluates to `Class::method` in the current method context, to the current
  function name in function context, and to an empty string outside a function.
- static variables backed by per-scope materialized symbol tables
- direct variable removal: `unset($name)` removes static variables from the
  current scope and treats undefined names as no-ops; `unset(...)` may include
  multiple supported operands and executes them left to right
- by-reference function and method return declarations such as
  `function &identity(...)` and `public function &make(...)` parse as runtime
  boundaries. Guarded or declaration-contained declarations can be loaded, but
  if execution invokes one of them, `phpc run` reports `unsupported call
  <name>(): reference returns are not implemented`. This does not create PHP
  reference-return binding, alias cells, or copy-on-write containers.
- by-reference function and method parameters may be declared. Calls that omit
  an optional by-reference parameter use that parameter's default value in the
  callee local scope without creating an alias. Calls that provide a
  by-reference parameter are supported only for direct variable arguments: the
  caller's current value is copied into the callee local parameter and the
  callee's final parameter value is copied back to the caller variable after
  execution. This is a bounded output-parameter path, not true alias binding
  during execution.
- by-reference assignment syntax `$alias =& $value;`,
  `$alias =& $array[$key];`, `$alias =& $object->method();`, and direct
  object-property array-offset targets such as `$object->items[$key] =& $value`
  parses in statement position for direct variable, direct array-offset, and
  method-call sources plus the documented direct object-property array-offset
  target shape. The executing subset is direct variable sources holding a
  current object or array value assigned into a direct variable; direct
  variable sources holding current object values may also be assigned into
  direct array offsets. These cases store the current value/object handle under
  the current value model. Scalar sources, array-offset and method-call
  sources, direct array-offset targets for array values, object-property array
  targets, source/target rebinding aliases, PHP reference containers,
  copy-on-write, mutation ordering beyond the stored value, and native lowering
  remain unsupported and report
  `unsupported call reference assignment: references and aliasing are not
  implemented` when reached.
- assignment statements, plus expression-position direct static-variable
  assignment `$name = expr` and direct array-offset assignment
  `$array[$key] = expr`, and direct public object-property assignment
  `$object->property = expr` with right-to-left chained assignment result
  values over the current value model. Direct append-offset assignment
  `$array[] = expr` is supported as a standalone assignment expression with an
  assignment result value. Direct array-offset
  assignment expressions evaluate the key before the right-hand expression and
  materialize undefined or `null` target variables as arrays. Nested
  direct-variable array-offset assignment expressions such as
  `$array[$outer][$inner] = expr` evaluate all index expressions left to right
  before the right-hand expression, materialize undefined, missing, or `null`
  array containers under the current no-reference/no-copy-on-write model, and
  return the assigned value. Direct append-offset assignment expressions
  evaluate the right-hand expression, append to direct array variables,
  materialize undefined or `null` target variables as arrays, and return the
  appended value. Append-at-depth assignment expressions such as
  `$array[$outer][] = expr` and `$array[$outer][$inner][] = expr` evaluate all
  path index expressions left to right before the right-hand expression,
  materialize undefined, missing, or `null` array containers under the current
  no-reference/no-copy-on-write model, append to the final nested array, and
  return the appended value. Direct object-property assignment expressions
  evaluate the right-hand expression, then write existing declared public
  property slots on direct object variables. Keyword-named direct properties
  such as `$object->public` and `$object->class` are accepted as property names
  after `->` for the current read/write subset, while keyword method calls
  remain unsupported. Nested object-property array
  assignment expressions such as `$object->items[$outer][$inner] = expr` and
  object-property append-at-depth expressions such as
  `$object->items[$outer][] = expr` evaluate all property path keys left to
  right, materialize `null` property values and missing/`null` intermediate
  containers as arrays, then write the property back through the existing
  visibility-aware object-property path. Dynamic property-name reads and direct
  assignment expressions such as `$object->$name`, `$object->{$expr}`,
  `$object->$name = expr`, and `$object->{$expr} = expr` are supported for
  direct object variables when the property-name expression evaluates to a
  string or integer name and resolves to an existing public slot; writes can
  also materialize public dynamic slots on `stdClass` objects and the
  WordPress `wpdb` compatibility class for reached table-name slots. Array literal
  reference elements such as `array(&$value)` and keyed values like
  `array('name' => &$value)` are parsed and evaluate the current value, but do
  not create PHP reference aliases yet. Reference assignment sources support
  direct variables, direct array offsets as a runtime boundary, method calls as
  a runtime boundary, and direct/dynamic object-property reads that copy current
  array/object values without aliasing. Nested compound
  assignment, nested `??=`, nested
  increment/decrement, mixed object/property/ArrayAccess
  targets, non-`stdClass` missing property materialization,
  references/copy-on-write, and native lowering remain unsupported. Simple
  positional statement-form array destructuring `list($a, $b) = expr;` is
  supported for direct variable targets when the right-hand side evaluates to a
  current ordered array. It reads numeric keys `0..n`, evaluates the right-hand
  side once before writes, assigns targets left to right, and assigns `null` for
  missing numeric offsets without emitting PHP's warning/notice yet.
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
- direct object-property array-offset compound assignment such as
  `$object->items[$outer][$inner] += expr` over an existing array-valued visible
  property and existing integer/string keyed nested entries. The target keys
  are evaluated left to right before the right-hand expression, the current
  left value is read before the right-hand expression, and the updated value is
  written back through the existing visibility-aware object-property path.
  Append forms, missing-key materialization, mixed object/property/ArrayAccess
  paths, references/copy-on-write, and native lowering remain unsupported.
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
  values (`null`, booleans, integers, floats, and strings), object handles,
  and ordered arrays with current integer/string keys and recursive strict
  value comparison over implemented values
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
- PHP error-control syntax `@expr` as a transparent runtime wrapper. The
  operand evaluates normally and existing runtime diagnostics are still
  reported; warning/notice/deprecation suppression is not implemented.
- `if` / `elseif` / `else`, including alternate
  `if (...) : ... elseif (...) : ... else: ... endif;` syntax
- `while`
- `for (initializer; condition; increment)` loops where each header slot is
  optional and comma-separated header lists execute left to right. Initializer
  and increment lists accept expressions and assignments from the current
  assignment subset, including direct static-variable compound assignment,
  direct array-offset compound assignment, and direct static-variable
  increment/decrement. Condition lists evaluate all expressions left to right
  and use the final expression's truthiness to decide whether the loop
  continues; an empty condition slot loops until control flow exits.
- `do ... while` loops with a block or single-statement body and a
  post-condition expression
- `switch ($value) { case ...: ... default: ... }` and alternate
  `switch ($value): case ...: ... default: ... endswitch;` statements over the
  current scalar loose-comparison subset, including `case`, `default`, `:` or
  `;` case/default separators, fallthrough, and `break;` to exit the switch
- `foreach ($array as $value)` and `foreach ($array as $key => $value)` over
  ordered arrays. By-reference value forms such as
  `foreach ($array as &$value)` and `foreach ($array as $key => &$value)` parse
  as runtime boundaries so guarded or declaration-contained code can be loaded,
  but reached loops report `unsupported call foreach: by-reference iteration is
  not implemented; only by-value iteration is supported`.
- `break;` for the innermost currently executing `while`, `for`,
  `do ... while`, `foreach`, or `switch`; `continue;` for the innermost
  currently executing loop
- function declarations with optional trailing commas in parameter lists,
  trailing variadic parameters such as `...$items` that collect extra
  positional arguments into a current ordered array, and syntax-only
  parameter/return type annotations for the current metadata slice. Top-level
  function declarations are registered before execution. Conditional or
  declaration-contained function declarations are registered only when
  execution reaches the declaration, so guarded forms such as
  `if (!function_exists("name")) { function name() {} }` work for the current
  braced statement-body subset. Skipped nested declarations remain absent, and
  repeated executed declarations report duplicate-function diagnostics.
  Unbraced nested declarations, full declaration timing edge cases,
  reference-return alias binding, and native lowering remain unsupported.
- positional function calls with optional trailing commas in argument lists
- dynamic function calls through string-valued expressions that resolve to the
  documented callable builtin subset or user-defined functions, with optional
  trailing commas in argument lists
- trailing default parameter values for user functions over the documented
  constant-expression subset, including bare references to previously defined
  unqualified constants, the current built-in global constant slice, and
  class-method `self::CONST` defaults resolved from the declaring class context
  when an omitted argument is bound
- recursive user-function calls up to a fixed 128-frame user-function call-depth
  guard
- `return`
- direct `exit()`/`die()` calls as a bounded termination construct, not a
  callable function. The current subset accepts no argument, an integer status
  argument, a string message argument, or `null`; string arguments append to
  stdout and terminate with exit code `0`, integer arguments terminate with the
  provided status when it fits in `i32`, and `null`/omitted arguments terminate
  with exit code `0`. `function_exists("exit")` and `is_callable("exit")`
  remain false. Exit inside complex expressions, shutdown functions,
  destructor/finally ordering, output buffering, exact status normalization,
  partial-output behavior beyond current stdout preservation, and native
  lowering remain unsupported.
- `throw expr;` statements parse and participate in normal statement
  reachability. Guarded/unreached throw statements can be skipped by existing
  control flow. If execution reaches a throw statement, `phpc run` reports the
  stable runtime boundary `unsupported call throw: exception objects and stack
  unwinding are not implemented` without evaluating the throw operand, because
  PHP exception objects and unwinding do not exist yet.
- `try { ... } catch (...) { ... } finally { ... }` blocks parse as statement
  syntax with class-name catch type lists and optional catch variables. Guarded
  or declaration-contained try blocks can be skipped by existing control flow.
  If execution reaches a try block, `phpc run` executes the try body on the
  normal no-throw path, skips catch bodies when no exception is thrown, and
  executes a finally body after normal try completion. Reached `throw`
  statements inside try bodies still report the current throw runtime boundary
  before catch matching, exception unwinding, or finally-during-exception
  execution exists.
- isolated local scopes for user-function calls; parameters and function-local
  assignments can shadow global names without mutating them
- top-level `global $name, ...;` declarations as no-op/import-compatible
  statements. Function-scope `global $name, ...;` imports direct variable names
  from the root global symbol table, materializes missing globals as `null`,
  routes direct reads and writes through the shared root slot, and treats
  `unset($name)` after import as removing the local import without deleting the
  root global value. Direct string-keyed `$GLOBALS['name']` reads and writes
  route to the same root global symbol table from top-level and function scope.
  Reference-backed alias objects, full PHP `$GLOBALS` array materialization,
  recursive `$GLOBALS` contents, non-string keyed `$GLOBALS` access, dynamic
  global names, exact warning/notice behavior, included-file scope
  interactions, copy-on-write, and native lowering remain unsupported.
- `$_SERVER` is seeded as a bounded root superglobal for `phpc run` with
  deterministic CLI request defaults for `SERVER_SOFTWARE`, `REQUEST_URI`,
  `HTTP_HOST`, `PHP_SELF`, `SCRIPT_NAME`, `SCRIPT_FILENAME`, and
  `QUERY_STRING`. Direct function-scope reads and writes of `$_SERVER` route
  through the root symbol
  table without a
  `global $_SERVER` declaration. Real SAPI request population, environment
  imports, complete server key catalogs, `$GLOBALS` aliasing, references,
  copy-on-write, mutation-ordering fidelity, `variables_order`, cookies,
  uploads, sessions, other superglobals, exact warning behavior, and native
  lowering remain unsupported.
- class declarations registered into the runtime metadata table:
  `class Name { ... }`, `abstract class Name { ... }`, `final class Name { ... }`,
  `readonly class Name { ... }`, and `class Child extends Parent { ... }` with
  single-parent metadata, property names, class constant names, method names,
  visibility, static flags, and abstract/final method flags for the documented
  subset, including abstract method signatures, final methods, compatible
  public/protected inherited property redeclarations sharing one runtime slot,
  and untyped static properties initialized from the current
  constant-expression default subset or `null`. Top-level classes are
  pre-registered before execution. Braced nested declarations in control-flow
  bodies, function/method bodies, switch cases, and included files register
  only when execution reaches the `class` statement; skipped branches do not
  define the class, and repeated reached declarations report a stable duplicate
  class runtime diagnostic.
- object instantiation with `new ClassName(...)` for declared classes. Classes
  without `__construct` are supported only with no constructor arguments.
  Declared or inherited public instance `__construct` methods execute with
  scoped `$this`, positional arguments, and the current default-parameter
  subset. Instantiating an abstract class reports a stable runtime boundary;
  abstract-method implementation enforcement, final inheritance/method
  enforcement, and readonly class semantics are not implemented. Magic
  class-name instantiation through `new self`, `new parent`, and `new static`
  is supported in active class/method contexts, including no-argument forms
  without parentheses, by resolving to the current, parent, or called class
  before using the same instantiation path. Contextless magic class
  instantiation reports a stable runtime boundary.
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
  class method/constructor context when the current class has a parent and the
  resolved parent-chain method is visible under the current rules. Static
  methods execute without `$this`; non-static methods reuse the current
  `$this` object. Positional arguments are evaluated left-to-right after
  metadata and arity checks, and the resolved method body runs with the
  declaring parent class as the active method context.
- explicit self method calls by static method name:
  `self::method(...)` is supported in active class method/constructor context
  when the resolved current-class or inherited method is visible under the
  current rules. Static methods execute without `$this`; non-static methods
  reuse the current `$this` object. Positional arguments are evaluated
  left-to-right after metadata and arity checks, and the resolved method body
  runs with the declaring class as the active method context.
- named static method calls by static method name:
  `ClassName::method(...)` is supported for declared or inherited static
  methods visible under the current public/protected/private rules. The call
  checks metadata and arity before evaluating positional arguments, executes
  without `$this`, and runs the method body with the declaring class as the
  active class context so current `self::class` and static-property access
  resolve lexically.
- dynamic static method calls by static method name:
  `$object::method(...)` and `$className::method(...)` evaluate the receiver
  expression, require an object or declared class-name string, resolve a
  declared or inherited visible static method from that receiver class, execute
  without `$this`, and preserve the receiver class as the called-class context
  for `static::` and `get_called_class()`.
- class constants declared as `const NAME = value;` or
  `public|protected|private const NAME = value;` with values from the current
  constant-expression subset. `ClassName::CONST`, `self::CONST`, and
  `parent::CONST` resolve declared or inherited constants case-sensitively,
  enforce public/protected/private visibility in the current class context,
  and return null, bool, int, float, string, or array values.
- static property reads and direct writes for untyped/no-default declared
  static properties through `ClassName::$name`, `self::$name`, and
  `parent::$name`, plus compound assignment, pre/post increment/decrement,
  `isset`, `empty`, `??`, `??=`, and stable diagnostics for PHP-forbidden
  `unset(...)` on those same static property forms. Storage is class-level,
  initialized to `null`, inherited static properties share the declaring class
  slot unless redeclared, names are case-sensitive, and current
  public/protected/private visibility checks apply in active class context.
- `isset($object->name)` for direct public instance property operands on direct
  object variables, plus private property operands owned by the active
  declaring class, protected property operands owned by the active class or an
  ancestor, and supported static property operands
- exact uppercase built-in global constants `CASE_LOWER`, `CASE_UPPER`,
  `ARRAY_FILTER_USE_KEY`, `ARRAY_FILTER_USE_BOTH`, `PREG_SPLIT_DELIM_CAPTURE`, `SORT_REGULAR`,
  `SORT_NUMERIC`, `SORT_STRING`, `PHP_VERSION_ID`, `PHP_VERSION`, and
  `PHP_INT_MAX`, and `PHP_SAPI`. The small integer constants evaluate to
  their documented PHP values (`CASE_LOWER` `0`, `CASE_UPPER` `1`,
  `ARRAY_FILTER_USE_KEY` `2`, `ARRAY_FILTER_USE_BOTH` `1`,
  `PREG_SPLIT_DELIM_CAPTURE` `2`, `SORT_REGULAR` `0`, `SORT_NUMERIC` `1`,
  and `SORT_STRING` `2`), and `PHP_VERSION_ID` evaluates to the current
  deterministic PHP 8.3 compatibility target `80300`; `PHP_VERSION` evaluates
  to the deterministic
  compatibility string `8.3.0`, `PHP_INT_MAX` evaluates to the
  host-independent 64-bit integer maximum, and `PHP_SAPI` evaluates to the
  current deterministic `cli` SAPI string.
- exact uppercase PHP error mask constants `E_ERROR`, `E_WARNING`, `E_PARSE`,
  `E_NOTICE`, `E_CORE_ERROR`, `E_CORE_WARNING`, `E_COMPILE_ERROR`,
  `E_COMPILE_WARNING`, `E_USER_ERROR`, `E_USER_WARNING`, `E_USER_NOTICE`,
  `E_STRICT`, `E_RECOVERABLE_ERROR`, `E_DEPRECATED`, `E_USER_DEPRECATED`, and
  `E_ALL` with the current PHP 8.3-compatible integer values used by
  `error_reporting()`
- runtime-defined constants through `define($name, $value)` over the current
  unqualified or qualified string-name and scalar/array value subset;
  `constant($name)` accepts unqualified names and qualified lookup names with
  an optional leading global namespace separator; `defined($name)` reports
  whether a supported unqualified or qualified name exists in the current
  built-in/runtime-defined constant table; simple double-quoted `$name`
  interpolation can build those runtime string names, and string-valued dynamic
  calls to `define`, `constant`, and `defined` use the same path. Runtime
  string lookup also accepts declared class-constant names in the form
  `ClassName::CONST` or `\ClassName::CONST`: `defined($name)` reports true
  only for declared public class constants, and `constant($name)` resolves the
  declared constant through the existing class-constant visibility checks.
- bare reads of runtime-defined unqualified constants over the same current
  name/value subset; array constant values are cloned on lookup
- top-level single and grouped `const NAME = value;` declarations for
  unqualified names at global scope and names resolved under the active
  unbracketed namespace. Values use the current constant-expression subset:
  `null`, booleans, integers, floats, strings, short and long arrays with
  supported keys, unary expressions, binary expressions over those values,
  and bare references to previously defined unqualified constants or the
  current built-in `CASE_*`, `ARRAY_FILTER_*`, `SORT_REGULAR`,
  `SORT_NUMERIC`, `SORT_STRING`, `PHP_VERSION_ID`, `PHP_VERSION`,
  `PHP_INT_MAX`, `PHP_SAPI`, and documented `E_*` error mask
  constants
- short array literals (`[]`, `[value]`, `[key => value]`) and long
  `array(...)` literals as an alias for that same array-literal subset
- ordered arrays with integer and string keys
- array indexed reads: `$array[$key]` for existing integer/string keyed array
  entries
- direct variable array writes: `$array[$key] = ...` and `$array[] = ...`
- direct and nested array offset removal: `unset($array[$key])` and
  `unset($array[$outer][$inner])` for direct array variables, plus nested
  object-property array offset removal such as
  `unset($object->items[$outer][$inner])` for direct object variables and named
  properties, over the current integer/string key subset. Direct and dynamic
  object-property removal such as `unset($object->property)` and
  `unset($object->$name)` are supported for direct object variables by writing
  `null` into the visible property slot in the current value model; multiple
  supported `unset(...)` operands execute left to right
- `foreach ($array as $value)` and `foreach ($array as $key => $value)`
  iteration in insertion order over a snapshot of the current array entries
- `isset($array[$key])` and nested direct-variable rooted array offset paths
  such as `isset($array[$outer][$inner])` over the current integer/string key
  subset
- direct object-property array-offset `isset(...)` paths such as
  `isset($object->items[$outer][$inner])` over visible array-valued properties
  and the current integer/string key subset. Missing/null/non-array path
  components return false; dynamic property paths, ArrayAccess, references,
  copy-on-write, and native lowering remain unsupported.
- `empty($name)`, `empty($array[$key])`, nested direct-variable array-offset
  paths such as `empty($array[$outer][$inner])`,
  `empty($object->publicProperty)`, direct object-property array-offset paths
  such as `empty($object->items[$outer][$inner])`, and supported static
  property operands for direct variables, direct array-variable offset
  operands, direct object-variable property operands, and the current static
  property slice over the current value model
- null coalescing `??` for direct static variables, direct array-variable
  offset operands, direct object-variable public-property operands, private
  object-property operands owned by the active declaring class, protected
  operands owned by the active class or an ancestor, and supported static
  property operands over the current value model; undefined
  variables, missing
  array keys, missing supported object properties, non-array/non-object
  targets, null variables, null array values, and null supported object
  property values, missing declared static property names, and null supported
  static property values evaluate the fallback, while falsey non-null values
  such as `false`, `0`, `""`, and `"0"` are returned without evaluating the
  fallback
- null coalescing assignment `$name ??= expr`, `$array[$key] ??= expr`,
  `$object->property ??= expr`, `ClassName::$prop ??= expr`,
  `self::$prop ??= expr`, and `parent::$prop ??= expr` for direct static
  variables, direct array-variable offset operands, direct object-variable
  public-property operands, supported private/protected object-property
  operands in active class context, and supported static property operands, in
  statement position and parenthesized
  expression position; undefined and `null` variables, undefined/null arrays,
  missing array keys, null array values, and null supported object property
  values evaluate and store the right-hand expression, while existing
  non-null values are preserved without evaluating the right-hand expression.
  Expression forms return the assigned or existing value. Undefined object
  targets, non-object property targets, and missing property names fail with
  stable runtime diagnostics instead of materializing objects or dynamic
  properties
- builtins for the documented subset: `strlen`, `strtolower`, `trim`, `ltrim`,
  `rtrim`, `strcasecmp`, `str_contains`, `str_ends_with`, `strpos`, `substr`,
  `preg_match`, `preg_replace`, `preg_split`, `preg_replace_callback`, `str_replace`, `substr_count`,
  `error_reporting`, `sprintf`, `vsprintf`, `call_user_func`, `call_user_func_array`,
  `implode`, `dirname`, `file_exists`,
  `is_dir`, `is_readable`, `register_shutdown_function`, `set_error_handler`, `restore_error_handler`, `date_default_timezone_set`,
  `version_compare`, `microtime`, `ini_get`, `min`, `rand`, `uniqid`,
  `hash_hmac`, `isset`, `empty`, `count`, `compact`, `define`, `constant`, `defined`,
  `array_key_exists`, `array_key_first`, `array_key_last`, `current`,
  `array_is_list`, `array_values`, `array_keys`, `array_reverse`, `array_slice`, `array_chunk`,
  `array_pad`, `array_merge`, `array_replace`, `array_combine`,
  `array_intersect_key`,
  `array_diff_key`, `array_diff`, `array_intersect`, `array_unique`,
  `array_flip`, `array_change_key_case`, `array_column`, `array_fill_keys`, `array_count_values`, `array_sum`,
  `array_product`, `array_reduce`, `array_filter`, `array_map`,
  `array_unshift`, `array_pop`, `next`, `ksort`,
  `in_array`, `array_search`, `gettype`, `is_null`, `is_bool`, `is_int`, `is_integer`,
  `is_long`, `is_float`, `is_double`, `is_string`, `is_array`, `is_scalar`,
  `is_numeric`, `is_countable`, `is_iterable`, `is_callable`,
  `function_exists`, `extension_loaded`, `mysqli_connect`,
  `mysqli_real_connect`, `mysqli_get_server_info`, `mysqli_get_host_info`, `mysqli_set_charset`,
  `mysqli_query`, `mysqli_errno`, `mysqli_error`, `mysqli_affected_rows`,
  `mysqli_insert_id`, `mysqli_ping`, `mysqli_select_db`, `mysqli_real_escape_string`,
  `mysqli_fetch_object`,
  `mysqli_fetch_assoc`, `mysqli_fetch_row`, `mysqli_fetch_array`,
  `mysqli_fetch_field`, `mysqli_num_fields`, `mysqli_num_rows`,
  `mysqli_data_seek`, `mysqli_free_result`, `mysqli_more_results`,
  `mysqli_next_result`, `mysqli_report`, `mysqli_init`, `header`,
  `header_remove`, `headers_sent`, `abs`, `assert`,
  `get_class`, `is_object`, `get_debug_type`, `class_exists`,
  `interface_exists`, `trait_exists`, `enum_exists`,
  `property_exists`, `method_exists`, `is_a`, `get_class_methods`, `get_class_vars`,
  `get_object_vars`, `get_mangled_object_vars`, `is_subclass_of`, `get_parent_class`,
  `get_declared_classes`, `get_declared_interfaces`, `get_declared_traits`,
  `spl_object_id`, `spl_object_hash`, `spl_autoload_register`, `var_dump`,
  and `print_r`;
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
  `version_compare($version1, $version2, $operator = null)` supports string
  versions made of dot, hyphen, or underscore separated non-negative integer
  components. With two arguments it returns `-1`, `0`, or `1`; with a string
  operator it returns a boolean for `<`/`lt`, `<=`/`le`, `>`/`gt`, `>=`/`ge`,
  `==`/`=`/`eq`, and `!=`/`<>`/`ne`. PHP's full version grammar, pre-release
  labels, arbitrary separators, invalid-argument warnings, extension version
  coupling, and native lowering remain unsupported.
  `microtime(true)` returns a finite float seconds value from the host system
  clock. The no-argument and `false` string-return forms, exact string format,
  precision guarantees, monotonicity, deterministic virtual time, broad
  coercions, exact diagnostics, and native lowering remain unsupported.
  `ini_get($option)` accepts one string option name and returns deterministic
  string values from the current compatibility registry, or `false` for
  unknown names. The registry currently covers WordPress-oriented options such
  as `memory_limit`, `max_execution_time`, `disable_functions`,
  `mbstring.func_overload`, upload/mail/error-output defaults, and related
  bootstrap settings. Host php.ini discovery, mutable INI state,
  `ini_set()`/`ini_restore()`, `ini_get_all()`, SAPI differences, extension
  ownership/access metadata, exact option catalogs, coercions, exact
  diagnostics, and native lowering remain unsupported.
  `sprintf($format, ...$values)` and `vsprintf($format, $values)` support
  string format values with literal text, escaped percent signs `%%`,
  sequential and positional `%s`, `%d`, `%f`, and `%F` placeholders, plus the
  reached WordPress width, precision, sign, zero/custom padding, and left-align
  subset. `vsprintf()` requires the second argument to be a current ordered
  array and consumes values in insertion order. String placeholders use the
  current PHP-shaped echo string conversion; numeric placeholders accept
  null/bool/int/float/numeric-string values in the current finite numeric
  subset. PHP's full format grammar, star width or precision, length
  modifiers, locale behavior, broad argument reordering, array/object/resource
  conversions, exact warning behavior, partial-output behavior, and native
  lowering remain unsupported.
  `strtolower($value)` supports exactly one scalar/null string-convertible
  argument and applies ASCII lowercase mapping over the current runtime UTF-8
  string value. Locale-sensitive case mapping, full Unicode case folding,
  binary string edge cases beyond valid UTF-8 runtime strings,
  array/object/resource coercions, exact PHP diagnostics, and native lowering
  remain unsupported.
  `trim($value)` supports exactly one scalar/null string-convertible argument
  and trims PHP's default whitespace characters for represented runtime
  strings. Custom character masks, binary/null-byte string edge cases beyond
  the current represented runtime-string subset, array/object/resource
  coercions, exact PHP diagnostics, and native lowering remain unsupported.
  `ltrim($value)` supports the same default whitespace mask on the left side.
  `ltrim($value, $mask)` also supports non-empty literal character masks
  without range syntax, including the reached WordPress `'/'` and
  `"\r\n\t ("` masks. Character-mask ranges, empty masks, binary/null-byte
  edge cases beyond the current represented runtime-string subset,
  array/object/resource coercions, exact PHP diagnostics, and native lowering
  remain unsupported.
  `rtrim($value)` supports the same default whitespace mask on the right side.
  `rtrim($value, $mask)` also supports non-empty literal character masks
  without range syntax, including the reached WordPress `'/'` mask.
  Character-mask ranges, empty masks, binary/null-byte edge cases beyond the
  current represented runtime-string subset, array/object/resource coercions,
  exact PHP diagnostics, and native lowering remain unsupported.
  `array_unshift($array, ...$values)` supports direct calls and string-valued
  direct dynamic calls when the first argument is a direct variable containing
  a current ordered array. It evaluates prepended values left to right, writes
  the mutated array back to that variable, reindexes integer keys, preserves
  string keys, and returns the new count. Non-variable array targets, non-array
  first arguments, value-only dynamic calls such as
  `call_user_func("array_unshift", ...)`, broad by-reference argument
  handling, references/copy-on-write, exact warnings, and native lowering
  remain unsupported.
  `strcasecmp($left, $right)` supports exactly two scalar/null
  string-convertible arguments, compares with ASCII case folding, and returns
  `-1`, `0`, or `1`. Array operands, object/resource coercions, binary string
  edge cases beyond valid UTF-8 runtime strings, locale-sensitive behavior,
  exact PHP diagnostics, and native lowering remain unsupported.
  `str_contains($haystack, $needle)` supports exactly two scalar/null
  string-convertible arguments and returns whether the current UTF-8 runtime
  haystack contains the current UTF-8 runtime needle. Empty needles return
  `true`. Array operands, object/resource coercions, binary string edge cases
  beyond valid UTF-8 runtime strings, exact PHP diagnostics, and native
  lowering remain unsupported.
  `str_ends_with($haystack, $needle)` supports exactly two scalar/null
  string-convertible arguments and returns whether the current UTF-8 runtime
  haystack ends with the current UTF-8 runtime needle. Empty needles return
  `true`. Array operands, object/resource coercions, binary string edge cases
  beyond valid UTF-8 runtime strings, exact PHP diagnostics, and native
  lowering remain unsupported.
  `strpos($haystack, $needle, $offset = 0)` supports scalar/null
  string-convertible haystack and needle arguments, an optional integer offset,
  byte-position matching over the current runtime string bytes, empty needles
  returning the effective offset, negative offsets measured from the end of the
  haystack, and `false` for no match. Offset coercions beyond integers,
  PHP-exact `ValueError` diagnostics, array/object/resource coercions,
  encoding-sensitive edge cases beyond represented runtime strings, and native
  lowering remain unsupported.
  `substr($string, $offset, $length = null)` supports scalar/null
  string-convertible input, integer offsets, and optional integer lengths.
  Positive and negative offsets, positive and negative lengths, and
  out-of-range empty results are implemented over byte positions when the
  resulting runtime string remains valid UTF-8. Float/string offset and length
  coercions, object/resource operands, invalid UTF-8 byte ranges, exact PHP
  diagnostics, and native lowering remain unsupported.
  `substr_count($haystack, $needle, $offset = 0, $length = null)` supports
  scalar/null string-convertible haystack and needle arguments, optional
  integer offset and length slicing, negative offsets and lengths within the
  current bounds rules, non-overlapping byte-position counts, and zero when the
  searched slice is shorter than the needle. Empty needles are a current
  runtime boundary instead of PHP-exact `ValueError`; broad scalar coercions,
  array/object/resource coercions, exact PHP diagnostics, encoding-sensitive
  edge cases beyond represented runtime strings, and native lowering remain
  unsupported.
  `preg_match($pattern, $subject, $matches = null)` supports two scalar/null
  string-convertible arguments and an optional third direct-variable matches
  output argument. The current regex slice supports
  slash-delimited literal contains/prefix/suffix/exact patterns with `^` and
  `$` anchors plus a small literal escape subset, accepts the `u` modifier as
  a no-op over the current valid UTF-8 runtime strings, returns integer `1`
  for a match and `0` for no match, and exists to cover the reached WordPress
  `wp_fix_server_vars()` SAPI-name pattern plus `_wp_can_use_pcre_u()`'s
  `//u` startup probe. With a direct `$matches` variable, literal patterns
  populate match `0`, failed matches clear the variable to an empty array, and
  the two exact WordPress `wpdb::parse_db_host()` named-capture patterns
  populate `0`, `host`/`1`, and optional `port`/`2` entries for the current
  IPv4-ish and bracketed IPv6-ish startup paths. The exact WordPress table
  prefix validation pattern `|[^a-z0-9_]|i` is also supported, returning a
  match for the first non-alphanumeric/non-underscore character and no match
  for conventional prefixes such as `wp_`. The exact WordPress safe-collation
  query classifier `/^(?:SHOW|DESCRIBE|DESC|EXPLAIN|CREATE)\s/i` is supported
  with ASCII-case-insensitive keyword matching and one following ASCII
  whitespace character. The exact adjacent `wpdb::query()` classifiers
  `/^\s*(create|alter|truncate|drop)\s/i`,
  `/^\s*(insert|delete|update|replace)\s/i`, and
  `/^\s*(insert|replace)\s/i` are supported with optional leading ASCII
  whitespace, ASCII-case-insensitive keyword matching, one following ASCII
  whitespace character, and match `0` population for direct `$matches`
  variables. The exact WordPress `wpdb::check_ascii()` non-ASCII
  byte detector `/[^\x00-\x7F]/` is supported over the current valid UTF-8
  runtime string model, returning whether any represented character is
  non-ASCII. Non-direct matches outputs, flags, offsets, optional
  unmatched-group fidelity, broad named-capture support, full PCRE syntax,
  modifiers other than the documented exact WordPress `i` patterns and `u`,
  invalid-pattern warnings, byte/Unicode edge cases, broad coercions, exact
  diagnostics, and native lowering remain unsupported.
  `preg_replace_callback($pattern, $callback, $subject)` supports exactly the
  WordPress `wp_sanitize_redirect()` verbose UTF-8 sanitizer regex shape with
  the string callback `_wp_sanitize_utf8_in_redirect`. It percent-encodes
  non-ASCII UTF-8 bytes in the current runtime string representation and
  leaves ASCII redirect paths unchanged. Pattern arrays, subject arrays,
  callback arrays/closures/method callables, broad callback invocation,
  captures/backrefs beyond the matched full string, limit/count/flags
  arguments, invalid-pattern warnings, byte/Unicode edge cases outside valid
  runtime strings, broad coercions, exact diagnostics, and native lowering
  remain unsupported.
  `preg_replace($pattern, $replacement, $subject)` supports exactly the
  WordPress database-version cleanup pattern `/[^0-9.].*/`, the WordPress
  path-tail cleanup pattern `#/[^/]*$#i`, the WordPress redirect sanitizer
  cleanup pattern `|[^a-z0-9-~+_.?#=&;,/:%!*\[\]()@]|i`, the WordPress
  mail-host cleanup pattern `#^www\.#`, and the KSES null cleanup patterns
  `/[\x00-\x08\x0B\x0C\x0E-\x1F]/` and `/\\\\+0+/`, all with an empty
  replacement string and a scalar/null string-convertible subject. The first
  returns the leading ASCII digits/dots prefix used by `wpdb::db_version()`.
  The second removes the final slash-delimited path segment used by
  `wp_guess_url()`. The third removes characters outside the current ASCII
  redirect allowlist while preserving already-percent-encoded bytes. The
  mail-host path removes a leading lowercase `www.`. The KSES paths remove
  represented ASCII control characters in the documented ranges and slash-zero
  sequences. The `wpdb::prepare()` placeholder path accepts the reached
  `/%(?:%|$|(?!($allowed_format)?[sdfFi]))/` shape after WordPress expands
  `$allowed_format`, with the exact `'%%\\1'` replacement, escaping
  unrecognized percent signs while preserving recognized placeholders in the
  current formatting-prefix subset. Pattern or replacement arrays, subject
  arrays, arbitrary non-empty replacements, limit/count arguments, callbacks,
  full PCRE replacement behavior, captures/backrefs beyond the reached
  placeholder replacement, invalid-pattern warnings, byte/Unicode edge cases,
  broad coercions, exact diagnostics, SQL semantics, and native lowering remain
  unsupported.
  `preg_split($pattern, $subject, $limit, $flags)` supports exactly the
  reached WordPress `wpdb::prepare()` placeholder extraction regex
  `/(^|[^%]|(?:%%)+)(%(?:$allowed_format)?[sdfFi])/` after `$allowed_format`
  expansion, with `limit` exactly `-1` and `flags` exactly
  `PREG_SPLIT_DELIM_CAPTURE` (value `2`). It returns a sequential array with
  the leading literal segment and the two captured delimiter groups for each
  recognized placeholder, matching the shape WordPress documents as one
  leading value plus three values per placeholder. Broad PCRE splitting,
  pattern arrays, subject arrays, other limits, `PREG_SPLIT_NO_EMPTY`,
  `PREG_SPLIT_OFFSET_CAPTURE`, flag combinations, invalid-pattern warnings,
  full capture semantics, SQL semantics, and native lowering remain
  unsupported.
  `error_reporting($mask = null)` supports no arguments to read the current
  integer mask and one integer argument to store a new current mask while
  returning the previous mask. The interpreter initializes the mask to `E_ALL`.
  PHP's warning/notice/deprecation filtering, ini integration,
  disabled-function policy, non-integer coercions, exact diagnostics, and
  native lowering remain unsupported.
  `str_replace($search, $replace, $subject, $count = null)` supports scalar or
  array search values when each search value is scalar/null
  string-convertible, a scalar/null string-convertible replacement, and a
  scalar/null string-convertible subject. Search arrays apply each search
  string sequentially. Direct calls and string-valued dynamic calls may pass a
  direct variable as the fourth `$count` output argument; the interpreter
  writes the aggregated non-overlapping replacement count as an integer. This
  is a bounded output-parameter path, not true PHP references. Replacement
  arrays, subject arrays, nested search arrays, non-variable count targets,
  indirect `call_user_func()` count output, object/resource coercions, exact
  warning behavior, binary string edge cases, and native lowering remain
  unsupported.
  `min($value, ...$values)` supports two or more integer arguments and returns
  the smallest integer. Array-form `min([..])`, mixed-type comparison rules,
  float/string/bool/null/object/resource operands, exact PHP diagnostics, and
  native lowering remain unsupported.
  `rand()` supports the reached no-argument form only and returns a
  deterministic integer for WordPress placeholder-salt exploration. Min/max
  arguments, random-state compatibility with PHP, seeding, `mt_rand()`/`srand()`
  coupling, cryptographic randomness, exact diagnostics, and native lowering
  remain unsupported.
  `uniqid($prefix = '', $more_entropy = false)` supports scalar/null
  string-convertible prefixes and a boolean entropy flag, returning a
  deterministic ID for the reached WordPress placeholder hash path.
  `hash_hmac('sha256', $data, $key, false)` supports scalar/null
  string-convertible data and key values and returns lowercase hex output.
  Other algorithms, `hash()`, `hash_equals()`, `hash_hmac_algos()`, raw binary
  output, exact time/entropy behavior, cryptographic guarantees for generated
  IDs, array/object/resource coercions, exact diagnostics, and native lowering
  remain unsupported.
  `call_user_func($callback, ...$args)` supports string callbacks resolving to
  current user functions or documented callable builtins and forwards evaluated
  positional values through the current value-call path. Array callables,
  closure invocation, `__invoke`, references, variadic unpacking, exact PHP
  warning behavior, and native lowering remain unsupported.
  `call_user_func_array($callback, $args)` supports string callbacks resolving
  to current user functions or documented callable builtins, public
  `[object, method]` instance callbacks, public `[class, method]` static
  callbacks, and integer-keyed ordered arrays expanded as positional argument
  lists. String-keyed named arguments, closure and `__invoke` callbacks,
  non-public methods, by-reference argument propagation, other callable array
  shapes, exact PHP warning behavior, and native lowering remain unsupported.
  `implode($array)` and `implode($separator, $array)` support current arrays
  containing only `null`, bool, int, float, and string values, preserve
  insertion order, ignore keys, and join values using PHP-shaped echo string
  conversion with an empty default separator. The legacy reversed argument
  order, nested arrays, object/resource values, exact warning behavior,
  partial-output behavior, and native lowering remain unsupported.
  `function_exists($name)` checks string names against the current runtime
  function table, including current user functions and documented callable
  builtins. Conditional/nested user-function declarations become visible only
  after their declaration statement executes. Non-string names are rejected in
  the current subset.
  `mysqli_connect(...)` is currently a database-extension boundary only: the
  name is visible through function/callability metadata and dynamic
  string-valued calls, but any attempted connection call reports a stable
  unsupported runtime diagnostic. `mysqli_real_connect($handle, ...)` accepts
  the current WordPress startup shape for a placeholder `mysqli` object,
  string/null hostname, username, password, and database arguments, int/null
  port, string/null socket, and int flags. It performs no host I/O, writes
  `connect_errno = 0` and `connect_error = null`, and returns `true` as a
  deterministic compatibility boundary. `mysqli_report($mode)` accepts the
  current WordPress startup modes `MYSQLI_REPORT_OFF` and
  `MYSQLI_REPORT_ERROR | MYSQLI_REPORT_STRICT`, stores the current mode, and
  returns `true`. `mysqli_init()` returns a current placeholder `mysqli` object
  with `connect_errno` set to `0` and `connect_error` set to `null`, but it is
  not backed by a host connection. `mysqli_get_server_info($handle)` accepts
  the placeholder `mysqli` object and returns the deterministic placeholder
  string `8.0.0-phpc-placeholder` for WordPress version-guard exploration.
  `mysqli_get_host_info($handle)` accepts the placeholder object and returns
  deterministic `localhost via TCP/IP (phpc-placeholder)` metadata without
  inspecting a real host, transport, socket, protocol, or live connection.
  `mysqli_set_charset($handle, "utf8mb4")` accepts the placeholder handle and
  returns deterministic `true` for the reached WordPress charset setup path
  without negotiating a real connection charset or collation state.
  `mysqli_query($handle, 'SELECT @@SESSION.sql_mode')` accepts the placeholder
  handle and that exact WordPress SQL mode probe, returning `false` as a
  deterministic empty/no-result boundary so WordPress skips SQL mode
  normalization without executing SQL. `mysqli_query(...)` also accepts the
  reached WordPress charset setup query
  `SET NAMES 'utf8mb4' COLLATE 'utf8mb4_unicode_520_ci'`, returning `true`
  as a deterministic successful no-result boundary without changing real
  connection charset state. `mysqli_query(...)` also accepts the
  reached WordPress options-table bootstrap reads
  `SELECT option_name, option_value FROM <prefix>options WHERE autoload IN (
  'yes', 'on', 'auto-on', 'auto' )` and
  `SELECT option_name, option_value FROM <prefix>options`, plus the reached
  empty option-cache reads using `WHERE option_name IN (...)` and
  `SELECT option_value FROM <prefix>options WHERE option_name = ... LIMIT 1`,
  plus reached empty WordPress metadata probes `SHOW FULL COLUMNS FROM ...`
  and `DESCRIBE ...`, returning deterministic empty boundaries without
  executing SQL. For the exact synthetic empty result query
  `SELECT * FROM wp_posts WHERE 1 = 0`, `mysqli_query()` returns a placeholder
  `mysqli_result` object. `mysqli_num_fields($result)` returns `0`,
  `mysqli_num_rows($result)` returns `0`,
  `mysqli_fetch_field($result)` and `mysqli_fetch_object($result)` return
  `false`, and `mysqli_free_result($result)` returns `null` for that
  placeholder empty result. For the exact deterministic seed-post query
  `SELECT ID, post_title FROM wp_posts WHERE ID = 1`, `mysqli_query()` returns
  a placeholder `mysqli_result` with two fields, `ID` and `post_title`, and
  one row. `mysqli_num_fields($result)` returns `2`,
  `mysqli_num_rows($result)` returns `1` without advancing the shared row
  cursor,
  `mysqli_fetch_field($result)` returns `stdClass` objects whose `name`
  properties are `ID` and `post_title` before returning `false`, and
  `mysqli_fetch_object($result)` returns one `stdClass` row with `ID = 1` and
  `post_title = "Hello world placeholder"` before returning `false`.
  `mysqli_fetch_assoc($result)` uses the same row cursor and returns one
  associative PHP array with `ID = 1` and
  `post_title = "Hello world placeholder"` before returning `false`.
  `mysqli_fetch_row($result)` returns one numeric PHP array with keys `0` and
  `1` over the same row cursor.
  `mysqli_fetch_array($result, MYSQLI_ASSOC)` returns the same associative
  array shape, `mysqli_fetch_array($result, MYSQLI_NUM)` returns numeric keys
  `0` and `1`, and `mysqli_fetch_array($result, MYSQLI_BOTH)` plus omitted
  mode/default `MYSQLI_BOTH` returns both numeric and associative keys. All
  fetch-array modes share that row cursor. The `MYSQLI_ASSOC`, `MYSQLI_NUM`,
  and `MYSQLI_BOTH` constants are exposed with their PHP integer values.
  `mysqli_data_seek($result, $offset)` accepts integer offsets for placeholder
  results, resets the row cursor for in-range offsets, and returns `false` for
  negative or out-of-range offsets.
  `mysqli_free_result($result)` releases that interpreter-owned placeholder
  result state and returns `null`. `mysqli_more_results($handle)` and
  `mysqli_next_result($handle)` return `false` for the placeholder connection.
  Other `SELECT` statements fail with an explicit unsupported diagnostic that
  non-empty `mysqli` result sets are not implemented in the current subset.
  For the same placeholder handle, `mysqli_errno($handle)` returns `0` and
  `mysqli_error($handle)` returns an empty string.
  `mysqli_affected_rows($handle)` and `mysqli_insert_id($handle)` return
  deterministic `0` for the clean placeholder connection state.
  `mysqli_ping($handle)` accepts the placeholder handle and returns
  deterministic `true` as a liveness-check boundary without probing a real
  connection or reconnecting.
  Mutation SQL passed to `mysqli_query()`, currently recognized by leading
  `INSERT`, `UPDATE`, `DELETE`, or `REPLACE`, reports an explicit unsupported
  diagnostic instead of changing connection or table state.
  `mysqli_select_db($handle, $database)` accepts the placeholder handle and a
  string or null database name, returning deterministic `true` for the reached
  WordPress `wpdb::select()` path without selecting a real database.
  `mysqli_real_escape_string($handle, $data)` accepts the placeholder handle
  and a scalar/null string-convertible value, returning deterministic
  MySQL-style escaping for NUL, newline, carriage return, backslash, single
  quote, double quote, and Ctrl-Z characters for the reached
  `wpdb::_real_escape()` option lookup path.
  Host connections, real mysqli resources/objects, real query execution,
  general non-empty result sets, real row/field metadata,
  real affected-row/insert-id state, connection charset state, binary or
  invalid-string behavior, exact escaping edge cases, errors/warnings,
  transactions, configuration beyond the report-mode flag, PDO behavior, and
  native database calls are not implemented.
  `compact($name, ...$names)` supports one or more direct string variable-name
  arguments, reads the current caller scope, returns an array keyed by each
  found variable name, and omits missing variables. This covers the reached
  WordPress `compact('charset', 'collate')` path. Array arguments, nested
  arrays, invalid names, PHP warning behavior for missing or invalid names,
  variable-variable interactions, exact diagnostics, and native lowering remain
  unsupported.
  `assert($assertion, $description = null)` evaluates one or two arguments
  normally and returns `true` for truthy assertions. The optional description
  is inert metadata in this slice and may be `null`, bool, int, float, or
  string. Failing assertions remain a stable runtime boundary; assertion INI
  policy, callbacks, `AssertionError`, `Throwable` descriptions, exact
  warning/fatal behavior, PHP 8.3 deprecations, partial-output behavior, and
  native lowering are not implemented.
  `dirname($path, $levels = 1)` accepts string paths and an optional positive
  integer level count. It performs lexical Unix-style slash parent-directory
  extraction for local paths used by the current WordPress bootstrap probes;
  it does not resolve the filesystem, symlinks, include paths, stream wrappers,
  Windows drive/UNC paths, locale/codepage details, or PHP's exact coercion and
  `ValueError` behavior.
  `file_exists($path)` accepts one string local path, rejects stream-wrapper
  paths, and returns `true` or `false` from the host filesystem metadata lookup
  for files and directories in the current interpreter run. Relative paths are
  checked against the process path first and then against the repository root
  for committed source-map fixture paths. It is a bounded WordPress bootstrap
  compatibility slice, not full PHP filesystem support: include-path lookup,
  stream wrappers, canonicalization/symlink policy, permissions and warning
  fidelity, open_basedir, stat-cache behavior, TOCTOU semantics, host
  filesystem coupling, partial-output behavior, and native lowering remain
  unsupported.
  `is_dir($path)` accepts one string local path, rejects stream-wrapper paths,
  returns `true` for host directories, and returns `false` for missing paths or
  non-directory paths. It shares the same current relative path policy as
  `file_exists`. Include-path lookup, stream wrappers,
  canonicalization/symlink policy, portable permissions, warning behavior,
  non-string coercions, stat-cache behavior, open_basedir, partial-output
  behavior, and native lowering remain unsupported.
  `is_readable($path)` accepts one string local path, rejects stream-wrapper
  paths, returns `false` for missing paths, and checks host readability for
  files with `File::open` and directories with `read_dir`. It shares the same
  current relative path policy as `file_exists`. Include-path lookup, stream
  wrappers, canonicalization/symlink policy, portable permissions, warning
  behavior, non-string coercions, stat-cache behavior, open_basedir,
  partial-output behavior, and native lowering remain unsupported.
  `spl_autoload_register($callback, $throw = true, $prepend = false)` accepts
  closure expressions or string callback names plus optional boolean flags and
  returns `true`. It currently records no autoload stack and never invokes the
  callback; this is a WordPress bootstrap compatibility boundary, not full SPL
  autoloading.
  `register_shutdown_function($callback, ...$args)` accepts a currently valid
  string callable, object/static array callable, or closure plus optional
  already-evaluated extra arguments and returns `null`. The current slice
  validates registration only; it does not store or execute callbacks at a
  shutdown phase. Callback ordering, argument delivery, by-reference callback
  behavior, output buffering, destructor/finally interaction, fatal-error
  context, exact diagnostics, and native lowering remain unsupported.
  `set_error_handler($callback, $error_levels = E_ALL)` accepts a currently
  valid string callable, object/static array callable, or closure plus an
  optional integer error-level mask, records the current handler value, and
  returns the previous handler value or `null`. The current slice validates
  registration only; it does not invoke handlers or route warnings, notices,
  deprecations, or fatal conditions through user callbacks.
  `restore_error_handler()` accepts no arguments, clears the current bounded
  handler registration, and returns `true`. True PHP handler-stack behavior,
  error-level filtering, callback invocation arguments, by-reference callback
  behavior, output buffering, shutdown/fatal interaction, exact diagnostics,
  and native lowering remain unsupported.
  `date_default_timezone_set($timezoneId)` accepts one string argument, returns
  `true` for `UTC`, and returns `false` for other identifiers without PHP's
  notice machinery. Full timezone database validation, global timezone state,
  `date_default_timezone_get()`, date extension integration, ini interactions,
  warning behavior, exact diagnostics, and native lowering remain unsupported.
  `header($header, $replace = true, $response_code = 0)` accepts a string
  header line plus optional bool replacement flag and optional integer response
  code, returns `null`, and records no header state. This is a WordPress
  bootstrap compatibility boundary only; response header storage, status-code
  state, replacement/removal behavior, output-sent warnings, web-server/SAPI
  integration, exact diagnostics, partial-output behavior, and native lowering
  remain unsupported.
  `header_remove($name = null)` accepts no argument or one string header name,
  returns `null`, and records no header state. Actual all-header removal,
  named-header storage/removal, output-sent warnings, SAPI/web-server behavior,
  exact diagnostics, partial-output behavior, and native lowering remain
  unsupported.
  `headers_sent()` accepts no arguments and returns `false` in the current
  no-header-state runtime shim. Filename/line output arguments, output-started
  tracking, header storage, output buffers, SAPI differences, exact warnings,
  and native lowering remain unsupported.
  `abs($value)` accepts current integer and finite-float runtime values,
  returning an integer for integer input and a float for finite-float input.
  Integer-minimum overflow, numeric string coercion, bool/null coercion,
  array/object/resource operands, NaN/infinity behavior, exact diagnostics, and
  native lowering remain unsupported.
  `extension_loaded($name)` accepts string extension names and currently
  answers from a deterministic bounded compiler/runtime compatibility registry.
  It returns `true` for `json` and `hash`, and `false` for other names,
  including WordPress probe names such as `mbstring` and `sodium`, without
  querying host PHP modules, `php.ini`, SAPI state, or dynamically loading
  extensions; non-string names are rejected in the current subset.
  `get_class` returns the declared class name for current minimal object
  values, `is_object` reports whether a value is one of those current object
  values, `get_debug_type` returns scalar/array type names or the current
  object's declared class name, `class_exists` checks the current declared
  class metadata by string name without autoloading, `interface_exists`
  accepts string names and checks current declared interface metadata without
  autoloading, `trait_exists` accepts string names and checks current declared
  trait metadata without autoloading,
  `enum_exists` accepts string names and checks current declared unit-enum
  metadata without autoloading,
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
  `get_declared_classes` returns a zero-indexed array of classes declared in
  the current program followed by declared unit enums,
  `get_declared_interfaces` returns a zero-indexed array of interfaces
  declared in the current program in declaration order,
  `get_declared_traits` returns a zero-indexed array of traits declared in the
  current program in declaration order,
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
  non-array `current` operands, non-array `array_is_list` operands,
  non-array `array_reverse` operands,
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
  function-scope `global` declarations, duplicate class/member metadata, undefined classes,
  unsupported object instantiation, undefined object properties, invalid
  property targets, unsupported non-public property access, non-object
  `get_class` operands, unsupported `property_exists` object/class or
  property arguments, unsupported `method_exists` object/class or method
  arguments, unsupported `is_a` class-name or allow-string arguments,
  non-object `get_object_vars` operands, non-object
  `get_mangled_object_vars` operands,
  unsupported `get_parent_class` object/class arguments,
  unsupported `get_called_class()` calls outside method or static class
  context,
  object-to-string conversion,
  unsupported strict identity array/object operands, invalid `foreach`
  iterables, invalid `break`/`continue` outside a loop, unsupported `continue;`
  inside `switch`, and runaway user-function recursion
- explicit parse diagnostics for unsupported function syntax: variadic argument
  unpacking, reference expressions, function-scope reference parameter
  invocation, reference returns, type declaration enforcement, named arguments,
  first-class callable syntax such as `strlen(...)` and `$callback(...)`, and
  `declare(strict_types=1)`
- explicit parse diagnostics for unsupported magic constants such as
  `__CLASS__`, `__TRAIT__`, and `__NAMESPACE__`
- narrow `require`, `require_once`, `include`, and `include_once` execution
  for local string paths in statement and expression position, including
  constant/string concatenation, source-file-relative path resolution, included
  file declaration registration, caller-scope execution, include return values,
  and `_once` de-duplication by resolved local file. Declaration-order
  dependencies across included files remain outside this slice.
- explicit parse diagnostics for unsupported direct `eval(...)` syntax
- one unbracketed named `namespace` declaration per file, plus simple
  top-level class `use` imports with optional `as` aliases. Class declarations
  and class-like references in `extends`, `new`, `instanceof`, static
  members, and `ClassName::class` resolve through the current namespace and
  class-import table. Namespace-scoped function declarations register under
  their resolved names, and unqualified direct calls inside a namespace resolve
  to same-namespace functions with global builtin/user-function fallback.
- explicit parse diagnostics for unsupported namespace forms and imports:
  bracketed/global/multiple namespaces, namespace-scoped constants, grouped
  imports, function imports, constant imports, qualified function calls, and
  namespace-qualified function calls such as `App\make()`
- explicit parse diagnostics for unsupported magic class names in `new`
  expressions such as `new self()`, `new parent()`, and `new static()`
- explicit parse diagnostics for unsupported nested, namespace-aware, or
  dynamic-value `const` declarations
- stable runtime diagnostics for unsupported bare global constants outside the
  current built-in/runtime-defined slice, such as `PHP_OS`
- explicit parse diagnostics for unsupported array spread elements and
  reference array keys
- explicit parse diagnostics for unsupported array/list destructuring beyond
  the current positional `list($a, $b) = expr;` statement slice with variable
  or skipped slots, such as `[$name] = $array`, expression-position
  `list(...)`, nested/keyed targets, references, and non-variable targets
- explicit parse diagnostics for unsupported `unset(...)` forms outside the
  current direct-variable, direct/nested array-offset, direct/dynamic
  object-property, nested object-property array-offset, and static-property
  diagnostic statement subset
- explicit parse diagnostics for unsupported `foreach` key-by-reference forms,
  destructuring loop targets, and expression-position `foreach`
- explicit parse diagnostics for unsupported expression-position `for` and
  comma-separated `for` header expression lists
- explicit parse diagnostics for unsupported expression-position `do ... while`
- explicit parse diagnostics for malformed or mixed alternate
  `if`/`elseif`/`else` colon/`endif` syntax
- explicit parse diagnostics for unsupported expression-position `switch` and
  malformed alternate colon/`endswitch` switch bodies
- explicit parse diagnostics for unsupported dynamic or invalid
  `break`/`continue` loop-depth arguments
- explicit parse diagnostics for unsupported exception-control syntax:
  throw expressions plus malformed or standalone `catch` and `finally`
- explicit parse diagnostics for unsupported generator `yield` and
  `yield from` expressions
- explicit parse diagnostics for unsupported PHP 8 `match` expressions
- bounded `goto target;` statements and `target:` labels in the current
  statement runtime; labels in the active statement list can be reached from
  nested statements that propagate the jump outward
- explicit parse diagnostics for unsupported exponentiation syntax: `**` and
  `**=`
- explicit parse diagnostics for unsupported unparenthesized nested ternary
  expressions
- explicit parse diagnostics for unsupported assignment-expression forms
  outside the documented direct-variable, direct/nested array-offset,
  append/append-at-depth, direct object-property, and supported static-property
  target subset, including append-offset chained assignments and complex mixed
  object/property/ArrayAccess targets
- explicit parse diagnostics for unsupported compound assignment targets
  outside direct static variables, direct array offsets, direct object
  properties, direct object-property array offsets, and supported static
  properties
- explicit parse diagnostics for unsupported increment/decrement targets
  outside direct static variables, direct array offsets, direct object
  properties, and supported static properties, plus chained
  increment/decrement expressions
- explicit parse diagnostics for unsupported chained coalescing and
  non-variable null coalescing assignment forms
- explicit parse diagnostics for unsupported object/class syntax: unbraced
  nested class declarations, broader inheritance forms beyond declared
  single-parent `extends`, interface inheritance/constants/non-public or
  static interface methods, trait members,
  trait use inside classes, backed enum declarations and enum members beyond
  bare cases,
  `abstract`/`final`/`readonly` class
  modifiers, `abstract`/`final`/`readonly` class member modifiers,
  typed instance property declarations, typed static property declarations
  before typed metadata/uninitialized state/write enforcement exist, instance
  property default values, multiple property declarations, unsupported class
  constant declaration forms such as typed, static, or multi-declarator class constants,
  malformed `clone` expressions, dynamic `instanceof` class operands,
  unsupported magic static receiver forms outside the current `static::class`,
  `static::method(...)`, and `static::$prop` slices,
  anonymous class expressions,
  and broader late-bound `static::` member forms
- explicit lex diagnostics for unsupported variable-variable syntax such as
  `$$name` and `${...}`
- double-quoted string interpolation for simple `$name`, braced `{$name}`,
  direct array offsets such as `{$items['name']}`, `{$items[$key]}`, and
  `$items[name]`, direct object properties such as `{$partial->id}`, and
  chained property/offset reads such as
  `{$block->context['displayLayout']['columns']}`. Array keys may currently be
  string literals, integer literals, bare string keys, or variable keys that
  coerce through the current array-key rules. Dynamic property names, static
  properties, `${...}`, variable variables, arbitrary expression
  interpolation, exact diagnostics, and native lowering remain unsupported.
- syntax-only PHP attributes beginning with `#[...]` are accepted and ignored
  before functions, classes, class members, and parameters. Attribute metadata,
  reflection visibility, target validation, namespace-aware attribute names,
  constructor argument evaluation, repeated-attribute rules, and native
  lowering remain unsupported; ordinary `#` comments, including `# [` with
  whitespace before the bracket, remain comments

## Partially Supported

- Variable storage: top-level code and each user-function call use materialized
  symbol tables keyed by variable name. Current static variable reads, writes,
  direct `unset($name)`, `isset($name)`, parameter binding, default-parameter
  evaluation, and direct array writes route through that symbol table path.
  Top-level `global $name, ...;` declarations preserve existing values and
  materialize missing listed names as `null`; function-scope `global`
  declarations import from the root symbol table through the existing
  function/global sharing path.
  Direct `unset($name)` removes the current-scope symbol and treats missing
  names as no-ops; later plain reads use the existing undefined-variable
  diagnostic. Multiple supported `unset(...)` operands run left to right.
  Runtime lookup by a value computed from PHP code is not implemented yet, so
  variable variables still do not execute.
- Null coalescing: `phpc run` supports an executable `??` slice where the left
  operand is a direct static variable, direct array-variable offset, direct
  object-variable public property, or supported static property. The left
  operand uses PHP-style isset semantics for the current value model:
  undefined variables, missing array keys, missing public properties, missing
  supported static properties, null variables, null array values, null public
  property values, null supported static property values, non-array
  array-offset targets, and non-object property targets use the fallback,
  while falsey non-null values are returned as-is and the fallback expression
  is not evaluated. `phpc run` also supports direct-variable `$name ??= expr`,
  direct array-offset `$array[$key] ??= expr`, direct public object-property
  `$object->property ??= expr`, and supported static-property `??=`
  statements. These statement forms evaluate the right-hand expression only
  when the target variable, array slot, or public property slot is undefined,
  missing, or null. Direct array-offset `??=` materializes undefined/null
  target variables as arrays; existing non-array targets fail with the current
  stable invalid-array-access diagnostic. Direct object-property `??=` writes
  only existing declared public properties on existing object values. Supported
  static-property `??=` writes only declared untyped static
  properties through `ClassName::$prop`, `self::$prop`, and `parent::$prop`
  after current visibility checks. Missing properties, undefined target
  variables, non-object target variables, and unsupported static-property
  contexts fail with stable diagnostics. Complex or nested `??` left operands,
  append-offset `??=` targets, dynamic property names, magic methods,
  unparenthesized chained coalescing, references/copy-on-write, exact native
  error objects, and native lowering remain unsupported.
- Include/require: `require path;`, `require_once path;`, `include path;`,
  and `include_once path;` execute in statement position and expression
  position for paths that evaluate to strings in the current subset, including
  constant and string-concatenated paths such as `ABSPATH . WPINC .
  '/load.php'`. Absolute paths resolve directly; relative paths resolve
  against the source file containing the construct. Included files are parsed
  with `<?php`, register top-level functions/classes, and run in the caller
  symbol table. Statement forms ignore top-level include return values.
  Expression forms return the included file's top-level `return` value, return
  `1` when the file completes normally, and return `true` for `_once`
  constructs when the resolved file was already loaded. `require_once` and
  `include_once` de-duplicate by resolved local file, including files loaded
  first through non-once `require`/`include`. Missing-file include
  warning/recovery, include-path lookup, stream wrappers, URL includes,
  `phar://`, opcache behavior, autoload interaction, declaration-order edge
  cases, source mapping for functions/classes after include, PHP's exact
  warning-vs-fatal recovery behavior, and native lowering are not implemented.
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
- Namespaces/imports: `phpc run` supports a bounded class-name plus
  same-namespace function slice: one
  unbracketed named namespace per file, simple top-level class imports with
  optional aliases, namespace-qualified class declarations, and class-like
  references for declarations, `extends`, `new`, `instanceof`, static
  members, and `ClassName::class`. Namespace-scoped function declarations
  register under resolved names, and unqualified direct calls inside a
  namespace resolve to same-namespace functions with global builtin/user-function
  fallback. String class names and dynamic string callable names remain literal and are
  not import-expanded. A namespaced `class Child extends Parent {}` resolves
  the parent name through the same lexical namespace/import table, but the
  parent must already be declared in the current program or an executed
  include/require path; autoload-triggered parent discovery is not implemented.
  Bracketed namespace blocks, global namespace blocks,
  multiple namespaces in one file, namespace-scoped constants,
  namespace-qualified function calls, grouped imports, function imports,
  constant imports, trait `use` execution, `__NAMESPACE__`, autoload
  interaction, exact PHP diagnostics, partial-output behavior, and
  namespace-aware native lowering are not implemented.
- Object/class model: `php_runtime` has a small metadata and object-value model
  for the first object slice. It records an ordered class table with stable
  `ClassId` handles, declared class names with case-insensitive class lookup,
  ordered property metadata with case-sensitive property lookup, ordered class
  constant metadata with case-sensitive lookup, ordered method metadata with
  case-insensitive method lookup, visibility flags,
  static/instance flags, object-shape derivation for instance properties,
  initialized object values, and structured duplicate class/member diagnostics.
  `phpc run` pre-registers top-level class declarations into this metadata
  table. Nested class declarations are marked in the AST and register only when
  execution reaches the statement, so false branches do not populate the class
  table and guarded declarations such as `if (!class_exists("Name")) { class
  Name {} }` can safely avoid repeated redeclaration in the current subset.
  The accepted member subset records untyped properties with optional
  constant-expression defaults and methods
  whose parameters/bodies use the existing function parser subset, including
  optional trailing commas after the final real parameter. `new
  ClassName(...)` looks up declared classes case-insensitively, initializes
  inherited and exact-class instance properties from supported defaults or
  `null`, skips static properties, collapses compatible public/protected inherited property
  redeclarations into one shared runtime slot, keeps private parent property
  redeclarations as separate child slots, treats object values as truthy, and
  lets direct `isset($object_variable)` return true. Public
  or inherited public instance `__construct` methods execute after object
  allocation with `$this` bound to the new object handle. Explicit
  `parent::__construct(...)` and `parent::method(...)` calls execute in active
  instance method/constructor context against the current single-parent chain.
  `self::method(...)` calls execute in active instance method/constructor
  context against the current class and inherited method chain.
  `ClassName::CONST`, `self::CONST`, and `parent::CONST` resolve declared or
  inherited class constants with case-sensitive names and current
  public/protected/private visibility checks.
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
  access on non-object values, non-public properties outside the current
  private/protected method-context slice, and non-object `get_class` arguments
  still fail with stable runtime diagnostics.
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
  string interface names, perform case-insensitive lookup against interfaces
  declared in the current parsed program, and are available through
  string-valued dynamic function calls. The autoload flag accepts current
  bool-like scalar values and does not trigger autoloading.
  `trait_exists($name)` and `trait_exists($name, $autoload)` accept string
  trait names, perform case-insensitive lookup against empty top-level traits
  declared in the current parsed program, and are available through
  string-valued dynamic function calls. The autoload flag accepts current
  bool-like scalar values and does not trigger autoloading.
  `enum_exists($name)` and `enum_exists($name, $autoload)` accept string enum
  names, perform case-insensitive lookup against top-level unit enums declared
  in the current parsed program, and are available through string-valued
  dynamic function calls. The autoload flag accepts current bool-like scalar
  values and does not trigger autoloading. `class_exists()` also reports true
  for declared enums in the current class-like metadata slice.
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
  declaration order, including public static properties, with current supported
  default values or `null` for properties without defaults. It is available
  through string-valued dynamic function calls.
  `get_object_vars($object)` accepts current object values and returns an array
  of public exact and inherited instance property names in parent-to-child slot
  order with their current slot values. Compatible public redeclarations expose
  the shared inherited slot once instead of duplicate parent/child entries.
  Protected/private slots and static properties are not included. It is
  available through string-valued dynamic function calls.
  Direct `empty($object->name)` accepts direct object-variable public-property
  operands, returns true for falsey public property slots, missing properties,
  undefined target variables, and non-object target variables, and uses a
  stable unsupported-property diagnostic for non-public properties.
  `get_mangled_object_vars($object)` accepts current object values and returns
  public, protected, and private instance slots in declaration order. Public
  property keys are emitted as the declared name, protected property keys are
  emitted as `\0*\0name`, and private property keys are emitted with the
  declaring class name as `\0ClassName\0name`; static properties are omitted.
  Dynamic properties, trait/interface properties, and
  non-public visibility-context behavior beyond the current declaring-class
  method context are not represented yet. It is available through
  string-valued dynamic function calls.
  Class `implements` clauses accept comma-separated class-like names and record
  them as class metadata. This metadata participates in relationship checks,
  including through parent classes, without enforcing interface methods,
  requiring the interface to be declared, or seeding built-in/internal
  interfaces such as `Iterator` for `interface_exists()`.
  `is_a($object_or_class, $class_name)` accepts current object values and
  checks exact class identity, a single-parent ancestor relationship, or a
  recorded `implements` relationship against the current declared class
  metadata using case-insensitive name lookup.
  `is_a($object_or_class, $class_name, true)` also accepts a string first
  argument and checks the same relationships. A false or omitted
  `allow_string` flag makes string first arguments return false. Missing
  source class names return false; missing target class names can still match
  recorded `implements` metadata, which is how unresolved internal interface
  names are represented in the current slice. String-valued dynamic calls to
  `is_a` use the same path.
  `is_subclass_of($object_or_class, $class_name[, $allow_string])` accepts the
  current object/string first-argument subset and string class names, considers
  two-argument string first arguments and three-argument string first arguments
  only when `allow_string` is true, checks parent and recorded `implements`
  metadata relationships, returns false for exact-class and no-relationship
  cases, and is available through string-valued dynamic calls.
  `get_parent_class($object_or_class)` accepts current object values or
  declared string class names, returns the immediate parent class name when
  one is recorded and false otherwise, and is available through string-valued
  dynamic calls.
  `get_declared_classes()` returns a zero-indexed array of classes declared in
  the current parsed program followed by declared unit enums and is available
  through string-valued dynamic calls.
  `get_declared_interfaces()` returns a zero-indexed array of interfaces
  declared in the current parsed program in declaration order and is available
  through string-valued dynamic calls. Built-in/internal interface entries are
  not represented yet.
  `get_declared_traits()` returns a zero-indexed array of empty top-level
  traits declared in the current parsed program in declaration order and is
  available through string-valued dynamic calls. Built-in/internal trait
  entries are not represented yet.
  Named static method expressions such as `ClassName::method(...)` execute for
  declared or inherited visible static methods under the current positional
  argument/default-parameter subset, `$object::method(...)` and
  `$className::method(...)` execute visible static methods using the receiver
  class as the called-class context,
  and `self::method(...)`, `parent::method(...)`, and `static::method(...)`
  execute resolved visible static methods while an active class/called-class
  context exists. `clone $object` expressions evaluate the operand, require a
  current object value,
  allocate a fresh process-local object handle, shallow-copy the object's
  current property slots, and return the cloned object. Object-valued
  properties keep their existing handles under the current no-reference and
  no-copy-on-write model. Classes that declare `__clone`, non-object operands,
  private/protected clone-method visibility behavior, destructor/reuse
  behavior, exact PHP `Error` objects, partial-output behavior, references,
  copy-on-write, and native lowering remain unsupported.
  `$value instanceof Name` executes for the current bounded runtime slice:
  non-object left operands return `false`, object operands check declared class
  metadata, the current single-parent chain, and recorded `implements`
  metadata, including unresolved internal interface names recorded on class
  declarations. Unknown names without a matching class or recorded
  `implements` entry return `false`.
  Dynamic right-hand class operands, namespace-qualified names, `self`/`parent`/`static`
  targets, autoload side effects, exact PHP diagnostics, and native lowering
  remain unsupported.
  `ClassName::class` expressions return the source-spelled class string without
  requiring class metadata. `self::class` resolves to the active declaring
  class name and `parent::class` resolves to that class's immediate parent
  name while executing in class context. `static::class` resolves to the active
  called class for current instance and static method calls; outside method or
  static class context it fails with a stable runtime diagnostic.
  Class constant declarations accept the current constant-expression value
  subset, and `ClassName::CONST`, `self::CONST`, `parent::CONST`, and
  late-bound `static::CONST` in active called-class context resolve declared
  or inherited class constants case-sensitively through `phpc run` with
  public/protected/private visibility checks in the current active class
  context. Typed constants, multiple constants in one class declaration,
  broader string-name lookup for `self::CONST`, `parent::CONST`,
  `static::CONST`, autoload-triggered class discovery, enum cases/interface
  constants beyond the current metadata, typed constants, multiple constants
  in one class declaration, and native lowering remain unsupported.
  Static property reads, direct writes, compound assignment, pre/post
  increment/decrement, `isset`, `empty`, `??`, `??=`, and stable diagnostics
  for PHP-forbidden `unset(...)` through
  `ClassName::$prop`, `self::$prop`, `parent::$prop`, and late-bound
  `static::$prop` in active called-class context use class-level
  storage initialized from the current constant-expression default subset or
  `null`, resolve inherited properties case-sensitively, and enforce current
  visibility checks. Dynamic receiver static property reads and direct writes
  through `$object::$prop` and `$className::$prop` are supported for receivers
  that evaluate to current objects or declared class-name strings, using the
  same storage, inherited-property lookup, and visibility rules. Typed
  properties, computed static property names beyond direct `::$name` tokens,
  storage-removing static-property unset, object/class-string receiver
  compound assignment, increment/decrement, `isset`, `empty`, `??`, `??=`,
  and `static::$prop` outside method/static class context remain unsupported.
  `parent::method(...)` and `self::method(...)` calls are the supported magic
  receiver slices for visible non-static or static method dispatch from active
  class context; non-static methods still require current `$this`.
  Public, same-class private, and protected same-class/child instance method
  dispatch supports static method names, inherited method lookup, and scoped
  `$this` binding. Named `ClassName::method(...)`, dynamic
  `$object::method(...)`/`$className::method(...)`, `self::method(...)`, and
  `parent::method(...)` static method dispatch is supported for the current
  visible declared/inherited static-method subset.
  Dynamic instance property names are supported only for existing public slots
  on current object values, public dynamic slots on `stdClass`, and public
  dynamic slots on the WordPress `wpdb` compatibility class, using string or
  integer property-name values. The parser accepts both
  `$object->$name` and braced `$object->{$expr}` forms in the current read and
  direct-variable-root write subset. Keyword-named direct properties are
  accepted after `->`; keyword method calls are still rejected with an explicit
  parse diagnostic. Dynamic methods, computed dynamic static property names
  beyond direct `::$name` tokens, non-public dynamic property access, magic
  property hooks, dynamic property-name
  `isset`/`empty`/`??`/`??=`, compound assignment,
  increment/decrement, string interpolation, missing-property creation outside
  `stdClass` and the bounded `wpdb` compatibility class, `#[AllowDynamicProperties]`
  attribute semantics, and exact PHP dynamic-property notices/deprecations
  remain unsupported. Non-public
  property/constructor visibility context beyond the current slice, static
  storage beyond direct static property reads/writes, broader class constant
  semantics, clone behavior beyond the current shallow property-slot copy, `__clone`,
  typed/default property compatibility, broader
  `parent::`/`self::`/`static::`, broader inheritance/interface relationship checks,
  namespace/autoload-aware class resolution, aliases and imports for class
  names, built-in/internal/extension class entries beyond the current
  metadata-only `Exception` and `stdClass` seeds for `get_declared_classes`,
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
  Direct `unset($array[$key])`, nested `unset($array[$outer][$inner])`, and
  nested object-property array-offset unset forms such as
  `unset($object->items[$outer][$inner])` remove matching entries from existing
  arrays, preserve the insertion order of remaining entries, do not rewind the
  next append key, treat missing keys and missing/`null` paths as no-ops, and
  treat undefined or `null` target variables/properties as no-ops. Multiple
  supported `unset(...)` operands execute left to right, including any
  array-offset key expressions. Existing non-array targets or intermediates
  fail with a stable invalid-array-access diagnostic.
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
  available through string-valued dynamic function calls. `current($array)`
  returns the value at the current cursor for the current ordered array model,
  initially the first inserted value, and returns `false` for empty or
  exhausted arrays. It is available through string-valued dynamic function
  calls.
  `next($array)` advances the current array cursor and returns the next value
  or `false` past the last element for direct variable arrays and the reached
  direct object-property array-offset shape. Full PHP internal pointer
  semantics, `reset()`/`end()`/`prev()` interaction, object operands,
  value-only dynamic calls, broad lvalue targets, references/copy-on-write,
  exact warnings, and native lowering remain unsupported.
  `array_pop($array)` removes and returns the last inserted value for direct
  variable arrays, returns `null` for empty arrays, updates the current cursor
  when needed, and follows the reached PHP append-index behavior after popping
  the last integer key. Non-variable targets, object-property array targets,
  value-only dynamic calls, broad by-reference handling,
  references/copy-on-write, exact warnings, and native lowering remain
  unsupported.
  `array_is_list($array)`
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
  dynamic function calls. `ksort($array, SORT_NUMERIC)` sorts direct variable
  arrays in place by numeric key and returns `true`; direct object-property
  array targets such as `ksort($object->callbacks, SORT_NUMERIC)` use the
  visible property path. Keys and values are preserved. Other sort flags,
  natural/locale sorts, broad key comparison, broad by-reference argument
  handling, exact diagnostics, and native lowering remain unsupported.
  `foreach ($array as $value)` iterates array values in
  insertion order over a snapshot of the current entries and writes the current
  value to the direct loop variable in the active scope. `foreach ($array as
  $key => $value)` additionally writes the current integer or string key as an
  `int` or `string` value to the direct key loop variable. By-reference value
  iteration forms parse but remain runtime boundaries; they do not create
  aliases, mutate array slots by reference, or model copy-on-write. Missing key reads
  still fail with a stable runtime error instead of PHP's
  warning-and-`null` recovery. Array truthiness, `count`, `array_key_exists`,
  `array_key_first`, `array_key_last`, `current`, `next`, `array_is_list`, `array_values`,
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
  runtime error. Truthiness is implemented for current scalar, array, and
  object values.
- Cast expressions: `(string)` is implemented for the current scalar/null value
  model. It converts `null` and `false` to `""`, `true` to `"1"`, integers to
  decimal strings, floats through the current PHP-style float formatter, and
  strings unchanged. `(int)`/`(integer)` is implemented for the current
  scalar/null value model: `null` and `false` become `0`, `true` becomes `1`,
  integers are unchanged, finite in-range floats truncate toward zero,
  well-formed numeric strings convert through the current numeric parser,
  bounded leading-numeric string prefixes with optional sign, decimal point,
  and exponent convert through the same integer/finite-float cast path, and
  empty or non-numeric strings become `0`. `(bool)`/`(boolean)` is implemented
  over the current value model using current PHP-shaped truthiness: `null`,
  `false`, integer/float zero, `""`, `"0"`, and empty arrays are false; other
  current scalars, non-empty arrays, and current objects are true.
  `(float)`/`(double)` is implemented for the current scalar/null value model:
  `null` and `false` become `0.0`, `true` becomes `1.0`, integers convert to
  floats, floats are unchanged, well-formed finite numeric strings convert
  through the current float parser, and empty or non-numeric strings become
  `0.0`. `(array)` is implemented for the current null/scalar/array subset:
  `null` becomes an empty array, booleans/integers/floats/strings become a
  one-element array at key `0`, and arrays are unchanged. Array/object behavior
  for `(string)`, `(int)`, and `(float)`, object-to-array property
  materialization and mangled visibility keys, Closure object array casts,
  exact PHP warning/recovery behavior for leading-numeric strings, numeric
  grammar outside the current bounded prefix scanner, non-finite or
  out-of-range float cast behavior, resources, `(real)`, `(object)`,
  `(unset)`, and `(binary)` cast forms, exact PHP diagnostics, and native
  lowering remain unsupported.
- Scalar comparisons: loose equality and relational operators are implemented
  for the current scalar values using PHP 8-style behavior for booleans,
  numeric strings, non-numeric strings, empty strings, `null`, integers, and
  floats. Strict identity operators `===` and `!==` execute for the current
  scalar values with type-and-value semantics and no numeric/string coercion,
  for object values by current object handle identity, and for arrays by
  comparing the same ordered key/value pairs recursively with strict value
  semantics. This is not PHP's full comparison matrix: resources, references,
  copy-on-write identity, recursive arrays, Closure object identity, exact PHP
  object comparison behavior beyond handle identity, and edge cases around
  `NAN`/`INF` and PHP-version-specific float string precision are not covered.
  Object loose comparisons in `phpc run` fail with explicit
  unsupported-comparison runtime errors.
- Conditionals: statement-form `if` supports zero or more `elseif` clauses and
  an optional `else` clause over the current expression and truthiness subset.
  Branch bodies may be brace blocks or single statements. Alternate
  `if`/`elseif`/`else` colon/`endif` conditional syntax executes through the
  same `Stmt::If` runtime path, including nested alternate conditionals.
  Malformed alternate conditional diagnostics, mixed brace/colon recovery,
  source mapping edge cases, and native conditional lowering remain
  unsupported.
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
  warning-and-break behavior. Positive integer literal loop-depth arguments
  such as `break 2;` and `continue 2;` are supported by consuming one active
  loop or switch level at a time; `continue 2;` can target an outer loop from
  inside a switch. Dynamic depth expressions, zero/negative depths, too-large
  depths, exact PHP diagnostics, and native lowering remain unsupported.
  `phpc run` also has an opt-in execution-step budget through
  `PHPC_MAX_EXECUTION_STEPS`. When the budget is exhausted during statement
  execution or empty loop-body iteration, it reports the last source location
  and current function context. This budget does not count parser work,
  declaration registration, or native lowering.
  `PHPC_TRACE_INCLUDES=1` emits include/require target paths to stderr before
  each target is parsed and executed; this is an operational trace facility,
  not PHP-visible output.
  `PHPC_TRACE_PARSE=1` emits parser frontier lines for top-level statements,
  class/interface/enum members, and block statements; this is also an
  operational trace facility and not PHP-visible output.
  Exception syntax is rejected separately at parse time, and native lowering is
  not implemented.
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
  unsupported constant introspection names outside the current unqualified and
  qualified string-name slices and non-string name arguments,
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
  unsupported `get_called_class()` calls outside method or static class
  context, non-object `spl_object_id` operands, non-object `spl_object_hash`
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
  `strtolower`, `trim`, `ltrim`, `rtrim`, `str_contains`, `str_ends_with`, `strpos`, `substr`, `substr_count`, `preg_match`, `preg_replace`, `preg_split`, `preg_replace_callback`,
  `error_reporting`, `min`, `rand`, `uniqid`, `hash_hmac`, `dirname`, `file_exists`,
  `is_dir`, `is_readable`, `register_shutdown_function`, `set_error_handler`, `restore_error_handler`, `date_default_timezone_set`,
  `mysqli_connect`, `mysqli_real_connect`, `mysqli_get_server_info`, `mysqli_get_host_info`,
  `mysqli_set_charset`, `mysqli_query`, `mysqli_errno`, `mysqli_error`, `mysqli_affected_rows`,
  `mysqli_insert_id`, `mysqli_ping`, `mysqli_select_db`, `mysqli_real_escape_string`,
  `mysqli_fetch_object`,
  `mysqli_fetch_assoc`, `mysqli_fetch_row`, `mysqli_fetch_array`,
  `mysqli_fetch_field`, `mysqli_num_fields`, `mysqli_num_rows`,
  `mysqli_data_seek`,
  `mysqli_free_result`, `mysqli_more_results`, `mysqli_next_result`,
  `mysqli_report`, `mysqli_init`,
  `compact`, `array_change_key_case`, `array_column`, `array_is_list`,
  `array_count_values`, `array_sum`, `array_product`, `array_reduce`, and
  `array_filter`, fold to `true`, and missing names fold to `false`.
  Direct `extension_loaded($name)` calls with already-lowerable string names
  fold against the same bounded compatibility registry: `json` and `hash`
  fold to `true`, while other names fold to `false`. Native code does not
  model host extension discovery, ini state, dynamic module loading, or
  extension side effects.
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
  built-in constant table. Qualified names such as
  `\Sodium\CRYPTO_AUTH_BYTES` and `Sodium\CRYPTO_AUTH_BYTES` remain rejected
  in native lowering, and interpolated string-name operands remain rejected
  before folding so native output cannot erase the runtime lookup boundary.
  Exact `CASE_LOWER`, `CASE_UPPER`,
  `ARRAY_FILTER_USE_BOTH`, `ARRAY_FILTER_USE_KEY`, `PREG_SPLIT_DELIM_CAPTURE`, `SORT_REGULAR`,
  `SORT_NUMERIC`, `SORT_STRING`, `PHP_VERSION_ID`, `PHP_VERSION`, and
  `PHP_INT_MAX` names
  fold to true;
  other supported unqualified names fold to false. The Milestone 569 and 573
  snapshots cover the `SORT_REGULAR` and `SORT_NUMERIC` additions without
  broadening native constant values, runtime-defined constant lookup, dynamic
  calls, arrays, objects, or exact native PHP error behavior. A
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
  arity/type diagnostics, direct `assert(...)`, unsupported `defined(...)` names, dynamic
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
  Native namespace declarations, namespace imports, namespace-qualified names,
  and namespace-aware name resolution are rejected before scalar folding or
  backend execution until generated code has native symbol tables, namespace
  context, aliases/imports, fallback function/constant lookup,
  class/autoload lookup, and exact native error behavior.
  Native arrays, array literals, array indexing, array assignment, list
  destructuring assignment, `foreach` array iteration, array offset unset, and
  array builtin function calls are
  rejected before body, operand, argument, or callback lowering with a specific
  codegen diagnostic until generated code has native array storage layout, key
  normalization, copy-on-write containers, references, callback dispatch, and
  exact native error objects.
  Native `require` statements and other include/require forms are rejected
  before file loading until generated code has multi-file loading, source-map
  handoff, caller-scope effects, declaration registration, include return
  values, and exact native error objects.
  Native `if`/`elseif`/`else`, including alternate colon/`endif` syntax,
  `while`, `for`, `do ... while`, `switch`, `break`, and `continue` are
  rejected before condition, body, case, or loop-control lowering with a
  specific codegen diagnostic until generated code has PHP truthiness, branch
  layout, loop control flow, switch fallthrough, references/copy-on-write
  side-effect behavior, and exact native error objects.
  Native compound assignment, null coalescing assignment,
  increment/decrement, assignment expressions, direct variable unset, object
  property unset, static property unset, and multiple-operand unset are
  rejected before operand or mutation-target lowering with a specific codegen
  diagnostic until generated code has read-modify-write ordering, null-aware
  mutation, unset symbol-table effects, references/copy-on-write, and exact
  native error objects.
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
  including optional trailing commas in argument lists. Top-level and
  namespace-scoped function declarations register under their resolved names in
  the current one-unbracketed namespace slice. Unqualified direct calls inside a
  namespace first look for a same-namespace function and then fall back to the
  global builtin/user-function table.
  Dynamic function calls are supported only when the callee expression evaluates
  to a string that case-insensitively resolves exactly to a user-defined function or to
  one of the documented callable builtins: `strlen`, `strtolower`, `trim`, `ltrim`, `rtrim`, `strcasecmp`,
  `str_contains`, `str_ends_with`, `strpos`, `substr`, `substr_count`, `preg_match`, `preg_replace`, `preg_split`, `preg_replace_callback`, `str_replace`, `error_reporting`,
  `sprintf`, `vsprintf`, `call_user_func`, `call_user_func_array`, `implode`, `file_exists`, `is_dir`, `abs`,
  `microtime`, `ini_get`, `min`, `count`, `compact`,
  `array_key_exists`, `array_key_first`, `array_key_last`, `current`, `next`, `array_is_list`,
  `array_values`, `array_keys`, `array_reverse`, `array_slice`, `array_chunk`,
  `array_pad`, `array_merge`, `array_replace`, `array_combine`, `define`,
  `constant`, `defined`,
  `array_intersect_key`, `array_diff_key`, `array_diff`, `array_intersect`,
  `array_unique`, `array_flip`, `array_fill_keys`, `array_count_values`,
  `array_sum`, `array_product`, `array_reduce`, `array_filter`, `array_map`,
  `array_unshift`, `array_pop`, `ksort`, `in_array`, `array_search`, `rand`, `uniqid`, `hash_hmac`, `gettype`, `is_null`, `is_bool`, `is_int`,
  `is_integer`, `is_long`, `is_float`, `is_double`, `is_string`, `is_array`,
  `is_scalar`, `is_numeric`, `is_countable`, `is_iterable`, `is_callable`,
  `function_exists`, `dirname`, `extension_loaded`, `mysqli_connect`,
  `mysqli_real_connect`, `mysqli_get_server_info`, `mysqli_query`,
  `mysqli_select_db`, `mysqli_real_escape_string`, `mysqli_report`,
  `mysqli_init`, `header`,
  `header_remove`, `headers_sent`,
  `get_class`, `is_object`, `get_debug_type`,
  `class_exists`, `interface_exists`, `trait_exists`, `enum_exists`,
  `property_exists`, `method_exists`, `get_class_methods`, `get_class_vars`,
  `get_object_vars`, `get_mangled_object_vars`,
  `is_a`, `is_subclass_of`, `get_parent_class`, `get_declared_classes`,
  `get_declared_interfaces`, `get_declared_traits`, `get_called_class`,
  `spl_object_id`, `spl_object_hash`, `var_dump`, or `print_r`.
  Dynamic string names do not apply lexical namespace expansion or imports.
  The `define`, `constant`, and `defined` names resolve through the documented
  runtime constant path. Unresolved names fail with a stable undefined-function
  runtime error, and non-string callees fail with a stable unsupported-call
  runtime error. Required parameters, optional trailing commas after the final
  real parameter, and trailing default parameter values are supported.
  Defaults may use the current constant-expression subset: `null`, booleans,
  integers, floats, strings, short and long arrays with supported keys, unary
  expressions, binary expressions over those values, bare references to
  unqualified constants that are defined in the current runtime constant table
  before the omitted argument is bound, and `self::CONST` defaults in class
  methods. `self::CONST` defaults resolve through the declaring method class
  context when the omitted argument is bound, including inherited method
  dispatch and same-class private constants. The exact uppercase built-in
  `ARRAY_FILTER_USE_KEY` and `ARRAY_FILTER_USE_BOTH` constants are also
  accepted in default expressions. String-valued dynamic calls accept the same
  optional trailing comma syntax after the final positional argument. Omitted
  arguments bind to their defaults;
  missing constant references fail with a stable undefined-constant runtime
  diagnostic; calls outside the supported required-to-total arity range
  fail with a stable arity diagnostic. Each user-function call gets a fresh
  local scope. Parameters and local assignments shadow global variables without
  mutating them, and functions do not import top-level variables implicitly.
  Top-level `global $name, ...;` declarations execute as no-op/import-compatible
  statements. Function-scope `global` declarations still fail with a stable
  runtime error because reference-backed global imports are not implemented.
  Recursive user-function calls are supported until the fixed 128-frame
  user-function call-depth guard is reached.
  That guard is a project-specific runtime diagnostic, not PHP's native stack or
  memory exhaustion behavior; it is not configurable and does not produce stack
  traces. Forward constant references at omitted-argument binding time,
  namespace-aware constants, `ClassName::CONST`, `parent::CONST`,
  `static::CONST`, class-name constants such as `self::class`, dynamic
  defaults, references/copy-on-write behavior, and native lowering for defaults
  are not implemented. `self::CONST` defaults outside class method context parse
  but fail with a stable runtime diagnostic when an omitted argument is bound.
  Native lowering for user-function declarations and returns is
  explicitly rejected until function symbol tables, stack-frame layout, default
  parameter binding, recursion guards, return-value flow, and exact native
  error behavior exist. Non-constant defaults such as variables, calls,
  dynamic calls, and indexed reads are rejected by the parser. Required
  parameters after default parameters are also rejected instead of modeling
  PHP's deprecation and implicit-required behavior. Empty parameter slots such
  as `function f(,)` remain rejected. Parameter and return type declarations,
  including nullable, union, intersection, and namespace-qualified names, are
  accepted as syntax-only metadata so WordPress-style helper signatures can be
  registered. Invoking a function with parameter/return type annotations fails
  with a stable runtime error because type enforcement, coercion, exact
  `TypeError` behavior, `strict_types`, variance, and reflection metadata are
  not implemented. Reference parameter declarations are also accepted as
  metadata, but invoking those functions fails with a stable runtime error until
  reference binding exists. Anonymous closure syntax with parameter lists,
  optional return-type metadata, block bodies, and `use (...)` capture lists is
  parsed so containing functions can be registered, including by-reference
  capture syntax such as `use (&$name)`. Evaluating an anonymous closure
  expression, a `static function (...) { ... }` expression, or an arrow
  function expression creates an inert runtime closure value that can be
  stored, read, and tested for truthiness. Explicit `use (...)` capture names
  are looked up at closure creation and the current values are stored on the
  inert closure value; by-reference captures record the requested capture mode
  but do not create alias cells. Closure bodies are not executed. Arrow
  implicit capture binding and execution, closure invocation, `$this` binding,
  true by-reference capture aliasing, copy-on-write, static closure binding
  semantics, callback integration beyond registration validation, exact PHP
  `Closure` object behavior, and native lowering are unsupported. Arrow
  function syntax `fn (...) => expr` is parsed as a closure-shaped expression
  with a synthetic return body.
  Variadic parameters
  and argument unpacking, reference returns, reference expressions, named
  arguments, first-class callable syntax such as `strlen(...)` and
  `$callback(...)`, empty call arguments, and `declare(strict_types=1)` are
  rejected with stable parse diagnostics. Function-local `static` declarations
  are supported for the current bounded direct-variable storage slice:
  `static $name;` and `static $name = value;` initialize per-function storage
  once, materialize the value into the active function scope on each call, and
  preserve later direct-variable writes across calls. Initializers use the
  documented constant-expression/default-value subset. Dynamic initializers,
  references, variable variables, recursion/reentrancy edge behavior,
  included-file edge cases, exact PHP diagnostics, reflection behavior, and
  native lowering remain unsupported. The `__LINE__` magic constant evaluates
  to the source line of the
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
  string outside a function. The `__METHOD__` magic constant evaluates to the
  current `Class::method` name in ordinary expressions and default parameter
  values when a method class context exists, to the current function name in
  function context, and to an empty string outside a function. `__CLASS__`
  evaluates to the current class name when a method class context exists, and
  to an empty string outside class context. `__TRAIT__` fails with a stable
  parse diagnostic tied to the
  current missing trait declaration/use and trait-context tracking boundary.
  `__NAMESPACE__` fails with a stable parse diagnostic tied to the current
  missing namespace-aware name-resolution boundary. Nullable, union, and intersection
  types, `mixed`, `void`/`never`, class/interface type names, coercive versus
  strict typing, variance, static local behavior outside the bounded
  declaration/default subset, reference-backed static locals,
  recursion/reentrancy edge behavior, canonical absolute
  `__FILE__`/`__DIR__` paths matching PHP exactly, eval/include source mapping,
  namespace and trait magic constants, closure invocation and capture binding,
  closure function-name context, magic
  constant native lowering, array callables, object/method callables,
  first-class callable syntax, namespace-qualified callable
  resolution, autoload interaction, and native lowering for type declarations
  are unsupported.
- Builtins: `strlen`, `strtolower`, `trim`, `ltrim`, `rtrim`, `strcasecmp`, `str_contains`,
  `str_ends_with`, `strpos`, `substr`, `substr_count`, `str_replace`, `sprintf`, `vsprintf`,
  `call_user_func`, `call_user_func_array`, `implode`, `file_exists`, `is_dir`, `is_readable`, `register_shutdown_function`, `set_error_handler`, `restore_error_handler`, `date_default_timezone_set`, `abs`, `microtime`, `ini_get`, `min`, `isset`, `empty`, `count`,
  `define`, `constant`,
  `defined`, `array_key_exists`, `array_key_first`, `array_key_last`,
  `current`, `array_is_list`, `array_values`, `array_keys`, `array_reverse`,
  `array_slice`, `array_chunk`, `array_pad`, `array_merge`, `array_replace`,
  `array_combine`, `array_intersect_key`, `array_diff_key`, `array_diff`,
  `array_intersect`, `array_unique`, `array_flip`, `array_fill_keys`,
  `array_count_values`, `array_sum`, `array_product`, `array_reduce`,
  `array_filter`, `array_map`, `array_unshift`, `array_pop`, `next`, `ksort`, `in_array`,
  `array_search`, `gettype`,
  `is_null`, `is_bool`, `is_int`, `is_integer`, `is_long`, `is_float`,
  `is_double`, `is_string`, `is_array`, `is_scalar`, `is_numeric`,
  `is_countable`, `is_iterable`, `is_callable`, `function_exists`, `rand`,
  `uniqid`, `hash_hmac`,
  `dirname`, `extension_loaded`, `mysqli_connect`, `mysqli_real_connect`,
  `mysqli_get_server_info`, `mysqli_query`, `mysqli_errno`, `mysqli_error`,
  `mysqli_select_db`, `mysqli_real_escape_string`, `mysqli_report`, `mysqli_init`, `header`,
  `header_remove`, `headers_sent`, `assert`,
  `spl_autoload_register`, `get_class`, `is_object`, `get_debug_type`,
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
  `TypeError`/deprecation behavior are not implemented. `assert(...)` is a
  runtime-only builtin in this slice: truthy assertions return `true`, while
  failing assertions, assertion INI policy, callbacks, `AssertionError`,
  `Throwable` descriptions, exact warning/fatal behavior, PHP 8.3
  deprecations, partial-output behavior, and native lowering are not
  implemented. `extension_loaded`
  accepts string extension names, returns true for `json` and `hash` from the
  current bounded compatibility registry, returns false for other names, and
  rejects non-string names. Its native folding uses the same direct string-name
  registry for already-lowerable string names.
  `mysqli_connect`, `mysqli_real_connect`, `mysqli_get_server_info`,
  `mysqli_query`, `mysqli_errno`, `mysqli_error`, `mysqli_select_db`,
  `mysqli_real_escape_string`, `mysqli_fetch_object`,
  `mysqli_fetch_assoc`, `mysqli_fetch_array`, `mysqli_fetch_field`,
  `mysqli_num_fields`, `mysqli_free_result`, `mysqli_more_results`,
  `mysqli_next_result`, `mysqli_report`, and `mysqli_init` are recognized for function/callability
  metadata and dynamic lookup. `mysqli_real_connect(...)` executes only the
  current placeholder-handle success boundary,
  `mysqli_get_server_info(...)` returns only the current deterministic
  placeholder string, `mysqli_get_host_info(...)` returns only deterministic
  placeholder host metadata, `mysqli_set_charset(...)` returns only deterministic
  success for the current `utf8mb4` placeholder charset, and
  `mysqli_query(...)` returns only the current false
  SQL-mode-probe, true charset-setup, WordPress empty-options-query, and exact
  synthetic empty-result boundaries;
  `mysqli_errno(...)` and `mysqli_error(...)` expose only clean placeholder
  diagnostics; `mysqli_affected_rows(...)` and `mysqli_insert_id(...)` expose
  only deterministic zero clean-state metadata; `mysqli_ping(...)` returns only
  deterministic placeholder liveness success; `mysqli_select_db(...)`
  returns only deterministic success for the placeholder handle;
  `mysqli_real_escape_string(...)` returns
  only deterministic escaping over the placeholder handle and current
  scalar/null string-convertible values; direct `mysqli_connect(...)` calls
  are still a stable unsupported runtime boundary, and direct native
  `mysqli_connect(...)`/`mysqli_real_connect(...)`/
  `mysqli_get_server_info(...)`/`mysqli_get_host_info(...)`/
  `mysqli_set_charset(...)`/`mysqli_query(...)`/
  `mysqli_errno(...)`/`mysqli_error(...)`/
  `mysqli_affected_rows(...)`/`mysqli_insert_id(...)`/`mysqli_ping(...)`/
  `mysqli_select_db(...)`/`mysqli_real_escape_string(...)`/
  `mysqli_fetch_object(...)`/`mysqli_fetch_assoc(...)`/
  `mysqli_fetch_row(...)`/`mysqli_fetch_array(...)`/
  `mysqli_fetch_field(...)`/`mysqli_num_fields(...)`/
  `mysqli_num_rows(...)`/`mysqli_data_seek(...)`/
  `mysqli_free_result(...)`/`mysqli_more_results(...)`/
  `mysqli_next_result(...)`/`mysqli_report(...)`/`mysqli_init(...)` calls
  still reject under the function-call boundary.
  `header` accepts the same current no-op header subset as the builtin section
  above; direct native `header(...)` calls still reject under the function-call
  boundary, while native function-table introspection recognizes the name.
  `header_remove` accepts the same current no-op removal subset as the builtin
  section above; direct native `header_remove(...)` calls still reject under the
  function-call boundary, while native function-table introspection recognizes
  the name.
  `headers_sent` accepts the same current no-argument `false` subset as the
  builtin section above; direct native `headers_sent(...)` calls still reject
  under the function-call boundary, while native function-table introspection
  recognizes the name.
  `abs` accepts the same current integer and finite-float subset as the builtin
  section above; direct native `abs(...)` calls still reject under the
  function-call boundary, while native function-table introspection recognizes
  the name.
  `microtime` accepts the same current `microtime(true)` subset as the builtin
  section above; direct native `microtime(...)` calls still reject under the
  function-call boundary, while native function-table introspection recognizes
  the name.
  `ini_get` accepts the same current deterministic registry subset as the
  builtin section above; direct native `ini_get(...)` calls still reject under
  the function-call boundary, while native function-table introspection
  recognizes the name.
  `strtolower` accepts the same current scalar/null string-convertible subset
  as the builtin section above; direct native `strtolower(...)` calls still
  reject under the function-call boundary, while native function-table
  introspection recognizes the name.
  `trim` accepts the same current default-mask scalar/null string-convertible
  subset as the builtin section above; direct native `trim(...)` calls still
  reject under the function-call boundary, while native function-table
  introspection recognizes the name.
  `ltrim` accepts the same current default-mask and literal-character-mask
  scalar/null string-convertible subset as the builtin section above; direct
  native `ltrim(...)` calls still reject under the function-call boundary,
  while native function-table introspection recognizes the name.
  `rtrim` accepts the same current default-mask and literal-character-mask
  scalar/null string-convertible subset as the builtin section above; direct
  native `rtrim(...)` calls still reject under the function-call boundary,
  while native function-table introspection recognizes the name.
  `array_unshift` accepts the same direct-variable ordered-array mutation
  subset as the builtin section above; direct native `array_unshift(...)`
  calls still reject under the function-call boundary, while native
  function-table introspection recognizes the name.
  `array_pop` accepts the same direct-variable ordered-array mutation subset
  as the builtin section above; direct native `array_pop(...)` calls still
  reject under the function-call boundary, while native function-table
  introspection recognizes the name.
  `next` accepts the same direct array-pointer mutation subset as the builtin
  section above; direct native `next(...)` calls still reject under the
  function-call boundary, while native function-table introspection recognizes
  the name.
  `current` accepts the same current ordered-array first-value subset as the
  builtin section above; direct native `current(...)` calls still reject under
  the function-call boundary, while native function-table introspection
  recognizes the name.
  `str_contains` accepts the same current scalar/null string-convertible
  haystack and needle subset as the builtin section above; direct native
  `str_contains(...)` calls still reject under the function-call boundary,
  while native function-table introspection recognizes the name.
  `str_ends_with` accepts the same current scalar/null string-convertible
  haystack and needle subset as the builtin section above; direct native
  `str_ends_with(...)` calls still reject under the function-call boundary,
  while native function-table introspection recognizes the name.
  `strpos` accepts the same current scalar/null string-convertible haystack and
  needle subset plus an optional integer offset as the builtin section above;
  direct native `strpos(...)` calls still reject under the function-call
  boundary, while native function-table introspection recognizes the name.
  `substr` accepts the same current scalar/null string-convertible input,
  integer offset, and optional integer length subset as the builtin section
  above; direct native `substr(...)` calls still reject under the function-call
  boundary, while native function-table introspection recognizes the name.
  `substr_count` accepts the same current scalar/null string-convertible
  haystack and needle subset plus optional integer offset and length arguments
  as the builtin section above; direct native `substr_count(...)` calls still
  reject under the function-call boundary, while native function-table
  introspection recognizes the name.
  `min` accepts the same current integer-only variadic subset as the builtin
  section above; direct native `min(...)` calls still reject under the
  function-call boundary, while native function-table introspection recognizes
  the name.
  `sprintf` and `vsprintf` accept the same current bounded format subset as the
  builtin section above; direct native `sprintf(...)` and `vsprintf(...)` calls
  still reject under the function-call boundary, while native function-table
  introspection recognizes both names.
  `strcasecmp` accepts the same current scalar/null string-convertible subset
  as the builtin section above; direct native `strcasecmp(...)` calls still
  reject under the function-call boundary, while native function-table
  introspection recognizes the name.
  `str_replace` accepts the same current scalar/null string-convertible and
  direct-variable count-output subset as the builtin section above; direct
  native `str_replace(...)` calls still reject under the function-call
  boundary, while native function-table introspection recognizes the name.
  `call_user_func` accepts the same current string-callable subset as the
  builtin section above; direct native `call_user_func(...)` calls still reject
  under the function-call boundary, while native function-table introspection
  recognizes the name.
  `call_user_func_array` accepts the same current string/array callable and
  integer-keyed positional argument-array subset as the builtin section above;
  direct native `call_user_func_array(...)` calls still reject under the
  function-call boundary, while native function-table introspection recognizes
  the name.
  `implode` accepts the same current scalar/null array-value subset as the
  builtin section above; direct native `implode(...)` calls still reject under
  the function-call boundary, while native function-table introspection
  recognizes the name.
  `dirname` accepts the same current lexical Unix-style local path subset as
  the builtin section above; direct native `dirname(...)` calls still reject
  under the function-call boundary.
  `spl_autoload_register` accepts closure and string callbacks in `phpc run`
  without storing or invoking them; direct native calls still reject under the
  function-call boundary, while native function-table introspection recognizes
  the name.
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
  string interface names and perform case-insensitive lookup against
  interfaces declared in the current parsed program; the autoload flag accepts
  current bool-like scalar values and does not trigger autoloading.
  `trait_exists($name)` and `trait_exists($name, $autoload)` accept string
  trait names and perform case-insensitive lookup against empty top-level
  traits declared in the current parsed program; the autoload flag accepts
  current bool-like scalar values and does not trigger autoloading.
  `enum_exists($name)` and `enum_exists($name, $autoload)` accept string enum
  names and perform case-insensitive lookup against top-level unit enums
  declared in the current parsed program; the autoload flag accepts current
  bool-like scalar values and does not trigger autoloading. `class_exists()`
  also reports true for declared enums.
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
  `get_declared_interfaces()` returns a zero-indexed array containing only the
  current parsed program's declared interface names in declaration order.
  Built-in/internal interface entries are not represented.
  `get_declared_traits()` returns a zero-indexed array containing only the
  current parsed program's empty top-level trait names in declaration order.
  Built-in/internal trait entries are not represented.
  `get_called_class()` is recognized as a zero-argument callable and returns
  the current called class while executing in current instance and static
  method contexts, including string-valued dynamic calls. Outside method or
  static class context it fails with a stable unsupported-call diagnostic.
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
  unqualified and qualified string names matching the current identifier
  segment subset, without a leading global namespace separator, and stores
  `null`, booleans, integers, floats, strings, and arrays containing only
  supported constant values. `constant($name)` accepts unqualified names and
  qualified lookup names with an optional leading global namespace separator,
  returning a cloned value from the runtime-defined table or from the exact
  built-in `ARRAY_FILTER_*` and `SORT_*` slice. Bare reads still only use the
  current unqualified runtime-defined and built-in constant table.
  `defined($name)` returns true for supported unqualified or qualified names
  present in that current table and false for supported names that are
  missing, including absent extension-style names such as
  `\Sodium\CRYPTO_AUTH_BYTES`.
  Top-level single and grouped `const NAME = value;` declarations accept
  unqualified names at global scope and resolve those names under the active
  unbracketed namespace. Their current constant-expression subset is `null`,
  booleans, integers, floats, strings, arrays, unary expressions, binary
  expressions over those values, bare references to previously defined
  unqualified constants and the current exact built-in `ARRAY_FILTER_*` and
  `SORT_*` constants, and named class constants such as `Box::VALUE`. Grouped
  declarations execute left to right, so references to earlier declarators in
  the same group work and duplicate diagnostics point to the later duplicate
  declarator in the current group. Duplicate definitions, redefinition of the
  built-in constants, forward or otherwise undefined const declaration
  references, non-string or unsupported names, unsupported object-containing
  values, unknown `constant(...)` names, non-string or unsupported
  `defined(...)` names, unknown bare constants, and the legacy third
  `define(...)` flag fail with stable diagnostics. Magic constants are rejected
  by the parser before runtime constant lookup. Constant names that are lexed
  as language keywords or literals cannot be read bare, and bare namespace
  constant fallback reads, full extension constant catalogs, host
  extension/module discovery, class-constant string lookup beyond declared
  `ClassName::CONST`/`\ClassName::CONST` names in the current loaded class
  metadata, nested declarations, dynamic declaration values,
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
  function calls. `ksort($array, SORT_NUMERIC)` sorts direct variable arrays
  and direct object-property arrays in place by numeric key while preserving
  keys and values. Other sort flags, natural/locale sorts, mixed non-numeric
  key comparison, broad by-reference argument handling, exact diagnostics, and
  native lowering remain unsupported. `isset` supports direct variable
  operands, direct array offset operands such as `isset($array[$key])` and
  nested direct-variable rooted array offset paths such as
  `isset($array[$outer][$inner])`, direct public object-property operands such
  as `isset($object->name)`, direct object-property array offset operands such
  as `isset($object->items[$outer][$inner])`, and supported static property
  operands such as `isset(ClassName::$prop)`.
  In active method context, direct private operands owned by the active
  declaring class and protected operands owned by the active class or an
  ancestor are also supported.
  `isset` can safely check undefined
  variables, missing/null array slots, missing/null intermediate array path
  entries, undefined array variables, non-array array targets, and undefined
  object-property targets. Non-variable array roots, append offset operands,
  object dimensions, dynamic property names, non-public property operands
  outside the current private/protected visibility context, complex lvalues,
  and general expression operands remain unsupported. `empty`
  supports one direct variable operand, direct nested array offset operands
  such as `empty($array[$outer][$inner])`, one direct public object-property
  operand such as `empty($object->name)`, direct object-property array offset
  operands such as `empty($object->items[$outer][$inner])`, or one supported
  static property operand such as `empty(ClassName::$prop)`. In active method
  context, direct private operands owned by the active declaring class and
  protected operands owned by the active class or an ancestor are also
  supported; undefined variables, missing array keys, missing/null/non-array
  intermediate array path entries, undefined array targets, non-array array
  targets, missing object properties, missing supported static properties,
  undefined object targets, and non-object property targets are treated as
  empty, and existing values use the current PHP truthiness rules. Nested
  offsets rooted in function calls or other non-direct expressions, dynamic
  property names, non-public property visibility context outside the current
  private/protected method context, append offset operands, complex lvalues,
  general expression operands, ArrayAccess, magic methods, and unsupported
  array-key coercions remain unsupported.
  `array_key_first`, `array_key_last`, `current`, `next`, `array_pop`, `array_is_list`, `array_values`,
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
- Object/class gaps: exact PHP nested and conditional declaration timing,
  redeclaration fatal text, class table ordering across all interleavings,
  source mapping for nested declarations loaded from included files,
  partial-output behavior on fatal declaration errors, unbraced nested class
  declarations, and constructor behavior beyond public/inherited public
  instance `__construct` and explicit parent calls,
  typed/default property compatibility,
  broader `parent::`/`self::`/`static::`, broader inheritance rules,
  interface constants, interface implementation enforcement, interface
  inheritance, built-in/internal interface catalogs,
  trait members, trait use inside classes,
  trait methods/properties/constants, trait conflict resolution, aliases,
  visibility changes, namespace-aware traits,
  backed enum declarations, enum case objects, backed enum values, enum
  methods, enum constants/properties, enum interface implementations,
  namespace-aware enum member access,
  abstract-method implementation enforcement, final inheritance/method
  enforcement, readonly class semantics, readonly properties, typed property
  storage and enforcement,
  property initialization rules beyond the current untyped constant-expression
  default subset, inheritance interactions,
  multiple properties in one declaration, per-property defaults in
  multi-property declarations, typed/multiple/final/interface/trait/enum class
  constants, typed static properties, static property storage removal, late
  static binding beyond the current `static::` method/property/constant and
  `new static` slices, magic methods, namespaces,
  autoloading, anonymous classes, attributes, reflection, dynamic properties
  beyond `stdClass` public slot materialization, dynamic property-name forms
  beyond existing public slots and `stdClass` including complex assignment
  roots, dynamic method names, protected method visibility outside
  same-class/child method contexts, non-public property access outside the
  current private/protected method context, broader
  constructor visibility context, static member
  execution through `::` except the current class-name, class-constant, and
  static-property slices, property assignment
  targets other than a direct variable, dynamic properties created outside
  declarations, autoload side effects from property introspection,
  object handle identity/aliasing,
  cloning, destructors, serialization hooks, visibility enforcement,
  broader `self`/`parent`/`static` behavior, object comparisons, full
  `instanceof` interface/class relationship metadata, object-to-string
  conversion, object callables, and native lowering are unsupported.
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
- Array gaps: array spread elements, reference array keys, short `[...]`
  destructuring assignment targets, expression-position `list(...)`, and
  keyed, nested, reference, or non-variable list targets are rejected with
  stable parse diagnostics. Array literal reference values are parsed and
  evaluated by current value only; real aliases, reference containers, and
  copy-on-write are not implemented. Object-property reference-assignment
  sources are parsed and copy current array/object values only; scalar sources,
  real aliases, reference containers, and copy-on-write are not implemented.
  Positional statement-form
  `list($a, $b) = expr;` is supported for direct variable targets and skipped
  slots only; exact PHP warning/notice emission for missing offsets and
  non-array right-hand sides is not implemented. `unset(...)` forms outside direct variables,
  direct/nested array-offset operands, direct/dynamic object-property operands,
  nested object-property array-offset operands, and static-property diagnostic operands,
  comma-separated `for` header expression lists,
  expression-form `do ... while`, expression-form `switch`, malformed
  alternate switch bodies, and exponentiation syntax `**`/`**=` are rejected
  with stable parse diagnostics; append-offset unset and complex mixed
  object/property/ArrayAccess unset operands are not implemented.
  Complex assignment lvalues outside the documented direct-variable,
  direct/nested array-offset, append/append-at-depth, direct object-property,
  and supported static-property target subset, nested/complex
  non-variable, dynamic object-property, or mixed object/ArrayAccess
  `isset(...)` array offset operands,
  nested/complex `empty(...)` array offset operands, native
  `isset($array[$key])` lowering, `$array[]` as a read expression, string
  offset access, by-reference `foreach`, object iteration, destructuring loop
  targets, array destructuring assignments with keyed, nested, reference,
  short-syntax, expression-position, or non-variable target semantics,
  references, copy-on-write containers, and
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
- include/require behavior outside the narrow local string-path statement and
  expression subset: include-path lookup, streams/URLs, `phar://`, autoload
  interaction, opcache behavior, declaration-order dependencies across
  required files, exact PHP warning/fatal recovery behavior, and native
  lowering remain unsupported
- `eval` execution; direct `eval(...)` currently fails with a stable parse
  diagnostic
- namespace forms outside the current one-unbracketed-namespace/simple-class-use
  plus same-namespace function slice: bracketed/global/multiple namespaces,
  namespace-scoped constants, grouped imports, function imports, constant
  imports, namespace-qualified function calls, `__NAMESPACE__`, string-name
  import expansion, autoload-aware lookup, and namespace-aware native lowering
- dynamic method names; `$object->$method()` currently fails with a stable
  parse diagnostic. Dynamic property-name support is limited to existing
  public slots, `stdClass` public dynamic slots, and the bounded WordPress
  `wpdb` dynamic table-name slot materialization path, without general
  `#[AllowDynamicProperties]`, magic methods, non-public dynamic access, or
  exact notice/deprecation behavior
- private instance method dispatch outside same-class method context,
  protected instance method dispatch outside same-class/child method context,
  non-static methods through dynamic static receivers, and `$this` outside
  instance method execution currently fail with stable runtime diagnostics
- non-public object property access and property writes to lvalues other than a
  direct variable
- constructor arguments for classes without a declared constructor, non-public
  constructors, and static constructors currently fail with stable runtime
  diagnostics
- unsupported class forms including nested/conditional declarations, broader
  inheritance rules beyond the current single-parent metadata chain, interface
  method enforcement, interface constants, interface inheritance,
  built-in/internal interface catalogs, trait
  declarations, enum declarations, enum cases/backing values/methods/interface
  implementation,
  autoload-triggered parent class resolution,
  typed property storage/enforcement, non-constant instance property defaults, multiple properties in
  one declaration, per-property defaults in multi-property declarations,
  typed/static/multi-declarator class constants, typed static properties,
  storage-removing static-property unset,
  and anonymous classes
- object receiver class constants, `$object::class`, and broader `static::`
  member forms through `::`
- variable variables; `$$name` and `${...}` are rejected with a stable lex
  diagnostic rather than executed
- full `global`/`$GLOBALS` semantics beyond direct string-keyed `$GLOBALS`
  root-symbol reads/writes: reference-backed aliases, recursive `$GLOBALS`
  materialization, dynamic global names, superglobals, included-file scope
  interactions, copy-on-write, exact warning/notice behavior, and native
  lowering
- default parameter values outside the documented constant-expression,
  unqualified constant-reference, and class-method `self::CONST` subset
- required parameters after default parameters
- variadic parameters outside the bounded final-parameter by-value slice, and
  variadic argument unpacking
- provided reference parameter invocation outside direct variable arguments,
  true alias binding for reference parameters, reference returns, executable
  reference assignments, reference assignments from nested
  offsets/properties/static members/function calls, by-reference iteration, and
  broader by-reference calls
- parameter/return type enforcement, coercion, exact `TypeError` behavior,
  `strict_types`, variance, reflection metadata, and native lowering for type
  declarations
- function-local static behavior outside the bounded runtime slice, including
  dynamic initialization expressions, references, variable variables,
  recursion/reentrancy edge behavior, included-file edge cases, exact PHP
  diagnostics, reflection behavior, and native lowering
- magic constants other than `__LINE__`, `__FILE__`, `__DIR__`,
  `__FUNCTION__`, `__CLASS__`, and `__METHOD__`, such as `__TRAIT__` and
  `__NAMESPACE__`; `__TRAIT__` specifically fails because trait declarations,
  trait use, and trait-context tracking are not implemented, and
  `__NAMESPACE__` specifically fails because namespace-aware name resolution is
  not implemented. `__FUNCTION__`, `__CLASS__`, and `__METHOD__` are limited to
  current user-function and declared-method contexts plus top-level
  empty-string behavior; closure context, trait-method context,
  anonymous-class exact names, original-name/case fidelity beyond the current
  declaration metadata, and exact namespace/source mapping are not implemented.
  `__FILE__` currently
  reports the `phpc run` input path string, and `__DIR__` derives from that
  same path string; neither is guaranteed to match PHP's canonical absolute
  filename or directory in all entry paths. Native lowering rejects executable
  magic constants `__LINE__`, `__FILE__`, `__DIR__`, `__FUNCTION__`,
  `__CLASS__`, and `__METHOD__` with a specific codegen diagnostic until
  source mapping, path canonicalization, and function/class/method-context
  lowering exist.
- array literal spread elements and array literal reference keys
- `unset(...)` forms outside direct variables, direct/nested array offsets,
  direct/dynamic object properties, nested object-property array offsets, and
  static-property diagnostic operands, including append-offset unset and
  complex mixed object/property/ArrayAccess operands;
  these fail with stable parse diagnostics
- executable by-reference `foreach`, object iteration, destructuring loop
  targets, key-by-reference loop variables, and expression-form `foreach`
- comma-separated `for` initializer, condition, or increment expression lists;
  only zero or one expression or assignment is supported in each header slot
- expression-form `for`; `for` is only supported as a statement
- malformed alternate `if`/`elseif`/`else` colon/`endif` forms, mixed
  brace/colon conditional recovery, exact PHP diagnostics, source mapping edge
  cases, and native lowering
- expression-form `do ... while`; `do ... while` is only supported as a
  statement
- expression-form `switch`, malformed alternate colon/`endswitch` switch
  bodies, and `continue;` behavior inside switch
- dynamic, zero, negative, or too-large `break`/`continue` loop-depth
  arguments; only statement-form positive integer literal depths are
  implemented for active `while`, supported `for`, supported `do ... while`,
  supported array `foreach`, and supported `switch` control stacks
- native lowering for `if`/`elseif`/`else`, including alternate colon/`endif`
  syntax, `while`, `for`, `do ... while`, `switch`, `goto` labels, `break`,
  and `continue`; generated code currently rejects those forms before lowering
  conditions, bodies, cases, jumps, or loop-control flow
- native lowering for cast expressions; generated code currently rejects casts
  before implying PHP scalar conversion, diagnostics, allocation behavior,
  references/copy-on-write, or exact native error behavior
- exception execution beyond the current statement boundaries: reached
  `throw expr;` statements fail with a stable runtime diagnostic before
  evaluating the operand; reached `try` blocks execute only the normal no-throw
  path, skip catch bodies without a thrown exception, and run finally bodies
  after normal try completion. Throw expressions, malformed try syntax, and
  standalone `catch`/`finally` still fail with stable parse diagnostics.
  `Exception` is seeded as a metadata-only built-in class: `class_exists`,
  `get_declared_classes`, no-argument `new Exception()`, and user classes
  extending `Exception` work through the current object metadata model.
  `Throwable` interface metadata, `Exception` constructor state (`message`,
  `code`, previous exception), `Exception` methods such as `getMessage()`,
  stack unwinding, catch matching, catch variable binding, multi-catch
  semantics beyond parsed type lists, finally execution during exception/error
  unwinding, stack traces, exact native error objects, and native lowering do
  not exist yet.
- PHP 8 `match` expressions currently fail with a stable parse diagnostic
  before expression-form branching exists. Strict arm matching, default arms,
  exhaustiveness errors, thrown expressions inside arms, value evaluation
  order, exact native error objects, and native lowering are not implemented.
- `goto` support is bounded to the current statement-list runtime slice.
  Broader PHP behavior such as exact compile-time target validation, duplicate
  label diagnostics, jumps into nested blocks, cross-function jumps, included
  file label boundaries, `finally` interaction, and native lowering remains
  unsupported.
- heredoc/nowdoc string syntax supports the current unindented identifier-label
  subset. Heredoc behaves like double-quoted strings over the current
  interpolation parts, while nowdoc remains literal. Indentation stripping,
  quoted labels beyond the simple current form, arbitrary label whitespace,
  malformed-label recovery, exact diagnostics, and native lowering remain
  unsupported.
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
  `($object->property ??= expr)`, plus supported static-property mutation
  expressions such as `(ClassName::$prop += expr)`, `(self::$prop++)`, and
  `(parent::$prop ??= expr)`. They write the active scope's static variable,
  current ordered array offset, appended array slot, existing declared public
  property slot, or declared static property slot and return the assigned,
  updated, previous, or existing value according to the operator.
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
  expressions, dynamic property names beyond the current direct
  `$object->$name`/`$object->{$expr}` assignment slice, append-offset `??=`
  targets, reference
  assignment, copy-on-write container aliasing, exact native error objects, and
  native lowering are not implemented.
- Compound assignment is limited to direct static variables, direct
  array-variable offsets, direct public object properties, private properties
  in active declaring-class method context, protected properties owned by the
  active class or an ancestor, and declared static properties through
  `ClassName::$prop`, `self::$prop`, and `parent::$prop` over the current
  scalar/object value model.
  The
  read-modify-write operation reuses the existing PHP-shaped scalar arithmetic,
  modulo, bitwise/shift, and string concatenation helpers, so undefined
  left-hand variables, missing array keys, missing object properties, non-array
  targets, non-object property targets, non-public properties outside the
  current private/protected visibility context, division by
  zero, modulo by zero, non-numeric strings, arrays, and objects as operand
  values fail through existing stable runtime diagnostics.
  Statement forms, expression forms such as `($name += expr)`,
  `($array[$key] += expr)`, `($object->property += expr)`, and
  `(ClassName::$prop += expr)`, and single C-style `for`
  initializer/increment actions are supported for those direct targets;
  expression forms return the updated value. Append offsets, nested
  offsets/properties, dynamic property names, unsupported static-property
  contexts, references/copy-on-write, PHP warning recovery, exact native error
  objects, and native lowering are not implemented.
- Pre/post increment and decrement is limited to direct static variables,
  direct array offsets, direct public object properties, and supported static
  properties whose current values are integers or floats, either as standalone
  statements, expressions, or single C-style `for` initializer/increment
  actions.
  Expression pre forms return the updated value and expression post forms
  return the previous value. Strings, arrays/objects as current values,
  undefined variables, missing array keys, non-array offset targets, append
  offsets, nested offsets/properties, dynamic property names, non-public
  visibility context, missing-property materialization, references,
  copy-on-write, exact native warning/error behavior, PHP string increment
  semantics, broader coercion recovery, and native lowering are not
  implemented.
- Null coalescing is limited to direct static variables, direct array-variable
  offsets, direct object-variable public properties, and supported static
  properties on the left side, plus direct-variable `$name ??= expr`, direct
  array-offset `$array[$key] ??= expr`, direct public object-property
  `$object->property ??= expr`, and supported static-property
  `ClassName::$prop ??= expr`, `self::$prop ??= expr`, and
  `parent::$prop ??= expr` statements and parenthesized expression forms.
  `??=` expression forms return the assigned fallback or existing
  non-null value. Object-property `??=` writes only existing declared public
  properties on existing object values; static-property `??=` writes only
  declared static property slots after current visibility checks. Missing
  properties, undefined target variables, non-object target variables, and
  unsupported static-property contexts fail with stable diagnostics.
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
- dynamic callables outside the documented string function-name and bounded
  `call_user_func_array()` array-callable subset, including closure
  invocation, `__invoke`, first-class callable syntax, non-public method
  callbacks, by-reference argument propagation, named arguments, and
  namespace/autoload-aware callable resolution
- `array_key_exists` lossy or non-finite float key coercion and PHP
  warning/deprecation behavior, array/object/resource/reference keys, exact
  native `TypeError` objects, reference/copy-on-write behavior, and native
  lowering
- `array_key_first`/`array_key_last`/`current`/`array_is_list` exact native
  `TypeError` objects, reference/copy-on-write container behavior, and native
  lowering
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
- `get_class_vars` property defaults beyond the current constant-expression
  subset, inheritance beyond the current single-parent chain, traits, interfaces,
  aliases/imports, namespace-aware names, autoloading,
  non-public/context-sensitive visibility listing, exact native ordering and
  `TypeError` behavior, and native lowering
- `get_object_vars` dynamic properties, visibility context for non-public
  properties, traits, interfaces, aliases/imports,
  namespace-aware names, references/copy-on-write, exact native ordering and
  `TypeError` behavior, and native lowering
- `get_mangled_object_vars` dynamic properties, property defaults beyond the
  current constant-expression subset, traits,
  interfaces, aliases/imports, namespace-aware names,
  non-public/context-sensitive visibility behavior beyond the current
  declaring-class slot ownership, references/copy-on-write, exact native
  ordering and `TypeError` behavior, and native lowering
- `property_exists` native true results, native declared property tables,
  object operands, built-in/internal/extension classes including `Exception`,
  autoloading,
  namespaces/import aliases, exact native `TypeError` behavior, and native
  lowering beyond direct string/string false folding
- `empty($object->name)` dynamic property names, non-public visibility
  context, complex lvalues, magic `__isset`/`__get` behavior,
  references/copy-on-write, exact native error behavior, and native lowering
- `unset($object->name)`/`unset($object->$name)` true property removal and
  property uninitialization, typed/uninitialized property behavior,
  non-public visibility context beyond the current visible-slot lookup, magic
  `__unset` behavior, references/copy-on-write, exact native error behavior,
  and native lowering
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
- `get_called_class` native lowering for called-class context,
  aliases/imports, namespace-aware names, exact native `Error` behavior, and
  broader late static binding
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
- `interface_exists` built-in/internal interface entries, autoloading, exact
  native `TypeError` behavior, and native lowering beyond direct string-name
  false folding
- `trait_exists` built-in/internal trait entries, autoloading,
  namespaces/import aliases beyond parsed declarations, exact native
  `TypeError` behavior, and native lowering beyond direct string-name false
  folding
- `enum_exists` built-in/internal enum entries, autoloading,
  namespaces/import aliases beyond parsed declarations, exact native
  `TypeError` behavior, and native lowering beyond direct string-name false
  folding
- `get_declared_interfaces` built-in/internal interface entries, autoloading,
  exact native ordering, and native lowering
- `get_declared_traits` built-in/internal trait entries, autoloading,
  namespaces/import aliases beyond parsed declarations, exact native ordering,
  and native lowering
- named arguments
- `declare(strict_types=1)` and PHP type declaration enforcement
- bare global constant resolution outside exact uppercase
  `ARRAY_FILTER_USE_KEY`, `ARRAY_FILTER_USE_BOTH`, `SORT_REGULAR`,
  `SORT_NUMERIC`, `SORT_STRING`, `PREG_SPLIT_DELIM_CAPTURE`, `PHP_VERSION_ID`, `PHP_VERSION`,
  `PHP_INT_MAX`, `PHP_SAPI`, the documented `E_*` error mask constants, and
  runtime-defined unqualified constants in the current name/value subset; PHP
  version component constants such as `PHP_MAJOR_VERSION`, patch-level host
  version coupling, SAPI/build metadata beyond the deterministic `cli` string,
  full extension constant catalogs, unsupported `define(...)` names
  or values, case-insensitive legacy constants, bare namespace constant
  fallback reads, nested `const` declarations, dynamic declaration values,
  broader `constant()`/`defined()` lookup for class constants, names lexed as
  language keywords or literals for bare reads, magic constants other than
  `__LINE__`, `__FILE__`, `__DIR__`, `__FUNCTION__`, and `__METHOD__`,
  reference/copy-on-write behavior for constant values, and native lowering
  remain unsupported
- namespace-aware behavior beyond the current class-name and same-namespace
  function declaration/call slice, including namespace-scoped constants,
  qualified or fully-qualified function calls, grouped/function/constant
  imports, string-name import expansion, `__NAMESPACE__`, autoload-aware
  lookup, and native lowering
- closure invocation, explicit and implicit capture binding/execution, and
  callable integration
- configurable recursion/call-stack limits matching PHP deployments
- exception objects and exception handling beyond the current throw/normal-try
  runtime boundaries
- interface inheritance/implementation enforcement, trait composition, and enum
  case objects/backed values/methods/interfaces
- generator functions, generator objects, `yield`, `yield from`, key/value
  yields, by-reference yields, `send`/`throw`/`return` generator semantics,
  and native lowering
- executable attribute declarations and reflection metadata beyond the current
  syntax-only skip boundary
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
- `mysqli_connect()`/`mysqli_real_connect()`/`mysqli_get_server_info()`/
  `mysqli_get_host_info()`/`mysqli_query()`/`mysqli_set_charset()`/`mysqli_select_db()`/`mysqli_real_escape_string()`/
  `mysqli_affected_rows()`/`mysqli_insert_id()`/`mysqli_ping()`/
  `mysqli_fetch_object()`/`mysqli_fetch_assoc()`/`mysqli_fetch_array()`/
  `mysqli_fetch_row()`/`mysqli_fetch_field()`/`mysqli_num_fields()`/
  `mysqli_num_rows()`/`mysqli_data_seek()`/`mysqli_free_result()`/
  `mysqli_more_results()`/`mysqli_next_result()`/`mysqli_report()`/
  `mysqli_init()` beyond the current
  metadata/report-mode/placeholder-object/fake successful real-connect and
  fake server-info/SQL-mode-query/charset-setup/database-selection/escaping/
  liveness-check/empty-result lifecycle boundary:
  mysqli extension loading, host/database connections,
  mysqli resources/objects with real connection state, host/transport/protocol
  metadata, queries, result sets, prepared statements, connection charset state, binary or invalid-string
  behavior, exact escaping edge cases, liveness or reconnect behavior,
  errors/warnings, transactions,
  configuration beyond the current report-mode flag, PDO behavior, exact PHP
  diagnostics, and native database calls
- `version_compare()` outside the current numeric-component subset: PHP's full
  version-string grammar, pre-release labels, arbitrary separators, invalid
  argument diagnostics, extension version coupling, and native lowering
- `sprintf()`/`vsprintf()` outside the current `%s`/`%d`/`%f`/`%F` subset:
  PHP's full format grammar, star width or precision, length modifiers, locale
  behavior, broad argument reordering, array/object/resource conversions,
  exact warning behavior, partial-output behavior, and native lowering beyond
  function-table introspection
- `strcasecmp()` outside the current exact-two-argument scalar/null
  string-convertible subset: array operands, object/resource coercions, binary
  string edge cases beyond valid UTF-8 runtime strings, locale-sensitive
  behavior, exact PHP diagnostics, and native lowering beyond function-table
  introspection
- `strtolower()` outside the current one-argument scalar/null
  string-convertible subset: locale-sensitive case mapping, full Unicode case
  folding, binary string edge cases beyond valid UTF-8 runtime strings,
  array/object/resource coercions, exact PHP diagnostics, and native lowering
  beyond function-table introspection
- `trim()` outside the current default-mask one-argument scalar/null
  string-convertible subset: custom character masks, binary/null-byte string
  edge cases beyond the current represented runtime-string subset,
  array/object/resource coercions, exact PHP diagnostics, and native lowering
  beyond function-table introspection
- `ltrim()` outside the current scalar/null string-convertible default-mask and
  non-empty literal-character-mask subset: character-mask ranges,
  binary/null-byte string edge cases beyond the current represented
  runtime-string subset, array/object/resource coercions, exact PHP
  diagnostics, and native lowering beyond function-table introspection
- `rtrim()` outside the current scalar/null string-convertible default-mask and
  non-empty literal-character-mask subset: character-mask ranges,
  binary/null-byte string edge cases beyond the current represented
  runtime-string subset, array/object/resource coercions, exact PHP
  diagnostics, and native lowering beyond function-table introspection
- `array_unshift()` outside the current direct-variable ordered-array mutation
  subset: non-variable first arguments, value-only dynamic calls such as
  `call_user_func("array_unshift", ...)`, broad by-reference argument
  handling, references/copy-on-write, exact warnings/errors, and native
  lowering beyond function-table introspection
- `array_pop()` outside the current direct-variable ordered-array mutation
  subset: non-variable targets, object-property array targets, value-only
  dynamic calls such as `call_user_func("array_pop", ...)`, broad by-reference
  argument handling, full internal pointer side effects, references/copy-on-write,
  exact warnings/errors, and native lowering beyond function-table
  introspection
- `current()` outside the current ordered-array first-value subset: PHP's
  mutable internal array-pointer model, interaction with `next()`/`reset()`,
  object operands, references/copy-on-write, exact warnings/errors, and native
  lowering beyond function-table introspection
- `next()` outside the current direct variable and direct object-property
  array-offset pointer-mutation subset: broad lvalue targets, value-only
  dynamic calls, full internal array-pointer semantics, object operands,
  `reset()`/`end()`/`prev()` interaction, references/copy-on-write, exact
  warnings/errors, and native lowering beyond function-table introspection
- `str_contains()` outside the current exact-two-argument scalar/null
  string-convertible subset: binary string edge cases beyond valid UTF-8
  runtime strings, array/object/resource coercions, exact PHP diagnostics, and
  native lowering beyond function-table introspection
- `str_ends_with()` outside the current exact-two-argument scalar/null
  string-convertible subset: binary string edge cases beyond valid UTF-8
  runtime strings, array/object/resource coercions, exact PHP diagnostics, and
  native lowering beyond function-table introspection
- `strpos()` outside the current scalar/null string-convertible haystack and
  needle plus optional integer offset subset: PHP-exact offset coercions and
  `ValueError` diagnostics, array/object/resource coercions, encoding-sensitive
  edge cases beyond represented runtime strings, and native lowering beyond
  function-table introspection
- `substr()` outside the current scalar/null string-convertible input plus
  integer offset and optional integer length subset: float/string offset and
  length coercions, object/resource operands, invalid UTF-8 byte ranges, exact
  PHP diagnostics, and native lowering beyond function-table introspection
- `substr_count()` outside the current scalar/null string-convertible haystack
  and needle plus optional integer offset/length subset: PHP-exact empty-needle
  `ValueError`, broad scalar coercions, array/object/resource coercions,
  encoding-sensitive edge cases beyond represented runtime strings, exact PHP
  diagnostics, and native lowering beyond function-table introspection
- `preg_match()` outside the current slash-delimited literal
  contains/prefix/suffix/exact pattern subset, the two exact WordPress db-host
  named-capture patterns, the exact WordPress table-prefix validation pattern
  `|[^a-z0-9_]|i`, the exact WordPress safe-collation query classifier, and
  the exact adjacent WordPress `wpdb::query()` DDL/DML classifiers, and the
  exact WordPress ASCII-check byte-range pattern: non-direct matches
  outputs, flags, offsets, optional unmatched-group fidelity, broad
  capture-group behavior, full PCRE syntax, bracket classes and ranges beyond
  the documented exact WordPress pattern, modifiers other than the documented
  exact WordPress `i` patterns and `u`,
  invalid-pattern warnings, byte/Unicode behavior beyond the current valid
  UTF-8 string model, broad coercions, exact diagnostics, and native lowering
  beyond function-table introspection
- `preg_replace()` outside the exact WordPress database-version cleanup pattern
  `/[^0-9.].*/`, path-tail cleanup pattern `#/[^/]*$#i`, and redirect
  sanitizer cleanup pattern `|[^a-z0-9-~+_.?#=&;,/:%!*\[\]()@]|i`, mail-host
  cleanup pattern `#^www\.#`, and KSES null cleanup patterns
  `/[\x00-\x08\x0B\x0C\x0E-\x1F]/` and `/\\\\+0+/`, all with an empty
  replacement and scalar/null subject, plus the exact WordPress
  `wpdb::prepare()` placeholder-escape shape with replacement `'%%\\1'`:
  pattern/replacement arrays, subject arrays, arbitrary non-empty
  replacements, limit/count arguments, callbacks, full PCRE replacement
  behavior, captures/backrefs beyond the reached placeholder replacement,
  invalid-pattern warnings,
  byte/Unicode behavior beyond the current valid UTF-8 string model, full PHP
  string escape semantics, broad coercions, exact diagnostics, and native
  lowering beyond function-table introspection
- `preg_split()` outside the exact WordPress `wpdb::prepare()` placeholder
  extraction pattern with `limit` `-1` and `PREG_SPLIT_DELIM_CAPTURE`: broad
  PCRE splitting, pattern arrays, subject arrays, other limits,
  `PREG_SPLIT_NO_EMPTY`, `PREG_SPLIT_OFFSET_CAPTURE`, flag combinations,
  invalid-pattern warnings, full capture semantics, exact diagnostics, and
  native lowering beyond function-table introspection
- `preg_replace_callback()` outside the exact WordPress
  `wp_sanitize_redirect()` UTF-8 sanitizer regex shape, exact
  `_wp_sanitize_utf8_in_redirect` string callback, scalar/null subject, and
  three arguments: pattern arrays, subject arrays, callback
  arrays/closures/method callables, broad callback invocation,
  limit/count/flags arguments, invalid-pattern warnings, exact diagnostics,
  and native lowering beyond function-table introspection
- `error_reporting()` outside the current no-argument read and one-integer
  mask-set subset: warning/notice/deprecation filtering, ini integration,
  disabled-function policy, non-integer coercions, exact diagnostics, and
  native lowering beyond function-table introspection
- `min()` outside the current two-or-more integer argument subset: array-form
  calls, mixed-type comparison rules, float/string/bool/null/object/resource
  operands, exact PHP diagnostics, and native lowering beyond function-table
  introspection
- `compact()` outside the current direct string variable-name argument subset:
  array arguments, nested arrays, invalid names, PHP warning behavior for
  missing or invalid names, variable-variable interactions, exact diagnostics,
  and native lowering beyond function-table introspection
- `str_replace()` outside the current scalar/null string-convertible subject
  subset with scalar/null replacement and scalar/null or one-level array
  search values: replacement arrays, subject arrays, nested search arrays,
  non-variable fourth `$count` targets, object/resource coercions, exact
  warning behavior, binary string edge cases, and native lowering beyond
  function-table introspection
- `implode()` outside the current scalar/null array-value subset: legacy
  reversed argument order, nested arrays, object/resource values, exact warning
  behavior, partial-output behavior, and native lowering beyond function-table
  introspection
- `dirname()` path behavior outside the current lexical Unix-style local path
  subset, including Windows drive and UNC paths, stream wrappers, filesystem
  canonicalization, symlink resolution, null-byte behavior, broad scalar
  coercions, exact `ValueError`/`TypeError` diagnostics, and native lowering
  beyond function-table introspection
- `is_dir()` behavior beyond the current one-string local path metadata slice:
  include-path lookup, stream wrappers, symlink/canonicalization policy,
  permission/open_basedir behavior, non-string coercions, stat-cache behavior,
  exact diagnostics, and native lowering beyond function-table introspection
- `header()` behavior beyond accepting current string/bool/int arguments as a
  no-op returning `null`: response header storage, status-code parsing/state,
  replacement/removal behavior, output-sent warnings, SAPI/web-server
  integration, exact `ValueError`/`TypeError` diagnostics, partial-output
  behavior, and native lowering beyond function-table introspection
- `header_remove()` behavior beyond accepting no argument or one string header
  name as a no-op returning `null`: actual all-header removal, named-header
  storage/removal, output-sent warnings, SAPI/web-server behavior, exact
  diagnostics, partial-output behavior, and native lowering beyond
  function-table introspection
- `headers_sent()` behavior beyond the current no-argument always-`false`
  shim: filename/line output arguments, output-started tracking, header
  storage, output buffers, SAPI differences, exact warnings, and native
  lowering beyond function-table introspection
- `abs()` behavior beyond the current integer and finite-float subset:
  integer-minimum overflow, numeric string coercion, bool/null coercion,
  array/object/resource operands, NaN/infinity behavior, exact diagnostics, and
  native lowering beyond function-table introspection
- `microtime()` behavior beyond the current `microtime(true)` float-seconds
  subset: no-argument and `false` string-return format, exact formatting,
  precision guarantees, monotonicity, deterministic virtual time, broad
  coercions, exact diagnostics, and native lowering beyond function-table
  introspection
- `ini_get()` behavior beyond the current deterministic registry: host php.ini
  discovery, mutable INI state, `ini_set()`/`ini_restore()`, `ini_get_all()`,
  SAPI differences, extension ownership/access metadata, exact option catalogs,
  coercions, exact diagnostics, and native lowering beyond function-table
  introspection
- `$_SERVER` behavior beyond the current deterministic CLI seed and direct
  root-symbol routing: real SAPI request population, environment imports,
  complete server key catalogs, `$GLOBALS` aliasing, references/copy-on-write,
  mutation-ordering fidelity, `variables_order`, exact warning behavior, and
  native lowering
- other superglobals such as `$_GET`, `$_POST`, `$_COOKIE`, `$_FILES`,
  `$_REQUEST`, `$_ENV`, `$_SESSION`, and `$GLOBALS`
- `exit()`/`die()` behavior beyond the current direct-call termination subset:
  callable/dynamic invocation, boolean/float/array/object argument handling,
  PHP's exact exit-status normalization, shutdown functions, destructors,
  finally ordering, output buffering, SAPI interaction, and native lowering
- `spl_autoload_register()` behavior beyond accepting closure/string callback
  registrations as a no-op success, including autoload stack storage,
  unregistering, invocation during class lookup, prepend ordering,
  namespace-aware class resolution, exact callable validation, exact
  `TypeError`/exception behavior, and native lowering beyond function-table
  introspection
- `extension_loaded()` behavior outside the deterministic bounded compatibility
  registry, including exact extension inventory policy, aliases, host
  PHP/module discovery, dynamic loading side effects, extension versions,
  extension functions/constants, `php.ini`/SAPI differences, exact PHP
  diagnostics for invalid arguments, and native lowering beyond direct
  string-name folding for already-lowerable string names
- PHP standard library beyond documented builtins
- `empty(...)` operands outside direct variables, direct nested array offsets,
  direct object-property operands, direct object-property array offsets, and
  supported static property operands, including function-call rooted offsets,
  dynamic property names, non-public property visibility context outside the
  current method-context slice, append offsets, complex lvalues, ArrayAccess,
  magic methods, and general expressions
- Zend extension loading
- full WordPress compatibility beyond the documented deterministic
  bootstrap/front-controller placeholder probes
- PHP's warning-and-continue behavior for undefined variables; plain reads fail
  with a runtime error in the current subset, while `isset($name)` remains the
  supported presence check
- actual PHP error-control behavior for `@expr`, including suppressing
  warnings/notices/deprecations, recoverable diagnostic severity, expression
  recovery values, and `error_reporting()` mask interactions
- PHP `Throwable`/`Error` objects, stack traces, recoverable warnings, notices,
  and user error handlers
- Preserving partial stdout emitted before a runtime failure; the current
  runtime-error path aborts the command with a diagnostic instead of modeling
  PHP's output buffering and fatal-error behavior

Unsupported code should fail with an explicit parse, runtime, or codegen error.
