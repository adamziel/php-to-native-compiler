# Lane 1682-D Review: ArrayAccess Append `offsetSet(null)` Bucket Reference Slots

This lane has landed as Lane 1682-C. The runtime implementation, focused Rust
tests, CLI fixtures, and system PHP comparisons passed for both exact
empty-key and branchy append-key bridges.

## Target Slice

- Direct append writes on a direct `ArrayAccess` object, such as
  `$bag[] = $array`, should preserve nested reference slots when public
  `offsetSet($offset, $value)` has the exact body
  `$this->property[$offset] = $value;`.
- The append key for this exact bridge is PHP's `null` argument to
  `offsetSet(null, $value)`. Because the bridge body writes
  `$this->property[$offset]`, the stored bucket is the backing array's `""`
  string key, not the next integer append key.
- If the appended value is an array literal containing reference elements, the
  stored backing bucket should bind those reference slots.
- If the appended value is a copied array carrying mirrored alias metadata, the
  stored backing bucket should mirror those nested aliases.
- Later exact `offsetGet($offset) { return $this->property[$offset]; }` bucket
  copies from `$bag[null]` should preserve those nested reference slots while
  ordinary copied fields remain detached.
- The landed lane also covers the branchy append bridge
  `if ($offset === null) { $this->property[] = $value; return; }
  $this->property[$offset] = $value;` and attaches metadata to the actual
  appended integer key after the method call.

## Docs Updated

- `docs/ARCHITECTURE.md` now distinguishes append
  `offsetSet(null, $value)` stored-bucket reference-slot propagation from the
  already supported by-value append `offsetGet(null)` reference-source
  notice/no-op lane.
- `docs/SUPPORT.md` now states the two supported direct append stored-bucket
  bridges and keeps property-held/non-direct append stores unsupported.
- `docs/queue/review/lane-1681-d-arrayaccess-offsetset-bucket-reference-slots.md`
  correctly lists append `offsetSet(null)` stored-bucket reference slots as
  unsupported for Lane 1681's historical scope. Leave it as historical review
  context.

## Landing Checks

- Focused Rust tests cover the exact empty-key bridge and branchy append-key
  bridge.
- `cargo run -q -p phpc -- test --compare-php tests/fixtures/milestone1682`
  passed `2` fixtures with `2` system PHP comparisons and `0` skips.

## Unsupported Edge Cases To Name

- Property-held append stores such as `$holder->bag[] = $array`.
- Non-direct receiver append stores such as `$holders["box"]->bag[] = $array`,
  `$registry->holder()->bag[] = $array`, and `make_holder($bag)[] = $array`.
- Dynamic or magic-property receiver roots, including `$holder->{$name}[] =
  $array` and `$box->missing[] = $array`.
- Side-effecting, reordered, nested, or otherwise broader `offsetSet()` bodies
  beyond the exact bridge and the covered null-guard append bridge.
- Broader `offsetGet()` bodies beyond the exact
  `return $this->property[$offset];` bridge used to prove later bucket copies.
- Appended values produced by arbitrary expressions with untracked reference
  provenance.
- Mixed nested `ArrayAccess` chains and append stores below an `ArrayAccess`
  object returned from another `ArrayAccess` lookup.
- Full PHP references/COW, native reference lowering, and exact alias
  destruction/destructor ordering.
