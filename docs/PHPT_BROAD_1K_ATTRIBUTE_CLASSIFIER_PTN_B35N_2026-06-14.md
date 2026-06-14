# PHPT Broad 1k Attribute Classifier Split: 2026-06-14

Issue: `ptn-b35n`

This slice makes the broad PHPT classifier identify PHP attribute syntax and
internal attribute/reflection metadata as an explicit
`unsupported-attribute-metadata` bucket. This is not a support claim: the rows
still require generic parser, declaration metadata, validation, built-in
attribute class, and reflection work before they can become runnable.

The slice also fixes the classifier runner's FILE-section helper status
handling. Several helper AWK programs intentionally stop at the first blocker;
under `set -o pipefail`, the upstream section extractor can receive SIGPIPE
and hide the classification from `run-bounded-phpt.sh`. The helpers now return
the downstream AWK status, so direct classification and runner classification
agree.

## Broad 1k Evidence

Before command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-b35n-baseline-before
```

Before artifacts:

```text
.runtime/ptn-b35n-baseline-before/20260614T080219Z/phpt-baseline-1000.txt
.runtime/phpt-progress/classification-20260614T080220Z.tsv
```

After commands:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-b35n-baseline-after
PHPT_PROGRESS_DIR=.runtime/ptn-b35n-after-progress-rebased \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  .runtime/ptn-b35n-baseline-after/20260614T080903Z/phpt-baseline-1000.txt
```

After artifacts:

```text
.runtime/ptn-b35n-baseline-after/20260614T080903Z/phpt-baseline-1000.txt
.runtime/ptn-b35n-after-progress-rebased/classification-20260614T083036Z.tsv
```

Both broad runs use php-src PHPT corpus revision
`8c63ec400ce8e07c57a8d9499317b96a8beafb8b`.

| Run | Selected | Runnable | Excluded |
| --- | ---: | ---: | ---: |
| Before | 1000 | 424 | 576 |
| After | 1000 | 424 | 576 |

Relevant bucket movement:

| Bucket | Before | After |
| --- | ---: | ---: |
| `unsupported-attribute-metadata` | 0 | 149 |
| `unsupported-language` | 288 | 147 |
| `unsupported-class-metadata` | 143 | 135 |
| `unsupported-diagnostics-runtime` | 17 | 17 |

The 149-row new bucket is a classifier split: 141 rows moved out of the broad
`unsupported-language` bucket due PHP attribute syntax, and 8 rows moved out of
`unsupported-class-metadata` due internal attribute/reflection metadata.

## Focused Attribute Evidence

Command:

```sh
PHPT_PROGRESS_DIR=.runtime/ptn-b35n-attribute-focused-rebased \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  .runtime/ptn-b35n-after-progress-rebased/excluded-20260614T083036Z/unsupported-attribute-metadata.txt
```

Focused artifact:

```text
.runtime/ptn-b35n-attribute-focused-rebased/classification-20260614T083530Z.tsv
```

| Selected | Runnable | Excluded | Bucket |
| ---: | ---: | ---: | --- |
| 149 | 0 | 149 | `unsupported-attribute-metadata` |

Representative rows:

```text
Zend/tests/attributes/001_placement.phpt
Zend/tests/attributes/004_name_resolution.phpt
Zend/tests/attributes/020_userland_attribute_validation.phpt
Zend/tests/attributes/026_unpack_in_args.phpt
Zend/tests/attributes/constants/multiple_attributes_grouped.phpt
Zend/tests/attributes/delayed_target_validation/validator_Attribute.phpt
Zend/tests/attributes/deprecated/functions/001.phpt
Zend/tests/attributes/nodiscard/001.phpt
Zend/tests/attributes/override/010.phpt
```

## Why This Is A Blocker

Generic attribute support needs the shared PHP metadata stack, not row-specific
output shaping:

- parser and AST nodes for grouped attributes, namespaced names, arguments,
  source spans, and allowed declaration targets;
- constant-expression evaluation for attribute arguments and validation of
  repeatability and target masks;
- declaration attachment for functions, classes, methods, properties, class
  constants, parameters, closures, traits, interfaces, and enums;
- built-in attribute classes such as `Attribute`, `Deprecated`, `NoDiscard`,
  `Override`, `AllowDynamicProperties`, and `SensitiveParameter`;
- `ReflectionAttribute` plus `Reflection*::getAttributes()` metadata and
  delayed validation behavior.

Until those surfaces are implemented generically, this explicit bucket keeps
149 broad 1k rows out of generic parser/runtime noise while preserving the same
overall runnable/excluded totals.

## Verification

```sh
bash -n tools/phpt-classifier.sh
cargo fmt --check
cargo test --test phpt_classifier
PHPT_PROGRESS_DIR=.runtime/ptn-b35n-debug-progress-rebased \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  .runtime/ptn-b35n-debug/error-exception-getseverity.txt
PHPT_PROGRESS_DIR=.runtime/ptn-b35n-after-progress-rebased \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  .runtime/ptn-b35n-baseline-after/20260614T080903Z/phpt-baseline-1000.txt
PHPT_PROGRESS_DIR=.runtime/ptn-b35n-attribute-focused-rebased \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  .runtime/ptn-b35n-after-progress-rebased/excluded-20260614T083036Z/unsupported-attribute-metadata.txt
```
