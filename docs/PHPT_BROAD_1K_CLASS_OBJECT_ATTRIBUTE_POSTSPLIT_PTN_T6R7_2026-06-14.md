# PHPT Broad 1k Class/Object/Attribute Post-Split Map: 2026-06-14 ptn-t6r7

Issue: `ptn-t6r7`

This slice refreshes the broad PHPT 1k classifier after the attribute metadata
split landed and records the current combined class/object/attribute metadata
boundary. It is a blocker map, not a runtime support claim.

The earlier combined metadata crosswalk used the coarse
`unsupported-attribute-metadata` bucket. Current `master` now splits those same
149 rows into `unsupported-attribute-syntax-metadata` and
`unsupported-internal-attribute-metadata`. This map keeps the 284-row combined
frontier intact while making the implementation split explicit.

## Broad 1k Evidence

Command:

```sh
PHPT_PROGRESS_DIR=.runtime/ptn-t6r7-1k-progress \
  tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-t6r7-1k
```

Generated broad manifest:

```text
.runtime/ptn-t6r7-1k/20260614T115227Z/phpt-baseline-1000.txt
```

Classifier artifacts:

```text
.runtime/ptn-t6r7-1k-progress/classification-20260614T115227Z.tsv
.runtime/ptn-t6r7-1k-progress/runnable-20260614T115227Z.txt
.runtime/ptn-t6r7-1k-progress/excluded-20260614T115227Z.tsv
```

State:

```text
PTN: 98ef0cd0cc71
php-src PHPT corpus revision: 8c63ec400ce8e07c57a8d9499317b96a8beafb8b
```

Broad classifier result:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 1,000 | 424 | 576 |

## Focused Post-Split Manifest

Committed manifest:

```text
tools/phpt-class-object-attribute-postsplit-ptn-t6r7-manifest.txt
```

Selection from `classification-20260614T115227Z.tsv`:

```sh
awk -F'\t' '$2 ~ /^(unsupported-attribute-syntax-metadata|unsupported-internal-attribute-metadata|unsupported-magic-method-metadata|unsupported-property-visibility-metadata|unsupported-typed-property-metadata|unsupported-autoload-metadata|unsupported-class-contract-metadata|unsupported-method-visibility-metadata|unsupported-readonly-property-metadata|unsupported-internal-reflection-metadata)$/ {print $1}'
```

Focused classify-only replay:

```sh
PHPT_PROGRESS_DIR=.runtime/ptn-t6r7-class-object-attribute-focused-rebased \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-class-object-attribute-postsplit-ptn-t6r7-manifest.txt
```

Focused artifacts:

```text
.runtime/ptn-t6r7-class-object-attribute-focused-rebased/classification-20260614T120100Z.tsv
.runtime/ptn-t6r7-class-object-attribute-focused-rebased/excluded-20260614T120100Z.tsv
```

Focused result:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 284 | 0 | 284 |

## Category Split

| Classification | Rows |
| --- | ---: |
| `unsupported-attribute-syntax-metadata` | 141 |
| `unsupported-magic-method-metadata` | 69 |
| `unsupported-property-visibility-metadata` | 19 |
| `unsupported-typed-property-metadata` | 12 |
| `unsupported-autoload-metadata` | 9 |
| `unsupported-class-contract-metadata` | 9 |
| `unsupported-internal-attribute-metadata` | 8 |
| `unsupported-readonly-property-metadata` | 7 |
| `unsupported-method-visibility-metadata` | 7 |
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

This manifest is the current post-split union of the existing attribute and
class/object metadata frontiers:

| Existing focus | Rows in this manifest | Role |
| --- | ---: | --- |
| `tools/phpt-attribute-metadata-classifier-ptn-61f9-manifest.txt` | 149 | Attribute rows now split into syntax and internal metadata buckets. |
| `tools/phpt-class-object-metadata-granular-ptn-gt7b-manifest.txt` | 135 | Class/object metadata rows after the aggregate metadata bucket split. |
| `tools/phpt-class-object-attribute-current-ptn-ft4r-manifest.txt` | 284 | Earlier combined crosswalk using the pre-`ptn-30ji` coarse attribute category. |

## Blocker Boundary

The 284 rows are above the broad-slice threshold, but one patch should not try
to open the entire frontier. Generic support needs separate layers:

- PHP attribute parser/AST support, declaration attachment, target validation,
  repeatability, constant-expression argument evaluation, and source metadata;
- internal attribute classes, `Attribute::TARGET_*` constants,
  `ReflectionAttribute`, and `Reflection*::getAttributes()` over modeled
  metadata tables;
- magic method availability, signature validation, dispatch, object
  conversion, callback/reflection behavior, and dump integration;
- property and method visibility metadata, including typed, readonly,
  asymmetric, inherited, and uninitialized property states;
- class contract metadata for abstract/final declarations and compatibility;
- autoload and runtime class-table mutation boundaries;
- internal reflection metadata for complete arginfo, properties, methods,
  attributes, and closure binding.

The next implementation slice should choose one category from this manifest,
land the generic semantics, and then re-run this focused union plus the broad
1k classifier.

## Verification

```sh
PHPT_PROGRESS_DIR=.runtime/ptn-t6r7-1k-progress \
  tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-t6r7-1k
PHPT_PROGRESS_DIR=.runtime/ptn-t6r7-class-object-attribute-focused-rebased \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-class-object-attribute-postsplit-ptn-t6r7-manifest.txt
cargo fmt --check
cargo test --test phpt_classifier
```
