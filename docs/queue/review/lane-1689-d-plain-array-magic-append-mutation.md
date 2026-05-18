# Lane 1689-D Review: Plain-Array Magic Append Mutation

Lane 1689 should cover the next COW blocker after Lane 1688-C: append stores
below magic properties when `__get()` returns a plain array cell by reference.
Lane 1687-C and Lane 1688-C proved the `ArrayAccess` object append-store
route through `offsetSet(null, $value)`. This lane should stay on the
plain-array route and must not reuse the `ArrayAccess` append-store shortcut
for reference-source reads.

## Landing Summary

Lane 1689-C landed the focused plain-array append-store slice for visible
public `__get($name)` returning a direct variable by reference. Covered forms
are direct named, direct dynamic, non-direct named, and dynamic non-direct
empty appends. The runtime appends into the returned array/null cell,
canonicalizes metadata from the hidden magic temporary back to a visible
static variable sharing that cell, and preserves nested reference slots for
appended array literals and copied arrays carrying mirrored alias metadata.
By-value `__get()` returning a plain array or `null` remains a PHP-compatible
indirect-modification notice/no-op for the covered array RHS shape.

## Target Slice

- Direct named magic append store: `$box->missing[] = $array`.
- Direct dynamic magic append store: `$box->{$name}[] = $array`, with the
  property-name expression evaluated once.
- Non-direct holder named magic append store:
  `$holders["box"]->missing[] = $array`, with the holder expression evaluated
  once before magic lookup.
- Non-direct holder dynamic magic append store:
  `$holders["box"]->{$name}[] = $array`, with both holder and property-name
  expressions evaluated once.
- Only the plain-array case is in scope: visible public `__get($name)` returns
  a direct variable by reference, that returned cell currently holds an array
  or can be materialized as an array, and the append writes into that returned
  array cell.
- Preserve nested reference slots when the appended value is an array literal
  containing reference elements or a copied array carrying mirrored alias
  metadata.
- Prove later reads through the same magic array cell observe mutation of
  referenced nested elements, while ordinary copied fields remain detached.
- Keep the existing `ArrayAccess` append-store lanes separate:
  `$box->missing[] = $array` should use `offsetSet(null, $value)` only when
  magic `__get()` returns an `ArrayAccess` object. The plain-array lane should
  append through array alias metadata under the returned reference cell.

## PHP Semantics To Prove

- By-reference `__get()` plain-array append mutates the backing returned cell.
  A later read through `$box->missing` or `$box->missing[$key]` should see the
  appended element.
- By-value `__get()` plain-array append is a notice/no-op style indirect
  modification path. It may read a detached value for the operation, but it
  must not mutate the object's backing storage.
- By-reference `__get()` that returns an `ArrayAccess` object remains the
  Lane 1687/Lane 1688 object path: append store dispatches
  `offsetSet(null, $value)`.
- By-value `__get()` returning an `ArrayAccess` object for reference-source
  forms such as `$alias =& $box->missing[]` remains the Lane 1680
  `offsetGet(null)` notice/no-op path and must not be rerouted through
  append-store handling.
- Direct and dynamic property names should have the same observable semantics
  after the property expression has been evaluated once.
- Non-direct holder forms should mutate the object produced by the holder
  expression, not re-evaluate the holder or mutate a temporary detached copy.
- `__get($name)` should be called exactly once for each covered append store.
  `__set()` should not be called for the by-reference returned array cell.

## Tests Needed

- Direct named by-reference plain-array append store with an array literal that
  contains a reference element. Mutating the original referenced variable after
  append should be visible through the appended magic array slot.
- Direct dynamic by-reference plain-array append store with the same reference
  preservation proof and a property-name evaluate-once counter.
- Non-direct holder named by-reference plain-array append store with a holder
  evaluate-once counter and the same nested reference-slot proof.
- Non-direct holder dynamic by-reference plain-array append store with both
  holder and property-name evaluate-once counters.
- Copied-array source case: append a copied array that already carries mirrored
  alias metadata, then prove by-reference foreach or direct slot mutation
  still writes through covered nested reference slots.
- By-value `__get()` plain-array append regression proving the backing storage
  is unchanged and the expected indirect-modification diagnostic behavior
  remains stable.
- `ArrayAccess` regression proving Lane 1687/Lane 1688 append stores still
  call `offsetSet(null, $value)`.
- Reference-source regression proving `$alias =& $box->missing[]` and
  `$alias =& $holders["box"]->missing[]` still use the existing
  reference-source path, not the append-store path.
- Guards for `__get()` called once and `__set()` not called for the covered
  by-reference plain-array append mutation.
- CLI fixtures under a new `tests/fixtures/milestone1689` directory with
  system PHP comparison for direct named, direct dynamic, non-direct named,
  non-direct dynamic, and by-value notice/no-op contrast cases.
- Native rejection checks for `phpc compile --emit-ir` and `--emit-asm`;
  native lowering should continue rejecting magic-property/reference/COW
  shapes instead of emitting misleading code.

## Likely Implementation Hooks

- Parser assignment targets already recognize direct and non-direct
  magic-property append forms from the recent `ArrayAccess` lanes; keep any
  parser changes narrow if a missing plain-array variant appears.
- `evaluate_magic_get_array_append_reference_source_alias()` and
  `evaluate_non_direct_holder_magic_get_array_append_reference_source_alias()`
  already model by-reference `__get()` append sources. The store path likely
  needs a sibling that performs an append write and then binds or mirrors
  nested reference metadata onto the appended alias.
- `call_magic_get_reference_return_cell()` is the right boundary for
  by-reference `__get()` plain-array mutation. By-value `__get()` should stay
  outside the mutation path except for producing the documented no-op notice.
- `scope.append_array_offset_reference_alias()` and
  `scope.append_object_property_array_offset_reference_alias()` show the
  current append-alias machinery. The plain-array magic case should append to
  a temporary name bound to the returned `__get()` cell, then use the returned
  alias root and keys as the metadata target.
- `bind_or_mirror_array_references_to_alias_root()`,
  `bind_array_literal_references_to_alias_root_with_prefix()`, and
  `mirror_copied_array_aliases_to_alias_root()` are the likely metadata
  propagation hooks for array literals and copied arrays.
- `write_magic_get_array_access_append_with_reference_propagation()` is a
  useful contrast point, not the implementation target. It handles
  `ArrayAccess` append stores and must remain separate from plain-array
  mutation.
- Native lowering rejection strings in `compiler/src/codegen.rs` should remain
  broad enough to reject the new interpreter-only shape.

## Unsupported Boundaries

- Magic `__get()` bodies that return properties, offsets, method calls, object
  fields, or arbitrary expressions instead of a direct variable by reference.
- By-value `__get()` plain-array mutation. This lane should document and test
  the notice/no-op behavior, not turn it into a mutation path.
- Append stores below an `ArrayAccess` object returned by magic `__get()` are
  already Lane 1687/Lane 1688; keep them only as regressions.
- Reference-source forms such as `$alias =& $box->missing[]` are not append
  stores. They should stay on the existing reference-source alias or
  notice/no-op paths.
- Non-empty nested append paths such as `$box->missing["outer"][] = $array`
  and `$holders["box"]->{$name}["outer"][] = $array`, unless explicitly
  implemented and tested as a later slice.
- Method-return or factory holder roots such as
  `$registry->holder()->missing[] = $array` and
  `make_holder()->missing[] = $array`.
- Inaccessible declared-property edge cases beyond the existing bounded magic
  fallback behavior.
- Broad same-container identity after replacing the object, replacing the
  returned backing array variable, unsetting the returned cell, or assigning a
  non-array value through another alias.
- Mixed nested `ArrayAccess` chains, full PHP references/COW, native reference
  lowering, and exact alias destruction/destructor ordering.

## Landing Gate

- `cargo fmt --check`
- `git diff --check`
- Focused `functions_and_scopes` filters for direct named, direct dynamic,
  non-direct named, non-direct dynamic, by-value `__get()` no-op, `ArrayAccess`
  append-store regression, reference-source regression, evaluate-once, and
  `__set()` guard cases.
- `cargo run -q -p phpc -- test --compare-php tests/fixtures/milestone1689`
- Adjacent fixture comparisons for `tests/fixtures/milestone1680`,
  `tests/fixtures/milestone1687`, and `tests/fixtures/milestone1688`.
- Broader verification if implementation touches shared assignment-target,
  magic-property, temporary-holder, append-alias, or reference-slot propagation
  helpers.
