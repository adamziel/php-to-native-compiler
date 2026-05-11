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

## Loop Event 2026-05-11T22:56:48Z

- Codex round 3 exit code: `0`. Log: `/home/claude/php-to-native-compiler/.codex-yolo/logs/round-20260511T225324Z-3.log`.

## Loop Event 2026-05-11T22:56:49Z

- Post-round 3 test exit code: `0`.

## Loop Event 2026-05-11T22:56:49Z

- Post-round 3 tests passed; running checkpoint for this round.

## Loop Event 2026-05-11T22:56:51Z

- Starting round 4 at 20260511T225651Z from HEAD `d0fe00e`.

## Loop Event 2026-05-11T22:56:53Z

- Pre-round 4 test exit code: `0`.

## Loop Event 2026-05-11T23:06:19Z

- Task attempted: implemented the first Milestone 3 array slice: an ordered
  `php_runtime` array value with int/string key normalization, short array
  literal parsing/interpreter support, array `count`/`print_r`/`var_dump`
  behavior, invalid array-key diagnostics, and explicit native-codegen
  rejection for arrays.
- Files changed: `runtime/src/lib.rs`, `compiler/src/ast.rs`,
  `compiler/src/lexer.rs`, `compiler/src/parser.rs`,
  `compiler/src/interpreter.rs`, `compiler/src/codegen.rs`,
  `compiler/tests/milestone1.rs`, `compiler/tests/runtime_errors.rs`,
  `tests/fixtures/milestone3/array_literals.php`,
  `tests/fixtures/milestone3/array_literals.stdout`,
  `tests/fixtures/runtime_errors/unsupported_call.*`,
  `tests/fixtures/runtime_errors/unsupported_array_key.*`, `README.md`,
  `docs/ARCHITECTURE.md`, `docs/SUPPORT.md`, `docs/PROGRESS.md`,
  `docs/NEXT_TASKS.md`, and `docs/LOOP_MEMORY.md`.
- Tests run this round: `cargo test -p php_runtime array_` passed with 4
  focused array tests; `cargo test -p phpc --test runtime_errors` passed with
  7 runtime error tests; `cargo test -p phpc --test milestone1
  emit_ir_rejects_arrays_until_native_lowering_exists` passed; `cargo run -p
  phpc -- test tests/fixtures/milestone3` passed with 1 fixture; `cargo run -p
  phpc -- test --compare-php tests/fixtures/milestone3` passed with 1 system
  PHP comparison; `cargo run -p phpc -- test tests/fixtures/runtime_errors`
  passed with 6 fixtures; `cargo test -p phpc --test runtime_error_cli`
  passed; `cargo run -p phpc -- run
  tests/fixtures/runtime_errors/unsupported_array_key.php` exited `1` with the
  expected stable diagnostic; `cargo run -p phpc -- compile
  tests/fixtures/milestone3/array_literals.php --emit-ir` exited `1` with the
  expected array codegen rejection; `PATH=/nonexistent ./target/debug/phpc test
  --compare-php tests/fixtures/milestone3` passed with 1 PHP comparison
  skipped; `tools/run-tests.sh` passed with 24 fixtures, 18 system PHP
  comparisons, and 6 `.phpc-only` skips.
- Remaining semantic gaps: array indexing and array assignment syntax are still
  unsupported, including `$array[] = ...`, indexed reads, indexed writes,
  `unset`, `foreach`, long `array()` syntax, destructuring, spread, references,
  copy-on-write containers, object/resource keys, and PHP's full boolean/null/
  float key coercions. Negative-key auto-index behavior is not claimed beyond
  the current non-negative allocator, and arrays still reject native lowering.
- Next concrete task: implement array append assignment, indexed reads, and
  indexed writes in the interpreter with fixture CLI coverage.
- Checkpoint: pending `tools/checkpoint.sh "arrays: add ordered values and literals"`
  after the full suite passes.

## Loop Event 2026-05-11T23:07:19Z

- Codex round 4 exit code: `0`. Log: `/home/claude/php-to-native-compiler/.codex-yolo/logs/round-20260511T225651Z-4.log`.

## Loop Event 2026-05-11T23:07:21Z

- Post-round 4 test exit code: `0`.

## Loop Event 2026-05-11T23:07:21Z

- Post-round 4 tests passed; running checkpoint for this round.

## Loop Event 2026-05-11T23:07:23Z

- Starting round 5 at 20260511T230723Z from HEAD `4b78f07`.

## Loop Event 2026-05-11T23:07:25Z

- Pre-round 5 test exit code: `0`.

## Loop Event 2026-05-11T23:18:42Z

- Task attempted: implemented the next Milestone 3 array slice: parsed and
  interpreted direct array indexed reads, direct variable indexed writes, and
  direct variable append writes. Added undefined/`null` write materialization
  for the supported write subset, stable diagnostics for missing array keys and
  invalid array access, fixture coverage, and explicit native-codegen rejection
  for array indexing/assignment.
- Files changed: `runtime/src/lib.rs`, `compiler/src/ast.rs`,
  `compiler/src/parser.rs`, `compiler/src/interpreter.rs`,
  `compiler/src/codegen.rs`, `compiler/tests/runtime_errors.rs`,
  `compiler/tests/milestone1.rs`,
  `tests/fixtures/milestone3/array_indexing.php`,
  `tests/fixtures/milestone3/array_indexing.stdout`,
  `tests/fixtures/runtime_errors/undefined_array_key.php`,
  `tests/fixtures/runtime_errors/undefined_array_key.stderr`,
  `tests/fixtures/runtime_errors/undefined_array_key.exit`,
  `tests/fixtures/runtime_errors/undefined_array_key.phpc-only`,
  `tests/fixtures/runtime_errors/undefined_array_key.cli`, `README.md`,
  `docs/SUPPORT.md`, `docs/PROGRESS.md`, `docs/NEXT_TASKS.md`, and
  `docs/LOOP_MEMORY.md`.
- Tests run this round: `cargo test -p phpc --test runtime_errors` passed with
  9 runtime error tests; `cargo test -p phpc --test milestone1
  emit_ir_rejects_array` passed with array literal/indexing/assignment codegen
  rejection coverage; `cargo test -p php_runtime array_` passed with 4 focused
  array value tests; `cargo run -p phpc -- test tests/fixtures/milestone3`
  passed with 2 fixtures; `cargo run -p phpc -- test --compare-php
  tests/fixtures/milestone3` passed with 2 system PHP comparisons; `cargo run
  -p phpc -- test tests/fixtures/runtime_errors` passed with 7 fixtures;
  `cargo test -p phpc --test runtime_error_cli` passed; `cargo run -p phpc --
  run tests/fixtures/milestone3/array_indexing.php` printed the committed array
  indexing output; `cargo run -p phpc -- run
  tests/fixtures/runtime_errors/undefined_array_key.php` exited `1` with the
  expected stable diagnostic; `cargo run -p phpc -- compile
  tests/fixtures/milestone3/array_indexing.php --emit-ir` exited `1` with an
  explicit array codegen rejection; `PATH=/nonexistent ./target/debug/phpc test
  --compare-php tests/fixtures/milestone3` passed with 2 PHP comparisons
  skipped; `tools/run-tests.sh` passed with 26 fixtures, 19 system PHP
  comparisons, and 7 `.phpc-only` skips.
- Remaining semantic gaps: nested indexed writes, complex assignment lvalues,
  `$array[]` as a read expression, string offset access, `isset($array[key])`,
  `unset`, `foreach`, long `array()` syntax, destructuring, spread, references,
  copy-on-write containers, object/resource keys, PHP's full key coercion
  rules, missing-key warning/`null` recovery, and automatic conversion for
  existing non-array scalar write targets other than `null` are unsupported.
  Arrays, array indexing, and array assignment still reject native lowering.
- Next concrete task: separate local and global scope behavior for user
  functions, with tests for shadowing and unsupported `global`.
- Checkpoint: pending `tools/checkpoint.sh "arrays: add indexed reads and writes"`
  after the full suite passes.

## Loop Event 2026-05-11T23:14:39Z

- Codex round 5 exit code: `0`. Log: `/home/claude/php-to-native-compiler/.codex-yolo/logs/round-20260511T230723Z-5.log`.

## Loop Event 2026-05-11T23:14:41Z

- Post-round 5 test exit code: `0`.

## Loop Event 2026-05-11T23:14:41Z

- Post-round 5 tests passed; running checkpoint for this round.

## Loop Event 2026-05-11T23:14:42Z

- Starting round 6 at 20260511T231442Z from HEAD `2774b83`.

## Loop Event 2026-05-11T23:14:44Z

- Pre-round 6 test exit code: `0`.

## Loop Event 2026-05-11T23:19:16Z

- Task attempted: completed the first Milestone 4 function-scope slice by
  adding explicit local/global scope coverage for user functions, parsing
  `global` declarations as an unsupported statement with a stable runtime
  diagnostic, adding shadowing and implicit-global-read tests, and adding
  explicit native-codegen rejection coverage for `global`.
- Files changed: `runtime/src/lib.rs`, `compiler/src/ast.rs`,
  `compiler/src/lexer.rs`, `compiler/src/parser.rs`,
  `compiler/src/interpreter.rs`, `compiler/src/codegen.rs`,
  `compiler/tests/functions_and_scopes.rs`, `compiler/tests/milestone1.rs`,
  `tests/fixtures/milestone4/function_local_scope.php`,
  `tests/fixtures/milestone4/function_local_scope.stdout`,
  `tests/fixtures/runtime_errors/implicit_global_read.*`,
  `tests/fixtures/runtime_errors/unsupported_global.*`, `README.md`,
  `docs/ARCHITECTURE.md`, `docs/SUPPORT.md`, `docs/PROGRESS.md`,
  `docs/NEXT_TASKS.md`, and `docs/LOOP_MEMORY.md`.
- Tests run this round: `cargo test -p phpc --test functions_and_scopes`
  passed with 3 tests; `cargo test -p phpc --test runtime_errors` passed with
  9 tests; `cargo test -p phpc --test runtime_error_cli` passed; `cargo test
  -p phpc --test milestone1 emit_ir_rejects_global_declarations_until_scope_imports_exist`
  passed; `cargo run -p phpc -- test tests/fixtures/milestone4` passed with 1
  fixture; `cargo run -p phpc -- test --compare-php tests/fixtures/milestone4`
  passed with 1 system PHP comparison; `cargo run -p phpc -- test
  tests/fixtures/runtime_errors` passed with 9 fixtures; `cargo run -p phpc --
  run tests/fixtures/milestone4/function_local_scope.php` printed the committed
  scope-shadowing output; `cargo run -p phpc -- run
  tests/fixtures/runtime_errors/unsupported_global.php` exited `1` with the
  expected stable diagnostic; `cargo run -p phpc -- run
  tests/fixtures/runtime_errors/implicit_global_read.php` exited `1` with the
  expected stable diagnostic; `tools/run-tests.sh` passed with 29 fixtures, 20
  system PHP comparisons, and 9 `.phpc-only` skips.
- Remaining semantic gaps: `global` imports are not implemented, top-level
  variables are not visible inside functions unless passed as arguments, missing
  local reads fail with the current stable undefined-variable runtime error
  instead of PHP warning/`null` recovery, and default parameters, variadics,
  references, closures, dynamic calls, named arguments, strict types, static
  locals, recursion guards, and stack traces remain unsupported.
- Next concrete task: add recursion coverage and a documented runtime guard for
  runaway calls.
- Checkpoint: pending `tools/checkpoint.sh "functions: separate local and global scopes"`
  after the full suite passes.

## Loop Event 2026-05-11T23:19:48Z

- Codex round 6 exit code: `0`. Log: `/home/claude/php-to-native-compiler/.codex-yolo/logs/round-20260511T231442Z-6.log`.

## Loop Event 2026-05-11T23:19:50Z

- Post-round 6 test exit code: `0`.

## Loop Event 2026-05-11T23:19:50Z

- Post-round 6 tests passed; running checkpoint for this round.

## Loop Event 2026-05-11T23:19:52Z

- Starting round 7 at 20260511T231951Z from HEAD `b68c316`.

## Loop Event 2026-05-11T23:19:53Z

- Pre-round 7 test exit code: `0`.

## Loop Event 2026-05-11T23:22:51Z

- Task attempted: completed the Milestone 4 recursion slice by adding recursive
  user-function execution coverage, a fixed 128-frame user-function call-depth
  guard for runaway recursion, and stable runtime diagnostics with committed
  `phpc run` CLI snapshots.
- Files changed: `runtime/src/lib.rs`, `compiler/src/interpreter.rs`,
  `compiler/tests/functions_and_scopes.rs`, `compiler/tests/runtime_errors.rs`,
  `tests/fixtures/milestone4/recursive_factorial.php`,
  `tests/fixtures/milestone4/recursive_factorial.stdout`,
  `tests/fixtures/runtime_errors/runaway_recursion.php`,
  `tests/fixtures/runtime_errors/runaway_recursion.stderr`,
  `tests/fixtures/runtime_errors/runaway_recursion.exit`,
  `tests/fixtures/runtime_errors/runaway_recursion.phpc-only`,
  `tests/fixtures/runtime_errors/runaway_recursion.cli`, `README.md`,
  `docs/SUPPORT.md`, `docs/PROGRESS.md`, `docs/NEXT_TASKS.md`, and
  `docs/LOOP_MEMORY.md`.
- Tests run this round so far: `cargo test -p php_runtime
  call_depth_errors_keep_structured_kind_and_stable_message` passed;
  `cargo test -p phpc --test functions_and_scopes
  recursive_user_functions_can_return_values` passed; `cargo test -p phpc
  --test runtime_errors runaway_user_function_recursion_hits_stable_depth_guard`
  passed; `cargo run -p phpc -- test tests/fixtures/milestone4` passed with 2
  fixtures; `cargo run -p phpc -- test --compare-php tests/fixtures/milestone4`
  passed with 2 system PHP comparisons; `cargo run -p phpc -- test
  tests/fixtures/runtime_errors` passed with 10 fixtures; `cargo test -p phpc
  --test runtime_error_cli` passed; `cargo run -p phpc -- run
  tests/fixtures/milestone4/recursive_factorial.php` printed `120`; `cargo run
  -p phpc -- run tests/fixtures/runtime_errors/runaway_recursion.php` exited
  `1` with the expected stable diagnostic; `tools/run-tests.sh` passed with 31
  fixtures, 21 system PHP comparisons, and 10 `.phpc-only` skips.
- Remaining semantic gaps: the recursion guard is a fixed project limit, not
  PHP's native stack or memory exhaustion behavior; it is not configurable and
  does not produce stack traces. Function default parameters, variadics,
  references, closures, dynamic calls, named arguments, strict types, static
  locals, `global` imports, PHP warning/notice recovery, and preservation of
  partial stdout before fatal runtime errors remain unsupported.
- Next concrete task: implement default parameters for user functions with
  parser, runtime, and fixture coverage.
- Checkpoint: pending `tools/checkpoint.sh "functions: add recursion guard"`
  after the full suite passes.

## Loop Event 2026-05-11T23:24:08Z

- Codex round 7 exit code: `0`. Log: `/home/claude/php-to-native-compiler/.codex-yolo/logs/round-20260511T231951Z-7.log`.

## Loop Event 2026-05-11T23:24:09Z

- Post-round 7 test exit code: `0`.

## Loop Event 2026-05-11T23:24:09Z

- Post-round 7 tests passed; running checkpoint for this round.

## Loop Event 2026-05-11T23:24:11Z

- Starting round 8 at 20260511T232411Z from HEAD `98c1e8d`.

## Loop Event 2026-05-11T23:24:13Z

- Pre-round 8 test exit code: `0`.

## Loop Event 2026-05-11T23:28:42Z

- Task attempted: implemented Milestone 4 default parameters for user
  functions over a documented constant-expression subset. Function parameters
  now carry optional default expressions, parser validation rejects
  non-constant defaults and required parameters after defaults, user-function
  calls accept the required-to-total arity range, omitted arguments bind to
  defaults, and arity errors report the supported range.
- Files changed: `compiler/src/ast.rs`, `compiler/src/parser.rs`,
  `compiler/src/interpreter.rs`, `compiler/tests/functions_and_scopes.rs`,
  `tests/fixtures/milestone4/default_parameters.php`,
  `tests/fixtures/milestone4/default_parameters.stdout`, `README.md`,
  `docs/SUPPORT.md`, `docs/PROGRESS.md`, `docs/NEXT_TASKS.md`, and
  `docs/LOOP_MEMORY.md`.
- Tests run this round: `cargo fmt` passed; `cargo test -p phpc --test
  functions_and_scopes` passed with 8 tests; `cargo test -p phpc --test
  runtime_errors` passed with 10 tests; `cargo run -p phpc -- test
  tests/fixtures/milestone4` passed with 3 fixtures; `cargo run -p phpc -- test
  --compare-php tests/fixtures/milestone4` passed with 3 system PHP
  comparisons; `cargo run -p phpc -- run
  tests/fixtures/milestone4/default_parameters.php` printed the committed
  default-parameter output; `cargo test -p phpc --test milestone1
  emit_ir_rejects_global_declarations_until_scope_imports_exist` passed;
  `tools/run-tests.sh` passed with 32 fixtures, 22 system PHP comparisons, and
  10 `.phpc-only` skips.
- Remaining semantic gaps: default parameter support is limited to trailing
  defaults over the documented constant-expression subset. Variables, function
  calls, and indexed reads in default values are rejected by the parser.
  Required parameters after defaults are rejected instead of modeling PHP's
  deprecation and implicit-required behavior. Default expression runtime
  failures use stable project diagnostics rather than PHP `Throwable` objects
  and stack traces. Variadics, references, closures, dynamic calls, named
  arguments, strict types, static locals, and `global` imports remain
  unsupported.
- Next concrete task: document unsupported function features: variadics,
  references, closures, dynamic calls, named arguments, and strict types.
- Checkpoint: pending `tools/checkpoint.sh "functions: add default parameters"`
  after the full suite passes.

## Loop Event 2026-05-11T23:29:32Z

- Codex round 8 exit code: `0`. Log: `/home/claude/php-to-native-compiler/.codex-yolo/logs/round-20260511T232411Z-8.log`.

## Loop Event 2026-05-11T23:29:34Z

- Post-round 8 test exit code: `0`.

## Loop Event 2026-05-11T23:29:34Z

- Post-round 8 tests passed; running checkpoint for this round.

## Loop Event 2026-05-11T23:29:35Z

- Starting round 9 at 20260511T232935Z from HEAD `2776c53`.

## Loop Event 2026-05-11T23:29:37Z

- Pre-round 9 test exit code: `0`.

## Loop Event 2026-05-11T23:34:07Z

- Task attempted: completed the Milestone 4 unsupported function-feature
  documentation slice by adding explicit parser diagnostics for variadic
  parameters, variadic argument unpacking, reference parameters/returns and
  reference expressions, anonymous functions, arrow functions, dynamic function
  calls through expressions, named arguments, and `declare(strict_types=1)`.
- Files changed so far: `compiler/src/lexer.rs`, `compiler/src/parser.rs`,
  `compiler/tests/functions_and_scopes.rs`,
  `compiler/tests/unsupported_function_features_cli.rs`,
  `tests/fixtures/unsupported_function_features/*`, `README.md`,
  `docs/SUPPORT.md`, `docs/PROGRESS.md`, `docs/NEXT_TASKS.md`, and
  `docs/LOOP_MEMORY.md`.
- Tests run so far: `cargo test -p phpc --test functions_and_scopes` passed
  with 18 tests; `cargo test -p phpc --test unsupported_function_features_cli`
  passed; `cargo run -p phpc -- test
  tests/fixtures/unsupported_function_features` passed with 6 fixtures;
  `cargo run -p phpc -- test --compare-php
  tests/fixtures/unsupported_function_features` passed with 6 PHP comparisons
  skipped; `cargo run -p phpc -- run
  tests/fixtures/unsupported_function_features/unsupported_named_argument.php`
  exited `1` with the expected parse diagnostic; `cargo run -p phpc -- run
  tests/fixtures/unsupported_function_features/unsupported_strict_types.php`
  exited `1` with the expected parse diagnostic; `tools/run-tests.sh` passed
  with 38 fixtures, 22 system PHP comparisons, and 16 `.phpc-only` skips.
- Remaining semantic gaps: variadics, references, closures and arrow functions,
  dynamic calls, named arguments, strict type mode, parameter/return type
  declarations, static locals, magic function constants, and PHP's full
  function-call/type-enforcement semantics are still unsupported.
- Next concrete task: introduce a materialized symbol table path for future
  variable variables without changing current static variable behavior.
- Checkpoint: pending `tools/checkpoint.sh "functions: document unsupported features"`
  after the full suite passes.

## Loop Event 2026-05-11T23:35:35Z

- Codex round 9 exit code: `0`. Log: `/home/claude/php-to-native-compiler/.codex-yolo/logs/round-20260511T232935Z-9.log`.

## Loop Event 2026-05-11T23:35:37Z

- Post-round 9 test exit code: `0`.

## Loop Event 2026-05-11T23:35:37Z

- Post-round 9 tests passed; running checkpoint for this round.

## Loop Event 2026-05-11T23:35:39Z

- Starting round 10 at 20260511T233539Z from HEAD `bf02a0c`.

## Loop Event 2026-05-11T23:35:42Z

- Pre-round 10 test exit code: `0`.

## Loop Event 2026-05-11T23:40:39Z

- Task attempted: introduced a materialized interpreter symbol table for
  future variable-variable support without changing current static variable
  behavior. Static variable reads, writes, `isset($name)`, parameter binding,
  default-parameter evaluation, and direct array write materialization now route
  through named symbol-table APIs. Added a stable lex diagnostic and CLI
  snapshot for unsupported variable-variable syntax such as `$$name`.
- Files changed: `compiler/src/interpreter.rs`, `compiler/src/lexer.rs`,
  `compiler/tests/dynamic_features.rs`,
  `compiler/tests/unsupported_dynamic_features_cli.rs`,
  `tests/fixtures/milestone5/symbol_table_static_variables.php`,
  `tests/fixtures/milestone5/symbol_table_static_variables.stdout`,
  `tests/fixtures/unsupported_dynamic_features/unsupported_variable_variable.*`,
  `README.md`, `docs/ARCHITECTURE.md`, `docs/SUPPORT.md`,
  `docs/PROGRESS.md`, `docs/NEXT_TASKS.md`, and `docs/LOOP_MEMORY.md`.
- Tests run this round: `cargo fmt` passed; `cargo test -p phpc
  interpreter::tests::symbol_table` passed with 3 focused symbol-table tests;
  `cargo test -p phpc --test dynamic_features` passed; `cargo test -p phpc
  --test unsupported_dynamic_features_cli` passed; `cargo run -p phpc -- test
  tests/fixtures/milestone5` passed with 1 fixture; `cargo run -p phpc -- test
  --compare-php tests/fixtures/milestone5` passed with 1 system PHP
  comparison; `cargo run -p phpc -- test
  tests/fixtures/unsupported_dynamic_features` passed with 1 fixture; `cargo
  run -p phpc -- test --compare-php
  tests/fixtures/unsupported_dynamic_features` passed with 1 `.phpc-only` PHP
  comparison skipped; `cargo run -p phpc -- run
  tests/fixtures/milestone5/symbol_table_static_variables.php` printed the
  committed static symbol-table output; `cargo run -p phpc -- run
  tests/fixtures/unsupported_dynamic_features/unsupported_variable_variable.php`
  exited `1` with the expected stable diagnostic; `tools/run-tests.sh` passed
  with 40 fixtures, 23 system PHP comparisons, and 17 `.phpc-only` skips.
- Remaining semantic gaps: variable variables still do not execute; `$$name`
  and `${...}` fail with the current lex diagnostic instead of resolving a
  runtime-computed symbol name. Dynamic symbol lookup from PHP values,
  references, copy-on-write symbol containers, `global` imports, `GLOBALS`,
  superglobals, and include/eval scope effects remain unsupported.
- Next concrete task: design include/require resolution rules and add explicit
  unsupported diagnostics before implementing execution.
- Checkpoint: pending `tools/checkpoint.sh "runtime: add materialized symbol table"`
  after the full suite passes.

## Loop Event 2026-05-11T23:41:25Z

- Codex round 10 exit code: `0`. Log: `/home/claude/php-to-native-compiler/.codex-yolo/logs/round-20260511T233539Z-10.log`.

## Loop Event 2026-05-11T23:41:27Z

- Post-round 10 test exit code: `0`.

## Loop Event 2026-05-11T23:41:27Z

- Post-round 10 tests passed; running checkpoint for this round.

## Loop Event 2026-05-11T23:41:29Z

- Starting round 11 at 20260511T234129Z from HEAD `274d0fa`.

## Loop Event 2026-05-11T23:41:31Z

- Pre-round 11 test exit code: `0`.

## Loop Event 2026-05-11T23:49:58Z

- Task attempted: designed the first include/require resolution boundary and
  added explicit unsupported diagnostics for `include`, `include_once`,
  `require`, and `require_once`. The lexer now reserves those constructs, the
  parser rejects statement and expression forms with stable parse diagnostics,
  and unsupported dynamic-feature fixtures record committed `phpc run` CLI
  snapshots.
- Files changed: `compiler/src/lexer.rs`, `compiler/src/parser.rs`,
  `compiler/tests/dynamic_features.rs`,
  `tests/fixtures/unsupported_dynamic_features/unsupported_include.*`,
  `tests/fixtures/unsupported_dynamic_features/unsupported_include_once.*`,
  `tests/fixtures/unsupported_dynamic_features/unsupported_require.*`,
  `tests/fixtures/unsupported_dynamic_features/unsupported_require_once_expression.*`,
  `README.md`, `docs/ARCHITECTURE.md`, `docs/SUPPORT.md`,
  `docs/PROGRESS.md`, `docs/NEXT_TASKS.md`, and `docs/LOOP_MEMORY.md`.
- Tests run this round: `cargo test -p phpc --test dynamic_features` passed
  with 3 tests; `cargo test -p phpc --test unsupported_dynamic_features_cli`
  passed; `cargo run -p phpc -- test tests/fixtures/unsupported_dynamic_features`
  passed with 5 fixtures; `cargo run -p phpc -- test --compare-php
  tests/fixtures/unsupported_dynamic_features` passed with 5 `.phpc-only`
  PHP comparisons skipped; `cargo run -p phpc -- run
  tests/fixtures/unsupported_dynamic_features/unsupported_require_once_expression.php`
  exited `1` with the expected stable parse diagnostic; `tools/run-tests.sh`
  passed with 44 fixtures, 23 system PHP comparisons, and 21 `.phpc-only`
  skips.
- Remaining semantic gaps: include/require execution is not implemented.
  Include path lookup, current-working-directory fallback, stream wrappers, URL
  includes, `phar://`, `_once` de-duplication, caller-scope file execution,
  included-file return values, opcache/autoload interactions, and PHP's exact
  warning-vs-fatal recovery behavior remain unsupported.
- Next concrete task: add runtime lookup infrastructure for dynamic function
  calls and keep unresolved calls as explicit runtime errors.
- Checkpoint: pending `tools/checkpoint.sh "dynamic: reject include require"`
  after the full suite passes.

## Loop Event 2026-05-11T23:45:58Z

- Codex round 11 exit code: `0`. Log: `/home/claude/php-to-native-compiler/.codex-yolo/logs/round-20260511T234129Z-11.log`.

## Loop Event 2026-05-11T23:46:00Z

- Post-round 11 test exit code: `0`.

## Loop Event 2026-05-11T23:46:00Z

- Post-round 11 tests passed; running checkpoint for this round.

## Loop Event 2026-05-11T23:46:02Z

- Starting round 12 at 20260511T234602Z from HEAD `cd26615`.

## Loop Event 2026-05-11T23:46:04Z

- Pre-round 12 test exit code: `0`.

## Loop Event 2026-05-11T23:53:03Z

- Task attempted: added runtime lookup infrastructure for dynamic function
  calls through string-valued expressions. Dynamic calls now parse as runtime
  calls, resolve case-insensitively to user-defined functions or the documented
  callable builtin subset, and keep unresolved names and non-string callees as
  stable runtime errors. Native lowering still rejects function calls.
- Files changed: `compiler/src/ast.rs`, `compiler/src/parser.rs`,
  `compiler/src/interpreter.rs`, `compiler/src/codegen.rs`,
  `compiler/tests/dynamic_features.rs`,
  `compiler/tests/functions_and_scopes.rs`, `compiler/tests/milestone1.rs`,
  `tests/fixtures/milestone5/dynamic_function_lookup.*`,
  `tests/fixtures/runtime_errors/undefined_dynamic_function.*`,
  `tests/fixtures/runtime_errors/invalid_dynamic_callable.*`,
  `tests/fixtures/unsupported_function_features/unsupported_arrow_function.*`,
  removed the obsolete
  `tests/fixtures/unsupported_function_features/unsupported_dynamic_call.*`,
  and updated `README.md`, `docs/ARCHITECTURE.md`, `docs/SUPPORT.md`,
  `docs/PROGRESS.md`, `docs/NEXT_TASKS.md`, and `docs/LOOP_MEMORY.md`.
- Tests run this round: `cargo test -p phpc --test dynamic_features` passed
  with 6 tests; `cargo test -p phpc --test functions_and_scopes` passed with
  17 tests; `cargo test -p phpc --test milestone1` passed with 10 tests;
  `cargo test -p phpc --test runtime_error_cli` passed; `cargo test -p phpc
  --test unsupported_function_features_cli` passed; `cargo run -p phpc -- test
  tests/fixtures/milestone5` passed with 2 fixtures; `cargo run -p phpc -- test
  --compare-php tests/fixtures/milestone5` passed with 2 system PHP
  comparisons; `cargo run -p phpc -- test tests/fixtures/runtime_errors`
  passed with 12 fixtures; `cargo run -p phpc -- test
  tests/fixtures/unsupported_function_features` passed with 6 fixtures;
  `cargo run -p phpc -- test --compare-php
  tests/fixtures/unsupported_function_features` passed with 6 `.phpc-only`
  skips; `cargo run -p phpc -- run
  tests/fixtures/milestone5/dynamic_function_lookup.php` printed the committed
  dynamic lookup output; `cargo run -p phpc -- run
  tests/fixtures/runtime_errors/undefined_dynamic_function.php` exited `1`
  with the expected stable undefined-function diagnostic; `cargo run -p phpc
  -- run tests/fixtures/runtime_errors/invalid_dynamic_callable.php` exited
  `1` with the expected stable unsupported-call diagnostic; `tools/run-tests.sh`
  passed with 47 fixtures, 24 system PHP comparisons, and 23 `.phpc-only`
  skips.
- Remaining semantic gaps: dynamic calls are limited to string-valued function
  names resolving to current user functions or callable builtins (`strlen`,
  `count`, `var_dump`, and `print_r`). Dynamic access to `isset`, array
  callables, object/method callables, first-class callable syntax,
  `call_user_func`, namespace-qualified callable resolution, autoload
  interaction, named arguments, argument unpacking, references, and native
  lowering are unsupported.
- Next concrete task: define the `eval` fallback boundary: parser entry point,
  caller scope behavior, diagnostics, and unsupported cases.
- Checkpoint: pending `tools/checkpoint.sh "dynamic: add function lookup"`
  after the full suite passes.

## Loop Event 2026-05-11T23:53:58Z

- Codex round 12 exit code: `0`. Log: `/home/claude/php-to-native-compiler/.codex-yolo/logs/round-20260511T234602Z-12.log`.

## Loop Event 2026-05-11T23:54:00Z

- Post-round 12 test exit code: `0`.

## Loop Event 2026-05-11T23:54:00Z

- Post-round 12 tests passed; running checkpoint for this round.

## Loop Event 2026-05-11T23:54:02Z

- Starting round 13 at 20260511T235402Z from HEAD `0fe210a`.

## Loop Event 2026-05-11T23:54:03Z

- Pre-round 13 test exit code: `0`.

## Loop Event 2026-05-11T23:56:48Z

- Task attempted: defined the first `eval` fallback boundary and added explicit
  unsupported diagnostics. Direct `eval(...)` is now reserved by the lexer,
  rejected by the parser in statement and expression positions, documented as a
  future caller-scope fallback zone, and covered by committed `phpc run` CLI
  snapshots.
- Files changed: `compiler/src/lexer.rs`, `compiler/src/parser.rs`,
  `compiler/tests/dynamic_features.rs`,
  `tests/fixtures/unsupported_dynamic_features/unsupported_eval.*`,
  `tests/fixtures/unsupported_dynamic_features/unsupported_eval_expression.*`,
  `README.md`, `docs/ARCHITECTURE.md`, `docs/SUPPORT.md`,
  `docs/PROGRESS.md`, `docs/NEXT_TASKS.md`, and `docs/LOOP_MEMORY.md`.
- Tests run this round: `cargo fmt` passed; `cargo test -p phpc --test
  dynamic_features` passed with 7 tests; `cargo test -p phpc --test
  unsupported_dynamic_features_cli` passed; `cargo run -p phpc -- test
  tests/fixtures/unsupported_dynamic_features` passed with 7 fixtures; `cargo
  run -p phpc -- test --compare-php
  tests/fixtures/unsupported_dynamic_features` passed with 7 `.phpc-only`
  skips; `cargo run -p phpc -- run
  tests/fixtures/unsupported_dynamic_features/unsupported_eval.php` exited `1`
  with the expected parse diagnostic; `cargo run -p phpc -- run
  tests/fixtures/unsupported_dynamic_features/unsupported_eval_expression.php`
  exited `1` with the expected parse diagnostic; `tools/run-tests.sh` passed
  with 49 fixtures, 24 system PHP comparisons, and 25 `.phpc-only` skips.
- Remaining semantic gaps: eval execution is not implemented. Eval-fragment
  parsing without `<?php`, caller-scope execution, `return` values from
  evaluated code, diagnostics inside evaluated strings, functions/classes
  declared from evaluated code, nested eval, include/require inside eval,
  references/copy-on-write interactions, `GLOBALS`/superglobals, namespaces/use
  declarations, opcache behavior, and exact PHP `ParseError`/warning behavior
  remain unsupported.
- Next concrete task: sketch the minimal object/class metadata model before
  adding syntax.
- Checkpoint: pending `tools/checkpoint.sh "dynamic: reject eval"` after the
  full suite passes.
