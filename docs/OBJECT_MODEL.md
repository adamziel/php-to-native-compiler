# Object/Class Metadata Model

This document records the current object/class boundary. It is a narrow
object-execution slice, not full PHP object execution.

The current implementation parses top-level class declarations into metadata,
including a single `extends Parent` link between declared classes, and can
evaluate `new ClassName(...)` for declared classes, including public or
inherited public instance `__construct` execution. Missing named classes and
direct-variable string dynamic class names in `new` expressions invoke the
current string user-function autoload callbacks before the class table is
rechecked. Class declarations loaded by executed include/require paths invoke
the same string user-function callback stack for missing `extends` parent
classes, direct `implements` interfaces, direct class-body trait `use` names,
and parent interfaces reached from autoloaded interface declarations before
final registration validation.
Successfully allocated
objects whose class declares or inherits a public non-static no-argument
`__destruct` method are queued for shutdown destructor execution, including
clones created through the current shallow clone slice. It stores class identity
plus `null`
instance-property slots for the exact class and inherited public/protected/private
properties with declaring-class ownership. Compatible public/protected
property redeclarations share one runtime slot while private parent property
redeclarations remain separate child slots. The runtime can read/write public
instance properties by static property name and check
direct public property operands with
`isset($object->name)` and
`empty($object->name)`. Objects are now represented as process-local handles, so
assignment, function argument/return binding, array storage, and foreach
by-value over object values preserve object identity and shared property slots.
It can also dispatch public, same-class private, and protected same-class/child
instance methods by static method name through `$object->method(...)`; method
bodies run in a fresh local scope with `$this` bound to the current object
handle. Inherited method lookup walks the current single-parent chain from the
receiver class to ancestors. Child methods that redeclare inherited
non-private methods keep the current bounded visibility, static/non-static,
required-parameter-count, and type compatibility checks during class
registration. The type compatibility slice accepts exact type text
case-insensitively and simple declared class/interface contravariant parameter
and covariant return relationships when both type names resolve through current
metadata. Explicit `parent::method(...)` and
`parent::__construct(...)` calls are supported from active instance
method/constructor context and dispatch against the current class's parent
chain while reusing the current `$this` object. Explicit `self::method(...)`
calls are supported from active instance method/constructor context and
dispatch against the current class and inherited method chain while reusing
the current `$this` object. Dynamic property-name reads and direct writes are
supported for existing public slots and public dynamic `stdClass` slots when
the property-name value is a string or integer. Dynamic method names still
fail with explicit parse diagnostics. Named
`ClassName::method(...)` calls parse and report stable runtime diagnostics
before static dispatch. Static member access through `::` outside the current
named/self/parent method-call, class-name constant, class-constant, and
static-property slices still fails with explicit parse diagnostics until
late-bound static members exist.
`ClassName::class` resolves to the
source-spelled class string without requiring class metadata. `self::class`
and `parent::class` resolve from active instance method/constructor context.
Class constants declared with the current constant-expression value subset are
stored in class metadata and resolve case-sensitively through
`ClassName::CONST`, `self::CONST`, and `parent::CONST` with current
public/protected/private visibility checks. Untyped static properties are
initialized from supported defaults or `null` in class-level storage and resolve
case-sensitively through `ClassName::$prop`, `self::$prop`, `parent::$prop`,
and late-bound `static::$prop` in active called-class context. `static::class`
resolves from the current called-class context inside instance and static
methods. `static::method(...)` resolves visible static methods through the
current called class and forwards that context into nested calls.
`static::CONST` resolves class constants through the current called class.
The current introspection slice can check declared methods with
`method_exists($object_or_class, $method)` without executing or dispatching
those methods. It can also evaluate `is_a($object_or_class, $class_name[,
$allow_string])` as an exact-class, single-parent class ancestor, or recorded
interface metadata check. The current interface traversal includes
already-declared user interface inheritance with one or more parent interfaces.
`is_subclass_of($object_or_class, $class_name[, $allow_string])` validates the
same relationship-check boundary and walks the current single-parent class
metadata chain plus recorded interface metadata. `get_parent_class($object_or_class)`
validates current object/declared-string inputs and returns the immediate
parent class name when one is recorded, otherwise false.
`get_class_vars($class_name)` accepts declared string class names and returns
public declared and inherited properties with supported constant-expression
default values or `null` for properties without defaults.
`class_exists()` and `interface_exists()` check current class/interface
metadata and, when their autoload flag is truthy, invoke currently registered
string user-function autoload callbacks on misses before rechecking metadata.
Those callbacks can use the current include/require path to load local files
that declare additional class/interface metadata; missing `new` class lookup
uses the same bounded string-callback path. `trait_exists()` checks declared
top-level trait metadata and, when its autoload flag is truthy, invokes the
same currently registered string user-function callbacks on misses before
rechecking metadata. `enum_exists()` checks declared unit-enum metadata without
triggering autoloading. `class_exists()` reports true for declared enums in the
current class-like metadata slice.
`get_declared_interfaces()` and `get_declared_traits()` list declared user
interfaces and top-level traits in declaration order. `class_uses()` reports
the direct user traits recorded on a current object value or declared string
class name, without recursing into parent classes. `class_parents()` reports
declared parent classes from immediate parent to root, which enables covered
userland recursive trait-helper patterns. `ReflectionClass` on a declared
user trait exposes direct trait-body `use` declarations through
`getTraitNames()`/`getTraits()` and exposes simple methods imported from those
used traits through `hasMethod()`, `getMethod()`, and `getMethods()`. A declared interface may
extend one or more user interfaces declared before or after the child
interface; concrete implementors of the child interface must expose the child
and all parent public method names with matching static/non-static shape, and
relationship checks also recognize the parent interfaces. Public interface
constants declared as `const NAME = ...` or `public const NAME = ...` resolve
through `InterfaceName::CONST`, inherited parent interfaces, implementing
classes, `self::CONST`/`static::CONST` in implementing class methods, and
`defined()`/`constant()` string lookups. Simple public instance methods from
already-declared traits may be composed into a class with
`use TraitName;`, repeated simple trait-use declarations, or
`use TraitA, TraitB;` and called through ordinary object method dispatch.
Simple public aliases, including same-use qualified forms such as
`TraitA::method as public alias`, are registered as ordinary public methods
and may satisfy the current interface method-presence checks. Protected/private
aliases and visibility-only adaptations such as
`TraitA::method as protected helper` and `TraitA::method as protected` use the
current method visibility metadata, so they are visible to `method_exists()`
but omitted from global-context `get_class_methods()` when non-public.
Bounded public instance conflict adaptations such as
`TraitA::method insteadof TraitB` select the winner from traits listed in the
same class-body `use` declaration and skip the loser method during
composition. Comma-separated loser lists such as
`TraitA::method insteadof TraitB, TraitC` are accepted for the same public
instance method shape when every loser is listed in the same class-body `use`
declaration. That selected public instance winner can also be exposed through
a same-block explicit-public alias, such as
`TraitA::method insteadof TraitB; TraitA::method as public alias;`; the
original method and alias are both ordinary public methods for dispatch,
`method_exists()`, `get_class_methods()`, and current interface method-presence
checks. Public constants declared by already-declared traits as
`const NAME = ...` or `public const NAME = ...` are composed into consuming
classes as ordinary public class constants and can be resolved through the
current `ClassName::CONST`, `self::CONST`, `parent::CONST`, and late-bound
`static::CONST` class-constant paths.
`get_object_vars($object)` accepts current object values and returns public
exact and inherited instance property names with their current slot values.
`get_mangled_object_vars($object)` accepts current object values and returns
inherited and exact-class public/protected/private instance slots with
PHP-style mangled keys. Private keys use the declaring class name.
Visibility-context behavior beyond same-declaring-class private access and
class/ancestor protected access is not represented yet.
`get_called_class()` is recognized as a zero-argument callable and returns the
current called class while executing in current instance and static method
contexts. Outside method or static class context it fails with a stable
unsupported-call diagnostic.
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
- `PhpClassMetadata`: declared class name plus ordered property, class
  constant, and method metadata.
- `PhpClassConstantMetadata`: class constant name and visibility.
- `PhpPropertyMetadata`: property name, visibility, and static/instance flag.
- interpreter-owned static property storage keyed by declaring class and
  property name for the current untyped/no-default static property slice.
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
- class constant names are looked up case-sensitively;
- instance object shapes preserve exact-class instance-property declaration
  order and skip static properties;
- static properties are stored per declaring class outside object slots and
  inherited static property reads/writes share the declaring class slot unless
  a child redeclares the property;
- the core class table seeds metadata-only `Exception`, `stdClass`, `PDO`, and
  `PDOStatement` entries, with `PDO` exposing a bounded public integer
  constant catalog for current metadata checks;
- object values initialize inherited and exact-class non-static instance
  properties to `null` while preserving declaring class id/name for each slot;
  compatible inherited public/protected redeclarations share one slot with the
  effective visibility from the compatible descendant declaration, while
  private parent properties remain separate from same-name child properties;
- public instance property reads return the current slot value;
- missing direct instance property reads call a visible non-static
  `__get($name)` method when one is declared or inherited, and otherwise keep
  the existing undefined-property diagnostic;
- public instance property writes mutate the current object value stored in that
  variable;
- missing direct instance property writes call a visible non-static
  `__set($name, $value)` method when one is declared or inherited, ignore that
  method's return value, and keep the assignment expression result as the
  assigned value;
- dynamic property-name reads and direct writes resolve string and integer
  property-name values, can read/write existing public slots on current object
  values, and can materialize public dynamic slots on `stdClass` object
  handles;
- private property reads and direct writes require an active method context
  matching the slot's declaring class. Protected property reads and direct
  writes work from the declaring class or a child-class method context,
  including parent-declared protected slots on child objects and peer objects;
- direct `isset($object->name)` checks return true for non-null public slots,
  same-declaring-class private slots, and protected slots visible from the
  active class or an ancestor, false for null slots, and call visible
  non-static `__isset($name)` for missing slots;
- direct `empty($object->name)` checks return true for falsey public slots,
  same-declaring-class private slots, protected slots visible from the active
  class or an ancestor, undefined target variables, and non-object target
  variables. Missing slots call visible non-static `__isset($name)` first;
  when it returns truthy, `empty` calls `__get($name)` and checks the returned
  value's truthiness;
- direct `unset($object->name)` nulls existing visible slots under the current
  storage model. Missing direct property slots call visible non-static
  `__unset($name)` when one is declared or inherited;
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
  require an active same-class or child method context. Missing direct
  instance method calls dispatch to visible non-static `__call($name, $args)`
  when one is declared or inherited, with `$args` materialized as a
  zero-indexed PHP array of the evaluated positional arguments;
- bounded object string conversion dispatches visible non-static
  `__toString()` with no arguments for `echo $object`, `print $object`,
  `(string) $object`, binary concatenation, `.=` over the current supported
  compound-assignment targets, and the current double-quoted string/heredoc
  interpolation evaluator. The method must return a string in the current
  subset. A bounded core interface catalog is also available for current
  internal interface names, and `Stringable` relationship checks additionally
  recognize classes with a resolved public non-static `__toString()`;
  non-string returns, broader built-in interface catalogs and enforcement,
  `${...}`/dynamic/static
  property/arbitrary expression interpolation, exact PHP `TypeError` objects,
  and native lowering remain unsupported;
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
- `ClassName::CONST`, `self::CONST`, and `parent::CONST` resolve declared or
  inherited class constants from the named, active, or parent class context.
  Constant values use the current constant-expression subset and evaluate to
  null, bool, int, float, string, or array values. Public constants are visible
  everywhere, private constants require the declaring class context, and
  protected constants require the declaring class or a child-class context;
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
  single-parent metadata chain plus current recorded interface metadata;
- `get_parent_class($object_or_class)` accepts current object values or
  declared string class names and returns the immediate parent class name when
  one is recorded, otherwise false;
- `get_called_class()` validates its zero-argument call shape and returns the
  active called class in current instance and static method contexts;
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
Public non-static no-argument destructors, including inherited destructors, run
during normal script shutdown in reverse allocation order for objects reached
by the current allocation tracker. User-declared `__destruct` methods are
validated at class registration for the currently executable public,
non-static, parameterless shape, so non-public, static, or parameterized
destructors fail before object allocation rather than from the shutdown queue.
Undefined classes, constructor arguments for classes without constructors,
private constructors without same-class construction context, protected
constructors outside same-class/child-class construction context, parent
constructor calls outside active child instance context, and static
constructors produce stable runtime errors. Exact PHP fatal wording for
destructor declaration errors, destructor execution on runtime-error paths,
cyclic garbage collection, exact object lifetime ordering, handle reuse, and
native lowering remain unsupported.

The property syntax slice accepts `$object->name` reads and direct-variable
`$object->name = <expr>` writes when `name` is a declared public instance
property, a private slot owned by the active declaring class, or a protected
slot visible from the active class or an ancestor. It also accepts
`$object->$name` reads and direct `$object->$name = <expr>` writes for string
or integer dynamic names that resolve to existing public slots, plus public
dynamic slot creation on `stdClass`. It also accepts direct
`isset($object->name)` checks over direct object-variable operands and direct
`empty($object->name)` checks over direct object-variable operands for the
same public/private/protected context slice. For missing direct property
slots, ordinary reads can dispatch visible non-static `__get($name)`,
`isset` can dispatch visible non-static `__isset($name)`, and `empty` can
dispatch `__isset($name)` followed by `__get($name)` when `__isset` is truthy.
Missing direct property writes can dispatch visible non-static
`__set($name, $value)`. Missing direct property unsets can dispatch visible
non-static `__unset($name)`.
Property names remain
case-sensitive. Undefined properties, property access on non-object values,
and non-public properties outside the current method-context slice produce
stable runtime errors for ordinary reads/writes; `isset` returns false for
null slots, missing property names without `__isset`, undefined target
variables, and non-object target variables, while `empty` returns true for
falsey slots, missing property names without a truthy `__isset`, undefined
target variables, and non-object target variables.
Static properties are recorded as metadata and stored per declaring class, but
are not stored in object values. `ClassName::$prop`, `self::$prop`, and
`parent::$prop` support direct reads and writes, compound assignment, pre/post
increment/decrement, `isset`, `empty`, `??`, and `??=` for the current
untyped static property slice. Static property storage is initialized from the
current constant-expression default subset or `null`. `unset(ClassName::$prop)`,
`unset(self::$prop)`, and `unset(parent::$prop)` are parsed and report a
stable runtime diagnostic because PHP forbids unsetting static properties; they
do not remove static storage. Named static method expressions such as
`ClassName::method(...)` execute for declared or inherited visible static
methods under the current positional/default-parameter subset, without `$this`,
and with the declaring class as the active class context. Missing named static
method calls dispatch to visible static `__callStatic($name, $args)` when one
is declared or inherited, with `$args` materialized as a zero-indexed PHP
array of evaluated positional arguments.
`$object::method(...)` and `$className::method(...)` evaluate the receiver
object or class-name string, resolve a visible static method from that receiver
class, execute without `$this`, and use the receiver class as the called-class
context. Missing dynamic-receiver static method calls also dispatch to visible
static `__callStatic`. `self::method(...)` and `parent::method(...)` also
execute resolved visible static methods while running inside active class
context, while missing `self::method(...)` and late `static::method(...)` calls
dispatch to visible static `__callStatic` in the current class/called-class
context.
`ClassName::class` returns the syntactic class string, and `self::class` /
`parent::class` resolve only while executing with active class context.
Class constants are accepted as `const NAME = value;` or
`public|protected|private const NAME = value;` and resolve through
`ClassName::CONST`, `self::CONST`, `parent::CONST`, and `static::CONST`.
Typed constants, multiple constants in one declaration, namespace/alias-aware
constant lookup, and dynamic string lookup through `constant()`/`defined()` are
outside the current slice.
Typed static properties, dynamic static property names, storage-removing
static-property unset, and top-level `static::$prop` execution are outside the
current slice.

The method-call syntax slice accepts `$object->method(...)` when `method` is a
static identifier naming a declared or inherited public instance method, a
private method called from a same-class method context, or a protected method
called from a same-class/child method context. The receiver is evaluated first,
arguments are evaluated left to right after metadata checks, and the method
body runs with `$this` bound to the receiver object handle. Named
`ClassName::method(...)`, `$object::method(...)`, `$className::method(...)`,
`self::method(...)`, and `parent::method(...)` calls execute visible static
methods without binding `$this`. Missing methods,
non-object receivers, private methods outside same-class method context,
protected methods outside same-class/child context, non-static methods through
dynamic static receivers, non-static `self::`/`parent::` calls without current
`$this`, and `$this` outside instance or static method execution report stable
runtime diagnostics.

Native lowering rejects class declarations, object instantiation, object
property reads/writes, class-name constants, class constants, parent method
calls, dynamic static method calls, and instance method calls until metadata,
object allocation, property slots, object handles, method dispatch, and
diagnostics have explicit lowering support.
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
`get_called_class` is rejected through that function-call boundary until called
class context lookup has native support.
`spl_object_id` is rejected through that function-call boundary until PHP object
handle identity has native support.
`spl_object_hash` is rejected through that function-call boundary until PHP
object handle hash behavior has native support.

## Unsupported Edge Cases

The implemented class-declaration parser intentionally excludes nested and
conditional class declarations, typed/non-public/abstract/final or
multi-constant interface declarations, cyclic parent-interface inheritance
beyond stable rejection,
full interface signature enforcement,
trait properties, non-public/typed/abstract/final/static trait constants,
multi-constant trait declarations, trait constant adaptations, conflicting
trait/class constants, static/abstract/final or non-public trait methods,
conflicting trait use beyond the bounded `insteadof` shape,
trait alias/adaptation edge cases beyond the current simple public, qualified
public-alias, same-block winner public-alias, protected/private alias, and
single-trait visibility-only slices,
unqualified visibility-only adaptations across multiple used traits,
unqualified `insteadof`, `__TRAIT__`, nested/conditional trait
registration, backed enum
declarations, enum case objects, enum methods/constants/properties, enum interface implementation,
abstract-method enforcement, method visibility compatibility enforcement,
readonly class and property semantics, constructor promotion, typed properties,
instance property default values, multiple properties in one declaration, typed
or multi-declarator class constants, typed static properties, late static
binding, magic methods beyond the current direct missing-property
`__get`/`__isset`/`__set` slice, namespaces,
autoloading beyond string user-function callbacks for `class_exists()`,
`interface_exists()`, `trait_exists()`, missing `new` class instantiation, and
included class/interface/trait declaration dependencies, anonymous
classes, attributes, reflection, dynamic property
semantics beyond current `stdClass` public slot materialization,
cloning beyond the current shallow clone plus bounded visible non-static
`__clone` dispatch slice, serialization hooks, broader visibility enforcement,
`self`/`parent`/`static` beyond the current explicit self/parent method-call,
class-constant, and static-property slices, constructor behavior beyond public/inherited public
instance `__construct` and explicit parent calls, constructor arguments for classes without constructors,
typed/default property compatibility,
non-public property access outside the current private/protected method context,
non-public constructor access beyond the current constructor slice, dynamic
method names, dynamic property-name forms beyond existing public slots and
`stdClass` public dynamic slots,
property assignment targets other than a direct variable, object comparisons,
object callables, ArrayAccess beyond current direct object-offset
read/write/append/isset/empty/`??`/unset, compound-assignment, and
integer/float increment/decrement forms plus the current direct
object-property `ArrayAccess` single-key read/write/isset/empty/`??`/unset/
append, keyed compound-assignment, and integer/float increment/decrement slice,
non-public property `isset` operands outside the current private/protected
method context, complex object-property `isset` operands, dynamic
property-name `empty` operands, non-public property
visibility context for `empty` outside the current private/protected method context, complex
object-property `empty` operands, magic `__isset`/`__get` behavior for
dynamic property names and object-property dimensions, true object-property
removal/uninitialization,
typed/uninitialized property behavior, inaccessible-property `__unset`
fidelity,
dynamic property-name magic, property-array-offset magic, inaccessible-property
`__set` fidelity, inaccessible-method `__call` fidelity, dynamic method-name
magic, inaccessible-method `__callStatic` fidelity, parent missing-method
`__callStatic`, broader built-in interface catalogs and exact `Stringable`
enforcement/diagnostics, exact
`__toString()` non-string-return error objects, static member execution through `::` beyond the current class-name constant and
called-class slices, interface traversal for
`is_a`/`is_subclass_of`, default `$this` behavior for `get_parent_class()`,
native lowering for `get_called_class` called-class context, broader late
static binding,
`spl_object_id` handle reuse after destruction, clone semantics, destructors,
`spl_object_hash` exact system PHP hash formatting, handle reuse after
destruction, clone semantics, destructors,
`get_class_methods` inheritance/trait/interface and non-public
context-sensitive method listing, `get_class_vars` property defaults beyond
the current constant-expression subset,
inheritance/trait/interface properties, context-sensitive visibility, object
inputs, `get_object_vars` dynamic properties, non-public visibility context,
references/copy-on-write, exact native ordering,
`get_mangled_object_vars` protected/private property-name mangling, dynamic
properties, non-public visibility context, references/copy-on-write, exact
native ordering,
`interface_exists` true results for built-in/internal interfaces and interface
implementation relationships,
`trait_exists` true results for built-in/internal traits,
`enum_exists` true results for built-in/internal enums,
`get_declared_interfaces` built-in/internal interface entries,
`get_declared_traits` built-in/internal trait entries,
`class_uses` remains non-recursive; recursive trait helpers are limited to
userland code combining `class_parents()` and `class_uses()` or bounded
`ReflectionClass` trait metadata.
Built-in/internal trait entries, `class_parents` internal parent metadata,
exact missing-class warning behavior, broader Reflection integration,
interfaces, traits, aliases/imports, namespace-aware class names, autoloading,
exact native `TypeError` behavior, and native lowering remain unsupported.
