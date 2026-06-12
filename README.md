# PTN From Scratch

PTN compiles PHP subsets into native binaries:

`PHP source -> lexer/parser -> AST -> IR -> C runtime -> native executable`

Rule: implement reusable PHP semantics; no PHPT row special-cases.

## Shape

- Rust crate and `phpc` compiler binary.
- Boxed C runtime for PHP-like values.
- Native tests cover parser, backend, runtime, and PHP behavior.
- Strings cover interpolation, legacy `${name}` deprecations, common/control
  escapes, hex/octal byte escapes, and inline HTML output.
- Top-level functions and declared methods include magic constants, call-frame
  introspection, scalar type hints, array defaults, by-reference returns, typed
  coercion, and constructor dispatch/metadata intrinsics.
- Includes share caller file scope and return values; bounded dynamic
  include/require path expressions dispatch to compiled helpers when all
  candidate string paths are statically enumerable.
- Unbracketed namespaces resolve current top-level functions/constants/classes,
  qualified names, `__NAMESPACE__`, and simple class/function/const imports.
- Full and short ternary expressions lower through the boxed value path with
  lazy branch evaluation; unparenthesized nested ternaries remain an explicit
  diagnostic boundary.
- Direct variable references, array element references, and by-reference
  userland parameters cover the first COW/reference boundary slice.
- Dynamic variable roots support scalar reads/writes, array/string-offset
  writes and unsets, and inc/dec expression targets through shared
  symbol-table and array-path helpers.
- Arithmetic models non-numeric string/array `TypeError` diagnostics while
  preserving leading-numeric warnings.
- Scalar float stringification honors `phpc -d precision=N` and PHP-style
  exponent spelling across output and string conversions.
- Array internals including set operations, `count()`/`sizeof()` with current
  array modes, `array_chunk()`, `array_combine()`, `array_filter()`,
  `array_key_exists()`, `array_keys()`, key-boundary
  probes, `array_merge()`, `array_pad()`, `array_pop()`, `array_product()`,
  `array_push()`, `array_shift()`, `array_unshift()`, `array_search()`,
  `arsort()`, `asort()`, `krsort()`, `ksort()`, `natcasesort()`, `natsort()`,
  `sort()`, `rsort()`, and `shuffle()` use ordered-array/COW paths; direct
  regular sort-family calls also accept an explicit `SORT_REGULAR` flag.
- `var_export()` covers scalars, arrays, declared objects through
  `__set_state(array(...))`, and `stdClass` through `(object) array(...)`.
- `pow()` uses the same boxed numeric exponentiation helper as `**`, and
  `call_user_func_array()` expands ordered arrays through callable dispatch.
- `assert()` throws catchable `AssertionError` values with compiler-generated
  default messages for direct calls; bounded `highlight_file()` shares
  file-return paths.
- Modeled version/SAPI metadata includes `phpversion()`, `php_sapi_name()`,
  `zend_version()`, `PHP_VERSION`, `PHP_SAPI`, and `get_loaded_extensions()`
  for the current CLI/core/standard boundary.
- Direct variable, array-offset, property, and static-property
  increment/decrement support statement and expression pre/post forms over
  boxed numeric values, null, booleans, numeric strings, and alphanumeric
  string increment.
- Direct variable and variable-root array/append compound assignments share
  boxed operators and return assigned values.
- `join()` concatenates ordered-array values, and bounded scalar `sprintf()`
  covers `%s`, integer, unsigned/hex/oct, float, and `%%` formats.
- `str_pad()` supports byte-length padding with pad constants.
- `zend_version()` reports the modeled Zend Engine version alongside the
  existing CLI PHP version metadata.
- `strrev()` reverses current length-aware string operands without losing
  embedded NUL bytes.
- `trim()`, `ltrim()`, and `rtrim()` use length-aware operands, PHP default
  bytes, and bounded charlists.
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

Keep each under 500 words.

## Commands

```bash
cargo test
tools/run-native-smoke-matrix.sh
tools/run-post-merge-cow-gate.sh
cargo build --bin phpc
tools/run-phpt-manifest.sh tools/phpt-manifest-200.txt
tools/run-phpt-manifest.sh tools/phpt-include-manifest.txt
```

## RC Demo

```bash
cargo build --bin phpc
for f in examples/rc/*.php; do
  echo "== $f =="
  target/debug/phpc "$f"
done
```

RC examples exercise scalar control flow, arrays/internals, user functions,
public object shells, static/instance properties, `??=`, and property/static
inc/dec.
