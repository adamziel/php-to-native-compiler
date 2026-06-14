# PHPT Broad 1k Interface Category Frontier: 2026-06-14 ptn-l0h9

Issue: `ptn-l0h9`

This slice isolates the broad 1k interface declaration and interface
implementation blocker rows. It is a blocker map and focused manifest, not a
runtime implementation claim.

## Broad 1k Evidence

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-l0h9-baseline-1k
```

Generated broad manifest:

```text
.runtime/ptn-l0h9-baseline-1k/20260614T113053Z/phpt-baseline-1000.txt
```

Classifier artifacts:

```text
.runtime/phpt-progress/summary-20260614T113053Z.txt
.runtime/phpt-progress/classification-20260614T113053Z.tsv
.runtime/phpt-progress/runnable-20260614T113053Z.txt
.runtime/phpt-progress/excluded-20260614T113053Z.tsv
```

State:

```text
PTN: c3d51e67440c
php-src PHPT corpus revision: 8c63ec400ce8e07c57a8d9499317b96a8beafb8b
```

Broad result:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 1,000 | 424 | 576 |

Interface categories in that broad pass:

| Classification | Rows | Representative rows |
| --- | ---: | --- |
| `unsupported-interface-declaration` | 23 | `Zend/tests/anon/002.phpt`, `Zend/tests/anon/anon_class_name.phpt`, `Zend/tests/anon/bug77652.phpt` |
| `unsupported-interface-implementation` | 15 | `Zend/tests/ArrayAccess/ArrayAccess_indirect_append.phpt`, `Zend/tests/ArrayAccess/bug30346.phpt`, `Zend/tests/ArrayAccess/bug33710.phpt` |

## Focused Interface Manifest

Committed focused manifest:

```text
tools/phpt-interface-current-ptn-l0h9-manifest.txt
```

Selection:

```sh
sort \
  .runtime/phpt-progress/excluded-20260614T113053Z/unsupported-interface-declaration.txt \
  .runtime/phpt-progress/excluded-20260614T113053Z/unsupported-interface-implementation.txt \
  -o tools/phpt-interface-current-ptn-l0h9-manifest.txt
```

Focused replay:

```sh
PHPT_PROGRESS_DIR=.runtime/ptn-l0h9-interface-current \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-interface-current-ptn-l0h9-manifest.txt
```

Focused artifact:

```text
.runtime/ptn-l0h9-interface-current/classification-20260614T115007Z.tsv
```

Focused result:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 38 | 0 | 38 |

Focused split:

| Classification | Rows |
| --- | ---: |
| `unsupported-interface-declaration` | 23 |
| `unsupported-interface-implementation` | 15 |

## Path Concentration

| Path family | Rows |
| --- | ---: |
| `Zend/tests/attributes` | 19 |
| `Zend/tests/ArrayAccess` | 8 |
| Top-level `Zend/tests` rows | 5 |
| `Zend/tests/anon` | 3 |
| `Zend/tests/arg_unpack` | 1 |
| `Zend/tests/autoload` | 1 |
| `ext/standard/tests/array` | 1 |

## Blocker Boundary

These rows should stay excluded until interface semantics are represented as a
shared parser, compiler, and runtime service. Opening the rows as runnable
before that work would convert one missing generic subsystem into scattered
parser, metadata, reflection, dispatch, and diagnostic failures.

The implementation boundary is:

1. Parse and lower interface declarations, interface constants, inherited
   interface lists, and abstract method contracts into class-table metadata.
2. Validate `implements` clauses against declared interfaces, including
   inherited interfaces, method presence, method visibility, and signature
   compatibility.
3. Publish runtime interface metadata for `instanceof`, object dispatch,
   reflection, attributes such as `Override`, and internal interface contracts
   such as `ArrayAccess`.
4. Keep interface metadata consistent with adjacent class-like work: traits,
   attributes, anonymous classes, autoload-time class-table mutation, and
   diagnostics.

## Verification

```sh
cargo fmt --check
cargo test --test phpt_classifier
PHPT_PROGRESS_DIR=.runtime/ptn-l0h9-interface-current \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-interface-current-ptn-l0h9-manifest.txt
```
