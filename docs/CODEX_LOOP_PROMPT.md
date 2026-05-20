# Codex Loop Prompt

You are working in `/home/claude/php-to-native-compiler`.

Start by reading:

- `AGENTS.md`
- `docs/PROGRESS.md`
- `docs/ARCHITECTURE.md`
- `docs/SUPPORT.md`
- `README.md`
- `docs/OPERATIONS.md`
- `docs/NEXT_TASKS.md`

Check `git status --short`. Other people may be changing this repository. Do
not revert or rewrite work you did not make.

Stability rules for this repository:

- Do not resume the old `019e3b39-8c72-7101-a6c7-fe216ada9a43` session. Its
  history has grown into multi-million-token turns and is a crash trigger.
  Continue from the worktree, docs, and logs instead.
- Run one heavyweight Rust/build/test/checkpoint command at a time.
- Use `CARGO_BUILD_JOBS=1`, `CARGO_INCREMENTAL=0`, `RUST_TEST_THREADS=1`, and
  `timeout` for expensive checks when the session is unstable.
- Do not run `rustfmt` or `cargo fmt` on `compiler/src/interpreter.rs` in the
  current 9 GiB/no-swap VM. On 2026-05-20 it was OOM-killed at about 7.7 GiB
  RSS. Use `git diff --check`, focused `cargo check`, and focused tests
  instead unless the work moves to a larger VM.
- Redirect verbose output to log files and inspect short tails instead of
  streaming full test or checkpoint output into the Codex transcript.
- Prefer focused checks before `tools/run-tests.sh` or `tools/checkpoint.sh`.

Take the first unchecked task in `docs/NEXT_TASKS.md` that can be completed
honestly in this round. Implement a small correct subset instead of broad
placeholder support. Unsupported edge cases must be named in docs.

A feature is done only when all of these exist:

- implementation code
- tests proving the behavior
- a CLI exercise path
- accurate documentation
- named unsupported edge cases

After implementing:

1. run `tools/run-tests.sh`
2. run any additional focused CLI command needed to prove the task
3. update docs, including `docs/SUPPORT.md` and `docs/PROGRESS.md` when behavior
   changes
4. mark the completed task in `docs/NEXT_TASKS.md`
5. commit through `tools/checkpoint.sh` with a specific message

If the task cannot be completed without guessing, leave the task unchecked,
document the blocker, run the tests that still apply, and do not claim feature
support.
