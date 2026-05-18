# Lane 1687-D Review: Magic-Property ArrayAccess Append `offsetSet(null)` Bucket Reference Slots

Lane 1687-C landed the direct magic-property `ArrayAccess` append-store slice.
The covered shape is not general overloaded-property mutation; it is the
narrow path where visible public `__get($name)` returns an `ArrayAccess`
object and append writes such as `$box->missing[] = $array` and
`$box->{$name}[] = $array` dispatch to that object's public
`offsetSet(null, $value)` method.

## Current Baseline

- Lane 1682-C landed direct `$bag[] = $array` append stored-bucket
  reference-slot propagation for the exact empty-string-key and branchy
  append-key `offsetSet(null, $value)` bridges.
- Lane 1683-C through Lane 1686-C walked that support through visible
  property-held, dynamic property-held, non-direct named holder, and
  non-direct dynamic holder receivers.
- Lane 1680-C already covers magic-property `ArrayAccess` reference-source
  reads, including by-value `offsetGet(null)` notice/no-op behavior for
  statement-form reference assignment sources.
- Direct magic-property append stores such as `$box->missing[] = $array` and
  `$box->{$name}[] = $array` are now covered for the focused `ArrayAccess`
  object return shape. Non-direct magic-property append stores remain
  unsupported.

## Landed Slice

- Accepts append assignment to a direct magic-property root where a visible
  public `__get($name)` returns an `ArrayAccess` object, for example
  `$box->missing[] = $array`.
- Includes the dynamic-name spelling `$box->{$name}[] = $array` when the
  property name expression is evaluated once and resolves to the same
  magic-property path.
- Calls `__get($name)` exactly once for the append operation, then dispatches
  `offsetSet(null, $value)` on the returned object. The path must not call
  `__set()` for the `ArrayAccess` object case.
- Covers both existing stored-bucket bridges:
  - exact bridge: `offsetSet($offset, $value) {
    $this->property[$offset] = $value; }`, where the backing bucket is the
    empty-string key because PHP passes `null` to `offsetSet()`;
  - branchy append bridge: `if ($offset === null) { $this->property[] =
    $value; return; } $this->property[$offset] = $value;`, where reference
    metadata must attach to the actual appended integer key after the user
    method returns.
- Preserves nested reference slots when the appended value is an array literal
  containing reference elements or a copied array carrying mirrored alias
  metadata.
- Proves that a later exact `offsetGet($offset) { return
  $this->property[$offset]; }` bucket copy from the stored backing bucket
  preserves those nested reference slots while ordinary copied fields remain
  detached.
- Native lowering continues to reject this shape through the existing
  object-property or ArrayAccess lowering boundary.

## PHP Semantics Questions

- Confirm the lane's exact `__get()` return shape. PHP 8.2 calls
  `offsetSet(null, ...)` when `__get()` returns an `ArrayAccess` object by
  value or by reference, because the object handle is still mutable. The
  compiler should decide whether Lane 1687 supports both declarations or only
  the already-modeled bounded direct-variable-returning body shape.
- Confirm dynamic property-name ordering for `$box->{expr()}[] = $array`:
  PHP evaluates the name expression once before `__get()`, then invokes
  `offsetSet(null, ...)` on the returned object.
- Confirm fallback ordering when `__get()` returns a plain array, scalar, or
  non-`ArrayAccess` object. Plain array by-value append should stay in the
  indirect-modification notice/no-op bucket, while plain array by-reference
  append should keep the existing magic-array behavior and must not be
  accidentally rerouted through this lane.
- Confirm inaccessible declared properties that trigger `__get()` remain
  distinct from visible concrete properties. Visible property-held
  `ArrayAccess` append stores are already covered by earlier lanes; this lane
  should only add the overloaded-property route.
- Confirm whether a `__get()` method returning a newly constructed
  `ArrayAccess` object is in scope. PHP will call `offsetSet()` on that
  temporary object, but later reads through the object may be unobservable
  unless the method stores or reuses the object.

## Landing Checks Run

- Focused Rust regression for `$box->missing[] = $array` where public
  `__get($name)` returns the `ArrayAccess` object and the exact
  `$this->property[$offset] = $value;` bridge stores under the empty-string
  backing key: passed.
- Focused Rust regression for the same receiver using the branchy null-guard
  append bridge, proving metadata attaches to the actual appended integer
  key: passed.
- Dynamic-name regressions for `$box->{$name}[] = $array`, covering exact and
  branchy append bridges: passed.
- Regression preserving the Lane 1680 magic `ArrayAccess` reference-source
  behavior: `$alias =& $box->missing[]` should still use `offsetGet(null)`
  notice/no-op semantics, not the append-store `offsetSet(null)` path:
  passed.
- Native boundary filters proving `phpc compile --emit-ir` rejects object and
  reference forms instead of emitting misleading native code: passed.
- `CARGO_TARGET_DIR=/tmp/phpc-target-1687-check CARGO_BUILD_JOBS=1
  CARGO_INCREMENTAL=0 cargo check -q -p phpc`: passed.
- `CARGO_TARGET_DIR=/tmp/phpc-target-1687-focus CARGO_BUILD_JOBS=1
  CARGO_INCREMENTAL=0 cargo test -q -p phpc --test functions_and_scopes
  magic_property_array_access -- --test-threads=1`: passed `8` tests.
- `CARGO_TARGET_DIR=/tmp/phpc-target-1687-compare CARGO_BUILD_JOBS=1
  CARGO_INCREMENTAL=0 cargo run -q -p phpc -- test --compare-php
  tests/fixtures/milestone1687`: passed `4` fixtures with `4` system PHP
  comparisons and `0` skips.

## Still Needed Outside This Subset

- Explicit `__get()` evaluate-once counter guard.
- Guard proving `__set()` is not called for the `ArrayAccess` object append
  case.
- Plain-array fallback guards: by-value `__get()` returning an array should
  still emit the indirect-modification notice/no-op behavior, and by-reference
  `__get()` returning an array should still append to the returned array cell.

## Unsupported Edges To Keep Named

- Non-direct magic-property append stores such as
  `$holders["box"]->missing[] = $array` and
  `$holders["box"]->{$name}[] = $array`, unless the implementation explicitly
  includes and tests them.
- Magic `__get()` bodies that return properties, offsets, method calls,
  constructors, or arbitrary expressions outside the bounded body shape chosen
  for this lane.
- Magic `__set()`, `__isset()`, or `__unset()` interaction beyond proving
  `__set()` is not called for the covered `ArrayAccess` object append store.
- Dynamic property names that resolve to visible declared properties already
  covered by earlier lanes, inaccessible declared properties with different
  visibility behavior, or names whose fallback result is not the covered
  `ArrayAccess` object.
- Non-empty nested append paths such as `$box->missing["outer"][] = $array`
  and `$box->{$name}["outer"][] = $array`.
- Side-effecting, reordered, guarded, nested, private/protected-only, or
  otherwise broader `offsetSet()` bodies beyond the exact bridge and covered
  null-guard append bridge.
- Broader `offsetGet()` bodies beyond the exact
  `return $this->property[$offset];` bridge used to prove later bucket copies.
- Appended values produced by arbitrary expressions with untracked reference
  provenance.
- Mixed nested `ArrayAccess` chains and append stores below an `ArrayAccess`
  object returned from another `ArrayAccess` lookup.
- Broad alias lifetime after replacing the magic backing object, replacing the
  storage property inside the `ArrayAccess` object, or reassigning the source
  variables that supplied nested reference slots.
- Full PHP references/COW, native reference lowering, exact alias
  destruction/destructor ordering, and native `compile` support.

## Docs To Update After Landing

- `docs/ARCHITECTURE.md`: describe the magic-property append-store route,
  including `__get()` evaluate-once behavior, the separation from Lane 1680
  `offsetGet(null)` reference-source behavior, and the two supported
  `offsetSet(null, $value)` backing-bucket bridges.
- `docs/SUPPORT.md`: move only the proven direct magic-property
  `ArrayAccess` append-store shapes out of the unsupported bucket, with the
  exact receiver, `__get()`, `offsetSet()`, and `offsetGet()` limits.
- `docs/PROGRESS.md`: record the implementation summary, focused Rust tests,
  fixture comparisons, native rejection checks, and remaining unsupported
  edges.
- `docs/NEXT_TASKS.md`: mark Lane 1687 complete only after tests and fixtures
  pass, then list the next receiver or COW gap without dropping non-direct
  magic-property and broader-body boundaries.
- This review note: convert from target guidance to landing summary and keep
  any unimplemented variants named as unsupported.

## Landing Gate

- `cargo fmt --check`
- `git diff --check`
- Focused `functions_and_scopes` filters for exact, branchy, dynamic-name,
  evaluate-once, `__set()` guard, and fallback/regression cases.
- `cargo run -q -p phpc -- test --compare-php tests/fixtures/milestone1687`
- Adjacent fixture comparisons for `tests/fixtures/milestone1680`,
  `tests/fixtures/milestone1682`, and `tests/fixtures/milestone1686`.
- Broader verification as warranted if the implementation touches shared
  parser assignment-target, magic-property, ArrayAccess dispatch, or
  reference-slot propagation helpers.
