# PHPT Broad 1k Attribute Metadata Split: 2026-06-14 ptn-30ji

Issue: `ptn-30ji`

This slice refines the broad 1k attribute blocker classification. It does not
claim runtime support for PHP attributes. The change keeps all rows excluded,
but moves the prior coarse 149-row `unsupported-attribute-metadata` bucket into
two implementation-facing buckets:

- `unsupported-attribute-syntax-metadata` for PHP `#[...]` parser, AST,
  declaration attachment, validation, and reflection metadata.
- `unsupported-internal-attribute-metadata` for internal attribute/reflection
  APIs such as `Reflection*::getAttributes()`, `Attribute::TARGET_*`, and
  modeled built-in attribute classes such as `Deprecated` and `NoDiscard`.

## Focused Attribute Replay

Focused manifest:

```text
tools/phpt-attribute-metadata-classifier-ptn-61f9-manifest.txt
```

Focused command:

```sh
PHPT_PROGRESS_DIR=.runtime/ptn-30ji-attribute-current \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-attribute-metadata-classifier-ptn-61f9-manifest.txt
```

Focused artifact:

```text
.runtime/ptn-30ji-attribute-current/classification-20260614T114449Z.tsv
```

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 149 | 0 | 149 |

Focused split:

| Bucket | Rows |
| --- | ---: |
| `unsupported-attribute-syntax-metadata` | 141 |
| `unsupported-internal-attribute-metadata` | 8 |

The syntax rows split by attribute test family as:

| Family | Rows |
| --- | ---: |
| root `Zend/tests/attributes/*.phpt` | 35 |
| `deprecated/` | 34 |
| `override/` | 18 |
| `delayed_target_validation/` | 18 |
| `constants/` | 17 |
| `nodiscard/` | 15 |
| `Attribute/` | 4 |

The internal metadata rows are:

```text
Zend/tests/attributes/007_self_reflect_attribute.phpt
Zend/tests/attributes/029_reflect_internal_symbols.phpt
Zend/tests/attributes/034_target_values.phpt
Zend/tests/attributes/deprecated/property_readonly_001.phpt
Zend/tests/attributes/deprecated/property_readonly_002.phpt
Zend/tests/attributes/deprecated/property_readonly_003.phpt
Zend/tests/attributes/nodiscard/property_readonly_001.phpt
Zend/tests/attributes/nodiscard/property_readonly_002.phpt
```

## Blocker Boundary

The 141 syntax rows need a parser/AST and declaration metadata slice before
they should become runnable. That work includes grouped attributes, namespaced
names, arguments, source locations, attachable declaration targets,
constant-expression evaluation, repeatability checks, and delayed validation.

The 8 internal metadata rows are a smaller runtime/reflection slice. They need
internal attribute class metadata, `Attribute::TARGET_*` constants,
`ReflectionAttribute`, and `Reflection*::getAttributes()` integration over the
modeled class/function/property metadata tables.

Keeping these buckets separate prevents parser-frontier work from being mixed
with internal reflection registry work.

## Verification

```sh
bash -n tools/phpt-classifier.sh
cargo test --test phpt_classifier
PHPT_PROGRESS_DIR=.runtime/ptn-30ji-attribute-current \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-attribute-metadata-classifier-ptn-61f9-manifest.txt
```
