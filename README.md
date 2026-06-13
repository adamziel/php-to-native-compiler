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
  introspection, scalar, array, and void return type hints, array defaults,
  by-reference returns, typed coercion, constructor dispatch, public destructor
  dispatch, inherited static call dispatch, and metadata intrinsics.
- Includes share caller file scope and return values; bounded dynamic
  include/require dispatch uses canonical once guards when candidate string
  paths are statically enumerable.
  qualified names, `__NAMESPACE__`, and simple/grouped class/function/const
  imports.
- Full and short ternary expressions lower through lazy boxed branches;
  unparenthesized nested ternaries remain diagnostic.
- Statement-form `(void)` casts evaluate operands for side effects and discard
  results while expression-context `(void)` remains diagnostic.
- Direct references and by-reference parameters cover the first COW/reference
  slice; dynamic roots support reads/writes, array/string-offset writes,
  unsets, compounds, null coalescing assignments, and inc/dec targets.
- Arithmetic models non-numeric string/array `TypeError`s while preserving
  leading-numeric warnings; float stringification honors `phpc -d precision=N`
  and PHP-style exponent spelling.
- Array internals cover set/key/list/search/unique/slice/pad/reverse/
  sum/product/fill/filter/chunk/merge/replace helpers, key-aware
  diff/intersect helpers with user comparators, sum/product warning and
  overflow parity,
  `count()`/`sizeof()` modes, and sort mutators through ordered-array/COW
  paths.
- `var_export()` covers scalars, arrays, declared objects through
  `__set_state(array(...))`, `stdClass` through `(object) array(...)`, and
  embedded-NUL string escaping.
- `pow()` uses the boxed exponentiation helper, `call_user_func_array()`
  expands ordered arrays through callable dispatch, callback dispatch observes
  `global` bindings for user functions reached through direct calls, and
  public `__invoke` objects can be called directly or through callback
  dispatch; `is_callable()` writes callable-name output for supported callable
  shapes.
- `assert()` throws catchable `AssertionError`; bounded `highlight_file()`
  shares file-return paths.
- Modeled metadata includes `phpversion()`, `php_sapi_name()`,
  `zend_version()`, PHP version/build/platform constants, `PHP_SAPI`,
  `get_loaded_extensions()`, stable PHP locale constants, bounded
  `setlocale()`/`localeconv()` helpers, `spl_object_id()`/
  `spl_object_hash()`, and bounded class/property existence checks.
- Direct variable, array-offset, property, and static-property inc/dec support
  statement and expression pre/post forms over boxed PHP values.
- Direct variable, variable-root array/append, property, and static-property
  compounds share boxed operators and return assigned values.
- `join()` concatenates ordered-array values, `explode()`, `str_split()`, and
  `nl2br()` handle length-aware scalar strings, `strncmp()`/`strncasecmp()`
  compare bounded byte prefixes, bounded
  `sprintf()`/`printf()` cover common formats, and `json_encode()` covers
  current boxed values.
- `strpos()`/`stripos()` and `strrpos()`/`strripos()` use length-aware byte
  search with PHP offset bounds; `strstr()`/`stristr()` return binary-safe
  slices and `substr_count()` counts non-overlapping byte matches.
- `str_pad()` supports byte-length padding with pad constants; `strrev()`
  preserves embedded NULs; `basename()` handles binary-safe path segments and
  suffix stripping; `pathinfo()` returns binary-safe dirname/basename/
  extension/filename components with `PATHINFO_*` flags; `crc32()` computes
  length-aware CRC-32 integers; `strpbrk()` returns binary-safe suffix slices;
  trim-family internals, including the `chop()` alias, use PHP default bytes
  plus bounded charlists.
- `str_replace()` supports scalar and ordered-array search/replacement/subject
  byte replacement, dereferences array entries, preserves subject-array keys,
  writes the optional `$count` argument by reference, and throws PHP-style
  TypeErrors for invalid resource/object operands and scalar-search/
  array-replace calls.
- Formatted output covers bounded `sprintf()`/`printf()` plus `fprintf()`,
  `vsprintf()`, `vprintf()`, and `vfprintf()` through one ordered-array
  argument expansion and stream-write path; exact PHP parity for some
  formatter flags and error diagnostics remains bounded.
- Declared instance properties keep public/protected/private defaults, dump
  metadata, quiet `isset()`, `empty()`, and `??`, and inherited parent-private
  slots distinct from child public redeclarations; `property_exists()` covers
  the current declared/static property metadata and stdClass dynamic slots.
  Public `__destruct()` runs on last-reference release and shutdown. Full
  visibility and inheritance remain bounded.
- Static properties support reads/writes, compounds, `??=`, plus quiet
  `isset()`, `empty()`, and `??`.
- Public class constants support scalar/array defaults, direct
  `Class::CONST`/`self::CONST` reads, and `constant()`/`defined()` lookup;
  typed/non-public/inherited/dynamic constants remain bounded.
- Stream resources from `STDIN`/`STDOUT`/`STDERR` and `fopen()`/`fclose()` are
  boxed with type, dump, and array-key cast behavior; `file_get_contents()`
  reads filesystem paths into binary-safe strings with bounded offset/length
  handling.
- Environment and include-path helpers cover `getenv()` snapshots/lookups,
  `putenv()` set/unset plus embedded-NUL/invalid-assignment diagnostics, and
  bounded `get_include_path()`/`set_include_path()`/`ini_restore()` state.
- Bounded PHPT telemetry uses `PHP_SRC_PHPT`, `/home/claude/php-src-phpt`, or
  `.runtime/php-src-phpt`.
- PHPT runners preclassify broad rows before execution and write selected,
  runnable, classification, excluded, and per-category manifests under
  `.runtime/phpt-progress`. Defaults model PTN's current `Core`, `date`,
  `pcre`, `Reflection`, and `standard` extension surface plus accepted runner ini keys
  (`date.timezone`, `display_errors`, `error_reporting`, `extension_dir`,
  `include_path`, `pcre.backtrack_limit`, `precision`, and
  `zend.assertions`); child-process control rows are classified until PTN has a
  native process boundary, and broad unsupported language surfaces such as
  anonymous classes, interfaces/traits, and call-site/array unpacking are
  classified with source evidence. `PTN_PHPT_CLASSIFY=0` gives raw php-src
  `run-tests.php` pass-through.
- Broad PHPT baseline telemetry can generate 1k/5k/10k manifests from
  `Zend/tests`, `ext/standard/tests`, and core `tests`, recording the php-src
  corpus revision plus pass/fail/skip/warn counts without requiring all rows to
  pass.

## Status

- `PROGRESS.md`: compact test/porting dashboard.
- `STATUS.md` and generated mirrors: current operating status.
- After changing `PROGRESS.md`, run `tools/update-progress-mirrors.sh` and
  commit the regenerated `progress.md`, `progress.html`, `STATUS.md`, and
  `STATUS.html` files.

## Commands

```bash
cargo test
tools/run-native-smoke-matrix.sh
tools/run-post-merge-cow-gate.sh
cargo build --bin phpc
tools/update-progress-mirrors.sh
tools/run-phpt-manifest.sh tools/phpt-manifest-200.txt
tools/run-phpt-manifest.sh tools/phpt-include-manifest.txt
tools/run-phpt-baseline.sh --generate-only
tools/run-phpt-baseline.sh --tier 1000
```
