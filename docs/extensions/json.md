# json Extension

Status: bounded interpreter subset implemented.

## Supported Functions

- `json_encode($value, int $flags = 0, int $depth = 512)`
- `json_decode($json, ?bool $associative = null, int $depth = 512, int $flags = 0)`
- `json_validate($json, int $depth = 512, int $flags = 0)`
- `json_last_error()`
- `json_last_error_msg()`

## Unsupported Functions

- remaining json extension functions and constants outside the bounded list

## Runtime Dependencies

None beyond the boxed runtime value model.

## Test Coverage

Focused Rust tests cover scalar/object/array encode/decode, JSON constants,
last-error state, numeric-string encoding, big integers, object-as-array
decode, bounded invalid UTF-8 ignore/substitute encode/decode behavior,
covered one-line decode error locations, catchable invalid decode depth, and
`json_validate()` depth/UTF-8 checks. Focused PHPT proof covers the current
JSON core encode/decode rows claimed by the Batch022 author packet plus the
bounded invalid UTF-8 and error-location rows listed in `docs/PROGRESS.md`.

## Semantic Gaps

The subset is intentionally bounded. It does not implement the full
`JsonSerializable` contract, `JSON_THROW_ON_ERROR`, exact exception
propagation for every serializer shape, broad byte-for-byte diagnostic
location parity, all UTF-16/UTF-8 corner cases, every JSON option
interaction, host-extension option catalogs, native lowering, or JSON
functions beyond the supported list above.
