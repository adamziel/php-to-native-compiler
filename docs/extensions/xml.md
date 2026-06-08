# xml Extension

Status: bounded parser option metadata only.

Implemented in `phpc run`:

- `xml_parser_create()` and `xml_parser_create_ns()` with zero arguments,
  returning an interpreter-owned parser handle.
- `XML_OPTION_CASE_FOLDING`, `XML_OPTION_TARGET_ENCODING`,
  `XML_OPTION_SKIP_TAGSTART`, and `XML_OPTION_SKIP_WHITE` constants.
- `xml_parser_get_option()` for case folding, target encoding, and the current
  placeholder integer values for skip-tagstart/skip-white.
- `xml_parser_set_option()` for case folding from integer/bool values and
  target encoding values `UTF-8`, `ISO-8859-1`, and `US-ASCII`.

Unsupported: XML parsing, callbacks/handlers, namespace parsing behavior,
explicit parser-create encodings/separators, XMLParser object identity,
`xml_parse()`, `xml_get_error_code()`, `xml_error_string()`, phpinfo module
reporting, exact warnings/coercions, and native lowering.
