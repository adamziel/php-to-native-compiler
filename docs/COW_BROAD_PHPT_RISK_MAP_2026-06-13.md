# COW Broad PHPT Risk Map: 2026-06-13

Issue: `ptn-550s.1`

This inventory expands the focused 29-row COW manifest into deterministic broad
1k/5k PHPT baseline evidence. It is a blocker map, not an implementation
change: rows are grouped by generic PHP semantic surface so later slices can
move large clusters without row-specific fixes.

## Evidence

Source commits:

- PTN: `1c33e9d8bfcab87704e57b8705ac5d83e9fd8bfa`
- php-src PHPT corpus: `8c63ec400ce8e07c57a8d9499317b96a8beafb8b`
- The php-src checkout had three unrelated untracked `fprintf_variation_*.txt`
  fixtures; the generated manifests below contain tracked PHPT paths.

Commands run:

```sh
rm -rf .runtime/phpt-baseline-cow-550s1
tools/run-phpt-baseline.sh --tier 1000 --classify-only --out-dir .runtime/phpt-baseline-cow-550s1
tools/run-phpt-baseline.sh --tier 5000 --classify-only --out-dir .runtime/phpt-baseline-cow-550s1
tools/run-bounded-phpt.sh tools/phpt-cow-manifest.txt
git rev-parse HEAD
git -C /home/claude/php-src-phpt rev-parse HEAD
git -C /home/claude/php-src-phpt status --short --branch
```

Generated broad artifacts:

- 1k run: `.runtime/phpt-baseline-cow-550s1/20260613T183629Z/phpt-baseline-1000.txt`
- 1k classification: `.runtime/phpt-progress/classification-20260613T183629Z.tsv`
- 5k run: `.runtime/phpt-baseline-cow-550s1/20260613T184028Z/phpt-baseline-5000.txt`
- 5k classification: `.runtime/phpt-progress/classification-20260613T184028Z.tsv`
- Focused COW run: `.runtime/phpt-progress/run-20260613T190317Z-*.log`

Broad classifier summaries:

| Tier | Selected | Runnable | Excluded | Main excluded buckets |
| --- | ---: | ---: | ---: | --- |
| 1k | 1,000 | 431 | 569 | unsupported-language 403, unsupported-ini 73, unsupported-class-metadata 51, unsupported-extension 20 |
| 5k | 5,000 | 2,564 | 2,436 | unsupported-language 1,061, unsupported-class-metadata 564, harness-cleanup 371, unsupported-ini 271, unsupported-extension 91 |

Focused COW sanity check:

```text
tools/run-bounded-phpt.sh tools/phpt-cow-manifest.txt
result: buckets=7 selected=29 runnable=29 excluded=0 tests=29 passed=29 failed=0 skipped=0 warned=0
```

The deterministic 5k tier selected 2,646 sorted `Zend/tests` rows through
`Zend/tests/inheritance/abstract_inheritance_001.phpt`, 1,924 sorted
`ext/standard/tests` rows through `ext/standard/tests/filters/bug46164-1.phpt`,
and 430 sorted core `tests` rows through `tests/lang/022.phpt`. This means the
5k tier is rich in reference/function/foreach/property rows and ext/standard
array internals, but it does not reach later sorted string-offset rows. String
offset COW remains tracked by the focused manifest and `ptn-550s.5`.

## Follow-up Map

| Cluster | Follow-up bead |
| --- | --- |
| Foreach and reference mutation | `ptn-550s.2` |
| Array internals and mutating helper separation | `ptn-550s.3` |
| Function boundary by-reference semantics | `ptn-550s.4` |
| String offset and scalar alias separation | `ptn-550s.5` |
| Object and property reference prerequisites | `ptn-550s.6` |
| Generator and fiber reference boundaries | `ptn-550s.7` |
| Interface/trait/abstract metadata prerequisite | `ptn-0pys` |
| Class constants/static properties/static variables prerequisite | `ptn-nya1` |
| First-class callable syntax/metadata prerequisite | `ptn-98lj` |
| Readonly property prerequisite | `ptn-qsmv.12` |

## Classified Broad Rows

The rows below are all from the broad 5k manifest unless explicitly noted. They
classify 92 COW-sensitive broad rows into implementation clusters or explicit
blockers.

### Assignment And Direct Reference Cells

Follow-up: `ptn-550s.2` / `ptn-550s.4`.

Mostly runnable; remaining blockers are metadata prerequisites, not row-specific
COW patches.

- runnable: `Zend/tests/__debugInfo_reference.phpt`
- runnable: `Zend/tests/array_append_COW.phpt`
- runnable: `Zend/tests/array_append_by_reference.phpt`
- runnable: `Zend/tests/array_unshift_COW.phpt`
- runnable: `Zend/tests/array_with_refs_identical.phpt`
- runnable: `Zend/tests/assign_dim_ref_free.phpt`
- runnable: `Zend/tests/assign_ref_error_var_handling.phpt`
- runnable: `Zend/tests/assign_ref_func_leak.phpt`
- runnable: `Zend/tests/bw_or_assign_with_ref.phpt`
- runnable: `Zend/tests/by_ref_optimization.phpt`
- runnable: `Zend/tests/div_by_zero_compound_refcounted.phpt`
- runnable: `Zend/tests/indirect_reference_this.phpt`
- unsupported-class-metadata: `Zend/tests/call_with_refs.phpt` needs unsupported magic method dispatch/reflection metadata.

### Function And Callable Boundaries

Follow-up: `ptn-550s.4`; callable syntax prerequisite: `ptn-98lj`.

These rows stress by-reference parameters, by-reference returns, unpacking,
callable dispatch, and reference identity crossing call boundaries.

- runnable: `Zend/tests/add_optional_by_ref_arg.phpt`
- runnable: `Zend/tests/assign_by_val_function_by_ref_return_value.phpt`
- runnable: `Zend/tests/call_user_functions/call_user_func_array_prefer_ref.phpt`
- runnable: `Zend/tests/call_user_functions/call_user_func_by_ref.phpt`
- runnable: `Zend/tests/function_arguments/sensitive_parameter_value_reflection.phpt`
- unsupported-language: `Zend/tests/arg_unpack/by_ref.phpt` needs call-site/array unpacking.
- unsupported-language: `Zend/tests/arg_unpack/by_ref_separation.phpt` needs call-site/array unpacking.
- unsupported-language: `Zend/tests/arg_unpack/traversable_with_by_ref_parameters.phpt` needs call-site/array unpacking.
- unsupported-language: `Zend/tests/dynamic_call/dynamic_call_to_ref_returning_function.phpt` needs anonymous class syntax.
- unsupported-language: `Zend/tests/exception_with_by_ref_message.phpt` needs userland `throw` lowering.
- unsupported-language: `Zend/tests/first_class_callable/first_class_callable_refs.phpt` needs first-class callable syntax/metadata.
- unsupported-language: `Zend/tests/first_class_callable/first_class_callable_signature.phpt` needs first-class callable syntax/metadata.
- unsupported-language: `Zend/tests/function_arguments/sensitive_parameter_closure.phpt` needs attribute syntax/reflection metadata.

### Foreach And Iteration Mutation

Follow-up: `ptn-550s.2`.

These rows are the broad continuation of the focused foreach COW bucket.

- runnable: `Zend/tests/foreach/foreach.phpt`
- runnable: `Zend/tests/foreach/foreach_005.phpt`
- runnable: `Zend/tests/foreach/foreach_by_ref_repacking_insert.phpt`
- runnable: `Zend/tests/foreach/foreach_by_ref_to_property.phpt`
- runnable: `Zend/tests/foreach/foreach_reference.phpt`
- runnable: `Zend/tests/foreach/foreach_temp_array_expr_with_refs.phpt`
- runnable: `Zend/tests/foreach/foreach_unset_globals.phpt`
- runnable: `Zend/tests/foreach/goto_in_foreach.phpt`
- unsupported-ini: `Zend/tests/foreach/foreach_002.phpt` needs `zend.enable_gc`.
- unsupported-language: `Zend/tests/foreach/foreach_003.phpt` needs interface implementation checks.

### Closure Capture And Closure Callable References

Follow-up: `ptn-550s.2` / `ptn-550s.4`; callable prerequisite: `ptn-98lj`.

The runnable rows are good candidates for closure capture reference reducers.
The blocked rows are unpacking/callable-boundary prerequisites.

- runnable: `Zend/tests/closures/closure_bindTo_preserves_used_variables.phpt`
- runnable: `Zend/tests/closures/closure_from_callable_reflection.phpt`
- runnable: `Zend/tests/closures/closure_invoke_ref_warning.phpt`
- runnable: `Zend/tests/closures/closure_use_auto_global.phpt`
- runnable: `Zend/tests/closures/closure_use_parameter_name.phpt`
- runnable: `Zend/tests/closures/closure_use_trailing_comma.phpt`
- runnable: `Zend/tests/closures/closure_use_variable_twice.phpt`
- unsupported-language: `Zend/tests/closures/closure_get_current.phpt` needs call-site/array unpacking.
- unsupported-language: `Zend/tests/closures/fake_closure_in_internal_func_leaks.phpt` needs call-site/array unpacking.

### Array Internals And Mutating Helpers

Follow-up: `ptn-550s.3`.

This is the strongest broad payoff cluster in the 5k tier. It combines array
callback helpers, reducer ownership, merge/replace recursive reference
unwrapping, shift/splice/unshift mutation, and `array_walk` callback mutation.

- runnable: `ext/standard/tests/array/array_filter.phpt`
- runnable: `ext/standard/tests/array/array_merge_recursive_basic1.phpt`
- runnable: `ext/standard/tests/array/array_merge_replace_recursive_refs.phpt`
- runnable: `ext/standard/tests/array/array_reduce.phpt`
- runnable: `ext/standard/tests/array/array_reduce_accumulator_refcount.phpt`
- runnable: `ext/standard/tests/array/array_reduce_return_by_ref.phpt`
- runnable: `ext/standard/tests/array/array_replace.phpt`
- runnable: `ext/standard/tests/array/array_replace_merge_recursive_ref.phpt`
- runnable: `ext/standard/tests/array/array_shift_basic.phpt`
- runnable: `ext/standard/tests/array/array_splice_basic.phpt`
- runnable: `ext/standard/tests/array/array_sum_on_reference.phpt`
- runnable: `ext/standard/tests/array/array_unshift_basic1.phpt`
- runnable: `ext/standard/tests/array/array_walk/array_walk_recursive.phpt`
- runnable: `ext/standard/tests/array/array_walk/bug69068_2.phpt`
- unsupported-language: `ext/standard/tests/array/array_map_001.phpt` needs userland `throw` lowering.
- unsupported-language: `ext/standard/tests/array/array_walk/array_walk.phpt` needs userland `throw` lowering.
- unsupported-language: `ext/standard/tests/array/array_walk/array_walk_closure.phpt` needs userland `throw` lowering.

### Object, Property, And Class Metadata References

Follow-up: `ptn-550s.6`. Prerequisites: `ptn-0pys`, `ptn-nya1`,
`ptn-qsmv.12`.

Do not treat these as ordinary COW runtime bugs until property metadata,
visibility, enum, and static-property semantics are represented generically.

- runnable: `Zend/tests/assign_obj_ref_byval_function.phpt`
- runnable: `Zend/tests/assign_obj_ref_return.phpt`
- runnable: `Zend/tests/assign_ref_to_overloaded_prop.phpt`
- runnable: `Zend/tests/exceptions/exception_getters_with_ref_props.phpt`
- unsupported-class-metadata: `Zend/tests/assign_obj_to_ref_inference.phpt` needs typed property metadata.
- unsupported-class-metadata: `Zend/tests/assign_typed_ref_result.phpt` needs typed property metadata.
- unsupported-class-metadata: `Zend/tests/enum/no-pass-properties-by-ref.phpt` needs enum declarations/case metadata.
- unsupported-class-metadata: `Zend/tests/enum/no-return-properties-by-ref.phpt` needs enum declarations/case metadata.
- unsupported-class-metadata: `Zend/tests/enum/no-write-properties-through-foreach-reference.phpt` needs enum declarations/case metadata.
- unsupported-class-metadata: `Zend/tests/enum/no-write-properties-through-references.phpt` needs enum declarations/case metadata.
- unsupported-class-metadata: `Zend/tests/gh10168/assign_prop_ref.phpt` needs typed property metadata.
- unsupported-class-metadata: `Zend/tests/gh10168/assign_static_prop_ref.phpt` needs typed property metadata.
- unsupported-language: `Zend/tests/asymmetric_visibility/object_reference.phpt` needs asymmetric property visibility.
- unsupported-language: `Zend/tests/asymmetric_visibility/reference.phpt` needs asymmetric property visibility.
- unsupported-language: `Zend/tests/asymmetric_visibility/reference_2.phpt` needs asymmetric property visibility.
- unsupported-language: `Zend/tests/asymmetric_visibility/static_props.phpt` needs asymmetric property visibility.
- unsupported-language: `Zend/tests/exceptions/exception_during_by_reference_magic_get.phpt` needs userland `throw` lowering.

### Generator And Fiber Reference Boundaries

Follow-up: `ptn-550s.7`.

The classifier marks many generator/fiber rows runnable, but PTN does not yet
document generator/fiber support as part of the current RC surface. Treat this
cluster as a prerequisite/runtime-boundary map before using it as a COW pass
count target.

- runnable: `Zend/tests/fibers/return-by-ref.phpt`
- runnable: `Zend/tests/generators/errors/non_ref_generator_iterated_by_ref_error.phpt`
- runnable: `Zend/tests/generators/errors/yield_const_by_ref_error.phpt`
- runnable: `Zend/tests/generators/errors/yield_non_ref_function_call_by_ref_error.phpt`
- runnable: `Zend/tests/generators/gc_with_iterator_in_foreach.phpt`
- runnable: `Zend/tests/generators/no_foreach_var_leaks.phpt`
- runnable: `Zend/tests/generators/return_from_by_ref_generator.phpt`
- runnable: `Zend/tests/generators/yield_array_offset_by_ref.phpt`
- runnable: `Zend/tests/generators/yield_by_reference.phpt`
- runnable: `Zend/tests/generators/yield_by_reference_optimization.phpt`
- runnable: `Zend/tests/generators/yield_from_by_reference.phpt`
- runnable: `Zend/tests/generators/yield_ref_function_call_by_reference.phpt`
- unsupported-language: `Zend/tests/generators/generator_method_by_ref.phpt` needs interface implementation checks.

### String Offset And Scalar Aliasing

Follow-up: `ptn-550s.5`.

The deterministic 1k/5k tiers do not reach the later sorted Zend string-offset
rows. The focused COW manifest covers this surface and passed 4/4 in the
`string-offsets` bucket:

- focused/runnable: `Zend/tests/str_offset_001.phpt`
- focused/runnable: `Zend/tests/str_offset_002.phpt`
- focused/runnable: `Zend/tests/str_offset_003.phpt`
- focused/runnable: `Zend/tests/string_offset_optimization.phpt`

`ptn-550s.5` should use either the focused rows plus hand-picked broad strings
rows or a later deterministic tier that actually samples `Zend/tests/str*`.

## Recommended Order

1. Continue `ptn-550s.3` for ext/standard array internals because the 5k tier
   contains a dense runnable cluster and the focused COW reducer already passes.
2. Continue `ptn-550s.2` and `ptn-550s.4` with the runnable Zend reference,
   foreach, closure, and callable rows above.
3. Keep `ptn-550s.6` blocked behind property/class metadata prerequisites when
   typed/asymmetric/enum rows are involved; only runnable property rows should
   enter immediate COW reducer work.
4. Treat `ptn-550s.7` as a blocker map first because generator/fiber rows are
   not part of the current documented RC surface.
5. Use `ptn-550s.5` for focused or higher-tier string/scalar aliasing; do not
   infer broad 5k coverage from the absence of string rows in this deterministic
   tier.
