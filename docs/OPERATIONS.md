# Operations

This project treats automation as guardrails, not proof of feature completion.
The scripts in `tools/` run the existing project test suite and create frequent
git checkpoints only after tests pass.

## Full Test Suite

Run the whole suite with:

```sh
tools/run-tests.sh
```

That currently runs:

- `cargo test`
- `cargo run -p phpc -- test`
- `cargo run -p phpc -- test --compare-php`

The PHP comparison command still passes when system `php` is absent; comparison
counts are reported as skipped while committed fixture expectations still run.
Fixtures with sibling `.phpc-only` marker files are also skipped by system PHP
comparison while remaining covered by committed `phpc` expectations.
The comparison summary breaks skipped fixtures down by reason so CI logs show
how many skips came from a missing `php` binary and how many came from
`.phpc-only` markers.
For fixture audits that should not execute code, use
`cargo run -p phpc -- test --list-fixtures [fixture-dir]`; it prints a sorted
manifest of fixtures, recognized expectation files, and PHP-comparison
eligibility.

Add new required project-wide checks to `tools/run-tests.sh` so checkpoint and
loop automation pick them up automatically.

## Focused Lane Tests

When a worker is handling one narrow lane milestone, run focused tests that
prove that slice before spending time on the full suite. Record the exact
commands in `docs/PROGRESS.md`.

Start with the narrowest executable proof:

- one affected Rust integration test file, or one named test inside it;
- one exact fixture directory when fixtures changed;
- PHP comparison only for fixture directories intended to match system PHP;
- one CLI snapshot integration test when `.cli` files changed;
- `cargo fmt --check` only when Rust files changed;
- `git diff --check -- <changed-files>` for every lane.

Do not run workspace `cargo test` as the first verification step for a narrow
lane slice. Escalate from a single named test, to the whole affected test file,
to the exact fixture directory, and then to `tools/run-tests.sh` when the change
touches shared infrastructure or before checkpoint batches.

Use this default shape:

```sh
export CARGO_TARGET_DIR=/dev/shm/phpc-target-<lane-or-milestone>
export CARGO_BUILD_JOBS=1
export CARGO_INCREMENTAL=0

cargo test -p phpc --test <affected-test-file> <test-name> -- --test-threads=1
cargo run -p phpc -- test tests/fixtures/<affected-fixture-dir>
cargo run -p phpc -- test --compare-php tests/fixtures/<affected-fixture-dir>
git diff --check -- <files-touched-by-this-lane>
```

For Rust edits, also run:

```sh
cargo fmt --check
```

The focused gate is a development-time filter, not a replacement for
`tools/run-tests.sh`. Run the full gate before checkpoint batches or document a
blocker in `docs/PROGRESS.md`.

For unsupported syntax boundaries, use the closure checklist in
`docs/LANE_WORKERS.md` before handoff. A complete boundary needs pinned
diagnostics, CLI fixture snapshots, support docs, named unsupported edges, and
focused commands recorded in `docs/PROGRESS.md`.

## Parallel Lane Worktrees

`GOAL.MD` work can be split across parser, IR/lowering, runtime,
compiler-output, and tests/docs workers. Use one git worktree per lane and one
active milestone per worker:

```sh
git worktree add ../phpc-parser-lane -b lane/parser HEAD
git worktree add ../phpc-ir-lane -b lane/ir-lowering HEAD
git worktree add ../phpc-runtime-lane -b lane/runtime HEAD
git worktree add ../phpc-output-lane -b lane/compiler-output HEAD
git worktree add ../phpc-tests-docs-lane -b lane/tests-docs HEAD
```

Each lane should use a unique `CARGO_TARGET_DIR`, inspect `git status --short`
before handoff, and avoid shared files unless the milestone requires a scoped
documentation or progress update. See `docs/LANE_WORKERS.md` for lane
ownership, subagent prompt templates, focused-test expectations, and handoff
notes.

## Checkpoints

Create a tested checkpoint commit with:

```sh
tools/checkpoint.sh "checkpoint: describe the completed work"
```

Behavior:

- runs `tools/run-tests.sh`
- refuses to commit if tests fail
- stages all current tracked, untracked, modified, and deleted files with
  `git add -A`
- commits with the provided message, or with `CHECKPOINT_MESSAGE`, or with a
  generated timestamp message
- creates no commit when the tree is clean after tests

Because this commits all current changes, inspect `git status --short` first
when working in a shared tree.

## Codex Supervisor Loop

Run one bounded Codex round:

```sh
CODEX_RUNNER='codex exec' tools/codex-loop.sh
```

Run more rounds explicitly:

```sh
CODEX_RUNNER='codex exec' MAX_ROUNDS=3 tools/codex-loop.sh
```

`CODEX_RUNNER` is required and must be a usable non-interactive Codex command
that reads the prompt from standard input. The script does not assume exact CLI
flags. Use `CODEX_LOOP_PROMPT=/path/to/prompt.md` to override the default prompt
file, `docs/CODEX_LOOP_PROMPT.md`.

Each round:

1. runs `tools/run-tests.sh` before Codex starts
2. pipes the loop prompt into `CODEX_RUNNER`
3. runs `tools/run-tests.sh` after Codex exits
4. calls `tools/checkpoint.sh "codex checkpoint: round N of M"`

The loop stops on test failure, runner failure, or checkpoint failure. It
defaults to `MAX_ROUNDS=1` and never runs forever by default.

## Infinite YOLO Loop

For unattended continuation, run:

```sh
tools/codex-yolo-forever.sh
```

This script intentionally has no loop-level stop condition. It runs until the
process is killed by the terminal, shell, OS, or external supervisor.

Behavior:

- generates a fresh prompt each round from current git status, `docs/PROGRESS.md`,
  `docs/NEXT_TASKS.md`, and `docs/LOOP_MEMORY.md`
- runs `codex exec --dangerously-bypass-approvals-and-sandbox`
- runs `tools/run-tests.sh` before and after each Codex pass
- calls `tools/checkpoint.sh` after a round only when the post-round test suite
  passes
- prints and logs a roadmap summary at the start of each round, including total
  checked-task percentage, an ASCII progress bar, milestone checkboxes, the next
  unchecked task, current HEAD, and dirty worktree file count
- appends machine-readable-ish events to `docs/LOOP_MEMORY.md`
- stores generated prompts and logs under `.codex-yolo/logs/`

Useful environment variables:

- `CODEX_BIN`: Codex executable name or path, default `codex`
- `CODEX_YOLO_MEMORY`: memory file path, default `docs/LOOP_MEMORY.md`
- `CODEX_YOLO_LOG_DIR`: log directory, default `.codex-yolo/logs`
- `CODEX_YOLO_PROGRESS_WIDTH`: progress bar width, default `32`, minimum `10`

The forever loop is intentionally aggressive. Use it only when you want the repo
to keep changing without manual prompts.

## Done Means Done

A feature is not complete unless all of these exist:

- implementation code
- tests proving the behavior
- a CLI exercise path
- accurate documentation
- named unsupported edge cases

Do not mark tasks complete or claim support for behavior based on scaffolding,
plans, or placeholders.
