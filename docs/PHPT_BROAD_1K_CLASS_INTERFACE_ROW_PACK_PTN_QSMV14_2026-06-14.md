# PHPT Broad 1k Class/Interface Row Pack: ptn-qsmv.14

Date: 2026-06-14

Corpus: `/home/claude/php-src-phpt`
Corpus revision: `8c63ec400ce8e07c57a8d9499317b96a8beafb8b`
Compiler branch commit: `2a83f3d535b4`
Pre-work comparison commit: `e6d9a2a86d8a`

## Scope

Implemented generic parser/lowering/runtime support for interface declarations,
class `implements`, interface type checks, abstract/interface method bodies,
`mixed` type hints, final-class declaration metadata and modifier diagnostics,
property-root array offset assignment/unset, and one-segment `ArrayAccess`
offset dispatch.

The class/interface declaration rows newly green from this slice are:

```text
Zend/tests/ArrayAccess/bug30346.phpt
Zend/tests/ArrayAccess/bug69955.phpt
Zend/tests/abstract-static.phpt
Zend/tests/abstract_method_optional_params.phpt
Zend/tests/access_modifiers/access_modifiers_001.phpt
Zend/tests/access_modifiers/access_modifiers_002.phpt
Zend/tests/access_modifiers/access_modifiers_003.phpt
Zend/tests/access_modifiers/access_modifiers_004.phpt
Zend/tests/access_modifiers/access_modifiers_005.phpt
Zend/tests/access_modifiers/access_modifiers_006.phpt
Zend/tests/access_modifiers/access_modifiers_007.phpt
Zend/tests/access_modifiers/access_modifiers_013.phpt
Zend/tests/bug32427.phpt
```

## Broad 1k Evidence

Final broad command:

```bash
PHPT_PROGRESS_DIR=.runtime/ptn-qsmv-broad-final-after \
  tools/run-bounded-phpt.sh \
  .runtime/ptn-qsmv-baseline-generated/20260614T124530Z/phpt-baseline-1000.txt
```

Final broad result:

```text
selected=1000 runnable=459 excluded=541 tests=459 passed=379 failed=80
zend:     selected=530 runnable=147 passed=84 failed=63
standard: selected=384 runnable=295 passed=287 failed=8
core:     selected=86  runnable=17  passed=8  failed=9
```

Pre-work comparison on the final pass-set candidates:

```bash
PHPT_PROGRESS_DIR=.runtime/ptn-qsmv-broad-before-current-pass \
  tools/run-bounded-phpt.sh \
  .runtime/ptn-qsmv-broad-after1/passed-20260614T132014Z.txt

PHPT_PROGRESS_DIR=.runtime/ptn-qsmv-broad-before-final-candidates \
  tools/run-bounded-phpt.sh \
  .runtime/ptn-qsmv-broad-final-after/candidate-newly-passing-vs-old-current-pass.txt
```

Measured comparison:

```text
old compiler on first final-pass subset: selected=322 runnable=319 passed=319 excluded=3
old compiler on remaining candidates:   selected=61  runnable=39  passed=0   failed=39 excluded=22
```

One row that old passed in the first comparison is not final-green after the
rebase: `ext/standard/tests/array/array_diff_assoc_variation9.phpt`.

Exact final pass-set delta versus the pre-work compiler is 61 rows:

```text
Zend/tests/ArrayAccess/bug30346.phpt
Zend/tests/ArrayAccess/bug69955.phpt
Zend/tests/abstract-static.phpt
Zend/tests/abstract_method_optional_params.phpt
Zend/tests/access_modifiers/access_modifiers_001.phpt
Zend/tests/access_modifiers/access_modifiers_002.phpt
Zend/tests/access_modifiers/access_modifiers_003.phpt
Zend/tests/access_modifiers/access_modifiers_004.phpt
Zend/tests/access_modifiers/access_modifiers_005.phpt
Zend/tests/access_modifiers/access_modifiers_006.phpt
Zend/tests/access_modifiers/access_modifiers_007.phpt
Zend/tests/access_modifiers/access_modifiers_013.phpt
Zend/tests/assert/expect_001.phpt
Zend/tests/assert/expect_003.phpt
Zend/tests/assert/expect_004.phpt
Zend/tests/assert/expect_005.phpt
Zend/tests/assert/expect_006.phpt
Zend/tests/assert/expect_008.phpt
Zend/tests/assert/expect_012.phpt
Zend/tests/assert/expect_013.phpt
Zend/tests/assert/expect_014.phpt
Zend/tests/assert/expect_016.phpt
Zend/tests/bug32427.phpt
ext/standard/tests/array/array_combine_variation3.phpt
ext/standard/tests/array/array_count_values2.phpt
ext/standard/tests/array/array_diff_variation9.phpt
ext/standard/tests/array/array_filter_variation10.phpt
ext/standard/tests/array/array_flip.phpt
ext/standard/tests/array/array_flip_variation3.phpt
ext/standard/tests/array/array_intersect_variation3.phpt
ext/standard/tests/array/array_intersect_variation4.phpt
ext/standard/tests/array/array_key_exists_variation3.phpt
ext/standard/tests/array/array_map_error.phpt
ext/standard/tests/array/array_map_object2.phpt
ext/standard/tests/array/array_map_variation10.phpt
ext/standard/tests/array/array_map_variation12.phpt
ext/standard/tests/array/array_map_variation7.phpt
ext/standard/tests/array/array_map_variation9.phpt
ext/standard/tests/array/array_merge.phpt
ext/standard/tests/array/array_merge_recursive_variation3.phpt
ext/standard/tests/array/array_merge_recursive_variation7.phpt
ext/standard/tests/array/array_next_error1.phpt
ext/standard/tests/array/array_next_error2.phpt
ext/standard/tests/array/array_pad_variation6.phpt
ext/standard/tests/array/array_push.phpt
ext/standard/tests/array/array_push_error2.phpt
ext/standard/tests/array/array_push_variation3.phpt
ext/standard/tests/array/array_rand_variation6.phpt
ext/standard/tests/array/array_reduce_variation1.phpt
ext/standard/tests/array/array_replace.phpt
ext/standard/tests/array/array_replace_merge_recursive_ref.phpt
ext/standard/tests/array/array_search_variation3.phpt
ext/standard/tests/array/array_shift_variation5.phpt
ext/standard/tests/array/array_shift_variation8.phpt
ext/standard/tests/array/array_udiff_assoc_variation.phpt
ext/standard/tests/array/array_udiff_assoc_variation5.phpt
ext/standard/tests/array/array_udiff_uassoc_variation6.phpt
ext/standard/tests/array/array_udiff_variation5.phpt
ext/standard/tests/array/array_uintersect_assoc_basic2.phpt
ext/standard/tests/array/array_uintersect_assoc_variation5.phpt
ext/standard/tests/array/array_uintersect_uassoc_variation6.phpt
```

## Focused Row Packs

Interface/ArrayAccess raw command:

```bash
PTN_PHPT_CLASSIFY=0 \
PHPT_PROGRESS_DIR=.runtime/ptn-qsmv-interface-after-rebase \
  tools/run-bounded-phpt.sh tools/phpt-interface-current-ptn-l0h9-manifest.txt
```

Result: 38 selected, 38 runnable, 6 passed, 32 failed.

Passing focused rows:

```text
Zend/tests/ArrayAccess/bug30346.phpt
Zend/tests/ArrayAccess/bug33710.phpt
Zend/tests/ArrayAccess/bug39297.phpt
Zend/tests/ArrayAccess/bug69955.phpt
Zend/tests/attributes/override/001.phpt
Zend/tests/bug32427.phpt
```

Class-contract classified command:

```bash
PHPT_PROGRESS_DIR=.runtime/ptn-qsmv-class-contract-classified-after-rebase \
  tools/run-bounded-phpt.sh \
  .runtime/ptn-qsmv-broad-after1/excluded-20260614T132014Z/unsupported-class-contract-metadata.txt
```

Result: 9 selected, 9 runnable, 9 passed, 0 failed.

Representative remaining failures:

```text
Zend/tests/ArrayAccess/ArrayAccess_indirect_append.phpt
Zend/tests/ArrayAccess/bug68896.phpt
Zend/tests/ArrayAccess/bug71731.phpt
Zend/tests/anon/bug77652.phpt
ext/standard/tests/array/array_fill_object.phpt
```

## Rust Verification

```bash
cargo fmt --check
cargo test --test compile_native parser_accepts_interface_declarations_and_implements_metadata
cargo test --test compile_native parser_accepts_property_root_array_assignment_expressions
cargo test --test compile_native compile_property_array_dimension_assignments_to_native_binary
cargo test --test compile_native parser_models_final_class_and_modifier_diagnostics
cargo test --test phpt_classifier
```

All passed on the rebased branch.
