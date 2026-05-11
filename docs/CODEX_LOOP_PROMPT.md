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
