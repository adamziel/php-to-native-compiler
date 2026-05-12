# Object/Class Metadata Model

This document records the current object/class boundary. It is a narrow
instantiation slice, not full PHP object execution.

The current implementation parses top-level class declarations into metadata and
can evaluate `new ClassName()` for declared classes that do not define
constructors. It stores class identity plus `null` instance-property slots.
Object property access and method calls still fail with explicit parse
diagnostics.

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

Native lowering rejects class declarations and object instantiation until
metadata, object allocation, and dispatch have explicit lowering support.

## Unsupported Edge Cases

The implemented class-declaration parser intentionally excludes nested and
conditional class declarations, inheritance, interfaces, traits,
abstract/final/readonly modifiers, constructor promotion, typed properties,
default property values, multiple properties in one declaration, constants,
static property storage, late static binding, magic methods, namespaces,
autoloading, anonymous classes, attributes, reflection, dynamic properties,
cloning, destructors, serialization hooks, visibility enforcement,
`self`/`parent`/`static`, `$this`, constructor execution, constructor
arguments, property reads/writes, method dispatch, object comparisons,
object-to-string conversion, object callables, and native lowering.
