# Architecture

## Pipeline

The intended compiler pipeline is:

```text
PHP source
-> lexer/parser
-> AST
-> semantic analysis
-> IR
-> lowering/type specialization
-> LLVM IR text
-> assembly/object/executable
-> linked runtime
```

Milestone 1 implements the lexer, parser, AST, a direct interpreter/runtime
execution path, and a narrow LLVM IR text emitter for simple straight-line code.
The interpreter runs top-level statements in a global symbol table and creates a
fresh local symbol table for each user-function call. Static reads, writes,
`isset($name)`, `empty($name)`, and direct `unset($name)` operate on the active
symbol table; importing globals into function scope through `global`
declarations is not implemented.

## Compiler Crate

`compiler/` contains:

- lexer and parser for the supported PHP subset
- AST definitions
- interpreter bridge for `phpc run`
- LLVM IR text emission for currently lowerable code
- CLI and fixture test runner

The parser is handwritten recursive descent. This keeps the early grammar easy
to audit while avoiding regex-based parsing.

## Runtime Crate

`runtime/` contains the PHP-shaped boxed value model used by the interpreter and
future generated code helper calls.

Implemented now:

- `Null`
- `Bool`
- `Int`
- `Float`
- `String`
- ordered PHP arrays with integer/string keys
- class metadata, object-shape descriptors, and minimal object values for
  `new ClassName()` over declared constructor-free classes
- structured runtime error categories with stable diagnostic messages for the
  currently supported runtime failures
- PHP-ish echo conversion
- PHP-ish truthiness for the implemented value types
- basic arithmetic, comparison, and concatenation helpers
- key normalization for array strings that are valid decimal integers

Planned runtime values and semantics:

- resources
- references
- copy-on-write containers
- `$this`, constructor calls, visibility enforcement for non-public properties,
  method dispatch, and PHP object handle identity

## Native Codegen

The first backend emits LLVM IR text and shells out to `clang` for assembly.
This is deliberately less work than an x86-64 backend and lets the project focus
on PHP semantics first.

Tradeoff: Milestone 1 native lowering is smaller than interpreter support. The
backend must return a codegen error for unsupported constructs rather than
pretend to compile them.

Current assembly emission order:

1. Generate LLVM IR text.
2. Use `clang` if available.
3. Use `llc` if available.
4. If no LLVM assembly tool is available, generate equivalent C for the same
   narrow lowerable subset and ask `cc -S` for assembly.

The C fallback exists only to keep `phpc compile --emit-asm` executable on
machines without LLVM tools. It must not grow into the primary backend without a
documented architecture decision.

Current native lowering rejects PHP comparison operators before operand
lowering. That keeps generated code from implying PHP comparison coercions,
array/object comparison behavior, `NAN`/`INF` edge cases, or exact native error
objects that only the interpreter path currently handles or diagnoses.
Native lowering also rejects logical operators before operand lowering until
generated code has explicit PHP truthiness conversion, short-circuit
side-effect ordering for `&&`, `||`, `and`, and `or`, and both-operand
evaluation for `xor`.
Native lowering rejects bitwise and shift operators before operand lowering
until generated code has PHP bytewise string operations, scalar-to-int
coercion, negative/large shift diagnostics, and exact native error behavior.
Native lowering rejects ternary and null coalescing expressions before branch
or operand lowering until generated code has PHP truthiness conversion,
null-aware variable/offset/property lookup, branch side-effect ordering, and
exact native error behavior.
Native lowering rejects direct and dynamic function calls before argument,
callee, or callback lowering until generated code has runtime call lookup,
stack-frame layout, arity/type diagnostics, callable builtin dispatch, dynamic
string-call dispatch, and exact native error behavior. The
`define()`/`constant()`/`defined()` constant-table builtins have a separate
global-constant rejection boundary.
Native lowering also rejects user-function declarations and return statements
before traversing function bodies until generated code has function symbol
tables, stack-frame layout, default parameter binding, recursion guards,
return-value flow, and exact native error behavior.
Native lowering rejects executable magic constants `__LINE__`, `__FILE__`,
`__DIR__`, and `__FUNCTION__` until generated code has source mapping, path
canonicalization, function-context tracking, eval/include source interaction
rules, and exact native error behavior.
Native lowering rejects built-in constants, runtime-defined constants, bare
constant reads, top-level `const` declarations, and
`define()`/`constant()`/`defined()` before operand/argument lowering until
generated code has native constant tables, source-order definitions,
namespace-aware lookup, and exact native error behavior.
Native lowering rejects class declarations, object instantiation, public
property reads/writes, and object metadata builtins before body, operand, or
argument lowering until generated code has native object layout, object
handles, visibility checks, method dispatch, class metadata access, and exact
native error behavior.
Native lowering rejects array literals, array indexing, array assignment,
`foreach` array iteration, array offset `unset`, and array builtin function
calls before body, operand, argument, or callback lowering until generated code
has native array storage layout, key normalization, copy-on-write containers,
references, callback dispatch, and exact native error behavior.

## Dynamic Features

Dynamic PHP features will be implemented as runtime fallback zones:

- dynamic function calls use runtime lookup; the first implemented slice accepts
  string-valued callees that resolve to the documented callable builtin subset
  or user-defined functions
- variable variables use materialized symbol tables; current variable-variable
  syntax is rejected with an explicit diagnostic before execution
- dynamic includes will use runtime include resolution
- `eval` will parse and execute in the caller scope
- namespaces and imports will need namespace-aware name resolution before they
  can affect function, class, or dynamic callable lookup
- global constants use a narrow interpreter constant table: exact uppercase
  `ARRAY_FILTER_USE_KEY` and `ARRAY_FILTER_USE_BOTH` are available as bare
  built-in constants, while runtime-defined constants can be created with
  `define($name, $value)`, queried with `defined($name)`, and read with
  `constant($name)` or a bare unqualified constant name for the documented
  string-name and scalar/array value subset. Top-level single and grouped
  `const NAME = value;` declarations define unqualified constants at statement
  execution time over the current constant-expression and scalar/array value
  subset, including references to previously defined unqualified constants and
  the current built-in constant slice

Only the string-valued dynamic function lookup slice is executable today.
Variable-variable execution, include/require execution, and `eval` remain design
boundaries; direct `eval(...)` syntax is reserved and rejected with a stable
parse diagnostic. Namespace declarations and top-level `use` import
declarations are also reserved and rejected with stable parse diagnostics.
Array/object callables, method calls, first-class callable syntax, and
namespace/autoload-aware callable resolution are still outside the implemented
dynamic-call subset. Constant names that are lexed as language keywords or
literals cannot be read bare, and case-insensitive legacy constants, extension
constants, namespace-qualified constants, nested or namespace-aware `const`
declarations, dynamic `const` values, class constants through
`constant(...)`/`defined(...)`, references/copy-on-write for constant values,
and constant lowering are still outside the implemented constant subset. Native
lowering currently rejects the entire global-constant slice explicitly rather
than emitting partial constant-table code.

## Namespace/Import Boundary

`namespace` declarations, top-level `use` declarations, and
namespace-qualified function/class references are reserved by the lexer/parser
today and rejected with stable parse diagnostics before execution.
The first executable namespace/import slice should define how declared
functions, classes, dynamic function lookup, object instantiation, and error
messages store and resolve fully qualified names.

Initial unsupported namespace/import behavior remains: bracketed namespace
blocks, global namespace blocks, multiple namespaces in one file, executable
qualified and fully qualified function/class references, aliased imports,
grouped imports, function imports, constant imports, trait `use` execution,
autoload interaction, and namespace-aware native lowering.

## Object/Class Boundary

The current object/class step is a narrow public-property boundary, not full PHP
object execution. `php_runtime` has a `PhpClassTable`, stable `ClassId` handles,
class metadata, property metadata, method metadata, visibility markers, derived
object shapes for instance-property layout, and minimal object values. Class and
method lookup are case-insensitive, property lookup is case-sensitive, and
duplicate class/member metadata produces structured runtime errors.

`phpc run` parses top-level `class Name { ... }` declarations into that metadata
registry. The accepted member subset records public/protected/private
visibility, static flags, property names without defaults, and method names
whose parameters/bodies use the existing function parser subset. `new
ClassName()` can instantiate a declared class when the class has no
`__construct` method and the call supplies no constructor arguments. The
allocated object stores class identity and `null` instance-property slots in
declaration order while skipping static properties.

`phpc run` can read and write public instance properties by static property
name, for example `$box->name` and `$box->name = "Ada"`. Writes mutate the
current object value stored in that variable. Direct `isset($box->name)` checks
the current public slot without treating a missing name as a property read.
Direct `empty($box->name)` checks the current public slot truthiness and treats
null slots, missing names, undefined target variables, and non-object target
variables as empty for the current direct-object-variable subset.
`get_class($object)` returns the declared class name stored in the current
minimal object value and is also available through string-valued dynamic
function calls. `get_debug_type($value)` reports current scalar/array type
names and the declared class name for current object values.
`class_exists($name[, $autoload])` checks the interpreter's already-registered
class metadata table by string class name, using the same case-insensitive
class lookup as instantiation. The autoload flag is accepted only as a boolean
and does not trigger autoloading in the current subset.
`interface_exists($name[, $autoload])` accepts the same string-name and boolean
autoload boundary, returns false for all supported calls because interface
metadata is not represented yet, and does not trigger autoloading.
`trait_exists($name[, $autoload])` accepts the same string-name and boolean
autoload boundary, returns false for all supported calls because trait metadata
is not represented yet, and does not trigger autoloading.
`enum_exists($name[, $autoload])` accepts the same string-name and boolean
autoload boundary, returns false for all supported calls because enum metadata
is not represented yet, and does not trigger autoloading.
`property_exists($object_or_class, $property)` checks the same declared
property metadata for current object values or string class names, with
case-sensitive property names and no autoload side effects.
`get_class_vars($class_name)` accepts declared string class names and returns
public declared property names in declaration order with `null` values because
property defaults are not implemented.
`get_object_vars($object)` accepts current object values and returns public
instance property names with their current slot values in declaration order.
`get_mangled_object_vars($object)` accepts current object values and returns
the same public instance property slice; protected/private property-name
mangling and visibility-context behavior remain outside the current object
model.
`is_a($object_or_class, $class_name[, $allow_string])` performs exact-class
identity checks against the current metadata table. `is_subclass_of(...)`
shares the same argument boundary, but because inheritance metadata is not
represented yet it returns false for exact-class and no-parent cases.
`get_parent_class($object_or_class)` accepts current object values or declared
string class names and returns false for all supported inputs because parent
metadata is not represented yet.
`get_declared_classes()` lists classes declared in the current parsed program;
`get_declared_interfaces()` returns an empty list because interface metadata is
not represented yet; `get_declared_traits()` returns an empty list because
trait metadata is not represented yet.
`get_called_class()` is reserved as a zero-argument runtime boundary and fails
with a stable unsupported-call diagnostic until method/static class context and
late static binding exist.
`spl_object_id($object)` is reserved as a one-argument runtime boundary and
fails with a stable unsupported-call diagnostic for object inputs until PHP
object handle identity is represented; non-object inputs fail with the current
stable type-boundary diagnostic.
`spl_object_hash($object)` is reserved as a one-argument runtime boundary and
fails with a stable unsupported-call diagnostic for object inputs until PHP
object handle hash behavior is modeled on top of object identity; non-object
inputs fail with the current stable type-boundary diagnostic.
Missing properties, non-object targets, and non-public properties still produce
stable runtime diagnostics for normal reads/writes. Objects do not bind `$this`,
execute methods, run constructors, enforce visibility for non-public
properties, expose reflection, implement dynamic property names, or model PHP
object handles/aliasing for executable identity builtins. Static member syntax
through `::`, including
`ClassName::$prop`, `ClassName::method()`, and `ClassName::CONST`, is rejected
with explicit parse diagnostics until static storage, dispatch, and class
constants exist. Native lowering rejects class declarations, object
instantiation, object property reads/writes, and object metadata builtins with a
specific object/class codegen diagnostic. See `docs/OBJECT_MODEL.md` for the
named unsupported edge cases.

## Include/Require Resolution Design

`include`, `include_once`, `require`, and `require_once` are reserved by the
lexer/parser today and rejected with stable parse diagnostics before execution.
The first executable include/require slice should use these rules:

- the interpreter carries the current file path, process working directory, and
  include stack in runtime execution context
- only paths that evaluate to PHP strings are accepted at first
- absolute paths resolve directly
- relative paths resolve against the directory of the file containing the
  include/require expression
- `include_once` and `require_once` de-duplicate by canonical absolute path when
  the filesystem can canonicalize the target, and by normalized absolute path
  otherwise
- included files execute in the caller scope and may return a value through
  PHP's `return` statement
- native lowering rejects include/require until file loading, scope effects,
  and return-value behavior have explicit lowering support

Initial unsupported include/require behavior remains: `include_path` lookup,
current-working-directory fallback, stream wrappers, `phar://`, URL includes,
autoload interaction, opcache behavior, cycle detection beyond `_once`
de-duplication, and PHP's warning-vs-fatal recovery details.

## Eval Fallback Design

`eval` is reserved by the lexer/parser today and rejected with a stable parse
diagnostic before execution. The first executable `eval` slice should use these
rules:

- parse direct `eval(<expr>)` as a special language construct, not as an
  ordinary function or dynamic callable
- require exactly one argument and evaluate that argument in the caller scope
- accept only string-valued code for the first slice
- parse the evaluated string with a dedicated eval-fragment parser entry point
  that reads a statement list without requiring a `<?php` opening tag
- execute the resulting statements against the caller's current symbol table, so
  assignments affect the same local or top-level scope that called `eval`
- let `return` inside the evaluated fragment produce the `eval(...)` expression
  value; falling off the end should produce `null`
- keep native lowering rejecting `eval` until parser re-entry, source mapping,
  caller-scope effects, and return behavior have explicit lowering support

Initial unsupported eval behavior remains: non-string eval arguments, exact
`ParseError` object semantics, source mapping for diagnostics inside evaluated
strings, functions/classes declared from evaluated code, nested eval,
include/require inside eval, references/copy-on-write interactions,
`GLOBALS`/superglobal behavior, namespaces/use declarations, opcache behavior,
and PHP's exact warning/fatal recovery details.

## Exception Boundary

Exception syntax is reserved by the lexer/parser today and rejected with stable
parse diagnostics before execution. `throw` statements/expressions and
`try`/`catch`/`finally` blocks do not build AST nodes yet because the runtime
does not model `Throwable`/`Exception` objects, stack unwinding, `finally`
execution, stack traces, or exact native error objects. Native lowering must
continue rejecting these constructs until exception control flow has explicit
runtime and IR semantics.

## Match Expression Boundary

PHP 8 `match` expressions are reserved by the lexer/parser today and rejected
with a stable parse diagnostic before execution. They do not build AST nodes yet
because expression-form branching needs explicit semantics for strict arm
matching, default arms, exhaustiveness errors, thrown expressions inside arms,
value evaluation order, and exact native error objects. Native lowering must
continue rejecting `match` until those runtime and IR semantics exist.

## Fixture Tests

Fixture tests are stored as `.php` files with sibling `.stdout`, `.stderr`, and
`.exit` files. The runner strips one final editor newline from `.stdout` and
`.stderr` fixtures. A fixture that needs to assert an actual trailing newline
should include a blank final line.

When `phpc` intentionally differs from system PHP, a sibling `.phpc-only` marker
keeps the fixture in the normal runner while skipping optional system PHP
comparison.

## Extension Model

Zend extension loading is not an early target. Selected extensions will be
implemented as runtime modules with documented dependencies and semantic gaps.
