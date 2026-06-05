# Missing-Window Undelivered-Assignment Root Cause

Architect: architect-10
Timestamp: 2026-06-05T08:49:00Z
Scope: harness/idle-alert-missing-window-undelivered-assignment

## Summary

This is a harness control-plane reliability bug, not PHP compiler work.
Assignments can be considered launched or queued before the harness has proved
that a live Codex process exists, that the intended tmux target is reachable,
or that the assignment prompt was actually delivered to that target. When the
tmux window later disappears, the system discovers the failure through idle
audits instead of through the assignment lifecycle itself.

The current cleanup has removed the immediate stale exact-running rows, but the
structural gap remains: spawn, assignment delivery, liveness reconciliation,
and idle-alert routing are separate best-effort paths without one durable state
machine.

## Evidence

- `bug_reports#4` is open with 28 occurrences. Its recorded root cause is a
  running agent row with stale tmux session/window and queued, undelivered
  assignment messages.
- Historical examples in the DB include developer-87, developer-89,
  developer-90, developer-107, developer-108, developer-109, developer-110,
  developer-111, developer-178, developer-195, and multiple stale auditor and
  integrator rows. The recurring shape is: clean worktree, no artifact, queued
  or superseded assignment message, and missing tmux window/pane.
- The harness zipapp implements `spawn_agent()` as: write prompt file, create
  or reuse a tmux window, then immediately `upsert_agent(...,
  current_status="running")`. There is no post-start handshake and no Codex
  process check before the row becomes running.
- `Tmux.ensure_window()` reuses a same-named window if it exists. It does not
  verify that the existing pane is running the intended Codex command, that the
  prompt file was consumed, or that the pane belongs to the current agent
  generation.
- `poke()` persists one message row, attempts delivery to selected agents, and
  marks the message `delivered` if at least one send succeeds. It does not
  store per-recipient delivery results, retire unreachable targets, or requeue
  lane work when the intended target cannot receive the prompt.
- `check_agent_liveness()` only scans `db.list_agents(conn, "running")`, and
  `db.list_agents(conn, "running")` is an exact display-status query. Existing
  focused tests prove the abstraction leak: freeform live statuses are not
  counted as running.
- Current focused harness tests fail:
  `python3 -m unittest discover -s .harness/tests -v` reports failures for
  freeform live status handling and `tools/run-tests.sh` discovery.
- Repeated recent test-loop runs used `python -m unittest discover -s tests -v`
  and recorded `NO TESTS RAN`. That unrelated command-selection failure then
  fed repeated control-plane bug escalation.

## Structural Root Cause

The scheduler has no assignment state machine. It stores agents, messages, and
spawn requests as independent records, then relies on auditors to infer whether
work reached a live worker. A tmux pane existing at one instant is treated as
equivalent to a started Codex worker, and a queued prompt being attempted is
treated as equivalent to target-specific delivery.

That design leaves four gaps:

1. Agent lifecycle is encoded in `current_status`, which is also human-readable
   display text. Liveness and capacity use exact string matches.
2. Spawn has no `starting -> running -> stale/failed` transition with an
   explicit post-start proof.
3. Messages have no per-target delivery ledger or acknowledgement path.
4. Missing-window reconciliation is reactive and alert-driven instead of being
   the first step before assignment and idle-alert candidate selection.

## Refactor Plan

1. Separate lifecycle from display text.
   - Add `lifecycle_state` or a central helper that treats active as
     `ended_at IS NULL` plus non-terminal status.
   - Keep `current_status` for display only.
   - Replace exact `current_status='running'` selectors in scheduler,
     dashboard counts, manager/auditor selection, and team capacity.

2. Make spawn two-phase.
   - Insert or update the agent row as `starting`, not `running`.
   - Create a unique generation token for the prompt/window.
   - After `tmux.ensure_window()`, verify the target still exists and, where
     possible, that a Codex process/pane command is associated with that
     generation.
   - Only then mark the agent active/running. On failure, mark the spawn
     request failed, end the agent row, and do not assign lane work to it.

3. Add assignment delivery records.
   - Either extend `messages` with per-recipient rows or add
     `message_deliveries(message_id, target_agent, status, delivered_at,
     failed_at, failure_reason)`.
   - A role-targeted or broadcast message should be considered complete only
     per recipient, not globally after one successful send.
   - A direct assignment to a missing target should be marked failed
     immediately and should trigger deterministic lane/message requeue.

4. Reconcile before assigning or alerting.
   - At the beginning of each scheduler tick, scan every active row, not just
     exact `running` rows.
   - Missing tmux target or missing Codex backing should end the agent row with
     a stable stale/crashed lifecycle state.
   - For stale agents with active lane ownership or queued assignment
     deliveries, atomically requeue the lane once and create at most one
     replacement spawn request.

5. Make idle alerts consume reconciled state only.
   - Idle candidate selection should run after reconciliation and should ignore
     ended/stale/missing-window rows.
   - A missing-window worker should be a lifecycle repair event, not an auditor
     investigation target.

6. Fix the test-loop command selection first.
   - `discover_test_command(root)` should prefer executable
     `tools/run-tests.sh` before generic pytest/unittest discovery.
   - This prevents empty harness test runs from repeatedly escalating the same
     control-plane bug.

## Acceptance Tests

- A fake tmux target that disappears during spawn leaves the agent ended/failed
  and does not mark the spawn request as started.
- A same-name pre-existing tmux window that is not the current Codex generation
  is not accepted as proof of a running agent.
- `poke(target=missing_agent)` records failed delivery for that agent, marks
  the message failed or partially delivered, and does not leave an ambiguous
  queued assignment forever.
- Broadcast or role-target messages record per-recipient delivery status.
- Scheduler reconciliation scans freeform active statuses and retires missing
  tmux targets before idle candidates are computed.
- A stale missing-window lane owner causes one lane requeue and one replacement
  request, not repeated auditor spawns.
- A genuine live idle lane owner still produces one deduped actionable alert.
- `python3 -m unittest discover -s .harness/tests -v` passes.

## Implementation Ownership

Lane 100 should own the harness zipapp and `.harness/tests` changes. This
failure should not consume compiler/runtime workers and should not claim PHPT
metric progress. The sibling architect-8 report covers the auditor-spawn-storm
dedupe ledger; this report adds the missing assignment-delivery and spawn
handshake contract that prevents stale workers from receiving lane ownership in
the first place.
