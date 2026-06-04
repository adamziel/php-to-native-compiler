# PHP Native Compiler PHPT Progress

Updated: 2026-06-04 11:03 CEST

Primary/public branch: `origin/master`
Latest accepted public-score source:
`0086ba77c61b1379951e3b23f8830e94f6cc0c79 fix: parse textual strtotime timezone suffixes`

Semantic source for current published score:
`0086ba77c61b1379951e3b23f8830e94f6cc0c79 fix: parse textual strtotime timezone suffixes`

Public PHPT metric:

`passed runnable PHPTs / total pinned runnable PHPTs`

Pinned denominator: `20294` total pinned runnable php-src PHPTs. Raw runner
denominators that exclude BORKED rows are not public progress.

Current public score: **7376 / 20294 pinned runnable PHPTs = 36.35%**.

Latest pushed source checkpoint:
`e340aa0b0b300711bd0f3c003665397f1e8ded44 fix: reject dynamic non-static callables`

Current score-gate status:
The latest full gate for source `a3889f45`:
`state/logs/phpt-full-current-score-20260604T083545Z-php-src-f97ff59-public-a3889f45-source-a3889f45`
completed `FINAL / BLOCKED-PASS-REGRESSIONS` against the accepted `0086ba77`
PASS baseline. It reported public-comparable `7602 / 20294 = 37.46%` and
`7598` normalized current passes, but it is not publication evidence because it
has `2` latest-published PASS regressions:
`php-src/Zend/tests/abstract_implicit.phpt` and
`php-src/Zend/tests/dynamic_call/dynamic_call_non_static.phpt`. That gate does
not include newer source `1b1e9b0d` or `e340aa0b`. The dynamic-call row now has
selected repair proof at current source `e340aa0b`; `abstract_implicit.phpt`
remains an outstanding publication blocker. The earlier replacement full gate
for source `6b0952e0` completed `FINAL / BLOCKED-PASS-REGRESSIONS` with one
latest-published PASS regression,
`php-src/Zend/tests/typehints/namespace_relative_scalar.phpt`; that row has
selected repair proof at source `a3889f45`, but the blocked `6b0952e0` gate
cannot be published. The older `6b0952e0` gate ended
`FINAL / GATE-INCOMPLETE-NO-EXIT-MARKERS / NOT-PUBLICATION-EVIDENCE`, the
previous `f13e45f4` gate also ended incomplete with no exit markers, and the
previous `daee77c1` gate completed with a broken aggregate caused by
shard-directory naming mismatch. The latest accepted full gate remains
`state/logs/phpt-full-current-score-20260604T043544Z-php-src-f97ff59-public-0086ba77-source-0086ba77`.
It completed `FINAL / GATE-AGGREGATE-COMPLETE / ZERO-PASS-REGRESSIONS`
against the accepted `21abc76f` PASS baseline.

The accepted source checkpoint chain through semantic source `0086ba77` has
been pushed to `origin/master` and is now reflected in the public score above.
The completed score gate at checkpoint `0086ba77` converted the post-`21abc76f`
source chain into a public PHPT movement of `7279 -> 7376` passed pinned rows,
a net gain of `97` public PHPT passes, with zero latest-published PASS
regressions. The previous score gate at checkpoint `21abc76f` converted the
post-`96ed077d` source chain into a public PHPT movement of `7240 -> 7279`
passed pinned rows, a net gain of `39` public PHPT passes, with zero
latest-published PASS regressions.

Unpublished source progress since the accepted public score source:

- Latest pushed source head is `e340aa0b0b300711bd0f3c003665397f1e8ded44`
  (`fix: reject dynamic non-static callables`), which is newer than the
  accepted `0086ba77` full gate and is not included in the public score above.
- There are `19` score-relevant source commits after the accepted public-score
  source: `01afaf19e44c2d4a23023469a0a1122ed81f7d90`
  (`fix: parse numeric literal separators`),
  `f4be414cce658cfc8c6f77ad5a17146a97c66e22`
  (`fix: materialize first-class callable constants`),
  `1fe5a48ad8e6e3bb4e224fb182ff1b06730c22fb`
  (`fix: preserve bounded timezone offsets`),
  `84fbeb8d8dc0637a8b722d38512bb5b856ae7cf7`
  (`fix: construct user classes via reflection`),
  `40609421fef30899c8fcd8fcd575a84526de3462`
  (`fix: handle bounded datetime diff offsets`),
  `d2709c1588fb10700542b07bbb161021e88293b4`
  (`fix: handle invalid list destructuring forms`),
  `daee77c182d84f4b3bd6255c5efb788822f4c778`
  (`fix: parse meridiem strtotime forms`),
  `0ee623687bebc402bab0488301b9b3e84c39774d`
  (`fix: honor disabled function metadata`),
  `003b77e6098a95c5741b03a5ed9a0fe8e75b87cb`
  (`fix: parse bounded createFromFormat dates`),
  `f13e45f48d96df1c7648e224370ce430e64ab0ed`
  (`fix: canonicalize dnf signature types`),
  `6eb8d8189a5d92db31dfe4bf0d2b122edb1f0d4b`
  (`fix: track PHP error diagnostics`),
  `22a45603987ba28705f678941cf6edfe12521e9e`
  (`fix: resolve namespace constants`),
  `2b3f00fc3cf77aa081509a3b64b4dd03357d1994`
  (`fix: handle ArrayObject object storage`),
  `6b0952e0507a6f810e162b4aeb4a40589753c05c`
  (`fix: preserve reset date parse defaults`),
  `6e1c1ebf30c0f138e53a7c7f4f846cf5a1b684d9`
  (`fix: dispatch magic static method calls`),
  `2c401b6d36d0641bed43cc3b745157b67e0b095d`
  (`fix: validate abstract trait requirements`),
  `a3889f457ab8a0d2fc2856db443b14779d3bf72e`
  (`fix: reject namespace-relative builtin types`),
  `1b1e9b0d557a8c124e7a05692bacbfd4ebba7fae`
  (`fix: honor get class vars scope`), and
  `e340aa0b0b300711bd0f3c003665397f1e8ded44`
  (`fix: reject dynamic non-static callables`).
- Staged focused selected-PHPT proof after the latest accepted public-score
  source totals `191` selected rows across `19` integrated source artifacts.
  These are source/staging facts, not a public score update unless a full
  pinned PHPT gate accepts them.
- The numeric literal separator packet moved selected PHPT `0/11 -> 11/11`
  for `Zend/tests/numeric_literal_separator/bug78454_1.phpt`,
  `Zend/tests/numeric_literal_separator/bug78454_2.phpt`,
  `Zend/tests/numeric_literal_separator/numeric_literal_separator_001.phpt`,
  `Zend/tests/numeric_literal_separator/numeric_literal_separator_002.phpt`,
  `Zend/tests/numeric_literal_separator/numeric_literal_separator_003.phpt`,
  `Zend/tests/numeric_literal_separator/numeric_literal_separator_004.phpt`,
  `Zend/tests/numeric_literal_separator/numeric_literal_separator_005.phpt`,
  `Zend/tests/numeric_literal_separator/numeric_literal_separator_006.phpt`,
  `Zend/tests/numeric_literal_separator/numeric_literal_separator_007.phpt`,
  `Zend/tests/numeric_literal_separator/numeric_literal_separator_008.phpt`,
  and `Zend/tests/numeric_literal_separator/numeric_literal_separator_009.phpt`.
  It also preserved the latest-published PASS scout `8/8`, passed focused
  Rust coverage, build, fmt, diff checks, and production row-name leakage scan.
- The first-class callable constexpr packet moved selected PHPT
  `0/13 -> 13/13` for
  `Zend/tests/first_class_callable/constexpr/basic.phpt`,
  `Zend/tests/first_class_callable/constexpr/case_insensitive.phpt`,
  `Zend/tests/first_class_callable/constexpr/class_const.phpt`,
  `Zend/tests/first_class_callable/constexpr/default_args.phpt`,
  `Zend/tests/first_class_callable/constexpr/error_unknown_function.phpt`,
  `Zend/tests/first_class_callable/constexpr/error_unknown_method.phpt`,
  `Zend/tests/first_class_callable/constexpr/error_unknown_class.phpt`,
  `Zend/tests/first_class_callable/constexpr/namespace_001.phpt`,
  `Zend/tests/first_class_callable/constexpr/namespace_002.phpt`,
  `Zend/tests/first_class_callable/constexpr/namespace_003.phpt`,
  `Zend/tests/first_class_callable/constexpr/static_call.phpt`,
  `Zend/tests/first_class_callable/constexpr/static_call_self.phpt`, and
  `Zend/tests/first_class_callable/constexpr/userland.phpt`. It also
  preserved the latest-published PASS scout `8/8`, passed focused Rust
  coverage, build, fmt, diff checks, and production row-name leakage scan.
- The DateTime timezone-offset packet moved selected PHPT `0/12 -> 12/12`
  for `ext/date/tests/bug26317.phpt`, `bug35218.phpt`, `bug37017.phpt`,
  `bug45081.phpt`, `bug46111.phpt`, `bug66985.phpt`, `bug75857.phpt`,
  `bug81097.phpt`, `bug81565.phpt`, `gh10218.phpt`, `gh11281.phpt`, and
  `gh20764.phpt`. It also preserved the latest-published PASS scout `8/8`,
  passed focused Rust coverage, build, fmt, diff checks, and production
  row-name leakage scan.
- The ReflectionClass construction packet moved selected PHPT `0/11 -> 11/11`
  for `ext/reflection/tests/001.phpt`, `007.phpt`,
  `ReflectionClass_newInstance_001.phpt`,
  `ReflectionClass_newInstanceArgs_001.phpt`,
  `ReflectionClass_newInstanceArgs_002.phpt`, `bug38217.phpt`,
  `bug42976.phpt`, `bug43926.phpt`, `bug52854.phpt`, `bug70982.phpt`, and
  `bug77882.phpt`. It also preserved the latest-published PASS scout `8/8`,
  passed focused Rust coverage, build, fmt, diff checks, and production
  row-name leakage scan.
- The DateTime diff/massive arithmetic packet moved selected PHPT
  `0/12 -> 12/12` for
  `ext/date/tests/DateTime_diff-fall-type2-type2.phpt`,
  `DateTime_diff-fall-type2-type3.phpt`,
  `DateTime_diff-fall-type3-type2.phpt`,
  `DateTime_diff-fall-type3-type3.phpt`,
  `DateTime_diff-spring-type2-type2.phpt`,
  `DateTime_diff-spring-type2-type3.phpt`,
  `DateTime_diff-spring-type3-type2.phpt`,
  `DateTime_diff-spring-type3-type3.phpt`, `DateTime_add-massive.phpt`,
  `DateTime_days-massive.phpt`, `DateTime_diff-massive.phpt`, and
  `DateTime_sub-massive.phpt`. It also preserved the active blocker row state,
  preserved the latest-published non-date PASS scout `8/8`, passed focused Rust
  coverage, build, fmt, diff checks, and production row-name leakage scan.
- The invalid array/list destructuring packet moved selected PHPT
  `0/11 -> 11/11` for `Zend/tests/list/list_007.phpt`,
  `list_008.phpt`, `list_010.phpt`, `list_011.phpt`, `list_012.phpt`,
  `list_013.phpt`, `list_014.phpt`, `list_empty_error.phpt`,
  `list_empty_error_keyed.phpt`, `list_keyed_leading_comma.phpt`, and
  `list_mixed_keyed_unkeyed.phpt`. It also preserved the latest-published
  non-date PASS scout `8/8`, passed focused Rust coverage, build, fmt, diff
  checks, isolated `bug60598`, rechecked the active blocker row, and passed the
  production row-name leakage scan.
- The `bug34771` PASS-regression repair packet moved selected PHPT
  `0/1 -> 1/1` for `ext/date/tests/bug34771.phpt`. It also preserved the
  blocker plus latest-published PASS scout `9/9`, passed focused Rust
  coverage, build, fmt, diff checks, and production row-name leakage scan.
  This restores the row that blocked the `1fe5a48a` full gate, but it is not
  public score movement unless a fresh full pinned PHPT gate accepts it.
- The disabled-function metadata packet moved selected PHPT `0/13 -> 13/13`
  for `tests/basic/bug31875.phpt`,
  `Zend/tests/assert/disable_assert_function.phpt`,
  `Zend/tests/bug69315.phpt`, `Zend/tests/bug79382.phpt`,
  `Zend/tests/exit/disabling_die.phpt`,
  `Zend/tests/exit/disabling_exit.phpt`,
  `ext/opcache/tests/bug76796.phpt`,
  `ext/reflection/tests/ReflectionFunction_isDisabled_basic.phpt`,
  `Zend/tests/bug48899.phpt`, `Zend/tests/bug48899-deprecated.phpt`,
  `Zend/tests/bug63111.phpt`,
  `Zend/tests/is_callable_trampoline_uaf-deprecated.phpt`, and
  `ext/standard/tests/general_functions/is_callable_abstract_method-deprecated.phpt`.
  It also preserved the latest-published non-date PASS scout `8/8`, passed
  focused Rust coverage, build, fmt, diff checks, and production row-name
  leakage scan.
- The DateTime create/parse-format packet moved selected PHPT `0/11 -> 11/11`
  for `ext/date/tests/bug50392.phpt`, `bug51393.phpt`, `bug51994.phpt`,
  `bug53879.phpt`, `bug54316.phpt`, `bug66836.phpt`, `bug68078.phpt`,
  `bug68078_negative.phpt`, `bug72963.phpt`, `bug76770.phpt`, and
  `date-lenient.phpt`. It also preserved the latest-published PASS scout
  `8/8`, passed focused Rust coverage, build, fmt, diff checks, and production
  row-name leakage scan.
- The DNF/intersection signature-types packet moved selected PHPT
  `0/17 -> 17/17` for
  `Zend/tests/type_declarations/dnf_types/dnf_intersection_and_null.phpt`,
  `Zend/tests/type_declarations/dnf_types/dnf_intersection_and_single.phpt`,
  `Zend/tests/type_declarations/dnf_types/variance/invalid_covariance_intersection_to_union1.phpt`,
  `invalid_covariance_intersection_to_union2.phpt`,
  `invalid_covariance_intersection_to_union3.phpt`, `valid2.phpt`,
  `valid4.phpt`, `valid5.phpt`, `valid6.phpt`, `valid7.phpt`,
  `valid8.phpt`, `valid9.phpt`,
  `Zend/tests/type_declarations/intersection_types/variance/invalid4.phpt`,
  `invalid5.phpt`, `invalid_covariance_intersection_to_union1.phpt`,
  `invalid_covariance_intersection_to_union2.phpt`, and
  `invalid_covariance_intersection_to_union3.phpt`. It also preserved the
  latest-published non-date PASS scout `8/8`, passed focused Rust coverage,
  build, fmt, diff checks, and production row-name leakage scan.
- The diagnostics error-reporting/error-handler packet moved selected PHPT
  `0/10 -> 10/10` for
  `Zend/tests/error_reporting/error_reporting03.phpt`,
  `Zend/tests/error_reporting/error_reporting05.phpt`,
  `Zend/tests/error_reporting/error_reporting08.phpt`,
  `Zend/tests/error_reporting/error_reporting09.phpt`,
  `Zend/tests/error_reporting/error_reporting10.phpt`,
  `Zend/tests/get_error_handler.phpt`,
  `Zend/tests/ignore_repeated_errors.phpt`,
  `Zend/tests/trigger_error_basic.phpt`,
  `ext/standard/tests/general_functions/error_get_last.phpt`, and
  `error_clear_last.phpt`. It also preserved the latest-published non-date PASS
  scout `8/8`, passed focused Rust coverage, build, fmt, diff checks, and
  production row-name leakage scan.
- The namespace constant/type-resolution packet moved selected PHPT
  `0/12 -> 12/12` for `Zend/tests/errmsg/bug43344_1.phpt`,
  `bug43344_6.phpt`, `bug43344_7.phpt`, `bug43344_8.phpt`,
  `bug43344_9.phpt`, `Zend/tests/constants/bug46304.phpt`,
  `Zend/tests/constants/constants_009.phpt`,
  `Zend/tests/namespaces/ns_041.phpt`, `ns_057.phpt`, `ns_077_3.phpt`,
  `ns_077_4.phpt`, and `Zend/tests/use_const/no_global_fallback.phpt`. It
  also preserved the latest-published non-date PASS scout `8/8`, passed
  focused Rust coverage, build, fmt, diff checks, and production row-name
  leakage scan.
- The ArrayObject object-backed storage packet moved selected PHPT
  `0/11 -> 11/11` for
  `ext/spl/tests/ArrayObject/ArrayObject_clone_other_std_props.phpt`,
  `ArrayObject_std_props_no_recursion.phpt`,
  `arrayObject___construct_basic1.phpt`,
  `arrayObject___construct_basic2.phpt`,
  `arrayObject___construct_basic3.phpt`,
  `arrayObject___construct_basic4.phpt`,
  `arrayObject___construct_basic5.phpt`,
  `arrayObject___construct_basic6.phpt`,
  `arrayObject_clone_basic2.phpt`, `arrayObject_clone_basic3.phpt`, and
  `arrayObject_exchangeArray_basic2.phpt`. It also preserved the
  latest-published non-date PASS scout `8/8`, passed focused Rust
  ArrayObject coverage, build, fmt, diff checks, and production row-name
  leakage scan.
- The `date-parse-by-format001` PASS-regression repair packet moved selected
  PHPT `0/1 -> 1/1` for
  `ext/date/tests/date-parse-by-format001.phpt`. It confirmed that row was the
  sole latest-published PASS regression in the blocked `6eb8d818` gate and was
  present in the latest accepted PASS baseline. It also preserved the
  latest-published non-date PASS scout `8/8`, passed focused Rust DateTime
  coverage, build, fmt, diff checks, and production row-name leakage scan.
  This repairs the exact known `6eb8d818` publication blocker at current source
  `6b0952e0`, but it is not public score movement unless the fresh full pinned
  PHPT gate accepts it.
- The magic-method dispatch packet moved selected PHPT `0/12 -> 12/12` for
  `Zend/tests/magic_methods/bug19859.phpt`, `bug42937.phpt`,
  `bug45186.phpt`, `bug45186_2.phpt`, `bug46238.phpt`, `bug47801.phpt`,
  `bug48533.phpt`, `bug53826.phpt`, `bug77339.phpt`,
  `call_static_003.phpt`, `call_static_006.phpt`, and
  `call_static_007.phpt`. It also preserved the latest-published PASS scout
  `8/8`, passed focused Rust object-model coverage, build, fmt, diff checks,
  and production row-name leakage scan.
- The abstract trait requirements packet moved selected PHPT `0/19 -> 19/19`
  for the selected `Zend/tests/traits/abstract_method_*`, trait bug, and
  `Zend/tests/traits/gh14009_*` rows covering abstract trait method
  requirements. It also preserved the latest-published PASS scout `8/8`,
  passed focused Rust object/syntax coverage, build, fmt, diff checks, and
  production row-name leakage scan.
- The `namespace_relative_scalar.phpt` PASS-regression repair packet moved
  selected PHPT `0/1 -> 1/1` for
  `Zend/tests/typehints/namespace_relative_scalar.phpt`. It confirmed that row
  was the sole latest-published PASS regression in the blocked `6b0952e0`
  replacement gate, preserved adjacent type/date scouts and latest-published
  PASS scout `8/8`, and passed focused Rust, build, fmt, diff checks, and
  production row-name leakage scan.
- The get-class-vars / late-static-binding packet moved selected PHPT
  `0/12 -> 12/12` for the selected `Zend/tests/get_class_vars/*`,
  `ext/standard/tests/class_object/get_class_vars_variation2.phpt`,
  `Zend/tests/lsb/*`, and `Zend/tests/traits/static_003.phpt` rows. It also
  preserved the latest-published PASS scout `8/8` after an isolated longer
  timeout rerun for `Zend/tests/bug60598.phpt`, passed focused Rust
  object-model coverage, build, fmt, diff checks, and production row-name
  leakage scan.
- The `dynamic_call_non_static.phpt` PASS-regression repair packet moved
  selected PHPT `0/1 -> 1/1` for
  `Zend/tests/dynamic_call/dynamic_call_non_static.phpt`. It repairs one of
  the two latest-published PASS regressions from the blocked `a3889f45` full
  gate at current source `e340aa0b`; `Zend/tests/abstract_implicit.phpt`
  remains the known publication blocker. It also preserved adjacent
  dynamic-call rows and latest-published PASS scout `8/8`, and passed focused
  Rust, build, fmt, diff checks, and production row-name leakage scan.
- The `a3889f45` full gate is blocked:
  `state/logs/phpt-full-current-score-20260604T083545Z-php-src-f97ff59-public-a3889f45-source-a3889f45`
  against the accepted `0086ba77` PASS baseline with `7372` normalized rows.
  It completed `FINAL / BLOCKED-PASS-REGRESSIONS` with public-comparable
  `7602 / 20294 = 37.46%`, `7598` normalized current passes, and `2`
  latest-published PASS regressions:
  `php-src/Zend/tests/abstract_implicit.phpt` and
  `php-src/Zend/tests/dynamic_call/dynamic_call_non_static.phpt`. It is not
  publication evidence for `a3889f45` or newer sources; the dynamic-call row now
  has selected repair proof at `e340aa0b`, and `abstract_implicit.phpt`
  remains outstanding.
- The `6b0952e0` replacement full gate is blocked:
  `state/logs/phpt-full-current-score-20260604T075946Z-php-src-f97ff59-public-6b0952e0-source-6b0952e0`
  against the accepted `0086ba77` PASS baseline with `7372` normalized rows.
  It completed `FINAL / BLOCKED-PASS-REGRESSIONS` with public-comparable
  `7567 / 20294 = 37.29%`, `7563` normalized current passes, and `1`
  latest-published PASS regression:
  `php-src/Zend/tests/typehints/namespace_relative_scalar.phpt`. It is not
  publication evidence for `6b0952e0` or newer sources; that exact row now has
  selected repair proof at source `a3889f45`.
- The earlier `6b0952e0` full gate is incomplete and not publication evidence:
  `state/logs/phpt-full-current-score-20260604T074408Z-php-src-f97ff59-public-6b0952e0-source-6b0952e0`
  ended `FINAL / GATE-INCOMPLETE-NO-EXIT-MARKERS /
  NOT-PUBLICATION-EVIDENCE` with empty shard exit markers and no aggregate,
  current-passes, or PASS-regression files.
- The `6eb8d818` full gate is blocked:
  `state/logs/phpt-full-current-score-20260604T070724Z-php-src-f97ff59-public-6eb8d818-source-6eb8d818`
  against the accepted `0086ba77` PASS baseline with `7372` normalized rows.
  It completed `FINAL / BLOCKED-PASS-REGRESSIONS` with public-comparable
  `7542 / 20294 = 37.16%`, `7538` normalized current passes, and `1`
  latest-published PASS regression:
  `php-src/ext/date/tests/date-parse-by-format001.phpt`. It is not
  publication evidence for `6eb8d818` or newer source `6b0952e0`; that exact
  blocker row now has selected repair proof at current source `6b0952e0`.
- The `f13e45f4` full gate is incomplete and not publication evidence:
  `state/logs/phpt-full-current-score-20260604T065227Z-php-src-f97ff59-public-f13e45f4-source-f13e45f4`
  ended `FINAL / GATE-INCOMPLETE-NO-EXIT-MARKERS /
  NOT-PUBLICATION-EVIDENCE` with empty shard exit markers and no aggregate,
  current-passes, or PASS-regression files.
- The `daee77c1` full gate is broken and not publication evidence:
  `state/logs/phpt-full-current-score-20260604T055910Z-php-src-f97ff59-public-daee77c1-source-daee77c1`
  completed with a shard-directory naming mismatch that produced a broken
  aggregate and apparent `7336` PASS regressions; it must not be used for
  publication.
- The `1fe5a48a` full gate is blocked:
  `state/logs/phpt-full-current-score-20260604T051404Z-php-src-f97ff59-public-1fe5a48a-source-1fe5a48a`
  against the accepted `0086ba77` PASS baseline with `7372` normalized rows.
  It completed `FINAL / BLOCKED-PASS-REGRESSIONS` with public-comparable
  `7428 / 20294 = 36.60%`, `7424` normalized current passes, and `1`
  latest-published PASS regression:
  `php-src/ext/date/tests/bug34771.phpt`. It is not publication evidence for
  `1fe5a48a` or any newer source. The row has selected repair proof at source
  `daee77c1`, and later gate state is tracked by the newer blocked/incomplete
  gate bullets above.
- The `c12ecf1b` full gate is blocked:
  `state/logs/phpt-full-current-score-20260604T034027Z-php-src-f97ff59-public-c12ecf1b-source-c12ecf1b`
  against the accepted `21abc76f` PASS baseline with `7275` normalized rows.
  It completed `FINAL / BLOCKED-PASS-REGRESSIONS` with public-comparable
  `7317 / 20294 = 36.05%`, `7313` normalized current passes, and `1`
  latest-published PASS regression:
  `php-src/ext/date/tests/bug35422.phpt`. It is not publication evidence. That
  row has selected repair proof at source `0086ba77`, and the later accepted
  `0086ba77` full gate below supersedes this blocked gate for publication.
  The newer `c912841a` final-constants, `93aafea2` class-name constants, and
  `0086ba77` repair sources are not included in the blocked `c12ecf1b` gate.
- The attempted `830d966d` full gate is not publication evidence:
  `state/logs/phpt-full-current-score-20260604T032315Z-php-src-f97ff59-public-830d966d-source-830d966d`
  ended `FINAL / GATE-INCOMPLETE-NO-AGGREGATE /
  NOT-PUBLICATION-EVIDENCE`. It produced no aggregate, current-passes, or
  regression summary files, and shards `01`, `08`, and `09` lacked `exit.tsv`
  markers. That gate included semantic source `e2c610c7`, not `c12ecf1b` or
  the newer `c912841a` / `93aafea2` / `0086ba77` sources, so it cannot move
  the public score even aside from the incomplete run state.
- The previous publication blocker was the blocked `52a79aa1` full gate:
  `state/logs/phpt-full-current-score-20260604T005236Z-php-src-f97ff59-public-52a79aa1-source-52a79aa1`
  had `1` latest-published PASS regression,
  `php-src/ext/spl/tests/ArrayObject/arrayObject_getIteratorClass_basic1.phpt`.
  Its normalized PASS comparison was `7115` baseline rows to `7134` current
  rows with `1` regression, so it cannot move the public score. The row now has
  selected repair proof at source `d1022750`.
- The later blocked `d1022750` full gate is also not publication evidence. It
  had `1` latest-published PASS regression,
  `php-src/ext/standard/tests/math/round_gh12143_expand_rounding_target.phpt`.
  That row now has selected repair proof at source `00a8fa10` and the accepted
  `00a8fa10` gate listed below has zero latest-published PASS regressions.
- The prior score-gate checkpoint
  `28662f8a482521f65fcbdc1b415163afbf65efd9` is not publication evidence
  because its gate is missing result files. The earlier `b2914aff` full gate
  remains blocked by the latest-published PASS regression
  `php-src/Zend/tests/bug60598.phpt`.

Accepted public gate: checkpoint `0086ba77` completed the pinned full PHPT
gate with `7376 / 20294 = 36.35%` and zero latest-published PASS regressions
against the `21abc76f` baseline. Full gate evidence is in
`state/logs/phpt-full-current-score-20260604T043544Z-php-src-f97ff59-public-0086ba77-source-0086ba77`;
the aggregate had `7376` passed rows, `7372` normalized current passes, and
`0` PASS regressions. The accepted PASS baseline is
`state/logs/phpt-full-current-score-20260604T043544Z-php-src-f97ff59-public-0086ba77-source-0086ba77/current-passes.normalized.txt`
with `7372` rows and SHA-256
`533d26badda5ae40d414a51611a76417eb2621d67d9ba5a83a38b47e7684316f`.
The invalid-proof-marker summary reported `0` hits.

Previous accepted public gate: checkpoint `21abc76f` completed the pinned full PHPT
gate with `7279 / 20294 = 35.87%` and zero latest-published PASS regressions
against the `96ed077d` baseline. Full gate evidence is in
`state/logs/phpt-full-current-score-20260604T030645Z-php-src-f97ff59-public-21abc76f-source-21abc76f`;
the aggregate had `7279` passed rows, `7275` normalized current passes, and
`0` PASS regressions. The accepted PASS baseline is
`state/logs/phpt-full-current-score-20260604T030645Z-php-src-f97ff59-public-21abc76f-source-21abc76f/current-passes.normalized.txt`
with SHA-256
`205f1109ae6a26361cb4ce1df92b6d44b4ec3358aafa16f2238d69294720916a`.
The invalid-proof-marker summary reported `0` hits.

Previous accepted public gate: checkpoint `96ed077d` completed the pinned full PHPT
gate with `7240 / 20294 = 35.68%` and zero latest-published PASS regressions
against the `00a8fa10` baseline. Full gate evidence is in
`state/logs/phpt-full-current-score-20260604T024037Z-php-src-f97ff59-public-96ed077d-source-96ed077d`;
the aggregate had `7240` passed rows, `7236` normalized current passes, and
`0` PASS regressions. The accepted PASS baseline is
`state/logs/phpt-full-current-score-20260604T024037Z-php-src-f97ff59-public-96ed077d-source-96ed077d/current-passes.normalized.txt`
with SHA-256
`b91a188270d1704b610e00b65f8f495f5ee2f91ccd0d91ce13f440b742f0d40b`.
The invalid-proof-marker summary reported `0` hits.

Previous accepted public gate: checkpoint `00a8fa10` completed the pinned full PHPT
gate with `7201 / 20294 = 35.48%` and zero latest-published PASS regressions
against the `5753eadf` baseline. Full gate evidence is in
`state/logs/phpt-full-current-score-20260604T020445Z-php-src-f97ff59-public-00a8fa10-source-00a8fa10`;
the aggregate had `7201` passed rows, `7197` normalized current passes, and
`0` PASS regressions. The accepted PASS baseline is
`state/logs/phpt-full-current-score-20260604T020445Z-php-src-f97ff59-public-00a8fa10-source-00a8fa10/current-passes.normalized.txt`
with SHA-256
`d2137a884f32d573a5163541c6326b704ea2eec0e8413f29c8d0bc3d813b6743`.
The invalid-proof-marker summary reported `0` hits.

Previous accepted public gate: checkpoint `5753eadf` completed the pinned full
PHPT gate with `7119 / 20294 = 35.08%` and zero latest-published PASS
regressions against the `4cbed170` baseline. Full gate evidence is in
`state/logs/phpt-full-current-score-20260604T003615Z-php-src-f97ff59-public-5753eadf-source-5753eadf`;
the aggregate had `7119` passed rows, `7115` normalized current passes, and
`0` PASS regressions. The accepted PASS baseline is
`state/logs/phpt-full-current-score-20260604T003615Z-php-src-f97ff59-public-5753eadf-source-5753eadf/current-passes.normalized.txt`
with SHA-256
`aa2e0f901686caf7436d8497e8b4a18b418c534b312b6723518ae9d555bd2a18`.

Previous accepted public gate: checkpoint `4cbed170` completed the pinned full
PHPT gate with `7090 / 20294 = 34.94%` and zero latest-published PASS
regressions against the `fd09b997` baseline. Full gate evidence is in
`state/logs/phpt-full-current-score-20260604T001248Z-php-src-f97ff59-public-4cbed170-source-4cbed170`;
the aggregate had `7090` passed rows, `7086` normalized current passes, and
`0` PASS regressions. The accepted PASS baseline is
`state/logs/phpt-full-current-score-20260604T001248Z-php-src-f97ff59-public-4cbed170-source-4cbed170/current-passes.normalized.txt`
with SHA-256
`33578febdffb60f9cdaa5dd6e8f66c29659be6ef38e1b82d68415008e02eb51c`.

Previous accepted public gate: checkpoint `fd09b997` completed the pinned full PHPT
gate with `7015 / 20294 = 34.57%` and zero latest-published PASS regressions
against the `64d95b6d` baseline. Full gate evidence is in
`state/logs/phpt-full-current-score-20260603T224452Z-php-src-f97ff59-public-fd09b997-source-fd09b997`;
the aggregate had `7015` passed rows, `7011` normalized current passes, and
`0` PASS regressions. The invalid-proof-marker summary is not a publication
blocker in this accepted gate.

Previous accepted public gate: checkpoint `64d95b6d` completed the pinned full PHPT
gate with `6628 / 20294 = 32.66%` and zero latest-published PASS regressions
against the `f9927a95` baseline. Full gate evidence is in
`state/logs/phpt-full-current-score-20260603T121455Z-php-src-f97ff59-public-64d95b6d-source-64d95b6d`;
the aggregate had `6628` passed rows, `6624` normalized current passes, and
`0` PASS regressions. The invalid-proof-marker summary reported `0` hits.

Previous accepted public gate: checkpoint `f9927a95` completed the pinned full PHPT gate
with `6498 / 20294 = 32.02%` and zero latest-published PASS regressions against
the `d22bef26` baseline. Full gate evidence is in
`state/logs/phpt-full-current-score-20260603T101148Z-php-src-f97ff59-public-f9927a95-source-f9927a95`;
the aggregate had `6498` passed rows, `6494` normalized current passes, and
`0` PASS regressions. The invalid-proof-marker summary reported `0` hits.

Previous accepted public gate: checkpoint `d22bef26` completed the pinned full PHPT gate
with `6396 / 20294 = 31.52%` and zero latest-published PASS regressions against
the `dcd07a3c` baseline. Full gate evidence is in
`state/logs/phpt-full-current-score-20260603T082116Z-php-src-f97ff59-public-d22bef26-source-d22bef26`;
the aggregate had `6396` passed rows, `6392` normalized current passes, and
`0` PASS regressions. The invalid-proof-marker summary reported `0` hits.

Previous accepted public gate: checkpoint `dcd07a3c` completed the pinned full PHPT gate
with `6265 / 20294 = 30.87%` and zero latest-published PASS regressions against
the `423a03d4` baseline. Full gate evidence is in
`state/logs/phpt-full-current-score-20260602T224641Z-php-src-f97ff59-public-dcd07a3c-source-dcd07a3c`;
the aggregate had `6265` passed rows, `6261` normalized current passes, and
`0` PASS regressions. The invalid-proof-marker summary reported `0` hits.

Previous accepted public gate: checkpoint `423a03d4` completed the pinned full
PHPT gate with `6194 / 20294 = 30.52%` and zero latest-published PASS
regressions against the `6ca895a9` baseline. Full gate evidence is in
`state/logs/phpt-full-current-score-20260602T211722Z-php-src-f97ff59-public-423a03d4-source-423a03d4`;
the aggregate had `6194` passed rows, `6190` normalized current passes, and
`0` PASS regressions. The invalid-proof-marker summary reported `0` hits.

Previous accepted public gate: checkpoint `6ca895a9` completed the pinned full
PHPT gate with `6141 / 20294 = 30.26%` and zero latest-published PASS
regressions against the `c307401c` baseline. Full gate evidence is in
`state/logs/phpt-full-current-score-20260602T200309Z-php-src-f97ff59-public-6ca895a9-source-6ca895a9`;
the aggregate had `6141` passed rows, `6137` normalized current passes, and
`0` PASS regressions. The invalid-proof-marker summary reported `0` hits.

Previous accepted public gate: checkpoint `c307401c` completed the pinned full
PHPT gate with `6090 / 20294 = 30.01%` and zero latest-published PASS
regressions against the `663e3142` baseline. Full gate evidence is in
`state/logs/phpt-full-current-score-20260602T185136Z-php-src-f97ff59-public-c307401c-source-c307401c`;
the aggregate had `6090` passed rows, `6086` normalized current passes, and
`0` PASS regressions. The invalid-proof-marker summary reported `0` hits.

Previous accepted public gate: checkpoint `663e3142` completed the pinned full
PHPT gate with `5941 / 20294 = 29.27%` and zero latest-published PASS
regressions against the `ac94984a` baseline. Full gate evidence is in
`state/logs/phpt-full-current-score-20260602T160646Z-php-src-f97ff59-public-663e3142-source-663e3142`;
the aggregate had `5941` passed rows, `5937` normalized current passes, and
`0` PASS regressions. The invalid-proof-marker summary reported `0` hits.

Earlier accepted public gate: checkpoint `ac94984a` completed the pinned full
PHPT gate with `5892 / 20294 = 29.03%` and zero latest-published PASS
regressions against the `538c136c` baseline. Full gate evidence is in
`state/logs/phpt-full-current-score-20260602T143814Z-php-src-f97ff59-public-ac94984a-source-ac94984a`;
the aggregate had `5892` passed rows, `5888` normalized current passes, and
`0` PASS regressions. The invalid-proof-marker summary reported `0` hits.

Earlier accepted public gate: checkpoint `538c136c` completed the pinned full
PHPT gate with `5816 / 20294 = 28.66%` and zero latest-published PASS
regressions against the `0793abd4` baseline. Full gate evidence is in
`state/logs/phpt-full-current-score-20260602T132934Z-php-src-f97ff59-public-538c136c-source-538c136c`;
the aggregate had `5816` passed rows, `5812` normalized current passes, and
`0` PASS regressions. The invalid-proof-marker summary reported `0` hits.

Earlier accepted public gate: checkpoint `0793abd4` completed the pinned full
PHPT gate with `5744 / 20294 = 28.30%` and zero latest-published PASS
regressions against the `12c1be0a` baseline. Full gate evidence is in
`state/logs/phpt-full-current-score-20260602T121433Z-php-src-f97ff59-public-0793abd4-source-0793abd4`;
the aggregate had `5744` passed rows, `5740` normalized current passes, and
`0` PASS regressions. The invalid-proof-marker summary reported `0` hits.

Earlier accepted public gate: checkpoint `12c1be0a` completed the pinned full
PHPT gate with `5690 / 20294 = 28.04%` and zero latest-published PASS
regressions against the `4f1c81d5` baseline. Full gate evidence is in
`state/logs/phpt-full-current-score-20260602T104841Z-php-src-f97ff59-public-12c1be0a-source-12c1be0a`;
the aggregate had `5690` passed rows, `5686` normalized current passes, and
`0` PASS regressions. The lone invalid-marker grep hit remains the known
expected socket `Permission denied` warning in `run-tests.log`, not a
publication blocker.

Earlier accepted public gate: checkpoint `4f1c81d5` completed the full pinned
PHPT gate with `5513 / 20294 = 27.17%` and zero latest-published PASS
regressions against the `2755fc15` baseline. The gate accepted the 6-row
stream/INI/array focused batch covering `bug71884.phpt`,
`stream_context_create_error.phpt`, `get_extension_funcs_basic.phpt`,
`ini_set_types.phpt`, `array_fill_keys_variation1.phpt`, and
`array_fill_keys_variation2.phpt`. Full gate evidence is in
`state/logs/phpt-full-batch024-next14-20260601T192923Z-php-src-f97ff59-public-2755fc15-source-4f1c81d5`;
the aggregate had `5513` passed rows, `5509` normalized current passes, and
`0` PASS regressions.

Historical source-head note: the local supervisor notes below are older
pre-gate proof retained for chronology. Current source and public score
accounting are recorded in the current-state section above.

Local source checkpoint note: checkpoint `1efcacf6` passed
`tools/checkpoint.sh` for the next post-public focused source batch. It covers
6 selected PHPT rows across `request_parse_body()` CLI option validation and
no-content-type exception handling, C/POSIX `nl_langinfo()` day/month/radix
metadata, and bounded caught `Throwable` stringification for the reached
`sprintf()` error row. Focused Rust passed `383 / 383` across
`request_parse_body_builtin`, `object_model`, `sprintf_builtin`, and
`locale_string_builtins`; `cargo build -p phpc --bin phpc` passed; selected
PHPT proof passed `6 / 6` for
`multipart_options_invalid_key.phpt`,
`multipart_options_invalid_quantity.phpt`,
`multipart_options_invalid_value_type.phpt`, `options_array_references.phpt`,
`nl_langinfo_basic.phpt`, and `sprintf_rope_optimization_002.phpt`. This is
not a public score update until a full pinned PHPT gate completes.

Public checkpoint note: checkpoint `4f1c81d5` passed `tools/checkpoint.sh` for
the next post-`1efcacf6` worker batch and then passed the full pinned PHPT gate
described above. Focused Rust passed `66 / 66` across
`stream_resource_builtin`, `general_function_builtins`, `ini_builtins`, and
`array_fill_keys`; `cargo build -p phpc --bin phpc` passed; selected PHPT proof
passed `6 / 6`; and the public comparable gate accepted the batch with zero
PASS regressions.

Local supervisor note: the next focused source slice is integrated for 3
additional target PHPT rows: `dir_variation5.phpt`, `dir_variation6.phpt`, and
`opendir_error2.phpt`. It preserves existing local directory resource and
`open_basedir` behavior while adding PHP-shaped `Failed to open directory`
warnings for missing, non-directory, or unreadable local paths before returning
`false`. Focused Rust passed `1 / 1`, build passed, `cargo fmt --check` and
`git diff --check` passed, selected directory PHPT verification passed `4 / 4`,
and the broader completed-worker smoke passed `17 / 17`. This is not a public
score update until checkpoint validation and a full pinned PHPT gate complete.

Local supervisor note: the next focused worker batch is integrated for 11
additional target PHPT rows: `DateTime_getOffset_basic1.phpt`,
`DateTime_getTimeZone_basic1.phpt`, `DateTime_setTimezone_basic1.phpt`,
`crc32.phpt`, `hash_algos.phpt`, `ReflectionFunction_isVariadic_basic.phpt`,
`ReflectionClass_getConstructor_basic.phpt`,
`ReflectionClass_hasMethod_002.phpt`, `umask_basic.phpt`,
`umask_variation1.phpt`, and `umask_variation2.phpt`. Focused Rust passed
`24 / 24`, `cargo build -p phpc --bin phpc` passed, and selected PHPT
verification passed `11 / 11`. This is not a public score update until
checkpoint validation and a full pinned PHPT gate complete.

Local supervisor note: the completed array/string worker lane is now repaired
on the current supervisor head. `array_intersect_variation9.phpt` now passes
after matching the reached two-dimensional-array `Array to string conversion`
warning counts; the full selected lane passed `6 / 6` for
`array_count_values.phpt`, `array_count_values2.phpt`,
`array_intersect_variation9.phpt`, `similar_text_basic.phpt`,
`str_word_count.phpt`, and `str_word_count1.phpt`. Focused Rust passed
`12 / 12` across `array_intersect` and `array_count_values`; build passed.
This is not a public score update until checkpoint validation and a full pinned
PHPT gate complete.

Local supervisor note: the next focused DateTime constants slice is integrated
for 2 additional target PHPT rows: `DateTime_constants.phpt` and
`date_constants.phpt`. `DateTime` now exposes bounded date-format class
constants matching the existing global `DATE_*` constants, with reached
`DATE_RFC7231` / `DateTime::RFC7231` deprecation diagnostics. Focused Rust
passed `14 / 14`, build passed, and selected PHPT verification passed `2 / 2`.
This is not a public score update until checkpoint validation and a full pinned
PHPT gate complete.

Local supervisor note: the next worker-integrated focused source batch covers
8 additional target PHPT rows: `substr.phpt`, `fread_error.phpt`,
`sleep_error.phpt`, `usleep_basic.phpt`, `usleep_error.phpt`,
`is_writable_error.phpt`, `base64_encode_basic_001.phpt`, and
`base64_loop_001.phpt`. Worker PHPT verification passed `8 / 8`; supervisor
focused Rust passed `59 / 59`, build passed, and integrated PHPT verification
passed `8 / 8`. This is not a public score update until checkpoint validation
and a full pinned PHPT gate complete.

Local supervisor note: the next string/security worker batch is integrated in
source for `bug45485.phpt`, `bug78003.phpt`, `bug51059.phpt`, and
`escapeshellarg_basic.phpt`; worker PHPT verification passed `4 / 4`,
supervisor focused Rust passed `11 / 11`, build passed, and integrated PHPT
verification passed `4 / 4`. This is not a public score update until
checkpoint validation and a full pinned PHPT gate complete.

Local supervisor note: the next array worker batch is integrated in source for
`array_keys_variation_003.phpt` and `array_udiff_variation5.phpt`; worker PHPT
verification passed `2 / 2`, supervisor focused Rust passed `12 / 12`, build
passed, and integrated PHPT verification passed `2 / 2`. The combined
supervisor focused gate across all touched worker suites passed `82 / 82` Rust
tests, and the combined selected PHPT check passed `14 / 14`.
This is not a public score update until checkpoint validation and a full pinned
PHPT gate complete.

Local supervisor note: five completed sidecar lanes are now integrated in
source for 11 additional focused PHPT rows: `putenv.phpt`,
`putenv_and_getenv_reject_null_bytes.phpt`, `file_exists_variation1.phpt`,
`file_get_contents_error_folder.phpt`, `bug61660.phpt`, `bug67249.phpt`,
`bug75075.phpt`, `bug78833.phpt`,
`array_column_scalar_index_strict_types.phpt`, `math/constants.phpt`, and
`math/bug27646.phpt`, plus the unique `is_writable_variation1.phpt` and
`is_writable_variation3.phpt` rows from the filesystem metadata lane. This is
verified by the supervisor with `162 / 162` focused Rust tests,
`cargo build -p phpc --bin phpc`, and `27 / 27` selected PHPT rows. This is
not a public score update until checkpoint validation and a full pinned PHPT
gate complete.

Local supervisor note: post-Batch024 focused source work is verified for
`count_symbol_table.phpt`, `strcasecmp_basic.phpt`, trim/rtrim rows, and
replacement-family rows. Focused Rust passed `68 / 68`, focused `php_runtime`
passed `2 / 2`, and focused PHPT passed `7 / 7`. This is not a public score
update until checkpoint validation and a full pinned PHPT gate complete.

Local supervisor note: the current high-parallel focused batch is verified for
24 selected PHPT rows: `array_change_key_case_flag_error.phpt`,
`array_chunk2.phpt`, `array_chunk_variation5.phpt`,
`array_fill_error2.phpt`, `array_pad_too_large_padding.phpt`,
`array_is_list.phpt`, `prev_error2.phpt`, `prev_error3.phpt`,
`fgets_error.phpt`, `call_user_func_002.phpt`,
`is_callable_variation2.phpt`, `join_error1.phpt`, `chr_error.phpt`,
`printf_error.phpt`, `fprintf_error.phpt`, `printf_64bit.phpt`,
`strcmp.phpt`, `strpos.phpt`, `stripos.phpt`, `stripos_error.phpt`,
`strlen_basic.phpt`, `strlen.phpt`, `Zend/tests/strlen.phpt`, and
`Zend/tests/strlen_deprecation_to_exception.phpt`. Focused Rust passed
`218 / 218`, and focused PHPT passed `24 / 24`. This is not a public score
update until checkpoint validation and a full pinned PHPT gate complete.

Local supervisor note: the next focused source batch is in progress for
`intval_binary_prefix.phpt`, array pointer rows, and follow-on array diagnostic
lanes. The current intval/array-pointer subset has focused Rust `25 / 25` and
focused PHPT `4 / 4` passing. This is not a public score update until
checkpoint validation and a full pinned PHPT gate complete.

Local supervisor note: strict-identity `--emit-ir` Rust assertions were
refreshed for the already-present boxed diagnostic-result echo boundary. The
focused strict-identity suite passes locally, but this is not a public score
update.

Local supervisor note: the `$_SESSION` undefined-read Rust baseline now matches
the current PHP-shaped warning-and-continue execution path before session
startup. The focused superglobals suite passes locally, but this is not a
public score update.

Local supervisor note: `syntax_boundaries` was refreshed for the current
parser/runtime/codegen boundaries, including accepted attribute/function-DNF
metadata, first-class callables, `\PHP_VERSION`, parenthesized dynamic `new`,
native spread lowering, and direct reference assignment. The focused
syntax-boundaries suite passes locally, but this is not a public score update.

Local supervisor note: Worker 01 landed a bounded `count()` / `sizeof()`
operand-name improvement that turns two focused public PHPT rows green in its
lane report, and native type-introspection assertions now keep non-string
`function_exists()` / `extension_loaded()` names at the explicit function-call
rejection boundary. Focused Rust suites pass locally, but this is not a public
score update.

Local supervisor note: the typed-property protected-inheritance Rust baseline
now matches the current PHP-shaped fatal text for typed children extending
untyped protected parent properties. Worker lanes for `$GLOBALS`,
`intval($value, $base)`, `str_ireplace(..., $count)`, `is_callable()`, and
filesystem `open_basedir` metadata predicates have returned candidate patches
for later integration, but the public score remains unchanged until a full
pinned PHPT gate is published.

Local supervisor note: the typed-property reference-coercion Rust baseline now
asserts the current PHP-shaped fatal execution path for incompatible writes
through typed property references. The focused suite passes locally, but this
is not a public score update.

Local supervisor note: the `variable_unset` Rust baseline now matches the
current PHP-shaped warning-and-continue path when reading a local after
`unset()`. The focused suite passes locally, but this is not a public score
update.

Local supervisor note: checkpoint `263f60c4` produced a candidate full pinned
PHPT score of `5361 / 20294 = 26.42%`, but publication was blocked by two
latest-published PASS regressions: `is_file_variation4.phpt` and
`vfprintf_error4.phpt`. Checkpoint `43262ab5` repaired both regressions and
the rerun published `5363 / 20294 = 26.43%` with zero latest-published PASS
regressions.

## Current Public Gate

Published gate: Batch024 regression repair.

- Gate run:
  `phpt-full-batch024-regression-repair-20260601T145651Z-php-src-f97ff59-source-43262ab5`
- Source: `43262ab5f81fe293a49829c9c270137be98f5e6d checkpoint: repair PHPT gate regressions`
- Score: **5363 / 20294 pinned runnable PHPTs = 26.43%**
- Regression result: zero latest-published PASS regressions against the
  Batch023 repair01 PASS baseline.
- Gate notes: the full `open_basedir_*` family was serialized; the known
  sockets expected-output marker was adjudicated as failed-row output, not a
  harness marker failure. The previously blocking Batch024 candidate
  regressions `is_file_variation4.phpt` and `vfprintf_error4.phpt` now pass.

No focused PHPT run, source checkpoint, status note, PR, or candidate gate
changes the public score until it is parsed, regression-checked against the
latest published PASS set, and recorded here.

## Blocked / Unpublished Candidates

- Batch023 checkpoint10 was superseded by Batch023 repair01. Its candidate
  score is no longer current public progress.

## Batch024 Staging Checklist

Batch024 is accumulating source fixes after the Batch023 repair01 full-suite
gate. Focused PHPT proof is used for each accepted source slot, but the public
percentage does not change until a supervisor-owned full-suite gate is run for
the batch, all latest-public PASS regressions are repaired or adjudicated, and
the accepted score is recorded here.

Accepted source slots as of public/source head
`67c7a328a1e819c05e723f9f012763210b219a21`:

- [x] Slot 1: `7fdd2f668f5f61a788e53292b42f32e682cbc72a` URI
  WhatWG residuals, patch `sha45916ebb`: source integrated after reviewer
  FINAL GO, two critic SAFE artifacts, p38-ready, supervisor proof, focused
  PHPT `34 / 34`, and `0 / 34` latest-public PASS overlap.
- [x] Slot 2: `49a8c6fceb0bd562837b5a03b8b36734529a3a70` array
  negative auto-key semantics, patch
  `2cd2baeb161e48342c04881d18d890f2b1f830cd2a3159f1b550b33acfc483f0`,
  focused PHPT `2 / 2`, and `0 / 2` latest-public PASS overlap.
- [x] Slot 3: `00679068891d394d7e34ec9e49ef9d54781ce620`
  tokenizer PhpToken error-line provenance, patch
  `0cb2f2a216cc816aca82d35a4b72d4da775b1a6087e53437c0700ee63dc36335`,
  focused PHPT `2 / 2`, and `0 / 2` latest-public PASS overlap.
- [x] Slot 4: `fc7b886987a06730ad9bf00354cbe8b6360eb06a` INI
  parse/scanner overflow semantics, patch
  `ef881c5249b61db4a348ff6617f89bc9a1faf9c88e5dd46e424a3c80fc22c269`,
  focused PHPT `2 / 2`, and `0 / 2` latest-public PASS overlap.
- [x] Slot 5: `0b1e988917a4067addd776ee4e99f817eba1b8d9` PCRE
  backtrack/preg semantics, patch
  `4af9d61cdb763be3b54c4d701a004b7403e0832605a8df6acba92727030b4c81`,
  focused PHPT `7 / 7`, and `0 / 6` latest-public PASS overlap.
- [x] Slot 6: `cc07c0517915ce9cd730fcb56b4429fdd42f35ce` INI
  config/readback arg-separator/raw-scanner semantics, patch
  `4b5e8166e7828364da251f956cdfd1b4da0d78ec6889953f88de823d32902403`,
  focused PHPT `3 / 3`, and `0 / 3` latest-public PASS overlap.
- [x] Slot 7: `8ca79b41ae7ccecdf1aa4e030d9a708fa509429d`
  Reflection static/default property metadata, patch
  `1768a94679b0e3c51f4ce68cfae0540680433b03659db5873b42ebf00c10da26`,
  focused PHPT `7 / 7`.
- [x] Slot 8: `488c15046f0e121041dd588f4fa121a6b04e6551` mbstring
  `mb_substr()` default encoding semantics, patch
  `cf583257d7dc3dec9b7dbb6596c5b5d9bf1efa935ee11117107c1357d7706b0d`,
  focused PHPT `6 / 6`.
- [x] Slot 9: `7d1df541423a10f0b84afe40871c3cd787c7ccde`
  ReflectionExtension registry semantics, patch
  `eccafe08a0b22878cdb8c122ee1aad6deb9e5651d8f72be12745b510406d8a23`,
  focused PHPT `4 / 4`.
- [x] Slot 10: `ae137fe082c55451ab5dbb939d8dd3b25da5eb88`
  standard `settype()` casting semantics, patch
  `19d05729f6e7b874ee698f9dd0180c0d1ced3e94fe6273ad4674439d75587974`,
  focused PHPT `8 / 8`.
- [x] Slot 11: `263c97b65b36a046c9992c6211ced5f1657795ea`
  random procedural function semantics, patch
  `49353024ec2a3c03680aac2fdb1e360abfc74f891787e39a90686c473b87f8e7`,
  focused PHPT `11 / 11`.
- [x] Slot 12: `0c8fc57b695a1ebb002c1cb3e43f3e8f891d30ff` Zend
  offset tail semantics, patch
  `c7ab2012f3b66e124ba08b691bbc8221fbd1e0b2fab608a600f0caa5b60ad3e1`,
  focused PHPT `7 / 7`.
- [x] Slot 13: `34cd7257c7eb5ba8dc8359b7d1cb515cac2c1a5f` Zend
  union bool/default semantics, patch
  `066b314ede348502db732031b6a246da3240443c77f364fe0d739070d5fe499c`,
  focused PHPT `2 / 2` and literal bool probes `4 / 4`.
- [x] Slot 14: `67c7a328a1e819c05e723f9f012763210b219a21`
  standard stream-context diagnostics, patch
  `ae68026fe03088451b9c904e2bba5fd11ceff4e10a21f41276a3839c3453cb36`,
  focused PHPT `3 / 3` and non-repeat Rust guard `1 / 1` for the old
  `fwrite('not-resource', 'x')` diagnostic path.
- [x] Batch024 full-suite gate: published as Batch024 regression repair at
  `5363 / 20294 = 26.43%` with zero latest-published PASS regressions.

Current rejected, reverted, stale, or still-pending Batch024 candidates:

- `d93cc660db8e49b5e8f311f101f73b8ff6492bd5` INI parse/scanner
  successor, patch
  `624013a7ed8f8ebee92a53c90054a629afe090c586ce7fc324a8ba231cc8d67b`,
  was reverted by `1db84c72f92be1d9c15dee22d984338450a12ff0` after a
  critic DO-NOT-SAFE overflow counterexample; `1db84c72` has no tree diff
  against prior good `49a8c6fceb0bd562837b5a03b8b36734529a3a70`.
- `181e5838` Zend union defaults is rejected for the legal
  `false|int $x = false` default counterexample.
- `c62d8fa3` PCRE helper APIs is rejected because
  `preg_replace_callback_array()` must preserve sequential callback side
  effects before a later invalid pattern returns `NULL`.
- `a86157c9` PCRE match hygiene and successor `0e8556de` are rejected for
  overbroad backtrack-limit behavior; later PCRE v2/v3/v4 packets
  `da55a954`, `4467ce8e`, and `1b231a6a` also have FINAL NO-GO artifacts,
  while accepted PCRE v5 is slot 5 above.
- `f96d5381` INI parse quantity and `cd6699e9` tokenizer PhpToken object
  diagnostics have reviewer NO-GO artifacts.
- Reflection static/default candidates `04e2c7f3`, `0d7353b5`, and
  `f2d73dc1` are rejected/stale; accepted Reflection v4 is slot 7 above.
- HTML entity split candidates `f75f5e48`, `e7da26fb`, and `c580522b` are
  rejected by formatter or focused-Rust gates.
- Tokenizer lexical-tail `a1a95bb0` is rejected for the
  `PhpToken_constructor.phpt` public-guard object id failure. The repaired
  author-ready patch
  `b11122f97088323365a078a7badc0e03969f7549e91847f033c71ea6dcb5e7b2`
  remains held after exact-current `bcs3-tokenizer-review-status.md` found
  broad object-id lifetime risk, numeric underscore/overflow mismatches,
  under-modeled `TOKEN_PARSE` trait adaptation contexts, and raw-byte
  non-canonical cast deprecation false positives. The supervisor integration
  worktree no longer carries tokenizer source/test changes for this candidate.
- Local supervisor native-link checkpoint maintenance refreshed stale
  generated-C assertion shapes and wired runtime callable `strpos(...)`
  through the string-search result path. The same checkpoint pass refreshed
  stale native logical-boundary emit-IR snapshots for the boxed
  diagnostic-result output path and stale native mutation-boundary assertions
  for boxed assignment/compound output, non-local unset diagnostics, and the
  current direct-variable reference-binding split. This is not a public score update:
  source-only `native_executable_c_source` passed `417 / 417`, full
  `native_link` passed `823 / 823`, focused `native_logical_boundary` passed
  `19 / 19`, focused `native_mutation_boundary` passed `12 / 12`, and focused
  `native_object_class_boundary` passed `57 / 57`, focused
  `native_runtime_abi` passed `80 / 80`, focused `native_scalar_echo_boundary`
  passed `8 / 8`, focused `native_string_arithmetic` passed `4 / 4`, and
  focused `native_type_introspection_boundary` passed `18 / 18`, and focused
  `native_unary_boundary` passed `32 / 32`.
- Local supervisor high-intensity worker candidates are now integrated as
  focused Batch024 source candidates, still not a public score update until the
  next full gate: trim-family PHPT proof passed `4 / 4`, numeric angle
  conversion PHPT proof passed `4 / 4`, `file()` open_basedir follow-on
  warning PHPT proof passed `2 / 2`, and `empty()` expression PHPT proof
  passed `1 / 1` through lowercase `run-tests.php -p` with the wrapper.
- Local supervisor `str_replace()` baseline maintenance refreshed stale Rust
  assertions for already-present callback-by-value count warnings and current
  one-level array replacement behavior. Direct-variable `$count` writeback
  remains the only supported direct mutation path; non-direct direct-call count
  targets still reject, while `call_user_func("str_replace", ..., 0)` warns and
  returns the replacement result. Focused `str_replace_builtin` passed `7 / 7`;
  this is not a public score update.
- Local supervisor `strcasecmp()` baseline maintenance refreshed the stale
  too-few-arguments expectation to the current PHP-shaped fatal execution
  result. Focused `strcasecmp_builtin` passed `4 / 4`; this is not a public
  score update.
- Local supervisor `open_basedir` metadata/directory denial repair is a
  Batch024 source checkpoint candidate and still not a public score update. It
  covers
  relative-parent escape denial for `file_exists()`, `filesize()`, `is_dir()`,
  `is_file()`, `is_readable()`, `is_writable()`, `is_link()`,
  `file_get_contents()`, `file_put_contents()`, `fopen()`, `opendir()`/`dir()`,
  and `scandir()`, including bounded follow-on open warnings for stream and
  directory open denials. Worker proof in
  `bcs3-openbasedir-author-status.md` passed focused Rust gates, a 10-row PHPT
  author packet, five guard rows, and adjacent `is_link`/`is_writable` rows;
  supervisor post-integration Rust and focused PHPT proof also passed. The
  relative-escape Rust test now guards its cwd-mutating `chdir()` /
  `open_basedir=.` cases so the default parallel test runner cannot race the
  process cwd; both default and `--test-threads=1` focused runs passed. During
  checkpoint preparation, stale `php_runtime` unit assertions were refreshed
  for existing binary-string, core-class, `natsort`, and typed static-property
  reference behavior, and the test-only call-arguments free counter was made
  thread-local so the default parallel Rust test runner tracks current runtime
  semantics without counter cross-talk. Stale generated-C assertions were also
  refreshed for existing dynamic object-property assignment and binary-string
  comparison lowering, stale default-parameter coverage was refreshed for the
  existing declared `ClassName::CONST` interpreter default-value path, and stale
  dynamic-feature assertions were refreshed for the existing PHP-shaped fatal
  execution, eval/import/constant, error-control, and variable-variable
  boundaries, with matching unsupported dynamic feature fixture/CLI sidecars
  refreshed for the current diagnostics. The `runtime_errors` fixture sidecars
  were also refreshed against current `phpc run` behavior while preserving
  their `phpc-only` status and replacing stale `PHP_OS` unknown-constant
  probes with `PHP_OS_MISSING`; `file_exists()` focused tests were refreshed
  for current PHP-shaped arity fatals and direct-interpreter relative source
  path fixture resolution, and `filesize()` focused tests were refreshed for current
  directory metadata, warning recovery, scalar path coercion, and arity fatal
  behavior. `fprintf()`/`vfprintf()` focused tests were refreshed for the
  current shared stream-resource native-lowering boundary, scalar format
  coercion, and PHP-shaped values-argument TypeError. `functions_and_scopes`,
  modulo, shift, native variable-read, `runtime_errors`, and `implode()` Rust
  baselines were refreshed for current PHP-shaped warning/fatal execution, accepted
  `declare(strict_types)` / `declare(encoding)` parser behavior, named
  reference-argument support under non-builtin-style function names,
  ArrayAccess/null-offset deprecation output, scalar `implode()` separator
  coercion, PHP-shaped `implode()` TypeErrors, and array-to-string warning
  recovery. `ini_builtins` default-registry assertions now use the shared
  `PHPC_PHPT_INI_FLAGS` lock/restore discipline so Rust's parallel test runner
  cannot leak PHPT memory-limit overrides across tests; milestone159,
  milestone160, and milestone162 fatal sidecars now match the current
  stdout/exit shape. Focused `is_dir()`/`is_file()`/`is_readable()`/
  `is_writable()` tests now assert current PHP-shaped zero-argument fatal
  execution, matching the already-refreshed `is_link()` boundary.
  `list_assignment` now proves the intended native array-destructuring blocker
  with literal RHS values, leaving RHS call-boundary routing to the dedicated
  native-array boundary test. Magic constant CLI/fixture sidecars for the
  non-trait-originated `__TRAIT__` and global-namespace `__NAMESPACE__` cases
  were refreshed to the current successful runtime output and no longer carry
  `phpc-only` markers. Whole-tree fixture execution was stabilized between
  cargo integration tests and `phpc test` by resolving existing repo-relative
  local filesystem operation paths through the repository root when cargo runs
  from the `compiler` crate, keeping self-referential metadata/directory
  fixtures aligned without changing missing-path behavior. The stale
  milestone1 native-boundary assertions were refreshed for the current
  variable-read and non-local-assignment blockers, the full fixture/CLI
  sidecar tree was refreshed against current `phpc run`, and remaining
  system-PHP comparison divergences for stream-context diagnostics,
  ArrayAccess append/null-offset deprecation output, whole-array copied-source
  COW identity, and `array_key_exists(null, ...)` deprecation output now carry
  explicit `phpc-only` reasons. `path_builtins` now asserts the current
  `dirname(42)` weak scalar-path coercion result and passed focused `14 / 14`.
  The shutdown callback runner now clears the pending `exit_signal` only while
  draining registered shutdown callbacks, so callbacks registered before
  `exit("...")` execute before the original exit status is restored; focused
  `shutdown_function_builtin` passed `6 / 6`. `is_executable()` now matches
  PHP's silent `false` result for regular-file trailing-separator probes while
  preserving executable-file and executable-directory behavior; focused
  `standard_file_metadata_residual_builtins` passed `2 / 2`. The standard
  file-metadata open_basedir tests now guard their cwd-mutating cases, and
  `standard_file_metadata_builtins` passed `7 / 7` under Rust's default
  parallel runner. String predicate baselines now match the current
  PHP-shaped runtime arity fatal path and direct native string-predicate ASM
  lowering for `str_contains()`, `str_starts_with()`, and `str_ends_with`;
  focused tests passed `5 / 5`, `6 / 6`, and `6 / 6`.
  `functions_and_scopes` system-PHP/runtime
  oracle assertions now normalize those null-offset deprecation lines while
  still comparing the payload behavior. Namespace-resolution baselines now
  expect PHP-shaped fatal executions for undefined imported/non-imported
  function calls and record the current generated-C object-instantiation
  lowering boundary for the imported-type-alias static-property probe. Native
  arithmetic baselines now track current boxed diagnostic-result echo output,
  scalar-coercion generated-C value-operation routing, LLVM string/unary-negative
  operand conversion routing, and the current modulo split where zero/dynamic
  divisors reject while unary negative literal divisors route through the
  value-result boundary. Native assembly CLI baselines now accept current
  helper-based IR/C output-call shapes and record the current `--emit-asm`
  rejection boundaries for unary and bitwise/shift cases that remain outside
  LLVM assembly lowering. Native bitwise baselines now track current boxed
  diagnostic-result echo output, avoid unrelated unary-negative lowering
  boundaries in all-ones/negative shift assertions, and refresh current
  bitwise/shift emit-IR sidecars. Native cast baselines now include `(object)`
  casts in the existing scalar/array cast rejection boundary and refresh the
  emit-IR/emit-ASM sidecars plus the shared runtime ABI assertion. Native
  comparison baselines now track current boxed diagnostic-result echo output,
  comparison/strict-identity emit-IR sidecars, folding snapshots, and explicit
  unsupported-comparison rejection boundaries. Generated-C dynamic string
  comparison operand baselines now avoid the unrelated variable-held
  conditional-expression boundary and assert the current native value
  byte-string materialization, explicit byte-length tracking, and native value
  comparison helper path. Native concatenation baselines now track the current
  boxed diagnostic-result echo path for dynamic string output, including
  empty-string identity concatenation over untracked string expressions,
  static string concatenation, and single-result string ternary concatenation
  emit-IR sidecars. Native conditional boundary baselines now track the current
  boxed diagnostic-result echo path for scalar, string, boolean, and null
  ternary output while preserving the existing unsupported conditional-expression
  rejection boundary and refreshing the conditional emit-IR sidecars. Native
  `empty()` boundary baselines now track the current boxed diagnostic-result
  echo path for direct-variable empty output and record the current split where
  array/property/static-property operands reject at the `empty()` boundary,
  while multi-argument and call-operand forms reject at the generic native
  function-call boundary. Native function-call boundary baselines now track
  the current boxed diagnostic-result echo path for folded `strlen(...)`
  output, refresh the `native_strlen` emit-IR CLI snapshot, keep unsupported
  direct-call argument diagnostics on their exact call-site columns, and
  separate generated-C rejection coverage from generated-C dynamic
  call/value-result forms that now compile through the bounded runtime
  callable path. Native global-constant boundary `defined(...)` emit-IR CLI
  snapshots now track the same boxed diagnostic-result echo path for folded
  builtin, missing, and sort-mode constant-name results. Native `isset()`
  boundary baselines now track the current boxed diagnostic-result echo path
  for direct-variable isset output and record the current split where
  array/property/static-property operands reject at the `isset()` boundary,
  while multi-argument and call-operand forms reject at the generic native
  function-call boundary. Focused preflight gates for those clusters passed,
  including `cargo test -p phpc --test milestone1`, `cargo test -p phpc --test
  namespace_resolution`, `cargo test -p phpc --test native_arithmetic_boundary`,
  `cargo test -p phpc --test native_assembly_cli`,
  `cargo test -p phpc --test native_bitwise_boundary`,
  `cargo test -p phpc --test native_cast_boundary`,
  `cargo test -p phpc --test native_comparison_boundary`,
  `cargo test -p phpc --test native_comparison_dynamic_string_operands`,
  `cargo test -p phpc --test native_concat_boundary`,
  `cargo test -p phpc --test native_conditional_boundary`,
  `cargo test -p phpc --test native_empty_boundary`,
  `cargo test -p phpc --test native_function_call_boundary`,
  `cargo test -p phpc --test native_global_constant_boundary`,
  `cargo test -p phpc --test native_isset_boundary`,
  `phpc test tests/fixtures`, and `phpc test --compare-php tests/fixtures` with
  `2419` clean fixtures, `1691` comparisons, and `728` `phpc-only` skips. This
  slot still needs the next full Batch024 gate before publication.
- Local supervisor null-coalescing baseline maintenance refreshed stale tests
  for the current PHP-shaped fatal execution result on external inaccessible
  private-property read and assignment paths. Focused
  `cargo test -p phpc --test null_coalescing` passed `21 / 21`; this is not a
  public score update.
- Local supervisor object-model baseline maintenance refreshed stale current
  core class/interface inventory assertions and PHP-shaped fatal execution
  expectations for object/type errors. Focused
  `cargo test -p phpc --test object_model` passed `362 / 362`; this is not a
  public score update.
- Stream-context diagnostics `e7c0dd44` is rejected for returning
  `Ok(Execution)` with fatal PHP output instead of a runtime `Diagnostic` for
  `fwrite('not-resource', 'x')`; accepted repaired stream patch `ae68026f` is
  slot 14 above.

## Score History

| Gate | Passed / pinned runnable | Percent | Publication note |
| --- | ---: | ---: | --- |
| Batch001 baseline | 1118 / 20294 | 5.51% | Initial pinned full-suite baseline |
| Batch002 | 1193 / 20294 | 5.88% | 0 PASS regressions |
| Batch003 | 1311 / 20294 | 6.46% | 0 PASS regressions |
| Batch004 checkpoint8 repair | 1369 / 20294 | 6.75% | 0 PASS regressions |
| Batch004 checkpoint10 | 1413 / 20294 | 6.96% | 0 PASS regressions |
| Batch005 checkpoint10 | 1618 / 20294 | 7.97% | 0 semantic regressions |
| Batch006 checkpoint10 | 1836 / 20294 | 9.05% | 0 PASS regressions |
| Batch007 checkpoint10 repair | 2047 / 20294 | 10.09% | 0 PASS regressions |
| Batch008 checkpoint5 | 2180 / 20294 | 10.74% | 0 PASS regressions |
| Batch008 checkpoint10 repair | 2286 / 20294 | 11.26% | 0 PASS regressions |
| Batch009 burst1 | 2388 / 20294 | 11.77% | 0 PASS regressions |
| Batch010 checkpoint10 repair | 2563 / 20294 | 12.63% | 0 semantic regressions |
| Batch011 burst1 | 2741 / 20294 | 13.51% | 0 PASS regressions |
| Batch012 dynamic-call repair | 2945 / 20294 | 14.51% | 0 PASS regressions |
| Batch013 checkpoint10 | 3170 / 20294 | 15.62% | 0 PASS regressions |
| Batch014 regression repair | 3378 / 20294 | 16.65% | 0 PASS regressions |
| Batch015 checkpoint9 | 3646 / 20294 | 17.97% | 0 semantic regressions; `bug75679.phpt` long-root guard |
| Batch016 selected integration | 3868 / 20294 | 19.06% | 0 semantic regressions; 6 platform-SKIPIF rows adjudicated |
| Batch016 regression7 repair | 4048 / 20294 | 19.95% | 0 PASS regressions |
| Batch017 checkpoint10 | 4132 / 20294 | 20.36% | 0 PASS regressions; invalid-marker hits adjudicated as failed-row output |
| Batch018 repair01 | 4178 / 20294 | 20.59% | 0 PASS regressions; invalid-marker hits adjudicated |
| Batch019 repair02 | 4321 / 20294 | 21.29% | 0 semantic regressions; `bug75679.phpt` and `open_basedir_filemtime.phpt` adjudicated |
| Batch020 repair01 | 4425 / 20294 | 21.80% | 0 PASS regressions; sockets marker adjudicated |
| Batch021 regression repair | 4685 / 20294 | 23.09% | 0 PASS regressions; sockets marker adjudicated |
| Batch022 repair02 | 4949 / 20294 | 24.39% | 0 PASS regressions; sockets marker adjudicated |
| Batch023 repair01 | 5173 / 20294 | 25.49% | 0 PASS regressions |
| Batch024 regression repair | 5363 / 20294 | 26.43% | 0 PASS regressions |
| Batch024 repair hash/session | 5481 / 20294 | 27.01% | 0 PASS regressions; sockets marker adjudicated |
| Batch024 next13 | 5498 / 20294 | 27.09% | 0 PASS regressions |
| Batch024 next14 | 5513 / 20294 | 27.17% | 0 PASS regressions; stream/INI/array rows |
| Batch024 current-score `12c1be0a` | 5690 / 20294 | 28.04% | 0 PASS regressions; +177 public PHPT passes |
| Current score gate `0793abd4` | 5744 / 20294 | 28.30% | 0 PASS regressions; +54 public PHPT passes |

## Operating Rules / Gates

- Public progress is only the pinned php-src PHPT full-suite pass rate.
- The total pinned runnable PHPT denominator stays `20294` until the pin or
  inventory policy is intentionally changed and documented here.
- A candidate can publish only after a full-suite gate is parsed, every
  latest-published PASS loss is reviewed, and semantic regressions are
  repaired.
- Focused PHPT proof must use lowercase `run-tests.php -p` with the
  `phpc-phpt-wrapper`; uppercase `-P` proof does not count for publication.
- Focused tests and source checkpoints are evidence for the next gate, not a
  public percentage change.
- Harness, platform, or expected-output adjudications must name the affected
  rows and evidence. Silent score substitution is not allowed.
- Blocked candidates may be listed here only as unpublished candidates, without
  replacing the current public score.

## Evidence Pointers

- php-src pin: `/home/claude/php-src-phpt` at
  `f97ff597429a2fe633665a7e02d97c8077f9f90f`
- PHPT wrapper:
  `/home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper`
- Current accepted gate evidence:
  `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260602T121433Z-php-src-f97ff59-public-0793abd4-source-0793abd4`
- Previous current-score gate evidence:
  `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260602T104841Z-php-src-f97ff59-public-12c1be0a-source-12c1be0a`
- Previous Batch024 next14 gate evidence:
  `/home/claude/supervised-php-compiler/state/logs/phpt-full-batch024-next14-20260601T192923Z-php-src-f97ff59-public-2755fc15-source-4f1c81d5`
- Previous Batch024 repair hash/session evidence:
  `/home/claude/supervised-php-compiler/state/logs/phpt-full-batch024-repair-hash-session-20260601T180507Z-php-src-f97ff59-source-1fe2b233`
- Previous Batch024 regression repair evidence:
  `/home/claude/supervised-php-compiler/state/logs/phpt-full-batch024-regression-repair-20260601T145651Z-php-src-f97ff59-source-43262ab5`
- Previous Batch023 baseline evidence:
  `/home/claude/supervised-php-compiler/state/logs/phpt-full-batch023-repair01-sharded-serialized-openbasedir-20260531T1308Z-php-src-f97ff59-public-54829387-source-54f3c2c3`
- Previous Batch022 baseline evidence:
  `/home/claude/supervised-php-compiler/state/logs/phpt-full-batch022-repair02-sharded-serialized-openbasedir-20260531T0839Z-php-src-f97ff59-public-5530d1da-source-69c5111f`
- Previous Batch021 baseline evidence:
  `/home/claude/supervised-php-compiler/state/logs/phpt-full-batch021-regression-repair-sharded-serialized-openbasedir-20260531T0838Z-php-src-f97ff59-public-049ff7b5-source-7e9c4fd8`
- Previous Batch020 baseline evidence:
  `/home/claude/supervised-php-compiler/state/logs/phpt-full-batch020-repair01-sharded-serialized-openbasedir-20260531T0415Z-php-src-f97ff59-public-5e8f521a-source-4e7a7a41`
- Skip / xfail ledger:
  `/home/claude/supervised-php-compiler/state/php-core-suite-skip-ledger.tsv`
- Detailed chronological implementation proof remains in `docs/PROGRESS.md`.
