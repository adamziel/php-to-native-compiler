# PHPT Asymmetric Visibility Blockers

Date: 2026-06-13
Slice: `ptn-c7iw`
Corpus revision: `8c63ec400ce8e07c57a8d9499317b96a8beafb8b`

This slice adds generic PHPT preclassification for PHP 8.4 asymmetric property
visibility modifiers such as `private(set)`, `protected(set)`, and related
`get`/`set` access modifiers.

These rows require property visibility metadata that PTN does not yet model:
scope-aware set/get access, typed property integration, constructor-promoted
properties, static properties, override checks, virtual/property-hook
interactions, and the corresponding fatal diagnostics. The broad rows were
previously runnable parser/runtime noise, not credible implementation targets
for this slice.

## Broad 1k Evidence

Same manifest: `.runtime/phpt-progress/manifest-20260613T171349Z.txt`

Before this branch, using the `origin/master` classifier:

- Selected: 1,000
- Runnable: 614
- Excluded: 386
- Unsupported-language exclusions: 271
- Asymmetric visibility exclusions: 0

After this branch:

- Summary: `.runtime/phpt-progress/summary-20260613T171349Z.txt`
- Selected: 1,000
- Runnable: 576
- Excluded: 424
- Unsupported-language exclusions: 309
- Asymmetric visibility exclusions: 38

Net movement: 38 broad 1k rows newly classified as unsupported language
blockers.

Unsupported-language reason counts after the change:

| Rows | Reason |
| ---: | --- |
| 153 | PHP attribute syntax and reflection metadata |
| 38 | Asymmetric property visibility metadata |
| 35 | Call-site or array unpacking |
| 25 | Trait declarations |
| 24 | Interface declarations |
| 18 | Anonymous class syntax |
| 16 | Interface implementation checks |

## Newly Classified Rows

- `Zend/tests/asymmetric_visibility/__set.phpt`
- `Zend/tests/asymmetric_visibility/__unset.phpt`
- `Zend/tests/asymmetric_visibility/bug001.phpt`
- `Zend/tests/asymmetric_visibility/bug002.phpt`
- `Zend/tests/asymmetric_visibility/bug003.phpt`
- `Zend/tests/asymmetric_visibility/bug004.phpt`
- `Zend/tests/asymmetric_visibility/cpp_no_type.phpt`
- `Zend/tests/asymmetric_visibility/cpp_private.phpt`
- `Zend/tests/asymmetric_visibility/cpp_protected.phpt`
- `Zend/tests/asymmetric_visibility/cpp_wider_set_scope.phpt`
- `Zend/tests/asymmetric_visibility/decrease_scope_private_private.phpt`
- `Zend/tests/asymmetric_visibility/decrease_scope_private_protected.phpt`
- `Zend/tests/asymmetric_visibility/decrease_scope_protected_protected.phpt`
- `Zend/tests/asymmetric_visibility/dim_add.phpt`
- `Zend/tests/asymmetric_visibility/duplicate_modifier.phpt`
- `Zend/tests/asymmetric_visibility/duplicate_modifier_2.phpt`
- `Zend/tests/asymmetric_visibility/gh19044.phpt`
- `Zend/tests/asymmetric_visibility/nested_write.phpt`
- `Zend/tests/asymmetric_visibility/no_type.phpt`
- `Zend/tests/asymmetric_visibility/object_reference.phpt`
- `Zend/tests/asymmetric_visibility/override_private_public.phpt`
- `Zend/tests/asymmetric_visibility/override_protected_private.phpt`
- `Zend/tests/asymmetric_visibility/override_protected_public.phpt`
- `Zend/tests/asymmetric_visibility/override_public_private.phpt`
- `Zend/tests/asymmetric_visibility/override_public_protected.phpt`
- `Zend/tests/asymmetric_visibility/private.phpt`
- `Zend/tests/asymmetric_visibility/protected.phpt`
- `Zend/tests/asymmetric_visibility/readonly.phpt`
- `Zend/tests/asymmetric_visibility/reference.phpt`
- `Zend/tests/asymmetric_visibility/reference_2.phpt`
- `Zend/tests/asymmetric_visibility/scope_rebinding.phpt`
- `Zend/tests/asymmetric_visibility/static_props.phpt`
- `Zend/tests/asymmetric_visibility/unset.phpt`
- `Zend/tests/asymmetric_visibility/unshared_rw_cache_slot.phpt`
- `Zend/tests/asymmetric_visibility/variation.phpt`
- `Zend/tests/asymmetric_visibility/variation_nested.phpt`
- `Zend/tests/asymmetric_visibility/virtual_get_only.phpt`
- `Zend/tests/asymmetric_visibility/virtual_set_only.phpt`

## Verification

- `cargo test --test phpt_classifier`
- `cargo fmt --check`
- `bash -n tools/phpt-classifier.sh tools/run-bounded-phpt.sh tools/run-phpt-baseline.sh`
- `tools/run-phpt-baseline.sh --tier 1000 --classify-only`
