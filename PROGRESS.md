# PTN Progress

Refresh: 2026-06-14T23:03Z.
Measured: `ptn-c284`; `ptn-xymv`; `ptn-j8b8/b35n/18tp/gkvr`; `ptn-gt7b`; `ptn-30ji`; `ptn-h0qa` broad 1k classify-only 424 runnable / 576 classified; `ptn-dkcs` call-unpacking 34 selected / 0 runnable / 34 classified; `ptn-ei36` unpacking split 20 call / 14 array classified; `ptn-550s.12` broad 1k array-helper row pack 0/10 -> 10/10; `ptn-qsmv.14` class/interface row pack; `ptn-qsmv.16` array callback/map-filter row pack; `ptn-qsmv.17` assertion/runtime-config +10 broad rows and broad 1k classify-only 440/560; `ptn-s80e` broad 1k array/reference row pack 10/20 -> 20/20; `ptn-j6gv` broad 1k string/runtime row pack 15/25 -> 25/25; `ptn-55u0` broad 1k unpack row pack 2/34 raw baseline -> 10/10 runnable after split; `ptn-tiqh` COW/reference row pack 21/21 on submitted base; `ptn-ouhx` object-string array-helper row pack 0/34 -> 34/34, object-string source bucket 19/61 -> 53/61, broad 1k 285 -> 419 passing (501 runnable / 499 classified after, stitched from timed broad run plus remaining slice); `ptn-lxw1` array COW/reference row pack 9/9 focused, 2/2 candidates, 19/20 mixed control; `ptn-xcmz` broad 1k property/object metadata row pack 0/19 current-base focused baseline -> 12/12 runnable; `ptn-s8cn` call-unpacking row pack 0/20 classified -> 11/11 runnable passed and broad 1k classify-only 472/528 -> 545/455; `ptn-1d60` array_map null-reference row 65/66 -> 66/66 and broad 1k classify-only 558/442 with 370 current standard-array runnable rows and 0 standard-strings rows; `ptn-qg7b` asymmetric-visibility row pack 4/23 current-base focused baseline -> 14/23 final branch, with completed broad 1k run 463/533 before final master fast-forward; `ptn-g7ta` object-string array row pack +24 broad 1k rows and focused 23/23; `ptn-mqvk` broad 1k classify-only 546/454 before -> 532/468 after and broad COW/reference row pack 22/28 -> 27/27 runnable plus 1 classified; COW 69/103 passed; `ptn-601n` anonymous-class/object metadata row pack 0/21 raw focused baseline -> 10/10 runnable after split and final broad 1k classify-only 558/442; `ptn-c5ar` object/class metadata runtime diagnostics row pack 9 passed / 7 failed / 4 classified -> 20/20 passed and current-target broad 1k classify-only 532/468 -> 563/437; `ptn-zhup` object/callback/merge row pack 13/18 -> 17/18 runnable plus 2 classified, merge slice 21/25 -> 24/25, and final broad 1k classify-only 558/442 on submitted base; `ptn-buig` break/continue diagnostics moved the broad 1k Zend bucket 137/198 -> 141/198 and focused Zend operator/control frontier 21/26; `ptn-u4el` string runtime row pack 1/10 -> 10/10 and broad 1k 477/558 -> 479/558; `ptn-z096` user-comparator sort row pack 0/20 classified -> 20/20 passed, array-internal COW frontier 35 runnable / 30 passed / 37 classified -> 59 runnable / 50 passed / 13 classified, broad 1k classify-only unchanged at 563/437.

## Dashboard

|Source|Ported|Passing|Gap|
|---|---:|---:|---:|
|Units|3|3|0|
|Native|732|732|0|
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
|Array-COW|72|50|22|
|COW-foreach|103|69|34|
|Foreach-list|4|4|0|
|Ref-call|12|10|2|
|CUF-edges|12|8|4|
|Zend-assign|32|23|9|
|Recursive-dump|4|2|2|
|Classes|78|1|77|
|Zend-bugs|37|18|19|
|Class-name|10|9|1|
|Asym-vis|23|14|9|
|Dynamic-type|44|0|44|
|Diagnostics|47|0|47|
|Non-array-meta|74|0|74|
|Class/object-meta|221|0|221|
|Class/object-prop-pack|19|12|7|
|Class/object-c5ar|20|20|0|
|Class/object-meta-granular|135|0|135|
|Class-metadata-split|143|0|143|
|Core/basic-op|34|18|16|
|Runtime-INI|73|0|73|
|Resource-limit|1|0|1|
|Magic/object|69|20|49|
|Magic-methods|8|0|8|
|Object-string-meta|61|53|8|
|Object-string-array-pack|34|34|0|
|Object-callback-merge-zhup|20|17|3|
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
|Anon-class-pack|21|10|11|
|Std-array-row-pack|10|10|0|
|Std-array-s80e|20|20|0|
|Std-array-tdei|71|61|10|
|Std-array-lxw1|20|19|1|
|array_rand|7|6|1|
|Zend-op/control|26|21|5|
|Binary-key|1|1|0|
|Runtime-config|54|10|44|
|COW-gate|26|26|0|
|COW-reference-tiqh|21|21|0|
|COW-reference-mqvk|28|27|1|
|1k-baseline|1000|479|521|

## 2026-06-14 ptn-z096 User Comparator Sort COW Row Pack

Focused manifest:
`tools/phpt-ptn-z096-user-comparator-sort-row-pack-manifest.txt`.

Current-base array-internal frontier artifact `.runtime/ptn-z096-array-internal-current`
selected 72 rows: 35 runnable, 30 passed, 5 failed, and 37 classified. Its
`user-comparator-sort` bucket selected 26 rows with 0 runnable rows: 24 were
classified `unsupported-internal` for missing `usort()`/`uasort()`/`uksort()`
helpers and 2 remained typed-property metadata blockers.

Final focused artifact `.runtime/ptn-z096-user-comparator-sort-rebased` at
`20260614T225226Z` selected 20 rows: 20 runnable, 20 passed, 0 failed, and
0 classified. Focused command:
`PHPT_PROGRESS_DIR=.runtime/ptn-z096-user-comparator-sort-rebased tools/run-bounded-phpt.sh --classify-harness-programs tools/phpt-ptn-z096-user-comparator-sort-row-pack-manifest.txt`.

Array-internal frontier artifact `.runtime/ptn-z096-array-internal-final` at
`20260614T222726Z` selected 72 rows: 59 runnable, 50 passed, 9 failed, and
13 classified. The broader `user-comparator-sort` bucket selected 26 rows:
24 runnable, 20 passed, 4 failed, and 2 typed-property rows classified. This
frontier run was taken before the final `ptn-u4el` rebase; the focused
20-row pack above was rerun on the final rebased runtime.

Final broad 1k classify-only artifact
`.runtime/ptn-z096-broad-1k-rebased/manifest-20260614T225623Z.txt` selected
1,000 rows: 563 runnable and 437 classified. This was classification only and
did not change the current-target broad 1k top line; pass-count evidence for
this bead is the focused 20-row pack and array-internal frontier above.

Newly passing focused rows:

- `ext/standard/tests/array/sort/uasort_basic1.phpt`
- `ext/standard/tests/array/sort/uasort_basic2.phpt`
- `ext/standard/tests/array/sort/uasort_variation10.phpt`
- `ext/standard/tests/array/sort/uasort_variation11.phpt`
- `ext/standard/tests/array/sort/uasort_variation3.phpt`
- `ext/standard/tests/array/sort/uasort_variation4.phpt`
- `ext/standard/tests/array/sort/uasort_variation5.phpt`
- `ext/standard/tests/array/sort/uasort_variation6.phpt`
- `ext/standard/tests/array/sort/uasort_variation7.phpt`
- `ext/standard/tests/array/sort/uasort_variation8.phpt`
- `ext/standard/tests/array/sort/uksort_basic.phpt`
- `ext/standard/tests/array/sort/usort_basic.phpt`
- `ext/standard/tests/array/sort/usort_stability.phpt`
- `ext/standard/tests/array/sort/usort_variation10.phpt`
- `ext/standard/tests/array/sort/usort_variation3.phpt`
- `ext/standard/tests/array/sort/usort_variation4.phpt`
- `ext/standard/tests/array/sort/usort_variation5.phpt`
- `ext/standard/tests/array/sort/usort_variation7.phpt`
- `ext/standard/tests/array/sort/usort_variation8.phpt`
- `ext/standard/tests/array/sort/usort_variation9.phpt`

Implemented behavior: parser/classifier modeling for `usort()`, `uasort()`,
and `uksort()` as direct-variable by-reference array mutators; backend emission
for callback-carrying variable helpers; runtime support for stable user
comparator sorting with COW separation, `usort()` key reindexing, `uasort()`
key preservation, `uksort()` key comparison, callback validation, dynamic
internal calls, and `function_exists()`.

Remaining broad `user-comparator-sort` gaps are object comparator rows,
malformed boolean comparator deprecation behavior, one multidimensional
`usort()` variation, and typed-property object rows.

## 2026-06-14 ptn-u4el Broad String Runtime Row Pack

Focused manifest: `.runtime/ptn-u4el-string-warning-probe/manifest.txt`.

Focused command:

```sh
PHPT_PROGRESS_DIR=.runtime/ptn-u4el-string-warning-probe/after-final tools/run-bounded-phpt.sh --classify-harness-programs .runtime/ptn-u4el-string-warning-probe/manifest.txt
```

Focused evidence: 10 selected, 10 runnable. Baseline on the current branch before
the runtime changes was 1 passed / 9 failed
(`.runtime/ptn-u4el-string-warning-probe/current/summary-20260614T194052Z.txt`);
the post-rebase final run at `551c3d6308a3` passed all 10
(`.runtime/ptn-u4el-string-warning-probe/after-rebase/summary-20260614T223251Z.txt`).
The pre-rebase final run at `95cd3f6dd184` also passed all 10
(`.runtime/ptn-u4el-string-warning-probe/after-final/summary-20260614T214401Z.txt`).

Implemented behavior: array-to-string warnings now use PHP spacing for array
key conversion, `echo`/`print`, `implode()`/`join()`, `sprintf()`/`printf()` and
`str_replace()` paths; `strval()` is registered in sorted internal-function
order; `sprintf()`/`printf()`/`vprintf()` support positional `%n$` string
formats and custom `%'<char>` padding; `str_replace()` emits null deprecations
for array|string parameters; and `serialize()` covers scalar, array, resource
and basic object payloads.

Controlling broad 1k KPI used the deterministic tier-1000 manifest against the
task-start `origin/master` state:

- Before: `f66ce2e112cc`, 1000 selected, 558 runnable, 477 passed, 81 failed
  (`/tmp/ptn-u4el-before-guard463/.runtime/phpt-progress/summary-20260614T195924Z.txt`).
- After (pre-rebase broad run): `95cd3f6dd184`, 1000 selected, 558 runnable, 479 passed, 79 failed
  (`.runtime/phpt-progress/summary-20260614T205240Z.txt`).
- Newly passing broad rows:
  `ext/standard/tests/array/array_fill_keys_variation1.phpt`;
  `ext/standard/tests/array/array_intersect_assoc_variation7.phpt`.
- New broad failures: 0.

The focused string rows are not present in this deterministic broad 1k slice;
the broad gain is therefore +2 rows, below the nominal +10 target but above the
no-progress threshold.

## 2026-06-14 ptn-zhup Object/Callback/Merge Row Pack

Final manifest:
`tools/phpt-ptn-zhup-object-callback-merge-row-pack-manifest.txt`.

Focused before run on `origin/master` (`ef9c4a73667d`):
`PHPT_PROGRESS_DIR=/home/claude/gt/ptn_from_scratch/polecats/guard-453/ptn_from_scratch/.runtime/ptn-zhup-row-pack-before-origin-final /tmp/ptn-zhup-before-origin/tools/run-bounded-phpt.sh /home/claude/gt/ptn_from_scratch/polecats/guard-453/ptn_from_scratch/tools/phpt-ptn-zhup-object-callback-merge-row-pack-manifest.txt`.
Artifact `run-20260614T213516Z-manifest.log` selected 20 rows: 18
runnable, 13 passed, 5 failed, and 2 classified
(`unsupported-object-string-conversion-metadata`).

Final focused run on `896f6dde6132`:
`PHPT_PROGRESS_DIR=.runtime/ptn-zhup-row-pack-final-rebased tools/run-bounded-phpt.sh tools/phpt-ptn-zhup-object-callback-merge-row-pack-manifest.txt`.
Artifact `run-20260614T212217Z-manifest.log` selected 20 rows: 18
runnable, 17 passed, 1 failed, and 2 classified
(`unsupported-object-string-conversion-metadata`).

The same broad-1k merge slice was also measured because the implemented merge
signature change is generic across the helper family:
`PHPT_PROGRESS_DIR=.runtime/ptn-zhup-merge-slice-before /tmp/ptn-zhup-before-origin/tools/run-bounded-phpt.sh .runtime/ptn-zhup-merge-slice-manifest.txt`
selected 25 runnable rows and passed 21 before; the final branch command
`PHPT_PROGRESS_DIR=.runtime/ptn-zhup-merge-slice-after tools/run-bounded-phpt.sh .runtime/ptn-zhup-merge-slice-manifest.txt`
passed 24/25.

Final broad 1k classify-only artifact
`.runtime/ptn-zhup-broad1k-classify-final-rebased-progress/summary-20260614T212557Z.txt`
selected 1,000 rows: 558 runnable and 442 classified. This was
classification-only; broad pass-count evidence for this bead is the focused
row pack above.

Rows newly green over current `origin/master` in the focused evidence:

- `ext/standard/tests/array/array_column_variant_objects.phpt`
- `ext/standard/tests/array/array_merge_recursive_variation1.phpt`
- `ext/standard/tests/array/array_merge_recursive_variation2.phpt`
- `ext/standard/tests/array/array_merge_variation2.phpt`

Implemented behavior retained in the final branch: object operator parsing now
accepts braced literal member names such as `$object->{0}` for property fetches
and method-call syntax, and `array_merge()`/`array_merge_recursive()` variadic
array type errors omit parameter names to match PHP's reported signatures.
Native regression tests also pin object string-cast errors, array callback
validation, `array_map(null, ...)` reference preservation, numeric object
property extraction through `array_column()`, and merge type errors.

Remaining focused failure is
`ext/standard/tests/array/array_merge_variation1.phpt`, where the TypeError text
matches but an uncaught fatal does not yet render PHP's stack trace. The two
classified rows require object-to-string conversion through array comparator or
callback helper metadata outside the currently modeled object-string subset.

## 2026-06-14 ptn-buig Break/Continue Control Diagnostics

Focused command:
`tools/run-bounded-phpt.sh tools/phpt-zend-operator-control-frontier-manifest.txt`.

Broad 1k classification artifact
`.runtime/ptn-buig-broad-before/20260614T211916Z/phpt-baseline-1000.txt`
selected 1,000 rows: 558 runnable and 442 classified. The full broad
execution attempt used `timeout 1800 tools/run-phpt-baseline.sh --tier 1000
--out-dir .runtime/ptn-buig-broad-before`; it completed the broad Zend bucket
and reached standard test 6/344 before the timeout, so no completed full-broad
pass count was produced.

Completed broad Zend-bucket evidence used
`.runtime/phpt-progress/buckets-20260614T211916Z/zend.paths`, the 198 runnable
Zend rows selected by the broad 1k run. Before:
`.runtime/phpt-progress/run-20260614T211916Z-zend.log` had 137 passed and 61
failed. After the parser change, the same bucket had 141 passed and 57 failed.
This completed broad Zend-bucket after-run was taken before the final rebase;
the rebased source was checked with the focused frontier artifact below.

Newly passing broad rows:

- `Zend/tests/break_error_001.phpt`
- `Zend/tests/break_error_002.phpt`
- `Zend/tests/break_error_003.phpt`
- `Zend/tests/break_error_004.phpt`

After rebasing onto current `origin/master`, focused frontier artifact
`.runtime/ptn-buig-focused-rebased/run-20260614T221857Z-manifest.log` selected
26 rows: 26 runnable, 21 passed, and 5 failed. The remaining focused failures
are backtick/floating AST serialization, constant-expression `(unset)`
rejection, `#[NoDiscard]` native-method metadata, and binary literal
formatting.

Implemented behavior: the parser now rejects zero and non-literal operands for
`break` and `continue`, validates that transfer levels target an enclosing loop
or switch, reports PHP-style fatal diagnostics for out-of-context and excessive
levels, and walks nested statements, expressions, methods, functions, and
anonymous functions so each function-like body gets its own fresh control
context.

## 2026-06-14 ptn-601n Anonymous Class/Object Metadata Row Pack

Final manifest: `tools/phpt-ptn-601n-anonymous-class-row-pack.txt`.

Pre-change focused baseline captured on `f66ce2e11` selected these 21 rows as
`unsupported-anonymous-class`; raw execution selected 21 runnable rows, with
0 passed and 21 failed.

Current rebased branch command:
`PHPT_PROGRESS_DIR=.runtime/ptn-601n-anon-row-pack-final-af64 timeout 1200s tools/run-bounded-phpt.sh --classify-harness-programs tools/phpt-ptn-601n-anonymous-class-row-pack.txt`.
Artifact `run-20260614T204218Z-manifest.log` selected 21 rows: 10 runnable,
10 passed, 0 failed, and 11 classified (`unsupported-anonymous-class` 8,
`unsupported-attribute-syntax-metadata` 3).

Final broad 1k classify-only artifact
`.runtime/ptn-601n-broad-1k-final-af64-classify/manifest-20260614T204507Z.txt`
selected 1,000 rows: 558 runnable and 442 classified. This was classification
only; broad pass-count evidence for this bead is the focused row pack above.

Newly passing focused rows:

- `Zend/tests/ArrayAccess/bug78356.phpt`
- `Zend/tests/anon/001.phpt`
- `Zend/tests/anon/002.phpt`
- `Zend/tests/anon/003.phpt`
- `Zend/tests/anon/004.phpt`
- `Zend/tests/anon/005.phpt`
- `Zend/tests/anon/006.phpt`
- `Zend/tests/anon/007.phpt`
- `Zend/tests/anon/012.phpt`
- `Zend/tests/attributes/override/019.phpt`

Implemented behavior: parser/IR/backend support for anonymous class expressions
with constructor arguments, `extends`, `implements`, declared metadata, object
construction, and `instanceof`; runtime support for `rand()`; nested
function/method frames inherit diagnostic suppression; array-offset warnings use
runtime warning emission so suppression/error-reporting gates apply; and the
classifier now splits the supported anonymous subset from remaining anonymous
class and attribute metadata blockers.

Remaining blockers in this pack are dynamic static member access through
anonymous class objects, runtime `class_alias()` metadata for anonymous names,
`Closure::bind()` scope binding, PHP hidden-suffix anonymous class names in
`get_class()`/diagnostics, abstract anonymous diagnostics, and broader
attribute validation metadata.

## 2026-06-14 ptn-qg7b Asymmetric Visibility Row Pack

Final manifest: `tools/phpt-ptn-qg7b-asymmetric-visibility-row-pack-manifest.txt`.

Current-base focused baseline on `origin/master` (`f66ce2e11`) selected 23
broad 1k asymmetric-visibility rows: 23 runnable, 4 passed, 19 failed.

Final branch focused run at `1b47d79ad` selected the same 23 rows: 23 runnable,
14 passed, 9 failed (`.runtime/phpt-progress/run-20260614T200501Z.log`).

Newly passing broad rows:

- `Zend/tests/asymmetric_visibility/bug004.phpt`
- `Zend/tests/asymmetric_visibility/cpp_no_type.phpt`
- `Zend/tests/asymmetric_visibility/cpp_private.phpt`
- `Zend/tests/asymmetric_visibility/cpp_protected.phpt`
- `Zend/tests/asymmetric_visibility/cpp_wider_set_scope.phpt`
- `Zend/tests/asymmetric_visibility/decrease_scope_private_protected.phpt`
- `Zend/tests/asymmetric_visibility/duplicate_modifier.phpt`
- `Zend/tests/asymmetric_visibility/duplicate_modifier_2.phpt`
- `Zend/tests/asymmetric_visibility/no_type.phpt`
- `Zend/tests/asymmetric_visibility/override_protected_private.phpt`

Remaining row-pack failures are asymmetric visibility AST printing, unset,
reference access-mode behavior, unshared r/w cache slots, and virtual
get-only/set-only property diagnostics.

Completed broad 1k run before the final `origin/master` fast-forward selected
1,000 rows, 533 runnable, 467 classified, 463 passed, 70 failed
(`.runtime/phpt-progress/run-20260614T183321Z-{zend,standard,core}.log`).
After rebasing onto `02cbacb3e`, the focused row pack, `cargo fmt --check`, and
`cargo test asymmetric --test compile_native` were rerun and passed as above;
the full broad tier was not rerun after that final fast-forward, so the
dashboard keeps the last completed global broad baseline row.

Implemented behavior: constructor-promoted properties now preserve
asymmetric/read-only property metadata in the AST and class property table,
constructor bodies receive synthetic `$this->prop = $prop` assignments, parser
diagnostics match PHP for missing typed asymmetric properties and duplicate set
visibility modifiers, and inherited set-visibility overrides are validated.

## 2026-06-14 ptn-mqvk Broad COW/Reference Row Pack

Final manifest:
`tools/phpt-ptn-mqvk-broad-cow-reference-row-pack-manifest.txt`.

Broad 1k classify-only snapshots on corpus revision
`8c63ec400ce8e07c57a8d9499317b96a8beafb8b`: before selected 1000 rows,
546 runnable, 454 classified; after selected 1000 rows, 532 runnable,
468 classified. Before artifact:
`.runtime/ptn-mqvk-baseline-before/20260614T191343Z/phpt-baseline-1000.txt`;
after artifact:
`.runtime/ptn-mqvk-baseline-after-rebased/20260614T205229Z/phpt-baseline-1000.txt`.

Focused command:
`tools/run-bounded-phpt.sh tools/phpt-ptn-mqvk-broad-cow-reference-row-pack-manifest.txt`.
Before current work: 28 selected, 28 runnable, 22 passed, 6 failed
(`.runtime/phpt-progress/run-20260614T193255Z-manifest.log`). After current
work on the rebased branch: 28 selected, 27 runnable, 27 passed, 0 failed,
1 classified `unsupported-object-string-conversion-metadata`
(`ext/standard/tests/array/array_map_variation17.phpt`;
`.runtime/phpt-progress/run-20260614T204741Z-manifest.log`).

Newly passing broad rows:

- `Zend/tests/array_append_reading_error.phpt`
- `Zend/tests/asymmetric_visibility/object_reference.phpt`
- `Zend/tests/asymmetric_visibility/reference.phpt`
- `Zend/tests/asymmetric_visibility/reference_2.phpt`
- `ext/standard/tests/array/array_map_variation2.phpt`

Implemented behavior: array append read-as-value diagnostics now use the PHP
fatal-error path with source location; invalid callback operand messages match
array/string callback validation; asymmetric `private(set)` property references
throw indirect-modification diagnostics or return object copies as PHP does. The
array_map null-reference row is green on this final branch via the current
`ptn-1d60` base behavior.

## 2026-06-14 ptn-c5ar Object/Class Metadata Runtime Diagnostics

Focused manifest: `tools/phpt-ptn-c5ar-object-runtime-metadata-row-pack.txt`.

Current-base focused baseline, stitched from the pre-fix granular replay and
raw classified-bucket checks, selected the same 20 broad 1k rows: 16 runnable,
9 passed, 7 failed, and 4 classified (`unsupported-method-visibility-metadata`
2, `unsupported-magic-method-metadata` 2).

Final focused artifact `.runtime/ptn-c5ar-row-pack-final-current` at
`20260614T213444Z` selected 20 rows: 20 runnable, 20 passed, 0 failed, and
0 classified. Focused command:
`PHPT_PROGRESS_DIR=.runtime/ptn-c5ar-row-pack-final-current tools/run-bounded-phpt.sh --classify-harness-programs tools/phpt-ptn-c5ar-object-runtime-metadata-row-pack.txt`.

Final broad 1k classify-only artifact
`.runtime/ptn-c5ar-broad-classify-final-current-direct/manifest-20260614T213719Z.txt`
selected 1,000 rows: 563 runnable and 437 classified, moving from the current
target's recorded `ptn-mqvk` broad classify-only 532/468 baseline. This was
classification only; the focused row pack above is the pass-count evidence. An
earlier completed pre-`ptn-mqvk` broad classify-only artifact moved 558/442 to
562/438.

Newly passing broad rows:

- `Zend/tests/access_modifiers/access_modifiers_012.phpt`
- `Zend/tests/bug34260.phpt`
- `Zend/tests/bug34678.phpt`
- `Zend/tests/bug37811.phpt`
- `ext/standard/tests/array/array_fill_keys_variation1.phpt`
- `ext/standard/tests/array/array_map_object3.phpt`
- `ext/standard/tests/array/array_map_variation17.phpt`
- `ext/standard/tests/array/array_merge_recursive_variation1.phpt`
- `ext/standard/tests/array/array_merge_recursive_variation2.phpt`
- `ext/standard/tests/array/array_merge_variation1.phpt`
- `ext/standard/tests/array/array_merge_variation2.phpt`

Implemented behavior: object string conversion now throws catchable PHP-style
`Error` for non-stringable objects while preserving modeled `__toString()`;
uncaught internal exceptions use active call frames for PHP-style stack traces;
callback argument diagnostics now distinguish scalar/object/resource callables
from malformed array callables; variadic `array_merge()` and
`array_merge_recursive()` array TypeErrors omit synthetic parameter names; array
key coercion warnings use the existing spaced warning form; and the classifier
keeps already-modeled `__call`/class-method callback rows runnable instead of
leaving them in metadata buckets.

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

## 2026-06-14 ptn-g7ta Object/String Array Row Pack

Final manifest: `tools/phpt-object-string-array-conversion-ptn-g7ta-manifest.txt`.

Broad 1k before (`a82844e88cf6`, stamp `20260614T172503Z`): 1,000
selected, 462 runnable, 538 classified out, 389 passed, 73 failed. The
object-string conversion classifier bucket held 62 rows.

Broad 1k after (`cab98ce6ba9e`, stamp `20260614T184356Z`): 1,000
selected, 497 runnable, 503 classified out, 413 passed, 84 failed. The
object-string conversion classifier bucket dropped to 26 rows. Exact broad
newly passing rows:

- `Zend/tests/bug37811.phpt`
- `ext/standard/tests/array/array_column_basic.phpt`
- `ext/standard/tests/array/array_column_object_cast.phpt`
- `ext/standard/tests/array/array_combine_variation5.phpt`
- `ext/standard/tests/array/array_diff_assoc_variation3.phpt`
- `ext/standard/tests/array/array_diff_variation8.phpt`
- `ext/standard/tests/array/array_fill_keys_variation1.phpt`
- `ext/standard/tests/array/array_fill_keys_variation2.phpt`
- `ext/standard/tests/array/array_fill_keys_variation4.phpt`
- `ext/standard/tests/array/array_fill_variation3.phpt`
- `ext/standard/tests/array/array_flip_variation4.phpt`
- `ext/standard/tests/array/array_intersect_assoc_variation7.phpt`
- `ext/standard/tests/array/array_intersect_assoc_variation8.phpt`
- `ext/standard/tests/array/array_intersect_variation7.phpt`
- `ext/standard/tests/array/array_intersect_variation8.phpt`
- `ext/standard/tests/array/array_key_exists_variation1.phpt`
- `ext/standard/tests/array/array_key_exists_variation8.phpt`
- `ext/standard/tests/array/array_merge_recursive_variation5.phpt`
- `ext/standard/tests/array/array_merge_variation3.phpt`
- `ext/standard/tests/array/array_pad_variation3.phpt`
- `ext/standard/tests/array/array_push_variation2.phpt`
- `ext/standard/tests/array/array_reverse_variation3.phpt`
- `ext/standard/tests/array/array_reverse_variation5.phpt`
- `ext/standard/tests/array/array_shift_variation2.phpt`

Focused PHPT command:
`PHPT_PROGRESS_DIR=.runtime/ptn-g7ta-focused-progress-rebased tools/run-bounded-phpt.sh --classify-harness-programs tools/phpt-object-string-array-conversion-ptn-g7ta-manifest.txt`
selected 23 rows, 23 runnable, 23 passed.

Implemented behavior: runtime-aware object `__toString()` conversion for
array-column keys, array key values, array set-operation string comparisons,
and `(string)` object casts that throw catchable `Error` for objects without
public `__toString()`. Array key conversion now emits the same spaced
array-to-string warning form used by set operations.

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
