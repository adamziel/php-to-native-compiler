# Disk and Build-Target Guardrail Audit

Developer: developer-422
Lane: 95
Timestamp: 2026-06-05T10:00Z
Scope: read-only ops report; no compiler/runtime edits, no deletion, no tests,
no full PHPT gate, no public score movement.

## Summary

Root disk pressure is still high even after the earlier Janitor activity:
`df -h /` reports `459G` total, `419G` used, `21G` available, and `96%` use.
That is above the 20 GB soft floor used by the earlier resource-throttle
report, but the margin is small. The root Cargo target alone is `22G`, so one
accidental default-target build can consume the whole safety margin.

Current CPU load is lower than the earlier storm snapshot:
`uptime` reported load averages `8.01, 13.54, 13.23` on a 20-core host.
Process shape is still agent-heavy: `ps` showed 20 `codex`, 20 `node`, and
20 `python3` processes, with no `cargo`, `rustc`, `cc`, or `clang` process
matched by direct `pgrep` checks at report time.

No cleanup was performed. The safe next action is a manager/integrator-owned
cleanup pass with an explicit candidate list, active-process exclusions, and a
recorded deletion plan.

## Coordination Note

developer-422 initially recorded an event-only claim for lane110 before
manager-24 assigned lane95 to this worktree. After reading current
`work_lanes`, developer-422 stood down from lane110 so developer-424 can own
the open-bug crosswalk. This report follows the later lane95 assignment:
`.harness/reports/disk-build-target-guardrails-dev422.md`.

There is still a possible duplicate signal: developer-425 recorded an
event-only claim for lane95 before manager-24 assigned lane95 to developer-422.
The current `work_lanes` row is authoritative for this report and lists
`work/developer-422` and this worktree.

## Disk Snapshot

Observed filesystem state:

- `/`, `/tmp`, and `/home/claude/php-to-native-compiler`: `459G` total,
  `419G` used, `21G` available, `96%` use.
- `/dev/shm`: `22G` total, `2.6G` used, `20G` available, `12%` use.
- Root Cargo target `/home/claude/php-to-native-compiler/target`: `22G`.
- Report artifacts directory
  `/home/claude/php-to-native-compiler/.harness/reports`: `748K`.
- Worktrees under `.harness/worktrees`: 417 top-level developer worktrees.
- Worktree-local `target` directories: 13.

Largest observed worktree-local targets:

- `developer-88/target`: `1.4G`
- `developer-40/target`: `921M`
- `developer-397/target`: `875M`
- `developer-61/target`: `874M`
- `developer-168/target`: `451M`
- `developer-288/target`: `306M`

Small worktree-local targets also exist in developer-118, developer-121,
developer-113, developer-317, developer-115, developer-226, and
developer-117. These should be considered cleanup candidates only after an
active-process check and manager/integrator approval.

## Temp Build Dirs

Top-level `/tmp/phpc*` directories:

- Count: 27
- Total from `du -ch`: about `2.8G`
- Notable examples from `du -sh`: `471M`
  `/tmp/phpc-developer-378-target`, `391M`
  `/tmp/phpc-target-dev389-str-ireplace`, `391M`
  `/tmp/phpc-target-dev387-similar-text`, `391M`
  `/tmp/phpc-target-dev379-ini`, `362M`
  `/tmp/phpc-target-dev400-probe`, and `313M`
  `/tmp/phpc-target-dev381`.

Top-level `/dev/shm/phpc*` directories:

- Count: 20
- Total from `du -ch`: about `2.6G`
- Notable examples from `du -sh`: `502M`
  `/dev/shm/phpc-lane70-replay-dev125`, `416M`
  `/dev/shm/phpc-target-628`, `379M`
  `/dev/shm/phpc-target-dev161`, `362M`
  `/dev/shm/phpc-target-runtime-628`, `362M`
  `/dev/shm/phpc-target-cast-helpers`, and `306M`
  `/dev/shm/phpc-target-dev225`.

Note: the total-size command used here was `du -ch` over top-level matches,
which recursively printed child paths before the final total. Future report
lanes should use `du -sch` for the same total without the noisy recursive
listing.

## Resource Events

Recent `resource_warning` events show the system recovered from a worse state
but has not returned to a comfortable disk margin:

- At `2026-06-05T08:57:09+00:00`, resource warning payload reported
  CPU `100.0%`, load1 `47.61`, and disk free `10.17 GB`.
- At `2026-06-05T09:01:41+00:00`, disk free was still only `7.97 GB`.
- By `2026-06-05T09:55:09+00:00`, warning payload reported CPU `100.0%`,
  load1 `21.27`, and disk free `21.84 GB`.
- At this report's shell snapshot, `df -h` showed `21G` available and load1
  was below `nproc`.

The current bottleneck is not active Rust compilation. The risk is repeat
growth from many reserve/report agents and stale build directories while root
disk is only narrowly above the low-space threshold.

## Guardrails

Use these admission rules until root free disk is comfortably above 30 GB:

- Report-only lanes must not run Cargo health checks unless a manager
  explicitly asks for one.
- Any allowed Rust command must set a unique `CARGO_TARGET_DIR`, preferably
  under `/dev/shm/phpc-target-<lane-or-agent>` while `/dev/shm` has space.
- If `/dev/shm` is unavailable, use `/tmp/phpc-target-<lane-or-agent>`, never
  a worktree-local `target` and never the root repository `target`.
- Keep `CARGO_BUILD_JOBS=1`, `CARGO_INCREMENTAL=0`, and
  `RUST_TEST_THREADS=1` for focused checks during recovery.
- Do not run broad `cargo test`, `tools/run-tests.sh`, or full PHPT gates from
  reserve/report lanes during resource recovery.
- Managers should avoid assigning the same queued report lane by event-only
  claims; use `work_lanes.branch` and `work_lanes.worktree` as the
  authoritative owner fields after reassignment.

## Cleanup Plan

No deletion should occur from a developer report lane. A deterministic cleanup
lane should:

1. Query or record active process args for `cargo`, `rustc`, `cc`, `clang`,
   `phpc`, `codex`, and `python3`.
2. Build a candidate list of top-level `/tmp/phpc*`, `/dev/shm/phpc*`, and
   worktree-local `target` directories with size and mtime.
3. Exclude any path visible in active process arguments.
4. Exclude current explicitly assigned worktrees unless the owning lane is
   completed, integrated, or stopped.
5. Record the candidate list and intended deletions in SQLite before deleting.
6. Delete only approved candidate directories; do not delete root
   `/home/claude/php-to-native-compiler/target` without a separate integrator
   approval because it is large but shared.

## Commands Run

```sh
sed -n '1,240p' AGENTS.md
sed -n '1,260p' docs/PROGRESS.md
sed -n '1,260p' docs/ARCHITECTURE.md
sed -n '1,260p' docs/SUPPORT.md
sed -n '1,260p' README.md
sed -n '1,260p' docs/LOOP_MEMORY.md
sed -n '1,260p' GOAL.MD
sed -n '1,240p' docs/OPERATIONS.md
df -h / /tmp /dev/shm /home/claude/php-to-native-compiler
uptime
nproc
du -sh /home/claude/php-to-native-compiler/target
du -sh /home/claude/php-to-native-compiler/.harness/reports
find /home/claude/php-to-native-compiler/.harness/worktrees -mindepth 1 -maxdepth 1 -type d | wc -l
find /home/claude/php-to-native-compiler/.harness/worktrees -mindepth 2 -maxdepth 2 -type d -name target | wc -l
find /home/claude/php-to-native-compiler/.harness/worktrees -mindepth 2 -maxdepth 2 -type d -name target -exec du -sh {} +
find /tmp -maxdepth 1 -type d -name 'phpc*' -exec du -sh {} +
find /dev/shm -maxdepth 1 -type d -name 'phpc*' -exec du -sh {} +
find /tmp -maxdepth 1 -type d -name 'phpc*' | wc -l
find /dev/shm -maxdepth 1 -type d -name 'phpc*' | wc -l
find /tmp -maxdepth 1 -type d -name 'phpc*' -exec du -ch {} +
find /dev/shm -maxdepth 1 -type d -name 'phpc*' -exec du -ch {} +
ps -eo comm= | sort | uniq -c | sort -nr | head -n 20
ps -eo pid,ppid,comm,%cpu,%mem,etime,args --sort=-%cpu | head -n 20
pgrep -a cargo
pgrep -a rustc
pgrep -a cc
pgrep -a clang
git status --short --branch
```

MCP SQLite queries were used for `work_lanes`, `agents`, `events`,
`bug_reports`, `test_runs`, and schema checks. Some late `resource_samples`
queries returned `database is locked`; the report uses successfully collected
`resource_warning` event payloads plus direct shell snapshots instead.
