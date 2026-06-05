# Idle-Alert Ended-Agents Reliability Report

Architect: architect-15
Timestamp: 2026-06-05T10:01:28Z
Scope: harness/idle-alert-ended-agents

## Current Finding

`harness/idle-alert-ended-agents` was reproducible in the live root harness at
the start of this investigation. A later root-level tactical patch by
developer-419 closed the focused `.harness` test reproduction, but it did not
remove the structural control-plane risk.

The current `harness` zipapp now has a broader tactical filter, but it is still
not the reliability refactor described by earlier architect reports:

- `llm_harness/db.py::is_active_agent_status(status)` looks only at status
  text and remains separable from the `ended_at` requirement, so callers can
  still forget half of the active predicate.
- `llm_harness/db.py::list_agents(conn, "running")` now filters
  `ended_at IS NULL` plus non-terminal status text, which fixes the focused
  compatibility test.
- `llm_harness/scheduler.py::check_agent_liveness()` and `prompt_auditor()`
  now include `not agent["ended_at"]`, which fixes the focused ended-row idle
  alert test.
- `llm_harness/scheduler.py::active_agent_count()`,
  `prompt_running_role()`, `prompt_manager()`, and broadcast `poke()` still use
  status text without consistently requiring `ended_at IS NULL`.
- `llm_harness/testing_loop.py::maybe_invoke_architect()` queues an Architect
  request for every repeated open bug on every failed run. It has no durable
  per-test escalation ledger or same-title dedupe.

## Pre-Patch Reproduction

Running the focused harness tests from the repository root at
2026-06-05T10:00Z failed:

```text
python3 -m unittest discover -s .harness/tests -v
Ran 7 tests
FAILED (failures=3)
```

Relevant failures:

- `test_freeform_live_status_counts_as_running`: `db.list_agents(conn,
  "running")` returned `[]` for `current_status="inspecting PHPT lane"`.
- `test_liveness_ignores_ended_running_rows`: an agent with
  `current_status="running"` and non-null `ended_at` still produced an idle
  auditor prompt.
- `test_test_loop_prefers_project_run_tests_script`: the test loop still chose
  `python -m unittest discover -s tests -v` instead of `tools/run-tests.sh`.

The third failure is bug_reports#2 rather than this specific idle-alert test,
but it matters operationally: false test-loop failures keep re-triggering
Architect escalation and obscuring whether lane100 has really fixed the
control-plane path.

## Post-Patch Check

After developer-419's root-level tactical patch, the focused reproduction is
green:

```text
python3 -m unittest discover -s .harness/tests -v
Ran 7 tests
OK
```

The live root selector also now returns the intended project command:

```text
discover_test_command(/home/claude/php-to-native-compiler)
=> ['tools/run-tests.sh']
```

This is useful progress, but it should be treated as a tactical closure of the
known tests, not proof that the harness has one lifecycle model. The remaining
uncovered scheduler paths and Architect-spawn dedupe gap are still structural.

## Live DB Evidence

The current database still has bug_reports#3 open with six occurrences for
`harness/idle-alert-ended-agents`.

The same test node has produced repeated Architect spawn requests:

```text
spawn_requests: 25, 29, 32, 41, 45, 49, 53, 56
title: Find systemic cause for repeated failure: harness/idle-alert-ended-agents
```

Earlier storm evidence in bug_reports#3 records thousands of active-ish stale
rows where tmux windows or panes were already gone. Current cleanup plus the
developer-419 tactical patch reduced the immediate focused symptom, but the
same test node still has no durable alert ledger or repeated-Architect dedupe.

## Deeper Structural Cause

There are two coupled systemic causes.

First, agent lifecycle has no authoritative machine state. `current_status` is
used both as human progress text and as lifecycle state, while `ended_at`, tmux
reachability, and prompt throttles are checked differently at each call site.
Manual retirement fixes one row at a time, but the scheduler can reclassify the
same class of stale rows as active on the next tick.

Second, the harness control-plane code under test is a root-level untracked
zipapp (`harness`), not a visible versioned source tree. Developer worktrees do
not contain that executable. This explains the non-reproducible completion
claims: events and test_runs at 2026-06-05T09:54Z recorded a 7/7 pass and a
`tools/run-tests.sh` selection, but re-running from the root at
2026-06-05T10:00Z failed 3/7 and still selected the wrong command. A later
root-level patch at roughly 2026-06-05T10:03Z made that focused check pass
again. This sequence shows why acceptance must verify the root executable that
the scheduler is actually running, not only a worktree or copied module.

## Reliability Refactor Plan

1. Add an authoritative lifecycle API in `llm_harness.db`.

   Provide `is_terminal_status(status)`, `is_agent_active(row)`,
   `list_active_agents(conn, role=None)`, and `count_active_agents(conn,
   role=None)`. Active must require `ended_at IS NULL` and a non-terminal
   lifecycle status. `list_agents(conn, "running")` should become a
   compatibility wrapper around this active predicate until callers are
   migrated.

2. Separate lifecycle from display status.

   Short term, keep using `ended_at` plus terminal-status compatibility.
   Longer term, migrate the agents table to a distinct lifecycle column such
   as `lifecycle_status` with values like `active`, `crashed`, `stopped`, and
   `completed`, leaving `current_status` as display text only. All lifecycle
   writes should go through `start_agent()`, `heartbeat_agent()`, and
   `end_agent()` helpers.

3. Reconcile active rows before alert selection.

   At the start of every scheduler tick, scan ended-at-null rows through the
   central active predicate. For rows with recorded tmux targets, missing
   window/pane/process backing should mark the row ended before idle candidates
   are computed. The idle-alert path must consume only reconciled active rows.

4. Replace per-agent prompt timestamps with a scheduler alert ledger.

   Add a durable table such as
   `scheduler_alerts(alert_type, target_key, status, first_seen_at,
   last_seen_at, last_sent_at, sent_count, assigned_agent, resolution)`, with
   one open alert per `(alert_type, target_key)`. Resolve idle alerts when the
   target heartbeats, is ended, or is superseded. This stops repeated
   same-target auditor churn while preserving one alert for a genuinely live
   idle worker.

5. Deduplicate repeated Architect escalation.

   Route `testing_loop.maybe_invoke_architect()` through the same alert ledger
   or a `queue_unique_spawn_request()` helper. For an open repeated-failure
   bug, one open `repeated_test_failure:<nodeid>` escalation is enough until
   the bug is fixed or the existing Architect investigation is explicitly
   closed as insufficient.

6. Make harness patches durable and source-addressable.

   Extract the zipapp source into a versioned harness source directory or add
   a deterministic unpack/patch/repack script that records the source hash,
   zipapp hash, and root-level verification command. Acceptance evidence must
   run against `ROOT/harness`, not a developer worktree that lacks the zipapp.

## Required Tests

Add focused `.harness` tests for:

- free-form active statuses count as active/running compatibility;
- `ended_at IS NOT NULL` rows never alert, even when
  `current_status='running'`;
- terminal display statuses with null `ended_at` are normalized or excluded;
- missing tmux window/pane rows are retired before idle-alert generation;
- repeated scheduler ticks for the same stale target produce no duplicate
  auditor spawn;
- one genuinely live idle worker still creates exactly one alert;
- repeated open bug_reports for one test node queue at most one Architect
  escalation while an open escalation exists;
- `discover_test_command(ROOT)` prefers `tools/run-tests.sh` for this Rust/PHP
  repository, so the test loop does not create fresh false failures.

## Acceptance

Immediate tactical acceptance has now passed once from the repository root:

```text
python3 -m unittest discover -s .harness/tests -v
Ran 7 tests
OK
discover_test_command(root) => ['tools/run-tests.sh']
```

Reliability-refactor acceptance should still require all of the following from
the repository root:

```sh
python3 -m unittest discover -s .harness/tests -v
python3 - <<'PY'
from pathlib import Path
import sys
root = Path('/home/claude/php-to-native-compiler')
sys.path.insert(0, str(root / 'harness'))
from llm_harness.testing_loop import discover_test_command
print(discover_test_command(root))
PY
```

The first command must pass with nonzero tests. The second command must print
`['tools/run-tests.sh']`. The final implementation report must also include
before/after counts for active rows, rows excluded by `ended_at`, missing-tmux
rows retired before alerting, idle-alert candidates, and repeated Architect
spawn requests for the same test node.

No compiler/runtime files should be edited for this fix, and no PHPT
compatibility score should be claimed.
