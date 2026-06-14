# PTN Progress

Refresh: 2026-06-14T19:47Z.
Measured: `ptn-c284`; `ptn-xymv`; `ptn-j8b8/b35n/18tp/gkvr`; `ptn-gt7b`; `ptn-30ji`; `ptn-h0qa` broad 1k classify-only 424 runnable / 576 classified; `ptn-dkcs` call-unpacking 34 selected / 0 runnable / 34 classified; `ptn-ei36` unpacking split 20 call / 14 array classified; `ptn-550s.12` broad 1k array-helper row pack 0/10 -> 10/10; `ptn-qsmv.14` class/interface row pack; `ptn-qsmv.16` array callback/map-filter row pack; `ptn-qsmv.17` assertion/runtime-config +10 broad rows and broad 1k classify-only 440/560; `ptn-s80e` broad 1k array/reference row pack 10/20 -> 20/20; `ptn-j6gv` broad 1k string/runtime row pack 15/25 -> 25/25; `ptn-55u0` broad 1k unpack row pack 2/34 raw baseline -> 10/10 runnable after split; `ptn-tiqh` COW/reference row pack 21/21 on submitted base; `ptn-ouhx` object-string array-helper row pack 0/34 -> 34/34, object-string source bucket 19/61 -> 53/61, broad 1k 285 -> 419 passing (501 runnable / 499 classified after, stitched from timed broad run plus remaining slice); `ptn-lxw1` array COW/reference row pack 9/9 focused, 2/2 candidates, 19/20 mixed control; `ptn-xcmz` broad 1k property/object metadata row pack 0/19 current-base focused baseline -> 12/12 runnable; `ptn-s8cn` call-unpacking row pack 0/20 classified -> 11/11 runnable passed and broad 1k classify-only 472/528 -> 545/455; `ptn-1d60` array_map null-reference row 65/66 -> 66/66 and broad 1k classify-only 558/442 with 370 current standard-array runnable rows and 0 standard-strings rows; COW 69/103 passed.

## Dashboard

|Source|Ported|Passing|Gap|
|---|---:|---:|---:|
|Units|3|3|0|
|Native|731|731|0|
|Bounded|486|479|7|
|Zend|119|119|0|
|ext/standard-PHPT|281|274|7|
|Array-key/cb|75|38|37|
|Array-cb-valid|66|66|0|
|Array-diff|61|59|2|
|Diff-cmp|76|67|9|
|Array-setops|119|67|52|
|Fill/pad|12|12|0|
|Array-set/cb|106|86|20|
|Array-cb-slice|38|28|10|
|FS/process|46|13|33|
|First-class-callable|12|10|2|
|basic+func+lang|78|78|0|
|COW-manifest|54|54|0|
|Nested-foreach|3|2|1|
|Array-COW|72|17|55|
|COW-foreach|103|69|34|
|Foreach-list|4|4|0|
|Ref-call|12|10|2|
|CUF-edges|12|8|4|
|Zend-assign|32|23|9|
|Recursive-dump|4|2|2|
|Classes|78|1|77|
|Zend-bugs|37|18|19|
|Class-name|10|9|1|
|Dynamic-type|44|0|44|
|Diagnostics|47|0|47|
|Non-array-meta|74|0|74|
|Class/object-meta|221|0|221|
|Class/object-prop-pack|19|12|7|
|Class/object-meta-granular|135|0|135|
|Class-metadata-split|143|0|143|
|Core/basic-op|34|18|16|
|Runtime-INI|73|0|73|
|Resource-limit|1|0|1|
|Magic/object|69|20|49|
|Magic-methods|8|0|8|
|Object-string-meta|61|53|8|
|Object-string-array-pack|34|34|0|
|Std-array-map|297|0|297|
|Std-arrays|296|263|33|
|Map/filter|30|25|5|
|Request/SAPI|41|1|40|
|Anon-class|15|0|15|
|Interface-decl|23|0|23|
|Interface-impl|15|0|15|
|Trait-decl|25|0|25|
|Call-unpack|20|11|9|
|Array-unpack|14|10|4|
|Type-hint|14|0|14|
|Function-state|11|0|11|
|Dynamic-symbol|8|3|5|
|Generator-runtime|1|0|1|
|Internal-call-bind|1|1|0|
|Attribute-syntax|141|0|141|
|Attribute-internal|8|0|8|
|Attribute-meta|204|0|204|
|Heredoc-array|70|20|50|
|Std-array-row-pack|10|10|0|
|Std-array-s80e|20|20|0|
|Std-array-tdei|71|61|10|
|Std-array-lxw1|20|19|1|
|array_rand|7|6|1|
|Zend-op/control|26|15|11|
|Binary-key|1|1|0|
|Runtime-config|54|10|44|
|COW-gate|26|26|0|
|COW-reference-tiqh|21|21|0|
|1k-baseline|1000|419|581|

## 2026-06-14 ptn-1d60 Array Map Null References

Focused manifest: `tools/phpt-array-callback-validation-manifest.txt`.

Pre-fix focused artifact `20260614T185146Z` selected 66 runnable rows, with
65 passed and the single failure `ext/standard/tests/array/array_map_variation2.phpt`.

Current rebased branch artifact `20260614T192727Z` selected the same 66 rows:
66 runnable, 66 passed, 0 failed. The single-row confirmation artifact
`20260614T192643Z` also passed `array_map_variation2.phpt`.

Current broad 1k classify-only artifact `20260614T193952Z` selected 1,000
rows: 558 runnable and 442 classified. The runnable split was 172 Zend, 370
`ext/standard`, and 16 core rows. All 370 current `ext/standard` runnable rows
were `ext/standard/tests/array/*`; there were 0 current broad-1k
`ext/standard/tests/strings/*` rows.

Implemented behavior: `array_map(null, ...)` now preserves array element
references in the null-callback path instead of dereferencing through the normal
callback argument helper. For the single-array null-callback case, recursive
self-references to the source array are re-rooted to the result array so
`var_dump()` reports `*RECURSION*`; multi-array null-callback zips still retain
references to the original input arrays.

## 2026-06-14 ptn-lxw1 Array COW/Reference Row Pack

Post-rebase focused PHPT checks on the final squashed branch state:

- 9-row array COW/reference pack: 9 selected, 9 runnable, 9 passed, 0 failed
  (`.runtime/ptn-lxw1-nine-final/run-20260614T182113Z-manifest.log`).
- 2 candidate rows: 2 selected, 2 runnable, 2 passed, 0 failed
  (`.runtime/ptn-lxw1-two-candidates-final-amended/run-20260614T182307Z-manifest.log`).
- 20-row mixed control: 20 selected, 20 runnable, 19 passed, 1 failed
  (`array_map_variation2.phpt`;
  `.runtime/ptn-lxw1-final-pack-final-amended/run-20260614T182354Z-manifest.log`).

Implemented behavior: `array_push()` and `array_unshift()` support mutable
array-dimension paths with normal COW separation; temporary by-reference
`array_shift()` calls mutate an owned temporary instead of the source expression;
`array_push()` checks append-key overflow through the runtime; `(array)` object
casts dereference property references and closure/exception casts produce empty
arrays; and `array_merge_recursive()` separates referenced result entries before
recursive mutation.

Earlier broad 1k measurement on the pre-rebase base `abfb48341ef2` was
unchanged before/after this work: 440 runnable rows, 366 passed, 74 failed.
The dashboard keeps the newer `ptn-ouhx` broad 1k baseline instead of that
older broad count.

## 2026-06-14 ptn-xcmz Broad Property/Object Metadata Row Pack

Final manifest: `tools/phpt-ptn-xcmz-property-object-metadata-row-pack.txt`.

Current-base baseline `origin/master` (`df6e3157a19e`) selected the 19 broad
1k rows: 1 runnable, 0 passed, 1 failed, 18 classified
`unsupported-property-visibility-metadata`.

Current branch selected the same 19 rows: 12 runnable, 12 passed, 0 failed,
7 classified (`unsupported-magic-method-metadata` 2,
`unsupported-method-visibility-metadata` 2,
`unsupported-property-visibility-metadata` 2, `unsupported-internal` 1).
Focused command:
`PHPT_PROGRESS_DIR=.runtime/ptn-xcmz-row-pack-final tools/run-bounded-phpt.sh --classify-harness-programs tools/phpt-ptn-xcmz-property-object-metadata-row-pack.txt`.

Newly passing broad rows:

- `Zend/tests/bug26010.phpt`
- `Zend/tests/bug27798.phpt`
- `Zend/tests/bug35509.phpt`
- `ext/standard/tests/array/007.phpt`
- `ext/standard/tests/array/array_column_property_visibility.phpt`
- `ext/standard/tests/array/array_intersect_1.phpt`
- `ext/standard/tests/array/array_udiff_assoc_basic.phpt`
- `ext/standard/tests/array/array_udiff_basic.phpt`
- `ext/standard/tests/array/array_udiff_uassoc_basic.phpt`
- `ext/standard/tests/array/array_uintersect_assoc_basic.phpt`
- `ext/standard/tests/array/array_uintersect_basic.phpt`
- `ext/standard/tests/array/array_uintersect_uassoc_basic.phpt`

Implemented behavior: public/non-public instance property metadata remains
runnable while non-public static property metadata stays classified;
`array_column()` object property extraction now uses modeled `__isset()` plus
`__get()` for inaccessible properties; plain `get_object_vars()` exports
properties visible in the current class scope; direct `parent::method()` calls
inside instance methods dispatch through the existing scoped method helper.
Unbraced `$object->property` interpolation in double-quoted strings now parses
as a property fetch.

The rebased full broad 1k rerun was attempted with
`tools/run-phpt-baseline.sh --tier 1000 --out-dir .runtime/ptn-xcmz-broad-final`
at `20260614T183916Z`, but `run-bounded-phpt.sh` blocked in classifier
`pipe_read` after 400 rows and before PHPT execution. The dashboard
`1k-baseline` row remains the last completed broad measurement (`ptn-ouhx`,
419 passing).

## 2026-06-14 ptn-s8cn Broad Call-Unpacking Row Pack

Focused manifest: `tools/phpt-call-unpacking-current-ptn-ei36-manifest.txt`.

Before broad 1k classify-only artifact `20260614T180636Z` selected 1,000
rows: 472 runnable, 528 classified, including 22
`unsupported-call-unpacking` rows.

After broad 1k classify-only artifact `20260614T184955Z` selected the same
1,000-row tier shape on the rebased final branch: 545 runnable, 455 classified,
with the old blanket
`unsupported-call-unpacking` bucket removed and split into by-reference,
Traversable/generator, and resource-limit buckets.

Focused execution artifact `20260614T184726Z` selected 20 call-unpacking rows:
11 runnable, 11 passed, 0 failed, 9 classified
(`unsupported-generator-runtime` 3,
`unsupported-call-unpacking-reference` 3,
`unsupported-call-unpacking-traversable` 1,
`unsupported-resource-limit` 2).

Newly passing call-unpacking rows:

- `Zend/tests/arg_unpack/dynamic.phpt`
- `Zend/tests/arg_unpack/internal.phpt`
- `Zend/tests/arg_unpack/invalid_type.phpt`
- `Zend/tests/arg_unpack/many_args.phpt`
- `Zend/tests/arg_unpack/method.phpt`
- `Zend/tests/arg_unpack/new.phpt`
- `Zend/tests/arg_unpack/positional_arg_after_unpack_error.phpt`
- `ext/standard/tests/array/array_diff_uassoc_basic.phpt`
- `ext/standard/tests/array/array_find_types.phpt`
- `ext/standard/tests/array/array_intersect_uassoc_basic.phpt`
- `ext/standard/tests/array/array_push_empty.phpt`

Implemented behavior: parser/AST/IR/backend support for call-site argument
unpacking on direct, dynamic, method, static, constructor, and internal calls;
runtime `PtnCallArguments` expansion for ordered array operands; PHP-style
`TypeError` invalid-operand diagnostics with object class names; and parser
diagnostics for positional arguments after unpack. Remaining call-unpacking
blockers are by-reference parameter preservation through spread expansion,
Traversable/SPL/generator spread inputs, and max-array-size/resource-limit
stress diagnostics.

## 2026-06-14 ptn-55u0 Broad Array-Unpack Row Pack

Final manifest: `tools/phpt-ptn-55u0-array-unpack-row-pack-manifest.txt`.

Baseline `origin/master` (`a82844e88cf6`) selected 34 broad 1k unpack rows with classification disabled: 34 runnable, 2 passed, 32 failed. The two passes were unrelated ext/standard array callback rows in the same former bucket.

Current branch selected the same 34 rows with the refined classifier: 10 runnable, 10 passed, 0 failed, 24 classified separately (`unsupported-call-unpacking` 20, `unsupported-generator-runtime` 3, `unsupported-typed-property-metadata` 1).

Newly passing broad rows:

- `Zend/tests/array_unpack/already_occupied.phpt`
- `Zend/tests/array_unpack/gh19303.phpt`
- `Zend/tests/array_unpack/gh9769.phpt`
- `Zend/tests/array_unpack/in_destructuring.phpt`
- `Zend/tests/array_unpack/in_destructuring_2.phpt`
- `Zend/tests/array_unpack/ref1.phpt`
- `Zend/tests/array_unpack/undef_var.phpt`
- `Zend/tests/array_unpack/unpack_invalid_type_compile_time.phpt`
- `Zend/tests/array_unpack/unpack_string_keys_compile_time.phpt`
- `Zend/tests/array_unpack_string_keys.phpt`

Implemented behavior: parser/IR/backend support for array-literal unpack (`...`) in short and long array literals; classifier split that keeps call-site unpacking blocked while allowing array-literal unpack rows to run; array-literal spread runtime append semantics with integer-key reindexing, string-key preservation/overwrite, reference dereference on spread, append-overflow errors, runtime invalid-operand `Error` ordering, destructuring spread diagnostics, and constant-expression array-unpack errors.

## 2026-06-14 ptn-j6gv Broad String/Runtime Row Pack

Final manifest: `tools/phpt-ptn-j6gv-key-string-runtime-row-pack-manifest.txt`.

Baseline `origin/master` (`abfb48341ef2`) selected 25 broad 1k rows: 21 runnable, 15 passed, 6 failed, 4 classifier-excluded.
Current branch selected the same 25 rows: 25 runnable, 25 passed, 0 failed, 0 classifier-excluded.

Newly passing broad rows:

- `Zend/tests/arrow_functions/003.phpt`
- `Zend/tests/bug38211.phpt`
- `ext/standard/tests/array/array_column_scalar_index_strict_types.phpt`
- `ext/standard/tests/array/array_combine.phpt`
- `ext/standard/tests/array/array_diff_assoc_variation9.phpt`
- `ext/standard/tests/array/array_filter_invalid_mode.phpt`
- `ext/standard/tests/array/array_intersect_assoc_variation9.phpt`
- `ext/standard/tests/array/array_intersect_variation9.phpt`
- `ext/standard/tests/array/array_keys_variation_005.phpt`
- `ext/standard/tests/array/array_search_variation4.phpt`

Implemented behavior: strict-types `declare` propagation for internal scalar binding, directory resources for `opendir()`/`closedir()`, nested-array stringification warnings in array set operations, character endpoint handling in `range()`, read-only dynamic variable rows plus dynamic `unset()`, and named internal binding for `array_filter(..., mode:)`.

## 2026-06-14 ptn-qsmv.14

Broad 1k class/interface row pack on corpus revision
`8c63ec400ce8e07c57a8d9499317b96a8beafb8b` is recorded in
`docs/PHPT_BROAD_1K_CLASS_INTERFACE_ROW_PACK_PTN_QSMV14_2026-06-14.md`.

Final broad run after rebasing on `origin/master`: 1,000 selected, 459 runnable,
541 excluded, 379 passed, 80 failed. The class/interface declaration slice
contributes 13 newly green broad rows: ArrayAccess interface rows
`bug30346`/`bug69955`, abstract method/class rows, access modifier diagnostics
`access_modifiers_001` through `006`, `007`, `013`, and interface diagnostic
`bug32427`. The final rebased pass-set comparison against pre-work commit
`e6d9a2a86d8a` shows 61 newly passing rows total; 39 were old runtime failures
and 22 were old classifier exclusions.

Focused PHPT checks: raw interface/ArrayAccess pack is 6/38; classified
class-contract bucket is 9/9. Rust checks passed: `cargo fmt --check`,
targeted `compile_native` parser/runtime tests, and `cargo test --test
phpt_classifier`.

## 2026-06-14 ptn-tiqh COW/Reference Row Pack

Final manifest: `tools/phpt-cow-reference-mutation-ptn-tiqh-manifest.txt`.

The submitted branch reported the focused COW/reference mutation row pack as
21 selected, 21 passed. Its broad 1k run was measured on an older base
(`abfb48341ef2`) before `ptn-j6gv` and `ptn-qsmv.14` landed, so the current
dashboard keeps the newer `ptn-qsmv.14` broad 1k baseline row rather than
downgrading to that stale broad count.
