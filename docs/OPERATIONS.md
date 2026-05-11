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

Add new required project-wide checks to `tools/run-tests.sh` so checkpoint and
loop automation pick them up automatically.

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

## Done Means Done

A feature is not complete unless all of these exist:

- implementation code
- tests proving the behavior
- a CLI exercise path
- accurate documentation
- named unsupported edge cases

Do not mark tasks complete or claim support for behavior based on scaffolding,
plans, or placeholders.
