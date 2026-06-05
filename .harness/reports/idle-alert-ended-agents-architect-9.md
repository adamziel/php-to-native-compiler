# Idle-Alert Ended-Agents Root Cause

Architect: architect-9
Timestamp: 2026-06-05T08:48:00Z
Scope: harness/idle-alert-ended-agents

## Finding

This is a harness control-plane lifecycle bug. The scheduler does not have one
authoritative definition of an active agent. It uses `current_status` as both a
machine state and human-readable progress text, while `ended_at`, tmux backing,
and prompt throttling are treated as secondary checks in scattered call sites.

That mismatch explains the repeated ended/stale-agent idle alerts and why the
problem recurs after individual auditors retire rows.

## Evidence

- `llm_harness/db.py::list_agents(conn, "running")` is an exact
  `current_status = 'running'` query.
- `llm_harness/scheduler.py::check_agent_liveness()` only scans that exact
  running set, then performs missing-tmux retirement and idle prompting inside
  the same loop.
- `llm_harness/scheduler.py::ensure_team()`, `prompt_auditor()`, and
  `prompt_manager()` also use exact `current_status = 'running'` selectors.
- Existing focused harness tests are already red:
  `python3 -m unittest discover -s .harness/tests -v` fails
  `test_freeform_live_status_counts_as_running`, proving the code disagrees
  with the intended active-agent contract.
- The DB still contains display-terminal rows with no `ended_at`: at inspection
  time there were 43 rows whose status text included completed/failed/crash-like
  terms while `ended_at IS NULL`.
- The related test-loop escalation is undeduped. The same repeated open bug
  produced Architect spawn requests `25`, `29`, and `32` for
  `harness/idle-alert-ended-agents`.

## Structural Cause

The scheduler needs a lifecycle state machine, but currently infers lifecycle
from display text.

Exact `running` selectors undercount real live agents that report freeform
progress text. Broader "not stopped" style selectors overcount completed,
failed, or retired rows when agents forget to set `ended_at`. Missing tmux
reconciliation happens only for whichever rows a caller selected first, so stale
rows can survive until another auditor manually retires them. The per-agent
`last_prompt_at` field hides immediate repeats, but it is not a durable
alert-resolution record and does not stop repeated architect escalation for the
same open bug.

## Reliability Refactor Plan

1. Add a single lifecycle predicate.
   - Short path: add `db.is_active_agent(row)` and
     `db.list_active_agents(role=None)`.
   - Active means `ended_at IS NULL` and lifecycle is not terminal; display
     text stays in `current_status`.
   - Terminal updates must go through `db.update_agent_status(..., ended=True)`
     or a new `db.end_agent(...)` helper.

2. Reconcile before alerting.
   - At the start of each scheduler tick, scan all active rows.
   - If recorded tmux window/pane/process backing is missing, mark the row
     ended as stale/crashed before idle candidates are computed.
   - Idle alert generation must consume only reconciled active rows.

3. Replace text throttles with an alert ledger.
   - Add `scheduler_alerts(alert_type, target_agent, status, first_seen_at,
     last_sent_at, sent_count, assigned_agent)`.
   - Enforce one open alert per `(alert_type, target_agent)`.
   - Close alerts when the target heartbeats, is ended, or is superseded.

4. Deduplicate repeated systemic escalations.
   - `maybe_invoke_architect()` should not queue another Architect for the same
     open `bug_reports.test_nodeid` while an Architect spawn request or live
     Architect agent for that title already exists.
   - This would have collapsed requests `25/29/32` into one investigation.

5. Add focused tests before broad test-loop reliance.
   - Ended rows with non-null `ended_at` are never idle-alert candidates.
   - Failed/stopped/completed display statuses with `ended_at NULL` are either
     normalized to ended or excluded from idle alerts.
   - Freeform live statuses remain active for team/liveness accounting.
   - Missing-window rows are retired before idle alerts.
   - Repeated ticks for one target produce one open alert.
   - One genuinely live idle lane owner still alerts once.

## Coordination

Lane 100 is the implementation lane for this control-plane fix. Keep the patch
inside the harness zipapp/source and `.harness/tests`; no compiler/runtime files
or PHPT compatibility claims are involved. Architect-8's
`idle-alert-auditor-spawn-storm` report covers the companion spawn-storm
symptom; this report narrows the ended-agent lifecycle contract.
