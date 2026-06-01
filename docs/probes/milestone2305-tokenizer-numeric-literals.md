# Milestone 2305 tokenizer numeric literal probes

These probes cover a bounded `ext/tokenizer` residual slice for
`token_get_all()`/`TOKEN_PARSE` numeric-literal rows. They are tokenization
proofs only; they do not claim parser/runtime execution support for the same
numeric literal spellings outside strings passed to tokenizer APIs.

## PHP-compatible control

`tests/fixtures/milestone2305/tokenizer_numeric_literals.php` tokenizes a PHP
source string with `TOKEN_PARSE` and matches system PHP for the selected number
rows:

```text
T_LNUMBER:1_000
T_LNUMBER:0xCAFE_F00D
T_LNUMBER:0b1010_0110
T_LNUMBER:0o755
T_DNUMBER:.5e1
T_DNUMBER:5.
T_DNUMBER:5.e+1_2
T_DNUMBER:9223372036854775808
```

The selected rows mirror php-src `ext/tokenizer` coverage around constants and
large numeric token classification, including `token_get_all_variation10.phpt`,
`invalid_octal_dnumber.phpt`, and `invalid_large_octal_with_underscores.phpt`,
with additional PHP 8.x numeric-separator and prefixed-literal spellings
verified against the local PHP 8.2 CLI.

## Unsupported edges

- `TOKEN_PARSE` still accepts the flag only for the current contextual-token and
  valid-source tokenizer slice; it does not raise PHP `ParseError` objects for
  invalid syntax or invalid numeric separator positions.
- Heredoc/nowdoc token parity and full tokenizer-grade source reconstruction
  remain outside this milestone.
- Parser/runtime execution of numeric separators, binary integer literals, and
  explicit `0o` octal literals in the outer `phpc run` source remains separate
  from this tokenizer-source support.
