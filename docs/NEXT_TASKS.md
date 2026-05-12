# Next Tasks

Use this as the queue for repeated Codex work. Take one unchecked task at a
time unless a task explicitly depends on another. Mark a task checked only after
code, tests, CLI coverage, documentation, and unsupported edge cases are all
handled.

## Milestone 2: Value Model and Runtime

- [x] Add structured runtime errors with stable messages and tests for at
  least undefined variables, arity mismatches, unsupported calls, and invalid
  arithmetic.
- [x] Complete scalar arithmetic coercion coverage for `Null`, `Bool`, numeric
  strings, non-numeric strings, `Int`, and `Float`; add fixture tests and
  document the remaining gaps from PHP's full coercion rules.
- [x] Add a scalar comparison behavior matrix for equality and relational
  operators across implemented value types, with runtime tests and fixture CLI
  coverage.
- [x] Complete optional system PHP comparison mode for the fixture runner,
  including docs, progress notes, and gating so the suite still passes when
  `php` is not installed.
- [x] Add explicit CLI exercises for representative runtime errors and record
  their stdout, stderr, and exit behavior.

## Milestone 3: Arrays

- [x] Implement an ordered PHP array value in `php_runtime` with int/string key
  normalization tests.
- [x] Parse and interpret short array literals `[]` and `[key => value]` for the
  supported scalar expression subset.
- [x] Implement array append, indexed reads, and indexed writes in the
  interpreter with fixture CLI coverage.
- [x] Document unsupported array behavior, including references, nested
  copy-on-write containers, destructuring, spread, and complex key coercions.
- [x] Add a small native-codegen rejection test proving arrays fail with an
  explicit codegen error until lowering exists.

## Milestone 4: Functions and Scopes

- [x] Separate local and global scope behavior for user functions, with tests
  for shadowing and unsupported `global`.
- [x] Add recursion coverage and a documented runtime guard for runaway calls.
- [x] Implement default parameters for user functions with parser, runtime, and
  fixture coverage.
- [x] Add the first small builtin function set with documented signatures,
  errors, tests, and CLI examples.
- [x] Document unsupported function features: variadics, references, closures,
  named arguments, strict types, and dynamic-call gaps outside the current
  string-name lookup subset.

## Milestone 5+: Dynamic PHP

- [x] Introduce a materialized symbol table path for future variable variables
  without changing current static variable behavior.
- [x] Design include/require resolution rules and add explicit unsupported
  diagnostics before implementing execution.
- [x] Add runtime lookup infrastructure for dynamic function calls and keep
  unresolved calls as explicit runtime errors.
- [x] Define the `eval` fallback boundary: parser entry point, caller scope
  behavior, diagnostics, and unsupported cases.
- [x] Sketch the minimal object/class metadata model before adding syntax.
- [ ] Parse class declarations into a metadata registry while keeping object
  instantiation and member access unsupported.
