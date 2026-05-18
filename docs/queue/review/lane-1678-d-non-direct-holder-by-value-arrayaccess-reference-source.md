# Lane 1678-D Review: Non-Direct Holder By-Value `ArrayAccess::offsetGet()` Reference Sources

This is review/triage only. Runtime and test edits observed during this review
were concurrent work and were not made by this note.

## Current Worktree Baseline

- Concurrent WIP edits exist in `compiler/src/interpreter.rs`,
  `compiler/tests/functions_and_scopes.rs`, and
  `tests/fixtures/milestone1678/`.
- The WIP adds non-direct holder coverage for forms such as
  `$alias =& $holders["box"]->bag[$key]`,
  `$alias =& $holders["box"]->{$property}["outer"]["slot"]`,
  `$alias =& $registry->holder()->bag["outer"]["slot"]`, and
  `$alias =& make_holder($bag)->bag["outer"]["slot"]` when the visible holder
  property contains an `ArrayAccess` object whose public by-value
  `offsetGet($offset)` has the exact
  `return $this->property[$offset];` body.
- The parser represents non-direct holder offset reference sources as
  `NonDirectObjectPropertyNestedArrayIndex` /
  `NonDirectDynamicObjectPropertyNestedArrayIndex`, including the single-offset
  spelling, so the runtime branch must handle both scalar and nested paths.
- An earlier standalone by-value probe shape would have consumed those
  non-direct source branches when it returned `None`, skipping the existing
  alias fallback for by-reference `offsetGet()` and plain property-array
  sources. The current WIP has moved to a combined
  `NonDirectReferenceSourceBinding` helper, which is the safer shape.

## Smallest Safe Runtime Shape

Keep `evaluate_storable_reference_source_alias()` as an alias-only API and add a
small non-direct binding helper used only by `execute_reference_assignment()`:

- Evaluate the non-direct holder expression exactly once.
- Store the resulting object in the same private temporary-root style already
  used by the non-direct alias path.
- Evaluate dynamic property names exactly once before the helper call.
- In that helper, try the by-value exact `ArrayAccess` detached-value path
  first. On success, emit the existing bounded indirect-modification notice and
  write the target with `write_detached_static()`.
- If the by-value path returns `None`, immediately try the existing
  by-reference `ArrayAccess` alias bridge, then the magic `__get()` array bridge,
  then the plain visible property-array alias fallback.
- Return an explicit outcome such as `DetachedValue(Value)` or
  `ArrayOffset(ArrayOffsetAlias)` so the caller cannot accidentally bind a
  detached value as alias metadata.

This keeps real alias sources working while adding only the non-direct holder
by-value notice/no-op behavior.

## Risks

- The main regression risk is fallback loss. A by-value-only branch that returns
  `None` must not prevent by-reference `offsetGet()`, magic `__get()` array
  sources, or plain non-direct holder property-array sources from binding.
- The helper should not be reused from `evaluate_storable_reference_source_alias()`;
  that function is used by reference assignments into array slots and should
  continue to return only aliasable sources.
- Holder expressions and dynamic property expressions must not be evaluated
  twice, especially for method-return and expression-root holders.
- Returned arrays containing nested reference elements remain a broader COW
  nuance. This lane should claim detached outer value/no backing write only
  unless nested reference-slot parity is separately tested.
- Append-source non-direct holder forms are still a separate lane unless
  `offsetGet(null)` behavior is explicitly probed and covered.

## Verification Run

Using `CARGO_TARGET_DIR=/tmp/phpc-target-1678d-review CARGO_BUILD_JOBS=1
CARGO_INCREMENTAL=0`:

- `cargo test -q -p phpc --test functions_and_scopes reference_assignment_non_direct_holder_array_access_source_by_value_detaches_with_notice -- --test-threads=1`
  passed `1` test.
- `cargo run -q -p phpc -- test --compare-php tests/fixtures/milestone1678`
  passed `1` fixture with `1` system PHP comparison and `0` skips.
- A temporary by-reference guard for
  `$alias =& $holders["box"]->bag["slot"]` with by-reference
  `offsetGet()` produced `changed|changed` under the current WIP, matching PHP's
  aliasing behavior.
- A temporary plain property-array non-direct holder guard for
  `$alias =& $holders["box"]->items["slot"]` produced `changed|changed`, keeping
  the non-ArrayAccess alias fallback alive.

## Recommended Tests Before Landing

- Keep the current non-direct by-value Rust test and PHP-comparable fixture.
- Add a Rust regression for the by-reference `offsetGet()` guard above so the
  detached-value probe cannot steal real aliases.
- Add a Rust regression for a plain non-direct holder property-array source so
  the fallback path is protected.
- If magic `__get()` fallback is considered in scope, add a guard proving the
  combined helper still binds non-direct magic-property array sources after the
  by-value probe returns `None`.
- Run the focused non-direct tests, the existing direct/property-held
  by-value `ArrayAccess` tests, and `cargo run -q -p phpc -- test --compare-php
  tests/fixtures/milestone1678` before updating supported docs or progress.
