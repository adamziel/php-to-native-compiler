# Live Capacity And Assignment Map - developer-423

Snapshot time: 2026-06-05T10:01:39Z

Scope: work_lanes#106, read-only capacity map after the M0/M1 deconflict wave.
No compiler/runtime/product source files were edited. No full PHPT gate was run.
No public score movement is claimed.

## Summary

- Live Developer rows in SQLite with matching tmux windows/panes: 15.
- Live Developers with exactly one in-progress lane: 15.
- Live unassigned reserve Developers: 0.
- Live overloaded Developer owners: 0.
- In-progress Developer work_lanes rows: 23.
- In-progress Developer lanes with no matching live Developer row in this
  snapshot: 8.
- Queued Developer lanes: 0.

The current live capacity is fully allocated. The main drift risk is not live
overload; it is stale `in_progress` lane rows that still point at older
developer worktrees with no live Developer row or tmux window in session `0`.

## Live Developer Map

All rows below have `agents.ended_at IS NULL`, `role='Developer'`,
`current_status='running'`, and a matching tmux session `0` window/pane.

| Developer | Tmux | Active lane |
| --- | --- | --- |
| developer-419 | developer-419/%274 | 143 - Fix failing tests from run 215 |
| developer-420 | developer-420/%275 | 73 - Completed-lane artifact/status reconciliation |
| developer-421 | developer-421/%276 | 138 - Quarantine self-selected product slice by developer-389 |
| developer-422 | developer-422/%277 | 95 - Disk and build-target guardrail audit |
| developer-423 | developer-423/%278 | 106 - Live capacity and assignment map after M0/M1 deconflict |
| developer-424 | developer-424/%279 | 110 - Open bug report to active lane crosswalk |
| developer-425 | developer-425/%280 | 89 - Manager-6 queue hygiene and integration handoff audit |
| developer-426 | developer-426/%281 | 112 - Deferred source-lane dirty-overlap report |
| developer-427 | developer-427/%282 | 114 - Stand down self-selected database slice; quarantine report |
| developer-428 | developer-428/%283 | 116 - Progress dashboard input audit |
| developer-429 | developer-429/%284 | 131 - Integrator handoff queue for report-only artifacts |
| developer-430 | developer-430/%285 | 111 - COW/runtime source-lane prerequisite audit only |
| developer-431 | developer-431/%286 | 144 - Run 215 post-fix recurrence timestamp and selector evidence audit |
| developer-432 | developer-432/%287 | 145 - Lane8/lane100 bug-status closure recommendation |
| developer-433 | developer-433/%288 | 146 - Zero-regression gate preflight command checklist |

## Stale In-Progress Lane Rows

These Developer lanes remain `in_progress` in SQLite, but their branch/worktree
does not match any live Developer row and the tmux window list for session `0`
does not include the corresponding developer window.

| Lane | Assigned branch | Title |
| --- | --- | --- |
| 74 | work/developer-416 | First repair backlog refinement from evidence reports |
| 83 | work/developer-411 | Focused replay: reflection regression rows |
| 107 | work/developer-415 | Blocked 221205Z artifact/source path map |
| 113 | work/developer-413 | Focused replay cookbook cross-check |
| 115 | work/developer-407 | PHPT gate blocker status board |
| 120 | work/developer-405 | Post-7f61915a integration state and dirty-overlap map |
| 127 | work/developer-417 | Active lane artifact missing-file check |
| 132 | work/developer-406 | Manager spawn/load mitigation status report |

Recommended deterministic action: a manager or integrator should reconcile
these eight lane rows against any completed artifacts/events before assigning
new capacity. If they still need work, reassign them to a live Developer with a
fresh artifact name; if completed or superseded, update their status so they do
not inflate active work.

## Duplicate And Overload Check

- Duplicate active owner rows by `(branch, worktree)`: none.
- Live Developers with more than one active lane: none.
- Live Developers with zero active lanes: none.
- Duplicate lane106 claim residue is already deconflicted:
  - events#94735: developer-426 stood down from lane106 after manager-24
    assigned lane106 to developer-423 and assigned developer-426 to lane112.
  - events#94734: developer-431 stood down from lane106 after SQLite showed
    manager-24 assigned lane106 to developer-423.
  - events#94739: developer-423 acknowledged manager-24 lane106 assignment.

## Evidence Commands

MCP memory queries were used first. A concurrent write caused a transient
`database is locked` failure during aggregate lane queries, so the same
read-only SQLite queries were rerun through Python sqlite3 with
`mode=ro`. No direct database writes were made.

Commands and queries used:

```sh
tmux list-windows -t 0 -F '#{window_index}\t#{window_name}\t#{window_panes}'
tmux list-panes -a -F '#{session_name}\t#{window_name}\t#{pane_id}\t#{pane_current_command}'
```

```sql
SELECT role, current_status, COUNT(*) AS count
FROM agents
WHERE ended_at IS NULL
GROUP BY role, current_status
ORDER BY role, current_status;

SELECT name, branch, current_status, last_seen_at, tmux_window, tmux_pane, worktree
FROM agents
WHERE role='Developer' AND ended_at IS NULL
ORDER BY name;

SELECT id, title, status, branch, worktree, notes
FROM work_lanes
WHERE role='Developer' AND status='in_progress'
ORDER BY id;

WITH live_devs AS (
  SELECT name, branch, worktree FROM agents
  WHERE role='Developer' AND ended_at IS NULL
), active_lanes AS (
  SELECT id, title, branch, worktree FROM work_lanes
  WHERE role='Developer' AND status='in_progress'
)
SELECT d.name, d.branch, COUNT(l.id) AS active_lane_count,
       GROUP_CONCAT(l.id || ':' || l.title, ' | ') AS lanes
FROM live_devs d
LEFT JOIN active_lanes l ON d.branch = l.branch OR d.worktree = l.worktree
GROUP BY d.name, d.branch
ORDER BY d.name;

WITH live_devs AS (
  SELECT name, branch, worktree FROM agents
  WHERE role='Developer' AND ended_at IS NULL
), active_lanes AS (
  SELECT id, title, branch, worktree FROM work_lanes
  WHERE role='Developer' AND status='in_progress'
)
SELECT l.id, l.title, l.branch, l.worktree
FROM active_lanes l
LEFT JOIN live_devs d ON d.branch = l.branch OR d.worktree = l.worktree
WHERE d.name IS NULL
ORDER BY l.id;

WITH active_lanes AS (
  SELECT id, title, branch, worktree FROM work_lanes
  WHERE role='Developer' AND status='in_progress'
)
SELECT branch, worktree, COUNT(*) AS lane_count,
       GROUP_CONCAT(id || ':' || title, ' | ') AS lanes
FROM active_lanes
GROUP BY branch, worktree
HAVING COUNT(*) > 1
ORDER BY lane_count DESC, branch;

SELECT status, COUNT(*) AS count
FROM work_lanes
WHERE role='Developer'
GROUP BY status
ORDER BY status;
```

## Verification

Planned verification for this report artifact:

```sh
git diff --check -- .harness/reports/live-capacity-assignment-map-dev423.md
```
