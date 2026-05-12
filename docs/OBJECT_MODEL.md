# Object/Class Metadata Model

This document records the current object/class boundary. It is a narrow
instantiation slice, not full PHP object execution.

The current implementation parses top-level class declarations into metadata and
can evaluate `new ClassName()` for declared classes that do not define
constructors. It stores class identity plus `null` instance-property slots and
can read/write public instance properties by static property name and check
direct public property operands with `isset($object->name)`. Method calls and
dynamic property names still fail with explicit parse diagnostics. Static
member access through `::` also fails with explicit parse diagnostics until
static property storage, static method dispatch, and class constants exist.

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
- `PhpObject`: an instantiated object with class identity and initialized
  instance-property slots.
- `Visibility`: `public`, `protected`, and `private` metadata markers.

The model follows the PHP lookup rules needed by the first object slice:

- class names are looked up case-insensitively while preserving the declared
  spelling;
- method names are looked up case-insensitively;
- property names are looked up case-sensitively;
- instance object shapes preserve instance-property declaration order and skip
  static properties;
- object values initialize supported instance properties to `null`;
- public instance property reads return the current slot value;
- public instance property writes mutate the current object value stored in that
  variable;
- direct `isset($object->name)` checks return true for non-null public slots
  and false for null or missing slots;
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

The current syntax slice also accepts `new ClassName()` with an identifier class
name and empty constructor argument list. Instantiation looks up the class
case-insensitively and returns an object value using the declared class spelling.
Undefined classes produce a stable runtime error. Classes with a `__construct`
method and `new` calls with constructor arguments also produce stable runtime
errors because constructor execution is not implemented.

The property syntax slice accepts `$object->name` reads and direct-variable
`$object->name = <expr>` writes when `name` is a declared public instance
property. It also accepts direct `isset($object->name)` checks over direct
object-variable operands. Property names remain case-sensitive. Undefined
properties, property access on non-object values, and non-public properties
produce stable runtime errors for ordinary reads/writes; `isset` returns false
for null slots, missing property names, undefined target variables, and
non-object target variables. Static properties are recorded as metadata but are
not stored in object values. Static member expressions such as
`ClassName::$prop`, `ClassName::method()`, and `ClassName::CONST` are rejected
by the parser instead of falling through to generic expression errors.

Native lowering rejects class declarations, object instantiation, object
property reads, and object property writes until metadata, object allocation,
property slots, and dispatch have explicit lowering support.

## Unsupported Edge Cases

The implemented class-declaration parser intentionally excludes nested and
conditional class declarations, inheritance, interfaces, traits,
abstract/final/readonly modifiers, constructor promotion, typed properties,
default property values, multiple properties in one declaration, constants,
static property storage, late static binding, magic methods, namespaces,
autoloading, anonymous classes, attributes, reflection, dynamic properties,
cloning, destructors, serialization hooks, visibility enforcement,
`self`/`parent`/`static`, `$this`, constructor execution, constructor
arguments, non-public property access, dynamic property names, property
assignment targets other than a direct variable, method dispatch, object
identity/handle aliasing, object comparisons, object-to-string conversion,
object callables, array-offset `isset` operands, non-public property `isset`
operands, complex object-property `isset` operands, static member execution
through `::`, `::class`, and native lowering.
