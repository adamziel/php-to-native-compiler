# xml Extension

Status: bounded DOM/XMLReader/libxml metadata subset.

Implemented:

- `DOMDocumentType` is registered as a DOM core class and `DOMNode` subclass.
- Direct `new DOMDocumentType(...)` materializes PHP's uninitialized doctype
  placeholder while evaluating ignored constructor arguments.
- Reads of `name`, `entities`, `notations`, `publicId`, `systemId`, and
  `internalSubset` on that placeholder raise catchable `DOMException` objects
  with code `11` and message `Invalid State Error`.
- `ReflectionExtension("dom")` includes `DOMDocumentType` in the bounded DOM
  class inventory.
- `XMLReader` is registered as a core class in the bounded `xmlreader`
  extension with node-type and parser-property constants, readonly public
  metadata properties, `extension_loaded("xmlreader")`, `get_loaded_extensions()`,
  and `ReflectionExtension("xmlreader")` coverage.
- The interpreter path supports a bounded `XMLReader` event stream for simple
  XML documents with XML declarations, elements, end elements, text, CDATA,
  comments, attributes, namespace declarations, `open()`, `XML()`,
  `fromUri()`, `fromString()`, `fromStream()` over readable local/memory
  streams, `close()`, `read()`, `next()`,
  `moveToFirstAttribute()`, `moveToNextAttribute()`, `moveToAttribute()`,
  `moveToAttributeNo()`, `moveToAttributeNs()`, `moveToElement()`,
  `getAttribute()`, `getAttributeNo()`, `getAttributeNs()`,
  `readInnerXml()`, `readOuterXml()`, `readString()`, `lookupNamespace()`,
  parser-property get/set state, selected encoding argument validation, and
  empty/preload `setSchema()` / `setRelaxNGSchema()` diagnostics.
- Core XMLReader factory calls allocate subclasses, run zero-argument subclass
  constructors after successful source loading, surface constructor failures,
  and allow userland static `open()` overrides to win over the core factory.
- `XMLReader::DEFAULTATTRS` is honored for simple external DTD `ATTLIST`
  default attributes when the XML source references a readable system DTD path.

Unsupported:

- Parser-backed doctype discovery/population through `DOMDocument::load()` or
  `loadXML()`.
- `DOMNamedNodeMap` entity and notation objects.
- Exact readonly DOM property-hook write diagnostics.
- Full libxml parser parity: malformed XML recovery, entity expansion,
  validation, real schema/RelaxNG evaluation, stream contexts beyond readable
  local/memory stream payloads, encoding conversion, processing-instruction
  event payload parity, broad DTD grammars, namespace edge cases beyond simple
  prefix declarations, XMLReader subclass constructor arguments/property-hook
  side effects, and virtual-property reflection hooks.
- Native DOM/libxml execution.
