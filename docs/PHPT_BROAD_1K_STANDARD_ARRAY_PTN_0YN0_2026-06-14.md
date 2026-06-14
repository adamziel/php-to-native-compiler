# PHPT Broad 1k Standard Array Slice: 2026-06-14 ptn-0yn0

Issue: `ptn-0yn0`

This slice refreshed the broad 1k PHPT baseline on current `origin/master` and
then focused the runnable `ext/standard/tests/array/` cluster. It records a
blocker map rather than an implementation change: the remaining failures still
cross several generic runtime boundaries, so a row-shaped patch would be the
wrong fix.

## Broad 1k Evidence

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only
```

Source state:

- PTN commit: `5de799cfcd6b`
- php-src PHPT corpus: `/home/claude/php-src-phpt`
- corpus revision: `8c63ec400ce8e07c57a8d9499317b96a8beafb8b`

Artifacts:

```text
.runtime/phpt-baseline/20260614T061041Z/phpt-baseline-1000.txt
.runtime/phpt-progress/classification-20260614T061042Z.tsv
.runtime/phpt-progress/runnable-20260614T061042Z.txt
.runtime/phpt-progress/excluded-20260614T061042Z.tsv
.runtime/phpt-progress/summary-20260614T061042Z.txt
```

Result:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 1000 | 424 | 576 |

Top exclusion buckets:

| Bucket | Rows |
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

## Focused Standard Array Evidence

The focused manifest was selected from the broad 1k runnable rows:

```sh
rg '^ext/standard/tests/array/' \
  .runtime/phpt-progress/runnable-20260614T061042Z.txt \
  > .runtime/ptn-0yn0-standard-array-runnable.txt
```

Compared with the older committed standard-array manifest, the current broad
classifier no longer leaves these rows runnable:

```text
ext/standard/tests/array/array_combine.phpt
ext/standard/tests/array/array_filter_invalid_mode.phpt
```

Focused run:

```sh
tools/run-bounded-phpt.sh .runtime/ptn-0yn0-standard-array-runnable.txt
```

Artifacts:

```text
.runtime/phpt-progress/classification-20260614T061700Z.tsv
.runtime/phpt-progress/runnable-20260614T061700Z.txt
.runtime/phpt-progress/run-20260614T061700Z-manifest.log
.runtime/phpt-progress/summary-20260614T061700Z.txt
```

Result:

| Selected | Runnable | Passed | Failed | Warned | Skipped |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 294 | 294 | 243 | 51 | 0 | 0 |

Known-good broad families in this run include all selected `array_chunk()`,
`array_fill()`/`array_fill_keys()`, predicate/find helpers, `array_key_first()`,
`array_key_last()`, `array_slice()`, `array_splice()`, `array_sum()`, and
`array_product()` rows.

## Residual Failure Buckets

| Bucket | Rows | Generic blocker |
| --- | ---: | --- |
| Key/value conversion, nested comparison, and scalar diagnostics | 14 | Shared array-key coercion, binary/heredoc string key rendering, skipped-value warning formatting, nested array/object comparison, float-key deprecation diagnostics, and strict scalar argument coercion still need central runtime paths. |
| Directory-resource comparison and lookup | 2 | `opendir()`/directory resources are not modeled through the resource table used by key/search helpers. |
| Callback and `array_map()`/`array_reduce()` edge semantics | 9 | Catchable callback arity diagnostics, object/static callable failure messages, built-in callback diagnostics, uneven `array_map()` zipping, reference propagation, and binary-safe callback arguments remain split across helper paths. |
| Ordered-array mutation, recursion, references, and temporary lvalues | 12 | Recursive merge/replace, push/shift mutation visibility, max-next-key overflow, by-reference temporary diagnostics for pointer helpers, and reference-preserving mutation paths need shared ordered-array/runtime-lvalue work. |
| `array_rand()` helper semantics | 7 | Random key selection, requested-count validation, associative key preservation, multidimensional inputs, and heredoc-string keys remain an isolated helper gap. |
| User-comparator set-operation ordering and diagnostics | 7 | Comparator result handling, incorrect callback diagnostics, duplicate matching/order behavior, and multi-array `array_u*`/`array_udiff*` parity still diverge. |

## Failed Rows By Bucket

Key/value conversion, nested comparison, and scalar diagnostics:

```text
ext/standard/tests/array/array_change_key_case_variation8.phpt
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
ext/standard/tests/array/array_pad_variation6.phpt
ext/standard/tests/array/array_search_variation3.phpt
```

Directory-resource comparison and lookup:

```text
ext/standard/tests/array/array_keys_variation_005.phpt
ext/standard/tests/array/array_search_variation4.phpt
```

Callback and `array_map()`/`array_reduce()` edge semantics:

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

User-comparator set-operation ordering and diagnostics:

```text
ext/standard/tests/array/array_udiff_assoc_variation.phpt
ext/standard/tests/array/array_udiff_assoc_variation5.phpt
ext/standard/tests/array/array_udiff_uassoc_variation6.phpt
ext/standard/tests/array/array_udiff_variation5.phpt
ext/standard/tests/array/array_uintersect_assoc_basic2.phpt
ext/standard/tests/array/array_uintersect_assoc_variation5.phpt
ext/standard/tests/array/array_uintersect_uassoc_variation6.phpt
```

## Next Implementation Splits

The most credible implementation splits are:

1. Centralize array-key/value conversion, nested array/object comparison, and
   scalar conversion diagnostics for key/value helpers and set operations.
2. Add directory-resource internals through the existing resource table, then
   re-run the key/search resource rows with the filesystem/path manifest.
3. Make callback arity and callable diagnostics catchable and consistent across
   internal callback helpers before reopening the `array_map()`/`array_reduce()`
   edge rows.
4. Extend ordered-array mutation and lvalue/reference helpers for recursive
   merge/replace, push/shift, max-next-key overflow, and temporary pointer
   diagnostics.
5. Implement `array_rand()` as a separate helper slice; it is only seven broad
   rows and should not be stretched into a 25-row implementation claim.

## Verification

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only
tools/run-bounded-phpt.sh .runtime/ptn-0yn0-standard-array-runnable.txt
```
