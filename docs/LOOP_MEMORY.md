# Codex YOLO Loop Memory

This file is durable memory for unattended Codex loop runs. The forever loop
injects this file into every prompt. Each Codex pass should update it with:

- current checkpoint commit
- task attempted
- files changed
- tests run and result
- blockers or semantic gaps
- next concrete task

## Current Baseline

- Latest checkpoint before the forever loop script: `b27c3a0 runtime errors: add structured diagnostics`.
- Full suite command: `tools/run-tests.sh`.
- Current next task queue: `docs/NEXT_TASKS.md`.
- Current rule: do not claim full PHP support; implement the next small tested
  behavior and checkpoint only when tests pass.

