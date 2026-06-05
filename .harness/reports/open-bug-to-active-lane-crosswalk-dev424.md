# Open Bug Report To Active Lane Crosswalk

## Scope

- Lane: `work_lanes#110`, assigned to `developer-424` by manager-24 at
  `2026-06-05T09:59:30Z`.
- Artifact: `.harness/reports/open-bug-to-active-lane-crosswalk-dev424.md`.
- Source state inspected: `8381ad99`.
- Report timestamp: `2026-06-05T10:02:48Z`.
- This is SQLite/report-only work. No compiler, runtime, harness, or product
  source files were edited.
- No full PHPT gate was run, and no public score movement is claimed.
- MCP memory reads/writes were attempted first. The database was intermittently
  locked, so read-only Python `sqlite3` fallback was used for the crosswalk.

## Executive Summary

There are five open `bug_reports` rows:

- `bug_reports#1` is the `php_runtime --lib` run62/runtime ABI cluster. It has
  candidate fix evidence and merge-prerequisite reporting, but no live active
  implementation owner. The remaining action is integration/proof on a clean
  target, not a new duplicate runtime lane.
- `bug_reports#3`, `#4`, `#5`, and `#6` are one overlapping harness liveness
  and idle-alert family. `work_lanes#100` is marked completed on stopped
  `developer-402`, but current manager evidence from `2026-06-05T10:00:13Z`
  and `2026-06-05T10:00:28Z` says the live root still selects the zero-test
  Python command and focused `.harness` tests fail `3/7`. Treat this family as
  not implementation-resolved in the live root.
- `bug_reports#2` is marked fixed, but `test_runs#215` is a fresh linked
  recurrence of the same command-selection symptom. It is covered by active
  `work_lanes#143` on live `developer-419` and read-only audit `work_lanes#144`
  on live `developer-431`.

## Crosswalk

| Bug | Status | Current lane owner | Live owner? | Required proof | Risk |
| --- | --- | --- | --- | --- | --- |
| `#1` `php_runtime::lib::run62_runtime_abi_expectation_cluster` | open | Evidence lanes: `#66` completed on `work/developer-120`; `#125` integrated report from `work/developer-308`; `#67` superseded. | No live implementation owner. `developer-120` and `developer-308` rows are stopped. | Cleanly integrate the canonical runtime pair already identified by lane `#125`, then run at least `git diff --check`, `cargo fmt --check`, and `cargo test -p php_runtime --lib` with a unique target dir. Update `bug_reports#1` only after integrated proof is attached. | Unowned for live integration/proof. Do not spawn another duplicate source repair without resolving dirty-overlap integration first. |
| `#3` `harness/idle-alert-ended-agents` | open | `#100` completed on `work/developer-402`; architect investigation live on `architect-15`. | No live lane owner for implementation. `developer-402`, `developer-393`, `developer-405`, and `developer-406` rows are stopped. | Focused `.harness` tests must pass in the live root, including ended-row exclusion and missing-pane/liveness predicates, plus before/after candidate counts. | Still open because manager evidence after run215 rejected the current live fix as ineffective. |
| `#4` `harness/idle-alert-missing-window-undelivered-assignment` | open | Same implementation family as `#100`; stale-row report `#121` completed on `work/developer-418`. | No live lane owner for implementation. `developer-418` is stopped; report evidence only. | Live root must exclude or retire missing-window/missing-pane actionable rows before assignment/alert routing, prove queued-message cleanup or redelivery semantics, and preserve one genuine live idle alert. | Duplicates/overlaps with `#3`, `#5`, and `#6`; should be tracked under one canonical liveness repair owner. |
| `#5` `harness::idle_alert_ended_agents` | open | Same ended-agent symptom as `#3`; no separate active lane found. | No live implementation owner. | Same proof as `#3`: ended agents cannot be selected for idle alerts after `ended_at` is set, with focused `.harness` coverage and live-root verification. | Duplicate-row risk. Prefer closing or linking this row after `#3` proof instead of creating another lane. |
| `#6` `harness/idle-alert-auditor-spawn-storm` | open | `#100` notes say it owns this too; current manager evidence says broader per-alert/atomic auditor-spawn dedupe remains unresolved. | No live implementation owner. | In addition to liveness filtering, prove per-alert or per-target auditor-spawn dedupe/throttle and an atomic lease or equivalent guard that prevents recursive auditor storms. | Highest duplicate-spawn risk. Needs a single live owner once manager-25 finishes current run215/liveness reorganization. |

## Linked Fixed Row

`bug_reports#2`
`harness::test_command_selection::python_unittest_zero_tests_run63` is marked
`fixed` at `2026-06-05T09:55:08Z`, but the latest failed test run shows the
same command-selection symptom:

- `test_runs#215`, started `2026-06-05T09:57:11Z`, ran
  `python -m unittest discover -s tests -v`, failed with zero passed tests, and
  recorded one synthetic failure.
- `events#94741` says a live recheck after run215 still returned the Python
  unittest command and focused `.harness` tests failed `3/7`.
- `events#94746` repeats that manager-25 verified the current root still
  selects Python unittest and that `test_test_loop_prefers_project_run_tests_script`,
  `test_liveness_ignores_ended_running_rows`, and
  `test_freeform_live_status_counts_as_running` fail.

Active linked work:

- `work_lanes#143` is in progress on live `developer-419` for the run215
  recurrence.
- `work_lanes#144` is in progress on live `developer-431` as a read-only
  post-fix timestamp/selector evidence audit.

This fixed row should not be reopened by this report-only lane, but managers
should treat its recurrence as blocking acceptance of the lane8/lane100
completion notes until a live-root harness proof supersedes run215.

## Duplicate And Unowned Risk

- Open bugs `#3`, `#4`, `#5`, and `#6` are a single control-plane family:
  ended-agent filtering, missing-window liveness, queued assignment delivery,
  and auditor-spawn dedupe. Splitting those into multiple source owners would
  increase drift because the predicate and scheduler alert path overlap.
- `work_lanes#100` is marked completed, but its recorded owners are stopped and
  the root live evidence after run215 contradicts the completion acceptance.
  It should be treated as evidence to inspect, not as a currently live owner.
- `work_lanes#119` is also stale for current acceptance: it ended with
  `completed_no_proof_ready`, and later events changed the facts.
- `bug_reports#1` is not currently blocked on another runtime developer
  writing code. The deterministic next step is a clean integration/proof
  sequence for the already identified runtime candidate pair.

## Recommended Deterministic Actions

1. Keep `developer-419` on `work_lanes#143` as the current live implementation
   owner for the run215 command-selection recurrence unless manager-25
   explicitly reassigns it.
2. After the run215 owner identifies the live-root patch path, assign one live
   owner to the whole open liveness family `#3/#4/#5/#6` instead of separate
   per-bug owners. Required proof should include focused `.harness` tests,
   selector dry-run, candidate counts, and dedupe/throttle evidence.
3. For `bug_reports#1`, have an integrator use lane `#125` to merge the
   canonical runtime candidate pair on a clean target and attach the
   `php_runtime --lib` proof before updating the bug row.

## Commands And Queries

Project context and git state:

```sh
rg --files -g 'AGENTS.md' -g 'DEVELOPMENT.md' -g 'CLAUDE.md' -g 'README.md' -g 'docs/PROGRESS.md' -g 'docs/ARCHITECTURE.md' -g 'docs/SUPPORT.md' -g 'docs/LOOP_MEMORY.md'
sed -n '1,240p' AGENTS.md
sed -n '1,260p' docs/PROGRESS.md
sed -n '1,260p' docs/ARCHITECTURE.md
sed -n '1,260p' docs/SUPPORT.md
sed -n '1,260p' README.md
sed -n '1,260p' docs/LOOP_MEMORY.md
git status --short --branch
git rev-parse --short HEAD
```

MCP queries attempted:

```sql
SELECT id, title, role, status, branch, worktree, expected_metric_delta, notes, ts FROM work_lanes WHERE id = 110;
SELECT id, test_nodeid, status, root_cause, resolution, occurrences, updated_at FROM bug_reports ORDER BY id;
SELECT id, started_at, ended_at, command, commit_sha, status, summary_json FROM test_runs ORDER BY id DESC LIMIT 20;
```

Read-only Python `sqlite3` fallback queries used after MCP lock errors:

```sql
SELECT id,test_nodeid,status,occurrences,root_cause,updated_at
FROM bug_reports
WHERE status NOT IN ('closed','fixed')
ORDER BY id;

SELECT id,test_nodeid,status,occurrences,updated_at
FROM bug_reports
ORDER BY id;

SELECT id,title,status,branch,worktree,substr(notes,1,1200) AS notes_head,
       substr(notes,-1200) AS notes_tail
FROM work_lanes
WHERE id IN (8,66,67,100,110,119,121,125,143,144)
ORDER BY id;

SELECT name,role,current_status,branch,worktree,last_seen_at,ended_at,notes
FROM agents
WHERE name IN (
  'developer-120','developer-122','developer-124','developer-308',
  'developer-393','developer-402','developer-405','developer-406',
  'developer-417','developer-418','developer-419','developer-424',
  'developer-431','architect-15','manager-25'
)
ORDER BY name,id DESC;

SELECT id,ts,type,agent_name,message,payload_json
FROM events
WHERE id>=94700
  AND (
    message LIKE '%run215%' OR message LIKE '%run 215%'
    OR message LIKE '%lane100%' OR message LIKE '%lane8%'
    OR message LIKE '%bug_reports#%' OR message LIKE '%idle-alert%'
    OR message LIKE '%discover_test_command%' OR message LIKE '%developer-424%'
  )
ORDER BY id DESC
LIMIT 50;

SELECT id,started_at,command,commit_sha,status,summary_json
FROM test_runs
WHERE status='failed'
ORDER BY id DESC
LIMIT 10;
```

Verification run for this report artifact:

```sh
git diff --check -- .harness/reports/open-bug-to-active-lane-crosswalk-dev424.md
```
