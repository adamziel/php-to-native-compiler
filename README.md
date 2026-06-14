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
  `Closure::fromCallable()` and first-class callable syntax wrap supported
  function, static-method, object-method, and closure callables with callable
  dump/reflection metadata, `Closure::__invoke` reference diagnostics use
  Closure method names at callable boundaries, and userland closure dumps
  expose source name/file/line and parameter metadata.
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
  throw bridging, `Throwable`, `getMessage()`, and `getTrace()` arrays for
  userland and modeled internal callback frames; bounded `highlight_file()`
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
  expectation rows, broad unsupported language surfaces such as anonymous
  classes, interfaces/traits, call-site/array unpacking,
  generator/Fiber execution boundaries including by-reference yields/returns,
  nullable type hints, interpolating heredoc bodies,
  variable variables,
  PHP attribute syntax/reflection metadata,
  class/reflection-metadata blockers,
  readonly static property diagnostics,
  and currently unmodeled mutating array-internal helpers such as
  `array_multisort()`, `usort()`, `uasort()`, and `uksort()` plus
  destructor-reentrant `array_splice()` cases, remaining runtime diagnostics
  and backtrace string APIs, user error/exception handler state,
  `ErrorException` metadata, and assertion runtime modes are mapped to blocker
  categories with source
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
- `docs/PHPT_BROAD_1K_ATTRIBUTE_CLASSIFIER_BUCKET_PTN_J8B8_2026-06-14.md`:
  broad 1k classifier split moving 141 PHP attribute syntax rows into the
  dedicated `unsupported-attribute-metadata` bucket and focused manifest.
- `docs/PHPT_BROAD_1K_ATTRIBUTE_CLASSIFIER_PTN_61F9_2026-06-14.md`:
  parallel broad 1k attribute syntax/reflection classifier split evidence with
  a focused 149-row manifest.
- `docs/PHPT_BROAD_1K_ATTRIBUTE_CLASSIFIER_PTN_B35N_2026-06-14.md`:
  current broad 1k explicit `unsupported-attribute-metadata` classifier split
  for 149 PHP attribute syntax/reflection metadata rows.
- `docs/PHPT_BROAD_1K_DIAGNOSTICS_ASSERTION_FRONTIER_PTN_CUEV_2026-06-14.md`:
  current broad 1k diagnostics/assertion runtime blocker map and focused
  manifest.
- `docs/PHPT_BROAD_1K_UNSUPPORTED_LANGUAGE_FRONTIER_PTN_A0R0_2026-06-14.md`:
  earlier broad 1k unsupported-language bucket split before the explicit
  attribute-metadata split, with 288 rows mapped by generic parser/runtime
  surface.
- `docs/PHPT_BROAD_1K_UNSUPPORTED_LANGUAGE_RESIDUAL_PTN_ROTK_2026-06-14.md`:
  pre-`ptn-18tp` broad 1k post-attribute-split unsupported-language residual
  manifest and blocker map, with 147 rows classified out.
- `docs/PHPT_BROAD_1K_LANGUAGE_CLASSIFIER_SPLIT_PTN_18TP_2026-06-14.md`:
  current broad 1k language-surface classifier split, moving the remaining
  147-row aggregate into semantic categories while attribute metadata remains
  separate.
- `docs/PHPT_BROAD_1K_LANGUAGE_CLASSIFIER_SPLIT_PTN_SU8H_2026-06-14.md`:
  reconciliation note for the older language-split MR against the current
  post-`ptn-18tp` bucket names.
- `docs/PHPT_BROAD_1K_POST_LANGUAGE_SPLIT_PTN_H5QY_2026-06-14.md`:
  transitional broad 1k post-language-split evidence, recorded before later
  class-metadata classifier refinements, with 424 runnable rows and 147
  language/runtime blockers replayed across seven semantic categories.
- `docs/PHPT_BROAD_1K_STANDARD_ARRAY_CURRENT_FRONTIER_2026-06-14.md`:
  refreshed broad 1k standard-array runnable frontier, focused manifest, and
  current 53-row residual blocker map.
- `docs/PHPT_BROAD_1K_STANDARD_ARRAY_PTN_0YN0_2026-06-14.md`: refreshed
  broad 1k standard-array blocker map on the current 294-row focused runnable
  set, with 243 passing and 51 residual rows split by runtime primitive.
- `docs/PHPT_BROAD_1K_STANDARD_ARRAY_RESIDUAL_PTN_KE94_2026-06-14.md`:
  current broad 1k standard-array residual blocker map after recent array and
  binary-string movement.
- `docs/PHPT_BROAD_1K_STANDARD_ARRAY_G7C1_BLOCKER_MAP_2026-06-14.md`:
  refreshed broad 1k standard-array blocker map for the current 294-row
  runnable subset, including the two rows now classified out of the older
  296-row frontier.
- `docs/PHPT_BROAD_1K_UNPACKING_BLOCKERS_2026-06-14.md`: broad 1k
  argument/array unpacking blocker map.
- `docs/PHPT_BROAD_1K_CALL_UNPACKING_CATEGORY_PTN_WD68_2026-06-14.md`:
  current broad 1k `unsupported-call-unpacking` category map with a committed
  34-row focused manifest.
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
- `docs/PHPT_BROAD_1K_CLASS_DECLARATION_CATEGORY_PTN_BO7Q_2026-06-14.md`:
  broad 1k class-declaration aggregate map, superseded by the later explicit
  trait/interface/implementation/anonymous-class classifier split.
- `docs/PHPT_BROAD_1K_CLASS_DECLARATION_CLASSIFIER_PTN_ZXE0_2026-06-14.md`:
  reconciliation note for the older class-declaration classifier MR against
  the aggregate class-declaration bucket evidence.
- `docs/PHPT_BROAD_1K_CLASS_DECLARATION_CLASSIFIER_PTN_LNXT_2026-06-14.md`:
  reconciliation note for the stale class-declaration classifier split against
  the current explicit trait/interface/implementation/anonymous-class buckets.
- `docs/PHPT_BROAD_1K_CLASS_DECLARATION_SPLIT_PTN_GKVR_2026-06-14.md`:
  current broad 1k class-declaration classifier split for trait, interface,
  implementation-check, and anonymous-class blocker categories.
- `docs/PHPT_BROAD_1K_PLAIN_HEREDOC_CLASSIFIER_2026-06-14.md`: broad 1k
  plain heredoc/nowdoc classifier refinement and focused PHPT evidence.
- `docs/PHPT_BROAD_1K_MAGIC_METADATA_BLOCKERS_2026-06-14.md`: current broad
  1k magic method metadata blocker map.
- `docs/PHPT_BROAD_1K_MAGIC_OBJECT_CONVERSION_FRONTIER_2026-06-14.md`: broad
  1k magic-method/object-conversion blocker map and raw focused evidence.
- `docs/PHPT_BROAD_1K_MAGIC_METHOD_METADATA_CURRENT_PTN_7FYM_2026-06-14.md`:
  current broad 1k `unsupported-magic-method-metadata` category map with a
  committed 69-row focused manifest.
- `docs/PHPT_BROAD_1K_MAGIC_METHOD_CLASSIFIER_PTN_U889_2026-06-14.md`:
  broad 1k classifier split evidence moving 69 magic-method metadata rows out
  of the aggregate class-metadata bucket, with current classifier tests.
- `docs/PHPT_BROAD_1K_MAGIC_METHOD_CLASSIFIER_PTN_LMO1_2026-06-14.md`:
  reconciliation note for the stale magic-method classifier split against the
  current `unsupported-magic-method-metadata` category evidence.
- `docs/PHPT_BROAD_1K_UNSUPPORTED_LANGUAGE_FRONTIER_2026-06-14.md`: earlier
  broad 1k unsupported-language blocker map and focused classifier evidence.
- `docs/PHPT_BROAD_1K_NONARRAY_CLASS_METADATA_FRONTIER_2026-06-14.md`: broad
  1k non-array class/object metadata blocker map and focused manifest.
- `docs/PHPT_BROAD_1K_CLASS_METADATA_CURRENT_FRONTIER_2026-06-14.md`: current
  broad 1k combined class/object metadata blocker map and focused manifest.
- `docs/PHPT_BROAD_1K_CLASS_OBJECT_METADATA_GRANULAR_PTN_GT7B_2026-06-14.md`:
  current broad 1k granular class/object metadata blocker map and 135-row
  focused manifest after the aggregate metadata bucket split.
- `docs/PHPT_BROAD_1K_CLASS_OBJECT_METADATA_ROLLUP_PTN_XYMV_2026-06-14.md`:
  current broad 1k class-like parser and object metadata blocker rollup.
- `docs/PHPT_BROAD_1K_CLASS_METADATA_SPLIT_PTN_0FMH_2026-06-14.md`: broad
  1k class/object metadata classifier split, focused 143-row manifest, and
  current magic-method/visibility/property metadata blocker counts.
- `docs/PHPT_BROAD_1K_REQUEST_SAPI_FRONTIER_2026-06-14.md`: broad 1k
  request/SAPI input boundary blocker map and raw focused evidence.
- `docs/PHPT_BROAD_1K_RUNTIME_INI_FRONTIER_2026-06-14.md`: broad 1k
  runtime INI/configuration blocker map and focused manifest.
- `docs/PHPT_BROAD_1K_RUNTIME_BOUNDARY_FRONTIER_PTN_YPMU_2026-06-14.md`:
  broad 1k request/SAPI, harness, process, environment, and host-path runtime
  boundary blocker map and focused manifest.
- `docs/PHPT_BROAD_1K_RUNTIME_BOUNDARY_CURRENT_PTN_2QQ9_2026-06-14.md`:
  current post-split broad 1k runtime/configuration/diagnostics boundary map
  with a committed 144-row focused manifest.
- `docs/PHPT_BROAD_1K_RUNTIME_CONFIG_BOUNDARY_PTN_3QJ5_2026-06-14.md`:
  focused 103-row runtime configuration, request/SAPI, assertion,
  process/harness, host-precondition, and resource-limit subset of the broader
  runtime boundary map.
- `docs/PHPT_BROAD_1K_ARRAY_OBJECT_METADATA_FRONTIER_2026-06-14.md`: broad
  1k standard-array object/magic/visibility metadata blocker map.
- `docs/PHPT_BROAD_1K_ARRAY_KEY_VALUE_FRONTIER_2026-06-14.md`: broad 1k
  standard-array key/value helper blocker map and focused manifest.
- `docs/PHPT_BROAD_1K_CLASS_OBJECT_METADATA_CLUSTER_PTN_IVFW_2026-06-14.md`:
  current broad 1k class/object metadata cluster blocker map with 362 focused
  classified rows.
- `docs/PHPT_BROAD_1K_ARRAY_CALLBACK_FRONTIER_PTN_J9KG_2026-06-14.md`: current
  broad 1k array callback/set-operation blocker map and committed focused
  manifest.
- `docs/PHPT_BROAD_1K_ZEND_BUG_REGRESSION_FRONTIER_2026-06-14.md`: broad 1k
  root-level Zend historical bug-regression blocker map and focused manifest.
- `docs/PHPT_BROAD_1K_ZEND_ROOT_CURRENT_PTN_XGK8_2026-06-14.md`: current
  broad 1k root-level Zend execution map with focused manifest and residual
  array/lvalue, object, control, and dispatch blocker buckets.
- `docs/PHPT_BROAD_1K_FUNCTION_DYNAMIC_TYPE_BLOCKERS_2026-06-14.md`: broad
  1k function-local static, nullable/never type, and variable-variable blocker
  map.
- `docs/PHPT_BROAD_1K_FUNCTION_DYNAMIC_CURRENT_PTN_RLZZ_2026-06-14.md`:
  current broad 1k 35-row function/dynamic type blocker map and focused
  manifest.
- `docs/PHPT_BROAD_1K_FUNCTION_DYNAMIC_TYPE_PTN_1TCO_2026-06-14.md`:
  current broad 1k function/dynamic-type focused manifest and blocker map.
- `docs/PHPT_BROAD_1K_CORE_BASIC_OPERATOR_FRONTIER_2026-06-14.md`: broad 1k
  core/basic operator-control-flow blocker map and focused manifest.
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
- `docs/PHPT_BROAD_1K_EXTENSION_ENVIRONMENT_FRONTIER_2026-06-14.md`: broad
  1k unavailable-extension, process-boundary, host-precondition, external
  service, and PHPT environment setup blocker map.
- `docs/PHPT_BROAD_1K_CURRENT_RESIDUAL_PTN_JD95_2026-06-14.md`: prior broad
  1k residual coverage map reconciling runnable rows against committed focused
  manifests and isolating `ErrorException::getSeverity()` as the then-current
  residual runner-runnable diagnostics row.
- `docs/PHPT_BROAD_1K_CLUSTER_SLICE_PTN_DBFC_2026-06-14.md`: refreshed broad
  1k cluster slice showing no single credible current implementation cluster
  reaches the 25-row target; records current classifier and blocker counts.
- `docs/PHPT_BROAD_1K_CURRENT_COVERAGE_PTN_FPG4_2026-06-14.md`: refreshed
  broad 1k coverage map after the explicit 149-row attribute metadata split,
  with 424 runnable rows reconciled against committed focused manifests and
  zero unmatched runnable rows.
- `docs/PHPT_BROAD_1K_CLUSTER_SLICE_PTN_U5MR_2026-06-14.md`: refreshed broad
  1k cluster blocker map showing the current 424 runnable rows are fully
  covered by committed focused manifests.
- `docs/PHPT_BROAD_1K_CLUSTER_SLICE_PTN_KMW3_2026-06-14.md`: current broad
  1k cluster blocker map with the granular classifier split, focused-manifest
  reconciliation, and next credible implementation frontiers.
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
tools/run-phpt-manifest.sh tools/phpt-first-class-callable-manifest.txt
tools/run-bounded-phpt.sh tools/phpt-array-callback-validation-manifest.txt
tools/run-phpt-baseline.sh --generate-only
tools/run-phpt-baseline.sh --tier 1000 --classify-only
tools/run-phpt-baseline.sh --tier 1000
```
