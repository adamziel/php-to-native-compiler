# COW/runtime source-lane prerequisite audit

Generated: 2026-06-05T09:59:52Z
Worker: developer-430
Lane: work_lanes#111
Artifact scope: report only; no compiler/runtime edits, no PHPT gate, no public score movement.

## Bottom line

Do not reopen COW/runtime source implementation lanes yet unless a Manager
explicitly authorizes a new source lane after the current M0/M1 control-plane
state is stable. The COW frontier is real and well documented, but source work
should wait until the test-loop command-selection recurrence and idle-alert
liveness/storm state have deterministic post-fix evidence.

Current SQLite state during this audit:

- `work_lanes#8` is marked `completed`, with focused evidence that harness
  command selection now returns `tools/run-tests.sh`.
- `work_lanes#100` is marked `completed`, with focused evidence for active
  agent liveness predicates, but `bug_reports#3` and `bug_reports#4` remain
  open.
- `work_lanes#143` is `in_progress` on developer-419 for a fresh
  `test_runs#215` zero-test recurrence after the prior command-selection
  fixed marker.
- `work_lanes#133`, the self-selected COW runtime lvalue handle source slice,
  is `deferred_m0_m1_priority`; its notes explicitly say not to continue
  implementation until control-plane/regression evidence gates allow source
  lanes again.

## Authoritative planning constraints

`/home/claude/php-to-native-compiler/PLAN.md` says the public metric remains
`7873 / 20294 = 38.79%`, while the later `221205Z` gate is blocked at
`7197 / 20294 = 35.46%` with `1166` PASS regressions. That blocked gate must
not move the public score.

The plan also keeps `eval` and variable variables as late-priority work and
sets the near-term ordering as:

- M0: classify and reduce the `221205Z` PASS regressions before broad source
  integration.
- M1: stabilize measurement and control-plane state.
- M2: resume compatibility expansion only after M0 is under control.

That means a new COW/runtime source lane needs explicit evidence that it will
not distract from M0/M1 or worsen integration churn.

## COW frontier state

`docs/LOOP_MEMORY.md`, `docs/NEXT_TASKS.md`, and
`docs/COW_COVERAGE_MATRIX.md` all point to the same hard-first rule: future
COW work should extend general mechanisms instead of adding case-specific
branches.

The active mechanism names are:

- `RPR`: runtime provenance resolver. `ArrayCopySource` roots should resolve
  through `RuntimeAliasLvalueHandle` identities before alias rehydration,
  mirror, promotion, import remapping, or writeback.
- `DMB`: dynamic mutation boundary. Assignment, compound assignment, unset,
  reference assignment, alias writes, property replacement, and related
  writes must capture old storage identity and use shared invalidation or
  rehydration.
- `CCA`: callback call-frame adapter. Direct calls, closures, dynamic calls,
  array-callables, `call_user_func()`, `call_user_func_array()`, by-value
  imports, by-reference bindings, and reference returns should move
  copied-source metadata through shared call-frame carriers.
- `GAP`: untracked containers, unreachable runtime cells, string COW, native
  COW lowering, and unsupported syntax remain explicit unsupported gaps until
  a shared mechanism owns them.

Already-migrated COW handle consumers include runtime-cell alias
rehydration/overlay, helper/callback copied-source mirror/import setup,
reference-return path promotion, existing-cell lookup, return-cell
rehydration, alias mirroring, helper/callee writeback, copied-source
reference-cell scanning, copied-source value reads, object-property
invalidation/detached-path checks, dirty copied-source detection, static
dirty metadata recovery, direct by-reference argument setup, closure capture
source recovery, assignment-expression source recovery, and nested-write
parent-replacement checks.

The remaining hard gaps named by current docs are broader dynamic holder
writeback, untracked containers that cannot expose a concrete bucket/object or
runtime cell, transform-remap extraction, callback-driven transforms, exact
PHP diagnostics/Throwable parity, string COW, and native reference/COW
lowering.

## Prerequisites before a new source lane

A new COW/runtime source lane should not start until all of these are true or
a Manager records an explicit exception:

1. Control-plane proof is stable:
   - No active fresh zero-test recurrence remains untriaged.
   - `test_runs#215` / `work_lanes#143` is resolved as either duplicate
     stale evidence or a concrete post-fix bug.
   - The harness selector proof includes a deterministic `discover_test_command`
     check that returns `tools/run-tests.sh` for this repo and a nonzero
     focused `.harness/tests` run.

2. Idle-alert/liveness proof is accepted:
   - Open `bug_reports#3` and `bug_reports#4` are either fixed or explicitly
     accepted as not blocking source work.
   - Evidence includes ended/failed/stopped filtering, missing tmux
     window/pane filtering or retirement, duplicate same-target alert
     dedupe/throttle, and one preserved alert for a genuinely live idle owner.

3. M0 regression state is not being bypassed:
   - The `221205Z` blocked gate remains reported as blocked, not public
     progress.
   - New source work has a focused PHPT or fixture target tied to the
     regression plan, not broad self-selected compatibility expansion.
   - Late-priority `eval` and variable-variable work remains deferred.

4. Integration hygiene is clean:
   - Source lane starts from a current, clean worktree.
   - Deferred branches such as `work/developer-349` are preserved for audit
     rather than continued in place without reassignment.
   - Expected dirty overlap is named before edits begin, especially shared
     docs such as `docs/PROGRESS.md`, `docs/SUPPORT.md`,
     `docs/ARCHITECTURE.md`, and `docs/NEXT_TASKS.md`.

5. COW intake is mechanism-first:
   - The lane identifies one owning mechanism: `RPR`, `DMB`, or `CCA`.
   - The lane names the unsupported edges it will leave out before coding.
   - The patch extends reusable resolver/mutation/call-frame machinery rather
     than another one-off wrapper branch.
   - Candidate source slices after M0/M1 clears: dynamic holder writeback
     through runtime lvalue handles, untracked-container propagation only
     where a concrete runtime cell/bucket can be exposed, or reusable
     transform-remap extraction under `RPR`.

6. Required completion evidence is available:
   - Implementation code.
   - Focused Rust unit or integration tests for the exact mechanism.
   - CLI fixture exercise path, normally `cargo run -p phpc -- test <fixture>`
     and `cargo run -p phpc -- test --compare-php <fixture>` when PHP parity
     is claimed.
   - Regression fixture coverage that keeps representative existing proofs
     such as `tests/fixtures/milestone2294` green when transform remapping or
     copied-source import changes are touched.
   - `cargo check -q -p phpc`, `git diff --check -- <changed files>`, and
     `cargo fmt --check` if Rust formatting could change.
   - Docs updated in `docs/PROGRESS.md`, `docs/SUPPORT.md`, and relevant
     architecture/next-task docs, with unsupported edges named.

## Recommended next deterministic actions

- Let developer-419 finish `work_lanes#143` before treating command-selection
  as fully stable.
- Keep `work_lanes#133` deferred unless a Manager explicitly reopens source
  work after M0/M1 evidence is accepted.
- If source work is reopened, create a fresh narrow lane with a named
  mechanism owner and a fixture/test plan before editing compiler/runtime
  files.
- Prefer `RPR` extraction for the next hard-first source lane if the goal is
  COW consolidation; prefer M0 regression repair lanes if the goal is public
  PHPT score recovery.

## Commands and queries used

- Read startup/project docs: `AGENTS.md`, `docs/PROGRESS.md`,
  `docs/ARCHITECTURE.md`, `docs/SUPPORT.md`, `README.md`,
  `docs/LOOP_MEMORY.md`, `GOAL.MD`, `docs/NEXT_TASKS.md`,
  `docs/COW_COVERAGE_MATRIX.md`, and
  `/home/claude/php-to-native-compiler/PLAN.md`.
- Confirmed `DEVELOPMENT.md` and `CLAUDE.md` were absent in this worktree.
- Claimed lane with a conditional SQLite update:
  `UPDATE work_lanes ... WHERE id=111 AND status='queued'`, which returned
  `rowcount=1`.
- Queried `work_lanes` for lanes `8`, `100`, `111`, `133`, and `143`.
- Queried `bug_reports` for current command-selection and idle-alert status.
- Queried `metadata` for current metric/control-plane status.
- Queried recent `events` and `test_runs` for fresh recurrence and proof
  evidence.

## Verification

This artifact is report-only. The required verification is whitespace/path
checking for this file, not compiler/runtime tests or PHPT gates.
