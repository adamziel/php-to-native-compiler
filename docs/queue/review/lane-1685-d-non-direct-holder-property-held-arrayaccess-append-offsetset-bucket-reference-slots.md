# Lane 1685-D Review: Non-Direct Holder Property-Held ArrayAccess Append `offsetSet(null)` Bucket Reference Slots

Lane 1685-C landed the named visible property subset of this review target.
Runtime, tests, fixtures, support docs, and progress docs now cover
`$holders["box"]->bag[] = $array` for the exact empty-string-key and branchy
append-key `offsetSet(null, $value)` bridges.

## Current Baseline

- Lane 1682-C landed direct `$bag[] = $array` stored-bucket reference-slot
  propagation for public `offsetSet(null, $value)` on a direct `ArrayAccess`
  object.
- Lane 1683-C landed direct visible named property-held append stores such as
  `$holder->bag[] = $array` for the exact empty-string-key bridge and branchy
  append-key bridge.
- Lane 1684-C landed direct visible dynamic property-held append stores such
  as `$holder->{$name}[] = $array` for the same two bridges.
- `docs/SUPPORT.md`, `docs/ARCHITECTURE.md`, `docs/PROGRESS.md`, and
  `docs/NEXT_TASKS.md` still name non-direct holder and magic-property append
  `offsetSet(null)` stored-bucket receivers as unsupported. Keep that boundary
  until the runtime and tests prove this lane.
- Existing non-direct holder `ArrayAccess` paths for selected reads,
  by-reference arguments, by-reference foreach, and copied-bucket provenance do
  not prove append-store metadata is attached to the stored backing bucket.

## Landed Slice

- Preserves nested reference slots when a non-direct holder expression evaluates
  once to an object whose visible property holds an `ArrayAccess` object and
  receives an append write, currently covered by `$holders["box"]->bag[] =
  $array`.
- Supports the narrow visible named setup assignment
  `$holders["box"]->bag = $value`, used to establish the held `ArrayAccess`
  object for the fixture path.
- Keeps the selected holder property concrete and visible in the current
  context.
- Reuses the same focused `offsetSet(null, $value)` storage bridges as the
  direct lanes:
  - exact bridge: `$this->property[$offset] = $value;`, where PHP's null
    offset stores under the backing array's empty-string key;
  - branchy append bridge: `if ($offset === null) { $this->property[] =
    $value; return; } $this->property[$offset] = $value;`, where metadata must
    attach to the actual appended integer key after the method call.
- If the appended value is an array literal containing reference elements, binds
  those nested reference slots onto the stored backing bucket.
- Proves later exact `offsetGet($offset) { return $this->property[$offset]; }`
  bucket copies preserve those nested reference slots while ordinary copied
  fields remain detached.

## Remaining From Original Review Target

- `$holders["box"]->{$name}[] = $array`
- `$registry->holder()->bag[] = $array`
- `make_holder($bag)->bag[] = $array`
- Dynamic property expressions that must be evaluated once and resolve to a
  visible concrete property.

## Docs Updated

- `docs/ARCHITECTURE.md`: extends the stored-bucket `ArrayAccess::offsetSet()`
  append discussion from direct and direct property-held receivers to bounded
  non-direct holder property-held receivers, including the evaluate-once
  holder rule.
- `docs/SUPPORT.md`: moves the covered example out of the unsupported
  non-direct holder append-store bucket and name the exact supported receiver,
  visibility, `offsetSet()`, and `offsetGet()` bridge shapes.
- `docs/PROGRESS.md`: records the focused Rust tests and CLI fixture
  comparison.
- `docs/NEXT_TASKS.md`: marks the lane complete and updates the next receiver
  boundary, keeping magic-property append stores and broader bodies named as
  future work.
- This review note: now records the landed subset and the remaining target
  variants.

## Unsupported Edges To Keep Named

- Magic-property append-store roots such as `$box->missing[] = $array` and
  `$box->{$name}[] = $array`.
- Dynamic non-direct holder property append stores such as
  `$holders["box"]->{$name}[] = $array`.
- Method-return or factory holder roots such as
  `$registry->holder()->bag[] = $array` and `make_holder($bag)->bag[] =
  $array`.
- Non-direct holder properties that are inaccessible in the active visibility
  context, trigger magic fallback, or do not hold an `ArrayAccess` object.
- Holder or property expressions that are evaluated more than once or whose
  side effects are not covered by focused tests.
- Side-effecting, reordered, guarded, nested, or otherwise broader
  `offsetSet()` bodies beyond the exact bridge and covered null-guard append
  bridge.
- Broader `offsetGet()` bodies beyond the exact
  `return $this->property[$offset];` bridge used to prove later bucket copies.
- Appended values produced by arbitrary expressions with untracked reference
  provenance.
- Mixed nested `ArrayAccess` chains and append stores below an `ArrayAccess`
  object returned from another `ArrayAccess` lookup.
- Broad alias lifetime after replacing the holder property or replacing the
  non-direct holder container.
- Full PHP references/COW, native reference lowering, exact alias
  destruction/destructor ordering, and native `compile` support.

## Landing Checks Run

- Focused Rust regression for a non-direct named property-held append store
  such as `$holders["box"]->bag[] = $array` using the exact empty-string-key
  bridge, proving nested reference-slot write-through and ordinary copied-field
  detachment after a later exact `offsetGet()` copy: passed.
- Focused Rust regression for the same non-direct holder receiver family using
  the branchy null-guard append bridge, proving metadata attaches to the actual
  appended integer key: passed.
- `CARGO_TARGET_DIR=/tmp/phpc-target-1685-check CARGO_BUILD_JOBS=1
  CARGO_INCREMENTAL=0 cargo check -q -p phpc`: passed.
- `CARGO_TARGET_DIR=/tmp/phpc-target-1685-focus CARGO_BUILD_JOBS=1
  CARGO_INCREMENTAL=0 cargo test -q -p phpc --test functions_and_scopes
  non_direct_holder_property_held_array_access -- --test-threads=1`: passed
  `4` tests.
- `CARGO_TARGET_DIR=/tmp/phpc-target-1685-compare CARGO_BUILD_JOBS=1
  CARGO_INCREMENTAL=0 cargo run -q -p phpc -- test --compare-php
  tests/fixtures/milestone1685`: passed `2` fixtures with `2` system PHP
  comparisons and `0` skips.

## Landing Checks Still Needed Outside This Subset

- Evaluate-once guard for a method-return or factory holder, such as
  `$registry->holder()->bag[] = $array` or `make_holder($bag)->bag[] =
  $array`.
- Dynamic property guard for `$holders["box"]->{$name}[] = $array` proving the
  property-name expression is evaluated once and routes through a visible
  concrete property.
- Adjacent guard coverage that Lane 1683 and Lane 1684 direct property-held
  append stores still pass, and that unsupported magic-property receivers do
  not silently overclaim support.
- PHP-comparable fixtures under a new milestone directory, exercised with
  `cargo run -q -p phpc -- test --compare-php`.
- `cargo fmt --check`, `git diff --check`, focused `functions_and_scopes`
  filters, and the relevant fixture comparison must pass before support docs
  or progress claim the lane as landed.
