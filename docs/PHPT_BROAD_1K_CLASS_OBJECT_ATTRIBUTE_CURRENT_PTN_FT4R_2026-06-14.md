# PHPT Broad 1k Class/Object/Attribute Current Map: 2026-06-14 ptn-ft4r

Issue: `ptn-ft4r`

This slice refreshes the broad PHPT 1k classifier on current `origin/master`
and records the combined class/object/attribute metadata boundary. It is a
blocker map, not a runtime implementation claim. The current broad runnable
surface is already covered by focused manifests, and the largest remaining
metadata frontier is excluded for generic class-table, reflection, visibility,
attribute, autoload, and magic dispatch semantics that do not fit one narrow
implementation patch.

## Broad 1k Evidence

Command:

```sh
PHPT_PROGRESS_DIR=.runtime/ptn-ft4r-rebased-1k-progress \
  tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-ft4r-rebased-1k
```

Generated broad manifest:

```text
.runtime/ptn-ft4r-rebased-1k/20260614T112444Z/phpt-baseline-1000.txt
```

Classifier artifacts:

```text
.runtime/ptn-ft4r-rebased-1k-progress/classification-20260614T112444Z.tsv
.runtime/ptn-ft4r-rebased-1k-progress/runnable-20260614T112444Z.txt
.runtime/ptn-ft4r-rebased-1k-progress/excluded-20260614T112444Z.tsv
```

State:

```text
PTN: 2314e739f307
php-src PHPT corpus revision: 8c63ec400ce8e07c57a8d9499317b96a8beafb8b
```

Broad classifier result:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 1,000 | 424 | 576 |

The excluded rows now split into three high-level semantic frontiers plus one
single-row internal blocker:

| Frontier | Rows | Notes |
| --- | ---: | --- |
| Class/object/attribute metadata | 284 | Current manifest added by this slice. |
| Language declaration/call/type blockers | 147 | Covered by the language and class-declaration split maps. |
| Runtime/configuration/harness boundary | 144 | Covered by the current runtime-boundary map. |
| Other internal blocker | 1 | `unsupported-internal`. |

The runnable side remains 424 rows. Its largest cluster is the existing
294-row standard-array frontier, already covered by committed focused
manifests and residual blocker maps.

## Focused Metadata Manifest

Committed manifest:

```text
tools/phpt-class-object-attribute-current-ptn-ft4r-manifest.txt
```

Selection from `classification-20260614T112444Z.tsv`:

```sh
awk -F'\t' '$2 ~ /^(unsupported-attribute-metadata|unsupported-magic-method-metadata|unsupported-property-visibility-metadata|unsupported-typed-property-metadata|unsupported-autoload-metadata|unsupported-class-contract-metadata|unsupported-method-visibility-metadata|unsupported-readonly-property-metadata|unsupported-internal-reflection-metadata)$/ {print $1}'
```

Focused classify-only replay:

```sh
PHPT_PROGRESS_DIR=.runtime/ptn-ft4r-rebased-class-object-attribute-focused \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-class-object-attribute-current-ptn-ft4r-manifest.txt
```

Focused artifacts:

```text
.runtime/ptn-ft4r-rebased-class-object-attribute-focused/classification-20260614T113018Z.tsv
.runtime/ptn-ft4r-rebased-class-object-attribute-focused/excluded-20260614T113018Z.tsv
```

Current integration replay:

```text
.runtime/ptn-ft4r-class-object-attribute-current/classification-20260614T113735Z.tsv
.runtime/ptn-ft4r-class-object-attribute-current/excluded-20260614T113735Z.tsv
```

Focused result:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 284 | 0 | 284 |

## Category Split

| Classification | Rows |
| --- | ---: |
| `unsupported-attribute-metadata` | 149 |
| `unsupported-magic-method-metadata` | 69 |
| `unsupported-property-visibility-metadata` | 19 |
| `unsupported-typed-property-metadata` | 12 |
| `unsupported-autoload-metadata` | 9 |
| `unsupported-class-contract-metadata` | 9 |
| `unsupported-method-visibility-metadata` | 7 |
| `unsupported-readonly-property-metadata` | 7 |
| `unsupported-internal-reflection-metadata` | 3 |

Path split:

| Path group | Rows |
| --- | ---: |
| `Zend/tests/attributes` | 149 |
| `ext/standard/tests/array` | 70 |
| Root and miscellaneous `Zend/tests` | 26 |
| `Zend/tests/asymmetric_visibility` | 16 |
| `Zend/tests/access_modifiers` | 12 |
| `Zend/tests/autoload` | 9 |
| `Zend/tests/backtrace` | 2 |

## Relation To Existing Maps

This manifest is a current union crosswalk for metadata work. It complements,
rather than replaces, the narrower maps:

| Existing focus | Rows | Role |
| --- | ---: | --- |
| `tools/phpt-attribute-metadata-classifier-ptn-61f9-manifest.txt` | 149 | Current attribute syntax/reflection metadata rows. |
| `tools/phpt-magic-method-metadata-current-ptn-7fym-manifest.txt` | 69 | Current magic method metadata rows. |
| `tools/phpt-broad-class-metadata-split-manifest.txt` | 143 | Earlier split of class metadata sub-buckets before the current full attribute bucket was rolled into this crosswalk. |

## Blocker Boundary

The 284 rows are above the broad-slice threshold, but implementing them as one
patch is not credible. Generic support needs independent runtime layers:

- attribute syntax/reflection metadata for userland and internal symbols;
- magic method dispatch and object conversion across property access, array
  helper callbacks, comparison, dump, and stringification paths;
- property and method visibility metadata, including typed, readonly,
  asymmetric, inherited, and uninitialized property states;
- class contract metadata for abstract/final declarations and method
  compatibility;
- autoload and class-table mutation during native execution;
- internal reflection metadata for complete arginfo, properties, methods,
  attributes, and closure binding.

Until those layers land, reclassifying this frontier as runnable would turn one
coherent metadata boundary into scattered parser/runtime/diagnostic failures.
The next implementation slice should choose one category from the focused
manifest, land the generic semantics, and then remove only the classifier
branch that is actually supported.

## Verification

```sh
PHPT_PROGRESS_DIR=.runtime/ptn-ft4r-rebased-1k-progress \
  tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-ft4r-rebased-1k
PHPT_PROGRESS_DIR=.runtime/ptn-ft4r-rebased-class-object-attribute-focused \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-class-object-attribute-current-ptn-ft4r-manifest.txt
cargo fmt --check
cargo test --test phpt_classifier
```

Current integration results:

- `cargo fmt --check`: passed.
- `cargo test --test phpt_classifier`: passed, 35 tests.
- Focused class/object/attribute replay: 284 selected, 0 runnable, 284
  excluded across the nine metadata buckets above.
