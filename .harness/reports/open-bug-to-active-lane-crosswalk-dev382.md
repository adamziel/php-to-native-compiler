# Open Bug Report To Active Lane Crosswalk - developer-382

Snapshot time: 2026-06-08T13:54:41Z

Scope: assigned worklane/card `110`, report-only crosswalk. No compiler,
runtime, harness executable, product tests, or support docs were edited. No
full PHPT gate was run and no public score movement is claimed.

Source state inspected: `b8ae8f0c4397` on `work/developer-382`.

## Summary

There are 13 open `bug_reports` rows in the current harness database.

- `bug_reports#8` through `#16` are one native lookup/invoke/constructor
  runtime-test cluster. They are covered by live `worklanes#2075`
  (`developer-381`, `work/developer-381`) because that lane owns the exact
  `tools/run-tests.sh` global gate. Several rows pass in some recent runs and
  fail in others, matching the known rotating native helper failure pattern.
- `bug_reports#17` and `#18` are stale-open codegen assertion rows. Their
  latest recorded results passed in `test_runs#42233`, and card `2074` has
  integration evidence for the codegen fix. No new source owner should be
  spawned for these rows unless they recur after closure.
- `bug_reports#20` (`abs_rejects_forms_outside_current_subset`) is open with a
  latest recorded failure in `test_runs#42233`. The only matching lane found is
  queued Architect worklane `2087`, currently unowned after `architect-80`
  ended without an accepted report.
- `bug_reports#19` (`rustc-LLVM`) is an infrastructure/toolchain-looking error
  from commit `ba11d24a...`; it has no active owner and no recent recurrence in
  the latest `b8ae8f0c...` loop rows inspected here.

The `issues` table has open rows for the same test keys, but every open issue
row inspected has `worklane_id = NULL`, so lane ownership currently has to be
inferred from `worklanes`, reports, and events rather than issue links.

## Crosswalk

| Bug | Latest recorded result | Active lane owner | Live owner? | Required proof | Risk |
| --- | --- | --- | --- | --- | --- |
| `#8` `tests::native_constructor_allocation_invoke_reference_carrier_owns_receiver_cell` | passed in `test_runs#42255` at `b8ae8f0c` | `worklanes#2075` `Fix global test suite failures` | Yes: `developer-381`, agent row `4049`, running on `work/developer-381` | Card `2075` must either make `tools/run-tests.sh` pass or file a structured report naming root cause, resolution, and remaining quarantined failures. | Keep under `2075`; do not spawn per-test duplicate owners just because this row remains open. |
| `#9` `tests::native_magic_method_lookup_rejects_malformed_signature_metadata_before_fallback` | failed in `test_runs#42255` at `b8ae8f0c` | `worklanes#2075` | Yes: `developer-381` live | Same exact `tools/run-tests.sh` proof for `2075`. | Active failure in the rotating native cluster. |
| `#10` `tests::native_constructor_allocation_invoke_carrier_owns_receiver_arguments_and_diagnostics` | passed in `test_runs#42255` at `b8ae8f0c` | `worklanes#2075` | Yes: `developer-381` live | Same exact `tools/run-tests.sh` proof for `2075`; close only after the global gate owner reports stable resolution. | Stale-open/rotating-row risk; avoid duplicate work. |
| `#11` `tests::native_lookup_plus_invoke_helpers_free_arguments_once_across_target_families` | failed in `test_runs#42255` at `b8ae8f0c` | `worklanes#2075` | Yes: `developer-381` live | Same exact `tools/run-tests.sh` proof for `2075`. | Active top-occurrence failure. |
| `#12` `tests::native_method_lookup_plus_invoke_dispatches_missing_and_inaccessible_methods_to_magic_call` | failed in `test_runs#42255` at `b8ae8f0c` | `worklanes#2075`; Architect root-cause report `1898` also exists | Yes for Developer owner; Architect row `1898` is queued/stale, not a live source owner | Same exact `tools/run-tests.sh` proof for `2075`; use Architect evidence only as diagnosis. | Duplicate Architect rows exist; keep source ownership single. |
| `#13` `tests::native_static_method_lookup_plus_invoke_dispatches_missing_and_inaccessible_methods_to_magic_call_static` | failed in `test_runs#42255` at `b8ae8f0c` | `worklanes#2075` | Yes: `developer-381` live | Same exact `tools/run-tests.sh` proof for `2075`. | Active rotating native cluster. |
| `#14` `tests::native_method_lookup_plus_invoke_uses_access_context_for_private_diagnostics` | passed in `test_runs#42255` at `b8ae8f0c` | `worklanes#2075` | Yes: `developer-381` live | Same exact `tools/run-tests.sh` proof for `2075`; close after stable global-gate evidence. | Stale-open/rotating-row risk. |
| `#15` `tests::native_closure_invoke_helpers_bridge_call_arguments_to_call_results` | failed in `test_runs#42255` at `b8ae8f0c` | `worklanes#2075` | Yes: `developer-381` live | Same exact `tools/run-tests.sh` proof for `2075`. | Active rotating native cluster. |
| `#16` `tests::native_constructor_allocation_invoke_carrier_cleans_up_failure_paths` | failed in `test_runs#42255` at `b8ae8f0c` | `worklanes#2075` | Yes: `developer-381` live | Same exact `tools/run-tests.sh` proof for `2075`. | Active rotating native cluster. |
| `#17` `codegen::tests::generated_c_string_comparison_safety_uses_shared_pair_classifier` | passed in `test_runs#42233` at `b8ae8f0c` | Completed/integrated codegen lane `2074` evidence; queued duplicate Integrator lane `2077` remains scheduler cleanup | No live source owner needed | Scheduler/manager should close or mark fixed after consuming `integrator-27` report `666` and latest passing result. | Stale-open bug row can cause duplicate codegen spawns. |
| `#18` `codegen::tests::c_assembly_non_local_assignment_families_share_assignment_owner_boundary` | passed in `test_runs#42233` at `b8ae8f0c` | Completed/integrated codegen lane `2074` evidence; queued duplicate Integrator lane `2077` remains scheduler cleanup | No live source owner needed | Same closure proof as `#17`: report `666`, event `3790187`, latest pass in `42233`. | Stale-open bug row can cause duplicate codegen spawns. |
| `#19` `rustc-LLVM` | error in `test_runs#39718` at `ba11d24a` | None found | No | Assign an infrastructure/toolchain classification owner only if it recurs; otherwise mark as stale infrastructure after verifying no later current-commit recurrence. | Unowned row, but not currently driving latest `b8ae8f0c` failure summaries inspected here. |
| `#20` `abs_rejects_forms_outside_current_subset` | failed in `test_runs#42233` at `b8ae8f0c` | Queued Architect lane `2087` | No; `architect-80` ended without accepted report and lane `2087` is queued/unowned | Reassign/consume lane `2087` for systemic classification, then create a narrow Developer lane only if the failure is confirmed outside the active `2075` native cluster. | Currently unowned non-native failure row. |

## Native Cluster Ownership

The native cluster should stay under one source owner:

- `worklanes#2075` is assigned to `developer-381` and has the explicit
  acceptance command `tools/run-tests.sh`.
- `agents#4049` for `developer-381` is live/running, with notes saying it is
  working assigned card `2075` to restore native invocation cleanup tests.
- Coordinator events repeatedly route source ownership to card `2075` and
  warn against duplicate native/source fan-out.
- Architect evidence for this family identifies shared test instrumentation as
  a root-cause pattern, but the accepted implementation proof still belongs to
  the live global-gate Developer owner.

Recommended action: do not spawn separate Developers for individual native
bug rows `#8` through `#16` while `developer-381/card2075` is live and has not
reported a blocker or handoff.

## Stale-Open Closure Candidates

`bug_reports#17` and `#18` should be scheduler cleanup, not source work:

- `integrator-27` report `666` says card `2074` was integrated as commit
  `040f05b26ff1936839449118f34ee37903b6c322`, changing only
  `compiler/src/codegen.rs`.
- Event `3790187` records the same integration and focused codegen proof.
- `developer-373` report `675` verifies requeued card `2074` was duplicate/
  superseded by that integrated evidence.
- Latest per-test rows for both bug reports passed in `test_runs#42233` at
  current inspected commit `b8ae8f0c...`.

Recommended action: close or mark fixed/stale for rows `#17` and `#18`, and
retire duplicate queued integration cleanup `worklanes#2077` if it is still
present.

## Unowned Rows

`bug_reports#20` needs a fresh owner decision. The queued Architect card `2087`
is the matching deterministic next step, but it is currently unowned after an
ended architect row. If the Architect classification confirms a current
semantic failure, open a narrow Developer lane. If it was a transient or stale
selector row, close it with the exact run evidence.

`bug_reports#19` should be treated as infrastructure until proven otherwise.
It came from a different commit (`ba11d24a...`) and did not appear in the
latest current-commit rows inspected for this report.

## Queries And Commands Used

Project context:

```sh
sed -n '1,240p' AGENTS.md
sed -n '1,260p' docs/PROGRESS.md
sed -n '1,260p' docs/ARCHITECTURE.md
sed -n '1,260p' docs/SUPPORT.md
sed -n '1,260p' README.md
sed -n '1,260p' docs/LOOP_MEMORY.md
sed -n '1,260p' /home/claude/php-to-native-compiler/DEVELOPMENT.md
git status --short
git rev-parse --short=12 HEAD
git branch --show-current
```

Harness state:

```sql
SELECT id, test_nodeid, status, occurrences, first_failed_commit,
       fixed_commit, root_cause, resolution, created_at, updated_at
FROM bug_reports
WHERE status = 'open'
ORDER BY occurrences DESC, updated_at DESC, id ASC;

WITH open_bugs AS (
  SELECT id, test_nodeid FROM bug_reports WHERE status = 'open'
),
ranked AS (
  SELECT tr.nodeid, tr.status AS result_status, tr.message, tr.run_id,
         r.started_at, r.ended_at, r.command, r.commit_sha,
         r.status AS run_status,
         ROW_NUMBER() OVER (
           PARTITION BY tr.nodeid ORDER BY r.started_at DESC, tr.id DESC
         ) AS rn
  FROM test_results tr
  JOIN test_runs r ON r.id = tr.run_id
  JOIN open_bugs b ON b.test_nodeid = tr.nodeid
)
SELECT b.id AS bug_id, b.test_nodeid, ranked.run_id, ranked.started_at,
       ranked.command, ranked.commit_sha, ranked.run_status,
       ranked.result_status, substr(ranked.message,1,500) AS message
FROM open_bugs b
LEFT JOIN ranked ON ranked.nodeid = b.test_nodeid AND ranked.rn = 1
ORDER BY b.id;

SELECT id, title, role_type, priority, status, stage, owner_agent_id,
       branch_name, worktree_path, notes, acceptance_criteria, last_activity_at
FROM worklanes
WHERE id IN (1898,2074,2075,2087)
   OR title LIKE '%native%'
   OR title LIKE '%global test suite%'
   OR title LIKE '%codegen assertion%'
   OR title LIKE '%abs_rejects%'
ORDER BY CASE WHEN id IN (2075,2087,2074,1898) THEN 0 ELSE 1 END,
         id DESC
LIMIT 80;

SELECT id, name, role, current_status, tmux_window, tmux_pane, worktree,
       branch, last_seen_at, ended_at, notes
FROM agents
WHERE id IN (4049,4050)
   OR name IN ('developer-381','developer-382','architect-7','architect-80')
ORDER BY name, id DESC;

SELECT id, issue_key, title, status, severity, source, worklane_id,
       root_cause, resolution, created_at, updated_at
FROM issues
WHERE status = 'open'
ORDER BY severity DESC, updated_at DESC, id ASC
LIMIT 200;

SELECT id, created_at, agent_name, worklane_id, card_id, stage, status,
       report_json
FROM agent_reports
WHERE worklane_id IN (2074,2075,2087,1898)
   OR card_id IN (2074,2075,2087,1898)
ORDER BY id DESC
LIMIT 30;
```

Validation performed for this report artifact:

```sh
git diff --check -- .harness/reports/open-bug-to-active-lane-crosswalk-dev382.md
git diff --cached --check -- .harness/reports/open-bug-to-active-lane-crosswalk-dev382.md
```
