# Live Capacity And Assignment Map - developer-377

Snapshot time: 2026-06-08T13:10:37Z

Scope: work_lanes#106, read-only capacity map after the M0/M1 deconflict wave.
No compiler/runtime/product source files were edited. No full PHPT gate was run.
No public score movement is claimed; the only project score reported here is
accepted_public_phpt_passes / pinned_public_runnable_denominator =
7873 / 20294 = 38.79%.

## Summary

- Live `Developer` rows in SQLite with matching tmux windows/panes: 3.
- Live `Developer` rows with exactly one assigned active lane: 3.
- Live unassigned reserve `Developer` rows: 0.
- Live overloaded `Developer` owners: 0.
- Live `Developer` capacity consumed by non-Developer lane role_type: 1
  (`developer-375` on Conflict Resolver lane 2081).
- Active `Developer` role_type lanes with live owners: 2
  (lane 2075 and this lane 106).
- `Developer` role_type lanes still marked `in_progress` with no owner: 11.
- Queued unowned `Developer` role_type lanes: 27.

The current live Developer capacity is fully allocated. The actionable drift is
not live overload; it is stale or ownerless `in_progress` report lanes plus
queued follow-on placeholder lanes. The only live product/test repair owner is
`developer-376` on lane 2075, the global-suite restoration lane. Avoid
overlapping that owner unless it explicitly hands off a root cause or blocker.

## Live Developer Map

All rows below have `agents.ended_at IS NULL`, `role='Developer'`,
`current_status='running'`, and a matching tmux window/pane in
`llm-harness-php-to-native-compiler`.

| Developer | Tmux | Active lane |
| --- | --- | --- |
| developer-375 | developer-375/%1208 | 2081 - Conflict Resolver: Resolve integration failure for card #105: Lane100 proof evaluator and before/after candidate counts |
| developer-376 | developer-376/%1209 | 2075 - Developer: Fix global test suite failures |
| developer-377 | developer-377/%1210 | 106 - Developer: Live capacity and assignment map after M0/M1 deconflict |

## Live Non-Developer Context

One live Coordinator row was visible during this snapshot:

| Agent | Tmux | Notes |
| --- | --- | --- |
| coordinator-203 | coordinator-203/%1211 | Maintain Coordinator capacity |

No live Integrator rows were present in the `ended_at IS NULL` and
`current_status='running'` snapshot query.

## Unowned In-Progress Developer Rows

These rows remain `role_type='Developer'` and `status='in_progress'`, but have
`owner_agent_id IS NULL`. Their worktree paths point at older developer
worktrees, and none match the current live Developer windows.

| Lane | Branch | Title |
| --- | --- | --- |
| 107 | work/developer-415 | Blocked 221205Z artifact/source path map |
| 110 | work/developer-424 | Open bug report to active lane crosswalk |
| 113 | work/developer-413 | Focused replay cookbook cross-check |
| 115 | work/developer-407 | PHPT gate blocker status board |
| 116 | work/developer-428 | Progress dashboard input audit |
| 120 | work/developer-405 | Post-7f61915a integration state and dirty-overlap map |
| 127 | work/developer-417 | Active lane artifact missing-file check |
| 131 | work/developer-429 | Integrator handoff queue for report-only artifacts |
| 132 | work/developer-406 | Manager spawn/load mitigation status report |
| 149 | work/developer-435 | Harness restart durability source-path audit |
| 154 | work/developer-439 | Current-run live capacity assignment and throttle map |

Recommended deterministic action: a manager or Python scheduler pass should
reconcile these 11 rows against existing artifacts and agent reports before
allocating more Developer capacity. If still needed, reassign each row to a
live Developer with a fresh owner-specific artifact name. If completed,
superseded, or stale, retire the row so it stops inflating active work.

## Queued Developer Rows

SQLite currently shows 27 unowned queued `Developer` role_type lanes. Many are
follow-on capacity placeholders with empty goals or no worktree. The visible
concrete candidates in the first queued slice were:

| Lane | Title | Note |
| --- | --- | --- |
| 1836 | Gate reproducer for php_runtime invocation cleanup failures | Read-only support for card 1808; avoid same-file merge churn. |
| 2006 | Run canonical card 1980 control-plane repair for duplicate Architect request flood | Bounded harness/control-plane repair; no compiler/runtime edits authorized in notes. |
| 2072 | card2027 inventory: no-fail-fast failing target map | Inventory lane only; explicitly avoid developer-362 worktree/native_arithmetic_boundary files. |

The remaining queued rows in the sampled set were mostly "assign concrete
Developer card after capacity placeholder" follow-ups. They should not be
treated as product progress until rewritten or assigned with explicit goal,
acceptance criteria, allowed scope, deconflict notes, and focused verification.

## Duplicate And Overload Check

- Duplicate active owner rows: none found among assigned/in-progress rows.
- Live Developers with more than one assigned active lane: none.
- Live Developers with zero assigned active lanes: none.
- Cross-role capacity caveat: `developer-375` is a live Developer agent working
  a `Conflict Resolver` role_type lane, so Developer capacity is partly consumed
  by integration repair rather than direct Developer-role work.
- Current lane 106 requeue cause: events show `developer-373` was assigned
  lane 106 after filing superseded report675 for duplicate lane2074, then
  lane106 was requeued when `developer-373`'s tmux pane disappeared. No report
  exists for card/worklane 106 before this artifact.

## Resource Snapshot

Latest resource samples around this snapshot showed moderate load and ample
disk:

| Sample | CPU | RAM | Load1 | Disk free |
| --- | --- | --- | --- | --- |
| 2026-06-08T13:10:17Z | 19.2% | 18.75% | 3.84 | 153.9 GB |
| 2026-06-08T13:10:07Z | 19.57% | 19.15% | 3.91 | 153.89 GB |
| 2026-06-08T13:09:59Z | 20.83% | 18.66% | 4.17 | 153.96 GB |

Capacity pressure is primarily scheduling-state drift, not current CPU, RAM, or
disk pressure.

## Evidence Commands

MCP memory queries were used first; no Python sqlite fallback was needed. Local
tmux checks were read-only.

Commands used:

```sh
date -u +%Y-%m-%dT%H:%M:%SZ
git rev-parse HEAD
tmux list-sessions -F '#{session_name}'
tmux list-windows -a -F '#{session_name}:#{window_index}:#{window_name}:#{window_active}'
tmux list-panes -a -F '#{session_name}:#{window_name}:#{pane_id}:#{pane_current_command}'
rg --files .harness/reports | rg 'live-capacity|capacity|assignment-map'
git status --short --branch
```

SQLite/MCP queries used:

```sql
SELECT role, current_status, COUNT(*) AS n
FROM agents
GROUP BY role, current_status
ORDER BY role, current_status;

SELECT role_type, status, stage, COUNT(*) AS n
FROM worklanes
GROUP BY role_type, status, stage
ORDER BY role_type, status, stage;

SELECT name, role, current_status, tmux_session, tmux_window, tmux_pane,
       started_at, last_seen_at, last_prompt_at, ended_at, notes
FROM agents
WHERE ended_at IS NULL AND current_status='running'
ORDER BY role, name;

SELECT a.name, a.current_status, COUNT(wl.id) AS assigned_lane_count,
       group_concat(wl.id || ':' || wl.status || '/' || wl.stage || ':' || wl.title, ' | ') AS lanes
FROM agents a
LEFT JOIN worklanes wl ON wl.owner_agent_id = a.id
WHERE a.role='Developer' AND a.ended_at IS NULL
GROUP BY a.id
ORDER BY a.name;

SELECT wl.id, wl.title, wl.role_type, wl.status, wl.stage, wl.owner_agent_id,
       a.name AS owner_name, a.role AS owner_role, a.current_status AS owner_status,
       a.ended_at AS owner_ended_at, wl.branch_name, wl.worktree_path,
       wl.priority, wl.source_key, substr(wl.notes,1,1000) AS notes
FROM worklanes wl
LEFT JOIN agents a ON a.id = wl.owner_agent_id
WHERE wl.status IN ('assigned','in_progress')
ORDER BY wl.role_type, wl.status, wl.priority, wl.id;

SELECT wl.owner_agent_id, a.name AS owner_name, COUNT(*) AS lane_count,
       group_concat(wl.id || ':' || wl.status || '/' || wl.stage, ', ') AS lanes
FROM worklanes wl
LEFT JOIN agents a ON a.id = wl.owner_agent_id
WHERE wl.owner_agent_id IS NOT NULL
  AND wl.status IN ('assigned','in_progress','queued','planned')
GROUP BY wl.owner_agent_id
HAVING COUNT(*) > 1
ORDER BY lane_count DESC;

SELECT status, COUNT(*) AS n
FROM spawn_requests
WHERE role='Developer'
GROUP BY status
ORDER BY status;

SELECT *
FROM resource_samples
ORDER BY id DESC
LIMIT 10;
```

## Verification

Verification performed for this report artifact:

```sh
git diff --check -- .harness/reports/live-capacity-assignment-map-dev377.md
git status --short --branch
```

Result: passed. `git diff --check` emitted no output, and `git status` showed
only the new report artifact before commit.
