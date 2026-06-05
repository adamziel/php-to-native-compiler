# Architect 6 Report: idle-alert-ended-agents

Timestamp: 2026-06-05T08:40Z

## Finding

`harness/idle-alert-ended-agents` is not a one-off stale row. The harness has
split lifecycle state across `agents.current_status` free text and
`agents.ended_at`, then different scheduler paths interpret those fields
differently.

Observed control-plane code in the embedded `harness` zipapp:

- `llm_harness.db.list_agents(conn, "running")` filters by exact
  `current_status = 'running'`.
- `llm_harness.scheduler.ensure_team()` counts capacity with exact
  `current_status = 'running'`.
- `llm_harness.scheduler.check_agent_liveness()` calls that exact running
  filter and does not independently require `ended_at IS NULL`.
- `llm_harness.scheduler.prompt_auditor()` and `prompt_manager()` also look
  for exact running rows, so free-form active auditors/managers are skipped
  and stale rows can drive replacement churn.

This contradicts the existing focused harness test
`test_freeform_live_status_counts_as_running`, which expects a free-form status
such as `inspecting PHPT lane` to count as live while `stopped` does not.

## Current Evidence

`python3 -m unittest discover -s .harness/tests -v` failed before any local
fix:

- `test_freeform_live_status_counts_as_running`: `developer-live` was missing
  from `db.list_agents(conn, "running")`.
- `test_test_loop_prefers_project_run_tests_script`: unrelated stale
  test-runner discovery failure.

Bug report `bug_reports#3` already has six occurrences and records many stale
developer/auditor rows with absent tmux panes. Recent events show repeated
architect spawn requests for the same repeated test, which is a separate
dedupe gap in the test-loop escalation path.

## Structural Cause

The harness lacks one authoritative active-agent predicate. Agent liveness,
capacity, broadcast delivery, status counts, and alert routing should not each
interpret text status independently.

Correct active-agent semantics should be:

- `ended_at IS NULL` is mandatory.
- obvious terminal statuses such as `stopped`, `crash`, `crashed`, `failed...`,
  `completed...`, and `ended...` are inactive compatibility fallbacks.
- every other free-form status is active.
- active candidates must have reachable tmux backing before they can be idle
  alert targets; missing window/pane rows should be retired deterministically
  instead of producing another idle-alert auditor.

## Reliability Refactor Plan

1. Add central lifecycle helpers in `llm_harness.db`:
   `is_terminal_status(status)`, `is_agent_active(row)`,
   `list_active_agents(conn, role=None)`, and `count_active_agents(conn, role)`.
   Keep `list_agents(conn, "running")` as a compatibility wrapper around the
   active predicate so the existing test passes.

2. Replace exact `current_status = 'running'` queries in scheduler paths:
   `ensure_team`, `check_agent_liveness`, `prompt_auditor`, `prompt_manager`,
   and broadcast `poke`.

3. In `check_agent_liveness`, retire unreachable active rows before idle
   evaluation. If `_tmux_target()` is empty or `target_exists()` fails, set a
   terminal status and `ended=True`; do not include that row in the idle batch.

4. Add per-target idle-alert dedupe. A small metadata key such as
   `idle_alert:<agent_name>` is enough if it is cleared when the agent's
   `last_seen_at` advances. This preserves one alert for a genuine live idle
   owner while preventing duplicate stale-target auditor storms.

5. Add `testing_loop.maybe_invoke_architect()` dedupe for repeated-failure
   architect spawn requests. Before queuing another architect, check for an
   existing queued or recently started request with the same title.

6. Extend `.harness/tests` with focused tests:
   ended `current_status='running'` rows do not alert; failed/stopped rows do
   not alert; missing tmux target rows are retired without alert; free-form
   active status counts as live; one genuine live idle row still alerts once;
   repeating the tick does not create duplicate same-target alerts; repeated
   failing-test escalation queues only one architect per test node.

## Acceptance

Run:

```sh
python3 -m unittest discover -s .harness/tests -v
```

Report before/after candidate counts for:

- active agent rows
- active rows with missing tmux backing
- idle alert candidates
- auditor spawn requests for the same target/test within 10 minutes

This is a harness control-plane fix only. It should not claim any PHP compiler
or PHPT compatibility metric movement.
