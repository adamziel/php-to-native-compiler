# Project Goal

## Slash Goal Objective

Build this repository into a real, honest PHP-to-native compiler in stable Rust:
advance one small verified PHP feature or native-lowering slice at a time, keep
`phpc run` and `phpc compile` boundaries explicit, keep tests green, and never
claim support without executable code, CLI coverage, documentation, and named
unsupported edge cases.

## Product Target

The project should become a useful PHP compiler/runtime toolchain, not a demo.
Its first durable target is a documented PHP subset that can:

- parse PHP source with a real lexer/parser, not regex parsing;
- execute supported programs through `phpc run` with PHP-shaped boxed runtime
  values;
- compile a smaller but steadily growing supported subset through
  `phpc compile --emit-ir` and `phpc compile --emit-asm`;
- reject unsupported PHP constructs with stable parse, runtime, or codegen
  diagnostics instead of emitting misleading output;
- compare supported interpreter behavior against system PHP where practical;
- preserve accurate support documentation for every supported and unsupported
  feature.

Long term, the compiler should support enough PHP semantics to run meaningful
real-world, framework-shaped code. Dynamic PHP features may use interpreter or
runtime fallback zones, but those fallback zones must be explicit and tested.

## Current Baseline

The repository already has a working Rust workspace with:

- `php_runtime`: boxed PHP-shaped values for scalars, ordered arrays, class
  metadata, and minimal object values;
- `phpc`: lexer, parser, AST, interpreter, CLI, fixture runner, and narrow LLVM
  IR/assembly emission;
- `phpc run`: broad Milestone 1+ interpreter support for scalars, arrays,
  loops, functions, constants, selected builtins, minimal objects, and stable
  unsupported diagnostics;
- `phpc compile --emit-ir` and `--emit-asm`: intentionally narrower native
  lowering, mostly straight-line static scalar output and assignment behavior,
  plus explicit rejection coverage for unsupported constructs;
- fixture coverage under `tests/fixtures`, optional system PHP comparison, and
  operational automation under `tools/` and `docs/OPERATIONS.md`.

Read these before changing behavior: `AGENTS.md`, `docs/PROGRESS.md`,
`docs/ARCHITECTURE.md`, `docs/SUPPORT.md`, `README.md`, and
`docs/LOOP_MEMORY.md` when running unattended loop automation.

## Definition of Done

A feature or milestone is complete only when all of these are true:

- Implementation code exists in the compiler, runtime, or CLI.
- Unit, integration, or fixture tests cover the behavior.
- A CLI exercise path exists through `phpc run`, `phpc compile --emit-ir`,
  `phpc compile --emit-asm`, or an explicit CLI snapshot.
- Documentation in `README.md`, `docs/SUPPORT.md`, and any focused design doc
  is accurate.
- `docs/PROGRESS.md` records the change after meaningful behavior changes.
- Unsupported edge cases are named directly.
- Native lowering either implements the behavior honestly or rejects it with a
  specific codegen diagnostic.
- The relevant focused tests pass during the slice; checkpoint or integration
  batches run `tools/run-tests.sh` unless a blocker is documented.

## Engineering Constraints

- Use stable Rust for compiler and runtime code.
- Prefer small, correct PHP subsets over broad placeholder claims.
- Keep dynamic PHP features as explicit runtime fallback zones.
- Runtime values must remain PHP-shaped: boxed values first, ordered int/string
  keyed arrays, references and copy-on-write when implemented.
- Do not replace the parser with regular-expression parsing.
- Add crates only when they materially reduce work.
- Do not silently widen support in docs before tests prove it.
- Do not collapse interpreter support and native lowering into one claim:
  `phpc run` may support a construct while `phpc compile` still rejects it.
- Keep the `cc -S` assembly fallback documented as a bootstrap fallback, not the
  primary backend.

## Execution Lanes

The goal is intentionally broad, so day-to-day work should be split into
independent lanes. Each lane owns one milestone at a time and should use a
separate git worktree when running in parallel with other workers.

Lanes:

- Parser lane: lexer, parser, AST shape, parse diagnostics, and syntax-focused
  fixtures. This lane must not widen runtime or native support claims unless
  another lane implements and tests the behavior.
- IR/lowering lane: `phpc compile --emit-ir`, `phpc compile --emit-asm`,
  native codegen diagnostics, deterministic backend/fallback snapshots, and
  native support documentation.
- Runtime lane: `php_runtime`, interpreter semantics for `phpc run`, boxed
  value behavior, builtins, object/array semantics, and system PHP comparison
  fixtures.
- Compiler output lane: CLI behavior, emitted artifact shape, backend
  selection/failure behavior, future link/run output, stdout/stderr/exit
  contracts, and operational scripts.
- Tests/docs lane: fixture organization, focused/full test gates, support
  matrix accuracy, progress records, operations docs, and unsupported-edge
  naming.

Parallel work rules:

- One worktree per lane and one active milestone per worker. Avoid multiple
  workers editing the same source files, fixture directories, or docs section.
- Each worker records its lane and milestone in `docs/PROGRESS.md` before
  handing off or checkpointing.
- A worker may run focused tests for its lane during the slice, but a checkpoint
  batch must run the serialized full gate or document a blocker.
- Cross-lane changes require an explicit handoff note naming the affected lane,
  files, tests, and remaining risks.
- Shared files such as `docs/SUPPORT.md`, `docs/ARCHITECTURE.md`,
  `README.md`, `docs/NEXT_TASKS.md`, and `compiler/src/codegen.rs` require
  extra care: keep edits scoped, avoid broad rewrites, and do not overwrite
  another lane's unmerged work.

## Milestone Plan

### 0. Keep The Baseline Honest

Maintain the existing interpreter, native lowering, docs, and test suite as the
floor for all work.

Exit criteria:

- `tools/run-tests.sh` passes.
- `docs/NEXT_TASKS.md` has a clear next unchecked task.
- `docs/SUPPORT.md` distinguishes `phpc run` support from native compile
  support.
- New diagnostics are stable enough for fixture snapshots.

### 1. Finish Native Scalar Lowering

Grow `phpc compile --emit-ir` and `--emit-asm` from static scalar output toward
a coherent straight-line scalar subset.

Scope:

- static scalar reassignment and overwrite coverage;
- native scalar arithmetic for supported int/float/literal subsets;
- native comparison, logical, bitwise, unary, and conditional expression slices
  only when PHP coercion and error boundaries are explicit;
- compile-time diagnostics for unsafe static cases and explicit rejection for
  dynamic cases until runtime-backed checks exist.

Exit criteria:

- Lowered scalar programs have CLI assembly coverage and fixture coverage.
- Unsupported scalar cases fail before backend invocation with specific codegen
  diagnostics.
- Generated IR/assembly tests use deterministic assertions rather than
  platform-specific assembly text.

### 2. Link Runtime-Backed Native Execution

Move native compilation beyond standalone emitted assembly text by linking
generated code with runtime helpers.

Scope:

- define a stable ABI between generated code and `php_runtime`;
- lower boxed value creation, echo/print conversion, and selected runtime helper
  calls;
- add an executable native path only when process exit, stdout, stderr, and
  diagnostics can be tested;
- keep `--emit-ir` and `--emit-asm` useful as inspectable compiler stages.

Exit criteria:

- At least one non-trivial program can be compiled, linked, executed, and
  compared to `phpc run`.
- Runtime-backed native diagnostics are tested.
- Docs state exactly which constructs are native-executable versus
  interpreter-only.

### 3. Runtime Semantics: References and Copy-On-Write

Implement the PHP value semantics that affect real programs before expanding
syntax too far.

Scope:

- reference cells and aliasing boundaries;
- array/object copy-on-write behavior;
- function parameter and return interactions with references;
- mutation behavior for arrays, objects, foreach, and builtins;
- explicit unsupported diagnostics where exact PHP behavior remains out of
  scope.

Exit criteria:

- Aliasing and copy-on-write behavior has targeted runtime tests and CLI
  fixtures.
- Existing array/object builtins either honor the new semantics or document and
  test their temporary limits.
- Native lowering rejects or uses runtime helpers honestly.

### 4. Object Execution

Turn current class metadata and minimal object values into executable object
semantics.

Scope:

- `$this` binding and method dispatch;
- constructor execution and constructor arguments;
- visibility enforcement for public/protected/private access;
- static properties, static methods, class constants, and `::class`;
- object identity, `spl_object_id`, `spl_object_hash`, clone boundaries;
- inheritance after the single-class model is stable.

Exit criteria:

- Method calls execute through `phpc run` with scoped `$this`.
- Constructor and visibility behavior have fixtures and stable errors.
- Introspection builtins are updated to reflect implemented object semantics.
- Native object lowering remains explicit: implemented subset or precise
  rejection.

### 5. Program Structure

Add PHP program-structure features needed by real codebases.

Scope:

- include/require resolution and execution;
- namespaces, imports, aliases, and qualified name resolution;
- autoload boundary design before automatic class loading;
- source mapping for magic constants across files;
- clear fallback policy for highly dynamic cases.

Exit criteria:

- Multi-file programs can run through `phpc run` for a documented subset.
- Include paths, repeated includes, and failure modes have CLI coverage.
- Namespace-aware functions, constants, and classes are tested.
- Unsupported autoload and dynamic resolution cases are named.

### 6. Exceptions and Error Model

Replace project-specific fatal shortcuts with more PHP-shaped error behavior
where it matters.

Scope:

- `throw`, `try`, `catch`, and `finally`;
- `Throwable`, `Exception`, and core error object boundaries;
- partial stdout and output buffering behavior;
- warnings/notices where PHP continues execution;
- stack traces or an explicit documented substitute.

Exit criteria:

- Exception control flow has fixtures for normal, thrown, caught, and finally
  paths.
- Runtime diagnostics distinguish project compiler errors from PHP runtime
  errors.
- Tests pin stdout/stderr/exit behavior.

### 7. Standard Library and Extensions

Expand builtins according to real program needs and extension plans, not by
surface-area guessing.

Scope:

- prioritize core functions used by fixtures and target applications;
- implement `json`, `mbstring`, XML, `curl`, `openssl`, database, cache, and
  compression extension slices only behind focused docs in `docs/extensions/`;
- keep unsupported extension functions and constants documented.

Exit criteria:

- Every builtin or extension slice has signature docs, type/error behavior,
  fixture coverage, and unsupported edge cases.
- Extension support claims are scoped to executable code.
- Dynamic function-call availability is tested when a builtin is callable.

### 8. Application Compatibility Targets

Choose small real PHP programs as compatibility targets once the runtime is
ready enough.

Scope:

- start with self-contained scripts using arrays, functions, classes, includes,
  and selected core builtins;
- then move to framework-shaped examples;
- treat WordPress compatibility as a later explicit target, not an implied
  current capability.

Exit criteria:

- Each target has a committed fixture or sample app.
- `phpc run` behavior is compared against system PHP where feasible.
- Native compile behavior is either supported, partially supported with
  documented fallback, or rejected with exact diagnostics.

### 9. Release-Quality Toolchain

Make the compiler usable outside the test suite.

Scope:

- stable CLI UX and exit codes;
- structured diagnostics with source spans;
- installable binary packaging;
- reproducible backend discovery;
- benchmark and smoke-test commands;
- contributor docs that explain how to add a PHP feature correctly.

Exit criteria:

- A user can install/build `phpc`, run examples, inspect support docs, and
  understand unsupported failures.
- CI or local automation runs the full test suite and representative CLI
  smoke tests.
- Release notes distinguish interpreter, IR, assembly, and native-executable
  support.

## Near-Term Queue

Start from `docs/NEXT_TASKS.md`; it is the source of truth for checked and
unchecked work. The active split-lane queue is:

- Milestone 610: compiler-output lane, choose the next deterministic CLI
  artifact or backend contract coverage target from existing native output
  behavior.
- Milestone 613: runtime lane, choose the next small array/object builtin
  refinement from documented unsupported gaps.
- Milestone 612: parser lane, choose the next small syntax or parse-diagnostic
  boundary without widening runtime or native support claims.
- Milestone 611: IR/lowering lane, choose the next narrow native IR or lowering
  refinement, or a precise rejection boundary, from already documented
  interpreter behavior.
- Milestone 614: tests/docs lane, keep the lane queue, focused-test policy,
  support matrix, and progress records current after the active implementation
  batch.

Prefer depth in one documented subset over breadth across unimplemented PHP
features.

## Verification Commands

Use focused commands while developing:

```sh
cargo test -p php_runtime
cargo test -p phpc
cargo run -p phpc -- test
cargo run -p phpc -- test --compare-php tests/fixtures/milestoneN
cargo run -p phpc -- run tests/fixtures/milestoneN/example.php
cargo run -p phpc -- compile tests/fixtures/milestoneN/example.php --emit-ir
cargo run -p phpc -- compile tests/fixtures/milestoneN/example.php --emit-asm
```

Use the full gate before checkpointing or after an integration batch that
combines lane work:

```sh
tools/run-tests.sh
```

When running unattended loop automation, checkpoint only through:

```sh
tools/checkpoint.sh "short accurate message"
```

## Non-Goals For Now

- Full PHP compatibility claims.
- WordPress compatibility claims.
- Zend extension loading.
- Regex parser rewrites.
- A custom x86-64 backend before the LLVM/runtime path is justified.
- Silent native compilation of constructs that only the interpreter supports.
- Broad standard-library stubs that return placeholder values.
