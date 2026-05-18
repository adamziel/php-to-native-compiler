# Lane 1680-D Review: Magic-Property Root By-Value `ArrayAccess` Reference Sources

This started as a review-only note while runtime/tests/docs edits were active
in parallel. The integration status below records what changed before the lane
was prepared for checkpoint.

## Integration Status

The missing helper-name findings below were resolved before landing. The
implemented helper is
`evaluate_magic_get_array_access_reference_source_binding(...)`, and the
callers now use it for selected and append magic-property roots. Focused
runtime tests for the by-value notice/no-op path and the by-reference
`offsetGet()`/plain-array fallback path pass, and
`tests/fixtures/milestone1680` provides PHP-comparable CLI coverage.

The append fallback for by-reference `__get()` returning a plain array was also
fixed to append at the returned array cell's next integer key instead of
binding the empty-string key used only by the exact `ArrayAccess::offsetGet(null)`
bridge.

## Blocking Findings

- `compiler/src/interpreter.rs:8856`, `compiler/src/interpreter.rs:8936`,
  `compiler/src/interpreter.rs:9018`, `compiler/src/interpreter.rs:9096`,
  `compiler/src/interpreter.rs:10897`: the new call sites reference
  `evaluate_magic_get_array_reference_source_binding(...)`, but no such method
  exists. The old available helper was
  `evaluate_magic_get_array_reference_source_alias(...)`, and the only new
  binding helper currently present is
  `evaluate_magic_get_array_access_reference_source_binding(...)`.
- `compiler/src/interpreter.rs:9171`, `compiler/src/interpreter.rs:9248`,
  `compiler/src/interpreter.rs:11052`: append fallbacks similarly call missing
  `evaluate_magic_get_array_append_reference_source_binding(...)`. This blocks
  the whole `phpc` crate from compiling, so the new magic-property root
  by-value `ArrayAccess` tests cannot execute.

The focused command fails during compilation:

```sh
CARGO_TARGET_DIR=/tmp/phpc-target-1680-d-review CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 \
  cargo test -q -p phpc --test functions_and_scopes \
  reference_assignment_magic_get_array_access_source -- --test-threads=1
```

Rust reports `E0599` for the missing helper methods listed above.

## Risk Check

- `__get()` once: the new ArrayAccess probe at
  `compiler/src/interpreter.rs:10798` calls `__get()` before it knows whether
  the returned value is an `ArrayAccess` object. If a later plain-array magic
  fallback is restored by calling `__get()` again instead of reusing the same
  returned value/cell, existing plain-array `__get()` paths will double-call
  `__get()`. Add an explicit side-effect-count assertion for both named and
  dynamic magic roots before landing.
- By-value and by-reference `__get()`: the new test covers both a by-value
  `__get()` returning an `ArrayAccess` object and a by-reference `__get()`
  returning the same object with by-value `offsetGet()`. That is the right
  behavioral surface, but it is not executable until the missing helpers are
  fixed.
- By-value `offsetGet()` ordering: the intended order is visible in the direct
  call sites: by-value `ArrayAccess` detachment is probed before alias
  fallbacks, and detached outcomes use `scope.write_detached_static(...)`.
  The compile break prevents verifying this through the magic-property root.
- By-reference `offsetGet()` and plain-array fallbacks: the new call sequence
  attempts to preserve by-reference `ArrayAccess` and plain-array magic
  fallbacks after the by-value probe, but the missing fallback binding helpers
  block compilation. Once restored, these need focused guard tests to prove
  by-reference `offsetGet()` still aliases and plain array `__get()` still
  aliases without double-calling `__get()`.
- Append `offsetGet(null)`: append roots route through
  `array_access_append_reference_key()` for the by-value probe, so the intended
  empty-string backing key is still the right shape. This is also blocked from
  execution by the missing append fallback helper.
- Docs: this WIP does not edit support/progress docs, so docs do not currently
  overclaim the new magic-property root by-value `ArrayAccess` behavior.
  Landing still needs docs/progress updates after executable proof exists.

## Checks Run

- `git diff --check` passed.
- Before the concurrent WIP changed the compile state, this nearby guard passed:
  `cargo test -q -p phpc --test functions_and_scopes magic_get_array_access -- --test-threads=1`
  passed `1` test.
- After the WIP landed, focused filters for
  `reference_assignment_array_access_source_by_value`,
  `reference_assignment_array_access_append_source_by_value`,
  `reference_assignment_non_direct_holder_array_access`, and
  `reference_assignment_magic_get_array_access_source` all failed at compile
  time with the same `E0599` missing-helper errors.

## Recommended Before Landing

- Add or correctly name the missing magic plain-array and append fallback
  binding helpers, then rerun the focused `functions_and_scopes` filters.
- Ensure the magic `__get()` result is evaluated once and reused across the
  by-value ArrayAccess probe, by-reference ArrayAccess fallback, and plain
  array fallback.
- Add a PHP-comparable CLI fixture for the direct magic-property root by-value
  offset and append forms, including `offsetGet(null)`, before updating support
  docs and progress.
