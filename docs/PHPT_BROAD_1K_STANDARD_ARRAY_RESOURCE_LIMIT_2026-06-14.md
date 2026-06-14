# PHPT Broad 1k Standard Array Resource-Limit Slice: 2026-06-14

Issue: `ptn-7xxw`

This slice refreshed the broad 1k PHPT classifier on `origin/master`, then
narrowed the largest standard-array family. The broad implementation frontier
remains array helper semantics, but the single safe integrated change here is a
generic classifier guard for PHPT rows that intentionally require
multi-billion-element `array_fill()` allocation behavior.

## Broad 1k Evidence

Before change:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only
```

Generated manifest:
`.runtime/phpt-baseline/20260614T010237Z/phpt-baseline-1000.txt`

Classifier artifacts:

- `.runtime/phpt-progress/classification-20260614T010237Z.tsv`
- `.runtime/phpt-progress/runnable-20260614T010237Z.txt`
- `.runtime/phpt-progress/excluded-20260614T010237Z.tsv`

Before result: 1,000 selected, 447 runnable, 553 excluded.

After change before rebasing over `ptn-lrlt`/`ptn-oz24`:

```sh
tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  .runtime/phpt-baseline/20260614T010237Z/phpt-baseline-1000.txt
```

Classifier artifacts:

- `.runtime/phpt-progress/classification-20260614T012327Z.tsv`
- `.runtime/phpt-progress/runnable-20260614T012327Z.txt`
- `.runtime/phpt-progress/excluded-20260614T012327Z.tsv`

After result: 1,000 selected, 446 runnable, 554 excluded.

After rebasing over the runtime-diagnostics and attribute blocker classifiers:

```sh
tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  .runtime/phpt-baseline/20260614T010237Z/phpt-baseline-1000.txt
```

Classifier artifacts:

- `.runtime/phpt-progress/classification-20260614T013141Z.tsv`
- `.runtime/phpt-progress/runnable-20260614T013141Z.txt`
- `.runtime/phpt-progress/excluded-20260614T013141Z.tsv`

Current rebased result: 1,000 selected, 421 runnable, 579 excluded, including
one `unsupported-resource-limit` row.

Newly classified broad row:

```text
ext/standard/tests/array/array_fill_error2.phpt
```

The row remains in the `unsupported-resource-limit` bucket because it requires
PHP memory allocation failure/resource-limit diagnostics for a
multi-billion-element `array_fill()` request. Running it as a normal focused
PHPT row can leave long-lived generated native processes and does not measure a
currently modeled PHP semantic surface.

The classifier intentionally keeps this adjacent row runnable:

```text
ext/standard/tests/array/array_fill_variation6.phpt
```

That row uses `PHP_INT_MAX` as the start key with count `1`, so it exercises
array key overflow behavior rather than huge allocation.

## Focused Standard-Array Evidence

The current rebased broad 1k runnable set contains 275
`ext/standard/tests/array` rows. Top runnable families from
`.runtime/phpt-progress/classification-20260614T013141Z.tsv`:

| Family | Rows |
| --- | ---: |
| `array_diff*` | 39 |
| `array_chunk*` | 32 |
| other array rows | 25 |
| `array_intersect*` | 23 |
| `array_map*` | 19 |
| `array_sum*` | 12 |
| `array_filter*` | 10 |
| `array_slice*` | 9 |
| `array_merge_recursive*` | 9 |
| `array_merge*` | 9 |

The obvious 32-row `array_chunk()` family is already covered:

```sh
tools/run-bounded-phpt.sh .runtime/ptn-7xxw-array-chunk-broad.txt
```

Focused result: 32 selected, 32 runnable, 32 passed, 0 failed, 0 skipped,
0 warned. Log: `.runtime/phpt-progress/run-20260614T011055Z-manifest.log`.

## Blocker Map

The remaining high-yield standard-array work is not one narrow helper bug:

| Rows | Cluster | Current blocker shape |
| ---: | --- | --- |
| 62 | `array_diff*` / `array_intersect*` | PHP comparison stringification, binary and multiline string parity, catchable argument diagnostics, and callback validation. |
| 19 | `array_map*` | Callback dispatch diagnostics, null callback/zipping behavior, object callback support, and reference details. |
| 10 | `array_filter*` | Mode constants, callback diagnostics, and key/value argument shape. |
| 9 | `array_slice*` | Offset/length/key-preservation and reference visibility edge parity. |
| 9 | `array_merge*` | Reindexing plus reference/COW visibility. |
| 9 | `array_merge_recursive*` | Recursive merge and reference semantics. |

The largest credible next implementation split remains the shared callback
argument validation path for internal array helpers, followed separately by
set-operation comparison/stringification and array mutation/COW parity.

## Verification

```sh
cargo test --test phpt_classifier
tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  .runtime/phpt-baseline/20260614T010237Z/phpt-baseline-1000.txt
tools/run-bounded-phpt.sh .runtime/ptn-7xxw-array-chunk-broad.txt
```
