# PHPT Broad 1k Magic Method Classifier: ptn-lmo1

Issue: `ptn-lmo1`

The original `ptn-lmo1` branch split magic method dispatch/reflection metadata
rows out of broader class metadata accounting and into
`unsupported-magic-method-metadata`. That behavior is already active on current
`master` through the committed magic-method classifier split and the later
current-category evidence.

The stale branch was based on `f2a73c767` and would drop newer PHPT evidence if
merged directly, including later class-declaration and magic-method notes. This
integration keeps the current classifier/test implementation and records the
`ptn-lmo1` evidence against that current state.

The active focused manifest is:

```text
tools/phpt-magic-method-metadata-current-ptn-7fym-manifest.txt
```

Current category evidence is documented in
`PHPT_BROAD_1K_MAGIC_METHOD_METADATA_CURRENT_PTN_7FYM_2026-06-14.md`, and the
current classifier split evidence is documented in
`PHPT_BROAD_1K_MAGIC_METHOD_CLASSIFIER_PTN_U889_2026-06-14.md`.

## Current Replay

Command:

```text
PHPT_PROGRESS_DIR=.runtime/ptn-lmo1-magic-method-current \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-magic-method-metadata-current-ptn-7fym-manifest.txt
```

Artifacts:

```text
.runtime/ptn-lmo1-magic-method-current/manifest-20260614T111755Z.txt
.runtime/ptn-lmo1-magic-method-current/classification-20260614T111755Z.tsv
.runtime/ptn-lmo1-magic-method-current/excluded-20260614T111755Z.tsv
```

Current result:

| Selected | Runnable | Excluded | Bucket |
| ---: | ---: | ---: | --- |
| 69 | 0 | 69 | `unsupported-magic-method-metadata` |

The rows remain excluded until PTN has generic magic method availability,
visibility, signature validation, dispatch, conversion, reflection, and
diagnostic semantics.

## Verification

```text
bash -n tools/phpt-classifier.sh
cargo fmt --check
cargo test --test phpt_classifier
PHPT_PROGRESS_DIR=.runtime/ptn-lmo1-magic-method-current \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-magic-method-metadata-current-ptn-7fym-manifest.txt
```

Results:

- `bash -n tools/phpt-classifier.sh`: passed.
- `cargo fmt --check`: passed.
- `cargo test --test phpt_classifier`: passed, 35 tests.
- Focused magic-method replay: 69 selected, 0 runnable, 69 excluded as
  `unsupported-magic-method-metadata`.
