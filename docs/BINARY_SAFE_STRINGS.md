# Binary-Safe String Storage Plan

This note records the current generated C string representation and a small
migration path toward PHP-compatible length-aware strings. It is intentionally
design-only: the first implementation bead should make a narrow runtime change
with native tests, not a broad rewrite of every string helper.

## Current Representation

- `PtnValue` stores `PTN_STRING` as only `const char *` in the generated C
  runtime (`src/backend/runtime.rs:49`). The constructors `ptn_string()` and
  `ptn_owned_string()` both assign that pointer without carrying length or
  ownership (`src/backend/runtime.rs:182`, `src/backend/runtime.rs:189`).
- `PtnArrayKey` also stores string keys as `const char *`
  (`src/backend/runtime.rs:41`) and uses NUL-terminated operations for key
  parsing, equality, and diagnostics (`src/backend/runtime.rs:246`,
  `src/backend/runtime.rs:314`, `src/backend/runtime.rs:1116`).
- `PtnStringOperand` already has `data`, `owned`, and `len`
  (`src/backend/runtime.rs:108`), but it is only a transient concat helper. Its
  constructors still derive length with `strlen()`
  (`src/backend/runtime.rs:1929`, `src/backend/runtime.rs:1937`), and
  `ptn_concat()` returns a plain string pointer after appending a trailing NUL
  (`src/backend/runtime.rs:1976`).
- Runtime cleanup currently frees symbol and constant names, not stored
  `PtnValue` payloads (`src/backend/runtime.rs:490`,
  `src/backend/runtime.rs:759`). Any first string-storage slice must avoid
  adding value freeing unless it also introduces safe copy/refcount semantics
  for values stored in variables, constants, arrays, and function returns.

The current representation means embedded NUL bytes are stored in allocated
buffers but become invisible to any helper that calls `strlen()`, `strcmp()`,
`strstr()`, `fputs()`, or other C-string APIs.

## Affected Runtime Paths

High-priority binary-observable paths:

- Output and dumping: `ptn_echo()` uses `fputs()` for strings
  (`src/backend/runtime.rs:2292`), and `var_dump()` prints string length with
  `strlen()` before `fputs()` (`src/backend/runtime.rs:2322`).
- String conversion and concat: `ptn_value_to_string()` duplicates strings
  with `strlen()` (`src/backend/runtime.rs:1902`), while
  `ptn_value_to_string_operand()` and `ptn_concat()` are already shaped like
  the right abstraction but still derive stored string length from NUL
  termination (`src/backend/runtime.rs:1949`, `src/backend/runtime.rs:1976`).
- Byte length and byte observation: `strlen()`, `bin2hex()`, `ord()`, and
  string offset lookup all recompute length with `strlen()`
  (`src/backend/runtime.rs:2383`, `src/backend/runtime.rs:3107`,
  `src/backend/runtime.rs:3592`, `src/backend/runtime.rs:1294`).
- Byte comparison/search: `strcmp()`, scalar equality/identity/order, and
  `str_contains()` rely on `strcmp()` or `strstr()`
  (`src/backend/runtime.rs:1428`, `src/backend/runtime.rs:1448`,
  `src/backend/runtime.rs:1467`, `src/backend/runtime.rs:1530`,
  `src/backend/runtime.rs:2424`, `src/backend/runtime.rs:2442`).
- Byte transforms and decoders: bitwise string operations, `str_rot13()`,
  `quotemeta()`, `chunk_split()`, `strip_tags()`, `md5()`, `sha1()`,
  `substr()`, `hex2bin()`, `quoted_printable_decode()`, and `soundex()` use
  NUL-terminated inputs or outputs (`src/backend/runtime.rs:1780`,
  `src/backend/runtime.rs:2393`, `src/backend/runtime.rs:2510`,
  `src/backend/runtime.rs:2542`, `src/backend/runtime.rs:2590`,
  `src/backend/runtime.rs:2801`, `src/backend/runtime.rs:2903`,
  `src/backend/runtime.rs:2960`, `src/backend/runtime.rs:3144`,
  `src/backend/runtime.rs:3184`, `src/backend/runtime.rs:3286`).

Lower-priority paths that can stay C-string based initially:

- Runtime variable, constant, internal-function, and user-function names are
  identifiers rather than PHP string values. They should continue using
  NUL-terminated `const char *` names until PHP adds binary-sensitive dynamic
  symbol semantics.
- Numeric/base conversion helpers may accept length-aware views, but they can
  internally copy to a temporary NUL-terminated buffer until exact numeric
  string classification work broadens.
- Array string keys need length-aware storage for complete PHP string-key
  parity, but the safest first slice can preserve current key behavior and
  explicitly document that array-key NUL parity remains blocked.

## Target Shape

Introduce a generated C string value type with an explicit byte length and a
trailing NUL compatibility byte:

```c
typedef struct {
    const unsigned char *data;
    size_t len;
    unsigned char *owned;
} PtnString;
```

Then change `PtnValue.as.string` from `const char *` to `PtnString`. Keep the
trailing NUL invariant for temporary interoperability with existing C APIs, but
make the length authoritative.

Suggested helper surface:

- `ptn_string_z(const char *data)` for static NUL-terminated runtime literals.
- `ptn_string_bytes(const unsigned char *data, size_t len)` for static
  byte slices emitted by Rust codegen.
- `ptn_owned_string_z(char *data)` for existing owned C-string producers during
  migration.
- `ptn_owned_bytes(unsigned char *data, size_t len)` for binary-producing
  helpers such as `chr()`, `hex2bin()`, raw digests, bitwise strings, and
  quoted-printable decoding.
- `ptn_duplicate_bytes(const unsigned char *data, size_t len)` as the generic
  allocation primitive.
- `PtnStringView` or a revised `PtnStringOperand` for scalar string conversion
  results. It should carry `{ data, len, owned }` and free only conversion
  buffers, not borrowed `PTN_STRING` values.

Do not add broad `ptn_value_free()` in the first slice. Current values are
copied by value through arrays, symbol tables, constants, function returns, and
temporary expressions. Freeing owned payloads safely belongs with a later
reference-counting/copy-on-write ownership slice.

## Migration Strategy

1. Storage primitive only:
   - Add `PtnString` and constructor helpers.
   - Update `PTN_STRING` field access sites to compile using `.data` and
     `.len`.
   - Preserve existing trailing-NUL buffers and current leak profile.
2. Output and length proof:
   - Convert `ptn_echo()` and `var_dump()` string output to `fwrite(data, 1,
     len, stdout)`.
   - Convert `ptn_value_to_string_operand()`, `ptn_concat()`, `strlen()`,
     `bin2hex()`, `ord()`, and `ptn_string_offset_lookup()` to use stored
     lengths.
   - Convert `chr()` to create a one-byte `PtnString` with `len == 1` even
     when the byte is `0`.
3. Byte compare/search:
   - Replace string equality/identity/order helpers with `memcmp()` plus
     length tie-breaks.
   - Replace `strstr()` in `str_contains()` with a bounded byte search.
   - Convert `str_starts_with()` and `str_ends_with()` to stored lengths.
4. Byte transforms/decoders:
   - Migrate bitwise string operations, `str_rot13()`, `quotemeta()`,
     `chunk_split()`, `strip_tags()`, `substr()`, `hex2bin()`,
     `quoted_printable_decode()`, raw digest return values, and digest input
     hashing to length-aware byte buffers.
5. Ownership and copy-on-write:
   - Add value destruction, string refcounts, array value destruction, and
     copy-on-write only when value ownership semantics are designed together.
   - At that point, update `ptn_runtime_free()` and symbol/array replacement
     paths to release overwritten values safely.

## First Safe Implementation Beads

Suggested first bead:

> Introduce length-aware `PTN_STRING` storage and migrate output/length byte
> observers.

Scope:

- Change `PtnValue.as.string` to a `PtnString` carrying `data`, `len`, and an
  optional `owned` pointer.
- Migrate `ptn_string()`, `ptn_owned_string()`, `ptn_value_to_string_operand()`,
  `ptn_concat()`, `ptn_echo()`, `var_dump()`, `strlen()`, `bin2hex()`, `ord()`,
  `chr()`, and string offset reads.
- Keep array string keys, variable/constant names, function names, and
  ownership freeing unchanged.
- Add native tests for:
  - `strlen(chr(0)) == 1`;
  - `bin2hex(chr(0) . "A") == "0041"`;
  - `var_dump(chr(0))` reporting `string(1)`;
  - `ord(chr(0)) == 0`;
  - `echo "A", chr(0), "B"` producing three stdout bytes.

Suggested follow-up beads:

- Migrate byte comparisons and byte searches for `strcmp()`, equality,
  identity, ordering, `str_contains()`, `str_starts_with()`, and
  `str_ends_with()`.
- Migrate binary-producing transforms and decoders: bitwise string ops,
  `hex2bin()`, `quoted_printable_decode()`, raw `md5()`/`sha1()`, `substr()`,
  `quotemeta()`, `chunk_split()`, and `strip_tags()`.
- Design value destruction, string ownership, and future copy-on-write so
  runtime cleanup can free owned strings without double-freeing shallow copies.
- Make array string keys length-aware once arrays, mutation, references, and
  copy-on-write have a shared ownership story.

## Tests And PHPT Candidates

Native proof should use generated binaries and byte-level stdout checks rather
than PHP interpreter dependence. The current Rust test harness already supports
checking raw `execution.stdout`, so the first bead can assert bytes directly.

Candidate public rows or row clusters to re-evaluate after the storage and
search/transform follow-ups:

- string internal rows that combine `chr(0)` with `strlen()`, `bin2hex()`,
  `ord()`, `strcmp()`, `str_contains()`, `str_starts_with()`, and
  `str_ends_with()`;
- raw-output digest rows for `md5()` and `sha1()`;
- `hex2bin()` and `quoted_printable_decode()` rows where decoded output
  contains embedded NUL bytes;
- `substr()`, string-offset, bitwise string, and `var_dump()` rows whose
  observable lengths currently stop at the first NUL.

Treat these as candidate telemetry, not guaranteed passes. Several rows also
need unrelated PHP surface such as richer diagnostics, functions, try/catch,
or array/object/reference behavior.

## Compatibility And Performance Risks

- `PtnValue` will grow if `PtnString` is stored inline. That can increase copy
  cost for arrays, variables, constants, and function returns. A future
  refcounted string pointer may be better once copy-on-write work starts.
- Keeping a trailing NUL costs one extra byte per allocated string, but it lets
  unmigrated helpers be converted gradually while the stored length remains
  authoritative.
- Replacing `strlen()` with stored lengths should improve hot paths such as
  concat, `strlen()`, `bin2hex()`, prefix/suffix checks, and digest inputs by
  avoiding repeated scans.
- Replacing `strcmp()`/`strstr()` with bounded byte helpers may lose libc
  optimizations if implemented naively. Keep helpers simple first, then measure
  search-heavy rows and native benchmarks before optimizing.
- Adding value freeing before reference/copy semantics are ready is the highest
  correctness risk because current runtime values are shallow-copied in many
  places. Preserve the current cleanup behavior until ownership is designed.
