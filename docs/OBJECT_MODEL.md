# Object/Class Metadata Model

This document records the current object/class boundary. It is a narrow
object-execution slice, not full PHP object execution.

The current implementation parses top-level class declarations into metadata,
including a single `extends Parent` link between declared classes, and can
evaluate `new ClassName(...)` for declared classes, including public or
inherited public instance `__construct` execution. It stores class identity
plus `null`
instance-property slots for the exact class and inherited public/protected/private
properties with declaring-class ownership, and can read/write public instance
properties by static property name and check
direct public property operands with
`isset($object->name)` and
`empty($object->name)`. Objects are now represented as process-local handles, so
assignment, function argument/return binding, array storage, and foreach
by-value over object values preserve object identity and shared property slots.
It can also dispatch public, same-class private, and protected same-class/child
instance methods by static method name through `$object->method(...)`; method
bodies run in a fresh local scope with `$this` bound to the current object
handle. Inherited method lookup walks the current single-parent chain from the
receiver class to ancestors. Explicit `parent::method(...)` and
`parent::__construct(...)` calls are supported from active instance
method/constructor context and dispatch against the current class's parent
chain while reusing the current `$this` object. Explicit `self::method(...)`
calls are supported from active instance method/constructor context and
dispatch against the current class and inherited method chain while reusing
the current `$this` object. Dynamic method/property
names still fail with explicit parse diagnostics. Static
member access through `::` outside the current parent method-call slice also
fails with explicit parse diagnostics until static property storage, static
method dispatch, and class constants exist. Parent static property access,
parent class constants, and `parent::class` have distinct unsupported
diagnostics, as do self static properties, self class constants, and
`self::class`. Static receiver forms through `static::$prop`,
`static::method(...)`, `static::CONST`, and `static::class` also have distinct
unsupported diagnostics until late static binding is modeled.
The current introspection slice can check declared methods with
`method_exists($object_or_class, $method)` without executing or dispatching
those methods. It can also evaluate `is_a($object_or_class, $class_name[,
$allow_string])` as an exact-class or single-parent ancestor metadata check,
without interface relationship traversal. `is_subclass_of($object_or_class,
$class_name[, $allow_string])` validates the same relationship-check boundary
and walks the current single-parent metadata chain. `get_parent_class($object_or_class)`
validates current object/declared-string inputs and returns the immediate
parent class name when one is recorded, otherwise false.
`get_class_vars($class_name)` accepts declared string class names and returns
public declared and inherited properties with `null` values because property defaults are not
represented yet.
`get_object_vars($object)` accepts current object values and returns public
exact and inherited instance property names with their current slot values.
`get_mangled_object_vars($object)` accepts current object values and returns
inherited and exact-class public/protected/private instance slots with
PHP-style mangled keys. Private keys use the declaring class name.
Visibility-context behavior beyond same-declaring-class private access and
class/ancestor protected access is not represented yet.
`get_called_class()` is recognized as a zero-argument callable boundary, but it
currently fails with a stable unsupported-call diagnostic until method/static
class context and late static binding exist.
`spl_object_id($object)` accepts current object values and returns a stable
process-local handle id. `spl_object_hash($object)` accepts current object
values and returns a stable 32-character current-subset hash derived from that
handle id. Exact system PHP hash formatting and handle reuse after destruction
are not claimed yet.

## Runtime Metadata

`php_runtime` now defines:

- `PhpClassTable`: ordered class declarations plus case-insensitive class-name
  lookup.
- `ClassId`: stable numeric handle for one class entry in a class table.
- `PhpClassMetadata`: declared class name plus ordered property and method
  metadata.
- `PhpPropertyMetadata`: property name, visibility, and static/instance flag.
- `PhpMethodMetadata`: method name, visibility, and static/instance flag.
- `PhpObjectShape`: the instance-property layout derived from class metadata.
- `PhpObject`: a cloneable object handle with process-local object identity,
  class identity, and shared initialized
  instance-property slots.
- `Visibility`: `public`, `protected`, and `private` metadata markers.

The model follows the PHP lookup rules needed by the first object slice:

- class names are looked up case-insensitively while preserving the declared
  spelling;
- method names are looked up case-insensitively;
- property names are looked up case-sensitively;
- instance object shapes preserve exact-class instance-property declaration
  order and skip static properties;
- object values initialize inherited and exact-class non-static instance
  properties to `null` while preserving declaring class id/name for each slot;
- public instance property reads return the current slot value;
- public instance property writes mutate the current object value stored in that
  variable;
- private property reads and direct writes require an active method context
  matching the slot's declaring class. Protected property reads and direct
  writes work from the declaring class or a child-class method context,
  including parent-declared protected slots on child objects and peer objects;
- direct `isset($object->name)` checks return true for non-null public slots,
  same-declaring-class private slots, and protected slots visible from the
  active class or an ancestor, and false for null or missing slots;
- direct `empty($object->name)` checks return true for falsey public slots,
  same-declaring-class private slots, protected slots visible from the active
  class or an ancestor, missing slots, undefined target variables, and
  non-object target variables;
- direct object-property compound assignment and pre/post increment/decrement
  work for public slots, private slots owned by the active declaring class, and
  protected slots owned by the active class or an ancestor, reusing the current
  scalar helper behavior and return-value rules;
- direct object-property null coalescing and null coalescing assignment work
  for public slots, private slots owned by the active declaring class, and
  protected slots owned by the active class or an ancestor, preserving the
  current lazy fallback and null-vs-falsey behavior;
- public, same-class private, and protected same-class/child instance method
  calls use case-insensitive declared-or-inherited method lookup, evaluate
  arguments left to right, bind `$this` to the receiver object handle, and
  reuse the current user-function parameter/default/return subset. Private
  methods require an active same-class method context. Protected methods
  require an active same-class or child method context;
- explicit `parent::method(...)` and `parent::__construct(...)` calls require
  active instance method context, a parent class, and a public or protected
  resolved method in the parent chain. They evaluate arguments left to right,
  reuse the current `$this` object, and execute the resolved method with the
  declaring parent class as the active method context;
- explicit `self::method(...)` calls require active instance method context
  and a visible non-static method resolved from the current class and inherited
  method chain. They evaluate arguments left to right, reuse the current
  `$this` object, and execute the resolved method with the declaring class as
  the active method context;
- direct `unset($object->name)` is reserved with an explicit parse diagnostic
  until property uninitialization semantics are modeled;
- `method_exists($object_or_class, $method)` checks declared and inherited
  method metadata using case-insensitive method lookup for current object
  values or string class names;
- `get_class_methods($object_or_class)` returns public declared and inherited
  method names in child-to-parent declaration order for current object values
  or declared string class names;
- `get_class_vars($class_name)` returns public declared and inherited property
  names in child-to-parent declaration order with `null` values for declared
  string class names;
- `get_object_vars($object)` returns public exact and inherited instance
  property names in parent-to-child slot order with their current slot values
  for current object values;
- `get_mangled_object_vars($object)` returns inherited and exact-class
  public/protected/private instance properties with PHP-style mangled keys for
  current object values. Private keys use the slot's declaring class name;
- `is_a($object_or_class, $class_name[, $allow_string])` checks exact class
  identity and single-parent ancestor relationships using case-insensitive
  class metadata lookup; string first arguments are considered only when
  `allow_string` is true;
- `is_subclass_of($object_or_class, $class_name[, $allow_string])` accepts
  current object values and string first arguments, with string first
  arguments considered only when `allow_string` is true, and walks the current
  single-parent metadata chain while interface traversal remains unsupported;
- `get_parent_class($object_or_class)` accepts current object values or
  declared string class names and returns the immediate parent class name when
  one is recorded, otherwise false;
- `get_called_class()` validates its zero-argument call shape and then fails
  with a stable unsupported-call diagnostic because no method/static class
  context is tracked yet;
- `spl_object_id($object)` validates its one-argument call shape and returns the
  current object's process-local handle id for object inputs;
- `spl_object_hash($object)` validates its one-argument call shape and returns a
  stable current-subset hash for object inputs;
- duplicate class names, duplicate methods, and duplicate exact property names
  produce structured runtime errors.

## First Syntax Slice Target

The current syntax slice accepts:

- top-level `class Name { ... }` declarations only;
- property declarations with explicit or implicit visibility and no default
  values;
- method declarations that reuse the existing function parameter/body subset;
- duplicate-name diagnostics routed through the runtime metadata model.

`phpc run` registers those declarations in `PhpClassTable` before executing
top-level statements. Class declarations themselves are no-op statements at
runtime after registration.

The current syntax slice also accepts `new ClassName(...)` with an identifier
class name. Instantiation looks up the class case-insensitively, allocates an
object value using the declared class spelling, initializes instance properties
to `null`, and then executes a declared or inherited public instance
`__construct` method with `$this` bound to the new object handle. Constructor
arguments use the current positional argument and default-parameter subset.
Protected constructors can execute through `new ClassName(...)` from
same-class or child-class method context.
Undefined classes, constructor arguments for classes without constructors,
private constructors without same-class construction context, protected
constructors outside same-class/child-class construction context, parent
constructor calls outside active child instance context, and static
constructors produce stable runtime errors.

The property syntax slice accepts `$object->name` reads and direct-variable
`$object->name = <expr>` writes when `name` is a declared public instance
property. It also accepts direct `isset($object->name)` checks over direct
object-variable operands and direct `empty($object->name)` checks over direct
object-variable operands. Property names remain case-sensitive. Undefined
properties, property access on non-object values, and non-public properties
produce stable runtime errors for ordinary reads/writes; `isset` returns false
for null slots, missing property names, undefined target variables, and
non-object target variables, while `empty` returns true for falsey slots,
missing property names, undefined target variables, and non-object target
variables. Static properties are recorded as metadata but are
not stored in object values. Static member expressions such as
`ClassName::$prop`, `ClassName::method()`, and `ClassName::CONST` are rejected
by the parser instead of falling through to generic expression errors.

The method-call syntax slice accepts `$object->method(...)` when `method` is a
static identifier naming a declared or inherited public instance method, a
private method called from a same-class method context, or a protected method
called from a same-class/child method context. The receiver is evaluated first,
arguments are evaluated left to right after metadata checks, and the method
body runs with `$this` bound to the receiver object handle. Missing methods,
non-object receivers, private methods outside same-class method context,
protected methods outside same-class/child context, static methods called
through an object receiver, and `$this` outside instance method execution
report stable runtime diagnostics.

Native lowering rejects class declarations, object instantiation, object
property reads/writes, parent method calls, and instance method calls until metadata, object
allocation, property slots, object handles, method dispatch, and diagnostics
have explicit lowering support.
Native lowering also rejects `method_exists` through the current function-call
boundary until class metadata lookup has native support. `get_class_methods`
is rejected through the same function-call boundary until method-list metadata
lookup has native support. `is_a` and `is_subclass_of` are rejected through the
same function-call boundary until class relationship lookup has native support.
`get_parent_class` is rejected through that function-call boundary until parent
metadata lookup has native support. `get_class_vars` is rejected through the
same function-call boundary until property-list metadata lookup has native
support. `get_object_vars` is rejected through the same function-call boundary
until object property value extraction has native support.
`get_mangled_object_vars` is rejected through the same function-call boundary
until mangled object property extraction has native support.
`interface_exists` is rejected through that function-call boundary until
interface metadata lookup has native support.
`trait_exists` is rejected through that function-call boundary until trait
metadata lookup has native support.
`enum_exists` is rejected through that function-call boundary until enum
metadata lookup has native support.
`get_declared_interfaces` is rejected through that function-call boundary until
interface metadata lookup has native support.
`get_declared_traits` is rejected through that function-call boundary until
trait metadata lookup has native support.
`get_called_class` is rejected through that function-call boundary until
method/static class context lookup and late static binding have native support.
`spl_object_id` is rejected through that function-call boundary until PHP object
handle identity has native support.
`spl_object_hash` is rejected through that function-call boundary until PHP
object handle hash behavior has native support.

## Unsupported Edge Cases

The implemented class-declaration parser intentionally excludes nested and
conditional class declarations, interfaces, traits,
abstract/final/readonly modifiers, constructor promotion, typed properties,
default property values, multiple properties in one declaration, constants,
static property storage, late static binding, magic methods, namespaces,
autoloading, anonymous classes, attributes, reflection, dynamic properties,
cloning, destructors, serialization hooks, visibility enforcement,
`self`/`parent`/`static` beyond the current explicit self/parent method-call
slices, constructor behavior beyond public/inherited public instance
`__construct` and explicit parent calls, constructor arguments for classes without constructors,
property override compatibility,
non-public property access outside the current private/protected method context,
non-public constructor access beyond the current constructor slice, dynamic method/property names,
property assignment targets other than a direct variable, object comparisons,
object-to-string conversion,
object callables, array-offset `isset` operands, non-public property `isset`
operands outside the current private/protected method context, complex object-property `isset`
operands, dynamic property-name `empty` operands, non-public property
visibility context for `empty` outside the current private/protected method context, complex
object-property `empty` operands, magic `__isset`/`__get` behavior for
`empty`, object-property `unset`, property uninitialization,
typed/uninitialized property behavior, magic `__unset` behavior,
static member execution through `::`, `::class`, interface traversal for
`is_a`/`is_subclass_of`, default `$this` behavior for `get_parent_class()`,
`get_called_class` method/static class context, late static binding,
`spl_object_id` handle reuse after destruction, clone semantics, destructors,
`spl_object_hash` exact system PHP hash formatting, handle reuse after
destruction, clone semantics, destructors,
`get_class_methods` inheritance/trait/interface and non-public
context-sensitive method listing, `get_class_vars` property defaults,
inheritance/trait/interface properties, context-sensitive visibility, object
inputs, `get_object_vars` dynamic properties, non-public visibility context,
references/copy-on-write, exact native ordering,
`get_mangled_object_vars` protected/private property-name mangling, dynamic
properties, non-public visibility context, references/copy-on-write, exact
native ordering,
`interface_exists` true results for declared/built-in/internal interfaces,
`trait_exists` true results for declared/built-in/internal traits,
`enum_exists` true results for declared/built-in/internal enums,
`get_declared_interfaces` declared interface metadata and built-in/internal
interface entries, `get_declared_traits` declared trait metadata and
built-in/internal trait entries,
interfaces, traits, aliases/imports, namespace-aware class names, autoloading,
exact native `TypeError` behavior, and native lowering.
