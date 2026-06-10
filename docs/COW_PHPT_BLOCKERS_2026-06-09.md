# COW PHPT Blockers: 2026-06-09

Evidence base:

- Refreshed on `polecat/prime/ptn-kia@mq7unevm` at 2026-06-10T09:34Z.
- `tools/phpt-cow-manifest.txt` still has 29 rows in seven buckets.
- Focused COW manifest is 24/29. Nested rerun is 3/4; `bug38469` passes in the
  focused bucket, and only `bug35163` remains failing there.

## Focused COW Counts

| Bucket | Total | Pass | Fail |
| --- | ---: | ---: | ---: |
| assignment-aliasing | 4 | 4 | 0 |
| string-offsets | 4 | 4 | 0 |
| array-writes-appends-unset | 4 | 4 | 0 |
| nested-arrays | 4 | 3 | 1 |
| foreach-mutation | 4 | 3 | 1 |
| function-boundaries | 4 | 1 | 3 |
| reference-interaction | 5 | 5 | 0 |
| **Total** | **29** | **24** | **5** |

## Linked Generic Blocker Beads

| Generic blocker | Rows held | Bead |
| --- | ---: | --- |
| Nested recursive reference lvalues | 1 | `ptn-4yt.2` |
| Closure callback mutation through `array_walk()` and `$GLOBALS` | 1 | `ptn-4yt.7`, `ptn-4yt.3` |
| Call-result references and callback return references | 3 | `ptn-4yt.8`, `ptn-4yt.3` |

## Fixed Rows

| Bucket | PHPT row | Generic fix | Compact reducer |
| --- | --- | --- | --- |
| assignment-aliasing | `Zend/tests/assign_to_var_003.phpt` | Non-array offset reads emit shared scalar offset diagnostics; assignment through reference aliases keeps the shared cell visible. | `scalar_offset_assignment_aliasing` |
| array-writes-appends-unset | `Zend/tests/assign_dim_op_same_var.phpt` | Array-dim compound assignment snapshots overlapping RHS values before writeback. | `compile_array_path_self_assignment_snapshots_rhs_to_native_binary` |
| array-writes-appends-unset | `ext/standard/tests/array/array_unshift_basic1.phpt` | `array_unshift()` mutates direct variable arrays with COW detach and PHP key handling. | `array_unshift_shared_alias` |
| nested-arrays | `ext/standard/tests/array/array_merge_recursive_basic1.phpt` | `array_merge_recursive()` is registered and preserves array merge COW boundaries for the basic default-key row. | `compile_array_merge_recursive_to_native_binary` |
| nested-arrays | `ext/standard/tests/array/array_merge_replace_recursive_refs.phpt` | `array_replace_recursive()` is registered and recursively replaces ordered-map keys while cloning dereferenced values across COW boundaries. | `compile_array_replace_recursive_to_native_binary`, `compile_nested_array_cow_reducers_match_php_oracle` |
| foreach-mutation | `Zend/tests/foreach/foreach_reference.phpt` | `array_values()` and `array_reverse()` unwrap single-owner references while preserving shared references. | `array_reindexing_internals_unwrap_single_owner_refs` |
| foreach-mutation | `Zend/tests/foreach/foreach_by_ref_repacking_insert.phpt` | Whitespace-only source prelude before the first PHP open tag is skipped. | `parser_accepts_whitespace_prelude_and_reference_array_entries` |
| foreach-mutation | `Zend/tests/foreach/foreach_temp_array_expr_with_refs.phpt` | Array literal entries can store references, and temporary reference arrays feed by-reference `foreach`. | `compile_foreach_temporary_reference_array_literal_to_native_binary` |
| string-offsets | `Zend/tests/str_offset_002.phpt` | Array literal reference elements use runtime reference helpers and string-offset `Error` diagnostics. | `compile_string_offset_reference_array_literal_raises_error_to_native_binary` |
| string-offsets | `Zend/tests/string_offset_optimization.phpt` | Function-scoped string-offset reference literals use the same diagnostic path. | `compile_string_offset_reference_array_literal_raises_error_to_native_binary` |
| function-boundaries | `Zend/tests/return_types/return_reference_separation.phpt` | Scalar type hints and typed by-reference return separation lower through generic return slots. | `compile_typed_by_ref_return_separates_function_boundaries_to_native_binary` |
| reference-interaction | `Zend/tests/array_with_refs_identical.phpt` | Strict array comparison dereferences reference entries like PHP. | `parser_accepts_reference_array_literal_values` |
| reference-interaction | `ext/standard/tests/array/array_sum_on_reference.phpt` | `array_sum()` unwraps reference entries through numeric conversion. | `compile_reference_array_literals_and_internals_to_native_binary` |
| reference-interaction | `ext/standard/tests/strings/strtr_with_reference.phpt` | Two-argument `strtr()` reads replacement maps through normal value conversion. | `compile_reference_array_literals_and_internals_to_native_binary` |
| reference-interaction | `ext/standard/tests/general_functions/debug_zval_dump_refs.phpt` | `debug_zval_dump()` prints reference/refcount structure for arrays and references. | `compile_reference_array_literals_and_internals_to_native_binary` |
| reference-interaction | `Zend/tests/assign_dim_ref_free.phpt` | Append assignment expressions and by-reference list assignment expressions lower through generic reference-array assignment targets. | `compile_append_expression_with_reference_list_assignment_to_native_binary` |

## Remaining Blocker Rows

| Bucket | PHPT row | Current generic blocker | Bead |
| --- | --- | --- | --- |
| nested-arrays | `Zend/tests/bug35163.phpt` | Nested recursive reference lvalues remain unsupported. | `ptn-4yt.2` |
| foreach-mutation | `ext/standard/tests/array/array_walk/bug69068_2.phpt` | Closure with by-reference callback and global swap is unsupported. | `ptn-4yt.7`, `ptn-4yt.3` |
| function-boundaries | `Zend/tests/assign_by_val_function_by_ref_return_value.phpt` | Assignment by reference from a function result is rejected. | `ptn-4yt.8`, `ptn-4yt.3` |
| function-boundaries | `ext/standard/tests/array/array_reduce_accumulator_refcount.phpt` | `array_reduce()` callback/refcount behavior is unsupported. | `ptn-4yt.8`, `ptn-4yt.3` |
| function-boundaries | `ext/standard/tests/array/array_reduce_return_by_ref.phpt` | `array_reduce()` callback returning by reference is unsupported. | `ptn-4yt.8`, `ptn-4yt.3` |
