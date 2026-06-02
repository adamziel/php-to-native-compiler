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
last-error state, numeric-string encoding, big integers, object-as-array
decode, and invalid UTF-8 ignore/substitute handling for quoted JSON string
tokens plus encoded PHP binary strings. Focused PHPT proof covers the current
JSON core encode/decode rows claimed by the Batch022 author packet plus the
bounded depth/error-location rows `ext/json/tests/007.phpt` and
`ext/json/tests/json_decode_error.phpt`, the invalid UTF-8 decode row
`ext/json/tests/json_decode_invalid_utf8.phpt`, and the invalid UTF-8 encode
row `ext/json/tests/json_encode_invalid_utf8.phpt`.

## Semantic Gaps

The subset is intentionally bounded. It does not implement the full
`JsonSerializable` contract, exact exception propagation for every
serializer shape, exact byte-for-byte diagnostic locations, all UTF-16/UTF-8
corner cases beyond the covered bounded invalid-byte repair lanes, exact
escaped-string invalid-byte parity, multi-line or non-ASCII error-location
parity, host-extension option catalogs,
`JSON_THROW_ON_ERROR` exception behavior, native lowering, or JSON functions
beyond the listed bounded subset.
