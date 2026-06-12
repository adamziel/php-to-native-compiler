# PTN From Scratch

PTN compiles a growing PHP subset into native binaries:

`PHP source -> lexer/parser -> AST -> IR -> C runtime -> native executable`

Rule: implement reusable PHP semantics. Do not special-case PHPT filenames,
rows, or outputs.

## Current Shape

- Rust crate and `phpc` compiler binary.
- Boxed C runtime for PHP-like values.
- Native tests for parser, IR, backend, runtime, and PHP behavior.
- Double-quoted interpolation covers direct `$name`, simple `$items[$key]`
  offsets, braced `{$name}`/`{$items['key']}`, and deprecated legacy
  `${name}` variables.
- Double-quoted string escapes cover common control escapes plus `\xNN` and
  octal byte escapes.
- Inline HTML before, between, and after PHP blocks lowers through the shared
  output path.
- User functions and declared class methods include scoped magic constants,
  call-frame introspection, scalar type hints, literal-array defaults,
  by-reference return aliases, typed coercion, and public constructors.
- Direct variable references, array element references, and by-reference
  userland parameters cover the first COW/reference boundary slice.
- Dynamic variable roots support scalar and simple array/string-offset writes
  through shared symbol-table and array-path helpers.
- Arithmetic rejects non-numeric strings and mixed array operands with modeled
  catchable `TypeError` diagnostics while preserving leading-numeric warnings.
- Scalar float stringification honors `phpc -d precision=N` and PHP-style
  exponent spelling across echo, casts, concatenation, and string internals.
- `count()` handles arrays and raises catchable `TypeError` diagnostics for
  non-array operands in the boxed value domain.
- `array_chunk()`, `array_combine()`, `array_merge()`, and array set-operation
  internals build fresh ordered arrays through the shared array runtime.
- `array_filter()` preserves keys while filtering arrays by PHP truthiness or
  modeled callbacks.
- `array_key_exists()` handles current ordered arrays, including `null`
  key deprecation and resource-key integer casting.
- `range()` builds integer ranges through the ordered-array runtime.
- `pow()` uses the same boxed numeric exponentiation helper as `**`, and
  `call_user_func_array()` expands ordered arrays through callable dispatch.
- `assert()` throws catchable `AssertionError` values with compiler-generated
  default messages for direct calls; bounded `highlight_file()` shares
  file-return paths.
- `ksort()` and `shuffle()` mutate direct variable arrays through the shared
  ordered-array COW path, and `str_shuffle()` shuffles scalar strings by byte.
- Direct variable and variable-root array-offset increment/decrement support
  statement and expression pre/post forms over the current boxed numeric slice.
- `join()` concatenates ordered-array values, and bounded scalar `sprintf()`
  covers the current `%s`, integer, unsigned/hex/oct, float, and `%%` formats.
- Basic stream resources from `fopen()`/`fclose()` are boxed values with
  `gettype()`, `is_resource()`, `var_dump()`, and array-key casts.
- Bounded PHPT telemetry uses `PHP_SRC_PHPT`, `/home/claude/php-src-phpt`, or
  `.runtime/php-src-phpt`.

## Status Files

- `PROGRESS.md`: compact test and porting dashboard.
- `STATUS.md`: current operating status.
- `progress.md`, `progress.html`, `STATUS.html`: short generated mirrors.

Keep each under 500 words; progress patrol refreshes them about every 10
minutes.

## Commands

```bash
cargo test
tools/run-native-smoke-matrix.sh
tools/run-post-merge-cow-gate.sh
cargo build --bin phpc
tools/run-phpt-manifest.sh tools/phpt-manifest-200.txt
```

## RC Demo

```bash
cargo build --bin phpc
for f in examples/rc/*.php; do
  echo "== $f =="
  target/debug/phpc "$f"
done
```

The corpus exercises the current RC surface: scalar control flow, arrays and
internal functions, user functions, public class/object shells, direct static
properties, and public property `??=`.
