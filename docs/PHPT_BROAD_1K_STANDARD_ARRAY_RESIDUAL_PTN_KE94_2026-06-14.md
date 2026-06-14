# PHPT Broad 1k Standard Array Residual Frontier: 2026-06-14

Issue: `ptn-ke94`

This slice refreshes the broad 1k standard-array runnable frontier after the
latest array, binary-string, recursive-dump, and operator/control changes. It
is a blocker map, not a behavior change. The current residual failures are
still spread across several generic runtime surfaces, so a narrow patch would
not credibly move 25 broad rows without mixing unrelated semantics.

## Broad 1k Evidence

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-ke94-baseline
```

Generated manifest:

```text
.runtime/ptn-ke94-baseline/20260614T062526Z/phpt-baseline-1000.txt
```

Classifier artifacts:

```text
.runtime/phpt-progress/classification-20260614T062526Z.tsv
.runtime/phpt-progress/runnable-20260614T062526Z.txt
.runtime/phpt-progress/excluded-20260614T062526Z.tsv
```

Evidence command reported PTN commit: `a8856615c283`

Corpus revision: `8c63ec400ce8e07c57a8d9499317b96a8beafb8b`

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 1,000 | 424 | 576 |

Current runnable rows by source:

| Source | Runnable |
| --- | ---: |
| `ext/standard/tests/array` | 294 |
| `Zend/tests` root | 81 |
| `Zend/tests/asymmetric_visibility` | 22 |
| `tests/basic` | 16 |
| `Zend/tests/ast` | 4 |
| `Zend/tests/arrow_functions` | 3 |
| `Zend/tests/assert` | 2 |
| `Zend/tests/access_modifiers` | 1 |
| `Zend/tests/attributes/nodiscard` | 1 |

## Focused Standard-Array Evidence

Committed manifest:

```text
tools/phpt-standard-array-current-ptn-ke94-manifest.txt
```

Selection from `classification-20260614T062526Z.tsv`:

```sh
awk -F'\t' '$2=="runnable" && $1 ~ /^ext\/standard\/tests\/array\// {print $1}'
```

Focused run:

```sh
tools/run-bounded-phpt.sh .runtime/ptn-ke94-standard-array-runnable.txt
```

Artifacts:

```text
.runtime/phpt-progress/summary-20260614T063057Z.txt
.runtime/phpt-progress/run-20260614T063057Z-manifest.log
```

| Selected | Runnable | Passed | Failed | Skipped | Warned |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 294 | 294 | 244 | 50 | 0 | 0 |

Compared with the older 296-row standard-array manifest, two rows are now
classified before execution in the broad 1k baseline:

```text
ext/standard/tests/array/array_combine.phpt
ext/standard/tests/array/array_filter_invalid_mode.phpt
```

The current broad standard-array run keeps recent movement green, including
`array_change_key_case_variation8.phpt`, `array_chunk*`, fill/pad allocation
guards except the resource-limit classified row, and array sum/product rows.

## Residual Failure Buckets

| Bucket | Rows | Generic blocker |
| --- | ---: | --- |
| Key/value conversion and loose comparison | 15 | Shared array-key coercion, strict scalar argument coercion, nested array/object/resource comparison, and warning parity across key/value helpers. |
| Callback, map, filter, and reduce semantics | 9 | Catchable callback arity diagnostics, object callback failures, built-in callback dispatch, uneven `array_map()` zipping, reference propagation, binary-safe callback arguments, and reduce accumulator variation. |
| Ordered-array mutation, recursion, references, and temporary lvalues | 12 | Recursive merge/replace, push/shift mutation visibility, max-next-key overflow, by-reference temporary diagnostics for pointer helpers, and reference-preserving mutation paths. |
| `array_rand()` helper semantics | 7 | Random key selection, requested-count validation, associative key preservation, multidimensional inputs, and heredoc-string keys. |
| User-comparator set-operation ordering/diagnostics | 7 | Comparator result handling, incorrect callback diagnostics, duplicate matching/order behavior, and multi-array `array_u*`/`array_udiff*` parity. |

### Key/value conversion and loose comparison

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

### Callback, map, filter, and reduce semantics

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

### Ordered-array mutation, recursion, references, and temporary lvalues

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

### `array_rand()`

```text
ext/standard/tests/array/array_rand.phpt
ext/standard/tests/array/array_rand_basic1.phpt
ext/standard/tests/array/array_rand_basic2.phpt
ext/standard/tests/array/array_rand_variation3.phpt
ext/standard/tests/array/array_rand_variation4.phpt
ext/standard/tests/array/array_rand_variation5.phpt
ext/standard/tests/array/array_rand_variation6.phpt
```

### User-comparator set operations

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

1. Centralize array-key/value conversion and nested-value warning parity across
   `array_keys()`, `array_search()`, `array_flip()`, `array_count_values()`,
   `array_diff*()`, and `array_intersect*()`.
2. Split `array_map()`/`array_filter()`/`array_reduce()` callback semantics
   from set-operation comparators. Both use callable dispatch, but their
   arity, key/value argument shape, result construction, and reference
   behavior differ.
3. Keep recursive mutation/COW work separate from value comparison. The merge,
   replace, push, shift, and pointer-helper failures are lvalue/reference
   semantics, not helper-specific output gaps.
4. Treat `array_rand()` as a bounded helper follow-up. It is coherent but only
   seven broad 1k rows, so it does not satisfy the broad-slice target alone.

## Verification

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-ke94-baseline
tools/run-bounded-phpt.sh .runtime/ptn-ke94-standard-array-runnable.txt
cargo fmt --check
```
