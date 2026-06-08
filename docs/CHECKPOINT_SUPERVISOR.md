# Supervisor Goal: checkpoint stabilization plus PHPT score recovery

## Outcome

- Land the current dirty checkpoint through `tools/checkpoint.sh`.
- In parallel, run as many bounded worker lanes as the current agent tool allows
  against distinct PHPT score clusters, each producing either a focused patch
  or a concrete no-go report.
- Prefer small, provable runnable-PHPT gains over broad claims.
- Update progress/support docs only for behavior actually proved by code and
  tests.

## Intensity

- Level: high.
- Requested workers: 10.
- Active tool cap observed: 6 subagents plus the supervisor lane.
- Scaling rule: keep the subagent pool full up to the observed cap while useful
  independent lanes exist; each lane must have a distinct domain, focused tests,
  and inspectable output. The supervisor owns final integration, full
  checkpoint, and public-score measurement.

## Non-Goals

- Do not broaden compiler support claims without implementation and tests.
- Do not replace parser/runtime architecture with fixture-specific shortcuts.
- Do not run network tunnels or expose local services.
- Do not checkpoint outside `tools/checkpoint.sh`.

## Ground Truth

- `AGENTS.md`
- `docs/PROGRESS.md`
- `docs/ARCHITECTURE.md`
- `docs/SUPPORT.md`
- `README.md`
- `/tmp/phpc-checkpoint-native-link-7.log`
- `compiler/tests/native_type_introspection_boundary.rs`
- current public score baseline: `5173 / 20294` pinned runnable PHPTs
  (`25.49%`)

## Worker Topology

- supervisor: owns the current checkpoint failure, integration, final
  test/doc/checkpoint gates, and public-score measurement.
- worker 01, array values/counting: array builtins likely to add runnable PHPTs.
- worker 02, string search/slice: string builtins and edge diagnostics.
- worker 03, filesystem/stat/open_basedir: local path helpers and warning
  parity.
- worker 04, type/introspection runtime: `is_*`, `gettype`,
  callable/class/function metadata in `phpc run`.
- worker 05, numeric/math: numeric conversions and math builtin PHPTs.
- worker 06, variable/symbol semantics: `isset`, `empty`, unset, globals, and
  superglobal-adjacent runnable PHPTs.
- worker 07, arrays mutation/sorting: `array_*` mutation/sort helpers with
  bounded semantics.
- worker 08, functions/callables: user functions, callable dispatch, argument
  diagnostics, and runnable callback PHPTs.
- worker 09, parser/unsupported triage: syntax fixtures blocked by parser
  boundaries; implement only a small safe subset.
- worker 10, score/evaluator: measure candidate score deltas, identify highest
  yielding fixture clusters, and reject non-score-moving churn.

## Quality Gates

- Focused worker tests for each accepted lane.
- `cargo fmt --check`
- `git diff --check`
- `CARGO_TARGET_DIR=/dev/shm/phpc-target-checkpoint-openbasedir CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 cargo test -p phpc --test native_link native_executable_c_source -- --test-threads=1`
- `CARGO_TARGET_DIR=/dev/shm/phpc-target-checkpoint-openbasedir CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 cargo test -p phpc --test native_link -- --test-threads=1`
- `tools/checkpoint.sh "checkpoint: stabilize native link assertions"`
- public-score measurement after checkpoint lands.

## Rejected Distractions

- Broad fixture refreshes without implementation.
- Refactors that do not unlock runnable PHPTs or unblock checkpoint.
- Native-lowering churn unrelated to current checkpoint failures.
- Public support claims without focused proof.

## Final Acceptance Criteria

- The current checkpoint either commits successfully through
  `tools/checkpoint.sh` or has an exact blocker log.
- At least one worker lane yields a reviewed candidate patch or an exact,
  evidence-backed no-go report.
- Public score is re-measured after the checkpoint/integration lane is stable.
