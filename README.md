# PTN From Scratch

PTN compiles a PHP subset into native binaries:

`PHP source -> lexer/parser -> AST -> IR -> C runtime -> native executable`

Rule: implement reusable PHP semantics. Do not special-case PHPT filenames,
rows, or outputs.

## Shape

- Rust crate and `phpc` compiler binary.
- Boxed C runtime for PHP-like values.
- Native tests cover parser, IR, backend, runtime, and PHP behavior.
- Strings cover direct and braced interpolation, legacy `${name}`
  deprecations, common/control escapes, hex/octal byte escapes, and inline HTML
  through shared output paths.
- Top-level user functions and declared class methods include scoped magic
  constants, call-frame introspection, scalar type hints, literal-array
  defaults, by-reference return aliases, typed coercion, and public constructor
  dispatch and metadata intrinsics.
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
  exponent spelling across echo, casts, concatenation, and string internals.
- Array internals including set operations, `array_chunk()`, `array_combine()`,
  `array_filter()`, `array_key_exists()`, `array_merge()`, `array_pop()`,
  `array_push()`, `array_shift()`, `array_unshift()`, `asort()`, `ksort()`,
  `sort()`, and `shuffle()` use shared ordered-array and COW paths.
- `var_export()` covers scalars, arrays, declared objects through
  `__set_state(array(...))`, and `stdClass` through `(object) array(...)`.
- `pow()` uses the same boxed numeric exponentiation helper as `**`, and
  `call_user_func_array()` expands ordered arrays through callable dispatch.
- `assert()` throws catchable `AssertionError` values with compiler-generated
  default messages for direct calls; bounded `highlight_file()` shares
  file-return paths.
- Direct variable and array-offset increment/decrement support statement and
  expression pre/post forms over the current boxed numeric slice.
- Direct variable and variable-root array/append compound assignments share
  boxed operators and return assigned values.
- `join()` concatenates ordered-array values, and bounded scalar `sprintf()`
  covers the current `%s`, integer, unsigned/hex/oct, float, and `%%` formats.
- Declared instance properties accept public, protected, and private defaults,
  preserve private/protected dump metadata, and support quiet property
  `isset()`, `empty()`, and expression-form `??` probes; full visibility and
  inheritance remain bounded.
- Basic stream resources from `fopen()`/`fclose()` are boxed runtime values with
  current type, dump, and array-key cast behavior.
- Bounded PHPT telemetry uses `PHP_SRC_PHPT`, `/home/claude/php-src-phpt`, or
  `.runtime/php-src-phpt`.

## Status

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
properties, declared instance-property defaults, public property `??=`, and
property/static-property inc/dec.
