# PHPT Broad 1k Call Unpacking Category: 2026-06-14 ptn-wd68

Issue: `ptn-wd68`

This slice refreshes the broad 1k PHPT evidence after the language classifier
split and records the now-explicit `unsupported-call-unpacking` category. It is
a blocker map, not a runtime implementation claim.

The category covers call-site spread arguments and array unpacking. PTN already
has variadic parameter support in bounded paths, but it does not yet model
source-level unpacking at call sites or in array literals as a generic compiler
and runtime surface.

## Broad 1k Evidence

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-wd68-baseline-1k
```

Generated broad manifest:

```text
.runtime/ptn-wd68-baseline-1k/20260614T100635Z/phpt-baseline-1000.txt
```

Classifier artifacts:

```text
.runtime/phpt-progress/summary-20260614T100636Z.txt
.runtime/phpt-progress/classification-20260614T100636Z.tsv
.runtime/phpt-progress/runnable-20260614T100636Z.txt
.runtime/phpt-progress/excluded-20260614T100636Z.tsv
```

State:

```text
PTN: ca37037df918
php-src PHPT corpus revision: 8c63ec400ce8e07c57a8d9499317b96a8beafb8b
```

Broad result:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 1,000 | 424 | 576 |

Relevant classifier buckets:

| Classification | Rows |
| --- | ---: |
| `unsupported-attribute-metadata` | 149 |
| `unsupported-class-metadata` | 135 |
| `unsupported-class-declaration` | 78 |
| `unsupported-call-unpacking` | 34 |
| `unsupported-request-input-ini` | 28 |
| `unsupported-extension` | 20 |
| `unsupported-diagnostics-runtime` | 17 |
| `unsupported-assertion-ini` | 17 |

## Focused Category Evidence

Committed manifest:

```text
tools/phpt-call-unpacking-frontier-ptn-wd68-manifest.txt
```

It was copied from:

```text
.runtime/phpt-progress/excluded-20260614T100636Z/unsupported-call-unpacking.txt
```

Focused verification:

```sh
PHPT_PROGRESS_DIR=.runtime/ptn-wd68-call-unpacking-current \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-call-unpacking-frontier-ptn-wd68-manifest.txt
```

Focused artifact:

```text
.runtime/ptn-wd68-call-unpacking-current/classification-20260614T102301Z.tsv
```

Focused result:

| Selected | Runnable | Excluded | Bucket |
| ---: | ---: | ---: | --- |
| 34 | 0 | 34 | `unsupported-call-unpacking` |

## Category Split

Reason split from the current broad classifier:

| Generic blocker | Rows |
| --- | ---: |
| Call-site or array unpacking (`...`) | 34 |

Path split:

| Path group | Rows |
| --- | ---: |
| `Zend/tests/arg_unpack/` | 13 |
| `Zend/tests/array_unpack/` | 13 |
| `ext/standard/tests/array/` | 6 |
| `Zend/tests/array_unpack_string_keys.phpt` | 1 |
| `Zend/tests/arrow_functions/` | 1 |

## Implementation Boundary

These rows need a generic unpacking implementation, not PHPT-specific row
handling:

- parser and AST representation for unpacked call arguments and array elements;
- IR lowering that preserves PHP evaluation order across ordinary and unpacked
  operands;
- call dispatch expansion with named/positional argument merge rules,
  duplicate-key diagnostics, by-reference binding, and internal-function
  argument parsing;
- array literal spread semantics for integer and string keys, invalid operand
  diagnostics, reference/COW behavior, and traversable operands;
- diagnostics for unsupported spread positions such as destructuring contexts.

The category is above the 25-row target, but it spans parser, IR, calls,
arrays, diagnostics, and traversable boundaries. The focused manifest gives the
future implementation a stable target while keeping the current broad baseline
classified rather than noisy.

## Verification

```sh
cargo fmt --check
cargo test --test phpt_classifier
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-wd68-baseline-1k
PHPT_PROGRESS_DIR=.runtime/ptn-wd68-call-unpacking-current \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-call-unpacking-frontier-ptn-wd68-manifest.txt
```
