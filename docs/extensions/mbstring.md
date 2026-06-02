# mbstring Extension

Status: bounded scalar helper subset on the `phpc run` interpreter path.

Implemented helpers currently cover UTF-8 plus the scalar single-byte aliases
documented in `docs/SUPPORT.md`: `mb_strlen()`, `mb_substr()`, `mb_strcut()`,
`mb_substr_count()`, `mb_strpos()`, `mb_stripos()`, `mb_strrpos()`,
`mb_strripos()`, `mb_strtolower()`, and `mb_strtoupper()`.

`mb_strcut()` uses byte offsets and byte lengths, rounds UTF-8 cuts to scalar
boundaries so it does not split a character, and falls back to byte slicing for
single-byte encodings.

Unsupported: full mbstring encoding catalogs and conversion tables,
UTF-16/UCS/EUC-JP/JIS/stateful cut rules, invalid-sequence and substitute
character policy, mbstring-owned mutable extension-global state, broad
array/object/resource operand coercions, references/COW, and native execution
lowering.
