# Manager-6 Queue Hygiene and Integration Handoff Audit

Author: developer-115
Lane/card: 89 - Manager-6 queue hygiene and integration handoff audit
Snapshot: 2026-06-07T18:26Z harness SQLite state
Scope: read-only SQLite/report audit; no compiler/runtime/product source edits; no full PHPT gate; no public score movement.

## Summary

Current live capacity is small and developer-only: 6 live Developers and 1 live
Coordinator, with no live Integrator rows. Non-null active ownership is not
duplicated among live agents, but there are stale active/orphan rows and stale
message rows that can mislead scheduling after restart.

Highest-priority cleanup items:

1. `agent_messages` has two queued undelivered rows, ids 633 and 635, whose
   matching `messages` rows are already `cancelled`; both targets are non-live.
2. Twelve `worklanes` rows remain `in_progress/planned` with
   `owner_agent_id IS NULL` while retaining old `branch_name` and
   `worktree_path` values.
3. Integration has no live owner: one lane is `ready_for_integration`, eight
   lanes are `integration_failed`, and all shown integration owners are stopped
   or crashed.

## Current Live Assignment Map

Live agents at snapshot:

- Coordinator: `coordinator-25`.
- Developers: `developer-112`, `developer-113`, `developer-114`,
  `developer-115`, `developer-116`, `developer-117`.
- Integrators: none.

Current non-null active lane ownership:

- `developer-112`: lane 1808, `Fix global test suite failures`.
- `coordinator-25`: lane 1549, `Respond to scheduler alert`.
- `developer-113`: lane 1796, `Resolve card 1795 integration conflict for developer-92 str_replace slice`.
- `developer-114`: lane 133, `Self-selected COW runtime lvalue handle gap slice`.
- `developer-115`: lane 89, this audit.
- `developer-116`: lane 135, `Quarantine self-selected product slice by developer-379`.
- `developer-117`: lane 91, `Replay standard array/string accepted-vs-candidate PHPT samples`.

No non-null active owner has more than one active assigned/in-progress lane.

## Queue Hygiene Findings

`messages` currently has no `queued` rows. The stale queue risk is in
`agent_messages`:

- `agent_messages.id=633`, target `developer-45`, status `queued`,
  matching `messages.id=633` status `cancelled`; target is `stopped` with
  `ended_at=2026-06-07T06:09:14+00:00`.
- `agent_messages.id=635`, target `coordinator-22`, status `queued`,
  matching `messages.id=635` status `cancelled`; target is `crash` with
  `ended_at=2026-06-07T18:19:49+00:00`.

Additional non-terminal `messages` rows worth terminal cleanup:

- `messages.id=68` and `messages.id=86` are `delivered_unprocessed` to
  ended agents `developer-82` and `developer-88`.
- `messages.id=469` is `sent` to missing `developer-306`.
- `messages.id=490` is `read` to missing `developer-327`.

Recommended deterministic cleanup: mark `agent_messages` 633/635 terminal
(`cancelled` or `undeliverable`) to match `messages`, and mark stale
non-terminal `messages` 68/86/469/490 terminal or explicitly superseded.

## Orphan Active Rows

The following rows are active from the scheduler's point of view but have
`owner_agent_id IS NULL`, stale branch/worktree fields, and no structured
`agent_reports` in this DB snapshot:

- 106: `Live capacity and assignment map after M0/M1 deconflict`
  (`work/developer-423`).
- 107: `Blocked 221205Z artifact/source path map` (`work/developer-415`).
- 110: `Open bug report to active lane crosswalk` (`work/developer-424`).
- 113: `Focused replay cookbook cross-check` (`work/developer-413`).
- 115: `PHPT gate blocker status board` (`work/developer-407`).
- 116: `Progress dashboard input audit` (`work/developer-428`).
- 120: `Post-7f61915a integration state and dirty-overlap map`
  (`work/developer-405`).
- 127: `Active lane artifact missing-file check` (`work/developer-417`).
- 131: `Integrator handoff queue for report-only artifacts`
  (`work/developer-429`).
- 132: `Manager spawn/load mitigation status report` (`work/developer-406`).
- 149: `Harness restart durability source-path audit`
  (`work/developer-435`).
- 154: `Current-run live capacity assignment and throttle map`
  (`work/developer-439`).

These should not be treated as owned live work. Reassign them to live agents,
mark them queued, or mark them stale/superseded after artifact evidence is
checked. Leaving them `in_progress` with null owners makes active-capacity and
handoff views ambiguous.

## Integration Handoff Risks

There are no live Integrators at snapshot time.

Current integration-stage rows:

- Ready for integration: lane 1788, `Resolve queued lane 1778 runtime merge
  conflict only`, owner `conflict-resolver-7` is stopped. Its report says an
  integrator can merge `work/developer-94`.
- Integration failed: lanes 87, 1769, 1733, 1595, 66, 68, 1533, and 11.
  Owners are null, stopped, or crashed. The conflict files include
  `runtime/src/lib.rs`, `compiler/src/interpreter.rs`, `docs/PROGRESS.md`,
  `docs/SUPPORT.md`, `docs/ARCHITECTURE.md`, and related tests depending on
  lane.

Recommended deterministic handoff: assign or spawn Integrator capacity before
moving more report/source branches into integration. The first handoff target
is lane 1788 because it is already `ready_for_integration`; the eight failed
integration rows need explicit conflict-resolver/requeue decisions instead of
silent active backlog.

## Queries Used

Key SQLite queries run through MCP memory tools:

```sql
SELECT status, stage, COUNT(*) AS n
FROM worklanes
GROUP BY status, stage
ORDER BY status, stage;

SELECT role, COUNT(*) AS live_count
FROM agents
WHERE ended_at IS NULL
GROUP BY role
ORDER BY role;

SELECT w.id, w.title, w.status, w.stage, a.name AS owner, a.role,
       a.current_status, w.branch_name, w.worktree_path,
       w.assigned_at, w.last_activity_at
FROM worklanes w
LEFT JOIN agents a ON a.id=w.owner_agent_id
WHERE w.owner_agent_id IS NOT NULL
  AND w.status IN ('assigned','in_progress','ready_for_integration')
  AND w.stage NOT IN ('done')
ORDER BY w.priority, w.id;

SELECT w.owner_agent_id, a.name, a.current_status,
       COUNT(*) AS active_lane_count,
       GROUP_CONCAT(w.id || ':' || w.title || '[' || w.status || '/' ||
                    w.stage || ']', ' | ') AS lanes
FROM worklanes w
LEFT JOIN agents a ON a.id=w.owner_agent_id
WHERE w.status IN ('assigned','in_progress','ready_for_integration')
  AND w.stage NOT IN ('done')
GROUP BY w.owner_agent_id
HAVING COUNT(*) > 1
ORDER BY active_lane_count DESC, a.name;

SELECT am.id, am.target, am.status AS agent_message_status,
       m.status AS messages_status, am.created_at AS agent_created_at,
       m.ts AS message_ts, a.current_status, a.ended_at
FROM agent_messages am
LEFT JOIN messages m ON m.id=am.id
LEFT JOIN agents a ON a.name=am.target
WHERE am.status='queued'
ORDER BY am.id;

SELECT w.id, w.title, w.status, w.stage, w.branch_name, w.worktree_path,
       w.last_activity_at, w.assigned_at
FROM worklanes w
WHERE w.owner_agent_id IS NULL
  AND w.status IN ('assigned','in_progress','ready_for_integration')
ORDER BY w.id;

SELECT w.id, w.title, w.status, w.stage, a.name AS owner,
       a.current_status, w.branch_name, w.ready_for_integration_at,
       w.review_ready_at, w.reviewed_at
FROM worklanes w
LEFT JOIN agents a ON a.id=w.owner_agent_id
WHERE w.stage='integration'
   OR w.status IN ('ready_for_integration','integration_failed')
ORDER BY COALESCE(w.ready_for_integration_at, w.reviewed_at,
                  w.last_activity_at, w.assigned_at, w.created_at) DESC;
```

No test commands were run because this lane is a read-only coordination audit.
