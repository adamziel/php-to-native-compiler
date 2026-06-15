# PTN Progress

| Task | Ported tests | Passed tests |
| --- | ---: | ---: |
| ptn-w17z.31 iterable/DNF type row pack | 27 | 27 |
| ptn-w17z.36 extract typed references | 1 | 1 |
| ptn-2cij ENV/CLEAN SKIPIF preconditions | 33 | 0 |
| ptn-vj4r class_alias namespace metadata | 5 | 5 |
| ptn-144g parser residual row pack | 2 | 2 |
| ptn-evvk stream mode diagnostics | 8 | 8 |
| ptn-w17z.26 Serializable/SPL unserialize row pack | 5 | 5 |
| ptn-w17z.23 union type declaration parser row | 1 | 1 |
| ptn-utht filtered stream copy row pack | 4 | 4 |
| ptn-vqg1 temp stream wrappers row pack | 3 | 3 |
| ptn-w17z.30 foreach mutation visibility row pack | 11 | 7 |
| ptn-w17z.2 sort flags row pack | 78 | 72 |
| ptn-w17z.22 compile-time include path variables | 1 | 1 |
| ptn-w17z.28 collected Generator runtime row pack | 6 | 6 |
| ptn-w17z.34 output-buffer compact row | 1 | 1 |
| ptn-w17z.35 compact closure this row pack | 2 | 2 |
| ptn-w17z.33 extract globals refs row pack | 3 | 3 |
| ptn-w17z.25 serialize/unserialize reference identity row pack | 6 | 6 |
| ptn-w17z.15 iterator/generator row pack | 36 | 36 |
| ptn-w17z.12 magic methods/lifecycle row pack | 49 | 49 |
| ptn-w17z.10 SKIPIF harness row pack | 185 | 3 |
| ptn-w17z.14 Override attribute row pack | 34 | 34 |
| ptn-w17z.16 exception/backtrace/lifecycle row pack | 41 | 41 |
| ptn-w17z.18 numeric/math/conversions row pack | 35 | 35 |
| ptn-w17z.16.3 exception formatting row pack | 12 | 12 |
| ptn-kia6 by-ref/reference-boundary row pack | 107 | 75 |

Refresh: 2026-06-15T19:10Z.
Measured latest: `ptn-w17z.31` iterable/DNF type row pack selected 27 rows, kept 27 runnable, and passed 27/27 in the merged focused run.
Latest: `ptn-w17z.31` completed the checked-in iterable/DNF type row pack. Previous: `ptn-w17z.36` completed the checked-in extract typed references row pack.

Current hook: `ptn-w17z.31` iterable/DNF type row pack passed 27/27 at
`.runtime/ptn-w17z31-final-integrated/run-20260615T190737Z-manifest.log`.
Previous hook: `ptn-w17z.36` extract typed references passed 1/1 at
`.runtime/merge-ptn-w17z36-extract-typed-ref/summary-20260615T185434Z.txt`.

## 2026-06-15 ptn-w17z.31 Iterable/DNF Type Row Pack

Final manifest: `tools/phpt-ptn-w17z31-iterable-dnf-row-pack.txt`.

| Evidence | Ported tests | Passed tests |
| --- | ---: | ---: |
| Hook-start focused baseline (`.runtime/ptn-w17z31-before/run-20260615T175811Z-manifest.log`) | 27 selected / 27 runnable | 1 |
| Final focused run (`.runtime/ptn-w17z31-final-focused-27-pass/run-20260615T183817Z-manifest.log`) | 27 selected / 27 runnable | 27 |
| Merged focused run (`.runtime/ptn-w17z31-merged-focused/run-20260615T185932Z-manifest.log`) | 27 selected / 27 runnable | 27 |
| Final integrated focused run (`.runtime/ptn-w17z31-final-integrated/run-20260615T190737Z-manifest.log`) | 27 selected / 27 runnable | 27 |

Implemented behavior: parser/IR/runtime support for `object`, `iterable`,
union, intersection, and DNF user type hints; method signature compatibility
for iterable, object, and DNF variance; iterable union/intersection redundancy
diagnostics; declaration-time default diagnostics for iterable defaults; and
runtime checks/errors for composite parameter and return type boundaries.

## 2026-06-15 ptn-w17z.36 Extract Typed References

| Evidence | Ported tests | Passed tests |
| --- | ---: | ---: |
| Final rebased focused acceptance row (`.runtime/ptn-w17z36-extract-typed-ref-postrebase/run-20260615T173320Z-manifest.log`) | 1 selected / 1 runnable | 1 |
| Integrated focused merge run (`.runtime/merge-ptn-w17z36-extract-typed-ref/summary-20260615T185434Z.txt`) | 1 selected / 1 runnable | 1 |

## 2026-06-15 ptn-2cij ENV/CLEAN SKIPIF Preconditions

| Evidence | Ported tests | Passed tests |
| --- | ---: | ---: |
| Current `origin/master` classify-only (`.runtime/ptn-2cij/origin-baseline-rebased/summary-20260615T172331Z.txt`) | 2,086 selected / 255 runnable / 50 `harness-skipif` | 0 |
| Final rebased classify-only (`.runtime/ptn-2cij/env-clean-rebased/summary-20260615T171427Z.txt`) | 2,086 selected / 288 runnable / 17 `harness-skipif` | 0 |

## 2026-06-15 ptn-vj4r Class Alias Namespace Metadata

| Evidence | Ported tests | Passed tests |
| --- | ---: | ---: |
| Final rebased focused row pack (`.runtime/ptn-vj4r-class-alias-metadata-row-pack-rebased2/summary-20260615T172044Z.txt`) | 5 selected / 5 runnable | 5 |
| Integrated focused merge run (`.runtime/merge-ptn-vj4r-class-alias-metadata-row-pack/summary-20260615T183720Z.txt`) | 5 selected / 5 runnable | 5 |

## 2026-06-15 ptn-144g Parser Residual Row Pack

| Evidence | Ported tests | Passed tests |
| --- | ---: | ---: |
| Final rebased focused parser residuals (`.runtime/ptn-144g-after-rebase/summary-20260615T170654Z.txt`) | 2 selected / 2 runnable | 2 |
| Integrated focused merge run (`.runtime/merge-ptn-144g-parser-residuals/summary-20260615T182723Z.txt`) | 2 selected / 2 runnable | 2 |

## 2026-06-15 ptn-evvk Stream Mode Diagnostics

| Evidence | Ported tests | Passed tests |
| --- | ---: | ---: |
| Current master focused baseline (`.runtime/ptn-evvk-stream-mode-before/summary-20260615T163232Z.txt`) | 8 selected / 8 runnable | 7 |
| Final rebased focused run (`.runtime/ptn-evvk-stream-mode-rebased/summary-20260615T164445Z.txt`) | 8 selected / 8 runnable | 8 |
| Integrated focused merge run (`.runtime/merge-ptn-evvk-stream-mode-diagnostics/summary-20260615T181618Z.txt`) | 8 selected / 8 runnable | 8 |

## 2026-06-15 ptn-w17z.26 Serializable/SPL Unserialize Row Pack

| Evidence | Ported tests | Passed tests |
| --- | ---: | ---: |
| Hook-start focused baseline (`.runtime/ptn-w17z26-before/summary-20260615T155843Z.txt`) | 5 selected / 5 runnable | 0 |
| Final rebased branch focused (`.runtime/ptn-w17z26-final2/summary-20260615T164138Z.txt`) | 5 selected / 5 runnable | 5 |
| Integrated focused merge run (`.runtime/merge-ptn-w17z26-serializable-spl-unserialize/summary-20260615T180738Z.txt`) | 5 selected / 5 runnable | 5 |

## 2026-06-15 ptn-w17z.23 Union Type Declaration Parser Row

| Evidence | Ported tests | Passed tests |
| --- | ---: | ---: |
| Focused `fpow.phpt` run (`.runtime/ptn-w17z23-fpow/summary-20260615T164026Z.txt`) | 1 selected / 1 runnable | 1 |
| Integrated focused merge run (`.runtime/merge-ptn-w17z23-union-type-fpow/summary-20260615T180055Z.txt`) | 1 selected / 1 runnable | 1 |

## 2026-06-15 ptn-utht Filtered Stream Copy Row Pack

| Evidence | Ported tests | Passed tests |
| --- | ---: | ---: |
| Current master focused baseline (`/tmp/ptn-utht-before/.runtime/ptn-utht-filtered-stream-copy-before/summary-20260615T162046Z.txt`) | 4 selected / 4 runnable / 0 classified | 0 |
| Final rebased branch focused (`.runtime/ptn-utht-filtered-stream-copy-rebased/summary-20260615T162747Z.txt`) | 4 selected / 4 runnable / 0 classified | 4 |
| Integrated focused merge run (`.runtime/merge-ptn-utht-filtered-stream-copy/summary-20260615T175250Z.txt`) | 4 selected / 4 runnable | 4 |

## 2026-06-15 ptn-vqg1 Temp Stream Wrappers Row Pack

| Evidence | Ported tests | Passed tests |
| --- | ---: | ---: |
| Hook-start focused baseline (`.runtime/ptn-vqg1-temp-streams-before/run-20260615T160452Z-manifest.log`) | 3 selected / 3 runnable | 0 |
| Final rebased focused run (`.runtime/ptn-vqg1-temp-streams-rebased/run-20260615T162130Z-manifest.log`) | 3 selected / 3 runnable | 3 |
| Integrated focused merge run (`.runtime/merge-ptn-vqg1-temp-streams/summary-20260615T171433Z.txt`) | 3 selected / 3 runnable | 3 |

## 2026-06-15 ptn-w17z.30 Foreach Mutation Visibility Row Pack

| Evidence | Ported tests | Passed tests |
| --- | ---: | ---: |
| Hook-start focused baseline (`.runtime/ptn-w17z30-before-focused/summary-20260615T154026Z.txt`) | 10 selected / 10 runnable / 0 classified | 0 |
| Final rebased focused run (`.runtime/ptn-w17z30-after-rebase/summary-20260615T161455Z.txt`) | 11 selected / 7 runnable / 4 classified | 7 |
| Integrated focused merge run (`.runtime/merge-ptn-w17z30-foreach-mutation/summary-20260615T170725Z.txt`) | 11 selected / 7 runnable / 4 classified | 7 |

## 2026-06-15 ptn-w17z.2 Sort Flags Row Pack

| Evidence | Ported tests | Passed tests |
| --- | ---: | ---: |
| Integrated focused row pack (`.runtime/merge-ptn-w17z2-sort-flags/summary-20260615T164329Z.txt`) | 78 selected / 78 runnable | 72 |

## 2026-06-15 ptn-w17z.22 Compile-Time Include Path Variables

| Evidence | Ported tests | Passed tests |
| --- | ---: | ---: |
| Final rebased focused acceptance row (`.runtime/ptn-w17z22-acceptance-postrebase/summary-20260615T161501Z.txt`) | 1 selected / 1 runnable | 1 |

## 2026-06-15 ptn-w17z.28 Collected Generator Runtime Row Pack

| Evidence | Ported tests | Passed tests |
| --- | ---: | ---: |
| Final rebased branch focused (`.runtime/ptn-w17z28-generator-row-pack-final-head2/run-20260615T160837Z-manifest.log`) | 6 selected / 6 runnable | 6 |
| Integrated focused merge run (`.runtime/merge-ptn-w17z28-generator-runtime/summary-20260615T163051Z.txt`) | 6 selected / 6 runnable | 6 |
| Final broad frontier (`.runtime/ptn-w17z28-broad-rebased/run-20260615T155020Z-manifest.log`) | 46 selected / 39 runnable / 7 classified | 37 |

## 2026-06-15 ptn-w17z.34 Output Buffer Compact Row

| Evidence | Ported tests | Passed tests |
| --- | ---: | ---: |
| Final rebased branch focused (`.runtime/ptn-w17z34-output-buffer-compact-rebased/summary-20260615T154737Z.txt`) | 1 selected / 1 runnable | 1 |
| Integrated focused merge run (`.runtime/merge-ptn-w17z34-output-buffer-compact/summary-20260615T162323Z.txt`) | 1 selected / 1 runnable | 1 |

## 2026-06-15 ptn-w17z.35 Compact Closure This Row Pack

| Evidence | Ported tests | Passed tests |
| --- | ---: | ---: |
| Final rebased branch focused (`.runtime/ptn-w17z35-compact-closure-this-postrebase/summary-20260615T153730Z.txt`) | 2 selected / 2 runnable | 2 |
| Integrated focused merge run (`.runtime/merge-ptn-w17z35-compact-closure-this/summary-20260615T160939Z.txt`) | 2 selected / 2 runnable | 2 |

## 2026-06-15 ptn-w17z.16 Exception Lifecycle Row Pack

| Evidence | Ported tests | Passed tests |
| --- | ---: | ---: |
| Current master focused baseline (`/tmp/ptn-w17z16-before-final/.runtime/ptn-w17z16-row-pack-before-final41/summary-20260615T134106Z.txt`) | 41 selected / 40 runnable / 1 classified | 23 |
| Final rebased branch focused (`.runtime/ptn-w17z16-row-pack-after-final41-skipifbase/summary-20260615T144139Z.txt`) | 41 selected / 41 runnable / 0 classified | 41 |
| Integrated focused merge run (`.runtime/merge-ptn-w17z16-exception-lifecycle/summary-20260615T151015Z.txt`) | 41 selected / 41 runnable / 0 classified | 41 |

## 2026-06-15 ptn-w17z.25 Serialize/Unserialize Reference Identity Row Pack

| Evidence | Ported tests | Passed tests |
| --- | ---: | ---: |
| Final rebased branch focused (`.runtime/ptn-w17z25-after-rebase/summary-20260615T145920Z.txt`) | 6 selected / 6 runnable | 6 |
| Integrated focused merge run (`.runtime/merge-ptn-w17z25-serialize-reference-identity/summary-20260615T154738Z.txt`) | 6 selected / 6 runnable | 6 |

## 2026-06-15 ptn-w17z.33 Extract Globals Refs Row Pack

| Evidence | Ported tests | Passed tests |
| --- | ---: | ---: |
| Final rebased branch focused (`.runtime/ptn-w17z33-extract-globals-refs-rebased/summary-20260615T153503Z.txt`) | 3 selected / 3 runnable | 3 |
| Integrated focused merge run (`.runtime/merge-ptn-w17z33-extract-globals-refs/summary-20260615T155503Z.txt`) | 3 selected / 3 runnable | 3 |

## 2026-06-15 ptn-w17z.16.3 Exception Formatting Row Pack

| Evidence | Ported tests | Passed tests |
| --- | ---: | ---: |
| Final rebased branch focused (`.runtime/ptn-w17z16-3-exception-format-after-rebase/summary-20260615T153535Z.txt`) | 13 selected / 12 runnable / 1 classified | 12 |
| Integrated focused merge run (`.runtime/merge-ptn-w17z16-3-exception-format/summary-20260615T160318Z.txt`) | 13 selected / 12 runnable / 1 classified | 12 |

## 2026-06-15 ptn-w17z.32 SORT_REGULAR Mixed Ordering Row Pack

| Evidence | Ported tests | Passed tests |
| --- | ---: | ---: |
| Baseline focused residual (`.runtime/ptn-w17z32-before/summary-20260615T144637Z.txt`) | 6 selected / 6 runnable | 0 |
| Final residual slice (`.runtime/ptn-w17z32-after-string-warning-residuals/summary-20260615T151223Z.txt`) | 6 selected / 6 runnable | 6 |
| Object sort slice (`.runtime/ptn-w17z32-object-sort-after/summary-20260615T153243Z.txt`) | 8 selected / 8 runnable | 8 |
| Final focused row pack after rebase (`.runtime/ptn-w17z32-sort-pack-rebased/summary-20260615T160348Z.txt`) | 78 selected / 78 runnable | 78 |
| Integrated focused merge run (`.runtime/merge-ptn-w17z32-sort-regular/summary-20260615T172346Z.txt`) | 78 selected / 78 runnable | 78 |

## 2026-06-15 ptn-kia6 By-Reference Boundary Row Pack

Final checked-in focused manifest:
`tools/phpt-ptn-kia6-by-ref-reference-boundary-pack.txt`.

Source bucket: full php-src PHPT corpus revision
`8c63ec400ce8e07c57a8d9499317b96a8beafb8b`, selected from
COW/reference/refcount diagnostics, by-reference returns, pass-by-reference
errors, reference list unpacking, and adjacent array/object/function-boundary
reference rows. The final manifest comments record bucket
`by-ref-reference-boundary`.

Implemented behavior: by-reference returns now materialize non-lvalue return
values as temporary references while preserving live lvalue/call-result
references, including bare and implicit returns from by-reference functions
with PHP-compatible notices and the `void` deprecation. Userland by-reference
argument lowering now preserves return-reference and assignment-reference
sources, reports hard errors for declared by-reference parameters, formats
variadic argument errors without synthetic parameter names, and rejects bare
`$GLOBALS` as a by-reference argument. Reference assignment now returns the
established reference so nested argument assignments can pass the live slot.
Nested list reference assignment now binds through array-dimension references,
`$GLOBALS[...]` reference paths resolve through the real global table, `$this`
reference rebinding reports the PHP fatal error, and the
"Only variables should be assigned by reference" notice now uses the real
source path and line.

Focused evidence:
`PHPT_PROGRESS_DIR=.runtime/ptn-kia6-final-pack-rebased timeout 1800s tools/run-bounded-phpt.sh --classify-harness-programs tools/phpt-ptn-kia6-by-ref-reference-boundary-pack.txt`.
Artifact `.runtime/ptn-kia6-final-pack-rebased/summary-20260615T123313Z.txt`
selected 126 rows: 107 runnable, 75 passed, 32 failed, and 19 classified.

Integrated verification after merge conflict resolution:
`.runtime/merge-ptn-kia6-by-ref-boundary-after-metadata-gate/summary-20260615T134443Z.txt`
selected 126 rows, ran 107 after classification, and passed 75 with 32
expected pack failures and 19 classified rows.

Before evidence on pre-task commit `5bb8cc8a3d0b` used the same full-corpus
source bucket before the final two extra rows were added:
`/tmp/ptn-kia6-before/.runtime/ptn-kia6-union-before/summary-20260615T114002Z.txt`
selected 124 rows, with 105 runnable, 45 passed, 60 failed, and 19
classified. The two final extra rows were both red on the pre-task checkout in
`/tmp/ptn-kia6-before/.runtime/ptn-kia6-return-extra-before/summary-20260615T120011Z.txt`.
Normalized to the final 107 runnable rows, the pre-task baseline was 45
passed and 62 failed; the final run was 75 passed and 32 failed, with no
before-pass rows turning red.

Newly passing rows:

- `Zend/tests/bug31525.phpt`
- `Zend/tests/bug39944.phpt`
- `Zend/tests/bug72038.phpt`
- `Zend/tests/dereference/dereference_009.phpt`
- `Zend/tests/errmsg/errmsg_003.phpt`
- `Zend/tests/errmsg/errmsg_022.phpt`
- `Zend/tests/gh16515.phpt`
- `Zend/tests/list/bug73663_2.phpt`
- `Zend/tests/list/list_reference_001.phpt`
- `Zend/tests/match/027.phpt`
- `Zend/tests/match/028.phpt`
- `Zend/tests/named_params/cannot_pass_by_ref.phpt`
- `Zend/tests/restrict_globals/invalid_pass_by_ref.phpt`
- `Zend/tests/return_by_ref_from_void_function.phpt`
- `Zend/tests/return_ref_none.phpt`
- `Zend/tests/variadic/by_ref_error.phpt`
- `ext/standard/tests/array/bug31158.phpt`
- `tests/classes/constants_error_003.phpt`
- `tests/lang/bug20175.phpt`
- `tests/lang/bug21600.phpt`
- `tests/lang/passByReference_002.phpt`
- `tests/lang/passByReference_007.phpt`
- `tests/lang/passByReference_010.phpt`
- `tests/lang/returnByReference.003.phpt`
- `tests/lang/returnByReference.004.phpt`
- `tests/lang/returnByReference.005.phpt`
- `tests/lang/returnByReference.006.phpt`
- `tests/lang/returnByReference.007.phpt`
- `tests/lang/returnByReference.008.phpt`
- `tests/lang/returnByReference.009.phpt`

## 2026-06-15 ptn-w17z.14 Override Attribute Row Pack

| Item | Evidence |
| --- | --- |
| Final checked-in newly passing manifest | `tools/phpt-ptn-w17z14-override-attribute-row-pack.txt` |
| Hook-start focused source bucket | `.runtime/ptn-w17z14-override-current/summary-20260615T122622Z.txt`: 50 selected, 27 runnable, 23 classified; 13 passed, 14 failed |
| Final focused source bucket | `.runtime/ptn-w17z14-override-after-rebase2/summary-20260615T133232Z.txt`: 50 selected, 47 runnable, 3 classified; 47 passed, 0 failed |
| Checked-in row-pack run | `.runtime/ptn-w17z14-row-pack-after-rebase2/summary-20260615T133103Z.txt`: 34 selected, 34 runnable, 34 passed, 0 failed |
| Remaining source-bucket exclusions | `Zend/tests/attributes/override/002.phpt` is still generator/yield-from runtime; `014.phpt` and `015.phpt` are still enum metadata |
| Full-corpus 20k-family inventory | `.runtime/ptn-w17z14-full-after/20260615T130233Z/`: generated 21,867-row full inventory and 1k/5k/10k/20k/all manifests; `phpt-full-corpus-20000.txt` contains 20,156 deterministic rows and all 34 checked-in row-pack rows |
| Full-corpus 20k classify-only attempt | `timeout 900 tools/run-phpt-baseline.sh --scope full --tier 20000 --classify-only` timed out after generating the inventory/manifests and before writing a classification summary |

| Implemented behavior | Notes |
| --- | --- |
| Attribute lexing/parsing | `#[...]` now lexes as attribute syntax instead of a `#` comment; attribute groups are consumed on declarations/statements/parameters without adding reflection metadata |
| Override methods | `#[Override]` methods validate against visible parent methods, implemented/extended interface methods, and abstract methods from used traits; private parent methods and concrete trait methods do not satisfy Override |
| Override constructors | Concrete parent `__construct()` does not satisfy Override; abstract parent constructors do |
| Override properties | Instance/static properties validate against visible parent properties and interface properties; imported trait properties validate in the using class context |
| Interface property hooks | Minimal hook-block parsing supports interface property declarations such as `public mixed $i { get; }` for contract metadata |
| Constructor promotion | Promoted properties carry the Override bit into the lowered class property metadata |
| Classifier split | Override-validation rows with typed properties or constructor promotion are runnable; non-Override typed-property/promotion metadata rows remain classified |
| Reflection metadata emission | Generated ReflectionClass-returning metadata helpers are emitted only when the optional internal-function dispatch runtime chunk is present |

| Verification | Result |
| --- | --- |
| `cargo fmt --check` | passed |
| `cargo test override --test compile_native -- --nocapture` | passed 4/4 |
| `cargo test phpt_classifier_splits_attribute_metadata_blockers --test phpt_classifier -- --nocapture` | passed 1/1 |
| `PHPT_PROGRESS_DIR=.runtime/ptn-w17z14-row-pack-after-rebase2 tools/run-bounded-phpt.sh --classify-harness-programs tools/phpt-ptn-w17z14-override-attribute-row-pack.txt` | passed 34/34 |
| `PHPT_PROGRESS_DIR=.runtime/merge-ptn-w17z14-override-row-pack timeout 900s tools/run-bounded-phpt.sh --classify-harness-programs tools/phpt-ptn-w17z14-override-attribute-row-pack.txt` | passed 34/34 |
| `PHPT_PROGRESS_DIR=.runtime/ptn-w17z14-override-after-rebase2 tools/run-bounded-phpt.sh --classify-harness-programs .runtime/ptn-w17z14-override-properties-after/manifest-20260615T130108Z.txt` | passed 47/47 runnable |

## 2026-06-15 ptn-w17z.10 SKIPIF Harness Row Pack

| Ported Tests | Passed Tests |
|---:|---:|
|185|3|

| Integrated Verification | Result |
| --- | --- |
| `cargo fmt --check` | passed |
| `cargo test --test phpt_classifier -- --nocapture` | passed 59/59 |
| `PHPT_PROGRESS_DIR=.runtime/merge-ptn-w17z10-final-row-pack-classify timeout 600s tools/run-bounded-phpt.sh --classify-only --classify-harness-programs tools/phpt-ptn-w17z10-skipif-harness-row-pack.txt` | selected 185, runnable 185, excluded 0 |

## 2026-06-15 ptn-w17z.20 Stream/Path/Include Runtime Row Pack

Final manifest: `tools/phpt-ptn-w17z20-stream-path-runtime.txt`.

Implemented behavior: stream resources now cover `feof()`, `fflush()`,
`fgetc()`, `fgets()`, `fread()`, `fpassthru()`, `fseek()`, `ftell()`,
`rewind()`, `fstat()`, `ftruncate()`, `tmpfile()`, `stream_get_contents()`,
`stream_get_line()`, and plain `stream_copy_to_stream()` paths. Filesystem
helpers add `file()` flags/include-path lookup and `readfile()` output/count
semantics; directory resources add `readdir()`/`rewinddir()` validation and
directory-wrapper `fseek()` rewind behavior. CSV support includes modeled
`fgetcsv()`/`fputcsv()` named/default parameters and enough generic parsing for
the passing CSV-lite rows. PHP `x*` fopen modes map to native exclusive write
modes while preserving the PHP mode string on stream metadata, and stat arrays
now emit numeric keys before named keys.

Source bucket: full-corpus 20k-family stream/path/include candidate rows on
php-src corpus revision `8c63ec400ce8e07c57a8d9499317b96a8beafb8b`. The full
family generator selected 20,000 rows in
`.runtime/ptn-w17z20-full-family/20260615T101718Z/phpt-full-corpus-20000.txt`;
the stream/path/include candidate slice selected 1,249 rows, with 306 runnable
and 943 classified at
`.runtime/ptn-w17z20-candidates-classify-before/summary-20260615T101824Z.txt`.

Before evidence:
From `/tmp/ptn-w17z20-before` at `2056b888fba5`:
`PHPT_PROGRESS_DIR=/home/claude/gt/ptn_from_scratch/polecats/guard-477/ptn_from_scratch/.runtime/ptn-w17z20-final-before timeout 1200 tools/run-bounded-phpt.sh --classify-harness-programs /home/claude/gt/ptn_from_scratch/polecats/guard-477/ptn_from_scratch/tools/phpt-ptn-w17z20-stream-path-runtime.txt`.
Artifact `.runtime/ptn-w17z20-final-before/run-20260615T114521Z-manifest.log`
recorded 35 selected, 35 runnable, 0 passed, 35 failed.

Final rebased after evidence:
`PHPT_PROGRESS_DIR=.runtime/ptn-w17z20-final-rebased-head-after timeout 1200 tools/run-bounded-phpt.sh --classify-harness-programs tools/phpt-ptn-w17z20-stream-path-runtime.txt`.
Artifact `.runtime/ptn-w17z20-final-rebased-head-after/run-20260615T123716Z-manifest.log`
recorded 35 selected, 35 runnable, 35 passed, 0 failed.

Integrated verification after merge:
`.runtime/merge-ptn-w17z20-stream-path-runtime-rerun/summary-20260615T130149Z.txt`
selected 35 rows, all 35 runnable, and passed 35.

Newly passing runnable rows:

- `ext/standard/tests/dir/readdir_variation7.phpt`
- `ext/standard/tests/dir/rewinddir_variation3.phpt`
- `ext/standard/tests/file/directory_wrapper_fstat_basic.phpt`
- `ext/standard/tests/file/feof_basic.phpt`
- `ext/standard/tests/file/fgetc_basic.phpt`
- `ext/standard/tests/file/fgetc_variation2.phpt`
- `ext/standard/tests/file/fgetcsv_tab_delimiter.phpt`
- `ext/standard/tests/file/fgetcsv_variation12.phpt`
- `ext/standard/tests/file/fgetcsv_variation23.phpt`
- `ext/standard/tests/file/fgets_basic.phpt`
- `ext/standard/tests/file/fgets_error.phpt`
- `ext/standard/tests/file/fgets_variation2.phpt`
- `ext/standard/tests/file/file_basic.phpt`
- `ext/standard/tests/file/fpassthru_variation.phpt`
- `ext/standard/tests/file/fread_error.phpt`
- `ext/standard/tests/file/fread_fwrite_basic.phpt`
- `ext/standard/tests/file/fseek_dir_basic.phpt`
- `ext/standard/tests/file/fseek_ftell_rewind_basic1.phpt`
- `ext/standard/tests/file/fseek_ftell_rewind_error1.phpt`
- `ext/standard/tests/file/fseek_ftell_rewind_error2.phpt`
- `ext/standard/tests/file/fseek_ftell_rewind_error3.phpt`
- `ext/standard/tests/file/fseek_ftell_rewind_variation5.phpt`
- `ext/standard/tests/file/fseek_ftell_rewind_variation7.phpt`
- `ext/standard/tests/file/fseek_variation3.phpt`
- `ext/standard/tests/file/fstat.phpt`
- `ext/standard/tests/file/fstat_basic.phpt`
- `ext/standard/tests/file/fstat_variation8.phpt`
- `ext/standard/tests/file/ftruncate.phpt`
- `ext/standard/tests/file/ftruncate_variation4.phpt`
- `ext/standard/tests/file/readfile_error.phpt`
- `ext/standard/tests/file/readfile_variation9.phpt`
- `ext/standard/tests/file/stream_copy_to_stream_interleaved.phpt`
- `ext/standard/tests/file/stream_get_line.phpt`
- `ext/standard/tests/streams/stream_get_contents_001.phpt`
- `ext/standard/tests/streams/stream_get_contents_negative_length.phpt`

Residual focused 89-row failures are mostly outside this row pack:
full `fgetcsv()`/`fputcsv()` parity, write-only read diagnostics, deprecated
`FILE_BINARY`/`FILE_TEXT` constants, filtered stream-copy behavior, php://temp
spillover seek state, and include bookkeeping for include-path readfile rows.
Follow-up beads: `ptn-jrzu`, `ptn-evvk`, `ptn-klcf`, `ptn-utht`, `ptn-vqg1`,
and `ptn-fio3`.

## 2026-06-15 ptn-w17z.7 ReflectionClass Metadata Row Pack

|ptn-w17z.7 ported tests|ptn-w17z.7 passed tests|
|---|---:|
|ext/reflection/tests/ReflectionClass_CannotClone_basic.phpt|1|
|ext/reflection/tests/ReflectionClass_constructor_001.phpt|1|
|ext/reflection/tests/ReflectionClass_constructor_002.phpt|1|
|ext/reflection/tests/ReflectionClass_getExtension_variation.phpt|1|
|ext/reflection/tests/ReflectionClass_getExtensionName_variation.phpt|1|
|ext/reflection/tests/ReflectionClass_getConstructor_basic.phpt|1|
|ext/reflection/tests/ReflectionClass_getInterfaceNames_basic.phpt|1|
|ext/reflection/tests/ReflectionClass_getInterfaces_001.phpt|1|
|ext/reflection/tests/ReflectionClass_getMethod_001.phpt|1|
|ext/reflection/tests/ReflectionClass_getMethods_001.phpt|1|
|ext/reflection/tests/ReflectionClass_getModifiers_basic.phpt|1|
|ext/reflection/tests/ReflectionClass_getName_basic.phpt|1|
|ext/reflection/tests/ReflectionClass_getNamespaceName.phpt|1|
|ext/reflection/tests/ReflectionClass_getParentClass.phpt|1|
|ext/reflection/tests/ReflectionClass_hasConstant_001.phpt|1|
|ext/reflection/tests/ReflectionClass_hasConstant_002.phpt|1|
|ext/reflection/tests/ReflectionClass_hasConstant_basic.phpt|1|
|ext/reflection/tests/ReflectionClass_hasMethod_001.phpt|1|
|ext/reflection/tests/ReflectionClass_hasMethod_002.phpt|1|
|ext/reflection/tests/ReflectionClass_hasMethod_basic.phpt|1|
|ext/reflection/tests/ReflectionClass_hasProperty_002.phpt|1|
|ext/reflection/tests/ReflectionClass_hasProperty_basic.phpt|1|
|ext/reflection/tests/ReflectionClass_isAbstract_basic.phpt|1|
|ext/reflection/tests/ReflectionClass_isAnonymous.phpt|1|
|ext/reflection/tests/ReflectionClass_isFinal_basic.phpt|1|
|ext/reflection/tests/ReflectionClass_isInstance_basic.phpt|1|
|ext/reflection/tests/ReflectionClass_isInstantiable_basic.phpt|1|
|ext/reflection/tests/ReflectionClass_isInstantiable_variation.phpt|1|
|ext/reflection/tests/ReflectionClass_isInterface_basic.phpt|1|
|ext/reflection/tests/ReflectionClass_isInternal_basic.phpt|1|
|ext/reflection/tests/ReflectionClass_isIterateable_basic.phpt|1|
|ext/reflection/tests/ReflectionClass_isIterateable_variation1.phpt|1|
|ext/reflection/tests/ReflectionClass_isSubclassOf_002.phpt|1|
|ext/reflection/tests/ReflectionClass_isSubclassOf_basic.phpt|1|
|ext/reflection/tests/ReflectionClass_isUserDefined_basic.phpt|1|
|ext/reflection/tests/ReflectionClass_modifiers_001.phpt|1|
|TOTAL|36|

Integrated verification:
`.runtime/merge-ptn-w17z7-reflection-class/summary-20260615T122500Z.txt`
selected 36 rows, all 36 runnable, and passed 36.

## 2026-06-15 ptn-w17z.19 PHPT ENV/CLEAN Harness Row Pack

Full-corpus 20k-family instrument:
`.runtime/ptn-w17z19-full-family-before/20260615T105201Z/phpt-full-corpus-20000.txt`.
The 20k-family modeled-source ENV/CLEAN slice
`.runtime/ptn-w17z19-full-20k-env-clean-modeled-source.txt` selected 474 rows.
Old blocker defaults (`PTN_PHPT_ENVIRONMENT_SECTIONS=ENV`
`PTN_PHPT_HARNESS_SECTIONS=CLEAN`) classified 0 runnable and 474 excluded at
`.runtime/ptn-w17z19-full-20k-env-clean-before-defaults/summary-20260615T111746Z.txt`.
New defaults classified 201 runnable and 273 excluded at
`.runtime/ptn-w17z19-full-20k-env-clean-after-defaults/summary-20260615T111746Z.txt`.

Focused checked-in manifest:
`tools/phpt-ptn-w17z19-env-clean-harness-row-pack.txt`.
After merge-conflict resolution on the integrated tree, the checked-in focused
manifest selected 4 rows with 4 runnable, 4 passed, and 0 classified
(`.runtime/merge-ptn-w17z19-env-clean/summary-20260615T121644Z.txt`).
The old defaults classified the broader 527-row modeled-source ENV/CLEAN probe
as 0 runnable and 527 excluded; the new defaults classified it as 207 runnable
and 320 excluded. A focused 60-row cleanup probe produced 4 passed and 28
failed runnable rows; residual failures are filesystem/parser helper gaps
including `fscanf()`, `fopen()` mode variations, stat cache, `parse_ini_file()`,
`copy()`, `ftruncate()`, `glob()`, and `readfile()` include-path/stream-context
edges.

Newly passing checked-in focused rows:

- `ext/standard/tests/array/array_count_values_variation.phpt`
- `ext/standard/tests/file/file_get_contents_basic001.phpt`
- `ext/standard/tests/file/file_get_contents_variation7.phpt`
- `ext/standard/tests/file/filesize_basic.phpt`

## 2026-06-15 ptn-w17z.12 Magic Methods/Lifecycle Row Pack

| Ported tests | Passed tests |
| --- | --- |
| `tools/phpt-ptn-w17z12-magic-lifecycle-newly-passing.txt` | 49/49 current; 49 failed on saved base `5a4d735dfd5d` |

| Integrated verification | Result |
| --- | --- |
| `PHPT_PROGRESS_DIR=.runtime/merge-ptn-w17z12-magic-lifecycle timeout 1200s tools/run-bounded-phpt.sh --classify-harness-programs tools/phpt-ptn-w17z12-magic-lifecycle-newly-passing.txt` | selected 49, runnable 49, passed 49, failed 0; `.runtime/merge-ptn-w17z12-magic-lifecycle/summary-20260615T144358Z.txt` |

## 2026-06-15 ptn-w17z.21 Date/Time/Formatting Row Pack

Checked-in manifests:
`tools/phpt-ptn-w17z21-date-scalar-row-pack.txt`,
`tools/phpt-ptn-w17z21-formatting-row-pack.txt`, and
`tools/phpt-ptn-w17z21-newly-passing.txt`.

Implemented behavior: `sprintf`/`printf`/`fprintf` formatting now ignores
integer precision for integer conversions, normalizes scientific exponent
spelling, supports PHP custom numeric padding, right-pads zero-fill
left-adjusted floats, and truncates numeric formatting inputs with
formatter-local integer coercion. Scalar date/time internals now include
`date_default_timezone_get`, `date_default_timezone_set`, `time`, `mktime`,
`gmmktime`, `gmdate`, `checkdate`, `getdate`, `localtime`, and `idate`, plus
common `date()` tokens and global `DATE_*` constants. Runtime `date.timezone`
INI settings are bridged into native execution through `PTN_DATE_TIMEZONE`.

Full-corpus 20k-family source instrumentation:
`timeout 900 tools/run-phpt-baseline.sh --scope full --tier 20000 --classify-only --out-dir .runtime/ptn-w17z21-full-before`.
The run generated the deterministic full-corpus manifest family under
`.runtime/ptn-w17z21-full-before/20260615T094544Z/` from corpus revision
`8c63ec400ce8e07c57a8d9499317b96a8beafb8b` with 21,867 rows across 74
buckets; it timed out before a completed 20k classification summary.

Focused date evidence: saved-base execution selected 45 rows with 40 runnable,
0 passed, 40 failed, and 5 classified
(`/tmp/ptn-w17z21-before/.runtime/ptn-w17z21-date-scalar-before/summary-20260615T104122Z.txt`).
Current-branch execution selected the same 45 rows with 40 runnable, 29 passed,
11 failed, and 5 classified
(`.runtime/ptn-w17z21-date-scalar-after-final/summary-20260615T111604Z.txt`).
After merge-conflict resolution on the integrated tree, the same checked-in
date manifest selected 45 rows with 40 runnable, 29 passed, 11 failed, and
5 classified
(`.runtime/merge-ptn-w17z21-date-scalar/summary-20260615T115051Z.txt`).

Focused formatting evidence: saved-base execution selected 81 rows with 52
runnable, 29 passed, 23 failed, and 29 classified
(`/tmp/ptn-w17z21-before/.runtime/ptn-w17z21-formatting-before-clean/summary-20260615T100829Z.txt`).
Current-branch execution selected the same 81 rows with 52 runnable, 41 passed,
11 failed, and 29 classified
(`.runtime/ptn-w17z21-formatting-after-final/summary-20260615T111604Z.txt`).
After merge-conflict resolution on the integrated tree, the same checked-in
formatting manifest selected 81 rows with 52 runnable, 41 passed, 11 failed,
and 29 classified
(`.runtime/merge-ptn-w17z21-formatting/summary-20260615T115900Z.txt`).

Residual frontier: `DateTime` class constants/object APIs, invalid timezone
startup/notice diagnostics, ISO week/year `date()` tokens, exact
`mktime()` diagnostic/range edges, star width/precision formatting, formatter
error diagnostics, rope-optimization formatting rows, and remaining char/
scientific whitespace variations stay outside this row pack.

## 2026-06-15 ptn-w17z.3 Array Sort Flags Row Pack

Final checked-in newly passing manifest:
`tools/phpt-ptn-w17z3-array-sort-flags-row-pack.txt`.

Implemented behavior: direct variable `sort()`, `rsort()`, `asort()`,
`arsort()`, `ksort()`, and `krsort()` now accept validated PHP sort flags
instead of parser-rejecting anything beyond `SORT_REGULAR`. Runtime sorting
supports regular, numeric, binary string, locale-string-as-string, natural,
and string/natural `SORT_FLAG_CASE` comparisons for values and keys, reindexes
`sort()`/`rsort()`, preserves keys for associative sorts, detaches COW arrays
before mutation, and leaves dynamic by-reference-mismatch calls operating on a
temporary array so the caller variable is not mutated after the warning.

Source bucket and classification evidence:

- Full source bucket generated from `/home/claude/php-src-phpt/ext/standard/tests/array`
  contains 857 array PHPT rows, with `ext/standard/tests/array/sort` contributing
  159 sort-family rows.
- Full standard-array classify-only before:
  `.runtime/ptn-w17z3-full-array-classify-before/summary-20260615T090826Z.txt`
  selected 857 rows: 807 runnable and 50 classified.
- Full standard-array classify-only after:
  `.runtime/ptn-w17z3-full-array-classify-after/summary-20260615T102231Z.txt`
  selected 857 rows: 807 runnable and 50 classified.

Complete focused PHPT execution evidence used the 77-row classic sort subset
from that source bucket:

- Before on `origin/master` commit `5bb8cc8a3d0b`:
  `.runtime/ptn-w17z3-classic-before-master/summary-20260615T100448Z.txt`
  selected 77 rows, with 76 runnable and 1 classified; 17 passed and 59 failed.
- After on pre-squash branch commit `6d9582c01f90`:
  `.runtime/ptn-w17z3-classic-after-branch/summary-20260615T100448Z.txt`
  selected 77 rows, with 76 runnable and 1 classified; 63 passed and 13 failed.

The exact before-fail/after-pass delta is 46 rows with no pass-set regressions,
recorded in `tools/phpt-ptn-w17z3-array-sort-flags-row-pack.txt`.

Final branch verification:

- After rebasing onto `6aa9adf83`, `cargo fmt --check` passed.
- After rebasing onto `6aa9adf83`,
  `cargo test sort_flags --test compile_native -- --nocapture` passed 3/3.
- After rebasing onto `6aa9adf83`, the 8-row `sort` PHPT smoke passed 8/8 at
  `.runtime/ptn-w17z3-final-base-sort-smoke-run/summary-20260615T112545Z.txt`.
- After fast-forward integration at `6d4d4d724`, the checked-in 46-row PHPT
  manifest passed 46/46 at
  `.runtime/merge-ptn-w17z3-array-sort-flags/summary-20260615T113453Z.txt`.
- The checked-in 46-row PHPT manifest was split by sort function to avoid
  environment termination of a long single run; after the prior `941f915c0`
  rebase on code commit `33c3bdca8cb3`, the chunks passed 46/46:
  arsort 8/8 at
  `.runtime/ptn-w17z3-final-rebase2-arsort-run/summary-20260615T111108Z.txt`,
  asort 8/8 at
  `.runtime/ptn-w17z3-final-rebase2-asort-run/summary-20260615T111300Z.txt`,
  krsort 8/8 at
  `.runtime/ptn-w17z3-final-rebase2-krsort-run/summary-20260615T111453Z.txt`,
  ksort 8/8 at
  `.runtime/ptn-w17z3-final-rebase2-ksort-run/summary-20260615T111636Z.txt`,
  rsort 6/6 at
  `.runtime/ptn-w17z3-final-rebase2-rsort-run/summary-20260615T111819Z.txt`, and
  sort 8/8 at
  `.runtime/ptn-w17z3-final-rebase2-sort-run/summary-20260615T111946Z.txt`.

The native tests cover parser acceptance for sort flags, direct variable
mutation with COW detach, invalid flag `ValueError`, key/value flag
comparators, and dynamic by-reference mismatch behavior.

## 2026-06-15 ptn-7wxg Simple Trait Composition Row Pack

Final checked-in focused manifest:
`tools/phpt-ptn-7wxg-simple-trait-row-pack.txt`.

Implemented behavior: the parser now accepts simple `trait` declarations,
records top-level and namespaced traits in the AST/IR, and composes trait
methods, instance properties, static properties, and constants into using
classes before lowering. Recursive trait use is flattened for the supported
simple `use TraitName;` form; class-local methods keep precedence over
imported trait methods, duplicate imported method names produce a diagnostic,
and missing/cyclic trait references are rejected. Backend metadata now emits
declared trait tables, `trait_exists()` is modeled as an internal, and
`__TRAIT__` reports the defining trait for imported methods.

Classifier behavior: simple trait declarations and simple class/trait
composition are runnable. Trait adaptation blocks (`use T { a as b; }`),
alias/precedence conflict handling, direct trait reflection/instantiation,
`get_declared_traits()`, property hooks, and stricter abstract/signature/
property/constant conflict diagnostics remain in separate unsupported or
failing frontier buckets.

Full-corpus trait source bucket:
`.runtime/ptn-7wxg-probes/full-trait-scan.txt` selected 349 PHPT rows from
the php-src corpus revision `8c63ec400ce8e07c57a8d9499317b96a8beafb8b`.
Saved-base classify-only evidence from `/tmp/ptn-7wxg-before`:
`.runtime/ptn-7wxg-trait-scan-before-origin/summary-20260615T092841Z.txt`
selected 349 rows: 21 runnable and 328 classified, including 302
`unsupported-trait-declaration` rows. Current-branch classify-only evidence:
`.runtime/ptn-7wxg-trait-scan-after-classify/summary-20260615T092356Z.txt`
selected the same 349 rows: 212 runnable and 137 classified, with
`unsupported-trait-declaration` reduced to 61 rows. Executing the 212
current-runnable rows produced 80 passed and 132 failed
(`.runtime/ptn-7wxg-trait-runnable-after-probe/summary-20260615T092714Z.txt`);
the failures are residual trait reflection, adaptation, property hook,
variance/type, direct trait access, and strict conflict-diagnostic edges.

Focused red-to-green evidence for the checked-in 50-row pack: raw execution
against saved base commit `5bb8cc8a3d0b` with classification disabled selected
50 rows: 50 runnable, 1 passed, and 49 failed
(`.runtime/ptn-7wxg-row-pack-50-before-origin/summary-20260615T095842Z.txt`).
Current-branch execution selected the same 50 rows: 50 runnable, 50 passed, 0
failed, and 0 classified
(`.runtime/ptn-7wxg-row-pack-50-after-rebased-final/summary-20260615T102001Z.txt`).

Newly passing checked-in focused rows:

- `Zend/tests/ArrayAccess/bug64417.phpt`
- `Zend/tests/anon/009.phpt`
- `Zend/tests/anon/010.phpt`
- `Zend/tests/anon/014.phpt`
- `Zend/tests/attributes/override/018.phpt`
- `Zend/tests/attributes/override/gh12189.phpt`
- `Zend/tests/bug69420.phpt`
- `Zend/tests/class_exists_003.phpt`
- `Zend/tests/class_name/parent_class_name_without_parent.phpt`
- `Zend/tests/ctor_promotion/ctor_promotion_trait.phpt`
- `Zend/tests/gh20672.phpt`
- `Zend/tests/inheritance/bug71414.phpt`
- `Zend/tests/readonly_classes/gh9285_success.phpt`
- `Zend/tests/readonly_props/readonly_trait_match.phpt`
- `Zend/tests/static_variables/static_variable_in_private_trait_method.phpt`
- `Zend/tests/traits/abstract_method_2.phpt`
- `Zend/tests/traits/abstract_method_4.phpt`
- `Zend/tests/traits/abstract_method_7.phpt`
- `Zend/tests/traits/abstract_method_8.phpt`
- `Zend/tests/traits/bug55372.phpt`
- `Zend/tests/traits/bug55825.phpt`
- `Zend/tests/traits/bug60217a.phpt`
- `Zend/tests/traits/bug60536_003.phpt`
- `Zend/tests/traits/bug60536_004.phpt`
- `Zend/tests/traits/bug60536_005.phpt`
- `Zend/tests/traits/bug60809.phpt`
- `Zend/tests/traits/bug65419.phpt`
- `Zend/tests/traits/bug65576a.phpt`
- `Zend/tests/traits/bug65576b.phpt`
- `Zend/tests/traits/bug70958.phpt`
- `Zend/tests/traits/bug71275.phpt`
- `Zend/tests/traits/bug74607.phpt`
- `Zend/tests/traits/bug74607a.phpt`
- `Zend/tests/traits/bug74922.phpt`
- `Zend/tests/traits/bug74922a.phpt`
- `Zend/tests/traits/bug75607.phpt`
- `Zend/tests/traits/bug76773.phpt`
- `Zend/tests/traits/bug78787.phpt`
- `Zend/tests/traits/bugs/abstract-methods03.phpt`
- `Zend/tests/traits/bugs/overridding-conflicting-methods.phpt`
- `Zend/tests/traits/bugs/overridding-static-property-with-doc-block.phpt`
- `Zend/tests/traits/conflict002.phpt`
- `Zend/tests/traits/constant_002.phpt`
- `Zend/tests/traits/constant_003.phpt`
- `Zend/tests/traits/constant_017.phpt`
- `Zend/tests/traits/flattening001.phpt`
- `Zend/tests/traits/flattening002.phpt`
- `Zend/tests/traits/flattening003.phpt`
- `Zend/tests/traits/gh14009_003.phpt`

The remaining checked-in focused row,
`Zend/tests/grammar/semi_reserved_004.phpt`, was already passing in the raw
saved-base run and is retained as an adjacent control row.

## 2026-06-15 ptn-w17z.4 Parser/Control-Flow Row Pack

Final checked-in focused manifest:
`tools/phpt-parser-control-current-ptn-w17z4-manifest.txt`.

Implemented behavior: parser/IR/backend support for PHP `match` expressions,
including standalone match expression statements, default arms, trailing comma
condition lists, strict arm matching, catchable `UnhandledMatchError` messages,
and match results as ordinary non-reference temporaries. Catch clauses now
accept multi-catch type lists. Try/catch lowering carries a `finally` body for
normal completion, handled catches, and uncaught rethrow paths. Include files
with class declarations now contribute class metadata so simple class helper
includes can participate in exception control-flow rows. Runtime exception
metadata now includes `RuntimeException` and `UnhandledMatchError` in the
builtin hierarchy, and exception string casts produce PHP-style exception text.

Full-corpus/20k-family evidence: the hook generated
`.runtime/ptn-w17z4-full-before/20260615Tmanual/phpt-baseline-all.txt` with
21,867 PHPT rows and
`.runtime/ptn-w17z4-full-before/20260615Tmanual/phpt-baseline-20000.txt` with
20,000 rows. All 56 checked-in row-pack paths are present in that 20k manifest
(`manifest_rows=56`, `in_20k=56`).

Before evidence on corpus revision `8c63ec400ce8e07c57a8d9499317b96a8beafb8b`:
Zend match rows
`.runtime/ptn-w17z4-zend-match-before/summary-20260615T090823Z.txt` selected
35 rows: 30 runnable, 0 passed, 30 failed, 5 classified. Catch-union rows
`.runtime/ptn-w17z4-catch-union-before/summary-20260615T090927Z.txt` selected
22 rows: 10 runnable, 0 passed, 10 failed, 12 classified.

Final focused evidence after rebasing onto current `origin/master`:
`PHPT_PROGRESS_DIR=.runtime/ptn-w17z4-row-pack-rebased-final2 timeout 1200 tools/run-bounded-phpt.sh --classify-harness-programs tools/phpt-parser-control-current-ptn-w17z4-manifest.txt`.
Artifact `.runtime/ptn-w17z4-row-pack-rebased-final2/summary-20260615T103334Z.txt`
selected 56 rows: 39 runnable, 36 passed, 3 failed, and 17 classified.
Residual failed rows are `Zend/tests/match/009_ast_export.phpt`,
`Zend/tests/match/045.phpt`, and
`Zend/tests/match/match_scdf_cleanup.phpt`.

Newly passing full-corpus/20k-family rows:

- `Zend/tests/match/001.phpt`
- `Zend/tests/match/002.phpt`
- `Zend/tests/match/003.phpt`
- `Zend/tests/match/004.phpt`
- `Zend/tests/match/005.phpt`
- `Zend/tests/match/006.phpt`
- `Zend/tests/match/007.phpt`
- `Zend/tests/match/008.phpt`
- `Zend/tests/match/011.phpt`
- `Zend/tests/match/012.phpt`
- `Zend/tests/match/017.phpt`
- `Zend/tests/match/023.phpt`
- `Zend/tests/match/024.phpt`
- `Zend/tests/match/027.phpt`
- `Zend/tests/match/028.phpt`
- `Zend/tests/match/037.phpt`
- `Zend/tests/match/038.phpt`
- `Zend/tests/match/039.phpt`
- `Zend/tests/match/040.phpt`
- `Zend/tests/match/041.phpt`
- `Zend/tests/match/042.phpt`
- `Zend/tests/match/043.phpt`
- `Zend/tests/match/044.phpt`
- `Zend/tests/match/046.phpt`
- `Zend/tests/match/047.phpt`
- `Zend/tests/match/gh11134.phpt`
- `Zend/tests/match/match_of_phi_optimization.phpt`
- `Zend/tests/try/bug74444.phpt`
- `Zend/tests/try/catch_novar_1.phpt`
- `Zend/tests/try/try_multicatch_001.phpt`
- `Zend/tests/try/try_multicatch_002.phpt`
- `Zend/tests/try/try_multicatch_003.phpt`
- `Zend/tests/try/try_multicatch_004.phpt`
- `Zend/tests/try/try_multicatch_005.phpt`
- `Zend/tests/try/try_multicatch_006.phpt`
- `Zend/tests/try/try_multicatch_007.phpt`

## 2026-06-15 ptn-w17z.11 Namespaces/Includes Row Pack

| Artifact | Ported Tests | Passed Tests |
| --- | ---: | ---: |
| `.runtime/ptn-w17z-namespace-before/summary-20260615T091109Z.txt` | 123 | 58 |
| `.runtime/ptn-w17z-dirname-dirsep-before/summary-20260615T092858Z.txt` | 11 | 0 |
| `.runtime/merge-ptn-w17z11-row-pack-rebased/summary-20260615T111757Z.txt` | 26 | 26 |
| `tools/phpt-ptn-w17z11-namespace-include-row-pack.txt` | 26 | 26 |

| Test | Ported Tests | Passed Tests |
| --- | ---: | ---: |
| `Zend/tests/namespaces/bug46813.phpt` | 1 | 1 |
| `Zend/tests/namespaces/bug47593.phpt` | 1 | 1 |
| `Zend/tests/namespaces/ns_009.phpt` | 1 | 1 |
| `Zend/tests/namespaces/ns_012.phpt` | 1 | 1 |
| `Zend/tests/namespaces/ns_013.phpt` | 1 | 1 |
| `Zend/tests/namespaces/ns_019.phpt` | 1 | 1 |
| `Zend/tests/namespaces/ns_020.phpt` | 1 | 1 |
| `Zend/tests/namespaces/ns_032.phpt` | 1 | 1 |
| `Zend/tests/namespaces/ns_035.phpt` | 1 | 1 |
| `Zend/tests/namespaces/ns_036.phpt` | 1 | 1 |
| `Zend/tests/namespaces/ns_040.phpt` | 1 | 1 |
| `Zend/tests/namespaces/ns_041.phpt` | 1 | 1 |
| `Zend/tests/namespaces/ns_042.phpt` | 1 | 1 |
| `Zend/tests/namespaces/ns_044.phpt` | 1 | 1 |
| `Zend/tests/namespaces/ns_045.phpt` | 1 | 1 |
| `Zend/tests/namespaces/ns_047.phpt` | 1 | 1 |
| `Zend/tests/namespaces/ns_049.phpt` | 1 | 1 |
| `Zend/tests/namespaces/ns_051.phpt` | 1 | 1 |
| `Zend/tests/namespaces/ns_052.phpt` | 1 | 1 |
| `Zend/tests/namespaces/ns_054.phpt` | 1 | 1 |
| `Zend/tests/namespaces/ns_076.phpt` | 1 | 1 |
| `Zend/tests/namespaces/ns_077_1.phpt` | 1 | 1 |
| `Zend/tests/namespaces/ns_077_2.phpt` | 1 | 1 |
| `Zend/tests/namespaces/ns_077_3.phpt` | 1 | 1 |
| `Zend/tests/namespaces/ns_077_4.phpt` | 1 | 1 |
| `tests/lang/bug73172.phpt` | 1 | 1 |

## 2026-06-15 ptn-mucw Match Expression Row Pack

Final manifest: `tools/phpt-ptn-mucw-match-expression-row-pack.txt`.

Implemented behavior: the lexer recognizes `match`, the parser lowers PHP match
expressions with multi-condition arms, trailing commas before `=>`, default
arms, nested matches, and duplicate default diagnostics. IR/native emission
evaluates the subject once, checks arm conditions with strict identity
comparison, treats `default` as fallback even when it appears before later
specific arms, emits the selected arm value, and throws `UnhandledMatchError`
with PHP-shaped scalar/type messages when no arm matches. `UnhandledMatchError`
is modeled as an `Error` subtype for catch matching, and match-expression
results are treated as temporaries for by-reference method arguments.

Source bucket: full-corpus `Zend/tests/match` on php-src corpus revision
`8c63ec400ce8e07c57a8d9499317b96a8beafb8b`. The broad 20k-family generator for
the current corpus selected all 10,074 broad-source rows; the committed focused
manifest is the full `Zend/tests/match` bucket from that family. A long
20k-family classify-only run was stopped after 4,103/10,074 rows once the
focused full-corpus bucket met the row-pack target.

Before evidence:
`PHPT_PROGRESS_DIR=.runtime/ptn-mucw-match-before tools/run-bounded-phpt.sh --classify-harness-programs .runtime/ptn-mucw-match-all.txt`.
Result: 35 selected, 30 runnable, 5 classified, 0 passed, 30 failed.

Final rebased after evidence:
`PHPT_PROGRESS_DIR=.runtime/ptn-mucw-match-after-rebased tools/run-bounded-phpt.sh --classify-harness-programs tools/phpt-ptn-mucw-match-expression-row-pack.txt`.
Artifact `.runtime/ptn-mucw-match-after-rebased/run-20260615T094744Z-manifest.log`
recorded 35 selected, 30 runnable, 5 classified, 26 passed, 4 failed.

Newly passing runnable rows:

- `Zend/tests/match/001.phpt`
- `Zend/tests/match/002.phpt`
- `Zend/tests/match/003.phpt`
- `Zend/tests/match/004.phpt`
- `Zend/tests/match/005.phpt`
- `Zend/tests/match/006.phpt`
- `Zend/tests/match/007.phpt`
- `Zend/tests/match/008.phpt`
- `Zend/tests/match/011.phpt`
- `Zend/tests/match/012.phpt`
- `Zend/tests/match/017.phpt`
- `Zend/tests/match/023.phpt`
- `Zend/tests/match/024.phpt`
- `Zend/tests/match/027.phpt`
- `Zend/tests/match/028.phpt`
- `Zend/tests/match/038.phpt`
- `Zend/tests/match/039.phpt`
- `Zend/tests/match/040.phpt`
- `Zend/tests/match/041.phpt`
- `Zend/tests/match/042.phpt`
- `Zend/tests/match/043.phpt`
- `Zend/tests/match/044.phpt`
- `Zend/tests/match/046.phpt`
- `Zend/tests/match/047.phpt`
- `Zend/tests/match/gh11134.phpt`
- `Zend/tests/match/match_of_phi_optimization.phpt`

Residual runnable failures are not core match-selection gaps:
`Zend/tests/match/009_ast_export.phpt` needs `zend_ast_export`;
`Zend/tests/match/037.phpt` needs exception object string conversion;
`Zend/tests/match/045.phpt` and `Zend/tests/match/match_scdf_cleanup.phpt`
still differ on undefined-constant fatal wording/trace behavior.

## 2026-06-15 ptn-w17z.1 Full PHPT Corpus Family

Implemented behavior: `tools/run-phpt-baseline.sh` now has an explicit
`--scope full` mode that inventories every local php-src `.phpt` row, buckets
rows by full-corpus family (`Zend`, `ext/<extension>`, `sapi/<sapi>`, `tests`,
and other top-level roots), and writes deterministic
`phpt-full-corpus-1000.txt`, `phpt-full-corpus-5000.txt`,
`phpt-full-corpus-10000.txt`, `phpt-full-corpus-20000.txt`, and
`phpt-full-corpus-all.txt` manifests. The existing broad mode remains the
default and continues to generate the legacy Zend/ext-standard/core 1k/5k/10k
family. `tools/check-phpt-campaign-reports.sh` adds the campaign report gate:
reports must be markdown-table-only and contain only ported-test and
passed-test counts.

Before this task, `tools/run-phpt-baseline.sh` only generated the broad family
from 10,074 selected-source rows (`Zend/tests`, `ext/standard/tests`, and
core `tests`) and had no 20k or all-corpus manifest. After this task, the real
local corpus at revision `8c63ec400ce8e07c57a8d9499317b96a8beafb8b` generated
21,867 full-corpus rows across 74 buckets, including a 20,000-row tier and an
all-row tier.

Full-corpus family generation evidence:
`timeout 180 tools/run-phpt-baseline.sh --scope full --generate-only --out-dir .runtime/ptn-w17z1-full-family`.
Artifact `.runtime/ptn-w17z1-full-family/20260615T090903Z/inventory.txt`
recorded 21,867 rows across 74 buckets and wrote:

- `.runtime/ptn-w17z1-full-family/20260615T090903Z/phpt-full-corpus-1000.txt`
- `.runtime/ptn-w17z1-full-family/20260615T090903Z/phpt-full-corpus-5000.txt`
- `.runtime/ptn-w17z1-full-family/20260615T090903Z/phpt-full-corpus-10000.txt`
- `.runtime/ptn-w17z1-full-family/20260615T090903Z/phpt-full-corpus-20000.txt`
- `.runtime/ptn-w17z1-full-family/20260615T090903Z/phpt-full-corpus-all.txt`

Full-scope 1k classify-only evidence:
`PHPT_PROGRESS_DIR=.runtime/ptn-w17z1-full-1k-classify-progress timeout 900 tools/run-phpt-baseline.sh --scope full --tier 1000 --classify-only --out-dir .runtime/ptn-w17z1-full-1k-classify-baseline`.
Artifact `.runtime/ptn-w17z1-full-1k-classify-progress/summary-20260615T091057Z.txt`
selected 1,000 rows: 383 runnable and 617 classified across 67 selected
buckets. This was a tooling/KPI unlock with no PHPT execution and no newly
passing PHPT rows; the newly passing row list for this task is empty by design.

## 2026-06-15 ptn-psbp Broad Parser/Control Row Pack

Final focused manifest:
`tools/phpt-ptn-psbp-parser-control-row-pack-manifest.txt`.

Implemented behavior: assertion diagnostics now preserve source path/line,
assertion expression text for backticks, floats, and anonymous classes, and
throw supplied `Throwable` reasons through declared Throwable subclass
initialization without converting them to internal exceptions. Built-in
exception parent edges are emitted into generated metadata. Magic/asymmetric
property dispatch now distinguishes read, write, indirect write, and unset
contexts, direct unset enforces set visibility, parser checks reject invalid
virtual asymmetric properties, and declared typed property metadata preserves
the last concrete type so `var_dump()` reports unset typed slots as
`uninitialized(T)`.

Focused evidence:
`PHPT_PROGRESS_DIR=.runtime/ptn-psbp-row-pack-final-after-var-dump-fix tools/run-bounded-phpt.sh --classify-harness-programs tools/phpt-ptn-psbp-parser-control-row-pack-manifest.txt`.
Artifact
`.runtime/ptn-psbp-row-pack-final-after-var-dump-fix/summary-20260615T062231Z.txt`
selected 28 rows: 28 runnable, 28 passed, 0 failed. A targeted six-row
assertion/magic/asym recheck also passed 6/6 at
`.runtime/ptn-psbp-targeted-after-var-dump-fix/summary-20260615T062102Z.txt`.
After the final rebase over the latest `origin/master`, the focused manifest
reran 28/28 passing at
`.runtime/ptn-psbp-row-pack-final-rebased/summary-20260615T083757Z.txt`.

Completed broad tier-1000 evidence used corpus revision
`8c63ec400ce8e07c57a8d9499317b96a8beafb8b`:

- Before on current-base commit `57132a10a8f4`:
  `/tmp/ptn-psbp-before-57132a/.runtime/ptn-psbp-broad-before-current-progress/summary-20260615T044957Z.txt`
  selected 1,000 rows, with 609 runnable and 391 classified; 563 passed and
  46 failed.
- After on integrated commit `67c2a4af09aa`:
  `.runtime/ptn-psbp-broad-after-final-current-progress/summary-20260615T063605Z.txt`
  selected 1,000 rows, with 609 runnable and 391 classified; 573 passed and
  36 failed.

Bucket movement was isolated to Zend: `184/222 -> 194/222` passed. The
standard bucket stayed `371/371`, core stayed `8/16`, and the normalized
broad pass-set comparison had 10 newly passing rows with no no-longer-passing
rows. The exact broad delta is checked in at
`tools/phpt-ptn-psbp-broad-newly-passing-20260615T003102Z.txt`.

Newly passing broad rows:

- `Zend/tests/assert/bug71922.phpt`
- `Zend/tests/assert/expect_007.phpt`
- `Zend/tests/assert/expect_009.phpt`
- `Zend/tests/assert/expect_010.phpt`
- `Zend/tests/assert/expect_011.phpt`
- `Zend/tests/assert/expect_017.phpt`
- `Zend/tests/asymmetric_visibility/__set.phpt`
- `Zend/tests/asymmetric_visibility/__unset.phpt`
- `Zend/tests/asymmetric_visibility/bug001.phpt`
- `Zend/tests/asymmetric_visibility/bug002.phpt`

## 2026-06-15 ptn-ur6g Object/Class Metadata Row Pack

Final checked-in focused manifest:
`tools/phpt-ptn-ur6g-object-class-metadata-row-pack.txt`.

Implemented behavior: instance typed properties now carry parser/IR/backend
type metadata into runtime object property metadata, including nullable scalar
coercion, uninitialized typed-property reads, and references bound to typed
properties. Assignment expressions now read back the effective assigned value
for variable and array-path assignments that write through typed-property
references. Parser validation now reports inherited asymmetric property
override errors for final `private(set)` properties and omitted set-visibility
requirements. Internal callback validation now reports inaccessible
private/protected static method callbacks by visibility instead of the generic
invalid-callback message.

Classifier behavior: ordinary typed instance properties, top-level constant
array unpacking, and ordinary final method declarations are runnable. Typed
static properties, property hooks, class-scope unpack defaults, reflection
constructor bypass, and nested `isset`/`unset` diagnostics for possibly
uninitialized asymmetric typed properties remain classified under narrower
frontier buckets.

Focused row-pack evidence on final HEAD:
`PHPT_PROGRESS_DIR=.runtime/ptn-ur6g-row-pack-final-head timeout 900 tools/run-bounded-phpt.sh --classify-harness-programs tools/phpt-ptn-ur6g-object-class-metadata-row-pack.txt`.
Artifact `.runtime/ptn-ur6g-row-pack-final-head/summary-20260615T084203Z.txt`
selected 10 rows: 10 runnable, 10 passed, 0 failed, and 0 classified.

Broad tier-1000 classify-only evidence on corpus revision
`8c63ec400ce8e07c57a8d9499317b96a8beafb8b` used the hook-start manifest
`.runtime/ptn-ur6g-broad-before/20260615T065545Z/phpt-baseline-1000.txt`.
The hook-start summary `.runtime/phpt-progress/summary-20260615T065545Z.txt`
selected 1,000 rows: 611 runnable and 389 classified. The final classify-only
run
`PHPT_PROGRESS_DIR=.runtime/ptn-ur6g-broad-after-classify-rebased-final timeout 600 tools/run-bounded-phpt.sh --classify-only --classify-harness-programs .runtime/ptn-ur6g-broad-before/20260615T065545Z/phpt-baseline-1000.txt`
selected the same 1,000 rows: 635 runnable and 365 classified
(`.runtime/ptn-ur6g-broad-after-classify-rebased-final/summary-20260615T083228Z.txt`).
The 24 newly runnable rows versus hook start include upstream `ptn-z2ji`;
`ptn-ur6g` contributes the 10 newly passing broad-selected rows below.

Newly passing broad-selected rows:

- `Zend/tests/array_unpack/gh9769.phpt`
- `Zend/tests/assign_obj_to_ref_inference.phpt`
- `Zend/tests/assign_typed_ref_result.phpt`
- `Zend/tests/asymmetric_visibility/nested_write.phpt`
- `Zend/tests/asymmetric_visibility/override_private_public.phpt`
- `Zend/tests/asymmetric_visibility/override_protected_public.phpt`
- `Zend/tests/asymmetric_visibility/override_public_private.phpt`
- `Zend/tests/asymmetric_visibility/override_public_protected.phpt`
- `ext/standard/tests/array/array_filter_object.phpt`
- `ext/standard/tests/array/array_map_object1.phpt`

## 2026-06-15 ptn-z2ji INI Quantity/Memory-Limit Row Pack

Implemented behavior: `phpc -d` now parses `error_reporting` constant
bitmask expressions such as `E_ALL ^ E_WARNING`, models `memory_limit` and
`max_memory_limit` CLI INI values, clamps startup and runtime `memory_limit`
changes to finite `max_memory_limit`, keeps `max_memory_limit` immutable at
runtime, and exposes `ini_parse_quantity()` with PHP-compatible shorthand
quantity parsing and sourced warnings.

Focused broad-selected row-pack evidence:
`PHPT_PROGRESS_DIR=.runtime/ptn-z2ji-combined-after timeout 1200 tools/run-bounded-phpt.sh --classify-harness-programs .runtime/ptn-z2ji-probes/ini-memory-combined-pack.txt`.
Artifact `.runtime/ptn-z2ji-combined-after/summary-20260615T072024Z.txt`
selected 13 rows: 13 runnable and 13 passed.

Additional broad-selected memory rows:
`PHPT_PROGRESS_DIR=.runtime/ptn-z2ji-extra-memory-after timeout 600 tools/run-bounded-phpt.sh --classify-harness-programs .runtime/ptn-z2ji-probes/extra-memory-broad-pack.txt`.
Artifact `.runtime/ptn-z2ji-extra-memory-after/summary-20260615T072906Z.txt`
selected 3 rows: 3 runnable and 3 passed.

Broad tier-1000 evidence on corpus revision
`8c63ec400ce8e07c57a8d9499317b96a8beafb8b` used generated manifest
`.runtime/ptn-z2ji-broad-before/20260615T063431Z/phpt-baseline-1000.txt`.
Hook-start classify-only selected 1,000 rows: 611 runnable and 389 classified
(`.runtime/ptn-z2ji-broad-before-classify/summary-20260615T063514Z.txt`).
Final classify-only selected the same 1,000 rows: 625 runnable and 375
classified (`.runtime/ptn-z2ji-broad-after-classify/summary-20260615T072321Z.txt`).

Newly passing broad-selected rows confirmed by focused PHPT runs:

- `Zend/tests/bug36568.phpt`
- `Zend/tests/bug39438.phpt`
- `ext/standard/tests/array/array_sum.phpt`
- `tests/basic/gh17951_ini_parse_1.phpt`
- `tests/basic/gh17951_ini_parse_2.phpt`
- `tests/basic/gh17951_ini_parse_3.phpt`
- `tests/basic/gh17951_ini_parse_4.phpt`
- `tests/basic/gh17951_ini_parse_5.phpt`
- `tests/basic/gh17951_runtime_change_1.phpt`
- `tests/basic/gh17951_runtime_change_2.phpt`
- `tests/basic/gh17951_runtime_change_3.phpt`
- `tests/basic/gh17951_runtime_change_4.phpt`
- `tests/basic/gh17951_runtime_change_5.phpt`
- `tests/basic/gh17951_runtime_change_6.phpt`
- `tests/basic/ini_parse_quantity_basic.phpt`
- `tests/basic/ini_parse_quantity_warnings.phpt`

## 2026-06-15 ptn-dkyr By-Reference Call Unpack Row Pack

Final checked-in focused manifest:
`tools/phpt-ptn-dkyr-byref-unpack-row-pack.txt`.

Implemented behavior: direct userland calls, declared instance-method calls,
and known constructor calls now carry parameter by-reference modes into array
argument unpacking. Runtime array unpack helpers preserve element references
for by-reference formal positions, including by-reference variadic tails, and
separate source arrays before binding by-reference unpacked elements so shared
arrays keep copy-on-write semantics. Explicit by-reference operands in calls
that also contain unpacking now use by-reference call-argument lowering.

Classifier behavior: array unpack into by-reference userland parameters is no
longer blocked as unsupported call-unpacking reference. Traversable/generator
by-reference unpack remains classified by the existing generator/traversable
frontier buckets.

Focused row-pack evidence:
`PHPT_PROGRESS_DIR=.runtime/ptn-dkyr-byref-unpack-final-current timeout 600 tools/run-bounded-phpt.sh --classify-harness-programs tools/phpt-ptn-dkyr-byref-unpack-row-pack.txt`.
Artifact `.runtime/ptn-dkyr-byref-unpack-final-current/summary-20260615T061318Z.txt`
selected 3 rows: 2 runnable, 2 passed, and 1 classified
(`unsupported-generator-runtime`).

Existing call-unpacking row-pack evidence:
`PHPT_PROGRESS_DIR=.runtime/ptn-dkyr-call-unpack-final-current timeout 900 tools/run-bounded-phpt.sh --classify-harness-programs tools/phpt-call-unpacking-current-ptn-ei36-manifest.txt`.
Artifact `.runtime/ptn-dkyr-call-unpack-final-current/summary-20260615T061318Z.txt`
selected 20 rows: 13 runnable, 13 passed, and 7 classified.

Broad tier-1000 evidence on corpus revision
`8c63ec400ce8e07c57a8d9499317b96a8beafb8b` used the generated manifest
`.runtime/ptn-dkyr-broad-before/20260615T051703Z/phpt-baseline-1000.txt`.
The hook-start/stale-classifier summary
`.runtime/phpt-progress/summary-20260615T051703Z.txt` selected 1,000 rows:
609 runnable and 391 classified, including 3 rows under the now-removed
`unsupported-call-unpacking-reference` bucket. A full broad execution from
that manifest timed out after 40 minutes with exit 124 before aggregate
pass/fail totals were available. The final classify-only run
`PHPT_PROGRESS_DIR=.runtime/ptn-dkyr-broad-after-classify timeout 600 tools/run-bounded-phpt.sh --classify-only --classify-harness-programs .runtime/ptn-dkyr-broad-before/20260615T051703Z/phpt-baseline-1000.txt`
selected the same 1,000 rows: 611 runnable and 389 classified
(`.runtime/ptn-dkyr-broad-after-classify/summary-20260615T055859Z.txt`).

Newly passing broad-selected focused rows:

- `Zend/tests/arg_unpack/by_ref.phpt`
- `Zend/tests/arg_unpack/by_ref_separation.phpt`

This row pack produced 2 newly passing broad-selected rows. The inspected
remaining arg-unpack/COW-reference candidates were generator/traversable,
diagnostics/runtime-handler, or typed-property metadata frontiers and were left
classified or failing for separate generic work.

## 2026-06-15 ptn-uv4c Broad Parser/Control Row Pack

Final manifest: `tools/phpt-ptn-uv4c-parser-control-row-pack.txt`.

Broad tier-1000 classify-only baseline on corpus revision
`8c63ec400ce8e07c57a8d9499317b96a8beafb8b` selected 1,000 rows:
592 runnable and 408 classified
(`.runtime/ptn-uv4c-broad-before-classify-progress/summary-20260615T021051Z.txt`).
The final rebased post-implementation broad classify-only run selected the same 1,000 rows:
603 runnable and 397 classified
(`.runtime/ptn-uv4c-broad-after-classify-rebased-fixed-progress/summary-20260615T035623Z.txt`).

Focused row-pack command:
`PHPT_PROGRESS_DIR=.runtime/ptn-uv4c-row-pack-rebased-fixed tools/run-bounded-phpt.sh --classify-harness-programs tools/phpt-ptn-uv4c-parser-control-row-pack.txt`.
Artifact `.runtime/ptn-uv4c-row-pack-rebased-fixed/summary-20260615T035329Z.txt`
selected 26 rows: 12 runnable, 11 passed, 1 failed, and 14 classified.

Newly passing focused rows:

- `Zend/tests/access_modifiers/access_modifiers_011.phpt`
- `Zend/tests/anon/015.phpt`
- `Zend/tests/anon/016.phpt`
- `Zend/tests/arrow_functions/007.phpt`
- `Zend/tests/arrow_functions/gh7900.phpt`
- `Zend/tests/assign_obj_op_cache_slot.phpt`
- `Zend/tests/bug26802.phpt`
- `Zend/tests/bug27669.phpt`
- `Zend/tests/bug28072.phpt`
- `Zend/tests/bug29015.phpt`
- `Zend/tests/bug38287.phpt`

Implemented behavior: function-local `static` declarations now allocate stable
per-function reference slots; nullable `?T` and `never` type hints parse and
lower through parameter and return boundaries; dynamic property reads/writes,
compound assignments, dynamic object method calls, and dynamic static method
calls lower through runtime name conversion; leading-NUL dynamic property names
raise the PHP-style fatal path; normal property reads can dispatch modeled
`__get()` while array-column probing still uses `__isset()` gating; and
assertion-source rendering now preserves arrow-function syntax, typed variadic
parameters, nullable returns, and `never` returns.

Remaining focused residual: `Zend/tests/arrow_functions/006.phpt` is runnable
but still fails because by-reference assignment from a by-reference closure
return (`$ref =& $id($var)`) is not implemented. The other 14 focused rows
remain classified by existing attribute syntax, top-level static, or property
visibility metadata limits.
## Dashboard

|Source|Ported|Passing|Gap|
|---|---:|---:|---:|
|Units|3|3|0|
|Native|733|733|0|
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
|Asym-vis|23|22|1|
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
|Magic-object-texo|6|4|2|
|Magic-methods|8|0|8|
|Method-visibility-iuhj|9|6|3|
|Object-string-meta|61|53|8|
|Object-string-array-pack|34|34|0|
|Object-string-wxno|25|25|0|
|Object-callback-merge-zhup|20|17|3|
|Object-method-29k0|35|32|3|
|Object-method-hgfn|20|11|9|
|Static-property-ck7w|24|2|22|
|Std-array-map|297|0|297|
|Std-arrays|296|263|33|
|Map/filter|30|25|5|
|Request/SAPI|41|1|40|
|Anon-class|15|0|15|
|Interface-decl|23|0|23|
|Interface-impl|15|0|15|
|Trait-decl|25|0|25|
|Call-unpack|20|13|7|
|By-ref-call-unpack-dkyr|3|2|1|
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
|Std-string-array-ppri|20|18|2|
|User-comparator-sort-vq7w|26|20|6|
|Std-strings-m8pk|21|21|0|
|Std-strings-almd|13|12|1|
|Sort-flags-w17z2|78|72|6|
|array_rand|7|6|1|
|Zend-op/control|26|21|5|
|Zend-lit/op-3ijs|30|27|3|
|Parser-cfny|34|31|3|
|Parser-control-psbp|28|28|0|
|Binary-key|1|1|0|
|Runtime-config|54|10|44|
|COW-gate|26|26|0|
|COW-reference-tiqh|21|21|0|
|COW-reference-mqvk|28|27|1|
|COW-reference-vq7w|24|24|0|
|COW-reference-25s0|27|26|1|
|COW-reference-dgj9|22|8|14|
|COW-reference-d0lg|48|23|25|
|Broad-25s0-runnable|591|527|64|
|Broad-ppri-runnable|591|528|63|
|1k-baseline|1000|573|427|
|Full-PHPT inventory|21867|21867|0|
|Full-PHPT 1k runnable|1000|383|617|

## 2026-06-15 ptn-d0lg Broad 1k COW/Reference Row Pack

Final focused manifest: `tools/phpt-ptn-d0lg-cow-reference-row-pack.txt`.

Post-rebase focused PHPT evidence:
`.runtime/ptn-d0lg-focused-pack-post-rebase-final/summary-20260615T080832Z.txt`
selected 48 rows, 48 runnable, 23 passed, 25 failed.

Original focused PHPT evidence before the final upstream rebase:
`.runtime/ptn-d0lg-focused-pack-final/summary-20260615T043228Z.txt`
selected 48 rows, 48 runnable, 10 passed, 38 failed.

Clean broad 1k before/after evidence uses the deterministic tier-1000
manifest for PHP corpus revision
`8c63ec400ce8e07c57a8d9499317b96a8beafb8b`:

- Before, clean pre-work worktree
  `/tmp/ptn-d0lg-before-work/.runtime/ptn-d0lg-broad-before-progress`:
  1,000 selected, 592 runnable, 408 excluded, 536 passed, 56 failed.
  Buckets: Zend 157/205, standard 371/371, core 8/16.
- After, commit `9fec7c7669c3`,
  `.runtime/ptn-d0lg-broad-after-final2-progress/summary-20260615T054632Z.txt`:
  1,000 selected, 592 runnable, 408 excluded, 546 passed, 46 failed.
  Buckets: Zend 167/205, standard 371/371, core 8/16.

Newly passing broad rows:

- `Zend/tests/ArrayAccess/ArrayAccess_indirect_append.phpt`
- `Zend/tests/ArrayAccess/bug71731.phpt`
- `Zend/tests/assign_array_object_property.phpt`
- `Zend/tests/assign_dim_obj_null_return.phpt`
- `Zend/tests/bug31098.phpt`
- `Zend/tests/bug31525.phpt`
- `Zend/tests/bug33996.phpt`
- `Zend/tests/bug34064.phpt`
- `Zend/tests/bug34786.phpt`
- `Zend/tests/bug37251.phpt`

Implemented behavior: property-rooted array reference/append targets, nested
ArrayAccess quiet lookup with `offsetExists()` before `offsetGet()`, dynamic
`new $objects[0]` class-name values, catchable invalid array/scalar-property
write errors, nested `@` error-reporting restoration, and source-path aware
by-reference diagnostics. Direct user-call `$array[]` by-value arguments now
keep the fatal "Cannot use [] for reading" shape, while method/dynamic callback
paths remain catchable where PHP expects. Catchable user argument-count errors
now include source location for direct PHP call sites but keep pathless messages
when invoked by internal callbacks such as `array_filter()`.

Remaining broad failures are concentrated in existing unsupported areas such as
`ArrayObject` (`bug68896`), asymmetric visibility/attribute metadata, assertion
stack traces/runtime config, iterators/destructors, and core runtime environment
features.

## 2026-06-15 ptn-almd Stateless Ext-Standard String Helpers

Final manifest:
`tools/phpt-ptn-almd-stateless-string-helpers-row-pack.txt`.

Implemented behavior: added generic byte-string internals for `count_chars()`,
`convert_uuencode()`, `convert_uudecode()`, `levenshtein()`,
`quoted_printable_encode()`, and `str_word_count()`. The helpers are registered
through the parser/internal-function metadata path, use scalar argument
conversion, return PHP arrays/strings through the shared boxed runtime, and
raise `ValueError` for invalid `count_chars()` modes, `str_word_count()`
formats, and negative Levenshtein costs.

Focused broad-derived PHPT evidence:
`PHPT_PROGRESS_DIR=.runtime/ptn-almd-stateless-submit-final tools/run-bounded-phpt.sh --classify-harness-programs tools/phpt-ptn-almd-stateless-string-helpers-row-pack.txt`.
It selected 13 rows, kept 13 runnable, passed 12, and failed 1 remaining
`quoted_printable_encode_002.phpt` long-line folding row.

Before evidence for the same rows came from
`.runtime/ptn-almd-missing-strings-before/summary-20260615T053002Z.txt`: the
13 implemented rows were all failing as undefined functions inside a 28-row
missing string algorithm probe where only the existing
`quoted_printable_decode_basic.phpt` row passed. Newly passing focused rows:
`convert_uudecode_basic.phpt`, `convert_uuencode_basic.phpt`,
`count_chars.phpt`, `count_chars_basic.phpt`, `levenshtein.phpt`,
`levenshtein_bug_16473.phpt`, `levenshtein_bug_6562.phpt`,
`levenshtein_bug_7368.phpt`, `levenshtein_error_conditions.phpt`,
`quoted_printable_encode_001.phpt`, `str_word_count.phpt`, and
`str_word_count1.phpt`.

Broad manifest note: current deterministic broad tier-1000 and tier-5000
generated from corpus revision `8c63ec400ce8e07c57a8d9499317b96a8beafb8b`
contain zero `ext/standard/tests/strings/*` rows. The matching target rows
first appear in
`.runtime/ptn-almd-broad-before/20260615T042822Z/phpt-baseline-10000.txt`.
The attempted full tier-1000 before run
`PHPT_PROGRESS_DIR=.runtime/ptn-almd-broad-before-progress timeout 1800s tools/run-phpt-baseline.sh --tier 1000 --out-dir .runtime/ptn-almd-broad-before`
timed out while executing the Zend bucket, after writing classification
evidence for 1,000 selected rows, 598 runnable, and 402 classified.

## 2026-06-15 ptn-texo Magic Object/Class Metadata Row Pack

Final checked-in focused manifest:
`tools/phpt-ptn-texo-magic-object-metadata-row-pack.txt`.

Implemented behavior: generated class metadata now wires direct `__get()`,
`__set()`, `__unset()`, and `__debugInfo()` handlers into the native runtime.
Declared object property metadata tracks explicitly unset declared slots so
subsequent direct reads, writes, and unsets can invoke the PHP magic hooks
where PHP does. By-reference assignment to overloaded properties now throws the
overloaded-object reference error through a side-effect-free `__get` existence
probe. Top-level `var_dump($object)` uses `__debugInfo()` when present, and
unset of asymmetric set-visibility properties emits the unset-specific
diagnostic instead of the generic modify diagnostic.

Classifier behavior: direct `__get`, `__set`, `__unset`, and `__debugInfo`
rows are no longer blocked as unsupported magic method metadata. Heavier magic
metadata surfaces such as `__isset`, `__callStatic`, `__serialize`,
`__unserialize`, `__sleep`, and `__wakeup` remain classified unless they are in
an already-supported path.

Focused raw baseline on hook-start base `534139bfe4b2` with classification
disabled:
`PTN_PHPT_CLASSIFY=0 PHPT_PROGRESS_DIR=.runtime/ptn-texo-magic-raw-before tools/run-bounded-phpt.sh --classify-harness-programs /tmp/ptn-texo-before-guard463/.runtime/phpt-progress/excluded-20260615T030609Z/unsupported-magic-method-metadata.txt`.
Artifact `.runtime/ptn-texo-magic-raw-before/summary-20260615T031547Z.txt`
selected 6 rows: 6 runnable, 0 passed, 6 failed.

Final focused evidence after rebasing onto current `origin/master`:
`PHPT_PROGRESS_DIR=.runtime/ptn-texo-magic-focused-final-commit tools/run-bounded-phpt.sh --classify-harness-programs tools/phpt-ptn-texo-magic-object-metadata-row-pack.txt`.
Artifact
`.runtime/ptn-texo-magic-focused-final-commit/summary-20260615T040132Z.txt`
selected 6 rows: 6 runnable, 4 passed, 2 failed.

Final focused passing rows:

- `Zend/tests/__debugInfo_reference.phpt`
- `Zend/tests/assign_ref_to_overloaded_prop.phpt`
- `Zend/tests/asymmetric_visibility/__set.phpt`
- `Zend/tests/asymmetric_visibility/__unset.phpt`

Remaining focused failures:

- `Zend/tests/asymmetric_visibility/bug001.phpt`
- `Zend/tests/asymmetric_visibility/bug002.phpt`

The remaining two rows now reach the modeled magic/set-visibility path but
still miss typed declared-property dump metadata (`uninitialized(int)`) in
`var_dump($object)`, so they remain typed-property metadata frontier work. A
post-rebase full broad 1k run was not completed for this row pack.

## 2026-06-15 ptn-hgfn Object/Class Metadata Trace Row Pack

Final manifest: `tools/phpt-ptn-hgfn-object-class-metadata-row-pack-manifest.txt`.

Implemented behavior: uncaught exception stack traces emitted from declared
instance-method calls now preserve the runtime call-site file/line chain and
display generated instance method frames as `Class->method()`. The parser and
native compiler continue to accept declared private/protected/static method
metadata generically; unsupported magic, property-visibility, diagnostics, and
dynamic-dispatch rows remain classified out instead of being shaped to expected
output.

Focused PHPT command on the final rebased branch:
`PHPT_PROGRESS_DIR=.runtime/ptn-hgfn-focused-final-current-tip tools/run-bounded-phpt.sh tools/phpt-ptn-hgfn-object-class-metadata-row-pack-manifest.txt`.
It selected 20 rows, kept 11 runnable, passed 11, and classified 9 rows
(`unsupported-dynamic-member-dispatch` 1, `unsupported-diagnostics-runtime` 1,
`unsupported-magic-method-metadata` 6, and
`unsupported-property-visibility-metadata` 1).

Completed full broad 1k evidence used corpus revision
`8c63ec400ce8e07c57a8d9499317b96a8beafb8b`:

- Before on hook-start commit `52beba7e2a24`:
  `.runtime/ptn-hgfn-broad-before-progress/summary-20260614T230939Z.txt`
  selected 1,000 rows, with 563 runnable and 437 classified; 499 passed and
  64 failed.
- After on integrated commit `82ed04dfe1c8`, before later master fast-forwards:
  `.runtime/ptn-hgfn-broad-after-integrated-progress/summary-20260615T011823Z.txt`
  selected 1,000 rows, with 591 runnable and 409 classified; 535 passed and
  56 failed.

The normalized broad pass-set comparison had 36 newly passing rows and no
no-longer-passing rows:

- `Zend/tests/access_modifiers/access_modifiers_008.phpt`
- `Zend/tests/access_modifiers/access_modifiers_009.phpt`
- `Zend/tests/access_modifiers/access_modifiers_010.phpt`
- `Zend/tests/array_literal_next_element_error.phpt`
- `Zend/tests/array_merge_recursive_next_key_overflow.phpt`
- `Zend/tests/assign_op_type_error.phpt`
- `Zend/tests/assign_to_obj_002.phpt`
- `Zend/tests/ast/ast_serialize_backtick_literal.phpt`
- `Zend/tests/ast/ast_serialize_floats.phpt`
- `Zend/tests/bug21888.phpt`
- `Zend/tests/bug29210.phpt`
- `ext/standard/tests/array/array_diff_key_variation1.phpt`
- `ext/standard/tests/array/array_diff_key_variation2.phpt`
- `ext/standard/tests/array/array_diff_uassoc_variation1.phpt`
- `ext/standard/tests/array/array_diff_uassoc_variation2.phpt`
- `ext/standard/tests/array/array_diff_ukey_variation1.phpt`
- `ext/standard/tests/array/array_diff_ukey_variation2.phpt`
- `ext/standard/tests/array/array_intersect_key_variation1.phpt`
- `ext/standard/tests/array/array_intersect_key_variation2.phpt`
- `ext/standard/tests/array/array_intersect_uassoc_variation1.phpt`
- `ext/standard/tests/array/array_intersect_uassoc_variation2.phpt`
- `ext/standard/tests/array/array_intersect_ukey_variation1.phpt`
- `ext/standard/tests/array/array_intersect_ukey_variation2.phpt`
- `ext/standard/tests/array/array_map_variation4.phpt`
- `ext/standard/tests/array/array_map_variation5.phpt`
- `ext/standard/tests/array/array_udiff_assoc_variation1.phpt`
- `ext/standard/tests/array/array_udiff_assoc_variation2.phpt`
- `ext/standard/tests/array/array_udiff_uassoc_variation1.phpt`
- `ext/standard/tests/array/array_udiff_uassoc_variation2.phpt`
- `ext/standard/tests/array/array_udiff_variation1.phpt`
- `ext/standard/tests/array/array_udiff_variation2.phpt`
- `ext/standard/tests/array/array_uintersect_assoc_variation1.phpt`
- `ext/standard/tests/array/array_uintersect_assoc_variation2.phpt`
- `ext/standard/tests/array/array_uintersect_uassoc_variation1.phpt`
- `ext/standard/tests/array/array_uintersect_uassoc_variation2.phpt`
- `ext/standard/tests/array/array_uintersect_variation1.phpt`

After later master fast-forwards, the task diff rebased cleanly. Targeted Rust
checks and the focused PHPT pack were rerun on the final rebased branch; a latest-tip
classify-only broad probe recorded 592 runnable / 408 classified, with
`Zend/tests/bug28442.phpt` and `Zend/tests/bug30140.phpt` newly runnable versus
the completed full broad run and both passing in
`.runtime/ptn-hgfn-latest-new-runnable-progress/summary-20260615T031142Z.txt`.

## 2026-06-15 ptn-dgj9 Reference Lvalue Array Path Row Pack

Final manifest:
`tools/phpt-ptn-dgj9-reference-lvalue-row-pack-manifest.txt`.

Implemented behavior: reference lvalues can now be built through nested array
paths rooted at variables, object properties, and `ArrayAccess` receivers. The
runtime preserves `PTN_REFERENCE` values returned by `offsetGet()`, descends
nested `ArrayAccess` paths through live references, binds property-array-dim
reference targets, and handles quiet ArrayAccess lookup by checking
`offsetExists()` before `offsetGet()`. The parser/compiler now accepts
property-array reference targets and dynamic/function/method call results as
reference-assignment sources so the runtime can emit PHP-compatible fallback
diagnostics for by-value call results.

The backend also uses declared method metadata for exact receiver classes when
emitting append arguments, so `foo($a[])` uses by-reference append semantics
only for declared by-reference parameters while preserving virtual dispatch
fallback when subclasses may override the method.

Broad 1k before/after evidence was collected on active base
`0ab08a8fe20b` before `origin/master` advanced during the run:

- Before:
  `/tmp/ptn-dgj9-before-0ab08-guard463/.runtime/phpt-progress/summary-20260615T003402Z.txt`
  selected 1,000 rows: 591 runnable, 409 classified, 414 passed, 177 failed.
- After:
  `.runtime/phpt-progress/summary-20260615T003439Z.txt` at commit
  `200b0760078a` selected the same 1,000 rows: 591 runnable, 409 classified,
  418 passed, 173 failed.

Newly passing broad rows: 4. No broad pass-to-fail regressions were found by
log set comparison. This is below the target >=10 and stretch >=25.

- `Zend/tests/ArrayAccess/ArrayAccess_indirect_append.phpt`
- `Zend/tests/ArrayAccess/bug71731.phpt`
- `Zend/tests/bug34137.phpt`
- `Zend/tests/bug35163_3.phpt`

After the broad run, the branch was rebased cleanly onto current `origin/master`
`94d8b8cf9`. Final focused evidence:
`PHPT_PROGRESS_DIR=.runtime/ptn-dgj9-reference-lvalue-final tools/run-bounded-phpt.sh --classify-harness-programs tools/phpt-ptn-dgj9-reference-lvalue-row-pack-manifest.txt`.
Artifact `.runtime/ptn-dgj9-reference-lvalue-final/summary-20260615T024130Z.txt`
selected 22 rows: 21 runnable, 1 classified
(`unsupported-dynamic-member-dispatch`), 8 passed, and 13 failed.

Final focused passing rows:

- `Zend/tests/ArrayAccess/ArrayAccess_indirect_append.phpt`
- `Zend/tests/ArrayAccess/bug71731.phpt`
- `Zend/tests/assign_op_type_error.phpt`
- `Zend/tests/assign_to_obj_002.phpt`
- `Zend/tests/asymmetric_visibility/reference.phpt`
- `Zend/tests/asymmetric_visibility/reference_2.phpt`
- `Zend/tests/bug34137.phpt`
- `Zend/tests/bug35163_3.phpt`

Remaining focused failures are object/visibility/diagnostic frontiers:
`ArrayObject` availability, assign-into-object/property null behavior,
asymmetric visibility write checks, `bug31098` quiet wrong-type lookup
diagnostics, `bug31525` source-path formatting, `bug34064` fatal output
formatting, dead-object property assignment, and error-control string-offset
diagnostics.

## 2026-06-15 ptn-ppri Broad Standard String/Array Row Pack

Final checked-in focused manifest:
`tools/phpt-ptn-ppri-standard-string-array-row-pack.txt`.

Focused verification on the final rebased branch `97e806bdffa0`:
`PHPT_PROGRESS_DIR=.runtime/ptn-ppri-focused-final-rebased tools/run-bounded-phpt.sh --classify-harness-programs tools/phpt-ptn-ppri-standard-string-array-row-pack.txt`.
Artifact `.runtime/ptn-ppri-focused-final-rebased/summary-20260615T021843Z.txt`
selected 20 rows: 18 runnable, 18 passed, 0 failed, 2 classified
(`harness-skipif` 1, `sapi-behavior` 1).

Broad 1k controlling-manifest evidence used the deterministic tier-1000
manifest for PHP corpus revision `8c63ec400ce8e07c57a8d9499317b96a8beafb8b`:

- Before on hook-start current base `0ab08a8fe20b`:
  `.runtime/ptn-ppri-broad-before-current-progress/summary-20260615T002207Z.txt`
  selected 1,000 rows, with 591 runnable, 409 classified, 414 passed, and
  177 failed.
- After on implementation commit `011964bcfc57`:
  `.runtime/ptn-ppri-broad-after-current-progress/summary-20260615T002851Z.txt`
  selected the same 1,000 rows, with 591 runnable, 409 classified, 528 passed,
  and 63 failed. The pass-set comparison found 114 newly passing rows and 0
  pass-set regressions.

`origin/master` advanced while the broad runs were executing. The final branch
was rebased cleanly to `97e806bdffa0` and reran the focused pack above; a full
post-rebase broad rerun was not repeated.

Newly passing broad rows:

- `Zend/tests/add_002.phpt`
- `Zend/tests/add_003.phpt`
- `Zend/tests/add_004.phpt`
- `Zend/tests/add_006.phpt`
- `Zend/tests/add_007.phpt`
- `Zend/tests/arg_unpack/dynamic.phpt`
- `Zend/tests/arg_unpack/internal.phpt`
- `Zend/tests/arg_unpack/invalid_type.phpt`
- `Zend/tests/array_unpack/already_occupied.phpt`
- `Zend/tests/arrow_functions/001.phpt`
- `Zend/tests/arrow_functions/002.phpt`
- `Zend/tests/arrow_functions/003.phpt`
- `Zend/tests/arrow_functions/004.phpt`
- `Zend/tests/assert/expect_003.phpt`
- `Zend/tests/assert/expect_004.phpt`
- `Zend/tests/assert/expect_005.phpt`
- `Zend/tests/assert/expect_016.phpt`
- `Zend/tests/assert/expect_empty_stmt_bug.phpt`
- `Zend/tests/assign_ref_error_var_handling.phpt`
- `Zend/tests/ast/zend-pow-assign.phpt`
- `Zend/tests/asymmetric_visibility/reference_2.phpt`
- `Zend/tests/bug31720.phpt`
- `Zend/tests/bug38808.phpt`
- `ext/standard/tests/array/001.phpt`
- `ext/standard/tests/array/array_change_key_case_flag_error.phpt`
- `ext/standard/tests/array/array_chunk2.phpt`
- `ext/standard/tests/array/array_chunk_variation5.phpt`
- `ext/standard/tests/array/array_column_scalar_index_strict_types.phpt`
- `ext/standard/tests/array/array_column_scalar_index_weak_types.phpt`
- `ext/standard/tests/array/array_combine_error2.phpt`
- `ext/standard/tests/array/array_diff_1.phpt`
- `ext/standard/tests/array/array_diff_key.phpt`
- `ext/standard/tests/array/array_diff_leak_custom_type_checks.phpt`
- `ext/standard/tests/array/array_diff_single_array.phpt`
- `ext/standard/tests/array/array_diff_uassoc_basic.phpt`
- `ext/standard/tests/array/array_diff_uassoc_error.phpt`
- `ext/standard/tests/array/array_diff_uassoc_variation11.phpt`
- `ext/standard/tests/array/array_diff_uassoc_variation13.phpt`
- `ext/standard/tests/array/array_diff_uassoc_variation5.phpt`
- `ext/standard/tests/array/array_diff_uassoc_variation6.phpt`
- `ext/standard/tests/array/array_diff_uassoc_variation7.phpt`
- `ext/standard/tests/array/array_diff_uassoc_variation8.phpt`
- `ext/standard/tests/array/array_diff_uassoc_variation9.phpt`
- `ext/standard/tests/array/array_diff_ukey_basic.phpt`
- `ext/standard/tests/array/array_diff_ukey_variation10.phpt`
- `ext/standard/tests/array/array_diff_ukey_variation5.phpt`
- `ext/standard/tests/array/array_diff_ukey_variation6.phpt`
- `ext/standard/tests/array/array_diff_ukey_variation8.phpt`
- `ext/standard/tests/array/array_fill_error.phpt`
- `ext/standard/tests/array/array_fill_variation6.phpt`
- `ext/standard/tests/array/array_filter.phpt`
- `ext/standard/tests/array/array_filter_basic.phpt`
- `ext/standard/tests/array/array_filter_invalid_mode.phpt`
- `ext/standard/tests/array/array_filter_variation10.phpt`
- `ext/standard/tests/array/array_filter_variation3.phpt`
- `ext/standard/tests/array/array_filter_variation4.phpt`
- `ext/standard/tests/array/array_filter_variation5.phpt`
- `ext/standard/tests/array/array_filter_variation6.phpt`
- `ext/standard/tests/array/array_filter_variation7.phpt`
- `ext/standard/tests/array/array_filter_variation8.phpt`
- `ext/standard/tests/array/array_filter_variation9.phpt`
- `ext/standard/tests/array/array_find_types.phpt`
- `ext/standard/tests/array/array_intersect_key.phpt`
- `ext/standard/tests/array/array_intersect_uassoc_basic.phpt`
- `ext/standard/tests/array/array_intersect_uassoc_variation5.phpt`
- `ext/standard/tests/array/array_intersect_uassoc_variation6.phpt`
- `ext/standard/tests/array/array_intersect_uassoc_variation7.phpt`
- `ext/standard/tests/array/array_intersect_uassoc_variation8.phpt`
- `ext/standard/tests/array/array_intersect_ukey_basic.phpt`
- `ext/standard/tests/array/array_intersect_ukey_variation5.phpt`
- `ext/standard/tests/array/array_intersect_ukey_variation6.phpt`
- `ext/standard/tests/array/array_intersect_ukey_variation7.phpt`
- `ext/standard/tests/array/array_intersect_ukey_variation8.phpt`
- `ext/standard/tests/array/array_key_exists.phpt`
- `ext/standard/tests/array/array_key_exists_variation3.phpt`
- `ext/standard/tests/array/array_map_001.phpt`
- `ext/standard/tests/array/array_map_basic.phpt`
- `ext/standard/tests/array/array_map_error.phpt`
- `ext/standard/tests/array/array_map_variation1.phpt`
- `ext/standard/tests/array/array_map_variation10.phpt`
- `ext/standard/tests/array/array_map_variation11.phpt`
- `ext/standard/tests/array/array_map_variation12.phpt`
- `ext/standard/tests/array/array_map_variation13.phpt`
- `ext/standard/tests/array/array_map_variation14.phpt`
- `ext/standard/tests/array/array_map_variation15.phpt`
- `ext/standard/tests/array/array_map_variation16.phpt`
- `ext/standard/tests/array/array_map_variation19.phpt`
- `ext/standard/tests/array/array_map_variation2.phpt`
- `ext/standard/tests/array/array_map_variation3.phpt`
- `ext/standard/tests/array/array_map_variation6.phpt`
- `ext/standard/tests/array/array_map_variation7.phpt`
- `ext/standard/tests/array/array_map_variation8.phpt`
- `ext/standard/tests/array/array_map_variation9.phpt`
- `ext/standard/tests/array/array_pad_too_large_padding.phpt`
- `ext/standard/tests/array/array_product_empty_array.phpt`
- `ext/standard/tests/array/array_product_variation5.phpt`
- `ext/standard/tests/array/array_push_error2.phpt`
- `ext/standard/tests/array/array_rand.phpt`
- `ext/standard/tests/array/array_rand_variation5.phpt`
- `ext/standard/tests/array/array_reduce.phpt`
- `ext/standard/tests/array/array_reduce_accumulator_refcount.phpt`
- `ext/standard/tests/array/array_reduce_return_by_ref.phpt`
- `ext/standard/tests/array/array_reduce_variation1.phpt`
- `ext/standard/tests/array/array_replace.phpt`
- `ext/standard/tests/array/array_slice_variation1.phpt`
- `ext/standard/tests/array/array_sum_empty_array.phpt`
- `ext/standard/tests/array/array_sum_variation8.phpt`
- `ext/standard/tests/array/array_udiff_assoc_variation.phpt`
- `ext/standard/tests/array/array_udiff_assoc_variation5.phpt`
- `ext/standard/tests/array/array_udiff_uassoc_variation6.phpt`
- `ext/standard/tests/array/array_udiff_variation5.phpt`
- `ext/standard/tests/array/array_uintersect_assoc_basic2.phpt`
- `ext/standard/tests/array/array_uintersect_assoc_variation5.phpt`
- `ext/standard/tests/array/array_uintersect_uassoc_variation6.phpt`

Implemented behavior: the runtime now exposes `PHP_MAXPATHLEN`, implements
`php_strip_whitespace()`, preserves `addcslashes()` invalid range warnings,
accepts braced scalar object member names in the parser, and tightens standard
array helper diagnostics for invalid callbacks, array key conversion warning
spacing, and `array_merge()` / `array_merge_recursive()` non-array operands.
The generated C backend also inherits the current method-scope helper warning
cleanup from upstream.

## 2026-06-15 ptn-sq9m Object/Class Metadata Row Pack

Final manifest:
`tools/phpt-ptn-sq9m-object-class-metadata-row-pack.txt`.

Broad 1k classifier KPI used the deterministic tier-1000 manifest from
`.runtime/ptn-sq9m-broad-before-classify/20260615T020405Z/phpt-baseline-1000.txt`
against PHP source corpus revision `8c63ec400ce8e07c57a8d9499317b96a8beafb8b`:

- Before on hook-start base `5f0af40d45de`,
  `.runtime/ptn-sq9m-broad-before-classify-progress/summary-20260615T020406Z.txt`
  selected 1,000 rows: 592 runnable and 408 classified.
- After,
  `.runtime/ptn-sq9m-broad-after-uv4c-merge-classify-progress/summary-20260615T045615Z.txt`
  selected the same 1,000 rows: 611 runnable and 389 classified.

Pass-count evidence is the focused row pack:

- Hook-start base artifact
  `/tmp/ptn-sq9m-final-row-pack-base/summary-20260615T035605Z.txt`
  selected 10 rows: 4 runnable, 6 classified, 0 passed, 4 failed.
- Current artifact
  `.runtime/ptn-sq9m-final-row-pack-after-uv4c-merge/summary-20260615T045417Z.txt`
  selected 10 rows: 10 runnable, 10 passed, 0 failed.

Newly passing broad 1k rows:

- `Zend/tests/asymmetric_visibility/__set.phpt`
- `Zend/tests/asymmetric_visibility/__unset.phpt`
- `Zend/tests/asymmetric_visibility/bug001.phpt`
- `Zend/tests/asymmetric_visibility/bug002.phpt`
- `Zend/tests/asymmetric_visibility/bug003.phpt`
- `Zend/tests/asymmetric_visibility/dim_add.phpt`
- `Zend/tests/asymmetric_visibility/unset.phpt`
- `Zend/tests/asymmetric_visibility/unshared_rw_cache_slot.phpt`
- `ext/standard/tests/array/array_filter_object.phpt`
- `ext/standard/tests/array/array_map_object1.phpt`

Final focused command:
`PHPT_PROGRESS_DIR=.runtime/ptn-sq9m-final-row-pack-after-uv4c-merge timeout 900s tools/run-bounded-phpt.sh --classify-harness-programs tools/phpt-ptn-sq9m-object-class-metadata-row-pack.txt`.

Implemented behavior: object property metadata now records typed declared
properties, their display type, and explicit unset state. Typed properties
without defaults remain uninitialized instead of being initialized to `NULL`;
reads and dumps report uninitialized typed properties; asymmetric set
visibility distinguishes never-initialized properties from explicitly unset
properties when deciding whether `__set()`/`__unset()` should run; and
property array-dimension write-backs use the indirect set-visibility diagnostic.
Magic property dispatch wrappers restore the recursion guard before rethrowing
exceptions from magic methods, so caught magic-method failures do not suppress
later `__set()`/`__unset()` dispatch.
The classifier now admits the supported asymmetric `__set()`/`__unset()` rows.
The row pack also includes the final-method/callback object rows fixed in this
branch.

## 2026-06-15 ptn-vq7w Broad COW/Reference Row Pack

Final checked-in broad COW/reference manifest:
`tools/phpt-ptn-vq7w-broad-cow-reference-row-pack.txt`.

Broad 1k controlling-manifest evidence, using rows selected from the
deterministic tier-1000 manifest for PHP corpus revision
`8c63ec400ce8e07c57a8d9499317b96a8beafb8b`:

- Before, `.runtime/ptn-vq7w-current-broad-cow-probe/summary-20260615T011247Z.txt`
  selected 24 broad COW/reference rows: 24 runnable, 23 passed, 1 failed. The
  failed broad row was `Zend/tests/array_splice_empty_ht_iter_removal.phpt`.
- After merging current `origin/master`,
  `.runtime/ptn-vq7w-current-broad-cow-probe-final/summary-20260615T013730Z.txt`
  selected the same 24 rows: 24 runnable, 24 passed, 0 failed. The expanded
  broad array/reference slice
  `.runtime/ptn-vq7w-broad-array-cluster-final/summary-20260615T013008Z.txt`
  selected 25 rows and passed 25/25.

Effective broad 1k pass-count movement is 479 -> 480 from the exact selected
row evidence. This is +1 broad row, below the target >=10; no classifier-only
rows were counted as pass-count progress.

Newly passing broad 1k row:

- `Zend/tests/array_splice_empty_ht_iter_removal.phpt`

Final focused command:
`PHPT_PROGRESS_DIR=.runtime/ptn-vq7w-current-broad-cow-probe-final timeout 900 tools/run-bounded-phpt.sh --classify-harness-programs .runtime/ptn-vq7w-current-broad-cow-probe/manifest-20260615T011247Z.txt`.

Implemented behavior: by-reference foreach iterators created from variables now
watch the source slot and refresh their retained array if the iterated variable
is replaced during iteration. This preserves PHP's iterator behavior when an
array operation such as `array_splice()` replaces the iterated variable while
the by-reference loop is still live.

Additional integrated behavior outside the deterministic broad 1k:
`usort()`, `uasort()`, and `uksort()` are now modeled as by-reference array
mutators with user callback comparison. Before,
`.runtime/ptn-vq7w-array-internal-before/summary-20260615T005410Z.txt` had the
user-comparator-sort bucket at 26 selected / 0 runnable. Final integrated
evidence in
`.runtime/ptn-vq7w-user-comparator-sort-final/summary-20260615T013008Z.txt`
selected 26 rows: 26 runnable, 20 passed, and 6 failed. The remaining failures
cover object comparison ordering, boolean-comparator diagnostics, and nested
array comparison semantics.

## 2026-06-15 ptn-m8pk Ext-Standard String Helpers

Final manifest: `tools/phpt-ptn-m8pk-ext-standard-strings-row-pack.txt`.

Focused baseline artifact `.runtime/ptn-m8pk-candidate-before/summary-20260615T013412Z.txt`
selected the original 22-row candidate pack: 22 runnable, 0 passed, and 22
failed. The final checked-in manifest removes the parser-blocked full
`substr_replace.phpt` matrix and keeps the 21 executable rows from that pack.

Final focused command:
`PHPT_PROGRESS_DIR=.runtime/ptn-m8pk-focused-final-head timeout 900s tools/run-bounded-phpt.sh --classify-harness-programs tools/phpt-ptn-m8pk-ext-standard-strings-row-pack.txt`.
Artifact `.runtime/ptn-m8pk-focused-final-head/summary-20260615T022052Z.txt`
selected 21 rows: 21 runnable, 21 passed, 0 failed, 0 skipped, and 0 warned.

Newly passing focused rows:

- `ext/standard/tests/strings/htmlspecialchars.phpt`
- `ext/standard/tests/strings/htmlspecialchars_basic.phpt`
- `ext/standard/tests/strings/htmlspecialchars_decode_basic.phpt`
- `ext/standard/tests/strings/htmlspecialchars_decode_variation3.phpt`
- `ext/standard/tests/strings/htmlspecialchars_decode_variation4.phpt`
- `ext/standard/tests/strings/htmlspecialchars_decode_variation5.phpt`
- `ext/standard/tests/strings/htmlspecialchars_decode_variation6.phpt`
- `ext/standard/tests/strings/htmlspecialchars_decode_variation7.phpt`
- `ext/standard/tests/strings/number_format_basic.phpt`
- `ext/standard/tests/strings/substr_replace_array.phpt`
- `ext/standard/tests/strings/substr_replace_array_unset.phpt`
- `ext/standard/tests/strings/substr_replace_error.phpt`
- `ext/standard/tests/strings/ucwords_basic.phpt`
- `ext/standard/tests/strings/ucwords_variation2.phpt`
- `ext/standard/tests/strings/ucwords_variation3.phpt`
- `ext/standard/tests/strings/ucwords_variation4.phpt`
- `ext/standard/tests/strings/ucwords_variation5.phpt`
- `ext/standard/tests/strings/wordwrap.phpt`
- `ext/standard/tests/strings/wordwrap_basic.phpt`
- `ext/standard/tests/strings/wordwrap_error.phpt`
- `ext/standard/tests/strings/wordwrap_variation5.phpt`

Implemented behavior: runtime support for `htmlspecialchars()`,
`htmlspecialchars_decode()`, `ucwords()`, `wordwrap()`, `number_format()`,
`substr_replace()` scalar/array splice cases, and `decbin()`. The HTML constants
`HTML_SPECIALCHARS`, `HTML_ENTITIES`, `ENT_*`, and document-type flags now
resolve through `defined()`/`constant()` as builtin constants.

Broad 1k note: the deterministic tier-1000 manifest generated at
`.runtime/ptn-m8pk-broad1k-before-generate/20260615T012647Z/phpt-baseline-1000.txt`
contains 0 `ext/standard/tests/strings/*` rows. The hook-start full broad run
timed out after 175/205 runnable rows without writing a summary artifact, and
the after classify-only attempt in `.runtime/ptn-m8pk-broad1k-after-classify`
also timed out. The generated before/after tier-1000 manifests both contain 0
standard string rows, so the branch's measurable red-to-green evidence is the
focused 21-row ext-standard string pack above.

## 2026-06-15 ptn-cfny Parser/Control-Flow Row Pack

Final manifest: `tools/phpt-ptn-cfny-parser-asym-row-pack.txt`.

Broad 1k before artifact `.runtime/phpt-progress/summary-20260615T004347Z.txt`
selected 1,000 rows: 591 runnable and 409 classified. The command completed
the Zend bucket and timed out during the standard bucket; completed Zend-bucket
evidence was 155 passed and 49 failed out of 204 runnable rows.

Broad 1k after artifact `.runtime/phpt-progress/summary-20260615T015146Z.txt`
selected the same 1,000 rows: 591 runnable and 409 classified. It completed
the Zend bucket at 162 passed and 42 failed out of 204 runnable rows, then
timed out during the standard bucket. Current-branch focused probes confirmed
the three additional broad Zend rows added after that completed Zend bucket:
`.runtime/ptn-cfny-extra2/summary-20260615T024040Z.txt` passed
`bug27669`/`bug29104` 2/2, and
`.runtime/ptn-cfny-binary-after/summary-20260615T024909Z.txt` passed
`binary.phpt` 1/1.

Focused final command on the rebased branch:
`PHPT_PROGRESS_DIR=.runtime/ptn-cfny-focused-final-rebased2 tools/run-bounded-phpt.sh --classify-harness-programs tools/phpt-ptn-cfny-parser-asym-row-pack.txt`.
Artifact `.runtime/ptn-cfny-focused-final-rebased2/summary-20260615T030738Z.txt`
selected 34 rows: 34 runnable, 31 passed, and 3 failed.

Newly passing broad rows confirmed for this pack:

- `Zend/tests/ast/gh21072.phpt`
- `Zend/tests/asymmetric_visibility/ast_printing.phpt`
- `Zend/tests/asymmetric_visibility/bug003.phpt`
- `Zend/tests/asymmetric_visibility/dim_add.phpt`
- `Zend/tests/asymmetric_visibility/unset.phpt`
- `Zend/tests/asymmetric_visibility/virtual_get_only.phpt`
- `Zend/tests/asymmetric_visibility/virtual_set_only.phpt`
- `Zend/tests/binary.phpt`
- `Zend/tests/bug27669.phpt`
- `Zend/tests/bug29104.phpt`

Implemented behavior: local class declarations retain source text for later
assertion rendering while still hoisting class metadata, dynamic instance and
static method calls lower through runtime method dispatch, nested named
function declarations become runtime-gated declarations visible to
`function_exists()` only after execution, asymmetric property unset and
array-dim writes use set-visibility-aware storage resolution, virtual
get-only/set-only asymmetric property hooks report parser fatals, and oversized
binary integer literals use PHP's binary-overflow float boundary.

Remaining focused failures are `scope_rebinding` for `Closure::bindTo()` scope
rebinding, `unshared_rw_cache_slot`, and `variation` for the broader remaining
typed/uninitialized asymmetric property runtime behavior.

## 2026-06-14 ptn-wxno Object-String Array Helper Classifier Unblock

Final manifest: `tools/phpt-ptn-wxno-object-string-array-helper-row-pack.txt`.

Broad 1k classifier KPI used the deterministic tier-1000 manifest against the
same PHP source corpus revision `8c63ec400ce8e07c57a8d9499317b96a8beafb8b`:

- Before on the task base `b39bb4fbe0ec`:
  `.runtime/ptn-wxno-broad-before-progress/summary-20260614T220518Z.txt`
  selected 1,000 rows, with 563 runnable and 437 classified. The blocked set
  included 25 `unsupported-object-string-conversion-metadata` rows.
- After rebasing on current `origin/master`:
  `.runtime/ptn-wxno-broad-after-rebase-classify-progress/summary-20260614T224926Z.txt`
  selected 1,000 rows, with 588 runnable and 412 classified. This broad run was
  classify-only; pass-count evidence is the focused PHPT pack below.

The exact broad rows newly moved into runnable status are:

- `ext/standard/tests/array/array_diff_key_variation1.phpt`
- `ext/standard/tests/array/array_diff_key_variation2.phpt`
- `ext/standard/tests/array/array_diff_uassoc_variation1.phpt`
- `ext/standard/tests/array/array_diff_uassoc_variation2.phpt`
- `ext/standard/tests/array/array_diff_ukey_variation1.phpt`
- `ext/standard/tests/array/array_diff_ukey_variation2.phpt`
- `ext/standard/tests/array/array_intersect_key_variation1.phpt`
- `ext/standard/tests/array/array_intersect_key_variation2.phpt`
- `ext/standard/tests/array/array_intersect_uassoc_variation1.phpt`
- `ext/standard/tests/array/array_intersect_uassoc_variation2.phpt`
- `ext/standard/tests/array/array_intersect_ukey_variation1.phpt`
- `ext/standard/tests/array/array_intersect_ukey_variation2.phpt`
- `ext/standard/tests/array/array_map_variation4.phpt`
- `ext/standard/tests/array/array_map_variation5.phpt`
- `ext/standard/tests/array/array_udiff_assoc_variation1.phpt`
- `ext/standard/tests/array/array_udiff_assoc_variation2.phpt`
- `ext/standard/tests/array/array_udiff_uassoc_variation1.phpt`
- `ext/standard/tests/array/array_udiff_uassoc_variation2.phpt`
- `ext/standard/tests/array/array_udiff_variation1.phpt`
- `ext/standard/tests/array/array_udiff_variation2.phpt`
- `ext/standard/tests/array/array_uintersect_assoc_variation1.phpt`
- `ext/standard/tests/array/array_uintersect_assoc_variation2.phpt`
- `ext/standard/tests/array/array_uintersect_uassoc_variation1.phpt`
- `ext/standard/tests/array/array_uintersect_uassoc_variation2.phpt`
- `ext/standard/tests/array/array_uintersect_variation1.phpt`

Final focused command:
`PHPT_PROGRESS_DIR=.runtime/ptn-wxno-object-string-final-rebased timeout 600s tools/run-bounded-phpt.sh --classify-harness-programs tools/phpt-ptn-wxno-object-string-array-helper-row-pack.txt`.
Artifact `.runtime/ptn-wxno-object-string-final-rebased/summary-20260614T225647Z.txt`
selected 25 rows: 25 runnable, 25 passed, 0 failed, 0 skipped, and 0 warned.

Implemented behavior: the PHPT classifier no longer blocks public `__toString()`
coverage through the modeled array key/comparator helper and `array_map()`
callback surfaces. The runtime also now validates `__toString()` return values:
weak-mode scalar int/float/bool returns still coerce to string, strict mode
rejects non-string scalar returns, and null/array/resource/object returns throw
PHP-style `TypeError` messages rather than being silently converted.

## 2026-06-14 ptn-jlzj Broad Control-Flow/Call-Unpack Row Pack

Final manifest:
`tools/phpt-ptn-jlzj-call-unpack-break-row-pack-manifest.txt`.

Focused execution after the final rebase selected 26 rows: 16 runnable,
16 passed, 0 failed, 10 classified
(`unsupported-generator-runtime` 4,
`unsupported-call-unpacking-reference` 3,
`unsupported-call-unpacking-traversable` 1,
`unsupported-resource-limit` 2). Command:
`tools/run-bounded-phpt.sh tools/phpt-ptn-jlzj-call-unpack-break-row-pack-manifest.txt`
(`.runtime/phpt-progress/summary-20260614T225156Z.txt`).

Broad 1k before at hook-start base `df6e3157a19e` selected 1,000 rows:
533 runnable, 453 passed, 80 failed, 467 classified
(`.runtime/ptn-jlzj-broad-before-progress-run/summary-20260614T211623Z.txt`).
Broad 1k after on the measured branch state selected the same 1,000-row tier
shape: 558 runnable, 482 passed, 76 failed, 442 classified
(`.runtime/phpt-progress/summary-20260614T193710Z.txt`). There were no pass
regressions.

Newly passing broad rows:

- `Zend/tests/ArrayAccess/bug33710.phpt`
- `Zend/tests/arg_unpack/dynamic.phpt`
- `Zend/tests/arg_unpack/internal.phpt`
- `Zend/tests/arg_unpack/invalid_type.phpt`
- `Zend/tests/arg_unpack/many_args.phpt`
- `Zend/tests/arg_unpack/method.phpt`
- `Zend/tests/arg_unpack/new.phpt`
- `Zend/tests/arg_unpack/positional_arg_after_unpack_error.phpt`
- `Zend/tests/break_error_001.phpt`
- `Zend/tests/break_error_002.phpt`
- `Zend/tests/break_error_003.phpt`
- `Zend/tests/break_error_004.phpt`
- `Zend/tests/bug26010.phpt`
- `Zend/tests/bug27798.phpt`
- `Zend/tests/bug35509.phpt`
- `ext/standard/tests/array/007.phpt`
- `ext/standard/tests/array/array_column_property_visibility.phpt`
- `ext/standard/tests/array/array_diff_uassoc_basic.phpt`
- `ext/standard/tests/array/array_fill_object.phpt`
- `ext/standard/tests/array/array_find_types.phpt`
- `ext/standard/tests/array/array_intersect_1.phpt`
- `ext/standard/tests/array/array_intersect_uassoc_basic.phpt`
- `ext/standard/tests/array/array_push_empty.phpt`
- `ext/standard/tests/array/array_udiff_assoc_basic.phpt`
- `ext/standard/tests/array/array_udiff_basic.phpt`
- `ext/standard/tests/array/array_udiff_uassoc_basic.phpt`
- `ext/standard/tests/array/array_uintersect_assoc_basic.phpt`
- `ext/standard/tests/array/array_uintersect_basic.phpt`
- `ext/standard/tests/array/array_uintersect_uassoc_basic.phpt`

Implemented behavior: parser control-flow validation rejects non-positive
`break`/`continue` levels, non-integer operands, use outside loop/switch
contexts, and levels deeper than the current loop/switch stack, including
inside functions, methods, and closures. The final branch also includes the
current-master call-unpacking implementation for array operands on direct,
dynamic, method, constructor, and internal calls; unsupported by-reference,
Traversable/generator, and resource-limit cases remain classified.

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

## 2026-06-14 ptn-iuhj Parser/Method Visibility Row Pack

Final manifest:
`tools/phpt-ptn-iuhj-parser-visibility-row-pack.txt`.

Rebased focused command:
`PHPT_PROGRESS_DIR=.runtime/ptn-iuhj-row-pack-final2 tools/run-bounded-phpt.sh --classify-harness-programs tools/phpt-ptn-iuhj-parser-visibility-row-pack.txt`.
Artifact `run-20260614T230004Z-manifest.log` selected 43 rows: 41
runnable, 22 passed, 19 failed, and 2 classified
(`unsupported-dynamic-member-dispatch`,
`unsupported-diagnostics-runtime`).

Final broad 1k classify-only artifact
`.runtime/ptn-iuhj-broad-classify-final2-progress/summary-20260614T230538Z.txt`
selected 1,000 rows: 566 runnable and 434 classified. Pre-work
classify-only on `02cbacb3e7f5` selected 558 runnable and 442 classified,
including 9 rows in the old `unsupported-method-visibility-metadata` bucket.
The full broad after-run before the final rebase completed as 565 runnable,
379 passed, 186 failed, and 435 classified; final rebased pass-count evidence
for this bead is the focused row pack above.

Targeted rows green in the final focused evidence:

- `Zend/tests/break_error_001.phpt`
- `Zend/tests/break_error_002.phpt`
- `Zend/tests/break_error_003.phpt`
- `Zend/tests/break_error_004.phpt`
- `Zend/tests/access_modifiers/access_modifiers_008.phpt`
- `Zend/tests/access_modifiers/access_modifiers_009.phpt`
- `Zend/tests/access_modifiers/access_modifiers_012.phpt`
- `Zend/tests/bug21888.phpt`
- `Zend/tests/bug29210.phpt`
- `ext/standard/tests/array/array_map_object3.phpt`

Implemented behavior retained in the final branch: declared method metadata is
now visibility-aware for object dispatch, direct and scoped static dispatch,
`call_user_func()`, internal callback validation, and `is_callable()`.
Non-public object callbacks fall through to modeled `__call()` when available;
otherwise hidden declared methods throw a catchable `Error`. The classifier no
longer blanket-excludes non-public methods, splits dynamic member-name dispatch
into its own bucket, and keeps plain `__call()` rows runnable while still
excluding unmodeled magic-method metadata. On top of the upstream
break/continue validator, the parser also preserves PHP-style non-integer
operand diagnostics when a numeric transfer level is followed by expression
tokens.

Remaining targeted residuals are `access_modifiers_010.phpt`, where the
visibility error text matches but stack-frame rendering still differs,
`access_modifiers_011.phpt`, which requires dynamic member-name dispatch, and
`debug_backtrace_options.phpt`, which remains diagnostics-runtime metadata.
Other focused row-pack failures are the existing arithmetic, AST, binary, and
core/basic frontier rows in the mixed manifest.

## 2026-06-14 ptn-29k0 Object/Method Metadata Row Pack

Final manifest:
`tools/phpt-ptn-29k0-object-method-metadata-row-pack.txt`.

Hook-start broad 1k classify-only baseline:
`.runtime/phpt-progress/summary-20260614T221134Z.txt` selected 1,000 rows:
563 runnable and 437 classified.

Final broad 1k classify-only artifact:
`.runtime/phpt-progress/summary-20260614T235203Z.txt` selected 1,000 rows:
591 runnable and 409 classified. The current-base `ptn-wxno` entry above
already contributes the 25 object-string helper rows (588 runnable / 412
classified); this branch adds five method-visibility runnable rows over that
base. The `access_modifiers_010.phpt` row is newly runnable but still fails
stack-frame formatting, so the method-scope contribution is four newly passing
broad rows. Two hook-start runnable rows are now classified by upstream
dynamic/property metadata splits (`Zend/tests/assign_obj_op_cache_slot.phpt`
and `Zend/tests/bug29015.phpt`), so the net hook-start broad classify-only move
is 563/437 -> 591/409.

Final focused command:
`PHPT_PROGRESS_DIR=.runtime/ptn-29k0-row-pack-final-overlap tools/run-bounded-phpt.sh --classify-harness-programs tools/phpt-ptn-29k0-object-method-metadata-row-pack.txt`.
Artifact `run-20260614T234607Z-manifest.log` selected 35 rows: 32 runnable,
32 passed, 0 failed, and 3 classified (`unsupported-dynamic-member-dispatch` 1,
`unsupported-autoload-metadata` 1, `unsupported-diagnostics-runtime` 1).

Method rows newly green over the current `ptn-wxno` base:

- `Zend/tests/access_modifiers/access_modifiers_008.phpt`
- `Zend/tests/access_modifiers/access_modifiers_009.phpt`
- `Zend/tests/bug21888.phpt`
- `Zend/tests/bug29210.phpt`

Together with the 25 `ptn-wxno` object-string helper rows, the final integrated
branch has 29 newly passing broad rows relative to the hook-start broad 1k
baseline, with the net classify-only count at +28 runnable after the two
upstream split reclassifications.

Implemented behavior: declared class method metadata now preserves private and
protected methods with declaring-class information; object/static method
dispatch, `method_exists()`, and `is_callable()` apply scope-aware method
visibility; inaccessible instance methods may fall through to public `__call`;
and inaccessible static calls throw before evaluating arguments.

## 2026-06-14 ptn-3ijs Literal/Operator Row Pack

Final manifest:
`tools/phpt-ptn-3ijs-literal-operator-row-pack-manifest.txt`.

Broad 1k before command:
`timeout 2400 tools/run-phpt-baseline.sh --tier 1000 --out-dir .runtime/ptn-3ijs-broad-before`.
Classification artifact
`.runtime/phpt-progress/classification-20260614T223941Z.tsv` selected 1,000
rows: 563 runnable and 437 classified. The command completed the Zend bucket
and timed out during the standard bucket at test 66/346, so completed broad
pass-count evidence for this bead is the Zend bucket.

Completed broad Zend-bucket evidence used
`.runtime/phpt-progress/buckets-20260614T223941Z/zend.paths`, the 201 runnable
Zend rows selected by the broad 1k run. Before:
`.runtime/phpt-progress/run-20260614T223941Z-zend.log` had 145 passed and 56
failed. After:
`.runtime/ptn-3ijs-broad-after-zend.log` had 151 passed and 50 failed. This is
+6 broad Zend rows; the +10 target and +25 stretch target were not reached.

Focused final command on the rebased branch:
`PHPT_PROGRESS_DIR=.runtime/ptn-3ijs-focused-final-3 timeout 900s tools/run-bounded-phpt.sh tools/phpt-ptn-3ijs-literal-operator-row-pack-manifest.txt`.
Artifact `.runtime/ptn-3ijs-focused-final-3/summary-20260615T002115Z.txt`
selected 30 rows: 30 runnable, 27 passed, and 3 failed.

Rows newly green over the broad-before Zend bucket:

- `Zend/tests/array_literal_next_element_error.phpt`
- `Zend/tests/array_merge_recursive_next_key_overflow.phpt`
- `Zend/tests/assign_op_type_error.phpt`
- `Zend/tests/assign_to_obj_002.phpt`
- `Zend/tests/ast/ast_serialize_backtick_literal.phpt`
- `Zend/tests/ast/ast_serialize_floats.phpt`

Implemented behavior: array-literal auto-append now reports overflow through
the generic append-key guard, including variable and default-argument
`PHP_INT_MAX` keys, `array_merge_recursive()` auto-append checks the same guard,
array modulo and bit shifts reject unsupported operand types before numeric
conversion, missing non-quiet `$this` reads throw
`Error: Using $this when not in object context`, backtick literals parse as
shell-exec expressions for assertion serialization and lower through the
existing internal-call path, oversized radix integer tokens accumulate directly
to `f64` after integer overflow, and generated declared-method dispatch stubs
mark unused runtime parameters so C warnings do not contaminate PHPT output.

Remaining focused failures are `Zend/tests/ast/gh21072.phpt` for `(unset)` in a
constant expression, `Zend/tests/attributes/nodiscard/005.phpt` for native
method metadata, and `Zend/tests/binary.phpt`, which now reaches a float
formatting mismatch after the invalid-literal fatal was removed.

## 2026-06-15 ptn-ck7w Static Property Row Pack

Final manifest:
`tools/phpt-ptn-ck7w-static-property-row-pack.txt`.

Hook-start broad 1k classify-only baseline:
`.runtime/phpt-progress/summary-20260615T001604Z.txt` selected 1,000 rows:
591 runnable and 409 classified.

Final broad 1k classify-only artifact:
`.runtime/phpt-progress/summary-20260615T010015Z.txt` selected 1,000 rows:
592 runnable and 408 classified. The net broad movement is +1 runnable after
the classifier moved exposed autoload, top-level `static $x`, dynamic static
member, and class-scope spread-default residuals into explicit unsupported
buckets.

Final focused command:
`PHPT_PROGRESS_DIR=.runtime/ptn-ck7w-row-pack-final-rebased tools/run-bounded-phpt.sh --classify-harness-programs tools/phpt-ptn-ck7w-static-property-row-pack.txt`.
Artifact `.runtime/ptn-ck7w-row-pack-final-rebased/summary-20260615T010015Z.txt`
selected 24 rows: 2 runnable, 2 passed, 0 failed, and 22 classified
(`unsupported-function-state` 6, `unsupported-autoload-metadata` 1,
`unsupported-dynamic-member-dispatch` 1,
`unsupported-class-contract-metadata` 2,
`unsupported-class-constant-metadata` 1,
`unsupported-typed-property-metadata` 11).

Newly green focused rows:

- `Zend/tests/bug28442.phpt`
- `Zend/tests/bug30140.phpt`

Implemented behavior: static property reads and writes now resolve the storage
slot through the declared parent chain, so inherited static properties share the
ancestor slot while child redeclarations remain distinct. Visibility checks use
the declaring class resolved for the slot. The backend also resolves `parent`
static-property access to the lexical parent and keeps `static` aligned with
the current lexical class in the current bounded static-property model.

Classifier updates keep direct `Class::$property` rows runnable, stop treating
untyped `static $property` declarations as typed-property metadata, and split
the still-unmodeled residuals above into stable buckets.

## 2026-06-15 ptn-25s0 Broad COW/Reference Warning Row Pack

Broad manifest:
`.runtime/ptn-25s0-broad-before-classify/20260614T233541Z/phpt-baseline-1000.txt`.
Classify-only artifact
`.runtime/ptn-25s0-broad-before-classify-progress/summary-20260614T233542Z.txt`
selected 1,000 rows: 591 runnable and 409 classified.

Full broad before/after used the same deterministic runnable set. Before, on
`e66cda36499e`:
`.runtime/ptn-25s0-broad-before-full-progress/summary-20260614T235003Z.txt`
selected 1,000 rows, ran 591 rows, and passed 390. After the generated
declared-method scope helper marked its `runtime` parameter unused when no
generated branch needed it, artifact
`.runtime/ptn-25s0-broad-after-full-progress/summary-20260614T235003Z.txt`
ran the same 591 rows and passed 527. The pass-set comparison has 138 newly
passing rows and one broad-run pass-to-fail row; the full newly passing list is
saved at `.runtime/ptn-25s0-broad-new-passes.txt`.

The broad deltas are warning-cleanup rows, not new PHP runtime semantics. They
remove generated C `-Wunused-parameter` output from PHPT stdout/stderr so
already-correct diagnostics and callback/COW rows can compare cleanly. Newly
green broad rows include `Zend/tests/assign_ref_error_var_handling.phpt`,
`ext/standard/tests/array/array_filter*.phpt`,
`ext/standard/tests/array/array_map*.phpt`,
`ext/standard/tests/array/array_reduce_return_by_ref.phpt`, and
`ext/standard/tests/array/array_replace.phpt`.

The one broad pass-to-fail row was
`ext/standard/tests/array/array_splice_variation4.phpt`. A rebased one-row
rerun,
`.runtime/ptn-25s0-array-splice-variation4-rerun-after/run-20260615T013733Z-manifest.log`,
passed on the branch. The matching before rerun failed because the test opens
`__FILE__` and the run-tests generated `.php` companion was absent, so this is
tracked as harness byproduct sensitivity rather than a reproducible compiler
regression.

Rebased focused verification:

- `.runtime/ptn-25s0-broad-present-cow-rebased/summary-20260615T014229Z.txt`
  selected 13 broad-present COW/reference rows, ran 13, and passed 13.
- `.runtime/ptn-25s0-recursive-walk-rebased/summary-20260615T014230Z.txt`
  selected 14 recursive-walk rows, ran 14, and passed 13. The remaining
  failure is `array_walk_recursive_object1.phpt`, where object input is still
  not modeled as an array/property walk.

The backend marker itself was already present on the rebased `origin/master`
through the integrated literal/operator work, so this branch locks the behavior
with a native compile test assertion that the generated
`ptn_call_declared_method_in_scope()` definition includes `(void)runtime;`.

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
