# COW PHPT Blockers: 2026-06-09

Evidence base:

- Bounded manifest: `.runtime/phpt-progress/summary-20260609T182419Z.txt`,
  200 total, 145 pass, 55 fail.
- Focused COW manifest: `tools/phpt-cow-manifest.txt`, row-level rerun on
  `decc46bd2`, 29 total, 7 pass, 22 fail.
- Bucket-level COW runner produced numeric counts for the first three buckets,
  then `Zend/tests/bug38469.phpt` exhausted `run-tests.php` diff memory. That
  row is counted as a focused COW failure because direct native output recurses
  through an array cycle instead of separating the copied array.

## Bounded Counts

| Source | Total | Pass | Fail |
| --- | ---: | ---: | ---: |
| PHPT bounded manifest | 200 | 145 | 55 |
| Zend | 76 | 63 | 13 |
| ext/standard | 77 | 47 | 30 |
| tests/basic+func+lang | 45 | 33 | 12 |
| other | 2 | 2 | 0 |

## Focused COW Counts

| Bucket | Total | Pass | Fail |
| --- | ---: | ---: | ---: |
| assignment-aliasing | 4 | 3 | 1 |
| string-offsets | 4 | 2 | 2 |
| array-writes-appends-unset | 4 | 2 | 2 |
| nested-arrays | 4 | 0 | 4 |
| foreach-mutation | 4 | 0 | 4 |
| function-boundaries | 4 | 0 | 4 |
| reference-interaction | 5 | 0 | 5 |

## Blocker Rows

| Bucket | PHPT row | Current blocker | Map |
| --- | --- | --- | --- |
| assignment-aliasing | `Zend/tests/assign_to_var_003.phpt` | Float offset warning text differs while alias value result is correct. | unsupported: PHP-exact offset diagnostic wording |
| string-offsets | `Zend/tests/str_offset_002.phpt` | Reference to string offset parses as expression error instead of PHP `Error`. | unsupported: references to/from string offsets |
| string-offsets | `Zend/tests/string_offset_optimization.phpt` | Same string-offset reference form inside a function. | unsupported: references to/from string offsets |
| array-writes-appends-unset | `Zend/tests/assign_dim_op_same_var.phpt` | Compound dimension assignment with the same variable mutates to `int(1)`. | `ptn-cqu.47.19` dynamic read-slot COW |
| array-writes-appends-unset | `ext/standard/tests/array/array_unshift_basic1.phpt` | `array_unshift()` is not registered. | unsupported: mutating internal not implemented |
| nested-arrays | `Zend/tests/bug35163.phpt` | Nested reference lvalue is rejected explicitly. | unsupported: nested reference lvalues |
| nested-arrays | `Zend/tests/bug38469.phpt` | Self-assignment creates a recursive value; `var_dump()` recurses until failure. | `ptn-cqu.47.18` nested refcount/cycle stress |
| nested-arrays | `ext/standard/tests/array/array_merge_recursive_basic1.phpt` | `array_merge_recursive()` is not registered. | unsupported: recursive array internal not implemented |
| nested-arrays | `ext/standard/tests/array/array_merge_replace_recursive_refs.phpt` | Reference array literal blocks before recursive merge/replace semantics. | unsupported: reference array literal / recursive internals |
| foreach-mutation | `Zend/tests/foreach/foreach_reference.phpt` | Parser emits explicit by-reference foreach unsupported diagnostic. | `ptn-cqu.47.17` by-reference foreach |
| foreach-mutation | `Zend/tests/foreach/foreach_by_ref_repacking_insert.phpt` | PHPT file starts with inline whitespace before `<?php`; parser rejects mixed open-tag shape. | unsupported: mixed/inline PHP open-tag handling |
| foreach-mutation | `Zend/tests/foreach/foreach_temp_array_expr_with_refs.phpt` | Temporary array of references plus by-reference foreach is rejected. | `ptn-cqu.47.17` by-reference foreach |
| foreach-mutation | `ext/standard/tests/array/array_walk/bug69068_2.phpt` | Closure with by-reference callback and global swap is unsupported. | unsupported: closures/use/globals callback mutation |
| function-boundaries | `Zend/tests/return_types/return_reference_separation.phpt` | Return type hints and by-reference returns block parsing. | unsupported: typed by-reference returns |
| function-boundaries | `Zend/tests/assign_by_val_function_by_ref_return_value.phpt` | Assignment by reference from a function result is rejected. | unsupported: by-reference assignment from call result |
| function-boundaries | `ext/standard/tests/array/array_reduce_accumulator_refcount.phpt` | Closure callback plus `debug_zval_dump()` refcount surface is unsupported. | unsupported: closures and refcount inspection internal |
| function-boundaries | `ext/standard/tests/array/array_reduce_return_by_ref.phpt` | `array_reduce()` callback returning by reference is unsupported. | unsupported: by-reference callback returns |
| reference-interaction | `Zend/tests/array_with_refs_identical.phpt` | Array literal containing references blocks strict identity comparison. | unsupported: reference array literal identity |
| reference-interaction | `Zend/tests/assign_dim_ref_free.phpt` | Assigning a reference into next dimension is rejected. | unsupported: reference append lvalue |
| reference-interaction | `ext/standard/tests/array/array_sum_on_reference.phpt` | Reference array operand blocks `array_sum()` path. | unsupported: reference-aware numeric array internal |
| reference-interaction | `ext/standard/tests/strings/strtr_with_reference.phpt` | Reference operand blocks `strtr()` path. | unsupported: reference-aware string internal |
| reference-interaction | `ext/standard/tests/general_functions/debug_zval_dump_refs.phpt` | `debug_zval_dump()` reference formatting/refcount surface is unsupported. | unsupported: reference inspection internal |
