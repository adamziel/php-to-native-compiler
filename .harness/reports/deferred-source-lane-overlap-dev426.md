# Deferred Source-Lane Dirty-Overlap Report

Audit time: 2026-06-05T10:01:04Z
Auditor: developer-426
Lane: work_lanes#112
Branch: `work/developer-426`

This is read-only coordination work. It does not edit compiler, runtime, test,
or support source files. It does not merge branches, run a full PHPT gate, or
claim public score movement. Eval and variable-variable rows remain late
priority.

## Scope

Manager-24 assigned developer-426 to summarize deferred source lanes
40, 61, 63, 65, and 66 from SQLite lane notes, recent integration events,
test-run evidence, and branch file lists. I did not recursively scan
`.harness/worktrees`; branch metadata came from Git refs in this worktree.

Current report base is `8381ad99` on `work/developer-426`.

## Executive Summary

The only integration-ready source candidate among the requested lanes is the
combined lane 66 plus lane 61 runtime candidate:

- lane 66: `work/developer-120` at `e04e3df9a49f`
- lane 61: `work/developer-124` at `7a17b7eee5ed`
- verified combined candidate: `integration/integrator-34-runtime-mergecheck-20260605T0934`
  at `2dcf90cbd865`

That combined candidate changes only `runtime/src/lib.rs` and
`docs/PROGRESS.md`, and has repeated focused `php_runtime --lib` proof. It is
still not integrated to master because integrator notes report dirty shared-root
overlap in the same runtime/docs files.

Lanes 40, 63, and 65 are alternate or superseded runtime repair branches. They
touch the same runtime hot file, and lanes 40/63 also touch the same support
documentation set. Integrator events already confirm that lane 40 conflicts
with the verified 61+66 candidate in `runtime/src/lib.rs` and
`docs/PROGRESS.md`; lane 65 also conflicts with lane 66 in `runtime/src/lib.rs`.
These branches should remain audit/rebase inputs, not direct integration inputs.

## Lane Map

| Lane | Branch | Commit | DB status | Current disposition |
| --- | --- | --- | --- | --- |
| 40 | `work/developer-83` | `2f8aec28c427` | completed | Alternate runtime expectation repair. Conflicts with verified 61+66 candidate in `runtime/src/lib.rs` and `docs/PROGRESS.md`; do not direct-merge. |
| 61 | `work/developer-124` | `7a17b7eee5ed` | completed | Canonical companion to lane 66. Integrator-29 and integrator-34 verified combined candidate with lane 66. |
| 63 | `work/developer-114` | `26527dce7d95` | superseded | Duplicate/superseded native string ABI expectation slice. Overlaps lane 40 and lane 66 runtime/doc files; evidence only. |
| 65 | `work/developer-117` | `174370c4a137` | superseded | Alternate duplicate runtime repair. Integrator-22 and later notes report conflict with lane 66 in `runtime/src/lib.rs`; evidence only. |
| 66 | `work/developer-120` | `e04e3df9a49f` | completed | Canonical runtime assertion repair. Merge first, followed by lane 61, in a clean integration worktree. |

## Changed Path Overlap

`git diff --name-status --no-renames 8381ad99...<branch>` showed:

| Path | 40 | 61 | 63 | 65 | 66 | Risk |
| --- | --- | --- | --- | --- | --- | --- |
| `runtime/src/lib.rs` | yes | yes | yes | yes | yes | Direct overlap across every requested source lane. This is the critical conflict surface. |
| `docs/PROGRESS.md` | yes | yes | yes | no | no | Progress-log overlap between canonical lane 61 and alternate lanes 40/63. |
| `README.md` | yes | no | yes | no | no | Documentation overlap between duplicate/alternate lanes 40 and 63. |
| `docs/ARCHITECTURE.md` | yes | no | yes | no | no | Documentation overlap between duplicate/alternate lanes 40 and 63. |
| `docs/NATIVE_RUNTIME_ABI.md` | yes | no | yes | no | no | Documentation overlap between duplicate/alternate lanes 40 and 63. |
| `docs/SUPPORT.md` | yes | no | yes | no | no | Documentation overlap between duplicate/alternate lanes 40 and 63. |
| `compiler/tests/object_model.rs` | no | no | no | yes | no | Lane 65-only test touch; still not enough to make lane 65 safe because its runtime edit conflicts. |

Branch stats from current `HEAD`:

- `work/developer-83`: 6 files, 167 insertions, 137 deletions.
- `work/developer-124`: 2 files, 53 insertions, 4 deletions.
- `work/developer-114`: 6 files, 133 insertions, 123 deletions.
- `work/developer-117`: 2 files, 138 insertions, 83 deletions.
- `work/developer-120`: 1 file, 142 insertions, 75 deletions.
- Verified 61+66 candidate `2dcf90cbd865`: 2 files, 195 insertions,
  79 deletions.

## Verification Evidence

Canonical 61+66 evidence:

- Lane 66 developer proof: test_run#75 passed
  `cargo test -p php_runtime --lib -- --test-threads=1` with 419 passed.
- Lane 61 developer proof: test_run#67 passed five focused runtime filters,
  `cargo check -q -p php_runtime`, and `git diff --check`.
- Integrator-12 proof: test_run#99 passed combined disposable merge sequence
  `work/developer-120` then `work/developer-124` with `git diff --check`,
  `cargo fmt --check`, and `cargo test -p php_runtime --lib` at 420 passed.
- Integrator-29 proof: test_runs#147 and #148 passed the combined candidate
  in single-thread and default-thread `php_runtime --lib`, 420 passed.
- Integrator-34 proof: test_run#176 passed on current master `8381ad99`
  through candidate `2dcf90cbd865`, with `git diff --check`,
  `cargo fmt --check`, and `cargo test -p php_runtime --lib -- --test-threads=1`
  at 420 passed.

Conflict and deferral evidence:

- event#94431: integrator-34 confirmed lane 40 conflicts with the verified
  61+66 runtime candidate in `docs/PROGRESS.md` and `runtime/src/lib.rs`.
- lane 40 notes: integrator-29 and integrator-34 both deferred
  `work/developer-83` pending explicit reconciliation against lanes 61/66.
- lane 63 notes: branch `work/developer-114` was repeatedly marked duplicate
  or superseded; integrator-29 reported it merges individually but conflicts
  with the verified 61+66 cluster in `runtime/src/lib.rs` and docs.
- lane 65 notes: integrator-22 and integrator-28 identified
  `work/developer-117` as an alternate duplicate runtime repair that conflicts
  with `work/developer-120` in `runtime/src/lib.rs`.
- event#94123 and lane 66/61 notes: integration remains deferred because the
  shared root had dirty overlapping runtime/docs files at integration time.

## Recommended Integration Policy

1. Preserve `work/developer-83`, `work/developer-114`, and
   `work/developer-117` as audit inputs only.
2. Integrate only the verified lane 66 plus lane 61 candidate, and only from a
   clean integration target.
3. Use the proven sequence: `work/developer-120` first, then
   `work/developer-124`, or the already verified candidate branch
   `integration/integrator-34-runtime-mergecheck-20260605T0934`.
4. Before accepting, re-run the focused checks that match the prior evidence:
   `git diff --check`, `cargo fmt --check`, and
   `CARGO_TARGET_DIR=<unique> CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 cargo test -p php_runtime --lib`.
5. Do not bundle report-only integration, harness command-selection work, or
   superseded runtime expectation branches into the same source merge.
6. Do not claim PHPT score movement from this runtime merge alone. It removes a
   focused runtime test blocker for checkpoint/full-gate readiness, not a public
   PHPT pass-regression adjudication.

## Commands And Queries Used

- Read required startup docs: `AGENTS.md`, `docs/PROGRESS.md`,
  `docs/ARCHITECTURE.md`, `docs/SUPPORT.md`, `README.md`,
  `docs/LOOP_MEMORY.md`, and `docs/OPERATIONS.md`.
- SQLite MCP queries:
  - `SELECT id, ts, title, status, branch, worktree, notes FROM work_lanes WHERE id IN (40,61,63,65,66) ORDER BY id`
  - event searches for lanes 40/61/63/65/66 and developers 83/120/124
  - test-run searches for `php_runtime`, `git diff --check`, and
    `cargo fmt --check` evidence
- Git metadata:
  - `git for-each-ref` for candidate branch commits
  - `git show --stat --oneline --decorate --no-renames <branch>`
  - `git diff --name-status --no-renames 8381ad99...<branch>`
  - `git diff --stat --no-renames HEAD...<branch>`
- Existing report cross-check:
  - `.harness/reports/run62-runtime-candidate-merge-prereqs-dev308.md`

No source tests or PHPT replays were run for this report-only lane.
