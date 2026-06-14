# PHPT Broad 1k Function/Dynamic Type Slice: 2026-06-14 ptn-1tco

Issue: `ptn-1tco`

This slice uses the broad 1k PHPT baseline source and maps a parser/control-flow
cluster around function-local `static` variables, nullable type hints, `never`
return types, and variable variables. It is a blocker map, not a runtime support
claim: the rows cross function storage, type metadata, assertion diagnostics,
attribute metadata, and dynamic symbol-table/fallback boundaries.

## Broad 1k Source

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --generate-only \
  --out-dir .runtime/ptn-1tco-baseline-rebased
```

Generated broad manifest:

```text
.runtime/ptn-1tco-baseline-rebased/20260614T092829Z/phpt-baseline-1000.txt
```

Source state:

```text
PTN: d7d8a9bc0af0
php-src PHPT corpus: /home/claude/php-src-phpt
corpus revision: 8c63ec400ce8e07c57a8d9499317b96a8beafb8b
```

The broad tier selected 1,000 rows:

| Source bucket | Rows |
| --- | ---: |
| `Zend/tests` | 530 |
| `ext/standard/tests` | 384 |
| `tests` | 86 |

## Focused Cluster

Committed manifest:

```text
tools/phpt-function-dynamic-type-ptn-1tco-manifest.txt
```

Selection criteria matched the classifier's generic blockers:

- function-local static variables: `static $name` statements;
- nullable parameter or return type hints: `?T`;
- `never` return type declarations;
- variable variables: `$$name` and `${$expr}`.

The current broad 1k source contains 36 unique rows in this cluster:

| Surface | Rows |
| --- | ---: |
| Nullable type hints | 16 |
| Function-local `static` variables | 12 |
| Variable variables | 8 |
| `never` return type | 2 |
| Unique rows after overlap | 36 |

Focused classify-only verification:

```sh
tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-function-dynamic-type-ptn-1tco-manifest.txt
```

Current artifact from the committed focused manifest:

```text
.runtime/ptn-1tco-current/classification-20260614T100900Z.tsv
.runtime/ptn-1tco-current/summary-20260614T100900Z.txt
```

Focused result:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 36 | 0 | 36 |

Classifier split:

| Classification | Rows |
| --- | ---: |
| `unsupported-type-hint` | 14 |
| `unsupported-function-state` | 11 |
| `unsupported-dynamic-symbol` | 8 |
| `unsupported-assertion-ini` | 1 |
| `unsupported-attribute-metadata` | 2 |

## Rows By Surface

Function-local `static` variables:

```text
Zend/tests/anon/015.phpt
Zend/tests/anon/016.phpt
Zend/tests/assert/expect_015.phpt
Zend/tests/autoload/bug78868.phpt
Zend/tests/bind_static_exception.phpt
Zend/tests/bug26802.phpt
Zend/tests/bug28072.phpt
Zend/tests/bug28442.phpt
Zend/tests/bug32322.phpt
Zend/tests/bug38287.phpt
ext/standard/tests/array/array_filter_object.phpt
ext/standard/tests/array/array_map_object1.phpt
```

Nullable type hints:

```text
Zend/tests/arrow_functions/006.phpt
Zend/tests/arrow_functions/007.phpt
Zend/tests/assert/expect_015.phpt
Zend/tests/attributes/deprecated/class_constants/deprecated_constant_as_message_002.phpt
Zend/tests/attributes/deprecated/class_constants/error_code_001.phpt
Zend/tests/attributes/deprecated/constants/deprecated_constant_as_message_002.phpt
Zend/tests/attributes/deprecated/constants/error_code.phpt
Zend/tests/attributes/deprecated/functions/deprecated_handler_002.phpt
Zend/tests/attributes/deprecated/functions/deprecated_handler_003.phpt
Zend/tests/attributes/deprecated/functions/error_code_001.phpt
Zend/tests/attributes/deprecated/functions/throwing_error_handler_001.phpt
Zend/tests/attributes/deprecated/functions/throwing_error_handler_002.phpt
Zend/tests/attributes/deprecated/functions/throwing_error_handler_003.phpt
Zend/tests/attributes/deprecated/traits/throwing_error_handler.phpt
Zend/tests/attributes/nodiscard/error_code_001.phpt
Zend/tests/attributes/nodiscard/throwing_error_handler_001.phpt
```

Variable variables:

```text
Zend/tests/arrow_functions/003.phpt
Zend/tests/arrow_functions/005.phpt
Zend/tests/assert/expect_015.phpt
Zend/tests/backtrace/bug76047.phpt
Zend/tests/bug35163_2.phpt
Zend/tests/bug35470.phpt
Zend/tests/bug38211.phpt
ext/standard/tests/array/array_combine.phpt
```

`never` return types:

```text
Zend/tests/arrow_functions/gh7900.phpt
Zend/tests/attributes/nodiscard/unsupported_never_function.phpt
```

## Implementation Boundary

No single generic patch is credible for all 36 rows:

1. Function-local `static` storage needs per-function and per-method persistent
   slots, initializer timing, recursion behavior, reference identity, closure
   binding, and shutdown/destructor interactions.
2. Nullable type hints need parser/AST/IR metadata and shared parameter,
   variadic, return, closure, method, by-reference, default-value, and
   diagnostic boundaries. Several nullable rows are also attribute metadata
   tests, so nullable support alone would not make them runnable.
3. `never` return types need control-flow validation for normal returns,
   throws, fatal paths, callbacks, and assertion pretty-printing.
4. Variable variables are explicitly in PTN's dynamic fallback zone and need a
   runtime symbol-table lookup/mutation boundary that integrates with globals,
   locals, references, arrays, diagnostics, and future fallback execution.

The next implementation split should choose one of those generic surfaces,
remove only the corresponding classifier branch after support lands, and rerun
this focused manifest plus the broad 1k tier.

## Verification

```sh
tools/run-phpt-baseline.sh --tier 1000 --generate-only \
  --out-dir .runtime/ptn-1tco-baseline-rebased
tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-function-dynamic-type-ptn-1tco-manifest.txt
cargo fmt --check
```
