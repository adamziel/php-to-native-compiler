# Supervisor Recovery Note - 2026-06-02 00:22 CEST

## Current Base

- Integration worktree:
  `/home/claude/supervised-php-compiler/state/worktrees/batch024-current-supervisor-20260601T093225`
- Clean source base: `62e5e453 docs: publish latest checkpoint progress`
- Remote confirmation: `origin/master` points at `62e5e453`
- Public progress page now reports `5513 / 20294 = 27.17%` and names latest
  pushed source checkpoint `1f09a754`.

## Earlier Stall / Crash Causes

- AO lifecycle was not actually advancing worker sessions at one point:
  `ao status` reported `0 active sessions`, so the requested team was not
  running.
- The main checkout at `/home/claude/php-to-native-compiler` is dirty and far
  behind `origin/master`; using it as an integration base risks stale diffs and
  misleading status.
- Disk pressure was high enough to threaten builds and worker target
  directories. Cleanup improved free space from roughly `17G` to `44G`, but the
  filesystem is still about `91%` used.
- The supervisor serialized too much work behind full checkpoint/full-suite
  validation. That made local source progress wait on central long-running
  gates and hid worker output until late.
- Some AO sessions were `ready` rather than `working`; spawned sessions must be
  verified as active instead of assumed alive.

## Guardrails For The Next Loop

- Use the clean supervisor worktree as integration base. Do not integrate from
  the dirty main checkout unless it is explicitly reconciled.
- Keep at least 10 AO worker sessions alive, but measure `working` vs `ready`;
  nudge or replace silent sessions.
- Workers must produce branch/PR artifacts with focused Rust/PHPT proof only.
  They must not run `tools/checkpoint.sh` or full pinned PHPT gates.
- Supervisor owns checkpoint commits, public progress updates, and full
  pinned-PHPT gates in a separate acceptance lane.
- Maintain a disk floor: if free space drops below `30G`, pause spawning and
  run cleanup before more builds.
- Prefer small independent PHPT slices with named unsupported edges, CLI
  exercise paths, and docs updates. Reject broad refactors that do not move
  pinned runnable rows.

## Active Team Shape

- `bcs2-61`: standard math numeric strings PR lane.
- `bcs2-62`: disk cleanup lane.
- `bcs2-63` through `bcs2-66`: existing PR refresh/repair lanes.
- `bcs2-67` through `bcs2-70`: discovery and small implementation lanes.

