# PHPT Broad 1k Zend Assignment Frontier: 2026-06-14

Issue: `ptn-gwlo`

This slice refreshes the broad 1k baseline on `origin/master`, then narrows a
non-array-helper Zend cluster that is still large enough to matter but small
enough to map precisely: array/reference/object assignment and lvalue edge
cases.

## Broad 1k Baseline

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only --out-dir .runtime/ptn-gwlo-baseline-post-lrlt
```

Generated manifest:
`.runtime/ptn-gwlo-baseline-post-lrlt/20260614T011318Z/phpt-baseline-1000.txt`

Corpus revision: `8c63ec400ce8e07c57a8d9499317b96a8beafb8b`

Classifier result:

| Bucket | Rows |
| --- | ---: |
| Selected | 1,000 |
| Runnable | 422 |
| Excluded | 578 |
| `unsupported-language` | 351 |
| `unsupported-class-metadata` | 84 |
| `unsupported-request-input-ini` | 28 |
| `unsupported-extension` | 20 |
| `unsupported-assertion-ini` | 17 |
| `unsupported-diagnostics-runtime` | 16 |
| `unsupported-resource-limit-ini` | 15 |
| `sapi-behavior` | 13 |
| `unsupported-assertion-runtime` | 9 |
| Other classifier buckets | 25 |

Runnable broad rows by top-level source:

| Source | Rows |
| --- | ---: |
| `ext/standard/tests` | 276 |
| `Zend/tests` | 130 |
| `tests/basic` | 16 |

The existing broad array-helper maps cover the largest standard bucket. This
slice targets the 32-row Zend assignment/reference cluster from the runnable
manifest instead.

## Focused Manifest

Committed manifest:
`tools/phpt-zend-assignment-reference-frontier-manifest.txt`

Selection regex over `.runtime/phpt-progress/runnable-20260614T011318Z.txt`:

```text
^Zend/tests/(array_(add|addition|append|hash|literal|merge_recursive_next|self|splice|unshift|with_refs)|assign_|add_optional_by_ref_arg)
```

Focused run:

```sh
tools/run-bounded-phpt.sh tools/phpt-zend-assignment-reference-frontier-manifest.txt
```

Result at `.runtime/phpt-progress/run-20260614T012211Z-manifest.log`:

| Selected | Runnable | Passed | Failed | Skipped | Warned |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 32 | 32 | 22 | 10 | 0 | 0 |

## Row Outcomes

| Row | Result | Bucket |
| --- | --- | --- |
| `Zend/tests/add_optional_by_ref_arg.phpt` | pass | by-reference method parameter |
| `Zend/tests/array_add_indirect.phpt` | pass | array addition / indirect elements |
| `Zend/tests/array_addition_not_commutative.phpt` | pass | array addition order |
| `Zend/tests/array_append_COW.phpt` | pass | append COW split |
| `Zend/tests/array_append_by_reference.phpt` | fail | append expression as by-reference argument |
| `Zend/tests/array_append_reading_error.phpt` | fail | append expression read diagnostic |
| `Zend/tests/array_hash_zero.phpt` | pass | zero hash keys |
| `Zend/tests/array_literal_next_element_error.phpt` | fail | next-key overflow and array default constants |
| `Zend/tests/array_merge_recursive_next_key_overflow.phpt` | fail | nested next-key overflow and trailing-call syntax |
| `Zend/tests/array_self_add_globals.phpt` | pass | self-add globals |
| `Zend/tests/array_splice_empty_ht_iter_removal.phpt` | fail | iterator invalidation during splice |
| `Zend/tests/array_unshift_COW.phpt` | pass | unshift COW split |
| `Zend/tests/array_with_refs_identical.phpt` | pass | reference-aware identity |
| `Zend/tests/assign_array_object_property.phpt` | fail | object property lvalue through array slot |
| `Zend/tests/assign_by_val_function_by_ref_return_value.phpt` | pass | by-ref assignment diagnostic |
| `Zend/tests/assign_dim_obj_null_return.phpt` | fail | dimension/object assignment error ordering |
| `Zend/tests/assign_dim_op_same_var.phpt` | pass | compound array assignment |
| `Zend/tests/assign_dim_op_undef.phpt` | pass | undefined compound array assignment |
| `Zend/tests/assign_dim_ref_free.phpt` | pass | append reference assignment |
| `Zend/tests/assign_obj_op_cache_slot.phpt` | fail | dynamic property compound lvalue |
| `Zend/tests/assign_obj_ref_byval_function.phpt` | pass | object property by-ref assignment |
| `Zend/tests/assign_obj_ref_return.phpt` | pass | object assign return value |
| `Zend/tests/assign_op_type_error.phpt` | fail | compound operator TypeError parity |
| `Zend/tests/assign_property_null_object.phpt` | pass | null object assignment diagnostic |
| `Zend/tests/assign_ref_error_var_handling.phpt` | pass | ref-assign error precedence |
| `Zend/tests/assign_ref_func_leak.phpt` | pass | by-ref assignment cleanup |
| `Zend/tests/assign_to_obj_001.phpt` | pass | object assignment with refs |
| `Zend/tests/assign_to_obj_002.phpt` | fail | `$this` outside object context |
| `Zend/tests/assign_to_var_001.phpt` | pass | complex variable assignment |
| `Zend/tests/assign_to_var_002.phpt` | pass | complex variable assignment |
| `Zend/tests/assign_to_var_003.phpt` | pass | complex variable assignment |
| `Zend/tests/assign_to_var_004.phpt` | pass | complex variable assignment |

## Failure Buckets

The ten failures are real implementation blockers, but they do not form one
credible 25-row patch. They split into smaller generic runtime/parser
boundaries:

| Rows | Blocker | Representative evidence |
| ---: | --- | --- |
| 2 | Append-expression lvalue context needs to distinguish write, read, and by-reference argument binding. | `array_append_by_reference.phpt` currently rejects `$arr[]` before by-ref binding; `array_append_reading_error.phpt` reports PTN's generic append-target fatal instead of PHP's read diagnostic. |
| 2 | Next-free-index overflow is not represented uniformly across array literals, defaults, and recursive array helpers. | `array_literal_next_element_error.phpt` fails before runtime in a default-constant-expression path; `array_merge_recursive_next_key_overflow.phpt` fails at parser syntax before reaching helper overflow handling. |
| 1 | Live `foreach` by-reference iterator state is not invalidated after `array_splice()` empties the iterated array. | `array_splice_empty_ht_iter_removal.phpt` emits one fewer repeated `int(4)` than PHP. |
| 3 | Object/member lvalues through array slots and dynamic member names need broader object write lowering. | `assign_array_object_property.phpt` rejects `new $arr[0]`; `assign_dim_obj_null_return.phpt` reports only the first illegal offset; `assign_obj_op_cache_slot.phpt` rejects `$a->$b &= 1`. |
| 1 | Compound operator TypeError parity is incomplete for `%`, `<<`, and `>>` after array/string operands. | `assign_op_type_error.phpt` reports the earlier arithmetic TypeErrors but misses the final three operator diagnostics. |
| 1 | `$this` outside object context should be a catchable `Error` before property assignment, not an undefined-variable warning followed by null-property assignment. | `assign_to_obj_002.phpt`. |

## Implementation Boundary

The passing rows show that PTN already covers direct array addition, common
array COW/reference writes, reference-assignment cleanup, object-property
by-reference assignment, and complex direct variable assignment. The remaining
failures cross several separate boundaries:

- parser/expression lowering for append expressions and dynamic member names;
- constant-expression lowering for array default values with overflowing next
  keys;
- ordered-array next-key overflow checks in recursive helper insertion paths;
- object/member lvalue lowering through array slots and dynamic property names;
- PHP error-channel ordering for dimension/object assignment failures;
- compound operator TypeError coverage for modulo and bit shifts;
- function-frame `$this` lookup semantics outside object context.

Because the largest coherent failure group is three rows, this slice is a
blocker map rather than an implementation patch. A future implementation should
split first on append-expression lvalue context or dynamic object-member lvalue
lowering, then re-run this manifest plus the broad 1k classify-only tier.

## Verification

```sh
cargo fmt --check
cargo test phpt_classifier
tools/run-phpt-baseline.sh --tier 1000 --classify-only --out-dir .runtime/ptn-gwlo-baseline
tools/run-bounded-phpt.sh tools/phpt-zend-assignment-reference-frontier-manifest.txt
```
