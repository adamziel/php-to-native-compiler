# Object/Class Metadata Model

This document records the current object/class boundary before PHP object
syntax is executable.

The current implementation parses top-level class declarations into metadata,
but it does not instantiate objects, store object properties, bind `$this`, or
dispatch methods. `new`, object property access, and method calls still fail
with explicit parse diagnostics. The runtime model below keeps object work
separate from parser scaffolding.

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
- `Visibility`: `public`, `protected`, and `private` metadata markers.

The model follows the PHP lookup rules needed by the first object slice:

- class names are looked up case-insensitively while preserving the declared
  spelling;
- method names are looked up case-insensitively;
- property names are looked up case-sensitively;
- instance object shapes preserve instance-property declaration order and skip
  static properties;
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
runtime after registration. Native lowering rejects class declarations until
metadata, object values, and dispatch have explicit lowering support.

Object instantiation and member access remain unsupported until object values,
property storage, `$this`, method dispatch, constructor calls, and visibility
checks have tests.

## Unsupported Edge Cases

The implemented class-declaration parser intentionally excludes nested and
conditional class declarations, inheritance, interfaces, traits,
abstract/final/readonly modifiers, constructor promotion, typed properties,
default property values, multiple properties in one declaration, constants,
static property storage, late static binding, magic methods, namespaces,
autoloading, anonymous classes, attributes, reflection, dynamic properties,
cloning, destructors, serialization hooks, visibility enforcement,
`self`/`parent`/`static`, `$this`, object values, method dispatch, and native
lowering.
