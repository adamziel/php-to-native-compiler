# PHPT Broad 1k Parser/Control Classifier Split: 2026-06-14

Issue: `ptn-lv0u`

This slice refines PHPT blocker telemetry rather than compiler/runtime
semantics. The broad 1k classifier already detected multiple parser,
control-flow, and dynamic-symbol blockers, but emitted all of them as the
single `unsupported-language` category. The classifier now keeps the same
runnable/excluded boundary while splitting those rows into semantic blocker
categories.

## Broad 1k Evidence

Runner-recorded source state:

- PTN before artifact: `a01d77d37da7`
- PTN after artifact: `86a5170fa407`
- php-src PHPT corpus: `8c63ec400ce8e07c57a8d9499317b96a8beafb8b`

Generated broad manifest:

```text
.runtime/ptn-lv0u-baseline/20260614T061129Z/phpt-baseline-1000.txt
```

Before:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-lv0u-baseline
```

Artifacts:

```text
.runtime/phpt-progress/summary-20260614T061129Z.txt
.runtime/phpt-progress/classification-20260614T061129Z.tsv
```

After, using the same generated manifest:

```sh
tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  .runtime/ptn-lv0u-baseline/20260614T061129Z/phpt-baseline-1000.txt
```

Artifacts:

```text
.runtime/phpt-progress/summary-20260614T063334Z.txt
.runtime/phpt-progress/classification-20260614T063334Z.tsv
```

Broad classifier totals:

| Run | Selected | Runnable | Excluded |
| --- | ---: | ---: | ---: |
| Before | 1,000 | 424 | 576 |
| After | 1,000 | 425 | 575 |

The one-row global runnable/excluded delta is outside this parser/control
split. The rows affected by this slice were already excluded before and remain
excluded after; only their blocker categories changed.

## Category Movement

The old catch-all bucket:

| Before category | Rows |
| --- | ---: |
| `unsupported-language` | 288 |

The same 288 rows now split as:

| After category | Rows | Shared blocker |
| --- | ---: | --- |
| `unsupported-attribute-syntax` | 141 | PHP `#[...]` parsing plus attribute/reflection metadata. |
| `unsupported-class-declaration` | 78 | Interfaces, implementation checks, traits, and anonymous classes. |
| `unsupported-unpacking` | 34 | Call-site and array unpacking lowering. |
| `unsupported-type-hint` | 14 | Nullable type hints and `never` return control-flow validation. |
| `unsupported-function-state` | 11 | Static local variables and function-local persistent state. |
| `unsupported-dynamic-symbols` | 8 | Variable variables and runtime symbol-table lookup/mutation. |
| `unsupported-generator-runtime` | 1 | Generator/yield lowering and suspension runtime. |
| `unsupported-internal-call-lowering` | 1 | Named-argument binding for modeled array internal calls. |

This is classifier-only movement: the improvement is that broad telemetry now
points at implementation-sized semantic frontiers instead of one coarse
language bucket.

## Verification

```sh
bash -n tools/phpt-classifier.sh
cargo test --test phpt_classifier
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-lv0u-baseline
tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  .runtime/ptn-lv0u-baseline/20260614T061129Z/phpt-baseline-1000.txt
```
