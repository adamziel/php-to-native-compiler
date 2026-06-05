# Stale Active-Agent Retirement Candidates

Snapshot: 2026-06-05T09:54:19Z

Lane: 121, assigned to `developer-418`.

Scope: read-only scheduler hygiene. No agent rows, messages, lanes, source
files, or worktrees were updated or retired by this report.

## Inputs Checked

- SQLite database: `/home/claude/php-to-native-compiler/.harness/harness.sqlite3`
- Live tmux evidence:
  - `tmux list-windows -a -F '#{session_name}\t#{window_name}\t#{pane_id}'`
  - `tmux list-panes -a -F '#{session_name}\t#{window_name}\t#{pane_id}'`
- SQLite tables: `agents`, `work_lanes`, `messages`, `events`
- No recursive `.harness/worktrees` scan was used.

## Snapshot Counts

- Live tmux windows: 37
- Active agent rows with `ended_at IS NULL`: 19
- Active agent rows missing their recorded tmux window or pane: 1
- `status='in_progress'` lanes checked: 12
- `status='in_progress'` lanes with stale/missing owners: 0
- Queued or in-progress direct messages aimed at stale/ended targets: 3

## Retirement Candidate

| Priority | Agent | Evidence | Still represented by | Current canonical owner |
| --- | --- | --- | --- | --- |
| High | `developer-391` | `agents.ended_at IS NULL`; `current_status='in_progress: lane68 direct FAILED/BORKED triage report'`; recorded tmux target `0:developer-391/%236` has no live window and no live pane. | Queued direct message `#528` from `developer-401` about duplicate zero-test lane142/lane8 deconflict. No current `work_lanes.status='in_progress'` row points at `work/developer-391`. | Lane 68 is currently `in_progress` on live `developer-407`. |

Recommended deterministic action: manager/scheduler can retire or mark
`developer-391` ended after confirming no new live window was spawned, and can
supersede or close queued message `#528` because its content is deconflict
advice for already-superseded zero-test churn rather than a live work owner
assignment.

## Queue Cleanup, Not Row Retirement

| Target | Evidence | Why it is not a live-capacity retirement candidate | Suggested cleanup |
| --- | --- | --- | --- |
| `developer-394` | `ended_at='2026-06-05T09:50:35+00:00'`; recorded tmux target `0:developer-394/%239` is live; queued message `#560` is a stand-down for lane 82. | The row is already ended, and lane 82 is currently owned by live `developer-410`. | Deliver/supersede message `#560` once the stand-down has been observed; do not use this row as active capacity. |
| `developer-402` | `ended_at='2026-06-05T09:50:35+00:00'`; recorded tmux target `0:developer-402/%247` is live; queued message `#562` is a stand-down for lane100. | The row is already ended. There is also a control-plane race in recent events: manager event `#94632` stood down `developer-402` in favor of `developer-406`, while event `#94637` says `developer-402` completed focused verification for lane8/lane100. | Manager/integrator should reconcile lane8/lane100 ownership/completion before further reassignment; message `#562` should not remain a queued work item indefinitely. |

## Active Lane Owner Check

All lanes returned by `SELECT ... FROM work_lanes WHERE status='in_progress'`
had live owner windows and panes at the snapshot:

| Lane | Owner | Status |
| --- | --- | --- |
| 68 | `developer-407` | live |
| 78 | `developer-408` | live |
| 81 | `developer-409` | live |
| 82 | `developer-410` | live |
| 83 | `developer-411` | live |
| 85 | `developer-412` | live |
| 86 | `developer-413` | live |
| 87 | `developer-414` | live |
| 107 | `developer-415` | live |
| 117 | `developer-416` | live |
| 119 | `developer-417` | live |
| 121 | `developer-418` | live |

At this snapshot, lane8 and lane100 no longer appeared in the
`status='in_progress'` lane set. Recent events and message text still reference
lane100 ownership handoff between `developer-402` and `developer-406`; that is
a queue/integration reconciliation issue, not a missing-window active-row
retirement candidate in this report.

## Queries Used

Representative SQLite checks:

```sql
SELECT *
FROM agents
WHERE ended_at IS NULL
ORDER BY id;

SELECT id, title, status, branch, worktree
FROM work_lanes
WHERE status = 'in_progress'
ORDER BY id;

SELECT id, ts, target, status, message
FROM messages
WHERE target != 'broadcast'
  AND status IN ('queued', 'in_progress')
ORDER BY id;
```

The tmux window/pane sets were compared against each agent's recorded
`tmux_session`, `tmux_window`, and `tmux_pane`.
