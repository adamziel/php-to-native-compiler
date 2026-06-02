# xml Extension

Status: bounded DOM/libxml metadata subset.

Implemented:

- `DOMDocumentType` is registered as a DOM core class and `DOMNode` subclass.
- Direct `new DOMDocumentType(...)` materializes PHP's uninitialized doctype
  placeholder while evaluating ignored constructor arguments.
- Reads of `name`, `entities`, `notations`, `publicId`, `systemId`, and
  `internalSubset` on that placeholder raise catchable `DOMException` objects
  with code `11` and message `Invalid State Error`.
- `ReflectionExtension("dom")` includes `DOMDocumentType` in the bounded DOM
  class inventory.

Unsupported:

- Parser-backed doctype discovery/population through `DOMDocument::load()` or
  `loadXML()`.
- `DOMNamedNodeMap` entity and notation objects.
- Exact readonly DOM property-hook write diagnostics.
- Native DOM/libxml execution.
