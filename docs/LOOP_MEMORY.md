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


## Loop Event 2026-05-11T22:38:26Z

- Starting round 1 at 20260511T223826Z from HEAD `f62535f`.

## Loop Event 2026-05-11T22:38:27Z

- Pre-round 1 test exit code: `0`.

## Loop Event 2026-05-11T22:44:47Z

- Task attempted: completed scalar arithmetic coercion coverage for `Null`,
  `Bool`, numeric strings, non-numeric strings, `Int`, and `Float`.
- Files changed: `runtime/src/lib.rs`, `compiler/tests/runtime_errors.rs`,
  `tests/fixtures/milestone2/scalar_arithmetic_coercions.php`,
  `tests/fixtures/milestone2/scalar_arithmetic_coercions.stdout`,
  `tests/fixtures/runtime_errors/non_numeric_string_arithmetic.php`,
  `tests/fixtures/runtime_errors/non_numeric_string_arithmetic.stderr`,
  `tests/fixtures/runtime_errors/non_numeric_string_arithmetic.exit`,
  `tests/fixtures/runtime_errors/non_numeric_string_arithmetic.phpc-only`,
  `docs/SUPPORT.md`, `docs/PROGRESS.md`, `docs/NEXT_TASKS.md`, and
  `docs/LOOP_MEMORY.md`.
- Tests run so far: `cargo test -p php_runtime` passed;
  `cargo test -p phpc --test runtime_errors` passed;
  `cargo test -p phpc --test php_comparison` passed;
  `cargo run -p phpc -- test tests/fixtures/milestone2` passed;
  `cargo run -p phpc -- test --compare-php tests/fixtures/milestone2` passed;
  `cargo run -p phpc -- test tests/fixtures/runtime_errors` passed;
  `PATH=/nonexistent ./target/debug/phpc test --compare-php tests/fixtures/milestone2`
  passed with 6 PHP comparisons skipped;
  `cargo run -p phpc -- run tests/fixtures/runtime_errors/non_numeric_string_arithmetic.php`
  exits `1` with the expected stable diagnostic; `tools/run-tests.sh` passed
  with 21 fixtures, 16 system PHP comparisons, and 5 `.phpc-only` skips.
- Remaining semantic gaps: leading numeric strings with trailing non-numeric
  text are rejected instead of warning and continuing; PHP warning/notice
  recovery and exact integer-overflow promotion are still unsupported.
- Next concrete task: add a scalar comparison behavior matrix for equality and
  relational operators across implemented value types.
- Checkpoint: pending `tools/checkpoint.sh "runtime: complete scalar arithmetic coercions"`
  after the full suite passes.

## Loop Event 2026-05-11T22:45:59Z

- Codex round 1 exit code: `0`. Log: `/home/claude/php-to-native-compiler/.codex-yolo/logs/round-20260511T223826Z-1.log`.

## Loop Event 2026-05-11T22:46:01Z

- Post-round 1 test exit code: `0`.

## Loop Event 2026-05-11T22:46:01Z

- Post-round 1 tests passed; running checkpoint for this round.

## Loop Event 2026-05-11T22:46:03Z

- Starting round 2 at 20260511T224603Z from HEAD `d7ef45c`.

## Loop Event 2026-05-11T22:46:05Z

- Pre-round 2 test exit code: `0`.

## Loop Event 2026-05-11T22:52:25Z

- Task attempted: added a PHP 8-style scalar comparison behavior matrix for
  `==`, `!=`, `<`, `<=`, `>`, and `>=` across implemented scalar values:
  `null`, booleans, integers, floats, empty strings, numeric strings, and
  non-numeric strings.
- Files changed: `runtime/src/lib.rs`,
  `tests/fixtures/milestone2/scalar_comparison_matrix.php`,
  `tests/fixtures/milestone2/scalar_comparison_matrix.stdout`, `README.md`,
  `docs/SUPPORT.md`, `docs/PROGRESS.md`, `docs/NEXT_TASKS.md`, and
  `docs/LOOP_MEMORY.md`.
- Tests run this round: `cargo test -p php_runtime scalar_comparison_matrix_matches_php_8_scalar_subset`
  passed; `cargo test -p php_runtime` passed with 7 runtime unit tests;
  `cargo run -p phpc -- test tests/fixtures/milestone2` passed with 7
  fixtures; `cargo run -p phpc -- test --compare-php tests/fixtures/milestone2`
  passed with 7 system PHP comparisons; `cargo run -p phpc -- run
  tests/fixtures/milestone2/scalar_comparison_matrix.php` printed the
  committed 100-row matrix; `PATH=/nonexistent ./target/debug/phpc test
  --compare-php tests/fixtures/milestone2` passed with 7 PHP comparisons
  skipped; `tools/run-tests.sh` passed with 22 fixtures, 17 system PHP
  comparisons, and 5 `.phpc-only` skips.
- Remaining semantic gaps: strict identity operators (`===`, `!==`), arrays,
  objects, resources, and comparison edge cases around `NAN`/`INF` and
  PHP-version-specific float string precision are unsupported. Leading numeric
  strings with trailing non-numeric text are still rejected for arithmetic
  instead of warning and continuing.
- Next concrete task: add explicit CLI exercises for representative runtime
  errors and record their stdout, stderr, and exit behavior.
- Checkpoint: pending `tools/checkpoint.sh "runtime: add scalar comparison matrix"`
  after the full suite passes.

## Loop Event 2026-05-11T22:53:20Z

- Codex round 2 exit code: `0`. Log: `/home/claude/php-to-native-compiler/.codex-yolo/logs/round-20260511T224603Z-2.log`.

## Loop Event 2026-05-11T22:53:22Z

- Post-round 2 test exit code: `0`.

## Loop Event 2026-05-11T22:53:22Z

- Post-round 2 tests passed; running checkpoint for this round.

## Loop Event 2026-05-11T22:53:24Z

- Starting round 3 at 20260511T225324Z from HEAD `f940788`.

## Loop Event 2026-05-11T22:53:26Z

- Pre-round 3 test exit code: `0`.

## Loop Event 2026-05-11T22:55:53Z

- Task attempted: added explicit `phpc run` CLI snapshots for representative
  runtime errors, recording exit code, stdout, and stderr for undefined
  variables, user-function arity mismatches, unsupported `count()` calls,
  division by zero, and non-numeric string arithmetic.
- Files changed: `compiler/tests/runtime_error_cli.rs`,
  `tests/fixtures/runtime_errors/arity_mismatch.cli`,
  `tests/fixtures/runtime_errors/invalid_arithmetic.cli`,
  `tests/fixtures/runtime_errors/non_numeric_string_arithmetic.cli`,
  `tests/fixtures/runtime_errors/undefined_variable.cli`,
  `tests/fixtures/runtime_errors/unsupported_call.cli`, `docs/SUPPORT.md`,
  `docs/PROGRESS.md`, `docs/NEXT_TASKS.md`, and `docs/LOOP_MEMORY.md`.
- Tests run this round: `cargo test -p phpc --test runtime_error_cli` passed;
  `tools/run-tests.sh` passed with 22 fixtures, 17 system PHP comparisons, and
  5 `.phpc-only` skips.
- Remaining semantic gaps: runtime errors are stable diagnostics, not PHP
  `Throwable` objects; stack traces, warning/notice recovery, user error
  handlers, and preservation of partial stdout before a fatal runtime error are
  unsupported.
- Next concrete task: implement an ordered PHP array value in `php_runtime` with
  int/string key normalization tests.
- Checkpoint: pending `tools/checkpoint.sh "runtime: add CLI error snapshots"`
  after the full suite passes.
