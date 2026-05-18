# Lane 1686-D Review: Dynamic Non-Direct Holder Property-Held ArrayAccess Append `offsetSet(null)` Bucket Reference Slots

Lane 1686-C landed the direct array-held dynamic non-direct holder append
subset: `$holders["box"]->{$name}[] = $array` now preserves stored-bucket
nested reference slots for the exact empty-string-key and branchy append-key
`offsetSet(null, $value)` bridges.

## Current Baseline

- Lane 1682-C landed direct `$bag[] = $array` stored-bucket reference-slot
  propagation for public `offsetSet(null, $value)` on a direct `ArrayAccess`
  object.
- Lane 1683-C landed direct visible named property-held append stores such as
  `$holder->bag[] = $array`.
- Lane 1684-C landed direct visible dynamic property-held append stores such
  as `$holder->{$name}[] = $array`.
- Lane 1685-C landed non-direct holder visible named property-held append
  stores such as `$holders["box"]->bag[] = $array`, plus the narrow visible
  named setup write `$holders["box"]->bag = $value`.
- Current support docs move only the proven append-store shape out of the
  unsupported bucket. Dynamic non-direct whole-property setup assignment
  remains unsupported; fixtures use the Lane 1685 named setup write.

## Landed Slice

- Preserves nested reference slots when a non-direct holder expression evaluates
  once to an object whose evaluated visible dynamic property holds an
  `ArrayAccess` object and receives an append write, for example
  `$holders["box"]->{$name}[] = $array`.
- Evaluates both the holder expression and the dynamic property-name expression
  exactly once. The selected property must route through the same concrete
  public/context visibility rules used by the existing dynamic-property and
  non-direct-holder paths.
- Covers the same two public `offsetSet(null, $value)` storage bridges as the
  landed append lanes:
  - exact bridge: `$this->property[$offset] = $value;`, where PHP's null
    offset stores under the backing array's empty-string key;
  - branchy append bridge: `if ($offset === null) { $this->property[] =
    $value; return; } $this->property[$offset] = $value;`, where metadata
    must attach to the actual appended integer key after the method call.
- If the appended value is an array literal containing reference elements,
  binds those nested reference slots onto the stored backing bucket.
- Proves that later exact `offsetGet($offset) { return
  $this->property[$offset]; }` bucket copies preserve those nested reference
  slots while ordinary copied fields remain detached.
- Native lowering continues to reject this receiver shape through the
  existing object-property or ArrayAccess lowering boundary.
- Dynamic non-direct setup assignment such as
  `$holders["box"]->{$name} = $value` remains unsupported.

## Landing Checks Run

- Focused Rust regression for
  `$holders["box"]->{$name}[] = $array` using the exact empty-string-key
  bridge, proving nested reference-slot write-through and ordinary copied-field
  detachment after a later exact `offsetGet()` copy: passed.
- Focused Rust regression for the same dynamic non-direct holder receiver using
  the branchy null-guard append bridge, proving metadata attaches to the actual
  appended integer key: passed.
- `CARGO_TARGET_DIR=/tmp/phpc-target-1686-check CARGO_BUILD_JOBS=1
  CARGO_INCREMENTAL=0 cargo check -q -p phpc`: passed.
- `CARGO_TARGET_DIR=/tmp/phpc-target-1686-focus CARGO_BUILD_JOBS=1
  CARGO_INCREMENTAL=0 cargo test -q -p phpc --test functions_and_scopes
  dynamic_non_direct_holder_property_held_array_access -- --test-threads=1`:
  passed `4` tests.
- `CARGO_TARGET_DIR=/tmp/phpc-target-1686-compare CARGO_BUILD_JOBS=1
  CARGO_INCREMENTAL=0 cargo run -q -p phpc -- test --compare-php
  tests/fixtures/milestone1686`: passed `2` fixtures with `2` system PHP
  comparisons and `0` skips.
- Adjacent assignment-expression, syntax-boundary, object-model, native
  lowering, reference-boundary, `milestone1684`, and `milestone1685` checks:
  passed.

## Still Needed Outside This Subset

- Evaluate-once guard for the non-direct holder expression, preferably using a
  counted helper or method-return holder shape once that parser/runtime path is
  intentionally included in the lane.
- Evaluate-once guard for the dynamic property-name expression, including a
  side-effecting name expression that still resolves to a visible concrete
  property.
- Unsupported-shape guard for magic-property append stores such as
  `$holders["box"]->missing[] = $array` or dynamic names that trigger magic
  fallback, ensuring this lane does not silently overclaim magic receiver
  support.

## Unsupported Edges To Keep Named

- Magic-property append-store roots such as
  `$holders["box"]->missing[] = $array`, `$box->missing[] = $array`, and
  `$box->{$name}[] = $array`.
- Dynamic property names that resolve to inaccessible properties, trigger
  magic fallback, or do not reach a visible concrete property holding an
  `ArrayAccess` object.
- Method-return or factory holder roots such as
  `$registry->holder()->{$name}[] = $array` and
  `make_holder($bag)->{$name}[] = $array`, unless the lane explicitly includes
  and tests them.
- Non-empty nested append paths such as
  `$holders["box"]->{$name}["outer"][] = $array`.
- Side-effecting, reordered, guarded, nested, or otherwise broader
  `offsetSet()` bodies beyond the exact bridge and covered null-guard append
  bridge.
- Broader `offsetGet()` bodies beyond the exact
  `return $this->property[$offset];` bridge used to prove later bucket copies.
- Appended values produced by arbitrary expressions with untracked reference
  provenance.
- Mixed nested `ArrayAccess` chains and append stores below an `ArrayAccess`
  object returned from another `ArrayAccess` lookup.
- Broad alias lifetime after replacing the dynamic holder property or replacing
  the non-direct holder container.
- Full PHP references/COW, native reference lowering, exact alias
  destruction/destructor ordering, and native `compile` support.

## Docs To Update After Landing

- `docs/ARCHITECTURE.md`: extend the non-direct holder property-held append
  stored-bucket discussion to dynamic property names, including evaluate-once
  rules for the holder and property-name expressions.
- `docs/SUPPORT.md`: move only the proven dynamic non-direct holder shape out
  of the unsupported append-store bucket and name the exact receiver,
  visibility, `offsetSet()`, and `offsetGet()` bridge limits.
- `docs/PROGRESS.md`: record the implementation summary, focused Rust tests,
  fixture comparison, and remaining unsupported edges.
- `docs/NEXT_TASKS.md`: mark the lane complete only after tests and fixtures
  pass, then update the remaining receiver ladder without dropping magic
  property and broader-body boundaries.
- This review note: convert from target guidance to landing summary, preserving
  any remaining variants outside the implemented subset.

## Landing Gate

- `cargo fmt --check`
- `git diff --check`
- Focused `functions_and_scopes` filters for the new dynamic non-direct holder
  append receiver and adjacent Lane 1684/Lane 1685 regressions.
- `cargo run -q -p phpc -- test --compare-php tests/fixtures/milestone1686`
- Broader verification as warranted if the implementation touches shared
  parser, assignment-target, property-holder, or ArrayAccess helper code.
