# COW PHPT Blockers: 2026-06-09

Evidence base:

- Refreshed at 2026-06-09T23:02Z on `9d5c6070073e`
  (`origin/master`, `ptn-cqu.26`).
- No `tools/phpt-cow-manifest.txt` row changes were needed: 29 rows in seven
  buckets.
- Before evidence from 2026-06-09T20:32Z: 29 total, 9 pass, 20 fail.
- After evidence from 2026-06-09T23:02Z: 29 total, 9 pass, 20 fail.
- `tools/run-bounded-phpt.sh tools/phpt-cow-manifest.txt` still stops in the
  nested-arrays bucket because `Zend/tests/bug38469.phpt` exhausts
  `run-tests.php` diff memory. The other rows were rerun bucket-wise; counting
  `bug38469` as failing gives the manifest total.

## Focused COW Counts

| Bucket | Total | Pass | Fail |
| --- | ---: | ---: | ---: |
| assignment-aliasing | 4 | 3 | 1 |
| string-offsets | 4 | 2 | 2 |
| array-writes-appends-unset | 4 | 3 | 1 |
| nested-arrays | 4 | 0 | 4 |
| foreach-mutation | 4 | 1 | 3 |
| function-boundaries | 4 | 0 | 4 |
| reference-interaction | 5 | 0 | 5 |
| **Total** | **29** | **9** | **20** |

Before and after bucket counts are unchanged from the 2026-06-09T20:32Z
measurement.

## Linked Generic Blocker Beads

| Generic blocker | Rows held | Bead |
| --- | ---: | --- |
| PHP-exact assignment/offset diagnostics | 1 | `ptn-4yt.5` |
| String offset reference diagnostics and mutation COW | 2 | `ptn-4yt.6` |
| Overlapping array-dim compound assignment snapshots | 1 | `ptn-4yt.1` |
| Nested recursive reference lvalues, recursive array internals, and cycle-safe dumps | 4 | `ptn-4yt.2` |
| By-reference foreach over unsupported source forms and callback mutation | 3 | `ptn-4yt.7`, `ptn-4yt.3` |
| By-reference returns, call-result references, and callback return references | 4 | `ptn-4yt.8`, `ptn-4yt.3` |
| Reference array literals, reference append lvalues, reference-aware internals, and reference inspection | 5 | `ptn-4yt.9`, `ptn-4yt.3` |

## Fixed Rows

| Bucket | PHPT row | Generic fix | Compact reducer |
| --- | --- | --- | --- |
| array-writes-appends-unset | `ext/standard/tests/array/array_unshift_basic1.phpt` | `array_unshift()` mutates direct variable arrays, detaches shared payloads, prepends values, reindexes integer keys, and preserves string keys. | `array_unshift_shared_alias` |
| foreach-mutation | `Zend/tests/foreach/foreach_reference.phpt` | `array_reverse()` is registered, and `array_values()`/`array_reverse()` unwrap single-owner references while preserving shared references. | `array_reindexing_internals_unwrap_single_owner_refs` |

## Compact Reductions

The native reducer suite now covers 18 focused COW reducers plus 10 dynamic
temporary/read-slot reducers.

| PHPT row | Compact reduction | Current result |
| --- | --- | --- |
| `ext/standard/tests/array/array_unshift_basic1.phpt` | `$b=$a; array_unshift($b, 10);` | fixed |
| `Zend/tests/foreach/foreach_reference.phpt` | by-reference `foreach`, then `array_values()` and `array_reverse()` | fixed |
| `Zend/tests/assign_dim_op_same_var.phpt` | `$ary=[[]]; $ary[0]+=$ary; var_dump($ary[0]);` | still emits `int(1)`; needs overlapping array-dim assign-op snapshots |
| `Zend/tests/str_offset_002.phpt` / `Zend/tests/string_offset_optimization.phpt` | `$a="aaa"; $x=[&$a[1]];` | parser rejects the reference expression instead of raising PHP `Error` |
| `Zend/tests/bug35163.phpt` | `$a=[[1]]; $a[0][] =& $a[0]; $a[0][0]=2;` | needs nested recursive reference lvalues and cycle-safe dump |
| `Zend/tests/foreach/foreach_temp_array_expr_with_refs.phpt` | `foreach ([&$a, &$b] as &$value) { ... }` | needs temporary reference-array literals plus by-reference foreach |
| `Zend/tests/array_with_refs_identical.phpt` | `$array1=[&$foo]; $array2=[$foo]; $array1 === $array2` | needs strict comparison that dereferences array entries like PHP |
| `ext/standard/tests/array/array_sum_on_reference.phpt` | `$nums=[&$n, 100]; array_sum($nums);` | needs reference-aware numeric internals |

## Remaining Blocker Rows

| Bucket | PHPT row | Current generic blocker | Bead |
| --- | --- | --- | --- |
| assignment-aliasing | `Zend/tests/assign_to_var_003.phpt` | Value result is correct; float offset warning text differs. | `ptn-4yt.5` |
| string-offsets | `Zend/tests/str_offset_002.phpt` | `&$a[0]` is rejected while PHP raises `Error`. | `ptn-4yt.6` |
| string-offsets | `Zend/tests/string_offset_optimization.phpt` | Same string-offset reference form inside a function. | `ptn-4yt.6` |
| array-writes-appends-unset | `Zend/tests/assign_dim_op_same_var.phpt` | Compound array union assignment with overlapping LHS/RHS yields `int(1)`. | `ptn-4yt.1` |
| nested-arrays | `Zend/tests/bug35163.phpt` | Nested reference lvalue is rejected. | `ptn-4yt.2` |
| nested-arrays | `Zend/tests/bug38469.phpt` | Recursive copied value exhausts PHPT diff memory. | `ptn-4yt.2` |
| nested-arrays | `ext/standard/tests/array/array_merge_recursive_basic1.phpt` | `array_merge_recursive()` is not registered. | `ptn-4yt.2` |
| nested-arrays | `ext/standard/tests/array/array_merge_replace_recursive_refs.phpt` | Reference array literal blocks before recursive merge/replace semantics. | `ptn-4yt.2`, `ptn-4yt.9` |
| foreach-mutation | `Zend/tests/foreach/foreach_by_ref_repacking_insert.phpt` | Leading inline whitespace before `<?php` is rejected. | `ptn-4yt.7` |
| foreach-mutation | `Zend/tests/foreach/foreach_temp_array_expr_with_refs.phpt` | Temporary array literal containing references is rejected. | `ptn-4yt.7`, `ptn-4yt.3`, `ptn-4yt.9` |
| foreach-mutation | `ext/standard/tests/array/array_walk/bug69068_2.phpt` | Closure with by-reference callback and global swap is unsupported. | `ptn-4yt.7`, `ptn-4yt.3` |
| function-boundaries | `Zend/tests/return_types/return_reference_separation.phpt` | `int`/`string` hints and by-reference returns block parsing. | `ptn-4yt.8` |
| function-boundaries | `Zend/tests/assign_by_val_function_by_ref_return_value.phpt` | Assignment by reference from a function result is rejected. | `ptn-4yt.8`, `ptn-4yt.3` |
| function-boundaries | `ext/standard/tests/array/array_reduce_accumulator_refcount.phpt` | Closure plus `debug_zval_dump()` refcount surface is unsupported. | `ptn-4yt.8`, `ptn-4yt.3`, `ptn-4yt.9` |
| function-boundaries | `ext/standard/tests/array/array_reduce_return_by_ref.phpt` | `array_reduce()` callback returning by reference is unsupported. | `ptn-4yt.8`, `ptn-4yt.3` |
| reference-interaction | `Zend/tests/array_with_refs_identical.phpt` | Array literal containing references blocks strict identity comparison. | `ptn-4yt.9` |
| reference-interaction | `Zend/tests/assign_dim_ref_free.phpt` | Chained append/reference assignment is rejected. | `ptn-4yt.9` |
| reference-interaction | `ext/standard/tests/array/array_sum_on_reference.phpt` | Reference array literal blocks before `array_sum()` runs. | `ptn-4yt.9` |
| reference-interaction | `ext/standard/tests/strings/strtr_with_reference.phpt` | Reference array literal blocks before `strtr()` runs. | `ptn-4yt.9` |
| reference-interaction | `ext/standard/tests/general_functions/debug_zval_dump_refs.phpt` | Reference array literal blocks refcount formatting coverage. | `ptn-4yt.9` |
