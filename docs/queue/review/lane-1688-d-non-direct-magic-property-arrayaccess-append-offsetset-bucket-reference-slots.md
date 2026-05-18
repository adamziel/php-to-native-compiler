# Lane 1688-D Review: Non-Direct Magic-Property ArrayAccess Append `offsetSet(null)` Bucket Reference Slots

Lane 1688 targets the next receiver step after Lane 1687-C: append stores to
magic properties reached through a non-direct holder expression. The intended
shape is still narrow. It should cover cases where evaluating the holder once
produces an object whose visible public `__get($name)` returns an
`ArrayAccess` object, and append writes dispatch to that object's public
`offsetSet(null, $value)` method.

## Landing Summary

Lane 1688-C landed the covered named and dynamic non-direct magic-property
append-store shapes:

- `$holders["box"]->missing[] = $array`
- `$holders["box"]->{$name}[] = $array`

The runtime now evaluates the holder once, evaluates the dynamic property name
once for the dynamic spelling, calls visible public `__get($name)` once for
the append store, avoids `__set()` for the covered object case, and dispatches
`offsetSet(null, $value)` on the returned `ArrayAccess` object. The focused
fixtures cover both exact empty-string-key and branchy append-key
stored-bucket bridges, proving nested reference-slot write-through after a
later exact by-value `offsetGet()` bucket copy while ordinary copied fields
remain detached. Native lowering remains a rejection boundary for this
receiver shape.

## Target Slice

- Accept append assignment to a non-direct magic-property root, for example
  `$holders["box"]->missing[] = $array`.
- Include the dynamic-name spelling
  `$holders["box"]->{$name}[] = $array` when the property-name expression is
  evaluated once and resolves to the same magic-property route.
- Evaluate the non-direct holder expression exactly once, store it through the
  existing temporary-holder machinery used by prior non-direct receiver lanes,
  and invoke magic lookup on that materialized object.
- Call visible public `__get($name)` exactly once for the append operation,
  then dispatch `offsetSet(null, $value)` on the returned `ArrayAccess`
  object. The covered object case must not call `__set()`.
- Preserve nested reference slots when the appended value is an array literal
  containing reference elements or a copied array carrying mirrored alias
  metadata.
- Cover both existing stored-bucket bridges:
  - exact bridge: `offsetSet($offset, $value) {
    $this->property[$offset] = $value; }`, where PHP's null offset stores
    under the backing array's empty-string key;
  - branchy append bridge: `if ($offset === null) { $this->property[] =
    $value; return; } $this->property[$offset] = $value;`, where reference
    metadata must attach to the actual appended integer key after the user
    method returns.
- Prove that a later exact `offsetGet($offset) { return
  $this->property[$offset]; }` bucket copy from the stored backing bucket
  preserves those nested reference slots while ordinary copied fields remain
  detached.
- Keep native lowering as a rejection boundary. `phpc compile --emit-ir` and
  `--emit-asm` must not lower this receiver shape.

## Difference From Lane 1687 Direct Magic Stores

- Lane 1687 starts from a direct object variable such as `$box->missing[] =
  $array` or `$box->{$name}[] = $array`.
- Lane 1688 starts from a non-direct holder expression such as
  `$holders["box"]->missing[] = $array`, so the implementation must preserve
  PHP's evaluate-once behavior for the holder before invoking `__get()`.
- The magic-property dispatch should happen on the evaluated holder object,
  not on the root array or expression that produced it.
- Prior non-direct visible property-held lanes route through concrete visible
  properties. This lane should route through magic fallback only when the
  selected property is missing or inaccessible in the same bounded way as the
  existing magic-property paths.
- Lane 1687 already separated append stores from Lane 1680 reference-source
  reads. Lane 1688 must keep the same separation for non-direct holders:
  `$holders["box"]->missing[] = $array` uses `offsetSet(null, $value)`, while
  `$alias =& $holders["box"]->missing[]` remains an `offsetGet(null)`
  reference-source notice/no-op or alias path according to the already
  documented source semantics.

## Required Tests

- Focused Rust regression for
  `$holders["box"]->missing[] = $array` using the exact empty-string-key
  `offsetSet()` bridge, proving nested reference-slot write-through after a
  later exact `offsetGet()` bucket copy.
- Focused Rust regression for the same non-direct magic receiver using the
  branchy null-guard append bridge, proving metadata attaches to the actual
  appended integer key.
- Dynamic-name regressions for `$holders["box"]->{$name}[] = $array`,
  covering both exact and branchy append bridges.
- Evaluate-once guard for the holder expression, ideally with a counted
  function or method that returns the holder object and would expose duplicate
  evaluation.
- Evaluate-once guard for the dynamic property-name expression.
- Guard proving `__get($name)` is called exactly once for the append store.
- Guard proving `__set()` is not called for the covered `ArrayAccess` object
  append case.
- Regression preserving direct Lane 1687 behavior for `$box->missing[] =
  $array` and `$box->{$name}[] = $array`.
- Regression preserving Lane 1680/Lane 1679 reference-source behavior:
  `$alias =& $holders["box"]->missing[]` must not be rerouted through the
  append-store `offsetSet(null, $value)` path.
- Fallback guards for non-`ArrayAccess` magic returns, especially by-value
  plain arrays staying in the indirect-modification notice/no-op bucket and
  by-reference plain arrays preserving the existing magic-array append
  behavior.
- CLI fixture coverage under a new `tests/fixtures/milestone1688` directory
  with system PHP comparison for the exact bridge, branchy bridge, dynamic
  property name, and at least one reference-source regression.
- Native rejection checks proving `phpc compile --emit-ir` rejects the
  non-direct object-property/ArrayAccess shape instead of emitting misleading
  native code.

## Unsupported Edges To Keep Named

- Direct magic-property append stores are Lane 1687, not the new work except
  as regressions.
- Method-return or factory holder roots such as
  `$registry->holder()->missing[] = $array` and
  `make_holder()->missing[] = $array`, unless the implementation explicitly
  includes and tests them as the evaluate-once non-direct holder source.
- Magic `__get()` bodies that return properties, offsets, method calls,
  constructors, or arbitrary expressions outside the bounded body shape chosen
  for this lane.
- Magic `__set()`, `__isset()`, or `__unset()` interaction beyond proving
  `__set()` is not called for the covered `ArrayAccess` object append store.
- Dynamic property names that resolve to visible concrete properties already
  covered by earlier property-held lanes, inaccessible properties with
  unmodeled visibility behavior, or names whose magic fallback result is not
  the covered `ArrayAccess` object.
- Non-empty nested append paths such as
  `$holders["box"]->missing["outer"][] = $array` and
  `$holders["box"]->{$name}["outer"][] = $array`.
- Side-effecting, reordered, guarded, nested, private/protected-only, or
  otherwise broader `offsetSet()` bodies beyond the exact bridge and covered
  null-guard append bridge.
- Broader `offsetGet()` bodies beyond the exact
  `return $this->property[$offset];` bridge used to prove later bucket copies.
- Appended values produced by arbitrary expressions with untracked reference
  provenance.
- Mixed nested `ArrayAccess` chains and append stores below an `ArrayAccess`
  object returned from another `ArrayAccess` lookup.
- Broad alias lifetime after replacing the holder object, replacing the magic
  backing object, replacing the storage property inside the `ArrayAccess`
  object, or reassigning the source variables that supplied nested reference
  slots.
- Full PHP references/COW, native reference lowering, exact alias
  destruction/destructor ordering, and native `compile` support.

## Docs To Update After Landing

- `docs/ARCHITECTURE.md`: describe the non-direct magic-property append-store
  route, including holder evaluate-once behavior, `__get()` evaluate-once
  behavior, and the separation from reference-source `offsetGet(null)`
  semantics.
- `docs/SUPPORT.md`: move only the proven non-direct magic-property
  `ArrayAccess` append-store shapes out of the unsupported bucket, with exact
  receiver, `__get()`, `offsetSet()`, and later `offsetGet()` bridge limits.
- `docs/PROGRESS.md`: record the implementation summary, focused Rust tests,
  fixture comparisons, native rejection checks, and remaining unsupported
  edges.
- `docs/NEXT_TASKS.md`: mark Lane 1688 complete only after implementation,
  tests, fixtures, and docs land; keep broader magic, nested append, and COW
  gaps visible.
- This review note: convert from target guidance to landing summary and keep
  any unimplemented variants named as unsupported.

## Landing Gate

- `cargo fmt --check`
- `git diff --check`
- Focused `functions_and_scopes` filters for exact, branchy, dynamic-name,
  holder evaluate-once, property-name evaluate-once, `__get()` once,
  `__set()` guard, and fallback/regression cases.
- `cargo run -q -p phpc -- test --compare-php tests/fixtures/milestone1688`
- Adjacent fixture comparisons for `tests/fixtures/milestone1679`,
  `tests/fixtures/milestone1680`, `tests/fixtures/milestone1686`, and
  `tests/fixtures/milestone1687`.
- Broader verification as warranted if the implementation touches shared
  parser assignment-target, temporary-holder, magic-property, ArrayAccess
  dispatch, or reference-slot propagation helpers.
