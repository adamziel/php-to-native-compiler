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
  literals, plain heredoc escape decoding with literal nowdoc bodies, and
  inline HTML output.
- Top-level functions and declared methods include magic constants, call-frame
  introspection, scalar, array, class-name, and void return type hints, array
  defaults, by-reference returns, typed coercion, constructor dispatch, public
  destructor dispatch, inherited static call dispatch, `__callStatic()` fallback
  for inaccessible/missing static-style calls, and metadata intrinsics. Magic
  method declaration checks cover modeled arity, staticness, by-reference
  parameter rejection, and required string/array parameter types.
- Anonymous closures and arrow functions lower through the shared closure
  runtime, including explicit `use(...)` captures, implicit arrow by-value
  captures, nested capture propagation, by-reference returns, typed parameters,
  variadics, validated `use` lists with trailing commas, non-static closure
  `$this` binding, and static anonymous/arrow function `$this` exclusion.
  `Closure::bindTo()` clones preserve captured variables,
  `Closure::fromCallable()` and first-class callable syntax wrap supported
  function, static-method, object-method, and closure callables with callable
  dump/reflection metadata, `Closure::__invoke` reference diagnostics use
  Closure method names at callable boundaries, and userland closure dumps
  expose source name/file/line and parameter metadata.
- Yielding userland functions lower to bounded collected `Generator` objects
  that eagerly evaluate the body at call time, preserve yielded values and
  supported by-reference yield diagnostics, expose `Generator::current()` and
  class/method/interface metadata, work in `foreach`, and can be expanded by
  call-site argument unpacking.
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
  parity; object `foreach` covers bounded `Iterator`/`IteratorAggregate`
  protocol dispatch, array-backed SPL iterators, and bounded SPL wrappers
  including `CallbackFilterIterator`, `InfiniteIterator`, `LimitIterator`, and
  `RecursiveArrayIterator`; dynamic roots support
  reads/writes, array/string-offset writes, unsets, compounds, null coalescing
  assignments, and inc/dec targets.
- Arithmetic models non-numeric string/array `TypeError`s while preserving
  leading-numeric warnings; float stringification honors `phpc -d precision=N`,
  var-dump float formatting honors `phpc -d serialize_precision=N`, and both
  use PHP-style exponent spelling.
- Array internals cover set/key/list/search/unique/slice/pad/reverse/
  sum/product/fill/filter/chunk/merge/replace helpers, predicate/find helpers
  (`array_all()`, `array_any()`, `array_find()`, and `array_find_key()`), key-aware
  diff/intersect helpers with user comparators and PHP-style array-to-string
  conversion warnings for set-operation normalization, `array_rand()` key selection,
  `compact()` current-scope variable collection, sum/product warning and
  overflow parity,
  `compact()` symbol-table packing, `count()`/`sizeof()` modes including
  `Countable` dispatch, `array_splice()` mutation, sort mutators through
  ordered-array/COW paths with PHP `SORT_REGULAR` mixed key/value/object
  ordering and sort-time string-conversion diagnostics, a bounded callable
  `array_multisort()` path with
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
  can be called directly or through callback dispatch; array-backed positional
  call-site argument unpacking (`...`) works for direct, dynamic, method, static,
  constructor, and modeled internal calls; `is_callable()` writes callable-name
  output for supported callable shapes.
- `assert()` observes modeled `zend.assertions`, legacy `assert.*`, and
  `assert_options()` runtime state, including callback/warning/bail modes, and
  throws catchable `AssertionError` when exception mode is enabled;
  userland `throw` statements and PHP 8 throw expressions propagate boxed
  built-in exception values
  through the shared `try`/`catch` runtime, including `Exception`, `Error`
  families, declared `Exception`/`Error` subclasses with message-property
  throw bridging, `Throwable`, binary-safe `getMessage()`/`__toString()`,
  `getTrace()` arrays, and `getTraceAsString()` stack formatting for userland
  and modeled internal callback frames; stack argument summaries escape
  non-printable/high-byte string bytes and observe
  `zend.exception_string_param_max_len`. Bounded `highlight_file()` shares
  file-return paths.
- Modeled metadata includes `phpversion()`, `php_sapi_name()`,
  `zend_version()`, PHP version/build/platform constants, `PHP_SAPI`,
  `get_loaded_extensions()`, stable PHP locale constants, bounded
  `setlocale()`/`localeconv()` helpers, `spl_object_id()`/
  `spl_object_hash()`, `get_class()`/`get_parent_class()` explicit operands
  plus legacy no-argument lexical class scope, `get_called_class()`, Closure
  internal class metadata, Closure-backed `ReflectionFunction` count/name
  metadata, bounded `ReflectionClass::isIterateable()` and class/interface/
  trait/property/method existence checks including `trait_exists()`,
  abstract/final class metadata, interface
  constants, and duplicate/non-interface implementation diagnostics.
- Simple trait declarations compose into using classes, including imported
  methods, instance/static properties, and constants. `__TRAIT__` reports the
  source trait for imported methods. Trait adaptations/aliases, precedence
  conflict resolution, direct trait instantiation/reflection, and strict
  abstract/signature/property conflict diagnostics remain bounded.
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
- Output buffering covers a native buffer stack for `ob_start()` callback
  handlers, `ob_get_contents()`, `ob_get_length()`, `ob_get_level()`,
  `ob_list_handlers()`, `ob_end_clean()`, `ob_end_flush()`, `ob_get_clean()`,
  and `ob_get_flush()`; dynamic `compact()` callback invocation is rejected
  with PHP-style `Error` behavior.
- Declared instance properties keep public/protected/private defaults,
  asymmetric private(set)/protected(set) write visibility metadata, readonly
  write-once metadata for properties and readonly classes, readonly
  uninitialized-read and dynamic-property guards, dump metadata, quiet
  `isset()`, `empty()`, `??`, and modeled `__isset()` fallback, and inherited
  parent-private slots distinct from child public redeclarations;
  `property_exists()` covers the current
  declared/static property metadata and stdClass dynamic slots. Public
  `__destruct()` runs on last-reference release and shutdown. Full visibility,
  property type enforcement, and inheritance remain bounded.
- Static properties support read/write visibility, asymmetric set visibility,
  reads/writes, compounds, `??=`, plus quiet `isset()`, `empty()`, and `??`.
- Public class constants support scalar/array defaults, direct
  `Class::CONST`/`self::CONST` reads, dynamic `$class::CONST` reads, and
  `constant()`/`defined()` lookup; typed/non-public/inherited constants remain
  bounded.
- Stream resources from `STDIN`/`STDOUT`/`STDERR`, filesystem
  `fopen()`/`fclose()`, and `php://memory`/`php://temp` wrapper streams are
  boxed with type, dump, and array-key cast behavior; `php://temp` supports
  max-memory spill thresholds, append/overwrite modes, sparse writes, seeking,
  truncation, fstat size metadata, and PHP/TEMP metadata. `file_get_contents()`
  reads filesystem paths into binary-safe strings with bounded offset/length
  handling; stream internals cover `feof()`, `fflush()`, `fgetc()`, `fgets()`,
  `fread()`, `fpassthru()`, `fseek()`/`ftell()`/`rewind()`, `fstat()`,
  `ftruncate()`, `tmpfile()`, `stream_get_contents()`, `stream_get_line()`,
  `stream_filter_append()`/`stream_filter_prepend()` for built-in string
  filters, and filtered `stream_copy_to_stream()` paths; `file()`/`readfile()`
  support bounded include-path lookup; directory resources cover `readdir()`
  and `rewinddir()`; `fwrite()`/`fputs()` write stream bytes through modeled
  write filters with PHP append-mode logical positions and read/write mode
  diagnostics; and filesystem metadata helpers cover `stat()`/`lstat()`,
  scalar `file*` metadata,
  `filetype()`, `chmod()`/`touch()`, `clearstatcache()`, and
  readable/writable/executable/link path predicates.
- Environment and include-path helpers cover `getenv()` snapshots/lookups,
  `putenv()` set/unset plus embedded-NUL/invalid-assignment diagnostics, and
  bounded `get_include_path()`/`set_include_path()`/`ini_set()`/
  `ini_restore()` state for include-path, assertion configuration, and
  bounded runtime INI values including `pcre.backtrack_limit`, `pcre.jit`,
  `opcache.save_comments`, `user_agent`, and exception string parameter
  length; `ReflectionExtension::getINIEntries()` reports modeled extension INI
  entries.
- Bounded PHPT telemetry uses `PHP_SRC_PHPT`, `/home/claude/php-src-phpt`, or
  `.runtime/php-src-phpt`.
- PHPT runners preclassify broad rows before execution and write selected,
  runnable, classification, excluded, and per-category manifests under
  `.runtime/phpt-progress`. Defaults model PTN's current `Core`, `date`,
  `pcre`, `Reflection`, `standard`, and `uri` extension surface plus accepted runner ini keys
  (`assert.active`, `assert.bail`, `assert.callback`, `assert.exception`,
  `assert.warning`, `date.timezone`, `display_errors`, `error_reporting`,
  `extension_dir`, `include_path`, `opcache.save_comments`,
  `pcre.backtrack_limit`, `pcre.jit`, `precision`, `serialize_precision`,
  `sys_temp_dir`, `user_agent`, `zend.assertions`, and `zend.exception_string_param_max_len`); child-process
  control rows are classified until PTN has a native process boundary. Harness
  cleanup, environment setup, unsupported
  SAPI/stdio/source sections, run-tests self-tests, noisy external/flaky
  expectation rows, broad unsupported language surfaces such as anonymous
  classes, interfaces, trait adaptations/conflicts/reflection edges, remaining
  call-unpacking edges
  (by-reference spread, non-Generator Traversable/SPL spread inputs, and
  resource-limit stress rows),
  true generator/Fiber execution boundaries including lazy suspension,
  `yield from`, send/throw/next/getReturn timing, Fiber runtime, generator body
  cleanup/premature close, and remaining reference-timing edges,
  nullable type hints, interpolating heredoc bodies,
  variable variables,
  PHP attribute syntax/reflection metadata,
  class/reflection-metadata blockers,
  readonly static property diagnostics,
  and currently unmodeled mutating array-internal helpers such as
  `array_multisort()`, `usort()`, `uasort()`, and `uksort()` plus
  destructor-reentrant `array_splice()` cases, remaining runtime diagnostics
  and backtrace string APIs, user error/exception handler state,
  `ErrorException` metadata, assertion option callbacks/bail modes, and
  assertion AST pretty-printing are mapped to blocker categories with source
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
- Full-corpus PHPT telemetry uses `tools/run-phpt-baseline.sh --scope full` to
  inventory every local php-src `.phpt` row and write deterministic
  1k/5k/10k/20k/all manifests bucketed across the full extension/SAPI/core
  corpus. `tools/check-phpt-campaign-reports.sh` gates PHPT campaign status
  reports so they are table-only and contain only ported-test and passed-test
  counts.
- Focused string/scalar alias telemetry covers quiet string-offset
  `isset()`/`empty()` diagnostics, numeric string offsets, string-offset COW,
  concat-assignment aliasing, and classified blocker rows for unsupported
  interpolating heredoc/ini/typed-property/extension surfaces.
- Focused nested foreach/reference telemetry covers live by-reference iterator
  unset/rekey behavior, child-array rekeying through by-reference function
  parameters, and classified plain variable-variable unset blockers.

## Status

- `STATUS.md`: canonical generated PTN status dashboard.
- `STATUS.html`: generated HTML dashboard published through GitHub Pages.
- `tools/status-dashboard-features.tsv`: checked-in evidence source for the
  feature and hourly dashboard tables.
- GitHub Actions publishes `STATUS.html` as `index.html` on the legacy
  `gh-pages` branch via `tools/publish-status-gh-pages.sh`.
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
- `docs/PHPT_BROAD_1K_ATTRIBUTE_METADATA_SPLIT_PTN_30JI_2026-06-14.md`:
  current broad 1k attribute metadata bucket refinement, splitting the 149
  excluded rows into 141 attribute-syntax rows and 8 internal-reflection rows.
- `docs/PHPT_BROAD_1K_ATTRIBUTE_SYNTAX_CURRENT_PTN_LX5W_2026-06-14.md`:
  current broad 1k `unsupported-attribute-syntax-metadata` category map with a
  committed 141-row focused manifest.
- `docs/PHPT_BROAD_1K_DIAGNOSTICS_ASSERTION_FRONTIER_PTN_CUEV_2026-06-14.md`:
  current broad 1k diagnostics/assertion runtime blocker map and focused
  manifest.
- `docs/PHPT_BROAD_1K_DIAGNOSTICS_ASSERTION_CURRENT_PTN_KCR3_2026-06-14.md`:
  current post-split broad 1k diagnostics/assertion blocker map with a
  committed 48-row focused manifest.
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
- `docs/PHPT_BROAD_1K_LANGUAGE_DECLARATION_CALL_CURRENT_PTN_C9V6_2026-06-14.md`:
  current broad 1k 147-row language declaration/call/dynamic blocker union
  manifest after the class-declaration and attribute-metadata splits.
- `docs/PHPT_BROAD_1K_CALL_UNPACKING_CURRENT_PTN_DKCS_2026-06-14.md`:
  current broad 1k call-site/array unpacking blocker map, reusing the 34-row
  focused manifest against the latest classifier output.
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
- `docs/PHPT_BROAD_1K_UNPACKING_SPLIT_PTN_EI36_2026-06-14.md`:
  current broad 1k unpacking classifier split, refining the older 34-row
  aggregate into 20 call-site unpacking rows and 14 array/destructuring
  unpacking rows with focused manifests.
- `docs/PHPT_BROAD_1K_CALL_DYNAMIC_TYPE_CURRENT_PTN_NF1K_2026-06-14.md`:
  current broad 1k call-unpacking, function-state, dynamic-symbol, type-hint,
  generator, and internal-call binding blocker map with a 69-row focused
  manifest.
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
- `docs/PHPT_BROAD_1K_TRAIT_DECLARATION_CATEGORY_PTN_YXV2_2026-06-14.md`:
  current broad 1k `unsupported-trait-declaration` category map with a
  committed 25-row focused manifest.
- `docs/PHPT_BROAD_1K_INTERFACE_CATEGORY_PTN_L0H9_2026-06-14.md`:
  current broad 1k interface declaration/implementation blocker map with a
  committed 38-row focused manifest.
- `docs/PHPT_BROAD_1K_PLAIN_HEREDOC_CLASSIFIER_2026-06-14.md`: broad 1k
  plain heredoc/nowdoc classifier refinement and focused PHPT evidence.
- `docs/PHPT_BROAD_1K_MAGIC_METADATA_BLOCKERS_2026-06-14.md`: current broad
  1k magic method metadata blocker map.
- `docs/PHPT_BROAD_1K_OBJECT_STRING_CONVERSION_CLASSIFIER_PTN_I0P3_2026-06-14.md`:
  current broad 1k classifier split moving 61 `__toString()` object
  conversion rows into `unsupported-object-string-conversion-metadata`.
- `docs/PHPT_BROAD_1K_MAGIC_OBJECT_CONVERSION_FRONTIER_2026-06-14.md`: broad
  1k magic-method/object-conversion blocker map and raw focused evidence.
- `docs/PHPT_BROAD_1K_MAGIC_METHOD_METADATA_CURRENT_PTN_7FYM_2026-06-14.md`:
  current broad 1k `unsupported-magic-method-metadata` category map with a
  committed 69-row focused manifest.
- `docs/PHPT_BROAD_1K_MAGIC_METHOD_METADATA_CURRENT_PTN_XDL8_2026-06-14.md`:
  current `unsupported-magic-method-metadata` category replay against the
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
- `docs/PHPT_BROAD_1K_CLASS_METADATA_SPLIT_PTN_XFIE_2026-06-14.md`:
  current broad 1k class/object metadata split refresh, with the former
  `unsupported-class-metadata` bucket at 0 rows and a 135-row focused manifest.
- `docs/PHPT_BROAD_1K_CLASS_OBJECT_ATTRIBUTE_CURRENT_PTN_FT4R_2026-06-14.md`:
  current broad 1k combined class/object/attribute metadata blocker map with a
  committed 284-row focused manifest.
- `docs/PHPT_BROAD_1K_CLASS_OBJECT_ATTRIBUTE_POSTSPLIT_PTN_T6R7_2026-06-14.md`:
  current post-attribute-split combined class/object/attribute metadata blocker
  map with a committed 284-row focused manifest.
- `docs/PHPT_BROAD_1K_REQUEST_SAPI_FRONTIER_2026-06-14.md`: broad 1k
  request/SAPI input boundary blocker map and raw focused evidence.
- `docs/PHPT_BROAD_1K_REQUEST_SAPI_PTN_BAC4_2026-06-14.md`: current
  `ptn-bac4` request/SAPI refresh with 41 focused rows, 28 request-input INI
  blockers, 13 SAPI rows, and raw pass-through evidence.
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
- `docs/PHPT_FILESYSTEM_PATH_PROCESS_FRONTIER_PTN_WC6B_2026-06-14.md`:
  current `ptn-wc6b` filesystem/path/process blocker refresh with 13 runnable
  rows green and the 25-row native child-process boundary isolated.
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
- `docs/PHPT_BROAD_1K_CLUSTER_SLICE_PTN_AAOJ_2026-06-14.md`: refreshed broad
  1k cluster blocker map on current `master`, with 424 runnable rows, 576
  excluded rows, and zero runnable rows outside committed focused manifests.
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
- `docs/PHPT_BROAD_1K_CLUSTER_SLICE_PTN_P2I7_2026-06-14.md`: refreshed broad
  1k blocker map with exact classifier buckets, zero unmatched broad-runnable
  rows, and current 25+ row excluded-frontier boundaries.
- `docs/PHPT_BROAD_1K_CLASSIFY_HARNESS_STDIN_PTN_H0QA_2026-06-14.md`: broad
  1k classify-only harness fix evidence and current 1000-row blocker map with
  424 runnable and 576 classified rows.
- `docs/PHPT_BROAD_1K_CLUSTER_SLICE_PTN_CGFH_2026-06-14.md`: current broad
  1k cluster blocker map on `24318afd2014`, with 424 runnable rows fully
  covered by committed focused manifests and no credible single 25-row
  implementation cluster.
- `docs/PHPT_BROAD_1K_CLUSTER_SLICE_PTN_73Z5_2026-06-14.md`: refreshed broad
  1k cluster blocker map on current `master`, with 424 runnable rows, 576
  classified blockers, and zero runnable rows outside committed focused
  manifests.
- `docs/PHPT_BROAD_1K_CLUSTER_SLICE_PTN_666N_2026-06-14.md`: refreshed broad
  1k cluster blocker map with 424 runnable rows fully covered by committed
  focused manifests and 576 classified rows split by semantic owner.
- After changing status evidence, run `tools/update-status-dashboard.sh` and
  commit the regenerated `STATUS.md` and `STATUS.html` files. Implementation
  lanes should record evidence in their bead or merge request instead of
  editing these generated dashboards directly.

## Commands

```bash
cargo test
tools/run-native-smoke-matrix.sh
tools/run-post-merge-cow-gate.sh
cargo build --bin phpc
tools/update-status-dashboard.sh
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
tools/run-phpt-baseline.sh --scope full --generate-only
tools/run-phpt-baseline.sh --scope full --tier 1000 --classify-only
tools/check-phpt-campaign-reports.sh docs/REPORT.md
```
