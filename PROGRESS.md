# PTN Progress

Refresh: 2026-06-14T16:55Z.
Measured: `ptn-c284`; `ptn-xymv`; `ptn-j8b8/b35n/18tp/gkvr`; `ptn-gt7b`; `ptn-30ji`; `ptn-h0qa` broad 1k classify-only 424 runnable / 576 classified; `ptn-dkcs` call-unpacking 34 selected / 0 runnable / 34 classified; `ptn-ei36` unpacking split 20 call / 14 array classified; `ptn-550s.12` broad 1k array-helper row pack 0/10 -> 10/10; `ptn-qsmv.14` class/interface row pack; `ptn-qsmv.16` array callback/map-filter row pack; `ptn-qsmv.17` assertion/runtime-config +10 broad rows and broad 1k classify-only 440/560; `ptn-s80e` broad 1k array/reference row pack 10/20 -> 20/20; `ptn-j6gv` broad 1k string/runtime row pack 15/25 -> 25/25; `ptn-tiqh` COW/reference row pack 21/21 on submitted base; COW 69/103 passed.

## Dashboard

|Source|Ported|Passing|Gap|
|---|---:|---:|---:|
|Units|3|3|0|
|Native|730|730|0|
|Bounded|486|479|7|
|Zend|119|119|0|
|ext/standard-PHPT|281|274|7|
|Array-key/cb|75|38|37|
|Array-cb-valid|66|64|2|
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
|Class/object-meta-granular|135|0|135|
|Class-metadata-split|143|0|143|
|Core/basic-op|34|18|16|
|Runtime-INI|73|0|73|
|Resource-limit|1|0|1|
|Magic/object|69|20|49|
|Magic-methods|8|0|8|
|Object-string-meta|61|0|61|
|Std-array-map|297|0|297|
|Std-arrays|296|263|33|
|Map/filter|30|25|5|
|Request/SAPI|41|1|40|
|Anon-class|15|0|15|
|Interface-decl|23|0|23|
|Interface-impl|15|0|15|
|Trait-decl|25|0|25|
|Call-unpack|20|0|20|
|Array-unpack|14|0|14|
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
|array_rand|7|6|1|
|Zend-op/control|26|15|11|
|Binary-key|1|1|0|
|Runtime-config|54|10|44|
|COW-gate|26|26|0|
|COW-reference-tiqh|21|21|0|
|1k-baseline|1000|379|621|

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
