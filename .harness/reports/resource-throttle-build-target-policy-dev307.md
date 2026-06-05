# Resource Throttle and Build Target Policy Refresh

Developer: developer-307
Lane: 124
Timestamp: 2026-06-05T08:57Z
Scope: read-only ops report; no compiler/runtime edits, no process cleanup,
no file deletion, no full PHPT gate.

## Executive Summary

The harness is resource saturated. Recent SQLite resource samples and
`resource_warning` events show sustained 100% CPU, load well above the 20-core
host, and root filesystem pressure near the hard floor. This is currently a
control-plane capacity problem, not a product compatibility signal.

Immediate policy should be:

- Stop spawning new implementation/report workers while `load1 >= nproc` or
  root free disk is below 20 GB.
- Keep excess developers in `reserve_no_source_edits` and skip cargo health
  checks during saturation.
- Permit only the explicitly owned implementation lanes, currently lane 8 and
  lane 100 per manager-18, to run focused harness tests until load drops.
- Require unique, explicit `CARGO_TARGET_DIR` for any allowed Rust check; never
  let reserve/report lanes build into per-worktree default `target` dirs.
- Treat janitor cleanup as a manager/integrator action only after recording the
  candidate paths and proving no active cargo/rustc process owns them.

## Evidence Snapshot

Control-plane query source:

- Harness DB: `/home/claude/php-to-native-compiler/.harness/harness.sqlite3`
- SQLite access: Python stdlib fallback; MCP memory tools and `sqlite3` CLI
  were unavailable in this session.
- Project HEAD in this worktree: `7f61915aed09`
- `DEVELOPMENT.md`: not present under the repository root during this lane.

Resource state observed during this report:

- `uptime`: load averages `39.63, 33.71, 105.22` on `20` CPUs.
- `df -h /`: root filesystem `459G`, used `429G`, available `12G`, use `98%`.
- Latest DB resource sample at `2026-06-05T08:57:09+00:00`: CPU `100.0%`,
  RAM `27.34%`, free disk `10.17 GB`, load1 `47.61`.
- Active agent rows with `ended_at IS NULL`: 5 Architects, 1 Auditor,
  123 Developers, 9 Integrators, and 5 Managers.
- Excess spawn/load reserves already marked by manager-18: 27 active Developer
  rows with `reserve_no_source_edits: excess capacity during spawn/load
  mitigation`.

Recent resource warnings:

- Since `2026-06-05T08:40:00+00:00`: 27 `resource_warning` events, 21 with a
  killed PID recorded by the Janitor path.
- Since `2026-06-05T08:50:00+00:00`: 12 `resource_warning` events, 8 with a
  killed PID.
- Since `2026-06-05T08:55:00+00:00`: 3 `resource_warning` events, 2 with a
  killed PID.
- All sampled warnings in those windows report CPU `100.0%`.
- Load1 in those warnings ranged from `20.21` to `47.61`.
- Free disk in those warnings fell from `20.92 GB` at the earliest sampled
  point to `10.17 GB` at the latest sampled point.

Representative event sequence:

- `2026-06-05T08:50:58+00:00`: architect spawn for
  `harness/idle-alert-auditor-spawn-storm` was coalesced.
- `2026-06-05T08:54:26+00:00`: lane 95 was reassigned for disk/build-target
  guardrail audit.
- `2026-06-05T08:55:47+00:00`: manager-18 created lanes 119-132 as read-only
  capacity/report lanes and stated implementation remains limited to lanes 8
  and 100.
- `2026-06-05T08:56:38+00:00`: manager-18 marked newly spawned excess
  developers as reserve instead of creating unbounded low-value work because
  CPU/spawn pressure was high.

## Target Directory Risk

Shallow disk checks, avoiding recursive `.harness/worktrees` scans:

- Root Cargo target: `/home/claude/php-to-native-compiler/target` is `22G`.
- This worktree has no local `target` directory at report time.
- `/tmp` contains many lane-specific target/cache dirs; most reserve health
  targets are small `18M` dirs, but notable larger dirs include:
  `/tmp/phpc-target-dev233` at `380M`,
  `/tmp/phpc-target-developer-242` at `313M`, and
  `/tmp/phpc-target-developer-235` at `85M`.
- `/dev/shm` contains several active historical target dirs, including
  `/dev/shm/phpc-lane70-replay-dev125` at `502M`,
  `/dev/shm/phpc-target-628` at `416M`,
  `/dev/shm/phpc-target-cast-helpers` at `362M`,
  `/dev/shm/phpc-target-runtime-628` at `362M`, and
  `/dev/shm/phpc-target-dev161` at `379M`.
- `/dev/shm` still had about `20G` available when checked, so it is safer for
  short focused builds than root disk, but it still needs explicit per-lane
  naming and stale-dir cleanup discipline.

The highest disk risk is not one specific temporary directory; it is allowing
many concurrent agents to run Cargo without an explicit target directory. With
hundreds of worktrees, accidental per-worktree `target` dirs can multiply the
22G root target footprint and push the 98% full root filesystem into build or
SQLite failure territory.

## Scheduler Guardrails

1. Spawn throttle:
   - Do not spawn Developers, Integrators, Architects, or Auditors when
     `load1 >= nproc` or root `disk_free_gb < 20`.
   - Exception: one Manager-owned control-plane repair lane may be spawned only
     if it replaces a confirmed non-live owner and records the old/new owner in
     `work_lanes.notes`.

2. Reserve behavior:
   - New excess agents should be marked `reserve_no_source_edits` immediately.
   - Reserve agents should not run `cargo check` while `load1 >= nproc`, while
     any `resource_warning` occurred in the last 5 minutes, or while root free
     disk is below 20 GB.
   - Reserve agents may run read-only SQLite/report checks that do not scan
     `.harness/worktrees` recursively.

3. Test/build admission:
   - Under saturation, allow focused tests only for the active top control
     lanes named by Manager, currently lane 8 and lane 100.
   - Read-only report lanes must not run full PHPT gates or broad Cargo checks.
   - Integrators should prefer mergeability/diff checks and defer heavy cargo
     verification until CPU and disk thresholds recover.

4. Alert coalescing:
   - Resource alerts should open or update one durable alert key, for example
     `resource:saturation`, instead of spawning parallel diagnosis lanes.
   - If an owner lane already exists, subsequent warnings should append compact
     event evidence to that lane rather than creating more agents.

## Developer Build Policy

Use this default for any allowed focused Rust check:

```sh
umask 0002
export CARGO_BUILD_JOBS=1
export CARGO_INCREMENTAL=0
export RUST_TEST_THREADS=1
export CARGO_TARGET_DIR=/dev/shm/phpc-target-<agent-or-lane>
```

Fallback to `/tmp/phpc-target-<agent-or-lane>` only when `/dev/shm` has
insufficient free space. Do not use the default `target` directory in a
worktree. Do not share a target dir between concurrent lanes.

For report-only lanes:

- No Cargo health check is required under saturation.
- If a Manager explicitly requests a health check, record the exact command in
  SQLite first and use a unique `CARGO_TARGET_DIR`.
- Stop after one focused command; do not escalate to workspace tests unless a
  Manager/Integrator explicitly asks.

## Cleanup Guardrails

No cleanup was performed by this lane.

Deterministic cleanup should be a separate Manager/Integrator-authorized pass:

1. Query active process args for `cargo`, `rustc`, `cc`, `clang`, `phpc`, and
   `codex`.
2. Produce a candidate list of `/tmp/phpc-*` and `/dev/shm/phpc-*` dirs older
   than a chosen age threshold, with sizes and last modified times.
3. Exclude any candidate path visible in active process args.
4. Record the candidate list and intended deletions in SQLite before deleting.
5. Delete only candidate paths from `/tmp` and `/dev/shm`; do not delete
   repository-root `target` or worktree-local `target` dirs without explicit
   Integrator approval and a clean process check.

## Commands Run

Read-only commands used for this report:

```sh
python3 - <<'PY'  # SQLite schema, lane, event, and resource sample queries
PY
df -h / /tmp /dev/shm /home/claude/php-to-native-compiler
uptime && nproc
ps -eo comm= | awk '{count[$1]++} END { ... }'
ps -eo pid,ppid,comm,%cpu,%mem,etime,args --sort=-%cpu
du -sh /home/claude/php-to-native-compiler/target
find /tmp -maxdepth 1 -type d \( -name 'phpc*' -o -name '*phpc*target*' \) -print | sort | xargs -r du -sh
find /dev/shm -maxdepth 1 -type d \( -name 'phpc*' -o -name '*phpc*target*' \) -print | sort | xargs -r du -sh
git status --short
git rev-parse --short=12 HEAD
```

No tests were run because this was a read-only ops/report lane under CPU and
disk saturation.
