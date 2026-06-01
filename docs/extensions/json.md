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
decode, `json_validate()` state/flag/depth handling, and selected invalid
UTF-8 `JSON_INVALID_UTF8_IGNORE`/`JSON_INVALID_UTF8_SUBSTITUTE` encode/decode
rows. Focused fixture proof covers the current JSON core encode/decode rows
plus the invalid-UTF-8 flag recovery row under `phpc run`.

## Semantic Gaps

The subset is intentionally bounded. It does not implement the full
`JsonSerializable` contract, exact exception propagation for every
serializer shape, exact byte-for-byte diagnostic locations, all UTF-16/UTF-8
corner cases and flag interactions, host-extension option catalogs, native
lowering, or JSON functions beyond the five listed above.
