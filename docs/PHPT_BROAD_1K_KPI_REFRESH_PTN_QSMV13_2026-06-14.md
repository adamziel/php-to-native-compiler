# PHPT Broad 1k KPI Refresh: 2026-06-14 ptn-qsmv.13

Issue: `ptn-qsmv.13`

This refresh reran the broad 1k PHPT baseline on current `origin/master` and
records the pass/fail KPI, dashboard delta, final residual rows, and next
red-to-green implementation packs. It is a measurement and ledger update, not
a runtime behavior change.

## Broad 1k Evidence

Command:

```sh
PHPT_PROGRESS_DIR=.runtime/ptn-qsmv13-final-phpt-progress \
  tools/run-phpt-baseline.sh --tier 1000 \
  --out-dir .runtime/ptn-qsmv13-final-baseline
```

Generated broad manifest:

```text
.runtime/ptn-qsmv13-final-baseline/20260614T154606Z/phpt-baseline-1000.txt
```

Artifacts:

```text
.runtime/ptn-qsmv13-final-phpt-progress/summary-20260614T154606Z.txt
.runtime/ptn-qsmv13-final-phpt-progress/classification-20260614T154606Z.tsv
.runtime/ptn-qsmv13-final-phpt-progress/runnable-20260614T154606Z.txt
.runtime/ptn-qsmv13-final-phpt-progress/excluded-20260614T154606Z.tsv
.runtime/ptn-qsmv13-final-phpt-progress/run-20260614T154606Z-zend.log
.runtime/ptn-qsmv13-final-phpt-progress/run-20260614T154606Z-standard.log
.runtime/ptn-qsmv13-final-phpt-progress/run-20260614T154606Z-core.log
```

State:

```text
PTN source commit: abfb48341ef24451b368a9a40f1712ccebba5991
php-src PHPT corpus: /home/claude/php-src-phpt
corpus revision: 8c63ec400ce8e07c57a8d9499317b96a8beafb8b
```

## KPI Result

| Selected | Runnable | Excluded | Tests | Passed | Failed | Skipped | Warned |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1,000 | 440 | 560 | 440 | 366 | 74 | 0 | 0 |

Previous dashboard row before this refresh:

| Dashboard | Ported | Passing | Gap |
| --- | ---: | ---: | ---: |
| Before | 1,000 | 285 | 715 |
| After | 1,000 | 366 | 634 |

Delta: `+81` broad 1k passing rows versus the previous dashboard entry.

Per-bucket run result:

| Bucket | Selected | Runnable | Passed | Failed | Skipped | Warned |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `zend` | 530 | 130 | 71 | 59 | 0 | 0 |
| `standard` | 384 | 294 | 287 | 7 | 0 | 0 |
| `core` | 86 | 16 | 8 | 8 | 0 | 0 |

The refresh chased current `origin/master` while other broad row packs landed.
The measurement sequence recorded during this bead was:

| PTN commit | Passed | Failed | Runnable | Notes |
| --- | ---: | ---: | ---: | --- |
| `98b346bf7d8c` | 319 | 105 | 424 | Initial current-branch run. |
| `554c347b9a2c` | 329 | 95 | 424 | After upstream broad fixes. |
| `2bce9e7db851` | 356 | 84 | 440 | After classifier/runtime-config expansion. |
| `abfb48341ef2` | 366 | 74 | 440 | Final current `origin/master` run. |

## Red-To-Green Evidence

Against the immediately prior broad run in this bead (`2bce9e7db851`,
356/440), the final standard bucket moved from 277/294 to 287/294. The rows
that were red in that prior run and green in the final run are:

```text
ext/standard/tests/array/array_merge.phpt
ext/standard/tests/array/array_merge_recursive_variation7.phpt
ext/standard/tests/array/array_next_error1.phpt
ext/standard/tests/array/array_next_error2.phpt
ext/standard/tests/array/array_push.phpt
ext/standard/tests/array/array_push_error2.phpt
ext/standard/tests/array/array_push_variation3.phpt
ext/standard/tests/array/array_replace.phpt
ext/standard/tests/array/array_shift_variation5.phpt
ext/standard/tests/array/array_shift_variation8.phpt
```

The final standard residual is now seven rows:

```text
ext/standard/tests/array/array_column_scalar_index_strict_types.phpt
ext/standard/tests/array/array_diff_assoc_variation9.phpt
ext/standard/tests/array/array_intersect_assoc_variation9.phpt
ext/standard/tests/array/array_intersect_variation9.phpt
ext/standard/tests/array/array_keys_variation_005.phpt
ext/standard/tests/array/array_map_variation2.phpt
ext/standard/tests/array/array_search_variation4.phpt
```

## Next Row Packs

These are concrete 20-50 row packs from the final 74 failed runnable rows. They
are grouped by shared implementation cause and are suitable as follow-up
red-to-green ledgers.

### Class/Object Property Metadata And Member Access, 26 Rows

Shared cause: asymmetric property visibility, non-public member metadata,
native method metadata, and object/member access semantics.

```text
Zend/tests/access_modifiers/access_modifiers_006.phpt
Zend/tests/asymmetric_visibility/ast_printing.phpt
Zend/tests/asymmetric_visibility/bug003.phpt
Zend/tests/asymmetric_visibility/bug004.phpt
Zend/tests/asymmetric_visibility/cpp_no_type.phpt
Zend/tests/asymmetric_visibility/cpp_private.phpt
Zend/tests/asymmetric_visibility/cpp_protected.phpt
Zend/tests/asymmetric_visibility/cpp_wider_set_scope.phpt
Zend/tests/asymmetric_visibility/decrease_scope_private_protected.phpt
Zend/tests/asymmetric_visibility/duplicate_modifier.phpt
Zend/tests/asymmetric_visibility/duplicate_modifier_2.phpt
Zend/tests/asymmetric_visibility/no_type.phpt
Zend/tests/asymmetric_visibility/object_reference.phpt
Zend/tests/asymmetric_visibility/override_protected_private.phpt
Zend/tests/asymmetric_visibility/reference.phpt
Zend/tests/asymmetric_visibility/reference_2.phpt
Zend/tests/asymmetric_visibility/unset.phpt
Zend/tests/asymmetric_visibility/unshared_rw_cache_slot.phpt
Zend/tests/asymmetric_visibility/virtual_get_only.phpt
Zend/tests/asymmetric_visibility/virtual_set_only.phpt
Zend/tests/attributes/nodiscard/005.phpt
Zend/tests/bug27669.phpt
Zend/tests/bug29015.phpt
Zend/tests/bug31525.phpt
Zend/tests/bug33999.phpt
Zend/tests/bug34064.phpt
```

### Array/Value Mutation And Nested Comparison, 20 Rows

Shared cause: nested array comparison/search coercion, array/object element
assignment, mutation diagnostics, callback reference handling, and array
reference edge cases.

```text
Zend/tests/array_append_reading_error.phpt
Zend/tests/array_literal_next_element_error.phpt
Zend/tests/array_merge_recursive_next_key_overflow.phpt
Zend/tests/array_splice_empty_ht_iter_removal.phpt
Zend/tests/assign_array_object_property.phpt
Zend/tests/assign_dim_obj_null_return.phpt
Zend/tests/assign_obj_op_cache_slot.phpt
Zend/tests/assign_op_type_error.phpt
Zend/tests/assign_to_obj_002.phpt
Zend/tests/binary.phpt
Zend/tests/bug31098.phpt
Zend/tests/bug31720.phpt
Zend/tests/bug34137.phpt
ext/standard/tests/array/array_column_scalar_index_strict_types.phpt
ext/standard/tests/array/array_diff_assoc_variation9.phpt
ext/standard/tests/array/array_intersect_assoc_variation9.phpt
ext/standard/tests/array/array_intersect_variation9.phpt
ext/standard/tests/array/array_keys_variation_005.phpt
ext/standard/tests/array/array_map_variation2.phpt
ext/standard/tests/array/array_search_variation4.phpt
```

### Diagnostics, Assertion, Control, And Runtime Boundaries, 28 Rows

Shared cause: assertion runtime state, AST diagnostic serialization, control
flow fatal diagnostics, runtime/header/INI environment behavior, and error
suppression or trace metadata.

```text
Zend/tests/assert/expect_002.phpt
Zend/tests/assert/expect_007.phpt
Zend/tests/assert/expect_009.phpt
Zend/tests/assert/expect_010.phpt
Zend/tests/assert/expect_017.phpt
Zend/tests/ast/ast_serialize_backtick_literal.phpt
Zend/tests/ast/ast_serialize_floats.phpt
Zend/tests/ast/gh21072.phpt
Zend/tests/break_error_001.phpt
Zend/tests/break_error_002.phpt
Zend/tests/break_error_003.phpt
Zend/tests/break_error_004.phpt
Zend/tests/bug20240.phpt
Zend/tests/bug29104.phpt
Zend/tests/bug33996.phpt
Zend/tests/bug34786.phpt
Zend/tests/bug36513.phpt
Zend/tests/bug37251.phpt
Zend/tests/bug39018.phpt
Zend/tests/bug39018_2.phpt
tests/basic/bug45986.phpt
tests/basic/bug54514.phpt
tests/basic/build_date.phpt
tests/basic/encoding.phpt
tests/basic/header_register_callback.phpt
tests/basic/header_register_callback_after_output.phpt
tests/basic/ini_parse_quantity_basic.phpt
tests/basic/ini_parse_quantity_warnings.phpt
```

## Verification

```sh
PHPT_PROGRESS_DIR=.runtime/ptn-qsmv13-final-phpt-progress \
  tools/run-phpt-baseline.sh --tier 1000 \
  --out-dir .runtime/ptn-qsmv13-final-baseline
```

Result:

```text
result: buckets=3 selected=1000 runnable=440 excluded=560 tests=440 passed=366 failed=74 skipped=0 warned=0 elapsed=3379s
```
