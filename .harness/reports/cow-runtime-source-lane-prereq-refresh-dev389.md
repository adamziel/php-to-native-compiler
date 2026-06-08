# COW/runtime source-lane prerequisite audit refresh

Generated: 2026-06-08T14:42:44Z
Worker: developer-389
Lane: work_lanes#111
Artifact scope: report only; no compiler/runtime edits, no PHPT gate, no public score movement.

## Bottom line

Do not open a new broad COW/runtime source lane from this lane. The previous
lane-111 audit artifact is already present, and the current scheduler state no
longer has the exact June 5 command-selection blocker, but the system is still
not in a clean source-expansion posture:

- The accepted public PHPT score is still unchanged at `7873 / 20294`.
- `tools/run-tests.sh` is now the discovered command, but the current gate is
  in `quarantined_known_red` mode with repeated failures on commit
  `5ce154a6ec08d17e6ad0b7fa4f847f8ca9dc2dd9`.
- The scheduler has an active owner for the global test-suite failure lane
  (`work_lanes#2075`) and two active non-native-helper failure lanes
  (`work_lanes#2099` and `work_lanes#2100`).
- Integration/control-plane churn is still active. Recent coordinator state
  says capacity should be routed to integration or conflict-resolution support,
  and an integrator was spawned for stale integration/control-plane rows.

The deterministic next action is therefore not source implementation. Keep
COW source work mechanism-first and only reopen it through a fresh Manager or
Coordinator lane that names one owner mechanism, target fixtures, and known
unsupported edges.

## Current scheduler evidence

Current control-plane lanes:

- `work_lanes#8`, command-selection fix, is `integrated`.
- `work_lanes#100`, idle-alert filtering/dedupe, is `integrated`.
- `work_lanes#133`, the prior COW runtime lvalue-handle source slice, is
  `integrated`.
- `work_lanes#143`, the run-215 command-selection recurrence, is `stale` and
  superseded by global full-suite stabilization.
- `work_lanes#2075`, global test-suite failures, is assigned and references
  latest failing run `42406`.
- `work_lanes#2099` and `work_lanes#2100` are assigned Developer lanes for
  non-native-helper failures under known-red quarantine.

Current metadata:

- `test_gate_mode = quarantined_known_red`.
- `test_gate_failure_count = 9`.
- `test_gate_reason = 9 known failures are quarantined; metric acceptance
  remains blocked, but unrelated work may continue.`
- `test_gate_run_id = 42403` at query time, with later run rows continuing to
  fail under `tools/run-tests.sh`.
- `integration_backpressure_active = 1`.
- `best_progress_percent = 35.46`, which is the blocked 221205Z gate and not
  an accepted public score movement.

Recent event evidence:

- A manhole state summary at 2026-06-08T14:41:47Z records accepted public PHPT
  passes unchanged at `7873 / 20294`, current `tools/run-tests.sh` rows still
  failing on commit `5ce154a6`, and no source edits.
- A coordinator decision at 2026-06-08T14:42:09Z routes capacity to one
  integration/control-plane support replacement instead of feature development.
- Repeated integration-idle events report no `needs_verification` or
  `ready_for_integration` lanes with branches, while separate integration
  failures remain in the lane table.

## COW intake state

The COW frontier remains valid but should stay mechanism-owned:

- `RPR`: runtime provenance resolver. `ArrayCopySource` roots should resolve
  through `RuntimeAliasLvalueHandle` identities before rehydration, mirror,
  promotion, import remapping, or writeback.
- `DMB`: dynamic mutation boundary. Writes, unsets, compound writes,
  reference assignment, alias writes, and property replacement should capture
  old storage identity and reuse shared invalidation or rehydration.
- `CCA`: callback call-frame adapter. Direct calls, closures, dynamic calls,
  array-callables, `call_user_func()`, and `call_user_func_array()` should
  carry copied-source metadata through shared call-frame carriers.
- `GAP`: untracked containers, unreachable runtime cells, string COW, native
  COW lowering, unsupported syntax, and exact PHP diagnostics remain explicit
  unsupported gaps.

`docs/COW_COVERAGE_MATRIX.md` still rejects one-off wrapper branches and keeps
literal array transforms as proof coverage, not a separate mechanism. The open
`docs/NEXT_TASKS.md` COW item still targets broader dynamic holder writeback
and untracked-container propagation through handles, with
`milestone2294/magic_array_merge_integer_reindex_cow.php` as representative
integration risk coverage when remapping is touched.

## Prerequisites before a fresh source lane

A new source lane should require all of the following:

1. A fresh Manager/Coordinator assignment, not a self-selected continuation of
   lane 111.
2. One owning mechanism named up front: `RPR`, `DMB`, or `CCA`.
3. A focused test plan that includes Rust unit or integration coverage plus a
   CLI fixture path when behavior is claimed.
4. A conflict plan for shared files, especially `compiler/src/interpreter.rs`,
   `docs/PROGRESS.md`, `docs/SUPPORT.md`, `docs/ARCHITECTURE.md`, and
   `docs/NEXT_TASKS.md`.
5. Explicit unsupported edges retained in docs before review.
6. Current known-red and integration state acknowledged, with no public PHPT
   score claim unless a full zero-regression gate accepts it.

## Recommended next deterministic actions

- Let `work_lanes#2075`, `#2099`, and `#2100` finish their current
  known-red/focused-failure work before adding another source-expansion lane.
- Prefer integration/conflict-resolution support while
  `integration_backpressure_active = 1`.
- If COW source work is reopened later, prefer a small `RPR` extraction or
  handle-backed dynamic-holder slice over case-specific compatibility patches.
- Keep `eval` and variable-variable work deferred as late-priority scope.

## Verification

This lane is report-only. No compiler/runtime tests or PHPT gates were run.
Verification for this artifact should be limited to whitespace/path checks and
review of the recorded SQLite evidence.

Branch handoff note:

- Local commit `540714eb` was created on `work/developer-389`.
- Pushing to `origin/work/developer-389` was rejected because that remote
  branch is an older unrelated source branch at `6db16b03`
  (`runtime: add bounded str_ireplace builtin`) and is not a safe
  fast-forward target for this report-only lane.
- The report commit was pushed to `origin/work/developer-389-lane111-refresh`
  for review without rewriting the old remote branch.

Commands and queries used included:

- Required reading: `AGENTS.md`, `docs/PROGRESS.md`,
  `docs/ARCHITECTURE.md`, `docs/SUPPORT.md`, `README.md`,
  `docs/LOOP_MEMORY.md`, `/home/claude/php-to-native-compiler/DEVELOPMENT.md`,
  `/home/claude/php-to-native-compiler/PLAN.md`, `docs/NEXT_TASKS.md`, and
  `docs/COW_COVERAGE_MATRIX.md`.
- Git checks: `git status --short`, `git rev-parse HEAD`,
  `git diff --check -- .harness/reports/cow-runtime-source-lane-prereq-dev226.md docs/COW_COVERAGE_MATRIX.md docs/NEXT_TASKS.md`.
- SQLite/MCP queries over `worklanes`, `bug_reports`, `metadata`,
  `test_runs`, `events`, and `agent_reports`.
