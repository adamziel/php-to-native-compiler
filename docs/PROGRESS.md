# Progress Log

## 2026-05-12

Implemented:

- Initialized the repository and Rust workspace.
- Added project rules, README, architecture, support, roadmap, extension status,
  and progress documentation.
- Implemented a stable Rust `php_runtime` crate with scalar boxed values:
  `Null`, `Bool`, `Int`, `Float`, and `String`.
- Implemented scalar echo conversion, truthiness, arithmetic, concatenation, and
  comparison helpers.
- Implemented the `phpc` compiler crate with lexer, parser, AST, interpreter,
  fixture test runner, and CLI.
- Implemented `phpc run <input.php>` for the Milestone 1 subset.
- Implemented `phpc compile <input.php> --emit-ir` for a narrow straight-line
  scalar subset using LLVM IR text.
- Implemented `phpc compile <input.php> --emit-asm`; it prefers `clang`/`llc`
  and currently falls back to `cc -S` through generated C when LLVM tools are not
  installed.
- Added Milestone 1 fixtures for echo, literals, variables, assignment,
  arithmetic, concatenation, `if`/`else`, `while`, function declaration, function
  call, and `return`.
- Added a small Milestone 2 scalar slice: `print` statements, unary minus,
  logical not, and fixtures for `null`/bool/string truthiness.
- Added optional `phpc test --compare-php [fixture-dir]` support. When system
  `php` is installed it compares fixture stdout, stderr, and exit code against
  `phpc run`; when `php` is absent it skips comparison and still runs committed
  expected-output fixtures.
- Added two narrow Milestone 2 scalar comparison fixtures for echo conversion,
  truthiness, and numeric-string arithmetic. This does not mark broader
  Milestone 2 support complete.
- Added scalar builtin support for `strlen`, `isset`, `var_dump`, and `print_r`
  with fixture coverage. Array/object behavior for these functions remains
  unsupported.
- Added operational automation: `tools/checkpoint.sh`, `tools/codex-loop.sh`,
  `docs/OPERATIONS.md`, `docs/NEXT_TASKS.md`, and
  `docs/CODEX_LOOP_PROMPT.md`.
- Added structured runtime error categories and stable messages for undefined
  variables, arity mismatches, unsupported calls, and division by zero.
- Changed plain undefined variable reads to fail with a runtime error; direct
  `isset($name)` checks remain supported and return false for missing/null
  variables.
- Added `.phpc-only` fixture markers so project-specific runtime diagnostics can
  be exercised by the fixture runner without being compared to system PHP.

Tested:

- `cargo test` passes.
- `cargo test -p phpc --test runtime_errors` passes with 5 runtime error tests.
- `cargo run -p phpc -- test` passes with 19 fixture tests.
- `cargo run -p phpc -- test --compare-php` passes with system `php`
  installed, comparing 15 fixtures and skipping 4 `.phpc-only` fixtures.
- `cargo run -p phpc -- test --compare-php tests/fixtures/milestone2` passes
  with system `php` installed, comparing 5 Milestone 2 fixtures.
- `PATH=/nonexistent ./target/debug/phpc test --compare-php tests/fixtures/milestone2`
  passes, reporting 5 PHP comparisons skipped.
- `cargo run -p phpc -- test tests/fixtures/runtime_errors` passes with 4
  runtime error fixtures.
- `cargo run -p phpc -- run tests/fixtures/runtime_errors/undefined_variable.php`
  exits 1 and reports `runtime error at tests/fixtures/runtime_errors/undefined_variable.php:2:6: undefined variable '$missing'`.
- `tools/run-tests.sh` passes and now includes optional system PHP comparison.
- `cargo run -p phpc -- run examples/hello.php` prints `hello`.
- `cargo run -p phpc -- compile tests/fixtures/milestone1/basic_arithmetic.php --emit-ir`
  emits LLVM IR containing native arithmetic and `printf` calls.
- `cargo run -p phpc -- compile tests/fixtures/milestone1/basic_arithmetic.php --emit-asm`
  emits native assembly through the available `cc` fallback in this environment.

Still fails:

- No known failing tests.
- `--emit-asm` does not use LLVM tools in this environment because neither
  `clang` nor `llc` is installed; the documented `cc -S` fallback is used.
- LLVM/assembly lowering intentionally rejects functions, calls, control flow,
  comparisons, dynamic values, and unknown variables.

Next:

- Continue Milestone 2 by expanding scalar coercions, runtime errors, and PHP
  behavior comparison coverage.
