# Lane 1691-D Review: Deeper Magic Plain-Array Append Stores

## Landing Summary

Lane 1691-C landed the focused two-key plain-array append shape described
below. The runtime now passes the full evaluated parent-key path into the
magic plain-array append helper for direct named/dynamic and non-direct
named/dynamic roots, while still keeping magic `ArrayAccess` non-empty nested
append stores unsupported. Fixture coverage lives in
`tests/fixtures/milestone1691/` and the focused Rust filter is
`magic_property_plain_array_deep_nested_append`.

Validated before checkpoint: `php -l tests/fixtures/milestone1691/*.php`,
`cargo check -q -p phpc`, the focused `functions_and_scopes` filter with `5`
tests, and `cargo run -q -p phpc -- test --compare-php
tests/fixtures/milestone1691` with `5` fixtures, `5` system PHP comparisons,
and `0` skips.

Lane 1691 should cover the next plain-array magic append gap after
Lane 1690-C: deeper parent paths below a magic property whose visible public
`__get($name)` returns a direct variable by reference. Lane 1690-C proved the
one-parent-key shape such as `$box->missing["outer"][] = $array`; this lane
should extend the same plain-array route to paths such as
`$box->missing["outer"]["inner"][] = $array`.

## Target Slice

- Direct named deeper append store:
  `$box->missing["outer"]["inner"][] = $array`.
- Direct dynamic deeper append store:
  `$box->{$name}["outer"]["inner"][] = $array`, with the dynamic property-name
  expression evaluated once before magic lookup.
- Non-direct holder named deeper append store:
  `$holders["box"]->missing["outer"]["inner"][] = $array`, with the holder
  expression evaluated once before magic lookup.
- Non-direct holder dynamic deeper append store:
  `$holders["box"]->{$name}["outer"]["inner"][] = $array`, with both the
  holder and dynamic property-name expressions evaluated once.
- Only the plain-array case is in scope: visible public `__get($name)` returns
  a direct variable by reference, that returned cell currently holds an array
  or `null`, and every parent on the covered path is plain PHP array storage
  after materialization.
- The parent path should be treated as an ordered prefix, not a single special
  key. Missing or `null` parents below the returned magic cell should be
  materialized as arrays until the final `[]` append point is reached.
- Preserve nested reference slots when the appended value is an array literal
  containing reference elements or a copied array carrying mirrored alias
  metadata.
- Prove later reads through the same magic array cell observe writes through
  referenced nested elements, while ordinary copied fields remain detached.

## PHP Semantics To Prove

- By-reference `__get()` plain-array deeper append mutates the backing
  returned cell. A later `$box->missing["outer"]["inner"][0]` or equivalent
  read should see the appended element.
- Direct, dynamic, non-direct, and dynamic non-direct roots should converge on
  the same post-evaluation mutation behavior. Dynamic property-name
  expressions and holder expressions must be evaluated exactly once.
- `__get($name)` should be called exactly once for each covered append store.
- Missing parent buckets such as `"outer"` and `"inner"` should be
  materialized as arrays. Existing parent arrays should be reused without
  replacing sibling buckets or resetting their next append key.
- By-value `__get()` plain-array deeper append remains PHP's
  indirect-modification notice/no-op path. It must not mutate backing object
  storage and must not call `__set()`.
- Append-store semantics must remain separate from reference-source
  semantics. `$box->missing["outer"]["inner"][] = $array` is a write through
  the returned by-reference cell, while
  `$alias =& $box->missing["outer"]["inner"][]` is a reference-source form and
  should continue through the existing append reference-source alias or no-op
  path.
- Magic plain-array append stores must remain separate from magic
  `ArrayAccess` append stores. If the magic root or an intermediate parent is
  an `ArrayAccess` object, do not route the deeper plain-array write through
  the existing `offsetSet(null, $value)` append-store shortcut.

## Tests Needed

- Direct named by-reference deeper append with an array literal containing a
  reference element; mutating the original referenced variable after append
  should be visible through
  `$box->missing["outer"]["inner"][0]["ref"]`.
- Direct dynamic by-reference deeper append with the same reference-slot proof
  and a property-name evaluate-once counter.
- Non-direct holder named by-reference deeper append with a holder
  evaluate-once counter and the same nested reference-slot proof.
- Non-direct holder dynamic by-reference deeper append with both holder and
  property-name evaluate-once counters.
- Missing-parent materialization for a fully absent path:
  `$box->missing["outer"]["inner"][] = $array`.
- Partial materialization where `"outer"` exists as an array but `"inner"` is
  missing or `null`.
- Existing-parent append where `"outer"` and `"inner"` already exist and
  contain sibling buckets; the append should use the next integer key under
  `"inner"` without disturbing siblings.
- Null-root materialization if this lane keeps Lane 1689 and Lane 1690's
  array-or-null root behavior for nested stores.
- Copied-array source case: append a copied array that already carries
  mirrored alias metadata, then prove a later by-reference foreach or direct
  slot mutation still writes through covered nested reference slots.
- Ordinary copied-field detachment: mutate a non-reference field after append
  and prove the stored copy remains detached.
- By-value `__get()` plain-array deeper append regression proving backing
  storage is unchanged and the indirect-modification diagnostic behavior
  remains stable.
- `ArrayAccess` regressions proving existing magic append stores still use
  `offsetSet(null, $value)` only for the covered empty append-store shapes,
  and that deeper mixed paths are rejected or no-op according to the existing
  reference-source boundaries instead of being treated as plain-array stores.
- Reference-source regressions proving
  `$alias =& $box->missing["outer"]["inner"][]` and
  `$alias =& $holders["box"]->missing["outer"]["inner"][]` stay on the
  existing reference-source path instead of the append-store path.
- Guards for `__get()` called once and `__set()` not called for the covered
  by-reference plain-array deeper append mutation.
- CLI fixtures under a new `tests/fixtures/milestone1691` directory with
  system PHP comparison for direct named, direct dynamic, non-direct named,
  non-direct dynamic, by-value no-op contrast, and reference-source contrast
  cases.
- Native rejection checks for `phpc compile --emit-ir` and `--emit-asm`;
  native lowering should continue rejecting magic-property/reference/COW
  shapes instead of emitting misleading code.

## Likely Implementation Hooks

- Parser support should already carry multiple index expressions on the
  object-property array append targets used by Lane 1690. Confirm that direct
  named and dynamic forms reach `ObjectPropertyArrayAppend` and
  `DynamicObjectPropertyArrayAppend` with the complete parent prefix, and that
  non-direct named and dynamic forms preserve all prefix indices through
  `NonDirectObjectPropertyArrayIndex` and
  `NonDirectDynamicObjectPropertyArrayIndex`.
- The Lane 1690 extension to
  `write_magic_get_array_access_append_with_reference_propagation()` should be
  generalized from a one-key parent path to an arbitrary evaluated prefix.
  Keep the by-reference plain-array branch distinct from the `ArrayAccess`
  append-store branch.
- For direct named and direct dynamic stores, continue using the interpreter
  arms for `ObjectPropertyArrayAppend` and
  `DynamicObjectPropertyArrayAppend`. Preserve PHP evaluation order: holder
  or root object, dynamic property name when present, parent indices from left
  to right, then RHS.
- For non-direct named and dynamic stores, continue using the temporary holder
  root path introduced for prior non-direct lanes. The evaluated holder object
  should be the receiver for the single `__get()` call, and the deeper parent
  prefix should be passed through unchanged.
- `call_magic_get_reference_return_cell()` remains the supported boundary for
  by-reference `__get()` bodies. By-value `__get()` should stay in the
  indirect-modification no-op path.
- `scope.append_array_offset_reference_alias()` is the expected alias hook for
  the final append slot; pass the full evaluated parent prefix rather than
  truncating it to the Lane 1690 single-key prefix.
- `scope.canonical_equivalent_static_array_alias_root()` should still be used
  after binding the hidden magic temporary to the returned cell, so metadata
  lands on the visible static root sharing that cell rather than remaining on
  the temporary root.
- `bind_or_mirror_array_references_to_alias_root()`,
  `bind_array_literal_references_to_alias_root_with_prefix()`, and
  `mirror_copied_array_aliases_to_alias_root()` are the likely propagation
  hooks for literal reference elements and copied arrays. The important
  review point is that they receive the complete append destination prefix,
  including every deeper parent key and the actual appended key.
- Existing plain-array nested write helpers can be used as a value-store
  reference, but the magic path must preserve alias metadata under the
  returned `__get()` cell.
- Native lowering rejection strings in `compiler/src/codegen.rs` should remain
  broad enough to reject the new interpreter-only shape.

## Unsupported Boundaries

- Empty magic plain-array append stores are Lane 1689 and one-key nested
  stores are Lane 1690; keep them as regressions, not redefined behavior.
- By-value `__get()` plain-array mutation. This lane should test the
  notice/no-op behavior, not turn it into a mutation path.
- Magic `__get()` bodies that return properties, offsets, method calls,
  object fields, newly constructed arrays, or arbitrary expressions instead
  of a direct variable by reference.
- Append reference-source forms such as
  `$alias =& $box->missing["outer"]["inner"][]`; append-store and
  reference-source behavior must stay separate.
- Mixed nested `ArrayAccess` chains where the magic root or an intermediate
  parent resolves to an `ArrayAccess` object. Append stores and
  reference-source reads through `ArrayAccess` remain the prior bounded
  lanes, not this plain-array extension.
- Method-return or factory holder roots such as
  `$registry->holder()->missing["outer"]["inner"][] = $array` and
  `make_holder()->missing["outer"]["inner"][] = $array`.
- Dynamic, append, object, or side-effecting parent index expressions beyond
  the parser/evaluator subset explicitly tested for this lane.
- Scalar existing parents below the returned magic cell unless the
  implementation intentionally matches PHP's exact overwrite/error behavior
  and proves it with system PHP fixtures.
- Inaccessible declared-property edge cases beyond the existing bounded magic
  fallback behavior.
- Broad same-container identity after replacing the object, replacing the
  returned backing array variable, unsetting the returned cell, or assigning a
  non-array value through another alias.
- Full PHP references/COW, native reference lowering, exact alias
  destruction/destructor ordering, and native `compile` support.

## Landing Gate

- `cargo fmt --check`
- `git diff --check`
- Focused `functions_and_scopes` filters for direct named, direct dynamic,
  non-direct named, non-direct dynamic, missing-parent materialization,
  partial-parent materialization, existing-parent append, by-value `__get()`
  no-op, `ArrayAccess` append-store regression, reference-source regression,
  evaluate-once, and `__set()` guard cases.
- `cargo run -q -p phpc -- test --compare-php tests/fixtures/milestone1691`
- Adjacent fixture comparisons for `tests/fixtures/milestone1689`,
  `tests/fixtures/milestone1690`, and any touched `ArrayAccess`
  append-store/reference-source milestone fixtures.
- Native rejection checks for representative direct and non-direct deeper
  magic append stores with both `--emit-ir` and `--emit-asm`.
- Broader verification if implementation touches shared parser
  assignment-target, magic-property, temporary-holder, nested append-alias, or
  reference-slot propagation helpers.
