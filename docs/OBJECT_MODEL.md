# Object/Class Metadata Model

This document records the first object/class boundary before PHP object syntax
is executable.

The current implementation does not run class declarations, `new`, object
property access, or method calls. Those constructs fail with explicit parse
diagnostics. The runtime model below exists so the next syntax slice has a
small, tested target instead of inventing object semantics inside the parser.

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

The next executable step should parse class declarations into this metadata
model without enabling object execution. A conservative first slice can accept:

- top-level `class Name { ... }` declarations only;
- property declarations with explicit or implicit `public` visibility;
- method declarations that reuse the existing function parameter/body subset;
- duplicate-name diagnostics routed through the runtime metadata model.

Object instantiation and member access should remain unsupported until object
values, property storage, `$this`, method dispatch, constructor calls, and
visibility checks have tests.

## Unsupported Edge Cases

The metadata sketch intentionally excludes inheritance, interfaces, traits,
abstract/final/readonly modifiers, constructor promotion, typed properties,
default property values, constants, static property storage, late static
binding, magic methods, namespaces, autoloading, anonymous classes, attributes,
reflection, dynamic properties, cloning, destructors, serialization hooks,
visibility enforcement, `self`/`parent`/`static`, `$this`, method dispatch, and
native lowering.
