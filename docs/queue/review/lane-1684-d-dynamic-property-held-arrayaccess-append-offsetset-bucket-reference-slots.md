# Lane 1684-D Review: Dynamic Property-Held ArrayAccess Append `offsetSet(null)` Bucket Reference Slots

This lane has landed as Lane 1684-C. Implementation code, focused Rust
coverage, PHP-comparable fixtures, and the CLI fixture exercise passed for the
direct visible dynamic property-held receiver shape.

## Current Baseline

- Lane 1682-C landed direct `$bag[] = $array` stored-bucket reference-slot
  propagation for public `offsetSet(null, $value)` on a direct `ArrayAccess`
  object.
- Lane 1683-C landed direct visible named property-held append stores such as
  `$holder->bag[] = $array` for the same exact empty-string-key bridge and
  branchy append-key bridge.
- `docs/ARCHITECTURE.md`, `docs/SUPPORT.md`, `docs/PROGRESS.md`, and
  `docs/NEXT_TASKS.md` now describe the landed direct visible dynamic
  property-held append stored-bucket COW behavior.
- Existing dynamic property-held `ArrayAccess` paths cover other behaviors,
  including selected `offsetGet()` reference sources and copied-bucket reads,
  but that is not proof that append stores attach nested reference-slot
  metadata to the stored backing bucket.

## Target Slice

- Preserve nested reference slots when a direct holder's evaluated visible
  dynamic property contains an `ArrayAccess` object and receives an append
  write, for example `$holder->{$name}[] = $array`.
- Keep the receiver direct and property-visible in the current context. The
  dynamic property name expression must be evaluated once, and the selected
  property must route through the same public/context visibility rules used by
  existing dynamic property-held `ArrayAccess` paths.
- Cover the same two public `offsetSet(null, $value)` storage bridges as
  Lanes 1682-C and 1683-C:
  - exact bridge: `$this->property[$offset] = $value;`, where PHP's null
    offset stores under the backing array's empty-string key;
  - branchy append bridge: `if ($offset === null) { $this->property[] =
    $value; return; } $this->property[$offset] = $value;`, where metadata must
    attach to the actual appended integer key after the method call.
- If the appended value is an array literal containing reference elements,
  bind those nested reference slots onto the stored backing bucket.
- If the appended value is a copied array carrying mirrored alias metadata,
  mirror those nested aliases onto the stored backing bucket.
- Prove that later exact `offsetGet($offset) { return
  $this->property[$offset]; }` bucket copies preserve those nested reference
  slots while ordinary copied fields remain detached.

## Landing Summary

- The parser now accepts direct dynamic property append assignment targets such
  as `$holder->{$name}[] = $array`.
- The interpreter evaluates the dynamic property name once, reuses the
  property-held `ArrayAccess` hidden-root path, and binds or mirrors nested
  reference-slot metadata to the held object's backing bucket for the exact
  empty-key and branchy append-key bridges.
- Native lowering still rejects this target at the existing array-access
  lowering boundary.
- `cargo run -q -p phpc -- test --compare-php tests/fixtures/milestone1684`
  passed `2` fixtures with `2` system PHP comparisons and `0` skips.

## Unsupported Edges To Keep Named

- Non-direct receiver append stores such as
  `$holders["box"]->{$name}[] = $array`,
  `$registry->holder()->{$name}[] = $array`, and
  `make_holder($bag)->{$name}[] = $array`.
- Magic-property receiver roots such as `$box->missing[] = $array` and
  `$box->{$name}[] = $array` unless a later lane explicitly implements and
  tests stored-bucket propagation through magic roots.
- Dynamic property names that resolve to inaccessible properties, trigger
  magic fallback, or otherwise do not reach a visible concrete property holding
  an `ArrayAccess` object.
- Side-effecting, reordered, guarded, nested, or otherwise broader
  `offsetSet()` bodies beyond the exact bridge and the covered null-guard
  append bridge.
- Broader `offsetGet()` bodies beyond the exact
  `return $this->property[$offset];` bridge used to prove later bucket copies.
- Appended values produced by arbitrary expressions with untracked reference
  provenance.
- Mixed nested `ArrayAccess` chains and append stores below an `ArrayAccess`
  object returned from another `ArrayAccess` lookup.
- Broad alias lifetime after replacing the dynamic holder property.
- Full PHP references/COW, native reference lowering, exact alias
  destruction/destructor ordering, and native `compile` support.

## Landing Checks Run

- Focused Rust regression for `$holder->{$name}[] = $array` using the exact
  empty-string-key bridge, proving nested reference-slot write-through and
  ordinary copied-field detachment after a later exact `offsetGet()` copy.
- Focused Rust regression for `$holder->{$name}[] = $array` using the branchy
  null-guard append bridge, proving metadata attaches to the actual appended
  integer key.
- Guard coverage that Lane 1683 direct named property-held append stores still
  pass and unsupported non-direct or magic receiver shapes do not silently
  overclaim.
- PHP-comparable fixtures under a new milestone directory, exercised with
  `cargo run -q -p phpc -- test --compare-php`.
- `cargo fmt --check`, `git diff --check`, focused `functions_and_scopes`
  filters, and the relevant fixture comparison must pass before support docs
  or progress claim the lane as landed.
