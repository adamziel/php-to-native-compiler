# PHPT Broad 1k Magic Method Metadata Current Map: 2026-06-14 ptn-7fym

Issue: `ptn-7fym`

This slice records broad 1k PHPT evidence and the now-explicit
`unsupported-magic-method-metadata` category. It is a blocker map, not a
runtime implementation claim. The committed 69-row manifest was replayed on
current `origin/master` after the later class-declaration and runtime-boundary
maps.

Older magic-method maps were produced while these rows still lived under the
coarse `unsupported-class-metadata` bucket. The classifier has split class
metadata into focused categories; this document records the current 69-row
magic-method category and focused replay.

## Broad 1k Evidence

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-7fym-baseline-1k
```

Generated broad manifest:

```text
.runtime/ptn-7fym-baseline-1k/20260614T105254Z/phpt-baseline-1000.txt
```

Classifier artifacts:

```text
.runtime/phpt-progress/summary-20260614T105254Z.txt
.runtime/phpt-progress/classification-20260614T105254Z.tsv
.runtime/phpt-progress/runnable-20260614T105254Z.txt
.runtime/phpt-progress/excluded-20260614T105254Z.tsv
```

State:

```text
PTN: fe389e53cb84
php-src PHPT corpus revision: 8c63ec400ce8e07c57a8d9499317b96a8beafb8b
```

Broad result:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 1,000 | 424 | 576 |

Largest recorded excluded buckets:

| Classification | Rows |
| --- | ---: |
| `unsupported-attribute-metadata` | 149 |
| `unsupported-class-declaration` (later split by `ptn-gkvr`) | 78 |
| `unsupported-magic-method-metadata` | 69 |
| `unsupported-call-unpacking` | 34 |
| `unsupported-request-input-ini` | 28 |
| `unsupported-extension` | 20 |
| `unsupported-property-visibility-metadata` | 19 |
| `unsupported-diagnostics-runtime` | 17 |
| `unsupported-assertion-ini` | 17 |
| `unsupported-resource-limit-ini` | 15 |

## Focused Category Evidence

Committed manifest:

```text
tools/phpt-magic-method-metadata-current-ptn-7fym-manifest.txt
```

It was copied from:

```text
.runtime/phpt-progress/excluded-20260614T105254Z/unsupported-magic-method-metadata.txt
```

Focused verification:

```sh
PHPT_PROGRESS_DIR=.runtime/ptn-7fym-magic-method-focused \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-magic-method-metadata-current-ptn-7fym-manifest.txt
```

Focused artifact:

```text
.runtime/ptn-7fym-magic-method-focused/classification-20260614T105758Z.tsv
```

Focused result:

| Selected | Runnable | Excluded | Bucket |
| ---: | ---: | ---: | --- |
| 69 | 0 | 69 | `unsupported-magic-method-metadata` |

## Category Split

Path split:

| Path group | Rows |
| --- | ---: |
| `ext/standard/tests/array/` | 60 |
| `Zend/tests/asymmetric_visibility/` | 4 |
| Root `Zend/tests/` | 4 |
| `Zend/tests/backtrace/` | 1 |

Magic method name occurrences across the selected PHPT sources:

| Magic method | Occurrences |
| --- | ---: |
| `__toString` | 64 |
| `__call` | 4 |
| `__construct` | 4 |
| `__destruct` | 4 |
| `__set` | 2 |
| `__unset` | 2 |
| `__get` | 1 |
| `__isset` | 1 |

The 60 standard-array rows use array helpers as a compatibility probe for
object string conversion, property access hooks, comparator callbacks, and
debug/reflection metadata. The Zend rows cover asymmetric visibility hooks,
destructor/backtrace behavior, and broader magic dispatch metadata.

## Implementation Boundary

These rows need generic object and class-metadata semantics:

- declared magic method availability, visibility, staticness, and signature
  validation in class metadata;
- shared object-to-string conversion through public `__toString()` with PHP
  diagnostics, exception behavior, and failure propagation;
- property read/write/isset/unset dispatch through `__get`, `__set`,
  `__isset`, and `__unset`, including recursion guards and visibility context;
- array helper loose comparison, key conversion, merge/reverse/set-operation,
  and callback paths routed through the shared object conversion and property
  access machinery;
- destructor timing and stack-frame/reflection metadata for magic dispatch.

This category is above the 25-row target, but it crosses object metadata,
runtime dispatch, array internals, diagnostics, and destructor/backtrace
behavior. It should be reopened by landing generic object-runtime pieces and
then removing the classifier branch for the affected semantic subset, not by
special-casing the selected PHPT rows.

## Verification

```sh
cargo fmt --check
cargo test --test phpt_classifier
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-7fym-baseline-1k
PHPT_PROGRESS_DIR=.runtime/ptn-7fym-magic-method-focused \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-magic-method-metadata-current-ptn-7fym-manifest.txt
```
