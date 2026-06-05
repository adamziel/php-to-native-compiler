# Completed-Lane Artifact/Status Reconciliation

Developer: developer-420
Lane: 73
Generated: 2026-06-05T10:00Z
Worktree: `/home/claude/php-to-native-compiler/.harness/worktrees/developer-420`
Branch: `work/developer-420`

Scope: read-only reconciliation of completed/report-only branches, message
statuses, lane statuses, and stale active rows. No compiler/runtime/product
source files were edited. No branch merges were performed. No full PHPT gate,
Cargo build, or public score movement is claimed.

## Summary

The current lane/status truth is split across committed report artifacts,
completed but not integrated branch artifacts, and stale `in_progress` rows
whose owners were stopped by the latest harness restart.

Important current state:

- `work_lanes#73` is now assigned to `work/developer-420`; the manager-24 note
  names this artifact as
  `.harness/reports/completed-lane-status-reconciliation-dev420.md`.
- The launch prompt still only said `Maintain Developer capacity`; the lane was
  discovered from `work_lanes`, confirming the existing queued-message delivery
  audit finding that running workers must inspect SQLite state.
- Current branch `HEAD` is `8381ad999b89`; local `origin/master` is
  `4d95c3df99a7`.
- Public PHPT score remains unchanged: accepted `7873 / 20294 = 38.79%`;
  blocked 221205Z candidate remains `7197 / 20294 = 35.46%` with `1166`
  PASS regressions.

## Lane Counts

Observed `work_lanes.status` counts:

| Status | Rows |
| --- | ---: |
| `completed` | 17 |
| `completed_no_proof_ready` | 1 |
| `deferred_m0_m1_priority` | 1 |
| `in_progress` | 23 |
| `integrated` | 46 |
| `superseded` | 51 |
| `superseded_by_lane100_owner_conflict` | 1 |
| `superseded_by_lane8_duplicate_zero_test` | 4 |
| `superseded_by_manager20_m0_m1_priority` | 1 |
| `superseded_quarantined_self_selected_product_lane` | 1 |

## Completed But Not Integrated

These lanes are marked `completed` or `completed_no_proof_ready`, but their
report artifacts are not present in this worktree or `origin/master`. Each
artifact is available on the recorded `origin/work/*` branch, so integrators
can path-limit import without merging source changes.

| Lane | Status | Branch | Commit | Artifact |
| ---: | --- | --- | --- | --- |
| 68 | `completed` | `origin/work/developer-407` | `e5362cf0bff4` | `.harness/reports/221205Z-direct-failed-borked-triage.md` |
| 78 | `completed` | `origin/work/developer-408` | `418c063d62df` | `.harness/reports/221205Z-shard-rerun-smoke-dev116.md` |
| 81 | `completed` | `origin/work/developer-409` | `0e175a31f63a` | `.harness/reports/focused-replay-standard-filesystem-http-dev108.md` |
| 82 | `completed` | `origin/work/developer-394` | `eba7e3e745a8` | `.harness/reports/focused-replay-spl-dev109.md` |
| 85 | `completed` | `origin/work/developer-412` | `6cf81ac1b9e8` | `.harness/reports/focused-replay-secondary-ext-dev112.md` |
| 86 | `completed` | `origin/work/developer-413` | `437eeff73a40` | `.harness/reports/focused-replay-standard-scalar-misc-dev117.md` |
| 87 | `completed` | `origin/work/developer-414` | `500ba65d0fad` | `.harness/reports/absent-row-rerun-prioritizer-dev118.md` |
| 117 | `completed` | `origin/work/developer-416` | `feb4a15cc5b9` | `.harness/reports/first-repair-lane-evidence-readiness-dev236.md` |
| 119 | `completed_no_proof_ready` | `origin/work/developer-417` | `08bd962ad1e0` | `.harness/reports/lane8-lane100-proof-evaluator-dev302.md` |
| 121 | `completed` | `origin/work/developer-418` | `8affc931cd6b` | `.harness/reports/stale-active-agent-retirement-candidates-dev304.md` |

Lane 119 should remain labeled carefully: its artifact is useful, but the lane
status explicitly says no proof was ready.

## Integrated Artifact Caveat

Most `integrated` report artifacts referenced in lane notes are present in the
current worktree. The only integrated-row mismatch found by artifact-path
extraction is lane 88:

| Lane | Status | Missing Path | Note |
| ---: | --- | --- | --- |
| 88 | `integrated` | `.harness/reports/full-gate-restart-checklist.md` | Current branch contains `.harness/reports/full-gate-readiness-after-shard-fix-dev119.md`; older notes also mention `full-gate-restart-checklist.md`, which is not present. |

This matches the earlier branch-map note that `full-gate-restart-checklist.md`
was referenced by superseded/older lane text but absent from the integrated
report set.

## Stale In-Progress Rows

These `in_progress` lanes point to owner agents whose rows now have
`ended_at` set and `current_status='stopped'`. Their expected artifacts were
not present in this worktree. Local `origin/work/*` refs either do not exist or
do not contain the expected artifacts, so these should be requeued, reassigned,
or explicitly marked superseded instead of treated as live work.

| Lane | Owner Branch | Expected Artifact State |
| ---: | --- | --- |
| 74 | `work/developer-416` | No `first-repair-backlog-refined*` artifact on current branch or `origin/work/developer-416`. |
| 83 | `work/developer-411` | No local `origin/work/developer-411` ref for `focused-replay-reflection-dev110.md`. |
| 107 | `work/developer-415` | No local `origin/work/developer-415` ref for `221205Z-artifact-source-map-dev228.md`. |
| 113 | `work/developer-413` | `origin/work/developer-413` exists, but no focused-replay cookbook cross-check artifact. |
| 115 | `work/developer-407` | `origin/work/developer-407` exists for lane 68 only; no `phpt-gate-blocker-status*` artifact. |
| 120 | `work/developer-405` | No local `origin/work/developer-405` ref for post-integration-state artifacts. |
| 127 | `work/developer-417` | `origin/work/developer-417` exists for lane 119 only; no active-lane artifact-missing check report. |
| 132 | `work/developer-406` | `origin/work/developer-406` exists at `8381ad999b89`; no manager spawn/load mitigation report. |

## Live In-Progress Rows

The following `in_progress` lanes have live owner rows and missing expected
artifacts, which is normal for active work. They should not be integrated until
their owners record completion evidence:

- 89 `work/developer-425`
- 95 `work/developer-422`
- 106 `work/developer-423`
- 110 `work/developer-424`
- 111 `work/developer-430`
- 112 `work/developer-426`
- 114 `work/developer-427`
- 116 `work/developer-428`
- 131 `work/developer-429`
- 138 `work/developer-421`
- 143 `work/developer-419`
- 144 `work/developer-431`
- 145 `work/developer-432`
- 146 `work/developer-433`

Lane 73 itself is live on `developer-420` and is satisfied by this report.

## Message Status

Current messages still have mixed free-form statuses. For `messages.id >= 439`,
the observed statuses included `queued`, `cancelled`, `delivered`, `completed`,
`processed_by_*`, `superseded_by_*`, `acknowledged`, `done`, `read`, and
`sent`. The latest snapshot still had `15` queued messages in that window.

Operational consequence: `work_lanes` plus `agents` rows are currently more
reliable for ownership than `messages.status` alone. This report used
`work_lanes.branch/worktree`, owner `agents.ended_at`, and artifact blob
existence as the reconciliation source of truth.

## Deconflict Note

During startup, `developer-420` briefly claimed lane 116 before a fresh
ownership query showed lane 116 assigned to `work/developer-428`. That claim
was superseded by a `lane_standdown` event, and no lane 116 artifact was
created from this worktree.

## Recommended Integrator Actions

1. Path-limit import the completed-but-not-integrated report artifacts from
   lanes 68, 78, 81, 82, 85, 86, 87, 117, 119, and 121.
2. Requeue, reassign, or mark superseded stale `in_progress` lanes 74, 83,
   107, 113, 115, 120, 127, and 132.
3. Keep live `in_progress` lanes out of completed artifact queues until their
   owner events and artifact blobs exist.
4. Continue treating `work_lanes`/`agents` as authoritative until message
   status values are normalized.

## Commands And Queries

No full PHPT gate, Cargo test, Cargo check, or merge command was run.

MCP SQLite calls were attempted first. During one window the MCP database
handle returned `database is locked`, so read-only SQLite fallback queries used
Python stdlib with `mode=ro` and `busy_timeout=20000`.

Key commands:

```sh
sed -n '1,260p' /home/claude/php-to-native-compiler/.harness/prompts/developer-420.md
git status --short --branch
git rev-parse --short=12 HEAD
git rev-parse --short=12 origin/master
find .harness/reports -maxdepth 1 -type f -name '*.md' -printf '%f\n' | sort
```

Read-only SQLite queries inspected:

- `work_lanes` status counts
- `work_lanes#73` notes
- `work_lanes` artifact paths parsed from notes
- owner `agents.current_status`, `agents.ended_at`, and `agents.last_seen_at`
- recent lane completion and reassignment events
- `messages.id >= 439` status counts
- latest metric samples and public metric events

Artifact availability checks used:

```sh
git cat-file -e origin/work/<developer>:.harness/reports/<artifact>.md
git cat-file -e origin/master:.harness/reports/<artifact>.md
```

Constraint note: one early scouting command used a shallow `find` over
`/home/claude/php-to-native-compiler/.harness -maxdepth 3` and crossed into
top-level `.harness/worktrees` entries. No findings in this report rely on
that output; the reconciliation data above uses pruned/current report-dir
listing, SQLite rows, and Git blob existence checks.
