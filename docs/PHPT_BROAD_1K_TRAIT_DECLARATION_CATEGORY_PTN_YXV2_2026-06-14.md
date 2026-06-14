# PHPT Broad 1k Trait Declaration Category: 2026-06-14 ptn-yxv2

Issue: `ptn-yxv2`

This slice refreshes broad 1k PHPT evidence on current `origin/master` and
records the now-explicit `unsupported-trait-declaration` category. It is a
blocker map, not a runtime implementation claim.

The broad class-declaration aggregate is already split into traits,
interfaces, interface implementation checks, and anonymous classes. This
document records the 25-row trait category with a focused manifest so future
trait work has a stable target that is separate from the other class-table
features.

## Broad 1k Evidence

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-yxv2-postrebase-baseline-1k
```

Generated broad manifest:

```text
.runtime/ptn-yxv2-postrebase-baseline-1k/20260614T111331Z/phpt-baseline-1000.txt
```

Classifier artifacts:

```text
.runtime/phpt-progress/summary-20260614T111331Z.txt
.runtime/phpt-progress/classification-20260614T111331Z.tsv
.runtime/phpt-progress/runnable-20260614T111331Z.txt
.runtime/phpt-progress/excluded-20260614T111331Z.tsv
```

State:

```text
PTN: 8afe144675cc
php-src PHPT corpus revision: 8c63ec400ce8e07c57a8d9499317b96a8beafb8b
```

Broad result:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 1,000 | 424 | 576 |

Relevant current excluded buckets:

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
| `unsupported-anonymous-class` | 15 |
| `unsupported-interface-implementation` | 15 |

## Focused Category Evidence

Committed manifest:

```text
tools/phpt-trait-declaration-current-ptn-yxv2-manifest.txt
```

It was copied from:

```text
.runtime/phpt-progress/excluded-20260614T111331Z/unsupported-trait-declaration.txt
```

Focused verification:

```sh
PHPT_PROGRESS_DIR=.runtime/ptn-yxv2-trait-declaration-focused \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-trait-declaration-current-ptn-yxv2-manifest.txt
```

Focused artifact:

```text
.runtime/ptn-yxv2-trait-declaration-focused/classification-20260614T111915Z.tsv
```

Current integration replay:

```text
.runtime/ptn-yxv2-trait-declaration-current/classification-20260614T113107Z.tsv
```

Focused result:

| Selected | Runnable | Excluded | Bucket |
| ---: | ---: | ---: | --- |
| 25 | 0 | 25 | `unsupported-trait-declaration` |

## Category Split

Path split:

| Path group | Rows |
| --- | ---: |
| `Zend/tests/attributes/override/` | 14 |
| `Zend/tests/backtrace/` | 4 |
| `Zend/tests/anon/` | 3 |
| `Zend/tests/ArrayAccess/` | 1 |
| `Zend/tests/attributes/nodiscard/` | 1 |
| Root `Zend/tests/` | 1 |
| `tests/basic/` | 1 |

Representative surfaces in the selected rows:

- trait declarations and `use` composition inside ordinary classes;
- trait method aliases, including `TraitName::method as alias`;
- abstract method requirements supplied through traits;
- trait methods participating in interface contracts such as `ArrayAccess`;
- trait declarations inside rows that also exercise attributes, overrides,
  backtraces, and anonymous classes.

## Implementation Boundary

These rows need generic trait support, not row-level PHPT handling:

- parser and AST nodes for trait declarations and class `use` clauses;
- class metadata registration for traits and imported members;
- method/property import composition with aliasing, precedence, visibility
  changes, conflict detection, and diagnostics;
- abstract method requirement propagation from traits into composed classes;
- trait-aware reflection, backtrace names, attribute target validation, and
  interface compatibility checks;
- runtime dispatch over composed trait methods without losing source metadata.

The 25-row category exactly meets the broad-slice threshold, but it crosses
parser, semantic class-table construction, metadata validation, reflection,
and dispatch. It should be reopened by landing generic trait semantics and
then removing only the classifier branch for the supported subset.

## Verification

```sh
cargo fmt --check
cargo test --test phpt_classifier
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-yxv2-postrebase-baseline-1k
PHPT_PROGRESS_DIR=.runtime/ptn-yxv2-trait-declaration-focused \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-trait-declaration-current-ptn-yxv2-manifest.txt
```
