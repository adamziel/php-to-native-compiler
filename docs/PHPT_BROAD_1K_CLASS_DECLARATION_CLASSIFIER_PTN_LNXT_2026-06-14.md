# PHPT Broad 1k Class Declaration Classifier: ptn-lnxt

Issue: `ptn-lnxt`

The original `ptn-lnxt` branch split the broad 1k
`unsupported-class-declaration` aggregate into explicit class-declaration
blocker buckets:

| Generic blocker | Rows |
| --- | ---: |
| `unsupported-trait-declaration` | 25 |
| `unsupported-interface-declaration` | 23 |
| `unsupported-interface-implementation` | 15 |
| `unsupported-anonymous-class` | 15 |

That behavior is already active on current `master` through the later
`ptn-gkvr` split. The stale `ptn-lnxt` branch was based on `80ca9dfc3` and
would revert newer classifier evidence if merged directly, so this integration
keeps the current classifier/test implementation and records the `ptn-lnxt`
evidence against that current state.

The committed focused manifest remains:

```text
tools/phpt-class-declaration-frontier-manifest.txt
```

Aggregate class-declaration evidence is documented in
`PHPT_BROAD_1K_CLASS_DECLARATION_CATEGORY_PTN_BO7Q_2026-06-14.md`, while the
current explicit split is documented in
`PHPT_BROAD_1K_CLASS_DECLARATION_SPLIT_PTN_GKVR_2026-06-14.md`.

## Current Replay

Command:

```text
PHPT_PROGRESS_DIR=.runtime/ptn-lnxt-class-declaration-current \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-class-declaration-frontier-manifest.txt
```

Artifacts:

```text
.runtime/ptn-lnxt-class-declaration-current/manifest-20260614T111526Z.txt
.runtime/ptn-lnxt-class-declaration-current/classification-20260614T111526Z.tsv
.runtime/ptn-lnxt-class-declaration-current/excluded-20260614T111526Z.tsv
```

Current result:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 78 | 0 | 78 |

| Classification | Rows |
| --- | ---: |
| `unsupported-trait-declaration` | 25 |
| `unsupported-interface-declaration` | 23 |
| `unsupported-interface-implementation` | 15 |
| `unsupported-anonymous-class` | 15 |

The rows remain excluded until PTN has generic anonymous-class, interface
declaration, interface implementation, and trait declaration semantics.

## Verification

```text
bash -n tools/phpt-classifier.sh
cargo fmt --check
cargo test --test phpt_classifier
PHPT_PROGRESS_DIR=.runtime/ptn-lnxt-class-declaration-current \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-class-declaration-frontier-manifest.txt
```

Results:

- `bash -n tools/phpt-classifier.sh`: passed.
- `cargo fmt --check`: passed.
- `cargo test --test phpt_classifier`: passed, 35 tests.
- Focused class-declaration replay: 78 selected, 0 runnable, 78 excluded
  across the four current class-declaration buckets.
