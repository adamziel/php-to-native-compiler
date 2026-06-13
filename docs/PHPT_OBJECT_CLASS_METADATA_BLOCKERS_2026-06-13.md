# PHPT Object/Class Metadata Blockers: 2026-06-13

Issue: `ptn-n1q2`

Broad baseline source:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only
```

The rebased generated 1k broad manifest was
`.runtime/phpt-baseline/20260613T165950Z/phpt-baseline-1000.txt`, using
php-src revision `8c63ec400ce8e07c57a8d9499317b96a8beafb8b`.

This branch is based on `origin/master` `4d1912ed`, where `ptn-hu7e`/`ptn-cm8x`
already classify PHP attribute syntax as `unsupported-language`. This note
tracks the remaining object/class metadata rows classified by this slice.

This slice adds the generic `unsupported-class-metadata` classification for
language/runtime surfaces PTN does not yet preserve in its semantic model:

- Abstract/final class and method contract metadata.
- Non-public method visibility dispatch and diagnostics.
- Unsupported magic method dispatch/reflection metadata.
- Runtime class autoload symbol-table mutation.
- Enums and enum case metadata.
- Constructor property promotion.
- Typed, readonly, asymmetric-visibility, and typed static property metadata.
- Non-public and typed class constant metadata.

Evidence:

```sh
bash -n tools/phpt-classifier.sh
cargo fmt --check
cargo test --test phpt_classifier
tools/run-phpt-baseline.sh --tier 1000 --classify-only
```

The current rebased classify-only run selected 1,000 broad PHPT rows, kept 476
runnable, and excluded 524. Attribute rows are counted in the upstream
`unsupported-language` bucket; this slice accounts for 138
`unsupported-class-metadata` rows:

```text
classification.unsupported-language: rows=271
classification.unsupported-ini: rows=73
classification.unsupported-extension: rows=20
classification.unsupported-class-metadata: rows=138
classification.harness-cleanup: rows=4
classification.sapi-behavior: rows=13
classification.process-boundary: rows=3
classification.external-service: rows=1
classification.environment-assumption: rows=1
```

Representative newly classified broad rows:

```text
Zend/tests/abstract-static.phpt
Zend/tests/abstract_method_optional_params.phpt
Zend/tests/access_modifiers/access_modifiers_001.phpt
Zend/tests/access_modifiers/access_modifiers_002.phpt
Zend/tests/access_modifiers/access_modifiers_003.phpt
Zend/tests/access_modifiers/access_modifiers_004.phpt
Zend/tests/access_modifiers/access_modifiers_005.phpt
Zend/tests/access_modifiers/access_modifiers_007.phpt
Zend/tests/access_modifiers/access_modifiers_008.phpt
Zend/tests/access_modifiers/access_modifiers_009.phpt
Zend/tests/access_modifiers/access_modifiers_010.phpt
Zend/tests/access_modifiers/access_modifiers_011.phpt
Zend/tests/access_modifiers/access_modifiers_012.phpt
Zend/tests/access_modifiers/access_modifiers_013.phpt
Zend/tests/assign_obj_to_ref_inference.phpt
Zend/tests/assign_typed_ref_result.phpt
Zend/tests/asymmetric_visibility/__set.phpt
Zend/tests/asymmetric_visibility/__unset.phpt
Zend/tests/asymmetric_visibility/bug001.phpt
Zend/tests/asymmetric_visibility/bug002.phpt
Zend/tests/asymmetric_visibility/bug003.phpt
Zend/tests/asymmetric_visibility/bug004.phpt
Zend/tests/asymmetric_visibility/cpp_no_type.phpt
Zend/tests/asymmetric_visibility/cpp_private.phpt
Zend/tests/asymmetric_visibility/cpp_protected.phpt
Zend/tests/asymmetric_visibility/cpp_wider_set_scope.phpt
Zend/tests/asymmetric_visibility/decrease_scope_private_private.phpt
Zend/tests/asymmetric_visibility/decrease_scope_private_protected.phpt
Zend/tests/asymmetric_visibility/decrease_scope_protected_protected.phpt
Zend/tests/asymmetric_visibility/dim_add.phpt
Zend/tests/asymmetric_visibility/duplicate_modifier.phpt
Zend/tests/asymmetric_visibility/duplicate_modifier_2.phpt
Zend/tests/asymmetric_visibility/gh19044.phpt
Zend/tests/asymmetric_visibility/nested_write.phpt
Zend/tests/asymmetric_visibility/no_type.phpt
Zend/tests/asymmetric_visibility/object_reference.phpt
Zend/tests/asymmetric_visibility/override_private_public.phpt
Zend/tests/asymmetric_visibility/override_protected_private.phpt
Zend/tests/asymmetric_visibility/override_protected_public.phpt
Zend/tests/asymmetric_visibility/override_public_private.phpt
Zend/tests/asymmetric_visibility/override_public_protected.phpt
Zend/tests/asymmetric_visibility/private.phpt
Zend/tests/asymmetric_visibility/protected.phpt
Zend/tests/asymmetric_visibility/readonly.phpt
Zend/tests/asymmetric_visibility/reference.phpt
Zend/tests/asymmetric_visibility/reference_2.phpt
Zend/tests/asymmetric_visibility/scope_rebinding.phpt
Zend/tests/asymmetric_visibility/static_props.phpt
Zend/tests/asymmetric_visibility/unset.phpt
Zend/tests/asymmetric_visibility/unshared_rw_cache_slot.phpt
Zend/tests/autoload/bug26697.phpt
Zend/tests/autoload/bug33116.phpt
Zend/tests/autoload/bug37138.phpt
Zend/tests/autoload/bug39003.phpt
```

Next architecture step:

1. Add AST/IR storage for typed properties/constants, readonly and asymmetric
   visibility flags, abstract/final flags, enum cases, promoted constructor
   properties, and attribute/reflection metadata.
2. Extend class table construction to validate inheritance contracts,
   visibility compatibility, override/final rules, and non-public method
   dispatch.
3. Add runtime/reflection metadata APIs that expose the stored declaration
   metadata generically.
4. Re-open these PHPT rows by removing the corresponding classifier branches
   as each semantic surface lands.
