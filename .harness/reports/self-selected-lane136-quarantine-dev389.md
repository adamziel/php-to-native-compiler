# Developer-389 Self-Selected Slice Quarantine

Lane: 138
Generated: 2026-06-05T09:59Z
Report owner: developer-432
Subject branch: `work/developer-389`
Subject commit: `6db16b03702411195e7611690d7eb3abcb808d34`
Subject worktree: `/home/claude/php-to-native-compiler/.harness/worktrees/developer-389`

Scope is report-only. I did not merge, cherry-pick, rebase, edit compiler or
runtime source, run a full PHPT gate, or move the public score.

## Decision Summary

Do not integrate `work/developer-389` as current M0/M1 progress by branch head.
Manager-20 superseded self-selected product lanes 136, 137, and 139 at
2026-06-05T09:33:23Z and redirected developer-389 to this quarantine report.

The branch has useful focused evidence for bounded interpreter
`str_ireplace()` support, but it is source-changing product work outside the
current control-plane/M0/M1 priority. It also predates the current integration
head `8381ad99`; comparing `HEAD..6db16b03` shows deletion of nine already
integrated report artifacts. Any future integration must be explicit,
path-limited or rebased, and revalidated on the current integration base.

Accepted public score remains `7873 / 20294`. The blocked `221205Z` candidate
remains blocked at `7197 / 20294` with `1166` latest-public PASS regressions.
This report claims no score movement.

## Branch Content

Subject commit:

```text
6db16b03 runtime: add bounded str_ireplace builtin
```

Diff from the subject branch base `42e081b3`:

```text
M compiler/src/codegen.rs
M compiler/src/interpreter.rs
A compiler/tests/str_ireplace_builtin.rs
M docs/ARCHITECTURE.md
M docs/NEXT_TASKS.md
M docs/PROGRESS.md
M docs/SUPPORT.md
A tests/fixtures/milestone2306/str_ireplace_bounded.cli
A tests/fixtures/milestone2306/str_ireplace_bounded.php
A tests/fixtures/milestone2306/str_ireplace_bounded.stdout
```

Stat from the subject branch base:

```text
10 files changed, 395 insertions(+), 75 deletions(-)
```

Current-head hazard from `8381ad99..6db16b03`:

```text
D .harness/reports/221205Z-regression-status-summary-refresh-dev313.md
D .harness/reports/221205Z-shard-abort-root-cause.md
D .harness/reports/late-row-manifest-command-smoke-dev311.md
D .harness/reports/no-skipif-array-string-selector-sanity-dev312.md
D .harness/reports/phpt-binary-wrapper-availability-recheck-dev305.md
D .harness/reports/public-metric-status-consistency-dev230.md
D .harness/reports/queued-message-delivery-audit-dev309.md
D .harness/reports/run62-runtime-candidate-merge-prereqs-dev308.md
D .harness/reports/self-selected-source-branch-quarantine-dev306.md
```

Those deletions are an integration-base artifact, not intended product work,
but they make a plain branch-head merge unsafe.

## Recorded Evidence

Developer-389 recorded completion at 2026-06-05T09:39:36Z with commit
`6db16b03702411195e7611690d7eb3abcb808d34`, pushed to
`origin/work/developer-389`.

Recorded focused checks:

```text
cargo fmt --check
cargo test -q -p phpc --test str_ireplace_builtin -- --test-threads=1
cargo run -q -p phpc -- test --compare-php tests/fixtures/milestone2306
cargo run -q -p phpc -- run tests/fixtures/milestone2306/str_ireplace_bounded.php
cargo check -q -p phpc
```

Harness `test_runs` records show the focused Rust integration test passed
7 tests, the milestone2306 fixture compared against system PHP, the direct CLI
run produced `php php|2`, `%0DD|2`, and `b-b`, and `cargo check -q -p phpc`
passed with only existing warnings. No full suite or full PHPT gate was
recorded.

## Quarantine Handling

- Preserve `origin/work/developer-389` for future audit.
- Do not integrate it into current M0/M1 work or count it as accepted public
  PHPT progress.
- If a manager later opens a `str_ireplace()` compatibility lane, start from
  this commit as evidence but rebase or path-apply it onto the then-current
  integration base.
- Re-run focused tests on the current base before review, including the Rust
  integration test, the CLI fixture, system PHP comparison, `cargo fmt --check`,
  `git diff --check`, and `cargo check -q -p phpc`.
- Require docs to name unsupported edges before support is claimed. The
  developer-389 lane notes already name replacement/subject arrays, nested
  search arrays, non-variable count targets, object/resource coercions,
  non-ASCII case folding, exact diagnostics, binary edge cases, and native
  lowering as unsupported.

## Commands And Data Sources

```sh
git show --stat --oneline --decorate \
  6db16b03702411195e7611690d7eb3abcb808d34
git show --name-only --format=medium \
  6db16b03702411195e7611690d7eb3abcb808d34
git merge-base HEAD 6db16b03702411195e7611690d7eb3abcb808d34
git diff --stat 42e081b3d983786cba75d9de2f10df10cf687193..6db16b03702411195e7611690d7eb3abcb808d34
git diff --name-status 42e081b3d983786cba75d9de2f10df10cf687193..6db16b03702411195e7611690d7eb3abcb808d34
git diff --name-status 8381ad999b894ddafa329ea8cd0789ca8659e906..6db16b03702411195e7611690d7eb3abcb808d34 -- .harness/reports compiler src runtime docs tests README.md Cargo.toml Cargo.lock
git branch --contains 6db16b03702411195e7611690d7eb3abcb808d34 --all --format='%(refname:short)'
```

Harness memory inputs:

- `work_lanes#138`
- manager-20 event `self_selected_lanes_superseded`
- developer-389 event `developer_completed`
- `test_runs#184`, `test_runs#186`, `test_runs#187`, `test_runs#188`,
  and `test_runs#189`

No recursive `.harness/worktrees` scan was performed.
