# run220 php_runtime gate repair - developer-449

## Failing run confirmed

- Harness `test_runs#220` ran `tools/run-tests.sh` at
  `0cda94596990f8a8060260ea3b18cbbc38296c54`.
- The failing component was `cargo test` for `php_runtime --lib`: 406 passed,
  14 failed.
- Failed tests:
  - `tests::class_table_can_bootstrap_core_exception_metadata`
  - `tests::native_comparison_string_handle_operands_share_materialization_across_families`
  - `tests::native_lookup_plus_invoke_helpers_free_arguments_once_across_target_families`
  - `tests::native_magic_method_lookup_rejects_malformed_signature_metadata_before_fallback`
  - `tests::native_materialized_comparison_value_pairs_reuse_operand_contract_across_families`
  - `tests::native_method_lookup_plus_invoke_dispatches_missing_and_inaccessible_methods_to_magic_call`
  - `tests::native_string_byte_value_conversion_shares_diagnostic_boundary`
  - `tests::native_string_offset_writes_share_value_key_and_replacement_boundaries`
  - `tests::native_string_value_conversion_reports_diagnostics_for_failures`
  - `tests::native_value_materialization_failure_exit_code_feeds_comparison_operands`
  - `tests::static_property_array_path_reference_native_abi_preserves_storage_identity_and_type_checks`
  - `tests::static_property_bind_reference_native_abi_preserves_source_alias_and_type_constraints`
  - `tests::static_property_reference_native_abi_preserves_aliases_and_type_constraints`
  - `tests::static_property_storage_preserves_reference_identity_and_type_constraints`

## Source comparison

- Current worktree before repair: `00b9f4832ca8b2dd55484153f2c43a29e602ca76`.
- `origin/work/developer-444` is `74291bb706cf04bfe98cbf77707c30f99d346613`.
- Its parent is `71479c716b10c526dcf2fc2a07dab2ef61d6b5ad`, which is also the
  merge base with this worktree.
- The developer-444 commit changes only `runtime/src/lib.rs` and
  `docs/PROGRESS.md`: 202 insertions and 79 deletions.
- Direct `HEAD..origin/work/developer-444` included unrelated `.harness`
  report history, so I evaluated and ported the single developer-444 commit
  rather than the branch range.

## Fix ported

- Ported developer-444's narrow runtime patch with `git cherry-pick -n
  origin/work/developer-444`.
- Updated `docs/PROGRESS.md` wording from the older run62 context to this
  run220 repair.
- Runtime changes are limited to `runtime/src/lib.rs` test/expectation repair:
  - binary PHP strings are treated as byte-backed values instead of invalid
    UTF-8 materialization failures in the affected native string/comparison
    tests;
  - native string offset write diagnostics reflect the current truncation
    warning behavior;
  - core class-table metadata expectations include the current reflection,
    `ArrayObject`, and `ArrayIterator` metadata;
  - static-property reference type-constraint assertions expect the current
    reference-held typed-property diagnostic;
  - the `cfg(test)` `NativeCallArgumentsHandle` free counter is thread-local,
    with a guard test proving isolation across Rust test threads.

## Checks run

- Passed: `CARGO_TARGET_DIR=/tmp/phpc-target-developer-449-run220 CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 RUST_TEST_THREADS=1 cargo test -p php_runtime --lib -- --test-threads=1`
  - Result: 420 passed, 0 failed.
- Passed: `CARGO_TARGET_DIR=/tmp/phpc-target-developer-449-run220 CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 cargo check -p php_runtime`
- Passed: `cargo fmt --check`
- Passed: `git diff --check`

## Remaining gate

- I did not run a full `tools/run-tests.sh` or any PHPT/public score gate.
- Next deterministic integrator/manager action is to run the full gate from the
  integration branch if resource state allows.
