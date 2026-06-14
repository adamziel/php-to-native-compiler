# PHPT Broad 1k Attribute Classifier Split: ptn-61f9

Issue: `ptn-61f9`

This slice keeps PHP attribute support classified as a blocker, but splits it
out of the broad `unsupported-language` and `unsupported-class-metadata`
buckets. It is not a support claim: attributes still require parser, AST,
metadata, validation, built-in attribute, and Reflection work before these rows
can become runnable.

This evidence was produced in parallel with the currently merged
`ptn-j8b8/b35n` attribute classifier work. On current `master`, those semantics
are already present; this document and manifest preserve the `ptn-61f9`
focused 149-row source set without changing classifier behavior.

## Before Evidence

Source state:

- PTN before classifier split: `95deb32b4d1c`
- php-src PHPT corpus: `8c63ec400ce8e07c57a8d9499317b96a8beafb8b`

Commands:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only
```

Generated broad manifest:

```text
.runtime/phpt-baseline/20260614T082938Z/phpt-baseline-1000.txt
```

Classifier artifacts:

```text
.runtime/phpt-progress/summary-20260614T082939Z.txt
.runtime/phpt-progress/classification-20260614T082939Z.tsv
```

Before counts:

| Measurement | Rows |
| --- | ---: |
| Selected | 1,000 |
| Runnable | 425 |
| Excluded | 575 |
| `unsupported-language` | 288 |
| `unsupported-class-metadata` | 143 |
| `unsupported-attribute-metadata` | 0 |

## After Evidence

The after pass reused the same generated broad 1k manifest:

```sh
PHPT_PROGRESS_DIR=.runtime/ptn-61f9-after-progress \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  .runtime/phpt-baseline/20260614T082938Z/phpt-baseline-1000.txt
```

Classifier artifacts:

```text
.runtime/ptn-61f9-after-progress/summary-20260614T083703Z.txt
.runtime/ptn-61f9-after-progress/classification-20260614T083703Z.tsv
.runtime/ptn-61f9-after-progress/excluded-20260614T083703Z/unsupported-attribute-metadata.txt
```

Committed focused manifest:

```text
tools/phpt-attribute-metadata-classifier-ptn-61f9-manifest.txt
```

After counts:

| Measurement | Rows |
| --- | ---: |
| Selected | 1,000 |
| Runnable | 424 |
| Excluded | 576 |
| `unsupported-attribute-metadata` | 149 |
| `unsupported-language` | 147 |
| `unsupported-class-metadata` | 135 |

Movement into `unsupported-attribute-metadata`:

| Prior bucket | Rows |
| --- | ---: |
| `unsupported-language` | 141 |
| `unsupported-class-metadata` | 8 |

The one runnable-count change in the same after run is
`Zend/tests/ErrorException_getSeverity.phpt`, which the existing runtime
diagnostics classifier maps to `unsupported-diagnostics-runtime`. The attribute
split itself accounts for 149 newly classified attribute metadata rows.

Final focused verification after rebasing onto current `origin/master`:

```text
.runtime/ptn-61f9-final2-attribute-progress/summary-20260614T085057Z.txt
```

| Focused manifest | Selected | Runnable | Excluded |
| --- | ---: | ---: | ---: |
| `tools/phpt-attribute-metadata-classifier-ptn-61f9-manifest.txt` | 149 | 0 | 149 |

## Attribute Sub-Buckets

| Sub-bucket | Rows |
| --- | ---: |
| root `Zend/tests/attributes` rows | 38 |
| `deprecated/` | 37 |
| `delayed_target_validation/` | 18 |
| `override/` | 18 |
| `constants/` | 17 |
| `nodiscard/` | 17 |
| `Attribute/` | 4 |

## Remaining Architecture Work

Full attribute support still needs generic compiler/runtime work:

- Lexer/parser support for attribute groups before declarations and parameters.
- AST storage for attribute names, arguments, grouping, nesting, source spans,
  and declaration attachment points.
- Constant-expression evaluation for attribute arguments.
- Metadata tables for every attribute-bearing declaration surface.
- Validation for `Attribute` flags, targets, repeatability, delayed target
  checks, and built-in attributes such as `Deprecated`, `Override`, and
  `NoDiscard`.
- Reflection APIs for `ReflectionAttribute` and `getAttributes()` across all
  attachable declarations.

## Verification

```sh
cargo test --test phpt_classifier -- --nocapture
PHPT_PROGRESS_DIR=.runtime/ptn-61f9-after-progress \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  .runtime/phpt-baseline/20260614T082938Z/phpt-baseline-1000.txt
```
