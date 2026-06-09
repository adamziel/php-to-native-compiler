# COW PHPT Blockers: 2026-06-09

Evidence base:

- Current branch on `92442293aa88` plus the `array_unshift()` implementation.
- `tools/run-bounded-phpt.sh tools/phpt-cow-manifest.txt` still aborts in the
  nested-array bucket because `Zend/tests/bug38469.phpt` exhausts
  `run-tests.php` diff memory.
- Row-level rerun of the other manifest rows: 28 total, 8 pass, 20 fail.
  Counting `bug38469` as a fail gives 29 total, 8 pass, 21 fail.
- Fixed row: `ext/standard/tests/array/array_unshift_basic1.phpt` now passes
  through a generic mutating-internal implementation.

## Focused COW Counts

| Bucket | Total | Pass | Fail |
| --- | ---: | ---: | ---: |
| assignment-aliasing | 4 | 3 | 1 |
| string-offsets | 4 | 2 | 2 |
| array-writes-appends-unset | 4 | 3 | 1 |
| nested-arrays | 4 | 0 | 4 |
| foreach-mutation | 4 | 0 | 4 |
| function-boundaries | 4 | 0 | 4 |
| reference-interaction | 5 | 0 | 5 |
| **Total** | **29** | **8** | **21** |

## Fresh Reductions

- Fixed `array_unshift_basic1`: `$b = $a; array_unshift($b, 10);` detaches the
  variable array, prepends values, reindexes integer keys, and preserves string
  keys.
- `assign_dim_op_same_var`: compact repro `$ary = [[]]; $ary[0] += $ary;`
  currently prints `int(1)` instead of the original nested empty array.
- `foreach_reference`: by-reference foreach reaches runtime output, but
  `array_values()` over reference cells over-preserves references and
  `array_reverse()` is not registered.
- `bug35163`: compact repro `$a = [[1]]; $a[0][] =& $a[0];` is blocked at
  the nested reference-lvalue diagnostic.
- Reference array literal rows reduce to `[$foo]` vs `[&$foo]` forms; parser
  stops at reference expressions inside literals before internals run.
- `bug38469`: direct native output still builds a recursive value that makes
  PHPT diffing run out of memory instead of dumping `*RECURSION*`.

## Remaining Blocker Rows

| Bucket | PHPT row | Current blocker | Map |
| --- | --- | --- | --- |
| assignment-aliasing | `Zend/tests/assign_to_var_003.phpt` | Value result is correct; float offset warning text differs. | unsupported: PHP-exact offset diagnostic wording |
| string-offsets | `Zend/tests/str_offset_002.phpt` | `&$a[0]` is rejected while PHP raises `Error`. | unsupported: references to/from string offsets |
| string-offsets | `Zend/tests/string_offset_optimization.phpt` | Same string-offset reference form inside a function. | unsupported: references to/from string offsets |
| array-writes-appends-unset | `Zend/tests/assign_dim_op_same_var.phpt` | Compound array union assignment with overlapping LHS/RHS yields `int(1)`. | blocker: overlapping array-dim assign-op snapshot |
| nested-arrays | `Zend/tests/bug35163.phpt` | Nested reference lvalue is rejected. | unsupported: nested reference lvalues |
| nested-arrays | `Zend/tests/bug38469.phpt` | Recursive copied value exhausts PHPT diff memory. | blocker: recursive array dump/cycle handling |
| nested-arrays | `ext/standard/tests/array/array_merge_recursive_basic1.phpt` | `array_merge_recursive()` is not registered. | unsupported: recursive array internal |
| nested-arrays | `ext/standard/tests/array/array_merge_replace_recursive_refs.phpt` | Reference array literal blocks before recursive merge/replace semantics. | unsupported: reference array literals and recursive internals |
| foreach-mutation | `Zend/tests/foreach/foreach_reference.phpt` | By-ref loop runs, then `array_values()` reference surface and missing `array_reverse()` fail. | blocker: reference-aware array copy internals |
| foreach-mutation | `Zend/tests/foreach/foreach_by_ref_repacking_insert.phpt` | Leading inline whitespace before `<?php` is rejected. | unsupported: mixed/inline open-tag handling |
| foreach-mutation | `Zend/tests/foreach/foreach_temp_array_expr_with_refs.phpt` | Temporary array literal containing references is rejected. | unsupported: reference array literals |
| foreach-mutation | `ext/standard/tests/array/array_walk/bug69068_2.phpt` | Closure with by-reference callback and global swap is unsupported. | unsupported: closures/use/globals callback mutation |
| function-boundaries | `Zend/tests/return_types/return_reference_separation.phpt` | `int`/`string` hints and by-reference returns block parsing. | unsupported: typed by-reference returns |
| function-boundaries | `Zend/tests/assign_by_val_function_by_ref_return_value.phpt` | Assignment by reference from a function result is rejected. | unsupported: by-reference assignment from call result |
| function-boundaries | `ext/standard/tests/array/array_reduce_accumulator_refcount.phpt` | Closure plus `debug_zval_dump()` refcount surface is unsupported. | unsupported: closures and refcount inspection internal |
| function-boundaries | `ext/standard/tests/array/array_reduce_return_by_ref.phpt` | `array_reduce()` callback returning by reference is unsupported. | unsupported: by-reference callback returns |
| reference-interaction | `Zend/tests/array_with_refs_identical.phpt` | Array literal containing references blocks strict identity comparison. | unsupported: reference array literal identity |
| reference-interaction | `Zend/tests/assign_dim_ref_free.phpt` | Chained append/reference assignment is rejected. | unsupported: reference append lvalue |
| reference-interaction | `ext/standard/tests/array/array_sum_on_reference.phpt` | Reference array literal blocks before `array_sum()` runs. | unsupported: reference-aware numeric array internal |
| reference-interaction | `ext/standard/tests/strings/strtr_with_reference.phpt` | Reference array literal blocks before `strtr()` runs. | unsupported: reference-aware string replacement internal |
| reference-interaction | `ext/standard/tests/general_functions/debug_zval_dump_refs.phpt` | Reference array literal blocks refcount formatting coverage. | unsupported: reference inspection internal |
