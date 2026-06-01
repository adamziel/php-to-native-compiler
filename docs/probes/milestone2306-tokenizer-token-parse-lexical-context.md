# Milestone 2306 tokenizer TOKEN_PARSE lexical probes

These probes cover a bounded `ext/tokenizer` lexical slice where
`TOKEN_PARSE` changes semi-reserved words from keyword tokens to `T_STRING`.
The slice is tokenization-only and does not claim full parse-error validation.

## PHP-compatible control

`tests/fixtures/milestone2306/tokenizer_token_parse_lexical_context.php`
compares plain `token_get_all($code)` with `token_get_all($code, TOKEN_PARSE)`
for selected php-src rows:

```text
--plain
T_CONTINUE:continue
T_ARRAY:ARRAY
T_NAMESPACE:namespace
--parse
T_STRING:continue
T_STRING:ARRAY
T_STRING:namespace
```

The fixture mirrors php-src `ext/tokenizer` coverage from
`token_get_all_TOKEN_PARSE_001.phpt`, `token_get_all_TOKEN_PARSE_002.phpt`, and
`bug77966.phpt`: member access (`X::continue`), class constants named after
reserved words (`const ARRAY`), and trait adaptation aliases using
`namespace as`.

## Unsupported edges

- `TOKEN_PARSE` still does not perform full parse validation or throw PHP
  `ParseError` objects for invalid source.
- The `namespace as` contextual string rule is bounded to the reached trait
  adaptation alias shape; broader trait adaptation grammar and comments between
  `namespace` and `as` remain outside this probe.
- Full tokenizer heredoc/nowdoc parity remains unsupported.
