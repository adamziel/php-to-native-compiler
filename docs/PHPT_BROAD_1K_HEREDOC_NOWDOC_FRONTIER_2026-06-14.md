# PHPT Broad 1k Heredoc/Nowdoc Frontier: 2026-06-14

Issue: `ptn-4fd3`

This slice narrows the broad 1k PHPT baseline's heredoc/nowdoc blocker. PTN
already lexes and lowers plain heredoc and nowdoc strings; only interpolating
heredoc bodies still need broader string-interpolation support. The classifier
now keeps plain heredoc/nowdoc rows runnable and continues to classify
interpolating heredoc rows.

## Broad Baseline

Before:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only --out-dir .runtime/ptn-4fd3-baseline
```

After:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only --out-dir .runtime/ptn-4fd3-baseline-final
```

Both runs used php-src PHPT corpus revision
`8c63ec400ce8e07c57a8d9499317b96a8beafb8b`.

| Run | Classification | Runnable | Excluded | `unsupported-language` | `unsupported-class-metadata` |
| --- | --- | ---: | ---: | ---: | ---: |
| Before | `classification-20260614T013307Z.tsv` | 422 | 578 | 351 | 84 |
| After | `classification-20260614T015945Z.tsv` | 443 | 557 | 281 | 133 |

Net movement: 70 rows left the coarse heredoc/nowdoc language blocker. Of
those, 21 became runnable and 49 now expose the next generic blocker:
unsupported magic method dispatch/reflection metadata.

## Focused Manifest

Committed manifest:
`tools/phpt-heredoc-nowdoc-frontier-manifest.txt`

Selection from the before-run classifier:

```sh
awk -F'\t' '$3 ~ /heredoc\/nowdoc/ {print $1}' \
  .runtime/phpt-progress/classification-20260614T013307Z.tsv
```

Focused run:

```sh
tools/run-bounded-phpt.sh tools/phpt-heredoc-nowdoc-frontier-manifest.txt
```

Result at `.runtime/phpt-progress/run-20260614T020448Z-manifest.log`:

| Selected | Runnable | Excluded | Passed | Failed | Skipped | Warned |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 70 | 21 | 49 | 14 | 7 | 0 | 0 |

## Runnable Outcomes

Passing rows:

```text
ext/standard/tests/array/array_fill_basic.phpt
ext/standard/tests/array/array_fill_variation4.phpt
ext/standard/tests/array/array_filter_variation5.phpt
ext/standard/tests/array/array_intersect_assoc_variation3.phpt
ext/standard/tests/array/array_intersect_assoc_variation4.phpt
ext/standard/tests/array/array_intersect_assoc_variation5.phpt
ext/standard/tests/array/array_intersect_assoc_variation6.phpt
ext/standard/tests/array/array_intersect_variation5.phpt
ext/standard/tests/array/array_intersect_variation6.phpt
ext/standard/tests/array/array_key_exists_variation8.phpt
ext/standard/tests/array/array_merge_variation4.phpt
ext/standard/tests/array/array_push_variation6.phpt
ext/standard/tests/array/array_shift_variation3.phpt
ext/standard/tests/array/array_slice_variation7.phpt
```

Failing runnable rows:

```text
ext/standard/tests/array/array_combine_variation3.phpt
ext/standard/tests/array/array_flip_variation3.phpt
ext/standard/tests/array/array_intersect_variation3.phpt
ext/standard/tests/array/array_intersect_variation4.phpt
ext/standard/tests/array/array_merge_recursive_variation3.phpt
ext/standard/tests/array/array_pad_variation6.phpt
ext/standard/tests/array/array_rand_variation6.phpt
```

## Remaining Blockers

The 49 still-excluded rows all classify as:

```text
unsupported-class-metadata    requires unsupported magic method dispatch/reflection metadata
```

These rows use heredoc/nowdoc values together with array-helper cases that also
depend on object string conversion or related magic metadata. They should be
reopened with a generic `__toString()`/magic method metadata implementation,
not by re-adding a heredoc syntax classifier.

The 7 runnable failures are runtime parity gaps in array helper behavior once
heredoc values can reach execution:

| Rows | Surface |
| ---: | --- |
| 1 | `array_combine()` key/value coercion with heredoc keys |
| 1 | `array_flip()` valid-value filtering and diagnostics |
| 2 | `array_intersect()` loose comparison over heredoc-containing arrays |
| 1 | `array_merge_recursive()` nested heredoc merge behavior |
| 1 | `array_pad()` with mixed array inputs |
| 1 | `array_rand()` key selection/output parity with heredoc string keys |

## Verification

```sh
cargo fmt --check
cargo test phpt_classifier
tools/run-phpt-baseline.sh --tier 1000 --classify-only --out-dir .runtime/ptn-4fd3-baseline
tools/run-bounded-phpt.sh tools/phpt-heredoc-nowdoc-frontier-manifest.txt
tools/run-phpt-baseline.sh --tier 1000 --classify-only --out-dir .runtime/ptn-4fd3-baseline-final
```
