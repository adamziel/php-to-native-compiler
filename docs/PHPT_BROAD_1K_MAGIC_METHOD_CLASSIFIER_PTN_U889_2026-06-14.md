# PHPT Broad 1k Magic Method Classifier Split: 2026-06-14 ptn-u889

Issue: `ptn-u889`

This slice records the broad 1k magic-method object metadata classifier split
out of the aggregate `unsupported-class-metadata` bucket. Current master already
has the `unsupported-magic-method-metadata` bucket and the later `ptn-7fym`
focused manifest; this integration keeps the split evidence and adds direct
classifier coverage for representative magic hooks plus the sharper diagnostic
reason. It is a classifier precision change, not a runtime support claim: the
affected rows still need generic class metadata, magic dispatch, object
conversion, property hook, and reflection semantics before they should run as
native PHPT coverage.

## Before

The current pre-split broad map in
`docs/PHPT_BROAD_1K_CURRENT_COVERAGE_PTN_FPG4_2026-06-14.md` recorded:

```text
runtime artifact: .runtime/phpt-progress/classification-20260614T090208Z.tsv
selected: 1000
runnable: 424
excluded: 576
unsupported-class-metadata: 135
unsupported-magic-method-metadata: 0
```

The existing focused magic-method frontier in
`docs/PHPT_BROAD_1K_MAGIC_METHOD_METADATA_FRONTIER_2026-06-14.md` selected 69
rows from that aggregate class-metadata bucket with:

```sh
awk -F'\t' '$2=="unsupported-class-metadata" && $3 ~ /magic method/ {print $1}' \
  .runtime/phpt-progress/classification-20260614T025213Z.tsv
```

Focused pre-split result:

| Selected | Runnable | Excluded | Bucket |
| ---: | ---: | ---: | --- |
| 69 | 0 | 69 | `unsupported-class-metadata` |

## After

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-u889-after-rebased
```

Generated broad manifest:

```text
.runtime/ptn-u889-after-rebased/20260614T101034Z/phpt-baseline-1000.txt
```

Classification artifacts:

```text
.runtime/phpt-progress/classification-20260614T101034Z.tsv
.runtime/phpt-progress/runnable-20260614T101034Z.txt
.runtime/phpt-progress/excluded-20260614T101034Z.tsv
```

State:

```text
PTN commit: 71dbfe8ec469
php-src PHPT corpus: /home/claude/php-src-phpt
corpus revision: 8c63ec400ce8e07c57a8d9499317b96a8beafb8b
```

Broad 1k result:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 1000 | 424 | 576 |

Relevant bucket movement:

| Bucket | Before | After |
| --- | ---: | ---: |
| `unsupported-class-metadata` | 135 | 66 |
| `unsupported-magic-method-metadata` | 0 | 69 |
| Combined class plus magic blockers | 135 | 135 |

Focused after verification:

```sh
tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-magic-method-metadata-frontier-manifest.txt
```

Artifact:

```text
.runtime/phpt-progress/classification-20260614T100959Z.tsv
```

Focused after result:

| Selected | Runnable | Excluded | Bucket |
| ---: | ---: | ---: | --- |
| 69 | 0 | 69 | `unsupported-magic-method-metadata` |

The current `ptn-7fym` manifest replay verifies the same category on current
master: 69 selected, 0 runnable, 69 excluded as
`unsupported-magic-method-metadata`.

## Boundary

The split is generic: it keys on declared magic method metadata in PHPT
`--FILE--` code after string and comment stripping, not on row paths or expected
output. Rows remain excluded until the compiler/runtime can model:

- magic method availability, visibility, staticness, and signature validation
  in class metadata;
- `__get`, `__set`, `__isset`, and `__unset` dispatch for object property
  access with PHP-compatible recursion and visibility behavior;
- object-to-string and object comparison/keying conversion through shared
  magic dispatch paths used by array helpers;
- debug, backtrace, and reflection metadata for magic methods.

## Verification

```sh
bash -n tools/phpt-classifier.sh
cargo test --test phpt_classifier
PHPT_PROGRESS_DIR=.runtime/ptn-u889-magic-method-current \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-magic-method-metadata-current-ptn-7fym-manifest.txt
tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-magic-method-metadata-frontier-manifest.txt
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-u889-after-rebased
```
