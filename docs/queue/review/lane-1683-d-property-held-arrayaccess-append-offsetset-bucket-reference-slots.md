# Lane 1683-D Review: Property-Held ArrayAccess Append `offsetSet(null)` Bucket Reference Slots

This lane has landed as Lane 1683-C. Runtime implementation, focused Rust
coverage, PHP-comparable fixtures, and the CLI fixture exercise passed for the
direct visible named property-held receiver shape.

## Current Baseline

- Lane 1682-C landed direct `$bag[] = $array` stored-bucket reference-slot
  propagation for two public `offsetSet(null, $value)` shapes on a direct
  `ArrayAccess` object.
- `docs/SUPPORT.md`, `docs/ARCHITECTURE.md`, `docs/PROGRESS.md`, and
  `docs/NEXT_TASKS.md` now describe the landed direct visible named
  property-held append stored-bucket COW behavior.
- Older object semantics already dispatch property-held append writes such as
  `$holder->bag[] = $value` to `offsetSet(null, $value)`. That dispatch support
  is not the same as proving stored-bucket nested reference-slot preservation.

## Target Slice

- Preserve nested reference slots when a visible property-held `ArrayAccess`
  object receives an append write such as `$holder->bag[] = $array`.
- Keep the receiver shape direct and visible: named public properties such as
  `$holder->bag[] = $array`. Dynamic visible properties such as
  `$holder->{$name}[] = $array` remain outside this landed slice.
- Cover the same two public `offsetSet(null, $value)` storage bridges as Lane
  1682-C:
  - exact bridge: `$this->property[$offset] = $value;`, where PHP's null offset
    stores under the backing array's empty-string key;
  - branchy append bridge: `if ($offset === null) { $this->property[] =
    $value; return; } $this->property[$offset] = $value;`, where reference
    metadata must attach to the actual appended integer key after the method
    call.
- If the appended value is an array literal containing reference elements, bind
  those nested reference slots onto the stored backing bucket.
- If the appended value is a copied array carrying mirrored alias metadata,
  mirror those nested aliases onto the stored backing bucket.
- Prove later exact `offsetGet($offset) { return $this->property[$offset]; }`
  bucket copies preserve those nested reference slots while ordinary copied
  fields remain detached.

## Landing Summary

- The runtime detects direct visible named property-held `ArrayAccess` append
  assignment before the plain object-property append-reference path.
- It dispatches `offsetSet(null, $value)` once, then binds or mirrors nested
  reference-slot metadata to the held object's backing bucket for the exact
  empty-key bridge or branchy append-key bridge.
- `cargo run -q -p phpc -- test --compare-php tests/fixtures/milestone1683`
  passed `2` fixtures with `2` system PHP comparisons and `0` skips.

## Unsupported Edges To Keep Named

- Non-direct receiver append stores such as
  `$holders["box"]->bag[] = $array`, `$registry->holder()->bag[] = $array`,
  and `make_holder($bag)->bag[] = $array`.
- Magic-property receiver roots such as `$box->missing[] = $array` unless a
  later lane explicitly implements and tests them.
- Dynamic property-held receivers such as `$holder->{$name}[] = $array`.
- Side-effecting, reordered, guarded, nested, or otherwise broader
  `offsetSet()` bodies beyond the exact bridge and the covered null-guard
  append bridge.
- Broader `offsetGet()` bodies beyond the exact
  `return $this->property[$offset];` bridge used to prove later bucket copies.
- Appended values produced by arbitrary expressions with untracked reference
  provenance.
- Mixed nested `ArrayAccess` chains and append stores below an `ArrayAccess`
  object returned from another `ArrayAccess` lookup.
- Full PHP references/COW, native reference lowering, and exact alias
  destruction/destructor ordering.

## Landing Checks Run

- Focused Rust regression for `$holder->bag[] = $array` using the exact
  empty-string-key bridge, proving nested reference-slot write-through and
  ordinary copied-field detachment after a later exact `offsetGet()` copy.
- Focused Rust regression for `$holder->bag[] = $array` using the branchy
  null-guard append bridge, proving metadata attaches to the actual appended
  integer key.
- PHP-comparable fixtures under a new milestone directory, exercised with
  `cargo run -q -p phpc -- test --compare-php`.
- Adjacent guard coverage showing Lane 1682 direct append stores still pass and
  unsupported non-direct or magic receiver shapes do not silently overclaim.
- `cargo fmt --check`, `git diff --check`, focused `functions_and_scopes`
  filters, and the relevant fixture comparison must pass before support docs
  or progress claim the lane as landed.
