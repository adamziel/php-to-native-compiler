# PHPT Broad 1k Magic Method Metadata Current Map: ptn-xdl8

Issue: `ptn-xdl8`

The original `ptn-xdl8` branch recorded the broad 1k
`unsupported-magic-method-metadata` frontier after the class/object metadata
classifier split. The branch was based on `fe389e53c` and predates several
later evidence docs and focused manifests, so this integration keeps current
`master` state and records the magic-method evidence against the already-active
bucket.

This is a blocker map, not a runtime behavior change. The rows remain excluded
until PTN has generic class/object metadata and magic dispatch semantics.

## Current Manifest

The current committed focused manifest is:

```text
tools/phpt-magic-method-metadata-current-ptn-7fym-manifest.txt
```

It covers 69 broad PHPT rows classified as
`unsupported-magic-method-metadata`. Existing current evidence is documented in
`PHPT_BROAD_1K_MAGIC_METHOD_METADATA_CURRENT_PTN_7FYM_2026-06-14.md`, with
classifier-split reconciliation in
`PHPT_BROAD_1K_MAGIC_METHOD_CLASSIFIER_PTN_U889_2026-06-14.md` and
`PHPT_BROAD_1K_MAGIC_METHOD_CLASSIFIER_PTN_LMO1_2026-06-14.md`.

## Current Replay

Command:

```text
PHPT_PROGRESS_DIR=.runtime/ptn-xdl8-magic-method-current \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-magic-method-metadata-current-ptn-7fym-manifest.txt
```

Artifacts:

```text
.runtime/ptn-xdl8-magic-method-current/manifest-20260614T113350Z.txt
.runtime/ptn-xdl8-magic-method-current/classification-20260614T113350Z.tsv
.runtime/ptn-xdl8-magic-method-current/excluded-20260614T113350Z.tsv
```

Result:

| Selected | Runnable | Excluded | Bucket |
| ---: | ---: | ---: | --- |
| 69 | 0 | 69 | `unsupported-magic-method-metadata` |

The 69 rows are concentrated in object conversion, property magic hooks,
array-helper comparison/key paths, backtrace metadata, destructor timing, and
visibility interactions. Reopening them needs generic support for:

- declared magic method metadata, visibility, staticness, signatures, and
  reflection-visible method information;
- object property dispatch through `__get`, `__set`, `__isset`, and `__unset`;
- object-to-string conversion through `__toString` with PHP warning,
  exception, and evaluation-order behavior;
- magic call dispatch and diagnostics in direct calls, callbacks, array
  helpers, and backtrace frames;
- destructor registration and shutdown timing for rows combining magic
  dispatch with destructor or callback metadata.

## Verification

```text
cargo fmt --check
cargo test --test phpt_classifier
PHPT_PROGRESS_DIR=.runtime/ptn-xdl8-magic-method-current \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-magic-method-metadata-current-ptn-7fym-manifest.txt
```

Results:

- `cargo fmt --check`: passed.
- Focused magic-method replay: 69 selected, 0 runnable, 69 excluded as
  `unsupported-magic-method-metadata`.
