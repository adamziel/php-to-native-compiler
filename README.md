# PTN From Scratch

PTN compiles PHP subsets into native binaries:

`PHP source -> lexer/parser -> AST -> IR -> C runtime -> native executable`

Rule: implement reusable PHP semantics; no PHPT row special-cases.

## Shape

- Rust crate and `phpc` compiler binary.
- Boxed C runtime for PHP-like values.
- Native tests cover parser, backend, runtime, and PHP behavior.
- Numeric literals cover digit separators, integer radices, leading-zero
  decimal floats, and leading-dot float forms.
- Strings cover interpolation, legacy `${name}` deprecations, common/control
  escapes including `\e`, hex/octal byte escapes, binary-prefixed string
  literals, and inline HTML output.
- Top-level functions and declared methods include magic constants, call-frame
  introspection, scalar, array, class-name, and void return type hints, array
  defaults, by-reference returns, typed coercion, constructor dispatch, public
  destructor dispatch, inherited static call dispatch, and metadata intrinsics.
- Anonymous closures and arrow functions lower through the shared closure
  runtime, including explicit `use(...)` captures, implicit arrow by-value
  captures, nested capture propagation, by-reference returns, typed parameters,
  variadics, validated `use` lists with trailing commas, and `static fn`
  `$this` exclusion. `Closure::bindTo()` clones preserve captured variables,
  `Closure::fromCallable()` wraps supported callables, and `Closure::__invoke`
  reference diagnostics use Closure method names at callable boundaries.
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
  slice; live by-reference `foreach` iterators keep nested unset/rekey mutation
  parity; dynamic roots support reads/writes, array/string-offset writes,
  unsets, compounds, null coalescing assignments, and inc/dec targets.
- Arithmetic models non-numeric string/array `TypeError`s while preserving
  leading-numeric warnings; float stringification honors `phpc -d precision=N`
  and PHP-style exponent spelling.
- Array internals cover set/key/list/search/unique/slice/pad/reverse/
  sum/product/fill/filter/chunk/merge/replace helpers, predicate/find helpers
  (`array_all()`, `array_any()`, `array_find()`, and `array_find_key()`), key-aware
  diff/intersect helpers with user comparators, `array_rand()` key selection,
  sum/product warning and
  overflow parity,
  `count()`/`sizeof()` modes, `array_splice()` mutation, sort mutators through
  ordered-array/COW paths, a bounded callable `array_multisort()` path with
  prefer-ref argument separation, and recursive/non-recursive `array_walk()`
  callback diagnostics, userdata separation, key snapshots, and mutation
  visibility; `array_fill()` has bounded huge-count allocation diagnostics.
- `var_export()` covers scalars, arrays, declared objects through
  `__set_state(array(...))`, `stdClass` through `(object) array(...)`, and
  embedded-NUL string and string-key escaping.
- `pow()` uses the boxed exponentiation helper, `min()`/`max()` use the shared
  loose ordering helper, `flush()` flushes native stdout, `call_user_func_array()`
  expands ordered arrays through callable dispatch, `call_user_func()` and
  `call_user_func_array()` downgrade fixed-parameter callback by-reference
  mismatches to warnings, invoke user callbacks and strict by-reference array
  mutating internals against by-value locals, continue into userland type
  checks, and resolve scoped class/parent method callable forms with PHP-style
  deprecation diagnostics; callback dispatch observes `global` bindings for
  user functions reached through direct calls, and public `__invoke` objects
  can be called directly or through callback dispatch; `is_callable()` writes
  callable-name output for supported callable shapes.
- `assert()` throws catchable `AssertionError`; userland `throw` statements
  and PHP 8 throw expressions propagate boxed built-in exception values
  through the shared `try`/`catch` runtime, including `Exception`, `Error`
  families, declared `Exception`/`Error` subclasses with message-property
  throw bridging, `Throwable`, and `getMessage()`; bounded `highlight_file()`
  shares file-return paths.
- Modeled metadata includes `phpversion()`, `php_sapi_name()`,
  `zend_version()`, PHP version/build/platform constants, `PHP_SAPI`,
  `get_loaded_extensions()`, stable PHP locale constants, bounded
  `setlocale()`/`localeconv()` helpers, `spl_object_id()`/
  `spl_object_hash()`, `get_class()`/`get_parent_class()` explicit operands
  plus legacy no-argument lexical class scope, `get_called_class()`, Closure
  internal class metadata, Closure-backed `ReflectionFunction` count/name
  metadata, bounded class/interface/trait/property/method existence checks,
  abstract/final class metadata, interface constants, and
  duplicate/non-interface implementation diagnostics.
- Direct variable, array-offset, property, and static-property inc/dec support
  statement and expression pre/post forms over boxed PHP values.
- Direct variable, variable-root array/append, property, and static-property
  compounds share boxed operators and return assigned values; instance
  properties support by-reference assignment and property reference sources.
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
- `array_change_key_case()` preserves binary-safe string-key lengths while
  changing ASCII key case.
- `str_replace()` supports scalar and ordered-array search/replacement/subject
  byte replacement, dereferences array entries, preserves subject-array keys,
  writes the optional `$count` argument by reference, and throws PHP-style
  TypeErrors for invalid resource/object operands and scalar-search/
  array-replace calls.
- Formatted output covers bounded `sprintf()`/`printf()` plus `fprintf()`,
  `vsprintf()`, `vprintf()`, and `vfprintf()` through one ordered-array
  argument expansion and stream-write path; exact PHP parity for some
  formatter flags and error diagnostics remains bounded.
- Declared instance properties keep public/protected/private defaults,
  asymmetric private(set)/protected(set) write visibility metadata, readonly
  write-once metadata for properties and readonly classes, readonly
  uninitialized-read and dynamic-property guards, dump metadata, quiet
  `isset()`, `empty()`, and `??`, and inherited parent-private slots distinct
  from child public redeclarations; `property_exists()` covers the current
  declared/static property metadata and stdClass dynamic slots. Public
  `__destruct()` runs on last-reference release and shutdown. Full visibility,
  property type enforcement, and inheritance remain bounded.
- Static properties support read/write visibility, asymmetric set visibility,
  reads/writes, compounds, `??=`, plus quiet `isset()`, `empty()`, and `??`.
- Public class constants support scalar/array defaults, direct
  `Class::CONST`/`self::CONST` reads, dynamic `$class::CONST` reads, and
  `constant()`/`defined()` lookup; typed/non-public/inherited constants remain
  bounded.
- Stream resources from `STDIN`/`STDOUT`/`STDERR` and `fopen()`/`fclose()` are
  boxed with type, dump, and array-key cast behavior; `file_get_contents()`
  reads filesystem paths into binary-safe strings with bounded offset/length
  handling, `fwrite()`/`fputs()` write stream bytes, and filesystem metadata
  helpers cover `stat()`/`lstat()`, scalar `file*` metadata, `filetype()`,
  `chmod()`/`touch()`, `clearstatcache()`, and readable/writable/executable/link
  path predicates.
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
  native process boundary. Harness cleanup, environment setup, unsupported
  SAPI/stdio/source sections, run-tests self-tests, noisy external/flaky
  expectation rows, and broad unsupported language surfaces such as anonymous
  classes, interfaces/traits, PHP attributes, call-site/array unpacking,
  generator/Fiber execution boundaries including by-reference yields/returns,
  nullable type hints, interpolating heredoc bodies,
  variable variables,
  class/reflection-metadata blockers, readonly static property diagnostics,
  and currently unmodeled mutating array-internal helpers such as
  `array_multisort()`, `usort()`, `uasort()`, and `uksort()` plus
  destructor-reentrant `array_splice()` cases, runtime diagnostics/backtrace
  APIs, user error/exception handler state, `ErrorException` metadata, and
  assertion runtime modes are mapped to blocker categories with source
  evidence; broad baseline runs also opt into `--SKIPIF--` precondition harness
  classification, with static modeling for sanitizer environment gates,
  `PHP_INT_SIZE` comparisons, and host locale availability while arbitrary
  harness PHP remains excluded; plain heredoc/nowdoc literals are classified as
  supported.
  `PTN_PHPT_CLASSIFY=0` gives raw php-src `run-tests.php` pass-through.
- Broad PHPT baseline telemetry can generate 1k/5k/10k manifests from
  `Zend/tests`, `ext/standard/tests`, and core `tests`, recording the php-src
  corpus revision plus pass/fail/skip/warn counts without requiring all rows to
  pass; `--classify-only` writes the same selected/runnable/excluded manifests
  without building or running rows.
- Focused string/scalar alias telemetry covers quiet string-offset
  `isset()`/`empty()` diagnostics, numeric string offsets, string-offset COW,
  concat-assignment aliasing, and classified blocker rows for unsupported
  interpolating heredoc/ini/typed-property/extension surfaces.
- Focused nested foreach/reference telemetry covers live by-reference iterator
  unset/rekey behavior, child-array rekeying through by-reference function
  parameters, and classified plain variable-variable unset blockers.

## Status

- `PROGRESS.md`: compact test/porting dashboard.
- `docs/PHPT_BROAD_1K_ARRAY_FRONTIER_2026-06-13.md`: current broad 1k
  runnable array-frontier blocker map.
- `docs/PHPT_BROAD_1K_DIFF_INTERSECT_FRONTIER_2026-06-14.md`: current broad
  1k `array_diff*`/`array_intersect*` focused evidence and blocker map.
- `docs/PHPT_BROAD_1K_ARRAY_CALLBACK_FRONTIER_2026-06-13.md`: current broad
  1k callback/set-operation array-helper blocker map.
- `docs/PHPT_BROAD_1K_ATTRIBUTE_METADATA_FRONTIER_PTN_LZEF_2026-06-14.md`:
  current broad 1k `Zend/tests/attributes/*` metadata blocker map and focused
  manifest.
- `docs/PHPT_BROAD_1K_DIAGNOSTICS_ASSERTION_FRONTIER_PTN_CUEV_2026-06-14.md`:
  current broad 1k diagnostics/assertion runtime blocker map and focused
  manifest.
- `docs/PHPT_BROAD_1K_UNSUPPORTED_LANGUAGE_FRONTIER_PTN_A0R0_2026-06-14.md`:
  current broad 1k unsupported-language bucket split, with 288 rows mapped by
  generic parser/runtime surface.
- `docs/PHPT_BROAD_1K_STANDARD_ARRAY_CURRENT_FRONTIER_2026-06-14.md`:
  refreshed broad 1k standard-array runnable frontier, focused manifest, and
  current 53-row residual blocker map.
- `docs/PHPT_BROAD_1K_UNPACKING_BLOCKERS_2026-06-14.md`: broad 1k
  argument/array unpacking blocker map.
- `docs/PHPT_BROAD_1K_STANDARD_ARRAY_FRONTIER_2026-06-14.md`: broad 1k
  standard-array runnable frontier blocker map with current family counts.
- `docs/PHPT_BROAD_1K_ARRAY_KEY_COERCION_FRONTIER_2026-06-14.md`: broad 1k
  array key/coercion lookup focused evidence and blocker map.
- `docs/PHPT_BROAD_1K_ZEND_ASSIGNMENT_FRONTIER_2026-06-14.md`: broad 1k
  Zend assignment/reference/object-write blocker map and focused manifest.
- `docs/PHPT_BROAD_1K_HEREDOC_NOWDOC_FRONTIER_2026-06-14.md`: plain
  heredoc/nowdoc broad 1k classifier movement and focused array-helper
  frontier.
- `docs/PHPT_BROAD_1K_CLASS_DECLARATION_FRONTIER_2026-06-14.md`: broad 1k
  interface, trait, implementation-check, and anonymous-class blocker map.
- `docs/PHPT_BROAD_1K_PLAIN_HEREDOC_CLASSIFIER_2026-06-14.md`: broad 1k
  plain heredoc/nowdoc classifier refinement and focused PHPT evidence.
- `docs/PHPT_BROAD_1K_MAGIC_METADATA_BLOCKERS_2026-06-14.md`: current broad
  1k magic method metadata blocker map.
- `docs/PHPT_BROAD_1K_MAGIC_OBJECT_CONVERSION_FRONTIER_2026-06-14.md`: broad
  1k magic-method/object-conversion blocker map and raw focused evidence.
- `docs/PHPT_BROAD_1K_NONARRAY_CLASS_METADATA_FRONTIER_2026-06-14.md`: broad
  1k non-array class/object metadata blocker map and focused manifest.
- `docs/PHPT_BROAD_1K_REQUEST_SAPI_FRONTIER_2026-06-14.md`: broad 1k
  request/SAPI input boundary blocker map and raw focused evidence.
- `docs/PHPT_BROAD_1K_ARRAY_OBJECT_METADATA_FRONTIER_2026-06-14.md`: broad
  1k standard-array object/magic/visibility metadata blocker map.
- `docs/PHPT_BROAD_1K_ARRAY_KEY_VALUE_FRONTIER_2026-06-14.md`: broad 1k
  standard-array key/value helper blocker map and focused manifest.
- `docs/PHPT_BROAD_1K_ARRAY_CALLBACK_FRONTIER_PTN_J9KG_2026-06-14.md`: current
  broad 1k array callback/set-operation blocker map and committed focused
  manifest.
- `docs/PHPT_BROAD_1K_ZEND_BUG_REGRESSION_FRONTIER_2026-06-14.md`: broad 1k
  root-level Zend historical bug-regression blocker map and focused manifest.
- `docs/PHPT_BROAD_1K_FUNCTION_DYNAMIC_TYPE_BLOCKERS_2026-06-14.md`: broad
  1k function-local static, nullable/never type, and variable-variable blocker
  map.
- `docs/PHPT_BROAD_1K_ARRAY_SETOPS_FRONTIER_2026-06-14.md`: broad 1k
  array diff/intersect set-operation blocker map and focused PHPT evidence.
- `docs/PHPT_BROAD_1K_STANDARD_ARRAY_TDEI_2026-06-14.md`: broad 1k
  standard-array evidence, with `array_chunk()` green and residual callback/
  set-operation blocker splits.
- `docs/PHPT_BROAD_1K_ARRAY_RAND_SLICE_2026-06-14.md`: broad 1k
  `array_chunk()` green check, `array_rand()` implementation evidence, and
  remaining heredoc-key blocker.
- `docs/PHPT_BROAD_1K_ASYMMETRIC_VISIBILITY_FRONTIER_2026-06-14.md`: broad
  1k asymmetric property visibility frontier map.
- `docs/PHPT_BROAD_1K_CALLBACK_REGISTRY_SLICE_2026-06-14.md`: broad 1k
  callback registry fix evidence and remaining callback blocker map.
- `docs/PHPT_BROAD_1K_ARRAY_NAMED_CALLBACK_FRONTIER_2026-06-14.md`: current
  broad 1k array named-argument classifier movement plus focused
  `array_map()`/`array_filter()` blocker map.
- `docs/PHPT_BROAD_1K_ZEND_OPERATOR_CONTROL_FRONTIER_2026-06-14.md`: broad
  1k Zend operator/control/AST/assertion blocker map plus the `\e` escape and
  binary-safe array-key movement.
- `docs/PHPT_BROAD_1K_RECURSIVE_DUMP_FRONTIER_2026-06-14.md`: broad 1k
  recursive dump blocker map and `bug35239` fix evidence.
- `docs/PHPT_FILESYSTEM_PATH_PROCESS_FRONTIER_2026-06-14.md`: focused
  filesystem/path/process evidence showing runnable path metadata rows green
  and residual process-boundary/cleanup blocker counts.
- `docs/PHPT_BROAD_1K_RUNTIME_CONFIG_FRONTIER_2026-06-14.md`: broad 1k
  non-request INI/runtime-configuration blocker map and focused manifest.
- `docs/PHPT_BROAD_1K_ASSERTION_RUNTIME_FRONTIER_2026-06-14.md`: broad 1k
  assertion runtime/INI blocker map, with the current focused native assertion
  rows green.
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
tools/run-phpt-manifest.sh tools/phpt-filesystem-path-process-manifest.txt
tools/run-phpt-manifest.sh tools/phpt-string-scalar-alias-manifest.txt
tools/run-phpt-manifest.sh tools/phpt-foreach-nested-ref-manifest.txt
tools/run-bounded-phpt.sh tools/phpt-array-callback-validation-manifest.txt
tools/run-phpt-baseline.sh --generate-only
tools/run-phpt-baseline.sh --tier 1000 --classify-only
tools/run-phpt-baseline.sh --tier 1000
```
