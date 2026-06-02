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
binary strings, plus bounded character-based multiline and token-start
locations for validation/decode parse errors. Focused PHPT proof covers the
current JSON core encode/decode
rows claimed by the Batch022 author packet plus the bounded
depth/error-location rows `ext/json/tests/007.phpt` and
`ext/json/tests/json_decode_error.phpt`, the invalid UTF-8 decode row
`ext/json/tests/json_decode_invalid_utf8.phpt`, the invalid UTF-8 encode row
`ext/json/tests/json_encode_invalid_utf8.phpt`, the non-finite partial-output
row `ext/json/tests/inf_nan_error.phpt`, and the bounded public
error-location rows `ext/json/tests/json_last_error_msg_error_location_001.phpt`
through `_010.phpt`.

## Semantic Gaps

The subset is intentionally bounded. It does not implement the full
`JsonSerializable` contract, exact invalid `__toString()` return diagnostics,
exact exception propagation for every serializer shape, exact byte-for-byte
diagnostic locations for every malformed JSON grammar edge, all UTF-16/UTF-8
corner cases beyond the covered bounded
invalid-byte repair lanes, exact escaped-string invalid-byte parity, broad
partial-output interaction parity beyond current
non-finite/unsupported/recursion/depth placeholders, host-extension option
catalogs, `JSON_THROW_ON_ERROR` exception behavior, native lowering, or JSON
functions beyond the listed bounded subset.
