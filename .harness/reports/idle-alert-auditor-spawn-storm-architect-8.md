# Idle-Alert Auditor Spawn Storm Root Cause

Architect: architect-8
Timestamp: 2026-06-05T08:41:45Z
Scope: harness/idle-alert-auditor-spawn-storm

## Summary

The repeated failure is a harness control-plane design bug, not a PHP compiler
or PHPT metric issue. The scheduler treats agent liveness, alert routing, and
agent spawning as display-text conventions instead of a single durable state
machine. Once auditor rows begin to stale out, the scheduler can recursively
spawn auditors to investigate idle auditors, and the database has no stable
per-alert key or atomic spawn lease to collapse duplicates.

## Evidence

- The harness archive still implements `llm_harness/scheduler.py` with exact
  `current_status = 'running'` selectors for team capacity, liveness scans,
  auditor selection, and manager selection. `db.list_agents(conn, "running")`
  is also an exact-status query.
- Existing focused harness tests fail:
  `python3 -m unittest discover -s .harness/tests -v` reports 2 failures:
  freeform live statuses are not counted as running, and the test loop does not
  prefer `tools/run-tests.sh`.
- Recent DB evidence shows recursive auditor starts:
  - 2026-06-05T08:19: 207 `Started auditor-* for Investigate scheduler alert`
    events.
  - 2026-06-05T08:25: 110 more auditor starts.
  - 2026-06-05T08:33: 67 more auditor starts, including multi-start bursts in
    the same second.
- The same database currently has only 27 exact `current_status='running'`
  rows but 281 rows with `ended_at IS NULL`; 78 no-ended auditor rows were
  already stale by `last_seen_at > 30 minutes`.
- The goal/auditor summary recorded earlier storm counts at
  2026-06-05T08:25:51Z: 2579 active auditors, 2321 missing-tmux auditors,
  2151 idle auditors, and 1046 auditor starts in 10 minutes.

## Structural Root Cause

1. `current_status` is used both as a lifecycle state and a human-readable
   status message. Agents frequently replace `running` with freeform text such
   as `auditing scheduler idle alert`, so exact-status selectors lose live
   agents and do not cleanly distinguish active, completed, crashed, and stale
   rows.
2. Liveness reconciliation is embedded inside alert generation. Missing tmux
   panes are only retired for rows that are selected by the same exact-running
   path. Freeform active-looking rows with no `ended_at` can remain eligible
   for repeated higher-level diagnosis.
3. Idle alerts have no durable key. `last_prompt_at` is per agent, but there is
   no `idle:<agent>` alert row with open/resolved state, owner, cooldown, or
   sent count. That makes duplicate same-target alerts and recursive
   auditor-for-auditor alerts possible.
4. `prompt_auditor()` uses one global `last_auditor_spawn_epoch` metadata value
   as a throttle. That is not per target, not tied to an alert key, and not
   atomically claimed under a write transaction. Historical same-second bursts
   show the throttle did not act as a scheduler-wide spawn lease.
5. The test loop still rediscovers the stale `python -m unittest discover -s
   tests -v` command in this repo. That keeps creating repeated failing-test
   events and repeated architect spawn requests around the same control-plane
   bug.

## Reliability Refactor Plan

1. Add one lifecycle contract.
   - Minimal path: introduce helper queries in `llm_harness.db`:
     `active_agents(conn, role=None)`, `terminal_agent(row)`, and
     `mark_agent_ended(...)`, where active means `ended_at IS NULL` and status
     text is only display text.
   - Stronger path: migrate `agents` with a separate `lifecycle_state` enum
     (`running`, `completed`, `failed`, `crashed`, `stale`) and keep
     `current_status` as display text.

2. Split reconciliation from alerting.
   - At the start of each scheduler tick, reconcile every active row with tmux
     backing.
   - Missing pane/window/process rows become ended stale/crashed rows before
     idle candidates are computed.
   - Idle alert selection should consume only reconciled active rows.

3. Add an alert ledger.
   - Create a `scheduler_alerts` table or equivalent metadata helper keyed by
     `(alert_type, target_agent)`.
   - Record `status`, `first_seen_at`, `last_sent_at`, `sent_count`, and
     `assigned_agent`.
   - Enforce a unique open alert per key and close it when the target updates
     heartbeat or ends.

4. Stop recursive auditor spawning.
   - Do not spawn a new auditor to investigate a stale or missing-window
     auditor. Retire stale alert-auditor rows during reconciliation.
   - If a live auditor itself is idle, route one deduped escalation to Manager,
     not to another fresh auditor.
   - Preserve one genuine alert for a live idle Developer/Integrator/Manager
     lane owner.

5. Make spawn throttles atomic.
   - Wrap alert claim and spawn decision in `BEGIN IMMEDIATE`.
   - Use a per-alert-key spawn lease, not a global timestamp.
   - Before `spawn_agent`, check for an active agent with the same role/title
     or an open alert already assigned.

6. Add a scheduler singleton guard.
   - Use a repo-local lock file or SQLite lease with owner PID and heartbeat.
   - `./harness run` should refuse lifecycle/spawn work when another live
     scheduler owns the lease; `--once`/status commands should not spawn agents
     unless they hold the lease.

7. Fix test discovery.
   - `discover_test_command(root)` should prefer executable
     `tools/run-tests.sh` before pytest/unittest autodiscovery.
   - Keep the existing focused harness tests green before using broad test-loop
     failures as scheduler input.

## Acceptance Tests

- `db.list_agents(conn, "running")` or its replacement includes freeform active
  statuses and excludes stopped/failed/crashed/ended rows.
- Ended, failed, stopped, and missing-window agents are not idle-alert
  candidates.
- A missing-window auditor row is retired without spawning another auditor.
- Repeated ticks for the same stale target create one open alert and no more
  than one prompt/spawn inside the cooldown.
- Concurrent simulated scheduler ticks cannot spawn two auditors for the same
  alert key.
- One genuine live idle lane owner still produces one actionable alert.
- `python3 -m unittest discover -s .harness/tests -v` passes.

## Coordination

Lane 100 / developer-201 is the implementation owner for the harness zipapp and
`.harness/tests`. This architect report should be used to narrow that work; no
compiler/runtime source edits are needed for this failure.
