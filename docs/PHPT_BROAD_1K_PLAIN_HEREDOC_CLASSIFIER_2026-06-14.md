# PHPT Broad 1k Plain Heredoc Classifier Slice: 2026-06-14

Issue: `ptn-0cex`

This slice aligns broad PHPT classification with the current compiler surface:
plain heredoc and nowdoc literals are already accepted by the lexer/parser and
covered by native tests, while interpolating heredoc bodies still remain
unsupported.

## Baseline

Broad 1k source manifest:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only
```

Artifacts:

- baseline manifest:
  `.runtime/phpt-baseline/20260614T010430Z/phpt-baseline-1000.txt`
- before classification:
  `.runtime/phpt-progress/classification-20260614T010430Z.tsv`

Before this slice, the broad classifier selected 1,000 rows, left 447 runnable,
and excluded 553. The classifier mapped 70 broad rows containing `<<<` to:

```text
unsupported-language  requires heredoc/nowdoc string syntax (`<<<`)
```

## Change

The classifier now tracks heredoc/nowdoc bodies in the `--FILE--` section:

- plain heredoc and nowdoc bodies no longer block classification;
- heredoc bodies containing interpolation remain `unsupported-language`;
- heredoc body text is skipped for unrelated PHP syntax scans until the closing
  label, so body text does not create false blockers.

## After Classification

Focused after-classification command:

```sh
PHPT_PROGRESS_DIR=.runtime/ptn-0cex/after-heredoc-classify \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  .runtime/ptn-0cex/plain-heredoc-before-manifest.txt
```

The 70 previously heredoc-blocked broad rows now split as:

| Classification | Rows |
| --- | ---: |
| `runnable` | 21 |
| `unsupported-class-metadata` | 49 |

Full classify-only rerun on the same broad 1k manifest:

```sh
PHPT_PROGRESS_DIR=.runtime/ptn-0cex/combined-after-classify \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  .runtime/phpt-baseline/20260614T010430Z/phpt-baseline-1000.txt
```

After rebasing over the runtime/attribute classifier updates from `ptn-oz24`,
this slice moves the combined broad 1k classification from 421 runnable and 579
excluded to 442 runnable and 558 excluded:

| Classification | Before | After |
| --- | ---: | ---: |
| `runnable` | 421 | 442 |
| `unsupported-language` | 351 | 281 |
| `unsupported-class-metadata` | 84 | 133 |
| `unsupported-diagnostics-runtime` | 17 | 17 |
| `unsupported-assertion-runtime` | 9 | 9 |
| all other categories | unchanged | unchanged |

## Focused Executable Evidence

The 21 newly runnable rows were run through the native PHPT runner:

```sh
PHPT_PROGRESS_DIR=.runtime/ptn-0cex/final2-run-heredoc-runnable \
  tools/run-bounded-phpt.sh \
  .runtime/ptn-0cex/after-heredoc-classify/runnable-20260614T011804Z.txt
```

Result:

| Selected | Runnable | Passed | Failed | Skipped | Warned |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 21 | 21 | 14 | 7 | 0 | 0 |

Passing rows include `array_fill_basic.phpt`,
`array_intersect_assoc_variation3.phpt`,
`array_intersect_assoc_variation4.phpt`,
`array_filter_variation5.phpt`, `array_key_exists_variation8.phpt`,
`array_merge_variation4.phpt`, `array_push_variation6.phpt`, and
`array_slice_variation7.phpt`.

Failing rows now expose existing array-helper semantic gaps instead of being
hidden behind the broad heredoc syntax blocker:

```text
ext/standard/tests/array/array_combine_variation3.phpt
ext/standard/tests/array/array_flip_variation3.phpt
ext/standard/tests/array/array_intersect_variation3.phpt
ext/standard/tests/array/array_intersect_variation4.phpt
ext/standard/tests/array/array_merge_recursive_variation3.phpt
ext/standard/tests/array/array_pad_variation6.phpt
ext/standard/tests/array/array_rand_variation6.phpt
```

The full broad executable 1k run was not repeated in this slice; prior broad
1k executable runs are known to be noisy under current shared worker load and
can stall before reaching all buckets. The updated broad pass estimate is the
previous 265-pass broad baseline plus these 14 focused newly runnable passes.
