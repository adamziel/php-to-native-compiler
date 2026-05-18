# Lane 1692-D Review: Magic-Property ArrayAccess Nested Append Stores

## Landing Summary

Lane 1692-C landed the focused one-key magic-property `ArrayAccess` nested
append shape below. The runtime now routes `$box->missing["outer"][] =
$array` and the direct/dynamic/non-direct variants through exact public
by-reference `offsetGet($offset) { return $this->items[$offset]; }`, appends
into that backing bucket, and preserves nested reference-slot metadata for
later copied-bucket reads. The by-value `offsetGet()` shape remains a
notice/no-op boundary, and empty magic `ArrayAccess` append stores remain on
the existing `offsetSet(null, $value)` path.

Validation before checkpoint: PHP lint for `tests/fixtures/milestone1692`,
`cargo check -q -p phpc`, `cargo fmt --check`, `git diff --check`, the
`milestone1692` `functions_and_scopes` filter with `2` tests,
`cargo run -q -p phpc -- test --compare-php tests/fixtures/milestone1692`
with `5` fixtures and `5` system PHP comparisons, the adjacent magic
`ArrayAccess` append filter with `8` tests, and adjacent `milestone1687`,
`milestone1688`, and `milestone1691` fixture comparisons.

Lane 1692 should cover the focused `ArrayAccess` nested append gap left after
the empty magic-property append-store lanes and the plain-array nested magic
lanes. The target shape is a one-key append below an `ArrayAccess` object
returned by visible public `__get($name)`, where the selected bucket is reached
through public by-reference `offsetGet($offset)`.

This is not broad overloaded-property mutation. It is the narrow path where
`$box->missing["outer"][] = $array` first obtains the `ArrayAccess` object
through magic property lookup, then appends into the array bucket returned by
`offsetGet("outer")` by reference.

## Target Slice

- Direct named nested append store:
  `$box->missing["outer"][] = $array`.
- Direct dynamic nested append store:
  `$box->{$name}["outer"][] = $array`, with the dynamic property-name
  expression evaluated once before magic lookup.
- Non-direct holder named nested append store:
  `$holders["box"]->missing["outer"][] = $array`, with the holder expression
  evaluated once before magic lookup.
- Non-direct holder dynamic nested append store:
  `$holders["box"]->{$name}["outer"][] = $array`, with both the holder and
  dynamic property-name expressions evaluated once.
- The magic root must be a visible public `__get($name)` result that is an
  `ArrayAccess` object. The nested parent bucket is selected with public
  by-reference `offsetGet($offset)`.
- The covered `offsetGet()` bridge is the existing exact bounded shape:
  `return $this->property[$offset];`. The selected backing bucket must be an
  array or `null` before materialization.
- The final `[]` append should materialize a missing or `null` selected
  backing bucket as array storage, append under the actual next integer key,
  and attach reference metadata to that final slot.
- Preserve nested reference slots when the appended value is an array literal
  containing reference elements or a copied array carrying mirrored alias
  metadata.
- Prove later reads through the same `ArrayAccess` bucket observe writes
  through referenced nested elements, while ordinary copied fields remain
  detached.

## PHP Semantics To Prove

- `__get($name)` should be called exactly once for each covered append store.
- For dynamic property names, the property-name expression should be evaluated
  once before `__get()` dispatch.
- For non-direct holders, the holder expression should be evaluated once, and
  magic lookup should run on that evaluated holder object.
- By-reference `offsetGet("outer")` should be called for the selected parent
  bucket, and the append should mutate the backing bucket returned by
  reference.
- A missing selected bucket, or a selected bucket currently holding `null`,
  should be materialized as an array before the final append when that matches
  the system PHP fixture behavior for the covered shape.
- Existing array buckets should keep sibling keys and append at the next
  integer key under the selected backing bucket.
- By-value `offsetGet($offset)` for the same one-key nested append should stay
  in PHP's indirect-modification notice/no-op behavior. It must not mutate the
  backing `ArrayAccess` storage and must not be reported as supported
  mutation.
- By-value or non-reference `__get()` returning an `ArrayAccess` object still
  returns a mutable object handle for the root, but mutation below the selected
  bucket depends on by-reference `offsetGet()`. By-value `offsetGet()` remains
  the no-op boundary for this lane.
- Append-store semantics must remain separate from reference-source
  semantics. `$box->missing["outer"][] = $array` is a write through
  by-reference `offsetGet()`, while
  `$alias =& $box->missing["outer"][]` should continue through the existing
  append reference-source path or no-op behavior.
- Magic `ArrayAccess` nested append stores must remain separate from magic
  plain-array nested append stores. If `__get()` returns a plain array by
  reference, the Lane 1690 and Lane 1691 plain-array paths apply instead.

## Tests Needed

- Direct named by-reference nested append with an array literal containing a
  reference element; mutating the original referenced variable after append
  should be visible through the backing `ArrayAccess` bucket.
- Direct dynamic by-reference nested append with the same reference-slot proof
  and a property-name evaluate-once counter.
- Non-direct holder named by-reference nested append with a holder
  evaluate-once counter and the same reference-slot proof.
- Non-direct holder dynamic by-reference nested append with both holder and
  property-name evaluate-once counters.
- Existing selected bucket append: prepopulate the backing property's
  `"outer"` bucket as an array with sibling keys, append one value, and prove
  siblings remain intact.
- Missing or `null` selected bucket materialization if the implementation
  intentionally covers those PHP-compatible states.
- Copied-array source case: append a copied array that already carries
  mirrored alias metadata, then prove later mutation through a covered nested
  reference slot writes through.
- Ordinary copied-field detachment: mutate a non-reference field after append
  and prove the stored value remains detached.
- By-value `offsetGet()` contrast proving the nested append emits the bounded
  indirect-modification diagnostic path, leaves backing storage unchanged, and
  does not accidentally create an alias.
- Guard proving `__get($name)` is called once and `__set()` is not called for
  the covered `ArrayAccess` nested append case.
- Regression preserving empty magic-property `ArrayAccess` append stores:
  `$box->missing[] = $array` and `$holders["box"]->missing[] = $array` should
  still use `offsetSet(null, $value)`.
- Regression preserving magic plain-array nested append stores:
  `$box->missing["outer"][] = $array` should use the plain-array path only
  when `__get()` returns a direct variable by reference containing an array or
  `null`, not when it returns an `ArrayAccess` object.
- Reference-source regressions proving
  `$alias =& $box->missing["outer"][]` and
  `$alias =& $holders["box"]->missing["outer"][]` stay off the append-store
  mutation path.
- CLI fixtures under a new `tests/fixtures/milestone1692` directory with
  system PHP comparison for direct named, direct dynamic, non-direct named,
  non-direct dynamic, by-value `offsetGet()` no-op contrast, and
  reference-source contrast cases.
- Native rejection checks for `phpc compile --emit-ir` and `--emit-asm`;
  native lowering should continue rejecting magic-property, `ArrayAccess`,
  reference, and COW shapes instead of emitting misleading code.

## Likely Implementation Hooks

- Parser support should already carry the one-key parent prefix on object
  property array append targets used by the prior magic plain-array lanes.
  Confirm that direct named and dynamic forms reach the existing append target
  variants with the prefix intact, and that non-direct named and dynamic forms
  preserve the prefix through the temporary-holder path.
- Keep the magic `ArrayAccess` nested append branch distinct from both the
  empty `offsetSet(null, $value)` append-store path and the by-reference
  plain-array append path.
- Reuse the existing visible public `__get($name)` lookup route and temporary
  holder machinery from the direct and non-direct magic-property lanes.
- Reuse the exact public by-reference `offsetGet($offset) { return
  $this->property[$offset]; }` bridge already used for selected
  `ArrayAccess` reference-source and by-reference argument paths.
- After binding the selected backing bucket as the append destination, pass
  the actual appended key into the array reference-slot propagation hooks so
  literal references and copied alias metadata land under the stored bucket.
- Ensure by-value `offsetGet()` is detected before mutation and routed to the
  indirect-modification notice/no-op behavior.
- Native lowering rejection strings in `compiler/src/codegen.rs` should remain
  broad enough to reject the new interpreter-only shape.

## Unsupported Boundaries

- Empty magic-property `ArrayAccess` append stores are Lane 1687 and
  Lane 1688. They remain `offsetSet(null, $value)` stores, not nested
  `offsetGet()` stores.
- Plain-array magic nested append stores are Lane 1690 and Lane 1691. This
  lane should not change their by-reference `__get()` direct-variable return
  behavior.
- Deeper `ArrayAccess` nested append stores such as
  `$box->missing["outer"]["inner"][] = $array` unless the implementation
  explicitly includes and tests them.
- By-value `offsetGet()` mutation. This lane should test the notice/no-op
  behavior, not turn it into a write-through path.
- Magic `__get()` bodies that return properties, offsets, method calls,
  object fields, newly constructed objects, or arbitrary expressions outside
  the bounded return shape chosen for the root object.
- `offsetGet()` bodies beyond the exact public
  `return $this->property[$offset];` bridge, including side-effecting,
  guarded, private/protected-only, nested, computed-property, or expression
  return bodies.
- `offsetSet()` interaction for the nested one-key store, except as a
  regression proving empty append stores still use it and nested stores do
  not.
- Magic `__set()`, `__isset()`, or `__unset()` interaction beyond proving
  `__set()` is not called for the covered nested append store.
- Method-return or factory holder roots such as
  `$registry->holder()->missing["outer"][] = $array` and
  `make_holder()->missing["outer"][] = $array`.
- Mixed nested `ArrayAccess` chains where the selected `"outer"` bucket itself
  contains another `ArrayAccess` object that would require a second dispatch.
- Dynamic, append, object, or side-effecting parent index expressions beyond
  the parser/evaluator subset explicitly tested for this lane.
- Scalar existing selected buckets unless the implementation intentionally
  matches PHP's exact overwrite/error behavior and proves it with system PHP
  fixtures.
- Broad same-container identity after replacing the holder object, replacing
  the magic returned object, replacing the backing storage property, unsetting
  the selected bucket, or reassigning source variables that supplied nested
  reference slots.
- Full PHP references/COW, native reference lowering, exact alias
  destruction/destructor ordering, and native `compile` support.

## Landing Gate

- `cargo fmt --check`
- `git diff --check`
- Focused `functions_and_scopes` filters for direct named, direct dynamic,
  non-direct named, non-direct dynamic, existing-bucket append,
  missing-or-null bucket materialization if covered, by-value `offsetGet()`
  no-op, empty `offsetSet(null)` append-store regression, plain-array nested
  append regression, reference-source regression, evaluate-once, and `__set()`
  guard cases.
- `cargo run -q -p phpc -- test --compare-php tests/fixtures/milestone1692`
- Adjacent fixture comparisons for `tests/fixtures/milestone1687`,
  `tests/fixtures/milestone1688`, `tests/fixtures/milestone1690`, and
  `tests/fixtures/milestone1691`.
- Native rejection checks for representative direct and non-direct magic
  `ArrayAccess` nested append stores with both `--emit-ir` and `--emit-asm`.
- Broader verification if implementation touches shared parser
  assignment-target, magic-property, temporary-holder, `ArrayAccess`
  reference-source, nested append-alias, or reference-slot propagation
  helpers.
