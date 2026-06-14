# PHPT Broad 1k Attribute Classifier Bucket: 2026-06-14

Issue: `ptn-j8b8`

This slice improves broad PHPT blocker telemetry. It does not implement PHP
attributes. The classifier now reports real `#[...]` attribute syntax as the
dedicated `unsupported-attribute-metadata` bucket instead of hiding it inside
the generic `unsupported-language` bucket.

## Before

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only --out-dir .runtime/ptn-j8b8-baseline-before
```

Artifacts:

```text
.runtime/ptn-j8b8-baseline-before/20260614T080125Z/phpt-baseline-1000.txt
.runtime/phpt-progress/classification-20260614T080125Z.tsv
.runtime/phpt-progress/summary-20260614T080125Z.txt
```

Result:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 1,000 | 424 | 576 |

Relevant buckets before the split:

| Bucket | Rows |
| --- | ---: |
| `unsupported-language` | 288 |
| `unsupported-class-metadata` | 143 |

The `unsupported-language` reason split included 141 rows with:

```text
requires PHP attribute syntax (`#[...]`) and reflection metadata, outside PTN parser/metadata model
```

## After

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only --out-dir .runtime/ptn-j8b8-baseline-after
```

Artifacts:

```text
.runtime/ptn-j8b8-baseline-after/20260614T080952Z/phpt-baseline-1000.txt
.runtime/phpt-progress/classification-20260614T080952Z.tsv
.runtime/phpt-progress/summary-20260614T080952Z.txt
```

Result:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 1,000 | 424 | 576 |

Relevant buckets after the split:

| Bucket | Rows |
| --- | ---: |
| `unsupported-language` | 147 |
| `unsupported-attribute-metadata` | 141 |
| `unsupported-class-metadata` | 143 |

The focused 141-row manifest is committed at:

```text
tools/phpt-attribute-syntax-frontier-manifest.txt
```

Focused verification:

```sh
PHPT_PROGRESS_DIR=.runtime/ptn-j8b8-attribute-syntax-rebased \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-attribute-syntax-frontier-manifest.txt
```

Focused result:

| Selected | Runnable | Excluded | Bucket |
| ---: | ---: | ---: | --- |
| 141 | 0 | 141 | `unsupported-attribute-metadata` |

Attribute metadata frontier cross-check:

```sh
PHPT_PROGRESS_DIR=.runtime/ptn-j8b8-attribute-focused-after \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-attributes-metadata-frontier-manifest.txt
```

Cross-check result:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 204 | 1 | 203 |

Focused buckets:

| Bucket | Rows |
| --- | ---: |
| `unsupported-attribute-metadata` | 141 |
| `unsupported-language` | 46 |
| `unsupported-class-metadata` | 8 |
| `unsupported-extension` | 8 |
| `runnable` | 1 |

## Attribute Syntax Split

| Attribute path group | Rows |
| --- | ---: |
| root `Zend/tests/attributes` | 35 |
| `deprecated/` | 34 |
| `override/` | 18 |
| `delayed_target_validation/` | 18 |
| `constants/` | 17 |
| `nodiscard/` | 15 |
| `Attribute/` | 4 |

Remaining `unsupported-language` reasons after the split:

| Blocker | Rows |
| --- | ---: |
| Call-site or array unpacking (`...`) | 34 |
| Trait declarations | 25 |
| Interface declarations | 23 |
| Interface implementation checks | 15 |
| Anonymous class syntax | 15 |
| Nullable type-hint metadata | 14 |
| Static local variables | 11 |
| Variable variables | 8 |
| Named array-internal argument binding | 1 |
| Generator/yield lowering | 1 |

## Why This Is Not Runtime Support

The moved rows still require generic attribute architecture before execution:

- lexer/parser support for attribute groups before every attachable declaration;
- AST and metadata storage for names, arguments, grouping, source spans, and
  attachment targets;
- constant-expression evaluation for attribute arguments;
- target, repeatability, delayed validation, and built-in attribute semantics;
- Reflection APIs such as `ReflectionAttribute` and `getAttributes()`.

The new bucket makes broad 1k telemetry more actionable without reopening rows
that would currently fail as parser or metadata noise.

## Verification

```sh
cargo fmt --check
cargo test --test phpt_classifier
PHPT_PROGRESS_DIR=.runtime/ptn-j8b8-attribute-syntax-rebased tools/run-bounded-phpt.sh --classify-only --classify-harness-programs tools/phpt-attribute-syntax-frontier-manifest.txt
PHPT_PROGRESS_DIR=.runtime/ptn-j8b8-attribute-focused-after tools/run-bounded-phpt.sh --classify-only --classify-harness-programs tools/phpt-attributes-metadata-frontier-manifest.txt
tools/run-phpt-baseline.sh --tier 1000 --classify-only --out-dir .runtime/ptn-j8b8-baseline-before
tools/run-phpt-baseline.sh --tier 1000 --classify-only --out-dir .runtime/ptn-j8b8-baseline-after
```
