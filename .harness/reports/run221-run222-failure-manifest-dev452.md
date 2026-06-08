# Run221/Run222 Deterministic Failure Manifest

Developer: developer-26
Worklane: 161
Scope: report-only manifest. No compiler, runtime, product test, docs, README,
Cargo, or harness zipapp source edits.

## Verdict

- `run221` was a real `tools/run-tests.sh` failure. The failing component was
  `php_runtime --lib` at commit
  `58e62b828b683a6abcb08bbbad6641c1d3af6736`: `404 passed; 16 failed`.
- `run222` was not a compiler/runtime failure. It was the harness
  command-selection recurrence where the test loop selected
  `python -m unittest discover -s tests -v` and ran zero tests.
- `lane157` duplicated the runtime part of `lane156` after the lane156 source
  fix had proof but was not integrated into the dirty root checkout. Its extra
  codegen work was separate from the runtime duplicate, and the old-base branch
  later became stale against current `origin/master`.
- `lane158` was separate control-plane work for the live harness zipapp. It is
  not a duplicate of lane156. It repaired command selection and liveness
  predicates in `/home/claude/php-to-native-compiler/harness` and verified the
  focused `.harness` unittest path.

## Current SQLite Boundary

The live harness database available to this lane no longer contains
`test_runs` ids 220, 221, or 222. The table was reinitialized later and now
starts at id 634. The run221/run222 full-log details below therefore come from
the already integrated lane159/lane160 reports that captured those rows while
they were still present, cross-checked against current `worklanes`,
`agent_reports`, and `events` rows.

Queries used in this lane:

```sql
PRAGMA table_xinfo(test_runs);

SELECT count(*) AS n, min(id) AS min_id, max(id) AS max_id
FROM test_runs;

SELECT id, started_at, command, status, summary_json, length(full_log) AS full_log_len
FROM test_runs
WHERE id BETWEEN 215 AND 230
ORDER BY id;
```

Observed current result:

- `test_runs`: `min_id=634`, `max_id=9496`.
- `WHERE id BETWEEN 215 AND 230`: no rows.

## Run221 Failing Tests

The failure names below are copied from the integrated report
`.harness/reports/run221-runtime-duplicate-comparison-audit-dev453.md`, which
records the exact recursive SQLite extraction query used against `test_runs`
full logs for ids 220 and 221.

Run221 failed these 16 `php_runtime --lib` tests:

- `tests::class_table_can_bootstrap_core_exception_metadata`
- `tests::native_comparison_string_handle_operands_share_materialization_across_families`
- `tests::native_constructor_allocation_invoke_carrier_cleans_up_failure_paths`
- `tests::native_constructor_allocation_invoke_reference_carrier_owns_receiver_cell`
- `tests::native_lookup_plus_invoke_helpers_free_arguments_once_across_target_families`
- `tests::native_materialized_comparison_value_pairs_reuse_operand_contract_across_families`
- `tests::native_method_lookup_plus_invoke_dispatches_missing_and_inaccessible_methods_to_magic_call`
- `tests::native_static_method_lookup_plus_invoke_dispatches_missing_and_inaccessible_methods_to_magic_call_static`
- `tests::native_string_byte_value_conversion_shares_diagnostic_boundary`
- `tests::native_string_offset_writes_share_value_key_and_replacement_boundaries`
- `tests::native_string_value_conversion_reports_diagnostics_for_failures`
- `tests::native_value_materialization_failure_exit_code_feeds_comparison_operands`
- `tests::static_property_array_path_reference_native_abi_preserves_storage_identity_and_type_checks`
- `tests::static_property_bind_reference_native_abi_preserves_source_alias_and_type_constraints`
- `tests::static_property_reference_native_abi_preserves_aliases_and_type_constraints`
- `tests::static_property_storage_preserves_reference_identity_and_type_constraints`

Historical SQL recorded by lane160 for the run result line:

```sql
SELECT id, command, commit_sha, status, summary_json,
       substr(full_log, instr(full_log, 'test result:'), 120) AS result_line
FROM test_runs
WHERE id IN (220,221,222)
ORDER BY id;
```

Historical SQL recorded by lane160 for failing-test extraction:

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

Classification from that query:

- 13 tests were common to run220 and run221.
- Run221-only deltas were the two constructor allocation invoke carrier tests
  and `native_static_method_lookup_plus_invoke_dispatches_missing_and_inaccessible_methods_to_magic_call_static`.
- The deltas stay in the same native-call/argument-carrier diagnostic family
  as the lane156 runtime repair.

## Run222 Classification

Run222 selected:

```text
python -m unittest discover -s tests -v
```

Recorded output:

```text
Ran 0 tests in 0.000s
NO TESTS RAN
```

The authoritative integrated audit is
`.harness/reports/run222-command-selection-patch-point-audit-dev452.md`.
It classifies `test_runs#222` as a recurrence of the harness
command-selection bug caused by `discover_test_command()` preferring generic
Python unittest discovery from the mere presence of `tests/` before the
project gate `tools/run-tests.sh`.

Lane158 later repaired the deployed root zipapp. Root-only report
`/home/claude/php-to-native-compiler/.harness/reports/run222-command-selection-live-fix-dev451.md`
records:

- `discover_test_command(ROOT) == ['tools/run-tests.sh']`.
- Focused `.harness` unittest: `Ran 8 tests ... OK`.
- Product worktree proof for `tests docs compiler runtime README.md Cargo.toml Cargo.lock`
  was empty.

## Lane Duplication Assessment

Current worklane query:

```sql
SELECT id, title, status, stage, branch_name, notes
FROM worklanes
WHERE id IN (156,157,158,159,160,161)
ORDER BY id;
```

Current agent report query:

```sql
SELECT id, created_at, agent_name, worklane_id, role, status, stage, card_id,
       substr(report_json, 1, 1600) AS report_prefix
FROM agent_reports
WHERE worklane_id IN (156,157,158,159,160)
ORDER BY id;
```

Findings:

- `lane156` / `developer-449`: canonical runtime repair for run220/run221
  cluster. Commit `83c698fe` changed `runtime/src/lib.rs`,
  `docs/PROGRESS.md`, and the run220 report. Focused proof:
  `php_runtime --lib 420/420`, `cargo check -p php_runtime`, `cargo fmt --check`,
  and `git diff --check`. Integrator-40 mergechecked it at temp merge commit
  `a57ec70f`, but direct master integration was blocked by dirty overlapping
  `runtime/src/lib.rs` and `docs/PROGRESS.md`.
- `lane157` / `developer-450`: duplicate runtime port plus two same-gate phpc
  codegen assertion refreshes. Commit
  `af91813e418e576ca520eaa09c6a1c63ff40a77e` passed `php_runtime --lib 420/420`
  and the two focused phpc codegen tests under integrator-41, but integrator-42
  later requeued it as stale after `git merge-tree origin/master af91813e...`
  reported conflicts in `compiler/src/codegen.rs`, `docs/PROGRESS.md`, and
  `runtime/src/lib.rs`. Do not merge the old-base lane157 head directly.
- `lane158` / `developer-451`: separate harness/control-plane fix for run222,
  integrated. It patched the deployed root zipapp so command selection prefers
  `tools/run-tests.sh`, active-agent listing uses non-terminal `ended_at IS NULL`
  semantics, and liveness ignores ended rows. This should not be folded into
  the lane156 runtime repair.
- `lane159` and `lane160`: integrated report-only audits. They add evidence
  only and do not touch product source.
- `lane161`: this manifest lane. It should remain report-only.

## Commands Used

```sh
sed -n '1,260p' AGENTS.md
sed -n '1,260p' docs/PROGRESS.md
sed -n '1,260p' docs/ARCHITECTURE.md
sed -n '1,260p' docs/SUPPORT.md
sed -n '1,260p' README.md
sed -n '1,260p' docs/LOOP_MEMORY.md
sed -n '1,260p' /home/claude/php-to-native-compiler/DEVELOPMENT.md
rg -n "run ?22[012]|test_runs#22[12]|php_runtime|command-selection|zero-test|no-tests|developer-449|lane156|lane157|lane158" .harness/reports .harness/*.md docs
sed -n '1,260p' .harness/reports/run220-runtime-gate-repair-developer-449.md
sed -n '1,260p' .harness/reports/run221-runtime-duplicate-comparison-audit-dev453.md
sed -n '1,280p' .harness/reports/run222-command-selection-patch-point-audit-dev452.md
sed -n '1,280p' /home/claude/php-to-native-compiler/.harness/reports/run222-command-selection-live-fix-dev451.md
git show --stat --oneline --decorate 83c698fe
git show --stat --oneline --decorate af91813e418e576ca520eaa09c6a1c63ff40a77e
git show --stat --oneline --decorate ba3e30b6 dfa4ba7c
git status --short -- .harness/reports tests docs compiler runtime README.md Cargo.toml Cargo.lock
```

## No Source Edit Proof

Before writing this report, scoped product-path status was empty:

```sh
git status --short -- .harness/reports tests docs compiler runtime README.md Cargo.toml Cargo.lock
```

This lane writes only:

```text
.harness/reports/run221-run222-failure-manifest-dev452.md
```
