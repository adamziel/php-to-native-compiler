# PHPT Broad 1k Object String Conversion Classifier: 2026-06-14 ptn-i0p3

Issue: `ptn-i0p3`

This slice splits `__toString()` object conversion rows out of the broad
`unsupported-magic-method-metadata` bucket. It is a classifier/blocker-map
change, not a runtime support claim: these rows need object-to-string metadata
shared by comparisons, array helpers, diagnostics, and reflection before they
can become executable compatibility coverage.

## Broad 1k Before Evidence

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-i0p3-baseline-1k
```

Generated manifest:

```text
.runtime/ptn-i0p3-baseline-1k/20260614T110711Z/phpt-baseline-1000.txt
```

Classifier artifact:

```text
.runtime/phpt-progress/classification-20260614T110712Z.tsv
```

Corpus revision: `8c63ec400ce8e07c57a8d9499317b96a8beafb8b`

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 1000 | 424 | 576 |

Before this split, all 69 focused rows were classified as
`unsupported-magic-method-metadata`. A corpus scan of those 69 rows found 61
rows declaring `__toString()` and 8 residual rows using other magic hooks.

## Classifier Movement

The classifier now emits:

```text
unsupported-object-string-conversion-metadata
```

for declarations of `__toString()`, while `__call`, `__get`, `__set`,
`__isset`, `__unset`, `__debugInfo`, serialization, sleep, and wakeup hooks
remain in `unsupported-magic-method-metadata`.

Focused command:

```sh
PHPT_PROGRESS_DIR=.runtime/ptn-i0p3-object-string-final \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-magic-method-metadata-frontier-manifest.txt
```

Focused artifact for this split:

```text
.runtime/ptn-i0p3-object-string-final/classification-20260614T113038Z.tsv
```

| Manifest | Selected | Runnable | Excluded |
| --- | ---: | ---: | ---: |
| `tools/phpt-magic-method-metadata-frontier-manifest.txt` | 69 | 0 | 69 |

Focused split:

| Bucket | Rows |
| --- | ---: |
| `unsupported-object-string-conversion-metadata` | 61 |
| `unsupported-magic-method-metadata` | 8 |

Committed focused manifest:

```text
tools/phpt-object-string-conversion-metadata-ptn-i0p3-manifest.txt
```

## Broad 1k After Evidence

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-i0p3-baseline-final
```

Generated manifest:

```text
.runtime/ptn-i0p3-baseline-final/20260614T113114Z/phpt-baseline-1000.txt
```

Classifier artifact:

```text
.runtime/phpt-progress/classification-20260614T113114Z.tsv
```

Broad after result:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 1000 | 424 | 576 |

Actual broad after split:

| Bucket | Before | After |
| --- | ---: | ---: |
| `unsupported-magic-method-metadata` | 69 | 8 |
| `unsupported-object-string-conversion-metadata` | 0 | 61 |

The selected/runnable/excluded totals stay stable at 1000/424/576; the change
is the semantic bucket split for the 61 object string-conversion rows.

## Blocker Boundary

These rows are not just method declaration checks. The broad 1k set routes
objects through array helper comparison and key paths, callback helpers,
diagnostics, and backtrace/reflection contexts. Reopening them requires generic
runtime support for:

- storing `__toString()` availability, visibility, and signature metadata in
  class metadata;
- dispatching object-to-string conversion from the shared boxed value
  conversion path;
- preserving PHP exception, warning, and diagnostic behavior when conversion
  fails;
- reusing that conversion path in loose comparisons, array set operations,
  `array_map()`, merge/reverse helpers, and diagnostic rendering.

Keeping this bucket separate from property/method magic hooks makes future
implementation work smaller and avoids treating object conversion rows as
generic magic-method or array-helper failures.

## Verification

```sh
cargo test --test phpt_classifier
PHPT_PROGRESS_DIR=.runtime/ptn-i0p3-object-string-final \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-magic-method-metadata-frontier-manifest.txt
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-i0p3-baseline-final
```
