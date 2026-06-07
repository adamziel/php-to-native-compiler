# Replay SPL/Reflection Accepted-vs-Candidate PHPT Samples

| Field | Value |
| --- | --- |
| Title | Replay SPL/reflection accepted-vs-candidate PHPT samples |
| Owner | developer-109 |
| Lane | worklanes#92, Replay SPL/reflection accepted-vs-candidate PHPT samples |
| Mode | read-only superseded replay reconciliation |
| Created | 2026-06-07T18:20:20Z |
| Branch/worktree | `work/developer-109` / `/home/claude/php-to-native-compiler/.harness/worktrees/developer-109` |
| Source edits | none; report-only artifact under `.harness/reports/` |
| Full gate run | no |
| Public score movement | none |

## Decision

Do not run a combined SPL/reflection replay for lane 92. The lane's current
notes explicitly say this scope was superseded by manager reconciliation:
lanes 72, 82, and 83 already own the SPL/reflection replay sampling split.

Current harness state confirms that:

| Lane | Status | Evidence |
| --- | --- | --- |
| 72 | `done` / `stale` | Notes say the combined reflection/SPL replay scope was superseded to avoid duplication. |
| 82 | `done` / `integrated` | SPL focused replay artifact is present as `.harness/reports/focused-replay-spl-dev109.md`. |
| 83 | `done` / `integrated` | Reflection focused replay artifact is present as `.harness/reports/focused-replay-reflection-dev110.md`. |
| 92 | `development` / `assigned` | This reconciliation report records the no-op closure evidence for the duplicate combined lane. |

The correct disposition for lane 92 is stale/no-source-edits review closure, not
new replay work.

## Evidence Inputs

| Evidence | Path or value |
| --- | --- |
| SPL replay report | `.harness/reports/focused-replay-spl-dev109.md` |
| Reflection replay report | `.harness/reports/focused-replay-reflection-dev110.md` |
| Reflection shard report | `.harness/reports/221205Z-reflection.md` |
| SPL shard report | `.harness/reports/221205Z-spl.md` |
| Binary availability recheck | `.harness/reports/phpt-binary-wrapper-availability-recheck-dev305.md` |
| Harness database | `/home/claude/php-to-native-compiler/.harness/harness.sqlite3` |

Both split replay reports classify their selected rows as pending replay because
the authoritative historical accepted/candidate `PHPC_BIN` pair is missing.
Creating a combined lane-92 replay now would duplicate the same unavailable
replay state without improving evidence.

## Requested Output Status

| Requirement from lane 92 | Status |
| --- | --- |
| Select representative no-SKIPIF SPL/reflection rows | Satisfied by split artifacts: seven SPL rows in lane 82 and eight reflection rows in lane 83. |
| Replay accepted vs candidate | Not run in either split artifact because historical accepted/candidate release binaries are missing. |
| Classify absent rows | Satisfied by split artifacts: both SPL and reflection selected rows are control-plane/pending replay, not proven semantic failures. |
| Output `.harness/reports/replay-spl-reflection.md` | Satisfied by this reconciliation artifact. |
| No source edits/full gate | Satisfied. |

## Commands Used

Harness/lane inspection:

```text
memory_query SELECT id, title, status, stage, owner_agent_id, notes, done_at, reviewed_at, review_ready_at FROM worklanes WHERE id IN (72,82,83,92) ORDER BY id
memory_query SELECT id, card_id, worklane_id, created_at, agent_name, status, stage, substr(report_json,1,1000) AS report FROM agent_reports WHERE card_id IN (72,82,83,92) OR worklane_id IN (72,82,83,92) ORDER BY id DESC LIMIT 20
memory_query SELECT id, ts, type, agent_name, message, substr(payload_json,1,1000) AS payload FROM events WHERE payload_json LIKE '%"worklane_id": 92%' OR message LIKE '%lane 92%' ORDER BY id DESC LIMIT 30
```

Artifact checks:

```sh
test -f .harness/reports/replay-spl-reflection.md
ls -la .harness/reports/replay-spl-reflection.md
sed -n '1,260p' .harness/reports/focused-replay-spl-dev109.md
sed -n '1,260p' .harness/reports/focused-replay-reflection-dev110.md
git status --short --branch
```

No `run-tests.php` command was run. No full PHPT gate was run.

## Artifact Manifest

| Artifact | Purpose | Created by | Hash/check |
| --- | --- | --- | --- |
| `.harness/reports/replay-spl-reflection.md` | Lane 92 superseded replay reconciliation artifact | developer-109 | validate with `git diff --check -- .harness/reports/replay-spl-reflection.md` |

## Proposed Next Action

| Priority | Action | Owner type | Preconditions | Stop condition | Expected artifact |
| ---: | --- | --- | --- | --- | --- |
| 1 | Close lane 92 as stale/superseded with no further replay work | Reviewer/Python stage owner | This report and integrated lanes 82/83 are visible | Lane 92 no longer requeues duplicate SPL/reflection replay work | Stage/status movement only |
| 2 | Restore or rebuild durable accepted/candidate release `phpc` binaries, then run the split SPL and reflection replay selectors from lanes 82 and 83 | Developer or Integrator | Durable `PHPC_BIN` pair for accepted `0b917f67a37d9ca9779d77f87173b628431c2425` and candidate `56fe9377fb46be00db5fdd30c966fdba406dc581` | Accepted replay passes and candidate replay emits row-level statuses | Updated split replay artifacts, not a duplicate combined lane |

## Integration-Ready Checklist

- [x] Report states lane 92 is superseded by lanes 72, 82, and 83.
- [x] Report names the split SPL and reflection artifacts.
- [x] Report records that no full gate or replay was run.
- [x] Report gives deterministic next action for stage closure.
