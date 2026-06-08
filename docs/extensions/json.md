# json Extension

Status: bounded interpreter subset implemented.

## Supported Functions

- `json_encode($value, int $flags = 0, int $depth = 512)`
- `json_decode($json, ?bool $associative = null, int $depth = 512, int $flags = 0)`
- `json_validate($json, int $depth = 512, int $flags = 0)`
- `json_last_error()`
- `json_last_error_msg()`

## Unsupported Functions

- all other json extension functions and constants

## Runtime Dependencies

None beyond the boxed runtime value model.

## Test Coverage

Focused Rust tests cover scalar/object/array encode/decode, JSON constants,
last-error state, numeric-string encoding, non-finite float partial-output
replacement, big integers, object-as-array decode, PHP-shaped `$json`
argument boundaries for `json_decode()` / `json_validate()`, and invalid UTF-8
ignore/substitute handling for quoted JSON string tokens plus encoded PHP
binary strings, bounded character-based multiline and token-start locations for
validation/decode parse errors, bounded `JSON_THROW_ON_ERROR` `JsonException`
behavior, `JsonSerializable` thrown-exception propagation,
`JSON_UNESCAPED_LINE_TERMINATORS`, invalid decoded property names, invalid
array-key partial output, lowercase JSON float exponents, and overflow JSON
number decode to `INF`. Focused PHPT proof covers the current JSON core
encode/decode
rows claimed by the Batch022 author packet plus the bounded
depth/error-location rows `ext/json/tests/007.phpt` and
`ext/json/tests/json_decode_error.phpt`, the invalid UTF-8 decode row
`ext/json/tests/json_decode_invalid_utf8.phpt`, the invalid UTF-8 encode row
`ext/json/tests/json_encode_invalid_utf8.phpt`, the non-finite partial-output
row `ext/json/tests/inf_nan_error.phpt`, and the bounded public
error-location rows `ext/json/tests/json_last_error_msg_error_location_001.phpt`
through `_010.phpt`. Additional focused proof covers
`ext/json/tests/bug68546.phpt`, `bug68567.phpt`, `bug68992.phpt`,
`bug73113.phpt`, `json_decode_exceptions.phpt`,
`json_encode_exceptions.phpt`, `json_encode_u2028_u2029.phpt`,
`json_exceptions_error_clearing.phpt`, `pass001.phpt`, `pass001.1.phpt`, and
`pass001.1_64bit.phpt`.

## Semantic Gaps

The subset is intentionally bounded. It does not implement the full
`JsonSerializable` contract, exact invalid `__toString()` return diagnostics,
exact exception propagation for every serializer shape beyond direct thrown
`jsonSerialize()` diagnostics, exact byte-for-byte diagnostic locations for
every malformed JSON grammar edge, all UTF-16/UTF-8 corner cases beyond the
covered bounded invalid-byte repair lanes, exact escaped-string invalid-byte
parity, broad partial-output interaction parity beyond current
non-finite/unsupported/recursion/depth/invalid-key placeholders,
host-extension option catalogs, every `JSON_THROW_ON_ERROR` option
interaction outside the documented bounded throw paths, native lowering, or
JSON functions beyond the listed bounded subset.
