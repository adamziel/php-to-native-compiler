# PHPT Broad 1k Class Declaration Category: 2026-06-14 ptn-bo7q

Issue: `ptn-bo7q`

This slice refreshes the broad 1k PHPT evidence after the language classifier
split and records the now-explicit `unsupported-class-declaration` category.
It is a blocker map, not a runtime implementation claim.

The current classifier no longer leaves these rows in a generic
`unsupported-language` bucket. It assigns them to the class-declaration
frontier that needs generic interface, trait, implementation-check, anonymous
class, and related class metadata work before the rows should run.

## Broad 1k Evidence

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-bo7q-baseline-1k
```

Generated broad manifest:

```text
.runtime/ptn-bo7q-baseline-1k/20260614T095127Z/phpt-baseline-1000.txt
```

Classifier artifacts:

```text
.runtime/phpt-progress/summary-20260614T095127Z.txt
.runtime/phpt-progress/classification-20260614T095127Z.tsv
.runtime/phpt-progress/runnable-20260614T095127Z.txt
.runtime/phpt-progress/excluded-20260614T095127Z.tsv
```

State:

```text
PTN: 79945bc00ea9
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
tools/phpt-class-declaration-frontier-manifest.txt
```

The committed manifest's 78 unique rows match the current broad category:

```sh
awk 'NF && $1 !~ /^#/ {print $1}' \
  tools/phpt-class-declaration-frontier-manifest.txt \
  | LC_ALL=C sort -u > .runtime/ptn-bo7q-class-manifest-current.txt
LC_ALL=C sort -u \
  .runtime/phpt-progress/excluded-20260614T095127Z/unsupported-class-declaration.txt \
  > .runtime/ptn-bo7q-class-category-current.txt
comm -3 .runtime/ptn-bo7q-class-manifest-current.txt \
  .runtime/ptn-bo7q-class-category-current.txt
wc -l .runtime/ptn-bo7q-class-manifest-current.txt \
  .runtime/ptn-bo7q-class-category-current.txt
```

Result:

```text
  78 .runtime/ptn-bo7q-class-manifest-current.txt
  78 .runtime/ptn-bo7q-class-category-current.txt
```

Focused verification:

```sh
PHPT_PROGRESS_DIR=.runtime/ptn-bo7q-class-declaration-focused \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-class-declaration-frontier-manifest.txt
```

Focused artifact:

```text
.runtime/ptn-bo7q-class-declaration-focused/classification-20260614T095636Z.tsv
```

Focused result:

| Selected | Runnable | Excluded | Bucket |
| ---: | ---: | ---: | --- |
| 78 | 0 | 78 | `unsupported-class-declaration` |

## Category Split

Reason split from the current broad classifier:

| Generic blocker | Rows |
| --- | ---: |
| Trait declarations | 25 |
| Interface declarations | 23 |
| Interface implementation checks | 15 |
| Anonymous class syntax (`new class`) | 15 |
| Total | 78 |

Path split:

| Path group | Rows |
| --- | ---: |
| `Zend/tests/attributes/` | 34 |
| `Zend/tests/anon/` | 20 |
| `Zend/tests/ArrayAccess/` | 10 |
| `Zend/tests/backtrace/` | 4 |
| `Zend/tests/abstract_implicit.phpt` | 1 |
| `Zend/tests/arg_unpack/` | 1 |
| `Zend/tests/autoload/` | 1 |
| root-level `Zend/tests/bug*.phpt` singles | 5 |
| `ext/standard/tests/array/` | 1 |
| `tests/basic/` | 1 |

## Implementation Boundary

These rows should stay classified until the class declaration model is broader
than the current bounded class metadata support. Opening them generically needs:

- interface declarations, method/constant tables, and implementation contract
  checks;
- trait composition, adaptations, conflict diagnostics, and method provenance;
- anonymous class expressions with generated names, lexical scope, constructor
  dispatch, inheritance, and reflection metadata;
- integration with attributes such as `Override` and `ReturnTypeWillChange`;
- runtime behavior for `ArrayAccess`, autoload, backtrace metadata, and object
  dispatch surfaces that depend on those declarations.

The category is above the 25-row target, but it is not a credible narrow patch:
it spans parser/AST, semantic class tables, metadata validation, and runtime
dispatch. The committed focused manifest gives the next implementation slice a
stable target without reopening those rows as noisy runtime failures.

## Verification

```sh
cargo fmt --check
cargo test --test phpt_classifier
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-bo7q-baseline-1k
PHPT_PROGRESS_DIR=.runtime/ptn-bo7q-class-declaration-focused \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-class-declaration-frontier-manifest.txt
```
