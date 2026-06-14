# PHPT Broad 1k Function/Dynamic Type Current Frontier: 2026-06-14

Issue: `ptn-rlzz`

This slice refreshed the broad 1k PHPT classifier on `origin/master` at
`fcaa57c32` and records a focused blocker map for the dynamic function/type surface:
nullable type hints, function-local `static` variables, variable variables,
one generator boundary, and one named-argument internal-call row.

This is a blocker map, not a behavior change. These rows require shared parser,
IR, call-frame, type metadata, and dynamic symbol-table work before native PHPT
execution would produce useful signal.

## Broad 1k Evidence

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only
```

Generated broad manifest:

```text
.runtime/phpt-baseline/20260614T084528Z/phpt-baseline-1000.txt
```

Classifier artifacts:

```text
.runtime/phpt-progress/classification-20260614T084529Z.tsv
.runtime/phpt-progress/runnable-20260614T084529Z.txt
.runtime/phpt-progress/excluded-20260614T084529Z.tsv
```

PTN commit: `fcaa57c32`

Corpus revision: `8c63ec400ce8e07c57a8d9499317b96a8beafb8b`

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 1,000 | 425 | 575 |

Top broad classifier buckets:

| Classification | Rows |
| --- | ---: |
| `unsupported-language` | 288 |
| `unsupported-class-metadata` | 143 |
| `unsupported-request-input-ini` | 28 |
| `unsupported-extension` | 20 |
| `unsupported-assertion-ini` | 17 |
| `unsupported-diagnostics-runtime` | 16 |
| `unsupported-resource-limit-ini` | 15 |
| `sapi-behavior` | 13 |
| `unsupported-assertion-runtime` | 9 |

## Focused Frontier

Committed manifest:

```text
tools/phpt-function-dynamic-current-ptn-rlzz-manifest.txt
```

Selection from the broad classifier:

```sh
awk -F'\t' '$2=="unsupported-language" &&
  ($3 ~ /nullable type-hint|static local variables|variable variables|generator\/yield|named-argument/) {
    print $1
  }' .runtime/phpt-progress/classification-20260614T084529Z.tsv
```

Original focused classify-only validation:

```sh
tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-function-dynamic-current-ptn-rlzz-manifest.txt
```

Focused artifacts:

```text
.runtime/phpt-progress/classification-20260614T085328Z.tsv
.runtime/phpt-progress/runnable-20260614T085328Z.txt
.runtime/phpt-progress/excluded-20260614T085328Z.tsv
```

Result:

| Selected | Runnable | Excluded | Bucket |
| ---: | ---: | ---: | --- |
| 35 | 0 | 35 | `unsupported-language` |

On current `master` after `ptn-18tp`, the same committed manifest stays fully
classified but maps to the semantic buckets below:

```sh
PHPT_PROGRESS_DIR=.runtime/ptn-rlzz-focused-current \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-function-dynamic-current-ptn-rlzz-manifest.txt
```

Current focused artifact:

```text
.runtime/ptn-rlzz-focused-current/classification-20260614T095859Z.tsv
```

| Classification | Rows |
| --- | ---: |
| `unsupported-type-hint` | 14 |
| `unsupported-function-state` | 11 |
| `unsupported-dynamic-symbol` | 8 |
| `unsupported-generator-runtime` | 1 |
| `unsupported-internal-call-binding` | 1 |

## Blocker Split

| Generic blocker | Rows |
| --- | ---: |
| Nullable type-hint metadata and coercion (`?T`) | 14 |
| Function-local `static` variables | 11 |
| Variable variables and runtime symbol-table lookup/mutation | 8 |
| Named-argument binding for modeled array internal calls | 1 |
| Generator/yield lowering | 1 |
| Total | 35 |

Source split:

| Source family | Rows |
| --- | ---: |
| `Zend/tests` | 31 |
| `ext/standard/tests/array` | 4 |

## Blocked Rows

```text
Zend/tests/anon/015.phpt
Zend/tests/anon/016.phpt
Zend/tests/arrow_functions/003.phpt
Zend/tests/arrow_functions/005.phpt
Zend/tests/arrow_functions/006.phpt
Zend/tests/arrow_functions/007.phpt
Zend/tests/assert/bug70241.phpt
Zend/tests/assert/expect_015.phpt
Zend/tests/attributes/deprecated/class_constants/deprecated_constant_as_message_002.phpt
Zend/tests/attributes/deprecated/class_constants/error_code_001.phpt
Zend/tests/attributes/deprecated/constants/deprecated_constant_as_message_002.phpt
Zend/tests/attributes/deprecated/constants/error_code.phpt
Zend/tests/attributes/deprecated/functions/deprecated_handler_002.phpt
Zend/tests/attributes/deprecated/functions/error_code_001.phpt
Zend/tests/attributes/deprecated/functions/throwing_error_handler_001.phpt
Zend/tests/attributes/deprecated/functions/throwing_error_handler_002.phpt
Zend/tests/attributes/deprecated/functions/throwing_error_handler_003.phpt
Zend/tests/attributes/deprecated/traits/throwing_error_handler.phpt
Zend/tests/attributes/nodiscard/error_code_001.phpt
Zend/tests/attributes/nodiscard/throwing_error_handler_001.phpt
Zend/tests/autoload/bug78868.phpt
Zend/tests/backtrace/bug76047.phpt
Zend/tests/bind_static_exception.phpt
Zend/tests/bug26802.phpt
Zend/tests/bug28072.phpt
Zend/tests/bug28442.phpt
Zend/tests/bug32322.phpt
Zend/tests/bug35163_2.phpt
Zend/tests/bug35470.phpt
Zend/tests/bug38211.phpt
Zend/tests/bug38287.phpt
ext/standard/tests/array/array_combine.phpt
ext/standard/tests/array/array_filter_invalid_mode.phpt
ext/standard/tests/array/array_filter_object.phpt
ext/standard/tests/array/array_map_object1.phpt
```

## Why This Is A Blocker

The 35 rows are high-yield but not one credible narrow implementation patch:

- Nullable type hints need parser and IR metadata for `?T`, parameter and return
  coercion/diagnostics, default-value interaction, by-reference calls, closure
  metadata, and reflection-visible type state.
- Function-local `static` variables need persistent storage keyed by function or
  method identity, one-time initializer evaluation, recursion behavior,
  reference identity, closure/method binding, and shutdown visibility.
- Variable variables require a runtime symbol-table lookup/mutation boundary
  that integrates with locals, globals, references, dynamic roots, and future
  fallback zones.
- Generator/yield support needs suspension frames, iterator state, send/throw
  boundaries, cleanup, and by-reference diagnostics.
- Named arguments for internal calls need a shared internal-call binder rather
  than per-helper argument reshuffling.

Reopening the cluster before those generic layers exist would turn explicit
unsupported-language classifications into noisy parser, type, and runtime
failures. The next split should implement one of these surfaces end to end and
then reclassify this manifest.

## Verification

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only
tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-function-dynamic-current-ptn-rlzz-manifest.txt
PHPT_PROGRESS_DIR=.runtime/ptn-rlzz-focused-current \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-function-dynamic-current-ptn-rlzz-manifest.txt
cargo fmt --check
```
