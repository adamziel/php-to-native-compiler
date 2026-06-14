# PHPT Broad 1k Class Declaration Classifier Split: ptn-gkvr

Issue: `ptn-gkvr`

This slice splits the existing broad 1k class-declaration blocker bucket into
the generic compiler/runtime subsystems required by the rows. It is not a
runtime support claim: all affected rows remain classified and excluded until
PTN has the corresponding class-table, metadata, and dispatch semantics.

## Before

Broad 1k command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-gkvr-baseline-before
```

Artifacts:

```text
.runtime/ptn-gkvr-baseline-before/20260614T095417Z/phpt-baseline-1000.txt
.runtime/phpt-progress/classification-20260614T095417Z.tsv
.runtime/phpt-progress/excluded-20260614T095417Z.tsv
```

Broad result:

| Selected | Runnable | Excluded | `unsupported-class-declaration` |
| ---: | ---: | ---: | ---: |
| 1,000 | 424 | 576 | 78 |

Focused command:

```sh
PHPT_PROGRESS_DIR=.runtime/ptn-gkvr-class-decl-before \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-class-declaration-frontier-manifest.txt
```

Focused result:

| Selected | Runnable | Excluded | `unsupported-class-declaration` |
| ---: | ---: | ---: | ---: |
| 78 | 0 | 78 | 78 |

## After

The same 78 rows remain excluded, but the aggregate bucket is split by the
missing generic subsystem:

| Category | Rows | Required subsystem |
| --- | ---: | --- |
| `unsupported-trait-declaration` | 25 | Trait declarations, composition, aliases, precedence, and conflict diagnostics. |
| `unsupported-interface-declaration` | 23 | Interface declarations, constants, method contracts, and interface metadata tables. |
| `unsupported-interface-implementation` | 15 | Implementation checks, method compatibility validation, and runtime interface metadata. |
| `unsupported-anonymous-class` | 15 | Anonymous class synthesis, generated metadata, constructor dispatch, and reflection naming. |

Broad after command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-gkvr-baseline-after
```

Artifacts:

```text
.runtime/ptn-gkvr-baseline-after/20260614T095957Z/phpt-baseline-1000.txt
.runtime/phpt-progress/classification-20260614T095957Z.tsv
.runtime/phpt-progress/excluded-20260614T095957Z.tsv
```

Broad result:

| Selected | Runnable | Excluded | Newly split rows |
| ---: | ---: | ---: | ---: |
| 1,000 | 424 | 576 | 78 |

Focused after command:

```sh
PHPT_PROGRESS_DIR=.runtime/ptn-gkvr-class-decl-after \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-class-declaration-frontier-manifest.txt
```

Focused result:

| Selected | Runnable | Excluded | Split rows |
| ---: | ---: | ---: | ---: |
| 78 | 0 | 78 | 78 |

## Boundary

These rows should not be made runnable by narrow PHPT fixes. Reopening any
category needs the matching generic class model first:

- traits need parser/AST declarations, method/property import tables, aliasing,
  precedence, and conflict diagnostics;
- interfaces need declaration metadata, constants, inherited interface graphs,
  method contract validation, and reflection exposure;
- implementation checks need compatibility validation and runtime `instanceof`
  and interface-table behavior;
- anonymous classes need expression lowering, generated names, source metadata,
  constructor dispatch, capture/lexical scope integration, and reflection.

## Verification

```sh
bash -n tools/phpt-classifier.sh
cargo fmt --check
cargo test --test phpt_classifier
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-gkvr-baseline-before
PHPT_PROGRESS_DIR=.runtime/ptn-gkvr-class-decl-before \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-class-declaration-frontier-manifest.txt
PHPT_PROGRESS_DIR=.runtime/ptn-gkvr-class-decl-after \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-class-declaration-frontier-manifest.txt
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-gkvr-baseline-after
```

Results:

- `bash -n tools/phpt-classifier.sh`: passed.
- `cargo fmt --check`: passed.
- `cargo test --test phpt_classifier`: passed, 34 tests.
- Broad before: 1,000 selected, 424 runnable, 576 excluded, 78
  `unsupported-class-declaration`.
- Focused before: 78 selected, 0 runnable, 78 excluded as
  `unsupported-class-declaration`.
- Focused after: 78 selected, 0 runnable, 78 excluded across the four split
  categories above.
- Broad after: 1,000 selected, 424 runnable, 576 excluded, with the same 78
  rows split into the four class-declaration categories.
