# PHPT Broad 1k Standard Array Current Frontier: 2026-06-14

Issue: `ptn-bkcv`

This slice refreshes the broad 1k PHPT standard-array frontier on current
`origin/master` and records a blocker map rather than a behavior change. The
remaining failures are real array/runtime semantics, but no single narrow
implementation primitive is credible for moving 25 broad rows without crossing
several runtime boundaries.

## Broad 1k Evidence

Source state:

- PTN: `37dd64d2261b`
- php-src PHPT corpus: `8c63ec400ce8e07c57a8d9499317b96a8beafb8b`

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only
```

Generated broad manifest:

```text
.runtime/phpt-baseline/20260614T034158Z/phpt-baseline-1000.txt
```

Classifier artifacts:

```text
.runtime/phpt-progress/classification-20260614T034158Z.tsv
.runtime/phpt-progress/runnable-20260614T034158Z.txt
.runtime/phpt-progress/excluded-20260614T034158Z.tsv
```

Classifier result:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 1,000 | 430 | 570 |

Top current exclusions:

| Classification | Rows |
| --- | ---: |
| `unsupported-language` | 281 |
| `unsupported-class-metadata` | 144 |
| `unsupported-request-input-ini` | 28 |
| `unsupported-extension` | 20 |
| `unsupported-diagnostics-runtime` | 17 |
| `unsupported-assertion-ini` | 17 |
| `unsupported-resource-limit-ini` | 15 |
| `sapi-behavior` | 13 |
| `unsupported-assertion-runtime` | 9 |

## Focused Standard-Array Evidence

Focused manifest:

```text
tools/phpt-broad-standard-array-frontier-manifest.txt
```

It was derived from the broad 1k runnable set:

```sh
rg '^ext/standard/tests/array/' \
  .runtime/phpt-progress/runnable-20260614T034158Z.txt \
  > tools/phpt-broad-standard-array-frontier-manifest.txt
```

Focused run:

```sh
tools/run-bounded-phpt.sh tools/phpt-broad-standard-array-frontier-manifest.txt
```

Artifacts:

```text
.runtime/phpt-progress/run-20260614T034847Z-manifest.log
.runtime/phpt-progress/classification-20260614T034847Z.tsv
.runtime/phpt-progress/runnable-20260614T034847Z.txt
```

Result:

| Selected | Runnable | Passed | Failed |
| ---: | ---: | ---: | ---: |
| 296 | 296 | 243 | 53 |

Current passing signal that was previously noisy:

- `array_chunk()` broad family: selected rows passed.
- `array_sum()`/`array_product()` selected rows passed, including overflow and
  nested-array warning integration.
- Predicate/find helpers (`array_all()`, `array_any()`, `array_find()`,
  `array_find_key()`, `array_first()`, `array_last()`) passed in this focused
  set.
- Many callback validation rows that previously failed now pass, including
  missing callback names and object-method callbacks in set-operation helpers.

## Residual Failure Buckets

| Bucket | Rows | Generic blocker |
| --- | ---: | --- |
| Key/value conversion and loose comparison | 17 | Shared array-key coercion, strict scalar argument coercion, binary string key handling, nested array/object/resource comparison, and warning parity across value-search helpers. |
| Callback, mode, and `array_map()`/`array_reduce()` edge semantics | 10 | Catchable callback arity diagnostics, invalid mode constants, object callback failures, built-in callback dispatch, uneven `array_map()` zipping, reference propagation, and binary-safe callback arguments. |
| Ordered-array mutation, recursion, references, and temporary lvalues | 12 | Recursive merge/replace, push/shift mutation visibility, max-next-key overflow, by-reference temporary diagnostics for pointer helpers, and reference-preserving mutation paths. |
| `array_rand()` helper semantics | 7 | Random key selection, requested-count validation, associative key preservation, multidimensional inputs, and heredoc-string keys. |
| User-comparator set-operation ordering/diagnostics | 7 | Comparator result handling, incorrect callback diagnostics, duplicate matching/order behavior, and multi-array `array_u*`/`array_udiff*` parity. |

These buckets are a blocker map. Reopening them as implementation work should
be split by shared runtime primitive, not by individual PHPT rows.

## Failed Rows By Bucket

Key/value conversion and loose comparison:

```text
ext/standard/tests/array/array_change_key_case_variation8.phpt
ext/standard/tests/array/array_column_scalar_index_strict_types.phpt
ext/standard/tests/array/array_combine.phpt
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
ext/standard/tests/array/array_filter_invalid_mode.phpt
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

`array_rand()`:

```text
ext/standard/tests/array/array_rand.phpt
ext/standard/tests/array/array_rand_basic1.phpt
ext/standard/tests/array/array_rand_basic2.phpt
ext/standard/tests/array/array_rand_variation3.phpt
ext/standard/tests/array/array_rand_variation4.phpt
ext/standard/tests/array/array_rand_variation5.phpt
ext/standard/tests/array/array_rand_variation6.phpt
```

User-comparator set operations:

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

1. Key/value conversion helper: centralize array-key coercion and loose value
   comparison for `array_keys()`, `array_search()`, `array_flip()`,
   `array_count_values()`, `array_diff*()`, and `array_intersect*()`, including
   nested-array warning parity.
2. Callback and map/reduce semantics: keep callback validation shared, then add
   mode constant handling, catchable arity diagnostics, uneven `array_map()`
   zipping, and reference behavior.
3. Ordered-array mutation/COW: handle recursive merge/replace references,
   temporary-by-reference diagnostics, max-next-key overflow, and recursive
   shift/push visibility through the ordered-array runtime.
4. `array_rand()` can be a small follow-up helper slice, but it is only seven
   broad rows and does not satisfy the 25-row target alone.

## Verification

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only
tools/run-bounded-phpt.sh tools/phpt-broad-standard-array-frontier-manifest.txt
tools/run-bounded-phpt.sh --classify-only tools/phpt-broad-standard-array-frontier-manifest.txt
```

After rebasing over the later array-frontier blocker maps and named-argument
classification, the committed focused manifest classifies as 296 selected, 295
runnable, and 1 excluded:

```text
ext/standard/tests/array/array_filter_invalid_mode.phpt
```

The raw focused execution evidence remains the
`run-20260614T034847Z-manifest.log` result recorded above.
