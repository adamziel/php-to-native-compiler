# PTN Progress

Refresh: 2026-06-14T16:19Z.
Measured: `ptn-c284`; `ptn-xymv`; `ptn-j8b8/b35n/18tp/gkvr`; `ptn-gt7b`; `ptn-30ji`; `ptn-h0qa` broad 1k classify-only 424 runnable / 576 classified; `ptn-dkcs` call-unpacking 34 selected / 0 runnable / 34 classified; `ptn-ei36` unpacking split 20 call / 14 array classified; `ptn-550s.12` broad 1k array-helper row pack 0/10 -> 10/10; `ptn-jsvb` COW/reference row pack 41/46 focused and exact broad 1k 329/424 -> 345/424 with 16 FAIL->PASS and 0 PASS->FAIL; COW 69/103 passed.

## Dashboard

|Source|Ported|Passing|Gap|
|---|---:|---:|---:|
|Units|3|3|0|
|Native|729|729|0|
|Bounded|486|479|7|
|Zend|119|119|0|
|ext/standard-PHPT|281|274|7|
|Array-key/cb|75|38|37|
|Array-cb-valid|66|49|17|
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
|Std-arrays|296|253|43|
|Map/filter|30|21|9|
|Request/SAPI|41|1|40|
|Anon-class|15|0|15|
|Interface-decl|23|0|23|
|Interface-impl|15|0|15|
|Trait-decl|25|0|25|
|Call-unpack|20|0|20|
|Array-unpack|14|0|14|
|Type-hint|14|0|14|
|Function-state|11|0|11|
|Dynamic-symbol|8|0|8|
|Generator-runtime|1|0|1|
|Internal-call-bind|1|0|1|
|Attribute-syntax|141|0|141|
|Attribute-internal|8|0|8|
|Attribute-meta|204|0|204|
|Heredoc-array|70|20|50|
|Std-array-row-pack|10|10|0|
|COW/ref-row-pack|46|41|5|
|1k-runnable-jsvb|424|345|79|
|Std-array-tdei|71|61|10|
|array_rand|7|6|1|
|Zend-op/control|26|15|11|
|Binary-key|1|1|0|
|Runtime-config|54|0|54|
|COW-gate|26|26|0|
|1k-baseline|1000|265|735|

## Recent Evidence

`ptn-jsvb` (2026-06-14): focused `tools/phpt-ptn-jsvb-cow-reference-row-pack.txt` ran 46 selected / 46 runnable / 41 passed / 5 failed. The exact broad 1k manifest from `.runtime/phpt-progress/manifest-20260614T140547Z.txt` improved from 329/424 to 345/424 (`.runtime/ptn-jsvb-after-1k/summary-20260614T151829Z.txt`) with these FAIL->PASS rows and no PASS->FAIL rows: `Zend/tests/array_append_reading_error.phpt`, `Zend/tests/array_literal_next_element_error.phpt`, `Zend/tests/array_merge_recursive_next_key_overflow.phpt`, `Zend/tests/assign_op_type_error.phpt`, `Zend/tests/assign_to_obj_002.phpt`, `ext/standard/tests/array/array_map_error.phpt`, `ext/standard/tests/array/array_map_variation10.phpt`, `ext/standard/tests/array/array_map_variation9.phpt`, `ext/standard/tests/array/array_push_error2.phpt`, `ext/standard/tests/array/array_reduce_variation1.phpt`, `ext/standard/tests/array/array_search_variation3.phpt`, `ext/standard/tests/array/array_udiff_assoc_variation5.phpt`, `ext/standard/tests/array/array_udiff_uassoc_variation6.phpt`, `ext/standard/tests/array/array_udiff_variation5.phpt`, `ext/standard/tests/array/array_uintersect_assoc_variation5.phpt`, `ext/standard/tests/array/array_uintersect_uassoc_variation6.phpt`.
