# PHPT Broad 1k Standard Array Blocker Map: 2026-06-14 g7c1

Issue: `ptn-g7c1`

This slice refreshed the broad 1k classifier on current `origin/master` and
rechecked the largest runnable family, `ext/standard/tests/array/*`. The
current frontier is still a blocker-map slice rather than an implementation
patch: the residual failures split across array-key conversion, callback
dispatch, ordered-array mutation, random key selection, and user-comparator set
operations. No one shared primitive is credible for moving 25 broad rows without
crossing several runtime boundaries.

## Broad 1k Evidence

Source state:

- PTN: `024297118b28`
- php-src PHPT corpus: `8c63ec400ce8e07c57a8d9499317b96a8beafb8b`

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-g7c1-baseline-before
```

Generated broad manifest:

```text
.runtime/ptn-g7c1-baseline-before/20260614T073448Z/phpt-baseline-1000.txt
```

Classifier artifacts:

```text
.runtime/phpt-progress/classification-20260614T073448Z.tsv
.runtime/phpt-progress/runnable-20260614T073448Z.txt
.runtime/phpt-progress/excluded-20260614T073448Z.tsv
```

Classifier result:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 1,000 | 424 | 576 |

Top current exclusions:

| Classification | Rows |
| --- | ---: |
| `unsupported-language` | 288 |
| `unsupported-class-metadata` | 143 |
| `unsupported-request-input-ini` | 28 |
| `unsupported-extension` | 20 |
| `unsupported-diagnostics-runtime` | 17 |
| `unsupported-assertion-ini` | 17 |
| `unsupported-resource-limit-ini` | 15 |
| `sapi-behavior` | 13 |
| `unsupported-assertion-runtime` | 9 |

Runnable rows by broad source:

| Source | Runnable rows |
| --- | ---: |
| `ext/standard/tests/array` | 294 |
| `Zend/tests` | 114 |
| `tests/basic` | 16 |

The committed 296-row standard-array frontier manifest is now slightly stale for
this exact broad baseline. Two rows that were previously runnable are classified:

| Row | Current classification |
| --- | --- |
| `ext/standard/tests/array/array_combine.phpt` | `unsupported-language`: variable variables and runtime symbol-table lookup |
| `ext/standard/tests/array/array_filter_invalid_mode.phpt` | `unsupported-language`: named-argument binding for modeled array internals |

The current standard-array runnable subset was generated for this slice with:

```sh
grep '^ext/standard/tests/array/' \
  .runtime/phpt-progress/runnable-20260614T073448Z.txt \
  > .runtime/ptn-g7c1-standard-array-runnable-current.txt
```

## Focused Execution Evidence

The 294-row focused standard-array run was started with:

```sh
tools/run-bounded-phpt.sh .runtime/ptn-g7c1-standard-array-runnable-current.txt
```

Artifact:

```text
.runtime/phpt-progress/run-20260614T074045Z-manifest.log
```

That run was stopped after row 63 because it was spending most time re-running
known-green rows. The partial result is still useful as a sanity check:

- `array_change_key_case()` rows, including binary-safe string-key variation 8,
  passed.
- `array_chunk()` rows passed through variation 32 and the later variation
  rows reached before the stop.
- The first current failures matched the residual blocker map:
  `array_column_scalar_index_strict_types.phpt`,
  `array_combine_variation3.phpt`, and `array_count_values2.phpt`.

## Current Residual Blocker Map

After removing the two now-classified rows above from the latest committed
standard-array frontier, and after confirming
`array_change_key_case_variation8.phpt` now passes in the partial focused run,
the current residual standard-array blocker map has 50 runnable rows:

| Bucket | Rows | Generic blocker |
| --- | ---: | --- |
| Key/value conversion and loose comparison | 15 | Shared array-key coercion, strict scalar argument coercion, nested array/object/resource comparison, and warning parity across value-search helpers. |
| Callback, mode, and `array_map()`/`array_reduce()` edge semantics | 9 | Catchable callback arity diagnostics, object callback failures, built-in callback dispatch, uneven `array_map()` zipping, reference propagation, and binary-safe callback arguments. |
| Ordered-array mutation, recursion, references, and temporary lvalues | 12 | Recursive merge/replace, push/shift mutation visibility, max-next-key overflow, by-reference temporary diagnostics for pointer helpers, and reference-preserving mutation paths. |
| `array_rand()` helper semantics | 7 | Random key selection, requested-count validation, associative key preservation, multidimensional inputs, and heredoc-string keys. |
| User-comparator set-operation ordering/diagnostics | 7 | Comparator result handling, incorrect callback diagnostics, duplicate matching/order behavior, and multi-array `array_u*`/`array_udiff*` parity. |

Key/value conversion and loose comparison:

```text
ext/standard/tests/array/array_column_scalar_index_strict_types.phpt
ext/standard/tests/array/array_combine_variation3.phpt
ext/standard/tests/array/array_count_values2.phpt
ext/standard/tests/array/array_diff_variation9.phpt
ext/standard/tests/array/array_flip.phpt
ext/standard/tests/array/array_flip_variation3.phpt
ext/standard/tests/array/array_intersect_assoc_variation9.phpt
ext/standard/tests/array/array_intersect_variation3.phpt
ext/standard/tests/array/array_intersect_variation4.phpt
ext/standard/tests/array/array_intersect_variation9.phpt
ext/standard/tests/array/array_key_exists_variation3.phpt
ext/standard/tests/array/array_keys_variation_005.phpt
ext/standard/tests/array/array_pad_variation6.phpt
ext/standard/tests/array/array_search_variation3.phpt
ext/standard/tests/array/array_search_variation4.phpt
```

Callback, mode, and map/reduce edge semantics:

```text
ext/standard/tests/array/array_filter_variation10.phpt
ext/standard/tests/array/array_map_error.phpt
ext/standard/tests/array/array_map_object2.phpt
ext/standard/tests/array/array_map_variation10.phpt
ext/standard/tests/array/array_map_variation12.phpt
ext/standard/tests/array/array_map_variation2.phpt
ext/standard/tests/array/array_map_variation7.phpt
ext/standard/tests/array/array_map_variation9.phpt
ext/standard/tests/array/array_reduce_variation1.phpt
```

Ordered-array mutation, recursion, references, and temporary lvalues:

```text
ext/standard/tests/array/array_merge.phpt
ext/standard/tests/array/array_merge_recursive_variation3.phpt
ext/standard/tests/array/array_merge_recursive_variation7.phpt
ext/standard/tests/array/array_next_error1.phpt
ext/standard/tests/array/array_next_error2.phpt
ext/standard/tests/array/array_push.phpt
ext/standard/tests/array/array_push_error2.phpt
ext/standard/tests/array/array_push_variation3.phpt
ext/standard/tests/array/array_replace.phpt
ext/standard/tests/array/array_replace_merge_recursive_ref.phpt
ext/standard/tests/array/array_shift_variation5.phpt
ext/standard/tests/array/array_shift_variation8.phpt
```

`array_rand()` helper semantics:

```text
ext/standard/tests/array/array_rand.phpt
ext/standard/tests/array/array_rand_basic1.phpt
ext/standard/tests/array/array_rand_basic2.phpt
ext/standard/tests/array/array_rand_variation3.phpt
ext/standard/tests/array/array_rand_variation4.phpt
ext/standard/tests/array/array_rand_variation5.phpt
ext/standard/tests/array/array_rand_variation6.phpt
```

User-comparator set-operation ordering/diagnostics:

```text
ext/standard/tests/array/array_udiff_assoc_variation.phpt
ext/standard/tests/array/array_udiff_assoc_variation5.phpt
ext/standard/tests/array/array_udiff_uassoc_variation6.phpt
ext/standard/tests/array/array_udiff_variation5.phpt
ext/standard/tests/array/array_uintersect_assoc_basic2.phpt
ext/standard/tests/array/array_uintersect_assoc_variation5.phpt
ext/standard/tests/array/array_uintersect_uassoc_variation6.phpt
```

## Implementation Boundary

The largest coherent residual bucket is 15 rows, below the 25-row target, and
it mixes strict scalar coercion, key conversion, loose comparison, and warning
text. The 9-row callback family and 12-row mutation family are also distinct
runtime surfaces. A future implementation should split first on one shared
primitive, then re-run the current standard-array runnable subset and broad
1k classify-only tier:

1. Centralize array-key/value comparison and warning parity for `array_keys()`,
   `array_search()`, `array_flip()`, `array_count_values()`, and
   `array_diff*()`/`array_intersect*()`.
2. Extend callback dispatch with catchable arity diagnostics, object callback
   failures, and uneven `array_map()` zipping.
3. Move ordered-array mutation/COW parity through recursive merge/replace,
   pointer-helper by-reference diagnostics, and max-next-key overflow paths.

## Verification

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-g7c1-baseline-before
tools/run-bounded-phpt.sh .runtime/ptn-g7c1-standard-array-runnable-current.txt
```

The second command was intentionally stopped after enough current focused
evidence was collected to avoid re-running the entire known-green array helper
frontier.
