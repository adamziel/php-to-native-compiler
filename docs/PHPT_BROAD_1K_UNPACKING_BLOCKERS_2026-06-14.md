# Broad PHPT 1k Unpacking Blocker Map: 2026-06-14

Issue: `ptn-v1mu`

This slice used the broad PHPT baseline tooling on `origin/master` and selected
call-site/array unpacking as a high-yield semantic blocker. This is a blocker
map, not a support claim: unpacking crosses parser, AST/IR representation, call
argument expansion, array literal spread, by-reference calls, traversable
iteration, destructuring diagnostics, and internal-function argument handling.

## Evidence

Source state:

- PTN: `504c43df009c`
- php-src PHPT corpus: `8c63ec400ce8e07c57a8d9499317b96a8beafb8b`

Commands:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only --out-dir .runtime/ptn-v1mu-before
rg '^ext/standard/tests/array/array_chunk' \
  .runtime/phpt-progress/runnable-20260614T014121Z.txt \
  > .runtime/ptn-v1mu/array-chunk-runnable.txt
tools/run-bounded-phpt.sh .runtime/ptn-v1mu/array-chunk-runnable.txt
```

Generated broad manifest:

```text
.runtime/ptn-v1mu-before/20260614T014121Z/phpt-baseline-1000.txt
```

Classify-only result:

| Measurement | Selected | Runnable | Excluded |
| --- | ---: | ---: | ---: |
| broad 1k classify-only | 1000 | 422 | 578 |

Unpacking rows:

| Sub-bucket | Rows |
| --- | ---: |
| `Zend/tests/arg_unpack/` | 13 |
| `Zend/tests/array_unpack/` | 13 |
| `Zend/tests/array_unpack_string_keys.phpt` | 1 |
| `Zend/tests/arrow_functions/` | 1 |
| `ext/standard/tests/array/` | 6 |
| Total | 34 |

The adjacent `array_chunk()` broad slice is not a blocker on this branch:
the focused runnable manifest selected 32 rows and passed 32/32.

## Blocked Rows

```text
Zend/tests/arg_unpack/basic.phpt
Zend/tests/arg_unpack/by_ref.phpt
Zend/tests/arg_unpack/by_ref_separation.phpt
Zend/tests/arg_unpack/dynamic.phpt
Zend/tests/arg_unpack/internal.phpt
Zend/tests/arg_unpack/invalid_type.phpt
Zend/tests/arg_unpack/many_args.phpt
Zend/tests/arg_unpack/method.phpt
Zend/tests/arg_unpack/new.phpt
Zend/tests/arg_unpack/non_integer_keys.phpt
Zend/tests/arg_unpack/positional_arg_after_unpack_error.phpt
Zend/tests/arg_unpack/string_keys.phpt
Zend/tests/arg_unpack/traversable_with_by_ref_parameters.phpt
Zend/tests/array_unpack/already_occupied.phpt
Zend/tests/array_unpack/basic.phpt
Zend/tests/array_unpack/classes.phpt
Zend/tests/array_unpack/gh19303.phpt
Zend/tests/array_unpack/gh9769.phpt
Zend/tests/array_unpack/in_destructuring.phpt
Zend/tests/array_unpack/in_destructuring_2.phpt
Zend/tests/array_unpack/non_integer_keys.phpt
Zend/tests/array_unpack/ref1.phpt
Zend/tests/array_unpack/string_keys.phpt
Zend/tests/array_unpack/undef_var.phpt
Zend/tests/array_unpack/unpack_invalid_type_compile_time.phpt
Zend/tests/array_unpack/unpack_string_keys_compile_time.phpt
Zend/tests/array_unpack_string_keys.phpt
Zend/tests/arrow_functions/008.phpt
ext/standard/tests/array/GHSA-h96m-rvf9-jgm2.phpt
ext/standard/tests/array/array_diff_max_elements.phpt
ext/standard/tests/array/array_diff_uassoc_basic.phpt
ext/standard/tests/array/array_find_types.phpt
ext/standard/tests/array/array_intersect_uassoc_basic.phpt
ext/standard/tests/array/array_push_empty.phpt
```

## Why This Is A Blocker

PTN currently models variadic parameter declarations, but not spread/unpacking
at expression use sites. Generic support needs:

- AST and IR nodes that distinguish ordinary arguments/elements from unpacked
  expressions.
- Parser diagnostics for unsupported spread positions such as destructuring
  assignment.
- Call lowering that expands arrays and traversables in evaluation order while
  preserving named/positional argument rules, duplicate-key diagnostics, and
  by-reference parameter binding.
- Internal-function dispatch paths that accept expanded argument vectors rather
  than only source-level fixed argument lists.
- Array literal lowering that merges spread arrays with PHP key behavior,
  string-key overwrite rules, integer reindexing, reference/COW handling, and
  invalid operand diagnostics.
- Traversable iteration support at the spread boundary, including object
  iterator metadata that PTN still treats as bounded.

This crosses both compiler and runtime boundaries, so treating the 34 rows as
runnable would turn a missing semantic layer into noisy parser/runtime failures.

## Verification

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only --out-dir .runtime/ptn-v1mu-before
tools/run-bounded-phpt.sh .runtime/ptn-v1mu/array-chunk-runnable.txt
```

Expected evidence from this slice:

- Broad 1k classify-only: 1,000 selected, 422 runnable, 578 excluded.
- Call-site/array unpacking blockers: 34 rows.
- Focused `array_chunk()` broad probe: 32 selected, 32 passed, 0 failed.
