# PHPT Broad 1k Language Classifier Split: ptn-18tp

Issue: `ptn-18tp`

This slice splits the current broad PHPT classifier's remaining
`unsupported-language` bucket into stable semantic categories. It does not
claim new runtime support: all affected rows remain classified and excluded.
The PHP attribute syntax/reflection rows stay in the separate
`unsupported-attribute-metadata` bucket introduced by `ptn-j8b8/b35n`.

## Before

After the attribute split, the broad 1k classifier still had 147 rows grouped
under `unsupported-language`.

Artifacts:

```text
.runtime/ptn-b35n-baseline-post/20260614T090208Z/phpt-baseline-1000.txt
.runtime/phpt-progress/classification-20260614T090208Z.tsv
```

Result:

| Selected | Runnable | Excluded | `unsupported-language` | `unsupported-attribute-metadata` |
| ---: | ---: | ---: | ---: | ---: |
| 1000 | 424 | 576 | 147 | 149 |

## After

The same 147 language-surface rows remain excluded, but they are now
classified by the generic subsystem they require.

Command:

```sh
PHPT_PROGRESS_DIR=.runtime/ptn-18tp-broad-post \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  .runtime/ptn-b35n-baseline-post/20260614T090208Z/phpt-baseline-1000.txt
```

Artifacts:

```text
.runtime/ptn-18tp-broad-post/classification-20260614T094206Z.tsv
.runtime/ptn-18tp-broad-post/runnable-20260614T094206Z.txt
.runtime/ptn-18tp-broad-post/excluded-20260614T094206Z.tsv
```

Result:

| Selected | Runnable | Excluded | Language rows split | Attribute metadata |
| ---: | ---: | ---: | ---: | ---: |
| 1000 | 424 | 576 | 147 | 149 |

New language-surface categories:

| Category | Rows | Semantic frontier |
| --- | ---: | --- |
| `unsupported-class-declaration` | 78 | Interfaces, traits, implementation checks, and anonymous classes. |
| `unsupported-call-unpacking` | 34 | Call-site spread arguments and array unpacking. |
| `unsupported-type-hint` | 14 | Nullable and `never` type metadata/coercion/control-flow validation. |
| `unsupported-function-state` | 11 | Static local function storage. |
| `unsupported-dynamic-symbol` | 8 | Variable variables and runtime symbol-table lookup/mutation. |
| `unsupported-generator-runtime` | 1 | Generator/yield lowering and suspension runtime. |
| `unsupported-internal-call-binding` | 1 | Named arguments for modeled array internals. |

The focused replay over those new categories used:

```sh
awk -F'\t' '$2 ~ /^unsupported-(class-declaration|call-unpacking|type-hint|function-state|dynamic-symbol|generator-runtime|internal-call-binding)$/ {print $1}' \
  .runtime/ptn-18tp-broad-post/classification-20260614T094206Z.tsv \
  > .runtime/ptn-18tp-language-split-frontier.txt
PHPT_PROGRESS_DIR=.runtime/ptn-18tp-language-focused \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  .runtime/ptn-18tp-language-split-frontier.txt
```

Focused artifacts:

```text
.runtime/ptn-18tp-language-split-frontier.txt
.runtime/ptn-18tp-language-focused/classification-20260614T094652Z.tsv
```

Focused result:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 147 | 0 | 147 |

## Representative Rows

```text
Zend/tests/anon/001.phpt
Zend/tests/ArrayAccess/ArrayAccess_indirect_append.phpt
Zend/tests/arg_unpack/basic.phpt
Zend/tests/array_unpack/basic.phpt
Zend/tests/arrow_functions/003.phpt
Zend/tests/bug35163_2.phpt
ext/standard/tests/array/array_filter_invalid_mode.phpt
tests/basic/bug73969.phpt
```

## Verification

```sh
bash -n tools/phpt-classifier.sh
cargo fmt --check
cargo test --test phpt_classifier
PHPT_PROGRESS_DIR=.runtime/ptn-18tp-broad-post \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  .runtime/ptn-b35n-baseline-post/20260614T090208Z/phpt-baseline-1000.txt
PHPT_PROGRESS_DIR=.runtime/ptn-18tp-language-focused \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  .runtime/ptn-18tp-language-split-frontier.txt
```

Results:

- `bash -n tools/phpt-classifier.sh`: passed.
- `cargo fmt --check`: passed.
- `cargo test --test phpt_classifier`: passed, 33 tests.
- Broad classify-only: passed, 1000 selected, 424 runnable, 576 excluded.
- Focused classify-only: passed, 147 selected, 0 runnable, 147 excluded.
