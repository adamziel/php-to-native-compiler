# Run62 Runtime Candidate Merge Prerequisites

## Scope

- Lane: 125, read-only integration prerequisite report.
- Candidate branches reviewed: `work/developer-120`, `work/developer-124`,
  and `work/developer-122`.
- Current local head used for merge checks: `7f61915a`.
- Original candidate branch base for all three branches: `e147c033`.
- No compiler/runtime source was edited for this report.
- No full PHPT gate was run, and no public PHPT score movement is claimed.

## Executive Decision

Prefer the clean canonical candidate sequence:

1. Merge `work/developer-120` commit
   `e04e3df9a49f3a1cce20764279bc83cc81a48ebf`.
2. Merge `work/developer-124` commit
   `7a17b7eee5edb4ec2f2a12aa01d8ffddf2793d90`.
3. Do not merge `work/developer-122`
   `5294d6a85765bc714ca1fb006b6daf557e8c2a51` as part of that sequence.

`developer-122` is useful duplicate evidence, but its lane was superseded and
it conflicts after the `developer-120` plus `developer-124` sequence in
`runtime/src/lib.rs` and `docs/PROGRESS.md`.

## Branch Evidence

### `work/developer-120`

- Lane 66 status: completed.
- Commit: `e04e3df9a49f3a1cce20764279bc83cc81a48ebf`
  (`test: align runtime metadata assertions`).
- Changed paths: `runtime/src/lib.rs`.
- Scope: refreshed stale `php_runtime` expectations for byte-backed PHP string
  values, native comparison materialization, core class metadata, and
  reference-held typed-property diagnostics.
- Developer proof recorded in SQLite event `87224` and test run `75`:
  `CARGO_TARGET_DIR=/dev/shm/phpc-target-dev120 CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 cargo test -p php_runtime --lib -- --test-threads=1`
  passed with `419` passed, `0` failed.
- Integrator proof:
  - test run `98`: disposable merge verification passed
    `php_runtime --lib` with `419` passed, `0` failed.
  - test run `122`: disposable merge onto later master passed
    `php_runtime --lib` with `419` passed, `0` failed.
  - lane 66 notes record integrator-28 reverified merge cleanliness on current
    master `7f61915a` with `merge --no-commit`, `git diff --check`, and
    `cargo fmt --check`; heavy retest was deferred because the prior 419/419
    proof already existed.

### `work/developer-124`

- Lane 61 status: completed.
- Commit: `7a17b7eee5edb4ec2f2a12aa01d8ffddf2793d90`
  (`test: isolate native call argument free counter`).
- Changed paths: `runtime/src/lib.rs`, `docs/PROGRESS.md`.
- Scope: converts the test-only `NativeCallArgumentsHandle` free counter from a
  process-global atomic to a thread-local `RefCell<i64>` under `cfg(test)`, and
  adds a guard test proving the isolation. Runtime ABI behavior is unchanged.
- Developer proof recorded in SQLite event `87171`:
  - `cargo test -q -p php_runtime --lib call_arguments_free_count_is_thread_local_for_parallel_tests -- --nocapture`
  - `cargo test -q -p php_runtime --lib native_lookup_plus_invoke_helpers_free_arguments_once_across_target_families -- --nocapture`
  - `cargo test -q -p php_runtime --lib native_closure_invoke_helpers_bridge_call_arguments_to_call_results -- --nocapture`
  - `cargo test -q -p php_runtime --lib native_constructor_allocation_invoke_carrier_owns_receiver_arguments_and_diagnostics -- --nocapture`
  - `cargo test -q -p php_runtime --lib native_method_lookup_plus_invoke_dispatches_missing_and_inaccessible_methods_to_magic_call -- --nocapture`
  - `cargo check -q -p php_runtime`
  - `git diff --check`
- Integrator proof:
  - test run `80`: disposable merge verification of five focused filters
    passed with `5` passed, `0` failed.
  - test run `99`: combined disposable merge sequence
    `work/developer-120` then `work/developer-124` passed `git diff --check`,
    `cargo fmt --check`, and `cargo test -p php_runtime --lib` with `420`
    passed, `0` failed in default parallel test mode.

### `work/developer-122`

- Lane 60 status: superseded.
- Commit: `5294d6a85765bc714ca1fb006b6daf557e8c2a51`
  (`fix: stabilize runtime test gate`).
- Changed paths: `runtime/src/lib.rs`, `docs/PROGRESS.md`,
  `docs/OPERATIONS.md`, `tools/run-tests.sh`.
- Scope: combines stale runtime assertion refresh with a runner change that
  defaults `tools/run-tests.sh` to `RUST_TEST_THREADS=1`.
- Developer proof recorded in SQLite events `87366`, `87391`, and `87439`:
  - `cargo test -q -p php_runtime class_table_can_bootstrap_core_exception_metadata -- --test-threads=1 --nocapture`
    passed with `1` passed, `0` failed.
  - `cargo test -q -p php_runtime native_string -- --test-threads=1 --nocapture`
    passed with `16` passed, `0` failed.
  - `cargo test -q -p php_runtime native_comparison -- --test-threads=1 --nocapture`
    passed with `13` passed, `0` failed.
  - `cargo test -q -p php_runtime static_property -- --test-threads=1 --nocapture`
    passed with `10` passed, `0` failed.
  - `cargo test -q -p php_runtime` passed with `419` passed, `0` failed.
  - `cargo fmt --check` passed.
  - `git diff --check -- runtime/src/lib.rs tools/run-tests.sh docs/OPERATIONS.md docs/PROGRESS.md`
    passed.
- Integration concern: developer-122 later recorded that lane 60 had been
  superseded during concurrent work and left the lane status as superseded.
  Its runtime changes overlap the developer-120 assertion refresh, and its
  runner change overlaps the separate harness command-selection/control-plane
  lanes rather than the canonical runtime pair.

## Merge Conflict Findings

Individual merge-tree checks from current `HEAD` produced clean trees for all
three branches:

- `git merge-tree --write-tree HEAD work/developer-120` -> clean tree
  `5e90fa00a570a723602a044c119cb7b72a4fd013`.
- `git merge-tree --write-tree HEAD work/developer-124` -> clean tree
  `b65d18b979739cfb00b0bf1a9112e934cd63e893`.
- `git merge-tree --write-tree HEAD work/developer-122` -> clean tree
  `2f5305c194513c6d1aa6242c17758b16082f6748`.

The actual candidate sequence matters. A temporary disposable worktree check
with sequential `git merge --no-commit --no-ff` produced:

- `work/developer-120`: clean, committed as temporary `66afdcf3`.
- `work/developer-124`: clean, committed as temporary `2327ca6e`.
- `work/developer-122`: conflict in `runtime/src/lib.rs` and
  `docs/PROGRESS.md`; `docs/OPERATIONS.md` and `tools/run-tests.sh` would also
  be modified if accepted.

Therefore, `developer-122` should not be merged after the canonical
`developer-120` plus `developer-124` sequence without a deliberate integrator
decision to re-resolve duplicate runtime assertions and to accept the
`tools/run-tests.sh` serialized-test policy.

## Clean Merge Prerequisites

- Ensure the integration target worktree has no dirty `runtime/src/lib.rs`,
  `docs/PROGRESS.md`, `docs/OPERATIONS.md`, or `tools/run-tests.sh` changes.
  Lane 66 notes already mention a dirty shared root as a reason integration was
  deferred.
- Merge `work/developer-120` first, then `work/developer-124`.
- Re-run at minimum:
  - `git diff --check`
  - `cargo fmt --check`
  - `CARGO_TARGET_DIR=<unique-target> CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 cargo test -p php_runtime --lib`
- Treat `work/developer-122` as evidence only unless the integrator explicitly
  wants the runner policy change. If that policy is wanted, resolve it as a
  separate harness/operations decision after the runtime pair is integrated.
- Do not claim PHPT score movement from these merges. They remove a focused
  `php_runtime --lib` blocker for checkpoint/full-gate readiness; the accepted
  public PHPT score remains `7873/20294` until a zero-regression public gate or
  accepted adjudication moves it.

## Report Commands Run By Developer-308

- `rg --files -g 'AGENTS.md' -g 'DEVELOPMENT.md' -g 'README.md' -g 'docs/PROGRESS.md' -g 'docs/ARCHITECTURE.md' -g 'docs/SUPPORT.md' -g 'docs/LOOP_MEMORY.md'`
- `sed -n` reads of the required session docs and current status sections.
- Python `sqlite3` queries against `.harness/harness.sqlite3` for lane, agent,
  event, message, and test-run evidence.
- `git log --oneline --decorate --max-count=20` for each candidate branch.
- `git show --stat --oneline --decorate` for each candidate branch.
- `git diff --name-status` and `git diff --stat` for each candidate branch.
- `git diff --check work/developer-308...<candidate>` for each candidate
  branch; all three returned clean.
- `git merge-tree --write-tree HEAD <candidate>` for each candidate branch.
- Disposable sequential merge check:
  `HEAD -> work/developer-120 -> work/developer-124 -> work/developer-122`,
  where the third merge conflicted as described above.
