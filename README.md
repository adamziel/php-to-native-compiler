# PTN From Scratch

PTN compiles a PHP subset into native binaries:

`PHP source -> lexer/parser -> AST -> IR -> C runtime -> native executable`

Rule: implement reusable PHP semantics; do not special-case PHPT rows.

## Shape

- Rust crate and `phpc` compiler binary.
- Boxed C runtime for PHP-like values.
- Native tests cover parser, backend, runtime, and PHP behavior.
- Strings cover interpolation, legacy `${name}` deprecations, common/control
  escapes, hex/octal byte escapes, and inline HTML output.
- Top-level functions and declared methods include magic constants, call-frame
  introspection, scalar type hints, array defaults, by-reference returns, typed
  coercion, and constructor dispatch/metadata intrinsics.
- Full and short ternary expressions lower through the boxed value path with
  lazy branch evaluation; unparenthesized nested ternaries remain an explicit
  diagnostic boundary.
- Direct variable references, array element references, and by-reference
  userland parameters cover the first COW/reference boundary slice.
- Dynamic variable roots support scalar reads/writes, array/string-offset
  writes and unsets, and inc/dec expression targets through shared
  symbol-table and array-path helpers.
- Arithmetic models non-numeric string and array `TypeError` diagnostics while
  preserving leading-numeric warnings.
- Scalar float stringification honors `phpc -d precision=N` and PHP-style
  exponent spelling across output and string conversions.
- Array internals including set operations, `array_chunk()`, `array_combine()`,
  `array_filter()`, `array_key_exists()`, `array_keys()`, `array_search()`,
  key/list probes, `array_merge()`, `array_pop()`, `array_product()`,
  `array_push()`, `array_shift()`, `array_unshift()`, `arsort()`, `asort()`,
  `krsort()`, `ksort()`, `natcasesort()`, `natsort()`, `sort()`, `rsort()`,
  and `shuffle()` use
  ordered-array/COW paths.
- `var_export()` covers scalars, arrays, declared objects through
  `__set_state(array(...))`, and `stdClass` through `(object) array(...)`.
- `pow()` uses the same boxed numeric exponentiation helper as `**`, and
  `call_user_func_array()` expands ordered arrays through callable dispatch.
- `assert()` throws catchable `AssertionError` values with compiler-generated
  default messages for direct calls; bounded `highlight_file()` shares
  file-return paths.
- Direct variable, array-offset, property, and static-property
  increment/decrement support statement and expression pre/post forms over
  boxed numeric values, null, booleans, numeric strings, and alphanumeric
  string increment.
- Direct variable and variable-root array/append compound assignments share
  boxed operators and return assigned values.
- `join()` concatenates ordered-array values, and scalar `sprintf()`
  covers `%s`, integer, unsigned/hex/oct, float, and `%%` formats.
- `strrev()` reverses current length-aware string operands without losing
  embedded NUL bytes.
- Declared instance properties keep public/protected/private defaults, dump
  metadata, and quiet `isset()`, `empty()`, and `??`; full visibility and
  inheritance remain bounded.
- Static properties support reads/writes plus quiet `isset()`, `empty()`, and
  `??`.
- Stream resources from `fopen()`/`fclose()` are boxed with type, dump,
  and array-key cast behavior.
- Bounded PHPT telemetry uses `PHP_SRC_PHPT`, `/home/claude/php-src-phpt`, or
  `.runtime/php-src-phpt`.

## Status

- `PROGRESS.md`: compact test and porting dashboard.
- `STATUS.md`: current operating status.
- `progress.md`, `progress.html`, `STATUS.html`: short generated mirrors.

Keep each under 500 words; progress patrol refreshes them periodically.

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
properties, declared instance-property defaults, public property `??=`, and
property/static-property inc/dec.
