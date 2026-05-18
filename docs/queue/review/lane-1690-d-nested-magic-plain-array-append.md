# Lane 1690-D Review: Nested Magic Plain-Array Append Stores

Lane 1690 should cover the next plain-array magic append gap after
Lane 1689-C: non-empty nested append stores below a magic property whose
visible public `__get($name)` returns a direct variable by reference. Lane
1689-C proved empty appends such as `$box->missing[] = $array`; this lane
should extend only the plain-array route to paths such as
`$box->missing["outer"][] = $array`.

## Landing Summary

Lane 1690-C landed the focused one-key nested append-store shape for visible
public `__get($name)` returning a direct variable by reference. Covered forms
are direct named, direct dynamic, non-direct named, and dynamic non-direct
`["outer"][]` appends. The runtime passes the parent key path into the plain
array append-alias machinery, materializes missing/null parents under the
returned cell, canonicalizes metadata from the hidden magic temporary back to
a visible static variable sharing that cell, and preserves nested reference
slots for appended array literals and copied arrays. By-value `__get()`
returning a plain array or `null` remains a PHP-compatible
indirect-modification notice/no-op for the covered nested array RHS shape.
Magic `ArrayAccess` non-empty nested append remains unsupported.

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
- Only the plain-array case is in scope: visible public `__get($name)` returns
  a direct variable by reference, that returned cell currently holds an array
  or `null`, and the nested append writes through that returned array cell.
- The selected non-empty prefix should be materialized as PHP array storage
  when needed, then the final `[]` append should attach reference metadata to
  the actual appended slot under that prefix.
- Preserve nested reference slots when the appended value is an array literal
  containing reference elements or a copied array carrying mirrored alias
  metadata.
- Prove later reads through the same magic array cell observe writes through
  referenced nested elements, while ordinary copied fields remain detached.

## PHP Semantics To Prove

- By-reference `__get()` plain-array nested append mutates the backing
  returned cell. A later `$box->missing["outer"][0]` or equivalent read should
  see the appended element.
- A missing nested prefix such as `"outer"` is materialized as an array before
  appending. A `null` returned root may be materialized into an array in the
  same bounded way as Lane 1689's empty append.
- By-value `__get()` plain-array nested append remains PHP's
  indirect-modification notice/no-op path. It must not mutate backing object
  storage and must not call `__set()`.
- Direct and dynamic property spellings should have the same post-evaluation
  behavior; the dynamic property expression is evaluated once.
- Non-direct holder spellings should mutate the object produced by the holder
  expression; the holder must not be re-evaluated or replaced by a detached
  temporary array copy.
- `__get($name)` should be called exactly once for each covered append store.
- Append-store semantics must remain separate from reference-source semantics:
  `$box->missing["outer"][] = $array` is a write, while
  `$alias =& $box->missing["outer"][]` is a reference-source form and should
  continue through the existing append reference-source alias or no-op path.
- Magic plain-array append stores must remain separate from magic
  `ArrayAccess` append stores. If `__get()` returns an `ArrayAccess` object,
  the Lane 1687/Lane 1688 path dispatches `offsetSet(null, $value)` only for
  the empty append target already covered there; this lane should not broaden
  mixed nested `ArrayAccess` chains.

## Tests Needed

- Direct named by-reference nested append with an array literal containing a
  reference element; mutating the original referenced variable after append
  should be visible through `$box->missing["outer"][0]["ref"]`.
- Direct dynamic by-reference nested append with the same reference-slot proof
  and a property-name evaluate-once counter.
- Non-direct holder named by-reference nested append with a holder
  evaluate-once counter and the same nested reference-slot proof.
- Non-direct holder dynamic by-reference nested append with both holder and
  property-name evaluate-once counters.
- Missing-prefix materialization: `$box->missing["outer"][] = $array` should
  create the `"outer"` array bucket below the returned cell.
- Existing-prefix append: a preexisting `$backing["outer"]` array should
  receive the next integer append key without replacing sibling buckets.
- Null-root materialization if this lane chooses to match Lane 1689's
  array-or-null root behavior for nested stores.
- Copied-array source case: append a copied array that already carries
  mirrored alias metadata, then prove a later by-reference foreach or direct
  slot mutation still writes through covered nested reference slots.
- Ordinary copied-field detachment: mutate a non-reference field after append
  and prove the stored copy remains detached.
- By-value `__get()` plain-array nested append regression proving backing
  storage is unchanged and the indirect-modification diagnostic behavior
  remains stable.
- `ArrayAccess` regressions proving existing magic append stores still use
  `offsetSet(null, $value)` only for the covered empty append store, and that
  this lane does not route nested mixed shapes through that store shortcut.
- Reference-source regressions proving
  `$alias =& $box->missing["outer"][]` and
  `$alias =& $holders["box"]->missing["outer"][]` stay on the existing
  reference-source path instead of the append-store path.
- Guards for `__get()` called once and `__set()` not called for the covered
  by-reference plain-array nested append mutation.
- CLI fixtures under a new `tests/fixtures/milestone1690` directory with
  system PHP comparison for direct named, direct dynamic, non-direct named,
  non-direct dynamic, by-value no-op contrast, and reference-source contrast
  cases.
- Native rejection checks for `phpc compile --emit-ir` and `--emit-asm`;
  native lowering should continue rejecting magic-property/reference/COW
  shapes instead of emitting misleading code.

## Likely Implementation Hooks

- Parser support likely already represents direct nested property appends as
  `AssignTarget::ObjectPropertyArrayAppend { indices, .. }` and dynamic direct
  property appends as `AssignTarget::DynamicObjectPropertyArrayAppend {
  indices, .. }`. The non-direct parser path currently routes object-property
  array targets through `NonDirectObjectPropertyArrayIndex` and
  `NonDirectDynamicObjectPropertyArrayIndex`; confirm whether non-empty nested
  append syntax reaches those variants with `indices` including the prefix or
  needs a narrow AST distinction.
- `write_magic_get_array_access_append_with_reference_propagation()` now
  contains Lane 1689's plain-array by-reference branch for empty appends. A
  nested sibling or parameterized helper should append under the evaluated
  prefix instead of hard-coding `Vec::new()`.
- For direct named and direct dynamic stores, the relevant assignment handling
  is in the `ObjectPropertyArrayAppend` and
  `DynamicObjectPropertyArrayAppend` interpreter arms. Keep dynamic property
  evaluation before `__get()` and evaluate index expressions in PHP order.
- For non-direct named and dynamic stores, the likely entry points are the
  `NonDirectObjectPropertyArrayIndex` and
  `NonDirectDynamicObjectPropertyArrayIndex` arms that materialize the holder
  into a private temporary object root. Reuse that evaluate-once holder route.
- `call_magic_get_reference_return_cell()` remains the boundary for the
  supported by-reference `__get()` body. By-value `__get()` should remain in
  the no-op notice path.
- `scope.append_array_offset_reference_alias()` can append beneath a supplied
  key prefix; use it with the evaluated nested prefix instead of the empty
  prefix used by Lane 1689.
- `scope.canonical_equivalent_static_array_alias_root()` is needed after
  binding the hidden magic temporary to the returned cell, so metadata lands
  on the visible static root sharing that cell rather than staying trapped on
  the temporary name.
- `bind_or_mirror_array_references_to_alias_root()`,
  `bind_array_literal_references_to_alias_root_with_prefix()`, and
  `mirror_copied_array_aliases_to_alias_root()` are the expected propagation
  hooks for literal reference elements and copied arrays.
- Existing `write_nested_array_append()` behavior is useful as a value-store
  reference point, but the magic path must preserve alias metadata under the
  returned `__get()` cell.
- Native lowering rejection strings in `compiler/src/codegen.rs` should remain
  broad enough to reject the new interpreter-only shape.

## Unsupported Boundaries

- Empty magic plain-array append stores are Lane 1689 and should be kept as
  regressions, not redefined here.
- By-value `__get()` plain-array mutation. This lane should test the
  notice/no-op behavior, not turn it into a mutation path.
- Magic `__get()` bodies that return properties, offsets, method calls,
  object fields, newly constructed arrays, or arbitrary expressions instead
  of a direct variable by reference.
- Magic-property roots without an array prefix and reference-source forms
  such as `$alias =& $box->missing["outer"][]`; append-store and
  reference-source behavior must stay separate.
- Mixed nested `ArrayAccess` chains such as
  `$box->missing["outer"][] = $array` when `missing` or `"outer"` resolves to
  an `ArrayAccess` object. The `ArrayAccess` append-store route remains the
  existing `offsetSet(null, $value)` empty-append path.
- Method-return or factory holder roots such as
  `$registry->holder()->missing["outer"][] = $array` and
  `make_holder()->missing["outer"][] = $array`.
- Side-effecting, append, or unsupported computed nested prefix keys beyond
  the parser/evaluator subset explicitly tested for this lane.
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
  non-direct named, non-direct dynamic, missing-prefix materialization,
  existing-prefix append, by-value `__get()` no-op, `ArrayAccess`
  append-store regression, reference-source regression, evaluate-once, and
  `__set()` guard cases.
- `cargo run -q -p phpc -- test --compare-php tests/fixtures/milestone1690`
- Adjacent fixture comparisons for `tests/fixtures/milestone1680`,
  `tests/fixtures/milestone1687`, `tests/fixtures/milestone1688`, and
  `tests/fixtures/milestone1689`.
- Native rejection checks for representative direct and non-direct nested
  magic append stores with both `--emit-ir` and `--emit-asm`.
- Broader verification if implementation touches shared parser
  assignment-target, magic-property, temporary-holder, nested append-alias, or
  reference-slot propagation helpers.
