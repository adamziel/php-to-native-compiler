# Idle-Alert Auditor Spawn Storm Architecture

Architect: architect-11
Timestamp: 2026-06-05T08:44:00Z
Scope: harness control plane only. No compiler/runtime support claim.

## Structural Root Cause

The scheduler overloads `agents.current_status` as both a lifecycle enum and
free-form activity text. Agents correctly update it with statuses such as
`auditing scheduler idle alert`, but scheduler selection code still treats only
the exact string `running` as live:

- `llm_harness/db.py::list_agents(conn, "running")` filters with
  `current_status = 'running'`.
- `llm_harness/scheduler.py::ensure_team()` counts only exact `running` rows.
- `llm_harness/scheduler.py::prompt_auditor()` queries only exact `running`
  auditors.
- `llm_harness/scheduler.py::prompt_manager()` has the same exact-status
  assumption.

That mismatch turns a healthy auditor into invisible capacity as soon as it
records a meaningful free-form status. On each scheduler tick, `ensure_team()`
can then observe zero exact-running auditors and start another one. When stale
rows with missing tmux panes are also present, `prompt_auditor()` repeatedly
burns time marking old rows crashed and eventually starts more auditors.

The observed evidence matches this mechanism:

- Focused harness test `test_freeform_live_status_counts_as_running` fails:
  a live row with status `inspecting PHPT lane` is excluded from
  `db.list_agents(conn, "running")`.
- Historical events show auditor starts by minute peaking at `207` in
  `2026-06-05T08:19Z`, far above the intended single-auditor capacity model.
- The database currently contains `2729` auditor rows, including `2503`
  `crash` rows.
- `bug_reports#6` records repeated stale-auditor examples where the DB row
  still looked active to humans but the tmux window/pane was missing.

## Reliability Refactor Plan

1. Split lifecycle from activity text.
   - Short-term compatible patch: introduce a single helper such as
     `db.active_agent_where()` / `db.list_active_agents()` where active means
     `ended_at IS NULL` and status is not terminal (`crash`, `failed*`,
     `stopped`, `completed*`, `retired*`, `resolved*`).
   - Long-term schema cleanup: add a lifecycle column (`active`, `ended`,
     `crashed`, `retired`) and keep `current_status` as display text only.

2. Route all scheduler liveness through that helper.
   - `ensure_team()` should count active rows after retiring missing tmux
     targets, not exact `running` strings.
   - `check_agent_liveness()` should inspect active rows, not only exact
     `running` rows.
   - `prompt_auditor()`, `prompt_manager()`, and broadcast `poke()` should
     use the same active-agent predicate.

3. Add a single deterministic target validation boundary.
   - Resolve `_tmux_target(agent)` once.
   - If a recorded pane/window is missing, mark the row ended with a terminal
     status and emit one `agent_missing` event.
   - Do this before capacity accounting and before idle-alert candidate
     selection.

4. Dedupe idle alerts by target.
   - Track a stable key such as `idle_alert:<agent-name>` in `metadata`, or a
     small alert table if schema churn is acceptable.
   - Do not spawn another auditor for the same target while the key is within
     the throttle window or an active auditor has already received that alert.
   - Preserve the existing positive path: one genuinely live idle owner still
     creates one auditor prompt.

5. Dedupe repeated architect escalation.
   - `testing_loop.maybe_invoke_architect()` currently queues another Architect
     request for every open repeated bug on every failed run. Add a guard for
     an existing queued/started Architect request with the same title, or add
     `architect_requested_at` metadata keyed by bug id.

## Required Tests

Focused tests should use a fake tmux implementation and in-memory SQLite:

- Free-form active status counts as live capacity.
- Terminal statuses and `ended_at IS NOT NULL` never count as live.
- Missing tmux pane/window rows are retired before idle-alert spawning.
- Repeated idle checks for the same stale target do not start duplicate
  auditors.
- A real live idle agent still receives exactly one auditor prompt.
- Repeated `maybe_invoke_architect()` calls do not queue duplicate Architect
  requests for the same open repeated bug.

Acceptance command:

```sh
python3 -m unittest discover -s .harness/tests -v
```

Control-plane metric:

- Before patch: auditor starts reached hundreds per minute and stale active
  auditor rows dominated candidate counts.
- After patch: focused tests show zero duplicate stale-target auditor starts,
  exactly one preserved prompt for a live idle target, and no new duplicate
  Architect requests for unchanged repeated bug reports.
