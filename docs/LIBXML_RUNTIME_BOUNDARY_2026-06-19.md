# Libxml Runtime Boundary

PTN exposes `dom`, `libxml`, `xml`, `xmlreader`, and `xmlwriter` as one
runtime compatibility surface. The production boundary is libxml-compatible
document ownership, parser state, error buffering, namespaces, reader cursors,
writer targets, and DOM/SimpleXML handoff.

Current implementation status:

- `DOM*` objects own a bounded `PtnXmlNode` tree. Node insertion, wrong-document
  checks, namespace attributes, and serialization are local adapters that must
  converge on libxml tree ownership.
- `XMLReader` is a bounded cursor over the shared parse adapter. It must not
  fork XML parsing semantics away from DOM/libxml.
- `XMLParser` stores callback, namespace, option, and error-code state in a
  bounded SAX adapter. Parser errors and options should map to libxml concepts.
- `XMLWriter` owns bounded memory/URI/stream target state. Name and namespace
  validation must remain libxml-compatible until native writer state lands.
- `libxml_*` error functions are a request-global bounded error-buffer adapter.
- `SimpleXML` should hand off to the shared libxml DOM/tree boundary. Do not add
  an independent local SimpleXML tree.

The focused PHPT evidence is
`tools/phpt-ptn-hvvb4.4-libxml-boundary-row-pack.txt`.
