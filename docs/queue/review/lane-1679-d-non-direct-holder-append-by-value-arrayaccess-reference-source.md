# Lane 1679-D Review: Non-Direct Holder Append By-Value `ArrayAccess::offsetGet(null)` Reference Sources

This is review-only. Runtime, test, fixture, and support/progress docs were not
edited by this note. The active implementation is uncommitted concurrent WIP in
`compiler/src/interpreter.rs`, `compiler/tests/functions_and_scopes.rs`, and
`tests/fixtures/milestone1679/`.

## Integration Status

The duplicate test definitions reported below were a transient concurrent-lane
merge issue. They were deduplicated before landing this batch, and the focused
by-value append, by-reference append fallback, plain property-array append
fallback, full `functions_and_scopes`, and `milestone1679` fixture checks pass
in the integrated worktree.

## Blocking Findings

- `compiler/tests/functions_and_scopes.rs:3671`,
  `compiler/tests/functions_and_scopes.rs:3895`: the by-value non-direct holder
  append test is defined twice with the same Rust function name. This prevents
  the whole `functions_and_scopes` test target from compiling.
- `compiler/tests/functions_and_scopes.rs:3739`,
  `compiler/tests/functions_and_scopes.rs:3994`: the by-reference
  `offsetGet()` append guard is also duplicated and blocks compilation.
- `compiler/tests/functions_and_scopes.rs:3769`,
  `compiler/tests/functions_and_scopes.rs:4045`: the plain property-array
  append fallback guard is duplicated as well.

The focused command failed before running tests:

```sh
CARGO_TARGET_DIR=/tmp/phpc-target-1679-d-review CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 \
  cargo test -q -p phpc --test functions_and_scopes \
  reference_assignment_non_direct_holder_array_access_append_source -- --test-threads=1
```

Rust reported `E0428` for the three duplicate test names above.

The CLI fixture path itself did run successfully despite the Rust test-target
compile failure:

```sh
CARGO_TARGET_DIR=/tmp/phpc-target-1679-d-review CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 \
  cargo run -q -p phpc -- test --compare-php tests/fixtures/milestone1679
```

Result: `1` fixture passed, with `1` system PHP comparison and `0` skips.

## Risk Check

- Holder evaluation: the new helper at `compiler/src/interpreter.rs:10748`
  evaluates the holder expression once, stores the object in a temporary root,
  and reuses that root for the by-value, by-reference, magic, and plain append
  paths. The dynamic property branch at `compiler/src/interpreter.rs:9147`
  evaluates the property expression once before calling the helper.
- Fallback order: for true append sources with no suffix indices, the helper
  first probes the by-value exact `offsetGet(null)` detached path, then falls
  through to the by-reference append bridge, then magic `__get()` append, then
  the plain property-array append alias path. That matches the intended shape.
- Detached target: the caller writes `DetachedValue` outcomes with
  `scope.write_detached_static(...)` at `compiler/src/interpreter.rs:9131` and
  `compiler/src/interpreter.rs:9154`, so the by-value path does not install
  alias metadata into the target.
- Docs: the active diff does not update support/progress docs. Existing docs
  still describe non-direct append-source `ArrayAccess` forms as unsupported,
  so they do not overclaim, but the lane is not complete under repo rules until
  docs/progress are updated after tests compile.

## Recommended Before Landing

- Deduplicate or merge the three repeated Rust tests, then rerun the focused
  `functions_and_scopes` filter.
- Add an explicit side-effect-count assertion for method-return or factory
  holders if this lane wants executable proof that holder roots are evaluated
  once for append sources.
- Add the PHP-comparable CLI fixture/docs update only after the Rust test target
  compiles and the by-value, by-reference, magic/plain fallback guards pass.
