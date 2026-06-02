# mbstring Extension

Status: bounded scalar helper subset on the `phpc run` interpreter path.

Implemented helpers currently cover UTF-8 plus the scalar single-byte aliases
documented in `docs/SUPPORT.md`: `mb_strlen()`, `mb_substr()`, `mb_strcut()`,
`mb_substr_count()`, `mb_strpos()`, `mb_stripos()`, `mb_strrpos()`,
`mb_strripos()`, `mb_strtolower()`, and `mb_strtoupper()`.

`mb_strcut()` uses byte offsets and byte lengths, rounds UTF-8 cuts to scalar
boundaries so it does not split a character, and falls back to byte slicing for
single-byte encodings.

Optional nullable `$encoding` operands on the implemented helpers use the
shared PHP-shaped string boundary: omitted or `null` operands use the current
default encoding, scalar operands stringify into the existing encoding lookup,
supported visible `__toString()` objects are accepted, and arrays,
non-stringable objects, closures, and resources raise catchable `?string`
`TypeError`s.

Unsupported: full mbstring encoding catalogs and conversion tables,
UTF-16/UCS/EUC-JP/JIS/stateful cut rules, invalid-sequence and substitute
character policy, mbstring-owned mutable extension-global state, broad
array/object/resource data-operand coercions, exact invalid `__toString()`
return diagnostics, references/COW, and native execution lowering.
