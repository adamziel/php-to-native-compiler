# Lane Workers

Use this guide when splitting `GOAL.MD` work across subagents. The goal is to
let parser, IR/lowering, runtime, compiler-output, and tests/docs workers make
progress independently without overwriting each other's files or turning focused
test runs into unbounded waits.

## Worktree Setup

Start each lane from the same clean base commit when possible:

```sh
git worktree add ../phpc-parser-lane -b lane/parser HEAD
git worktree add ../phpc-ir-lane -b lane/ir-lowering HEAD
git worktree add ../phpc-runtime-lane -b lane/runtime HEAD
git worktree add ../phpc-output-lane -b lane/compiler-output HEAD
git worktree add ../phpc-tests-docs-lane -b lane/tests-docs HEAD
```

If the main tree is already dirty, do not use `tools/checkpoint.sh` from that
tree. Either checkpoint from a lane worktree whose status has been inspected, or
wait for the integration owner to batch and test the combined changes.

Each worker should set a unique target directory so cargo builds do not fight
over one shared `target/`:

```sh
export CARGO_TARGET_DIR=/dev/shm/phpc-target-parser
export CARGO_BUILD_JOBS=1
export CARGO_INCREMENTAL=0
```

Use a lane-specific suffix for `CARGO_TARGET_DIR`, such as `ir`, `runtime`,
`output`, or `tests-docs`.

## Lane Ownership

Parser lane:

- Owns lexer, parser, AST shape, parse diagnostics, and syntax fixtures.
- Avoids runtime semantics and native support claims unless another lane has
  implemented and tested them.
- Typical focused checks: parser unit tests, syntax boundary tests, targeted
  fixture directories, and `git diff --check`.

IR/lowering lane:

- Owns `phpc compile --emit-ir`, `phpc compile --emit-asm`, native codegen
  diagnostics, deterministic backend/fallback snapshots, and native support
  docs.
- Avoids parser/runtime rewrites unless a slice cannot be proved without a
  named cross-lane handoff.
- Typical focused checks: relevant `native_*_boundary` tests, relevant
  `native_assembly_cli` snapshot tests, targeted fixture directories, and
  `git diff --check`.

Runtime lane:

- Owns `php_runtime`, interpreter behavior for `phpc run`, boxed values,
  builtins, object/array semantics, and system PHP comparison fixtures.
- Avoids native compile support claims unless the IR/lowering lane implements
  and tests them.
- Typical focused checks: affected runtime/compiler integration tests, targeted
  fixture directories, `phpc test --compare-php` for those directories, and
  `git diff --check`.

Compiler-output lane:

- Owns CLI behavior, emitted artifact shape, backend selection and failure
  behavior, stdout/stderr/exit contracts, and operational scripts.
- Avoids changing expression lowering internals while an IR/lowering milestone
  is active.
- Typical focused checks: specific CLI snapshot tests, focused emitted-artifact
  fixtures, script dry-runs where available, and `git diff --check`.

Tests/docs lane:

- Owns fixture organization, focused/full gate documentation, support matrix
  accuracy, progress records, operations docs, and unsupported-edge naming.
- Avoids implementation files unless fixing the documented verification surface
  requires a named handoff.
- Typical focused checks: documentation grep checks, targeted fixture runner
  commands when fixtures change, and `git diff --check`.

## Subagent Prompt Template

Give each subagent a lane, one milestone, and a forbidden-file boundary:

```text
Lane: <parser|ir-lowering|runtime|compiler-output|tests-docs>
Workspace: <absolute worktree path>
Objective: complete exactly one small milestone under GOAL.MD.
Ownership: <files and behavior this lane owns>.
Forbidden without handoff: <files or lanes to avoid>.
Required reading: AGENTS.md, GOAL.MD, docs/NEXT_TASKS.md, docs/PROGRESS.md,
docs/SUPPORT.md, README.md, docs/OPERATIONS.md, and any lane-specific doc.
Required output: files changed, commands run, focused verification result,
unsupported or remaining gaps, and whether the full suite was deferred.
Do not checkpoint or commit unless explicitly instructed.
```

## Focused Test Policy

During a lane slice, run the smallest tests that prove the changed behavior.
The full gate remains mandatory before checkpoint batches unless a blocker is
recorded.

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

Record the exact focused commands in `docs/PROGRESS.md`. If the full gate is
deferred, say so directly and name the next point where it must run.

## Unsupported Syntax Boundary Checklist

Use this checklist when closing a parser-lane unsupported syntax boundary.
The goal is a stable, honest rejection surface, not partial support.

- Name the boundary as lex, parse, runtime, or codegen. Syntax that cannot be
  tokenized safely should stop in lexing; reserved tokens or grammar forms
  should stop in parsing; accepted interpreter-only constructs must still have
  explicit native codegen rejection when lowering is unsupported.
- Pin the diagnostic message and source location with focused coverage in
  `compiler/tests/syntax_boundaries.rs`. Include representative forms, casing
  where relevant, expression-position forms where relevant, and an
  `emit_ir_source(...)` assertion when native lowering must reject before
  backend invocation.
- Add or update a fixture under
  `tests/fixtures/unsupported_syntax_features/` with `.php`, `.stderr`,
  `.exit`, `.cli`, and `.phpc-only` files. The committed fixture should prove
  the `phpc run` CLI path, while `.phpc-only` keeps project-specific
  diagnostics out of system PHP comparison.
- Update `README.md`, `docs/SUPPORT.md`, and `docs/ARCHITECTURE.md` when the
  boundary changes the public support surface. Name the unsupported edge cases
  directly, including future runtime and native lowering work that is not being
  implemented.
- Update `docs/NEXT_TASKS.md` and `docs/PROGRESS.md` with the milestone,
  focused commands, full-gate status, and whether checkpointing was skipped
  because the active tree contains unrelated work.
- Run focused checks before any full gate:
  `cargo test -p phpc --test syntax_boundaries <boundary-name> -- --test-threads=1`;
  `cargo test -p phpc --test unsupported_syntax_features_cli -- --test-threads=1`;
  `cargo run -p phpc -- test tests/fixtures/unsupported_syntax_features`;
  `cargo run -p phpc -- test --compare-php tests/fixtures/unsupported_syntax_features`;
  direct `phpc run <fixture>` for the new boundary fixture; direct
  `phpc compile <fixture> --emit-ir` when the construct should fail before
  native lowering or backend invocation; plus scoped `git diff --check`.

Current examples:

- `yield` and `yield from`: parse boundary with unit coverage in
  `unsupported_yield_syntax_has_stable_parse_errors`,
  `emit_ir_rejects_yield_syntax_at_parse_boundary`, and the
  `unsupported_yield` fixture set.
- `goto` statements and labels: parse boundary with unit coverage in
  `unsupported_goto_syntax_has_stable_parse_errors`,
  `emit_ir_rejects_goto_syntax_at_parse_boundary`, and the `unsupported_goto`
  fixture set.
- heredoc/nowdoc strings: lex boundary with unit coverage in
  `unsupported_heredoc_nowdoc_syntax_has_stable_lex_errors`,
  `emit_ir_rejects_heredoc_nowdoc_syntax_at_lex_boundary`, and the
  `unsupported_heredoc` fixture set.

## Handoff Notes

When a lane touches shared files such as `README.md`, `docs/SUPPORT.md`,
`docs/ARCHITECTURE.md`, `docs/NEXT_TASKS.md`, or `docs/PROGRESS.md`, keep the
edit narrow and include a handoff note in the final report.

Cross-lane handoff notes must name:

- affected lane;
- files touched;
- focused commands already run;
- full-suite status;
- remaining unsupported or unverified behavior.

Do not treat a passing focused command as proof for another lane's milestone
unless the command covers that lane's requirements directly.

## Current Lane Queue

Use `docs/NEXT_TASKS.md` as the source of truth for milestone status. The
current split is:

- Tests/docs lane: Milestone 624 closed the 620-623 implementation-batch queue
  refresh; Milestone 629 is the next tests/docs slot after the next
  implementation batch.
- Parser lane: Milestone 622 closed the unsupported `trait` declaration
  boundary; Milestone 627 is the next parser slot, selecting a small syntax or
  parse-diagnostic boundary from documented unsupported gaps without widening
  runtime or native support claims.
- Compiler-output lane: Milestone 620 closed selected-`llc`
  empty-stdout-with-stderr no-`cc`-fallback coverage; Milestone 625 is the next
  compiler-output slot to choose another deterministic CLI artifact or backend
  contract coverage target from existing native output behavior.
- IR/lowering lane: Milestone 621 closed native callable lookup folding
  coverage for `array_is_list`; Milestone 626 is the next IR slot, selecting a
  narrow native IR/lowering refinement or precise rejection boundary from
  already documented interpreter behavior.
- Runtime lane: Milestone 623 closed current bool-like scalar autoload flag
  support for `class_exists`, `interface_exists`, `trait_exists`, and
  `enum_exists` through `phpc run`; Milestone 628 is the next runtime slot to
  choose a small array/object builtin refinement from the documented
  unsupported gaps.

Milestones 555-560 closed the first split-lane batch, Milestones 561, 571, 587,
592, 597, 602, 607, 612, and 617 closed recent parser slots, Milestones 565, 568,
572, 575, 577, 579, 581, 583, 585, 590, 595, 600, 605, 610, and 615 closed recent
compiler-output slots, Milestones 567, 570, 574, 576, 578, 580, 582, 584, 586,
593, 598, 603, 608, 613, and 618 closed recent runtime slots, and Milestones 569,
573, 588, 591, 596, 601, 606, 611, and 616 closed recent IR/lowering slots.
Milestones 604, 609, 614, and 619 closed recent tests/docs queue refreshes.
Milestone 1094 closed the runtime clone/reference-slot mirroring slice,
Milestone 1095 closed the promoted constructor property parameter parse
boundary, Milestone 1096 closed the native reference-assignment rejection
boundary, and Milestone 1097 closed fixture-runner compare-summary coverage.
Milestone 1098 closed context-aware non-public clone reference-slot mirroring,
Milestone 1099 closed the nullsafe object operator parse boundary, Milestone
1100 closed the native clone rejection boundary, and Milestone 1101 closed
compile emit-mode validation precedence.
The next batch should again keep one active milestone per lane and should use
separate worktrees and separate `CARGO_TARGET_DIR` values when workers run in
parallel.
