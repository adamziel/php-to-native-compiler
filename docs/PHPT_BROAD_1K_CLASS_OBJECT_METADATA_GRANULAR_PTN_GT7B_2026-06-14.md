# PHPT Broad 1k Class/Object Metadata Granular Frontier: 2026-06-14 ptn-gt7b

Issue: `ptn-gt7b`

This slice refreshes the broad 1k PHPT classifier from the `ptn-gt7b` branch
and records the class/object metadata rows after the aggregate metadata bucket
was split into explicit blocker categories. It is a blocker map, not a runtime
implementation claim.

The branch evidence was collected at PTN `f2a73c767658`. When integrated on
current `master`, class-declaration rows had already been split into trait,
interface, implementation-check, and anonymous-class buckets. The broad bucket
table below is reconciled to those current names while preserving the
class/object metadata manifest and blocker boundary from the branch evidence.

## Broad 1k Evidence

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-gt7b-baseline
```

Generated broad manifest:

```text
.runtime/ptn-gt7b-baseline/20260614T103842Z/phpt-baseline-1000.txt
```

Classifier artifacts:

```text
.runtime/phpt-progress/summary-20260614T103843Z.txt
.runtime/phpt-progress/classification-20260614T103843Z.tsv
.runtime/phpt-progress/runnable-20260614T103843Z.txt
.runtime/phpt-progress/excluded-20260614T103843Z.tsv
```

State:

```text
PTN: f2a73c767658
php-src PHPT corpus revision: 8c63ec400ce8e07c57a8d9499317b96a8beafb8b
```

Broad result:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 1,000 | 424 | 576 |

Largest classifier buckets:

| Classification | Rows |
| --- | ---: |
| `unsupported-attribute-metadata` | 149 |
| `unsupported-magic-method-metadata` | 69 |
| `unsupported-call-unpacking` | 34 |
| `unsupported-request-input-ini` | 28 |
| `unsupported-trait-declaration` | 25 |
| `unsupported-interface-declaration` | 23 |
| `unsupported-extension` | 20 |
| `unsupported-property-visibility-metadata` | 19 |
| `unsupported-diagnostics-runtime` | 17 |
| `unsupported-assertion-ini` | 17 |
| `unsupported-interface-implementation` | 15 |
| `unsupported-anonymous-class` | 15 |
| `unsupported-resource-limit-ini` | 15 |

## Focused Metadata Manifest

Committed focused manifest:

```text
tools/phpt-class-object-metadata-granular-ptn-gt7b-manifest.txt
```

Selection:

```sh
awk -F'\t' '$2 ~ /^unsupported-(magic-method|property-visibility|typed-property|class-contract|method-visibility|readonly-property|autoload|internal-reflection)-metadata$/ {print $1}' \
  .runtime/phpt-progress/classification-20260614T103843Z.tsv
```

Focused replay:

```sh
PHPT_PROGRESS_DIR=.runtime/ptn-gt7b-class-object-metadata-focused \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-class-object-metadata-granular-ptn-gt7b-manifest.txt
```

Focused artifact:

```text
.runtime/ptn-gt7b-class-object-metadata-focused/classification-20260614T104429Z.tsv
```

Focused result:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 135 | 0 | 135 |

After rebasing the branch over `fe389e53c`, the same focused replay on
`efc87ce40bcf` wrote:

```text
.runtime/ptn-gt7b-class-object-metadata-focused-rebased/classification-20260614T104810Z.tsv
```

with the same 135 selected, 0 runnable, and 135 excluded result.

Current integration replay on `master` wrote:

```text
.runtime/ptn-gt7b-class-object-metadata-current/classification-20260614T112756Z.tsv
```

with the same 135 selected, 0 runnable, and 135 excluded result:

| Classification | Rows |
| --- | ---: |
| `unsupported-magic-method-metadata` | 69 |
| `unsupported-property-visibility-metadata` | 19 |
| `unsupported-typed-property-metadata` | 12 |
| `unsupported-class-contract-metadata` | 9 |
| `unsupported-autoload-metadata` | 9 |
| `unsupported-method-visibility-metadata` | 7 |
| `unsupported-readonly-property-metadata` | 7 |
| `unsupported-internal-reflection-metadata` | 3 |

The focused manifest excludes exactly the granular class/object metadata rows
that replace the older aggregate `unsupported-class-metadata` broad bucket.
Attribute metadata remains in its separate 149-row bucket, and class
declaration syntax work remains in the current trait, interface,
implementation-check, and anonymous-class buckets.

## Reason Split

| Blocker | Rows |
| --- | ---: |
| Unsupported magic method dispatch/reflection metadata | 69 |
| Non-public property visibility metadata | 19 |
| Typed property metadata | 12 |
| Runtime class autoload symbol-table mutation | 9 |
| Non-public method visibility dispatch and diagnostics | 7 |
| Indirect readonly property mutation diagnostics | 7 |
| Abstract class/method contract metadata | 5 |
| Final class/method override metadata | 4 |
| Complete internal arginfo/class registry reflection | 3 |

## Path Concentration

| Path family | Rows |
| --- | ---: |
| `ext/standard/tests/array` | 70 |
| Top-level `Zend/tests` rows | 26 |
| `Zend/tests/asymmetric_visibility` | 16 |
| `Zend/tests/access_modifiers` | 12 |
| `Zend/tests/autoload` | 9 |
| `Zend/tests/backtrace` | 2 |

## Blocker Boundary

These rows should remain classified until class/object metadata is represented
as a shared compiler and runtime service. Opening them as runnable today would
turn missing generic metadata into noisy parser/runtime failures.

The next implementation boundary is:

1. Store method metadata for visibility, abstract/final contracts, magic method
   availability, and callback/reflection dispatch.
2. Store property metadata for private/protected visibility, typed properties,
   readonly indirect writes, inheritance, and property-table export.
3. Add autoload/runtime class-table mutation boundaries so class availability
   can change after compile-time collection.
4. Reuse the same metadata from array helper object handling, reflection
   helpers, backtrace diagnostics, and class dispatch.

## Verification

```sh
cargo fmt --check
cargo test --test phpt_classifier
PHPT_PROGRESS_DIR=.runtime/ptn-gt7b-class-object-metadata-focused \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-class-object-metadata-granular-ptn-gt7b-manifest.txt
```

The broad classify-only artifacts listed above are branch-collected evidence.
This integration is documentation/manifest-only and was checked on current
`master` with `cargo fmt --check`, `cargo test --test phpt_classifier`, the
focused 135-row replay, and repository diff sanity checks.
