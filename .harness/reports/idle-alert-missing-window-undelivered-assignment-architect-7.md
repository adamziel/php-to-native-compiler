# Idle Alert Missing Window / Undelivered Assignment Root Cause

Architect: architect-7
Timestamp: 2026-06-05T08:44Z
Bug: `harness/idle-alert-missing-window-undelivered-assignment`

## Finding

The repeated failure is a control-plane lifecycle bug, not a PHP compiler
feature failure.

The scheduler does not have one canonical definition of a live agent. Agent
lifecycle is inferred from free-form `current_status` text in some places,
exact `current_status = 'running'` in others, and `ended_at` in still others.
Tmux liveness is also checked through a loose target helper that prefers a
stored pane id without proving that the pane still belongs to the recorded
session/window/agent.

That allows these states to accumulate:

- an agent row still looks active while its tmux window/pane is gone;
- an assignment message stays queued because no live pane received it;
- later scheduler idle checks treat the same row as actionable idle work;
- auditor prompts are repeatedly spawned for stale rows instead of retiring the
  stale row and reassigning the lane once.

## Evidence

Database evidence before the concurrent cleanup:

- `bug_reports#4` had `28` occurrences for
  `harness/idle-alert-missing-window-undelivered-assignment`.
- `bug_reports#6` had `44` related occurrences for the auditor spawn storm.
- Snapshot during this investigation: `281` rows had `ended_at IS NULL`, but
  only `14` tmux panes existed.
- In that snapshot, inferred active rows had `223` missing tmux targets, exact
  `running` rows had `17` missing tmux targets, and idle candidates remained
  large enough to keep alerting.
- Current harness unit tests also fail
  `test_freeform_live_status_counts_as_running`, proving the active-agent
  predicate mismatch.

Concurrent mitigation has since retired many stale rows. Event `93575` records
`272` non-live rows retired by the manhole manager, and events `93599..93607`
show replacement developer spawn requests. This is useful mitigation, but it
needs to become a scheduler invariant.

## Structural Refactor

Implement one live-agent boundary and route every scheduler decision through it.

1. Add a single liveness abstraction, either in `llm_harness/db.py` plus
   `llm_harness/scheduler.py` helpers or a small `llm_harness/agents.py` module:
   - `is_terminal_status(status)`;
   - `is_active_agent(row)`: `ended_at IS NULL` and status is not terminal;
   - `probe_tmux_agent(row, tmux)`: live only when recorded session, window, and
     pane all match the current tmux pane table.

2. Run `reconcile_agent_liveness(conn)` at the beginning of every scheduler
   tick, before `handle_spawn_requests`, `ensure_team`, idle checks, manager
   prompts, and poke delivery:
   - retire rows with missing or mismatched tmux backing;
   - set `ended_at`;
   - log one `agent_missing` / `agent_retired` event;
   - mark direct queued messages for that stale agent as failed or superseded;
   - leave worktrees intact and requeue only clearly owned lanes.

3. Replace raw active queries:
   - `db.list_agents(conn, "running")`;
   - `SELECT ... current_status = 'running'`;
   - direct target queries in `poke()`;
   with the canonical active/live helpers.

4. Make prompt delivery liveness-aware:
   - `poke()` should never send to a row whose session/window/pane do not match;
   - direct messages to non-live targets should become `failed_no_live_target`
     instead of remaining silently `queued`;
   - broadcast delivery should report delivered count plus skipped stale count.

5. Add per-alert dedupe/throttle:
   - key idle alerts by target agent set or per-agent name plus last_seen value;
   - do not spawn a new auditor when the same stale target has already been
     alerted and no heartbeat changed;
   - de-dupe `test-loop` architect spawn requests for the same open bug title.

## Required Tests

Focused `.harness/tests` should prove:

- ended, failed, stopped, and retired rows are excluded from idle candidates;
- free-form active statuses count as active when their tmux target is live;
- a stored pane id that exists under a different window is treated as stale;
- missing-window/pane rows are retired before idle-alert auditor spawning;
- direct `poke()` to a stale target does not remain queued indefinitely;
- repeated ticks for the same stale target do not spawn duplicate auditors;
- one genuine live idle lane owner still produces one alert.

Acceptance should include before/after candidate counts:

- missing-live active rows: nonzero before, `0` after reconciliation;
- repeated stale same-target auditor starts over two ticks: `0`;
- direct assignment messages to stale rows: `failed_no_live_target` or
  `superseded`, not `queued`;
- `python -m unittest discover -s .harness/tests -v` passes.

No compiler/runtime edits are needed for this control-plane fix.
