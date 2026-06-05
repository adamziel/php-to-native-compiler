# Disk and Build-Target Guardrail Audit

Developer: developer-425
Lane: 95
Timestamp: 2026-06-05T10:01Z
Scope: read-only ops report. No compiler/runtime edits, no cleanup, no rebuild,
and no full PHPT gate.

## Summary

Root disk is still tight enough that build admission needs guardrails even
though the immediate free-space number has recovered from the earlier 10-12G
low point. Current `df` shows the root filesystem at `459G` total, `419G`
used, `21G` available, `96%` used. `/dev/shm` is much healthier at `22G`
total, `2.6G` used, `20G` available.

The main local disk consumers are deterministic:

- `/home/claude/php-to-native-compiler`: `59G`
- repository-root `target`: `22G`
- `.harness/worktrees`: `34G`
- `.git`: `3.5G`
- `/tmp`: `2.8G`
- `/dev/shm`: `2.6G`

The biggest avoidable risk is still accidental default Cargo targets in many
worktrees. This audit found `417` harness worktree directories and `13`
worktree-local `target` directories. Those 13 local targets account for about
`5G` of root usage, on top of the shared root `target` at `22G`.

## Current Evidence

`df -h / /tmp /dev/shm`:

```text
Filesystem  Size  Used Avail Use% Mounted on
root        459G  419G   21G  96% /
tmpfs        22G  2.6G   20G  12% /dev/shm
```

`df -i / /dev/shm` shows inode pressure is not the immediate blocker: root
inode use is `64%`, `/dev/shm` inode use is `1%`.

Current load was moderate relative to the prior resource-throttle report:

```text
load average: 7.04, 12.87, 13.02
nproc: 20
```

Process mix still shows many live harness sessions: `20` `codex` processes and
`21` `python3` processes were present in the sampled process list. The top CPU
consumers were Codex manager/developer/integrator sessions, not Rust builds.

MCP `resource_samples` reads hit `database is locked` during this audit, so
the current snapshot above uses bounded shell measurements. Earlier lane 124
evidence already recorded the saturation low point at root free disk near
`10.17 GB`; this report refreshes the disk shape rather than replacing that
policy.

## Target Directory Findings

Repository-local top-level sizes:

```text
59G  /home/claude/php-to-native-compiler
34G  /home/claude/php-to-native-compiler/.harness
22G  /home/claude/php-to-native-compiler/target
3.5G /home/claude/php-to-native-compiler/.git
46M  /home/claude/php-to-native-compiler/tests
15M  /home/claude/php-to-native-compiler/compiler
11M  /home/claude/php-to-native-compiler/.codex-yolo
8.1M /home/claude/php-to-native-compiler/docs
```

Harness worktree size:

```text
34G  /home/claude/php-to-native-compiler/.harness/worktrees
417  direct worktree directories
13   worktree-local target directories
```

Largest worktree-local targets:

```text
1.4G  developer-88/target
921M  developer-40/target
875M  developer-397/target
874M  developer-61/target
451M  developer-168/target
306M  developer-288/target
28M   developer-121/target
25M   developer-226/target
25M   developer-118/target
25M   developer-117/target
25M   developer-115/target
25M   developer-113/target
18M   developer-317/target
```

The oldest sampled worktree-local target modification times were from the
early local morning:

```text
2026-06-05 01:48  developer-40/target
2026-06-05 01:48  developer-61/target
2026-06-05 01:57  developer-88/target
```

Temporary target/cache directories:

- `47` distinct `/tmp` or `/dev/shm` directories matched `phpc*` or
  `*phpc*target*`.
- Notable larger `/dev/shm` dirs include `phpc-lane70-replay-dev125` at
  `502M`, `phpc-target-628` at `416M`, `phpc-target-dev161` at `379M`,
  `phpc-target-runtime-628` at `362M`, and `phpc-target-cast-helpers` at
  `362M`.
- Notable larger `/tmp` dirs include `phpc-developer-378-target` at `471M`,
  `phpc-target-dev389-str-ireplace` at `391M`,
  `phpc-target-dev387-similar-text` at `391M`,
  `phpc-target-dev379-ini` at `391M`, and `phpc-target-dev400-probe` at
  `362M`.

The report artifact directory itself is not a disk risk: `.harness/reports`
has `64` files and is only `748K`.

## Guardrails

Build admission:

- Treat root free space under `25G` as a yellow zone. Reserve/report agents
  should not run Cargo health checks in this zone.
- Treat root free space under `20G` as a hard stop for non-critical Rust
  builds. Only manager-approved top repair lanes should run focused checks.
- Any allowed Rust command must set a unique explicit target directory:
  `CARGO_TARGET_DIR=/dev/shm/phpc-target-<agent-or-lane>` when `/dev/shm` has
  at least `5G` available, otherwise `/tmp/phpc-target-<agent-or-lane>`.
- Never allow default per-worktree `target` directories for reserve/report
  work. The 13 existing worktree-local targets prove this pattern leaks root
  disk quickly.
- Keep `CARGO_BUILD_JOBS=1`, `CARGO_INCREMENTAL=0`, and
  `RUST_TEST_THREADS=1` for focused checks while the harness has many live
  Codex sessions.

Cleanup admission:

- Do not delete anything from a Developer lane. Cleanup should be a
  Manager/Integrator-authorized pass with a recorded candidate list.
- Before deletion, scan active process args for `cargo`, `rustc`, `cc`,
  `clang`, `phpc`, `run-tests.php`, and `codex`, and exclude any path that
  appears in a live command line.
- Prefer temporary cleanup first: `/tmp/phpc-*` and `/dev/shm/phpc-*` dirs
  older than an agreed age threshold and not visible in active process args.
- Treat `/dev/shm/phpc-lane70-replay-dev125` specially: lane 96 identified
  unpinned but executable scratch `phpc` binaries there. Do not remove it
  unless replay owners or a manager explicitly reject its provenance.
- Delete worktree-local `target` directories only after confirming the owning
  worktree has no live agent, no active integration owner, and no in-progress
  lane. The high-value candidates by size are developer-88, developer-40,
  developer-397, developer-61, developer-168, and developer-288.
- Do not delete the repository-root `target` without explicit Integrator
  approval; it is large (`22G`) but may be the only warm cache for root-level
  verification.

Worktree cleanup:

- Keep worktree deletion separate from target cleanup. Worktrees may contain
  unmerged or quarantined artifacts even when their agents are stopped.
- Integrators should prune only after verifying branch provenance, pushed
  state, and whether the worktree is ancestry-merged or intentionally
  quarantined.
- Because a normal clean worktree baseline is about `72M`, removing stale
  worktrees can recover meaningful space, but the branch/provenance checks are
  more important than raw size.

## Commands Run

Read-only commands used:

```sh
df -h / /home /tmp /dev/shm
df -i / /home /tmp /dev/shm
du -h --max-depth=1 /home/claude/php-to-native-compiler
du -h --max-depth=2 /home/claude/php-to-native-compiler/.harness
find /home/claude/php-to-native-compiler/.harness/worktrees -mindepth 1 -maxdepth 1 -type d -printf '%f\n' | wc -l
find /home/claude/php-to-native-compiler/.harness/worktrees -mindepth 2 -maxdepth 2 -type d -name target -printf '%p\n'
for d in /home/claude/php-to-native-compiler/.harness/worktrees/*/target; do du -sh "$d"; done
for d in /tmp/phpc* /tmp/*phpc*target* /dev/shm/phpc* /dev/shm/*phpc*target*; do du -sh "$d"; done
find /tmp /dev/shm -maxdepth 1 -type d \( -name 'phpc*' -o -name '*phpc*target*' \) -printf '.' | wc -c
uptime && nproc
ps -eo comm= | sort | uniq -c | sort -nr
ps -eo pid,ppid,comm,%cpu,%mem,etime,args --sort=-%cpu
git status --short
```

Focused tests were not run because this was a report-only disk audit. No full
PHPT gate was run and no public score movement is claimed.
