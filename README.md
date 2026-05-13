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
The test suite exercises `phpc compile --emit-asm` for the current lowerable
straight-line scalar echo/assignment subset with a normalized CLI summary rather
than snapshotting platform-specific assembly text.
It also snapshots a representative unsupported array program with backend tools
removed from `PATH`, proving `--emit-asm` reports the LLVM lowering rejection
before attempting assembly backend discovery.
A separate lowerable scalar snapshot removes backend tools from `PATH` and
records the stable missing-backend diagnostic for environments without
`clang`, `llc`, or `cc`.
Another lowerable scalar snapshot hides `clang` and `llc` while exposing only a
`cc` command, proving the documented `cc -S` fallback with normalized assembly
output checks instead of backend-specific assembly text.

## Current Status

Milestone 1 is in progress. The interpreter/runtime path supports a small PHP
subset:

- `echo`
- integer, float, and string literals
- static variables, assignment, direct static-variable assignment expressions
  (`$name = expr`) and direct array-offset assignment expressions
  (`$array[$key] = expr`) and direct public object-property assignment
  expressions (`$object->property = expr`) with right-to-left chained
  assignment result values, including compound-assignment and null-coalescing
  assignment expressions as chained right-hand values, direct append-offset
  assignment expressions (`$array[] = expr`) with standalone assignment result
  values,
  direct static-variable compound assignment (`+=`, `-=`, `*=`, `/=`, `%=`,
  `.=`, `&=`, `|=`, `^=`, `<<=`, `>>=`)
  in statements and
  expressions with assignment result values, direct array-offset compound
  assignment (`$array[$key] += expr` and related operators) in statements,
  expressions, and C-style `for` initializer/increment slots, direct
  public object-property compound assignment (`$object->property += expr` and
  related operators) in statements, expressions, and C-style `for`
  initializer/increment slots, direct array-offset pre/post increment and
  decrement for integer and float offset values in statements, expressions,
  and C-style `for` initializer/increment slots, direct public
  object-property pre/post
  increment and decrement for integer and float property values in statements,
  expressions, and C-style `for` initializer/increment slots, direct
  static-variable pre/post increment and decrement for integer and float
  variables in statements, expressions, and C-style `for`
  initializer/increment slots, and
  `unset($name)` through per-scope symbol tables. Assignment-expression values
  have executable coverage in function call arguments, array literal
  keys/values, `if`/loop conditions, and builtin arguments for the documented
  direct-target subset
- `+`, `-`, `*`, `/`, `%`, `.`
- loose scalar comparisons and scalar strict identity comparisons used by
  control flow
- logical `&&`, `||`, `and`, `xor`, and `or` over the current truthiness
  rules, with boolean result values, short-circuit evaluation for `&&`, `||`,
  `and`, and `or`, PHP-style `&&`/`||` precedence, and lower-than-assignment
  word-operator precedence in the current expression and statement parser
  subset
- bitwise `&`, `|`, `^`, unary `~`, and shift operators `<<`/`>>` over the
  current integer/string subset, including PHP-style precedence, integer unary
  bitwise-not results for integer operands, integer binary results for
  non-string-string operands after current scalar-to-int coercion, bytewise
  string results when the resulting runtime string remains valid UTF-8, and
  PHP-shaped large-count shift behavior for the current integer result model;
  direct static-variable, direct array-offset, and direct public
  object-property compound assignments also support `&=`, `|=`, `^=`, `<<=`,
  and `>>=` through the interpreter path
- full ternary conditional expressions `$condition ? $if_true : $if_false`
  and short ternary expressions `$value ?: $fallback` over the current
  expression/value subset, with truthiness-based condition selection,
  condition-value reuse for short ternary, lazy branch/fallback evaluation,
  and executable coverage for mixes with `??` and assignment-expression
  branches over the documented direct-target subset
- `if` / `elseif` / `else`
- `while`, C-style `for` loops over the documented header subset,
  `do ... while` post-condition loops, `switch`/`case`/`default` statements
  over the current scalar comparison subset, and array `foreach` in value-only
  and key/value forms, including `break;` for innermost loops or switch
  statements and `continue;` for the innermost executing loop
- function declarations, positional calls with trailing default parameter
  values over the documented constant-expression and unqualified constant
  reference subset, recursive calls up to the documented guard, `return`, and
  isolated local scopes for user-function calls
- dynamic function calls through string-valued expressions that resolve to the
  documented callable builtin subset or user-defined functions
- top-level class declarations registered as metadata, with property and method
  metadata for the documented subset
- minimal object instantiation with `new ClassName()` for declared classes that
  do not define constructors; public instance properties can be read and
  written by static property name and checked with `isset($object->name)`
- magic constants `__LINE__`, evaluated from the expression token's source
  line, `__FILE__`, evaluated from the current `phpc run` input path when one
  is available, `__DIR__`, evaluated as that path's parent directory, and
  `__FUNCTION__`, evaluated as the current user-function name or an empty
  string outside a function
- narrow global constant resolution: exact uppercase `ARRAY_FILTER_USE_KEY` and
  `ARRAY_FILTER_USE_BOTH` work as bare built-in constants, and
  `define($name, $value)`, `constant($name)`, `defined($name)`, and bare
  user-constant reads support unqualified names over the documented
  scalar/array value subset; top-level single and grouped `const NAME = value;`
  declarations are executable over the current constant-expression and
  scalar/array value subset, including references to previously defined
  unqualified constants and the current built-in constant slice
- short array literals and long `array(...)` literals with integer/string keys
- array indexed reads, indexed writes, and append writes for the documented
  direct-variable array subset
- `unset(...)` with one or more operands over direct static variables and
  direct array-variable offsets in the documented subset
- `foreach ($array as $value)` and `foreach ($array as $key => $value)` over
  the documented ordered array value model
- direct `isset($array[$key])` checks for the documented array-offset subset
- direct `empty($name)`, `empty($array[$key])`, and
  `empty($object->publicProperty)` checks for the documented
  variable/array-offset/public-property subset
- null coalescing `??` for direct static variables, direct array offsets, and
  direct public object properties over the current value model
- null coalescing assignment `$name ??= expr`, `$array[$key] ??= expr`, and
  `$object->publicProperty ??= expr` for direct static variables, direct array
  offsets, and direct public object properties over the current value model,
  including parenthesized expression forms that return the assigned or
  existing value
- builtins for the documented scalar/array/object subset: `strlen`, `isset`,
  `empty`, `count`, `array_key_exists`, `array_key_first`, `array_key_last`,
  `array_is_list`, `array_values`, `array_keys`, including loose and strict
  scalar search-value filtering, `array_reverse` with default reindexing and
  boolean preserve-key behavior, `array_slice($array, $offset)`,
  `array_slice($array, $offset, $length)`, and null-length
  `array_slice($array, $offset, null)` with default integer-key reindexing
  and string-key preservation, plus boolean preserve-key mode for
  `array_slice`, `array_chunk($array, $length)` with default chunk-key
  reindexing and boolean preserve-key mode, `array_pad($array, $length,
  $value)` with positive right-padding, negative left-padding, and integer-key
  reindexing when padding is needed, `array_merge` over zero or more array
  operands, `array_replace` over one or more arrays with key-preserving
  left-to-right overwrite behavior, `array_combine` over equal-length
  key/value arrays with
  integer/string key values, variadic `array_intersect_key` and
  `array_diff_key` over two or more arrays with first-array key/value
  preservation, variadic `array_diff` and `array_intersect` over two or more
  arrays with scalar value comparisons and first-array key/value preservation,
  `array_unique` with scalar string-form value deduplication and
  first-occurrence preservation,
  `array_flip` over integer/string array values,
  `array_fill_keys` over integer/string key values, `array_count_values` over
  integer/string array values, `array_sum` over the current scalar
  numeric-coercion subset, `array_product` over the current scalar
  numeric-coercion subset,
  `array_reduce($array, $callback[, $initial])` over the current string-valued
  callback subset, `array_filter` without a callback, with a `null` callback,
  and with string-valued value-only callbacks, including explicit integer mode
  flag `0` for those value-only paths, plus string-valued key-only callbacks
  through integer mode flag `2` and string-valued value/key callbacks through
  integer mode flag `1`, `define`, `constant`, and `defined` over the documented
  runtime-defined/built-in constant name slice, `array_map` over the current
  one-array null-callback identity, variadic null-callback zip, and variadic
  string-callback subset with one-array key preservation and multi-array
  reindexing,
  `in_array` and `array_search` including strict scalar searches,
  `get_class($object)`, `is_object($value)`, `get_debug_type($value)`,
  `class_exists($name[, $autoload])`, `interface_exists($name[, $autoload])`,
  `trait_exists($name[, $autoload])`, `enum_exists($name[, $autoload])`, and
  `property_exists($object_or_class, $property)` and
  `method_exists($object_or_class, $method)` and
  `get_class_methods($object_or_class)` and
  `get_class_vars($class_name)` and
  `get_object_vars($object)` and
  `get_mangled_object_vars($object)` and
  `is_a($object_or_class, $class_name[, $allow_string])` and
  `is_subclass_of($object_or_class, $class_name[, $allow_string])` and
  `get_parent_class($object_or_class)` and `get_declared_classes()` and
  `get_declared_interfaces()` and `get_declared_traits()` over the current
  minimal object value model and declared-class metadata,
  `spl_object_id($object)` as an explicit unsupported object-handle identity
  boundary, `spl_object_hash($object)` as an explicit unsupported
  object-handle hash boundary,
  `var_dump`, and `print_r`
- stable runtime diagnostics for the currently covered runtime errors,
  including unresolved or non-string dynamic function calls, unsupported
  `get_called_class()` calls before method/static class context exists,
  unsupported `spl_object_id($object)` calls before PHP object handle identity
  exists, unsupported `spl_object_hash($object)` calls before PHP object
  handle hash support exists,
  `global` declarations, duplicate or unsupported `define(...)` constant
  definitions, unsupported `defined(...)` name arguments, unknown bare global
  constants outside the current built-in/runtime-defined slice such as
  `PHP_VERSION`,
  invalid `break`/`continue` outside a loop, and runaway recursion
- stable lex/parse diagnostics for unsupported dynamic/function features
  including variable variables, include/require/eval constructs,
  namespace and `use` declarations, namespace-qualified function/class names,
  variadics, references, parameter and return type declarations, static local
  variable declarations, magic constants other than executable `__LINE__`,
  `__FILE__`, `__DIR__`, and `__FUNCTION__`, with `__METHOD__`,
  `__CLASS__`, `__TRAIT__`, and `__NAMESPACE__` held on
  context-specific parse diagnostics,
  closures, named arguments,
  `declare(strict_types=1)`,
  unsupported nested, namespace-aware, or dynamic-value `const` declarations,
  unsupported array spread/reference elements, unsupported broader
  `unset(...)` forms such as property, append-offset, and nested unset,
  unsupported `foreach` by-reference/destructuring forms, unsupported
  comma-separated `for` header expression lists, expression-form
  `do ... while`, alternate `if`/`elseif`/`else` colon/`endif` syntax,
  expression-form or alternate-syntax `switch`, `break`/`continue` depth
  arguments, unsupported exception syntax (`throw`, `try`, `catch`, and
  `finally`), unsupported PHP 8 `match` expressions, unsupported
  unparenthesized nested ternary expressions, unsupported
  append-offset chained assignment
  expressions, unsupported complex or nested assignment-expression targets,
  unsupported append-offset `??=`,
  unsupported compound assignment targets outside direct static variables,
  direct array offsets, and direct public object properties,
  unsupported increment/decrement targets outside direct static variables,
  direct array offsets, and direct public object properties, and chained
  increment/decrement expressions,
  unsupported chained coalescing and unsupported append-offset null
  coalescing assignment forms, object
  method calls, dynamic property names, anonymous classes,
  unsupported class forms, `abstract`/`final`/`readonly` class modifiers,
  unsupported `abstract`/`final`/`readonly` class member modifiers,
  unsupported typed property declarations, unsupported property defaults,
  unsupported multiple property declarations, unsupported class constant
  declarations, unsupported `$this` usage, unsupported `clone` expressions,
  unsupported `instanceof` expressions, unsupported `ClassName::class`
  expressions, unsupported magic static receivers such as `self::`,
  `parent::`, and `static::`, trait declarations, trait use inside classes,
  interface declarations, interface implementation clauses, enum declarations,
  static member access, and class constants

`php_runtime` also contains a tested object/class metadata registry and minimal
object values. `phpc run` can instantiate declared constructor-free classes,
read/write public instance properties by static name, and check those public
properties with `isset` and `empty`. `empty($object->name)` returns true for
null and falsey public slots, missing properties, undefined target variables,
and non-object target variables in the current direct-object-variable subset.
`get_class($object)` returns the declared class name for those minimal object
values, `is_object($value)` returns true only for those current object values,
and `get_debug_type($value)` returns scalar/array type names or the declared
class name for current object values.
`class_exists($name[, $autoload])` checks the current declared-class metadata
case-insensitively for string class names; the autoload flag is accepted only
as a boolean and does not trigger autoloading.
`interface_exists($name[, $autoload])` accepts the same string-name and boolean
autoload boundary, returns false for all supported calls because interface
metadata is not represented yet, and does not trigger autoloading.
`trait_exists($name[, $autoload])` accepts the same string-name and boolean
autoload boundary, returns false for all supported calls because trait metadata
is not represented yet, and does not trigger autoloading.
`enum_exists($name[, $autoload])` accepts the same string-name and boolean
autoload boundary, returns false for all supported calls because enum metadata
is not represented yet, and does not trigger autoloading.
`property_exists($object_or_class, $property)` accepts current object values or
string class names, uses case-sensitive declared property names, reports
declared public/protected/private and static properties, and returns false for
missing classes.
`method_exists($object_or_class, $method)` accepts current object values or
string class names, uses case-insensitive declared method names, reports
declared public/protected/private and static methods, and returns false for
missing classes. `get_class_methods($object_or_class)` accepts current object
values or declared string class names and returns public declared methods in
declaration order. `get_class_vars($class_name)` accepts declared string class
names and returns public declared property names in declaration order with
`null` values because property defaults are not implemented.
`get_object_vars($object)` accepts current object values and returns public
instance property names with their current slot values in declaration order.
`get_mangled_object_vars($object)` currently uses the same public instance
property slice as `get_object_vars`; protected/private property-name mangling
and visibility-context behavior are not represented yet.
`is_a($object_or_class, $class_name[, $allow_string])` checks exact class
identity only: object inputs are accepted, string object/class inputs are
considered only when `allow_string` is true, and target class names use the
current case-insensitive class metadata lookup.
`is_subclass_of($object_or_class, $class_name[, $allow_string])` validates the
same object/string and class-name argument boundary, but because inheritance is
not represented yet it returns false for exact-class and no-parent metadata
cases. `get_parent_class($object_or_class)` accepts current object values or
declared string class names and returns false for all supported inputs until
parent class metadata exists. `get_declared_classes()` returns a zero-indexed
array of classes declared in the current parsed program in declaration order,
and is also available through string-valued dynamic function calls.
`get_declared_interfaces()` returns an empty zero-indexed array because
interface declarations and internal interface metadata are not represented yet;
it is also available through string-valued dynamic function calls.
`get_declared_traits()` returns an empty zero-indexed array because trait
declarations and internal trait metadata are not represented yet; it is also
available through string-valued dynamic function calls.
`get_called_class()` is recognized as a zero-argument callable boundary, but
direct and string-valued dynamic calls currently fail with a stable unsupported
runtime diagnostic until method/static class context exists.
`spl_object_id($object)` is recognized as a one-argument callable boundary, but
object arguments currently fail with a stable unsupported runtime diagnostic
until PHP object handle identity, reuse, clone behavior, and destruction are
modeled; non-object arguments fail with a stable type-boundary diagnostic.
`spl_object_hash($object)` is recognized as a one-argument callable boundary,
but object arguments currently fail with a stable unsupported runtime diagnostic
until PHP object handle hash behavior is modeled on top of object identity;
non-object arguments fail with a stable type-boundary diagnostic.
Constructors, `$this` object context binding, method
dispatch, dynamic property names, visibility enforcement for non-public
properties, inheritance and interface relationship checks, object handle
identity, clone expressions, `instanceof` relationship checks, class-name
constant resolution through `::class`, property default values, constructor
arguments, multiple properties in one declaration, class constant
declarations, static property storage, static method dispatch, magic static
receivers such as `self::`, `parent::`, and `static::`, class constants, trait
use inside classes, enum declarations, built-in/internal/extension classes in
`get_declared_classes()`, true results from `interface_exists()`,
true results from `trait_exists()`, true results from `enum_exists()`,
declared/built-in/internal interface entries in `get_declared_interfaces()`,
declared/built-in/internal trait entries in
`get_declared_traits()`, non-public/context-sensitive method listing for
`get_class_methods()`, inherited/trait/interface methods, anonymous classes,
`get_called_class()` class context and late static binding behavior,
`spl_object_id()` handle reuse and clone/destructor interactions,
`spl_object_hash()` hash formatting, handle reuse, and clone/destructor
interactions,
exact native class/interface/method/property ordering, `get_class_vars()` property
defaults, inheritance/trait/interface properties, context-sensitive visibility,
`get_object_vars()` dynamic properties and non-public visibility context,
`get_mangled_object_vars()` protected/private name mangling, dynamic
properties, non-public visibility context, `empty($object->name)` dynamic
property names, non-public visibility context, complex lvalues, magic methods,
`unset($object->name)` property uninitialization, typed/uninitialized property
behavior, dynamic property names, non-public visibility context, magic
`__unset`, references/copy-on-write, and native object lowering are not
supported yet.

LLVM IR emission currently supports a smaller straight-line subset: literal
`null`, booleans, integers, floats, and strings, direct static-variable
assignments from those lowerable values, direct reads of previously assigned
static variables, `echo`, and `print`. Echo conversion for this native subset
is limited to the current literal scalar formatting path: `null` and `false`
emit nothing, `true` emits `1`, integers use `%lld`, floats use `%g`, and
strings are emitted through static string constants. Binary arithmetic
operators `+`, `-`, `*`, `/`, and `%` are rejected before operand lowering
until generated code has PHP numeric coercion, dynamic division/modulo zero
checks, modulo coercions, references/copy-on-write behavior, and exact native
error behavior. Reads of variables that were not statically assigned earlier
in the same straight-line native lowering pass are rejected with a specific
codegen diagnostic until generated code has native symbol-table storage,
undefined-variable diagnostics, references/copy-on-write behavior, and exact
native error behavior. Native string concatenation `.` is rejected before
operand lowering until generated code has PHP string conversion, dynamic
allocation, references/copy-on-write behavior, and exact native error behavior.
Native comparison operators are rejected with a specific
codegen diagnostic until generated code has PHP comparison coercions and
non-scalar comparison diagnostics. Native unary minus and logical not are
rejected before operand lowering until generated code has PHP numeric coercion,
truthiness conversion, references/copy-on-write behavior, and exact native
error behavior. Native logical operators are rejected before operand lowering
until generated code has PHP truthiness and short-circuit semantics. Native
bitwise and shift operators are rejected before operand
lowering until generated code has PHP bytewise string behavior,
scalar-to-int coercion, and shift diagnostics. Native ternary and null
coalescing expressions are rejected before branch/operand lowering until
generated code has PHP truthiness, null-aware lookup, and branch side-effect
ordering. Native function calls are rejected before argument/callee lowering
until generated code has runtime call lookup, stack frames, arity/type
diagnostics, callable builtin dispatch, and dynamic string-call dispatch.
Executable magic constants `__LINE__`, `__FILE__`, `__DIR__`, and
`__FUNCTION__` are rejected by native lowering until generated code has source
mapping, path canonicalization, and function-context tracking.
Built-in constants, runtime-defined constants, bare constant reads, top-level
`const` declarations, and `define()`/`constant()`/`defined()` are rejected by
native lowering until generated code has native constant tables, source-order
definitions, namespace-aware lookup, and exact native error behavior.
Class declarations, object instantiation, public property reads/writes, and
object metadata builtins are rejected by native lowering until generated code
has native object layout, handles, visibility, method dispatch, and exact
native error behavior.
Array literals, array offset reads/writes, `foreach` array iteration, array
offset `unset`, and array builtins are rejected by native lowering until
generated code has native array storage layout, key normalization,
copy-on-write, references, callback dispatch, and exact native error behavior.
Control-flow statements including `if`/`elseif`/`else`, `while`, `for`,
`do ... while`, `switch`, `break`, and `continue` are rejected by native
lowering until generated code has PHP truthiness, branch layout, loop control,
switch fallthrough, references/copy-on-write side-effect behavior, and exact
native error behavior.
Mutation forms that are still interpreter-only, including compound assignment,
null coalescing assignment, increment/decrement, assignment expressions, direct
variable `unset`, and multiple-operand `unset`, are rejected by native lowering
until generated code has read-modify-write ordering, null-aware mutation, unset
symbol-table effects, references/copy-on-write, and exact native error behavior.
Unsupported programs and broader PHP coercions are rejected with structured
codegen errors.

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
