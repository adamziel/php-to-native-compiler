# Run221 runtime duplicate comparison audit - developer-453

## Scope

- Worklane: `work_lanes#160`, "Run221 runtime failure duplicate comparison audit".
- Coordinator reconciliation artifact requested:
  `.harness/reports/run221-runtime-duplicate-comparison-audit-dev453.md`.
- Read-only audit only. I did not edit compiler/runtime source and did not run
  the full suite or PHPT/public-score gates.
- The earlier run222 command-selection evaluator poke was ignored per the
  Coordinator reconciliation that made `work_lanes#160` authoritative.

## Conclusion

Run221 is not a distinct runtime repair lane. It is a duplicate recurrence of
the run220 `php_runtime --lib` failure while the already-proven runtime repair
was still absent from current `HEAD`.

Classification:

- Primary cause: missing source integration of the lane156 runtime fix.
- Duplicate lane: yes for the runtime portion of lane157/run221.
- Parallelism/state leakage: the existing lane156/developer-444 patch includes
  a thread-local `NativeCallArgumentsHandle` test-counter repair, so the
  differing run220/run221 failure names are explained inside the already
  proven runtime fix rather than requiring a new runtime lane.

Recommended action:

- Integrate `origin/work/developer-449` or its mergecheck commit
  `a57ec70f` from a clean integration checkout, then rerun the focused
  `php_runtime --lib` smoke.
- Close or fold lane157's runtime repair work as duplicate once developer-450
  reports the focused proof.
- If a broader `tools/run-tests.sh` gate is required after the runtime merge,
  track the two remaining `phpc --lib` codegen failures as a separate precise
  lane:
  `codegen::tests::c_assembly_non_local_assignment_families_share_assignment_owner_boundary`
  and
  `codegen::tests::generated_c_string_comparison_safety_uses_shared_pair_classifier`.

## Run comparison

Observed via harness SQLite:

```sql
SELECT id, command, commit_sha, status, summary_json,
       substr(full_log, instr(full_log, 'test result:'), 120) AS result_line
FROM test_runs
WHERE id IN (220,221,222)
ORDER BY id;
```

- `test_runs#220`: `tools/run-tests.sh` at
  `0cda94596990f8a8060260ea3b18cbbc38296c54`, failed inside
  `php_runtime --lib`: `406 passed; 14 failed`.
- `test_runs#221`: `tools/run-tests.sh` at
  `58e62b828b683a6abcb08bbbad6641c1d3af6736`, failed inside
  `php_runtime --lib`: `404 passed; 16 failed`.
- `test_runs#222`: `python -m unittest discover -s tests -v`, `Ran 0 tests`.
  This is the separate command-selection recurrence and is out of lane160
  scope.

Failure names were extracted with this read-only recursive SQLite query:

```sql
WITH RECURSIVE lines(run_id, line, rest) AS (
  SELECT id, '', full_log || char(10) FROM test_runs WHERE id IN (220,221)
  UNION ALL
  SELECT run_id,
         substr(rest, 1, instr(rest, char(10))-1),
         substr(rest, instr(rest, char(10))+1)
  FROM lines
  WHERE rest != '' AND instr(rest, char(10)) > 0
),
fails AS (
  SELECT run_id, replace(replace(line, '---- ', ''), ' stdout ----', '') AS test_name
  FROM lines
  WHERE line LIKE '---- tests::% stdout ----'
)
SELECT 'only_run220' AS bucket, test_name FROM fails
WHERE run_id=220 AND test_name NOT IN (SELECT test_name FROM fails WHERE run_id=221)
UNION ALL
SELECT 'only_run221' AS bucket, test_name FROM fails
WHERE run_id=221 AND test_name NOT IN (SELECT test_name FROM fails WHERE run_id=220)
UNION ALL
SELECT 'common' AS bucket, test_name FROM fails
WHERE run_id=221 AND test_name IN (SELECT test_name FROM fails WHERE run_id=220)
ORDER BY bucket, test_name;
```

Common failing runtime tests, 13:

- `tests::class_table_can_bootstrap_core_exception_metadata`
- `tests::native_comparison_string_handle_operands_share_materialization_across_families`
- `tests::native_lookup_plus_invoke_helpers_free_arguments_once_across_target_families`
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

Run-specific deltas:

- Only run220:
  `tests::native_magic_method_lookup_rejects_malformed_signature_metadata_before_fallback`
- Only run221:
  `tests::native_constructor_allocation_invoke_carrier_cleans_up_failure_paths`
- Only run221:
  `tests::native_constructor_allocation_invoke_reference_carrier_owns_receiver_cell`
- Only run221:
  `tests::native_static_method_lookup_plus_invoke_dispatches_missing_and_inaccessible_methods_to_magic_call_static`

The shared failing cluster is dominant, and the run-specific deltas are in the
same native-call/argument-carrier diagnostic family covered by the lane156
runtime repair.

## Existing repair evidence

developer-444 produced the original runtime repair:

- Commit: `74291bb7` on `origin/work/developer-444`.
- Files: `runtime/src/lib.rs`, `docs/PROGRESS.md`.
- Event evidence says:
  `CARGO_TARGET_DIR=/tmp/phpc-target-dev444 CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 RUST_TEST_THREADS=1 cargo test -q -p php_runtime --lib -- --test-threads=1`
  passed `420/420`, followed by `cargo check -p php_runtime`,
  `cargo fmt --check`, and `git diff --check`.

developer-449/lane156 ported and documented the repair:

```sh
git show --stat --oneline --decorate --no-renames origin/work/developer-449 --
git show origin/work/developer-449:.harness/reports/run220-runtime-gate-repair-developer-449.md
```

Evidence from that report:

- Commit: `83c698fe` on `origin/work/developer-449`.
- Runtime patch: `runtime/src/lib.rs`, 249 touched lines.
- Focused proof:
  `CARGO_TARGET_DIR=/tmp/phpc-target-developer-449-run220 CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 RUST_TEST_THREADS=1 cargo test -p php_runtime --lib -- --test-threads=1`
  passed `420 passed, 0 failed`.
- Also passed: `cargo check -p php_runtime`, `cargo fmt --check`,
  `git diff --check`.

Integrator-40 independently mergechecked lane156:

- Temp branch: `integration/integrator-40-lane156-20260605T1950`.
- Temp merge commit: `a57ec70f`.
- Report status: `mergecheck_passed_blocked_by_dirty_root`.
- Test result: `php_runtime --lib 420 passed / 0 failed`,
  `cargo check -p php_runtime` passed, `cargo fmt --check` passed,
  `git diff --check` passed.
- Blocker: master checkout dirty and overlapping `runtime/src/lib.rs` plus
  `docs/PROGRESS.md`.

## Current integration state

Commands run from `work/developer-453`:

```sh
if git merge-base --is-ancestor origin/work/developer-444 HEAD; then
  echo 'origin/work/developer-444 is integrated into HEAD'
else
  echo 'origin/work/developer-444 is NOT integrated into HEAD'
fi

if git merge-base --is-ancestor origin/work/developer-449 HEAD; then
  echo 'origin/work/developer-449 is integrated into HEAD'
else
  echo 'origin/work/developer-449 is NOT integrated into HEAD'
fi

git diff --stat HEAD..origin/work/developer-449 -- runtime/src/lib.rs docs/PROGRESS.md
```

Observed:

- `origin/work/developer-444 is NOT integrated into HEAD`.
- `origin/work/developer-449 is NOT integrated into HEAD`.
- `HEAD..origin/work/developer-449` still changes
  `runtime/src/lib.rs` and `docs/PROGRESS.md`.

Root checkout state also confirms integrator-40's blocker:

```sh
git -C /home/claude/php-to-native-compiler status --short --branch
git -C /home/claude/php-to-native-compiler diff --name-status -- runtime/src/lib.rs docs/PROGRESS.md
```

Observed:

- Root `master` has broad dirty state.
- `runtime/src/lib.rs` and `docs/PROGRESS.md` are dirty in the root checkout,
  exactly overlapping lane156 integration files.

## developer-450 / lane157 state

developer-450 is the lane157 owner. I did not edit or interrupt that worktree.
Read-only inspection showed the runtime file is already aligned with
`origin/work/developer-444`; only `docs/PROGRESS.md` differs:

```sh
git -C /home/claude/php-to-native-compiler/.harness/worktrees/developer-450 \
  diff --name-status origin/work/developer-444 -- runtime/src/lib.rs docs/PROGRESS.md
```

Observed:

- `M docs/PROGRESS.md`
- No `runtime/src/lib.rs` delta versus `origin/work/developer-444`.

developer-450 also recorded this scheduler-visible event:

- After porting the runtime repair, `cargo test -q` passes `php_runtime`.
- The remaining same-command failures are two `phpc --lib` codegen tests:
  `codegen::tests::c_assembly_non_local_assignment_families_share_assignment_owner_boundary`
  and
  `codegen::tests::generated_c_string_comparison_safety_uses_shared_pair_classifier`.

That matches integrator-40's note that those two codegen failures are outside
lane156 and pre-existing for a full `tools/run-tests.sh` gate.

## Post-fix smoke

Expected smoke after integrating lane156 from a clean checkout:

```sh
CARGO_TARGET_DIR=/tmp/phpc-target-lane156-postfix \
CARGO_BUILD_JOBS=1 \
CARGO_INCREMENTAL=0 \
RUST_TEST_THREADS=1 \
cargo test -p php_runtime --lib -- --test-threads=1

CARGO_TARGET_DIR=/tmp/phpc-target-lane156-postfix \
CARGO_BUILD_JOBS=1 \
CARGO_INCREMENTAL=0 \
cargo check -p php_runtime

cargo fmt --check
git diff --check
```

Expected result:

- `php_runtime --lib`: `420 passed / 0 failed`.
- The run221 `php_runtime` failure cluster should not recur.
- Any remaining broad `tools/run-tests.sh` failure should be assigned to the
  separate `phpc --lib` codegen failures, not to another runtime duplicate
  lane.
