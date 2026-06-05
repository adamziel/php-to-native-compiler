# Developer-389 Self-Selected Source Slice Quarantine

Lane: 138
Report agent: developer-421
Generated: 2026-06-05T10:00Z
Artifact: `.harness/reports/self-selected-lane136-quarantine-dev389.md`

This is report-only control-plane work. No compiler, runtime, fixture, docs, or
PHPT source files were edited. No full PHPT gate was run and no public score was
changed.

The artifact filename uses the manager-requested `lane136` spelling, but the
source-changing branch being quarantined is `work_lanes#137`
(`work/developer-389`). This report itself is `work_lanes#138`.

## Decision

Do not integrate `work/developer-389` as current M0/M1 progress.

The branch contains a bounded `str_ireplace()` implementation with focused
evidence, but it was self-selected while manager notes were prioritizing
lane8/lane100 control-plane and 221205Z M0/M1 evidence work. Manager-20
superseded the source lane at 2026-06-05T09:33:23Z, and integrator-35 later
left the completed branch deferred because the root had dirty source/doc
overlap and no clean integration target.

## Quarantined Branch State

- Branch/worktree: `work/developer-389` /
  `/home/claude/php-to-native-compiler/.harness/worktrees/developer-389`
- HEAD: `6db16b03702411195e7611690d7eb3abcb808d34`
  (`runtime: add bounded str_ireplace builtin`)
- Remote: `origin/work/developer-389`
- Worktree status at inspection: clean and tracking `origin/work/developer-389`
- Branch containment: local `work/developer-389` and
  `remotes/origin/work/developer-389`

Observed source/doc/test delta against `master...HEAD`:

| Path | Status |
| --- | --- |
| `compiler/src/codegen.rs` | modified |
| `compiler/src/interpreter.rs` | modified |
| `compiler/tests/str_ireplace_builtin.rs` | added |
| `docs/ARCHITECTURE.md` | modified |
| `docs/NEXT_TASKS.md` | modified |
| `docs/PROGRESS.md` | modified |
| `docs/SUPPORT.md` | modified |
| `tests/fixtures/milestone2306/str_ireplace_bounded.cli` | added |
| `tests/fixtures/milestone2306/str_ireplace_bounded.php` | added |
| `tests/fixtures/milestone2306/str_ireplace_bounded.stdout` | added |

Git reported `10 files changed, 395 insertions(+), 75 deletions(-)`.

## Recorded Verification

Developer-389 recorded focused checks for commit `6db16b03`:

- `cargo fmt --check`
- `git diff --check`
- `cargo test -q -p phpc --test str_ireplace_builtin -- --test-threads=1`
- `cargo run -q -p phpc -- test --compare-php tests/fixtures/milestone2306`
- `cargo run -q -p phpc -- run tests/fixtures/milestone2306/str_ireplace_bounded.php`
- `cargo check -q -p phpc`

The lane notes also say the branch was pushed to `origin/work/developer-389`.
No full suite or public PHPT gate was run.

## Risk

- The branch changes product source (`compiler/src/interpreter.rs` and
  `compiler/src/codegen.rs`) plus shared docs and fixtures, so it is not safe to
  path-limit as a report artifact.
- The docs touched by this branch overlap files that have been repeatedly dirty
  or conflicted in recent integration notes: `docs/PROGRESS.md`,
  `docs/SUPPORT.md`, `docs/ARCHITECTURE.md`, and `docs/NEXT_TASKS.md`.
- The lane was explicitly superseded before completion under the current
  M0/M1 priority. Integrating it now would violate reviewer/manager intent even
  though the focused implementation evidence is recorded.
- Public metric remains unchanged. The accepted public score is still governed
  by the latest accepted full-gate/adjudication policy, not by this focused
  fixture lane.

## Future Lane Proposal

If manager/integrator policy reopens product-source compatibility lanes after
the current control-plane and M0/M1 blockers, `work/developer-389` can be used
as candidate evidence for a fresh explicit `str_ireplace()` lane.

Minimum deterministic reopen steps:

- Rebase or cherry-pick onto a clean integration target after docs overlap is
  resolved.
- Re-run the focused checks listed above with a pinned `CARGO_TARGET_DIR`.
- Add or run selected standard string PHPT rows that exercise `str_ireplace()`
  before claiming public compatibility impact.
- Keep unsupported edges named in docs: replacement/subject arrays, nested
  search arrays, non-variable count targets, object/resource coercions,
  non-ASCII case folding, exact diagnostics, binary edge cases, and native
  lowering.
- Require integrator acceptance before any source merge or metric movement.

## Commands And Data Sources

Harness evidence came from SQLite MCP queries against `work_lanes`, `events`,
`agents`, and `messages`. Lane `#138` was claimed with Python's standard
`sqlite3` module because the `sqlite3` CLI is unavailable in this environment.

Commands used:

```sh
git status --short --branch
git log --oneline -6
git diff --name-status master...HEAD
git show --stat --oneline --decorate --no-renames HEAD
git diff --stat master...HEAD
git remote -v
git branch --contains 6db16b03702411195e7611690d7eb3abcb808d34 --all
```

No recursive `.harness/worktrees` scan was performed.
