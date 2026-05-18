# Lane 1693-D Review: Deeper Magic-Property ArrayAccess Nested Append Stores

## Landing Summary

Lane 1693-C landed the focused two-key parent path below a
magic-property `ArrayAccess` bucket. The intended write shape is
`$box->missing["outer"]["inner"][] = $array`: visible public `__get($name)`
returns an `ArrayAccess` object, public by-reference `offsetGet("outer")`
selects the backing bucket, and the remaining `"inner"` parent path is plain
array storage below that bucket before the final append.

This is the deeper companion to Lane 1692-C's one-key
`$box->missing["outer"][] = $array` path. It should not become broad
overloaded-property mutation, broad `ArrayAccess` path evaluation, or mixed
nested `ArrayAccess` dispatch. By-value `offsetGet()` remains PHP's
indirect-modification notice/no-op boundary for the covered deeper append
shape.

Validated before checkpoint: PHP lint for `tests/fixtures/milestone1693`,
`cargo check -q -p phpc`, `cargo fmt --check`, the `milestone1693`
`functions_and_scopes` filter with `2` tests, and `cargo run -q -p phpc --
test --compare-php tests/fixtures/milestone1693` with `5` fixtures, `5`
system PHP comparisons, and `0` skips.

## Target Slice

- Direct named deeper nested append store:
  `$box->missing["outer"]["inner"][] = $array`.
- Direct dynamic deeper nested append store:
  `$box->{$name}["outer"]["inner"][] = $array`, with the dynamic
  property-name expression evaluated once before magic lookup.
- Non-direct holder named deeper nested append store:
  `$holders["box"]->missing["outer"]["inner"][] = $array`, with the holder
  expression evaluated once before magic lookup.
- Non-direct holder dynamic deeper nested append store:
  `$holders["box"]->{$name}["outer"]["inner"][] = $array`, with both the
  holder and dynamic property-name expressions evaluated once.
- The magic root must resolve through visible public `__get($name)` to an
  `ArrayAccess` object.
- The first parent key, such as `"outer"`, must be selected through the
  existing exact public by-reference `offsetGet($offset) { return
  $this->property[$offset]; }` bridge.
- The remaining parent path below the returned bucket, such as `"inner"`,
  must be plain PHP array storage after materialization.
- Missing or `null` parents below the selected `offsetGet()` bucket may be
  materialized as arrays when system PHP comparison proves that behavior for
  the covered shape.
- Preserve nested reference slots when the appended value is an array literal
  containing reference elements or a copied array carrying mirrored alias
  metadata.
- Prove later reads through the same magic `ArrayAccess` bucket observe
  writes through referenced nested elements, while ordinary copied fields
  remain detached.

## PHP Semantics To Prove

- `__get($name)` is called exactly once for each covered append store.
- For dynamic property names, the property-name expression is evaluated once
  before `__get()` dispatch.
- For non-direct holders, the holder expression is evaluated once, and magic
  lookup runs on that evaluated holder object.
- By-reference `offsetGet("outer")` is called for the first parent bucket,
  and the append mutates the backing bucket returned by reference.
- The `"inner"` parent below the selected bucket is treated as a plain-array
  parent. Existing sibling keys below `"outer"` must be preserved, and the
  final `[]` append must use the next integer key under `"inner"`.
- Missing or `null` plain-array parents below the returned bucket should be
  materialized only for states proven against system PHP fixtures.
- By-value `offsetGet($offset)` for the same two-key nested append stays in
  PHP's indirect-modification notice/no-op behavior. It must not mutate the
  backing `ArrayAccess` storage and must not be documented as supported
  mutation.
- Append-store semantics remain separate from reference-source semantics.
  `$box->missing["outer"]["inner"][] = $array` is a write through the
  by-reference `offsetGet("outer")` bucket plus a plain-array suffix, while
  `$alias =& $box->missing["outer"]["inner"][]` should stay on the existing
  append reference-source path or no-op behavior.
- Empty magic-property `ArrayAccess` append stores remain on
  `offsetSet(null, $value)`, and Lane 1692's one-key nested stores remain the
  first `offsetGet()` bucket append path.

## Tests Needed

- Direct named by-reference deeper append with an array literal containing a
  reference element; mutating the original referenced variable after append
  should be visible through
  `$box->missing["outer"]["inner"][0]["ref"]`.
- Direct dynamic by-reference deeper append with the same reference-slot proof
  and a property-name evaluate-once counter.
- Non-direct holder named by-reference deeper append with a holder
  evaluate-once counter and the same reference-slot proof.
- Non-direct holder dynamic by-reference deeper append with both holder and
  property-name evaluate-once counters.
- Missing or `null` `"inner"` parent materialization below an existing
  `offsetGet("outer")` bucket, if included.
- Existing selected bucket append where `"outer"` contains an `"inner"` array
  with sibling keys; the append should preserve siblings and use the next
  integer key under `"inner"`.
- Copied-array source case: append a copied array that already carries
  mirrored alias metadata, then prove later mutation through a covered nested
  reference slot writes through.
- Ordinary copied-field detachment: mutate a non-reference field after append
  and prove the stored value remains detached.
- By-value `offsetGet()` contrast proving the deeper nested append emits the
  bounded indirect-modification diagnostic path, leaves backing storage
  unchanged, and does not accidentally create an alias.
- Guard proving `__get($name)` is called once and `__set()` is not called for
  the covered `ArrayAccess` nested append case.
- Regression preserving Lane 1692 one-key magic-property `ArrayAccess` nested
  append stores.
- Regression preserving empty magic-property `ArrayAccess` append stores:
  `$box->missing[] = $array` and `$holders["box"]->missing[] = $array` should
  still use `offsetSet(null, $value)`.
- Regression preserving magic plain-array deeper append stores: the
  plain-array path applies only when `__get()` returns a direct variable by
  reference containing an array or `null`, not when it returns an
  `ArrayAccess` object.
- Reference-source regressions proving
  `$alias =& $box->missing["outer"]["inner"][]` and
  `$alias =& $holders["box"]->missing["outer"]["inner"][]` stay off the
  append-store mutation path.
- CLI fixtures under a new `tests/fixtures/milestone1693` directory with
  system PHP comparison for direct named, direct dynamic, non-direct named,
  non-direct dynamic, by-value `offsetGet()` no-op contrast, and
  reference-source contrast cases.
- Native rejection checks for `phpc compile --emit-ir` and `--emit-asm`;
  native lowering should continue rejecting magic-property, `ArrayAccess`,
  reference, and COW shapes instead of emitting misleading code.

## Likely Implementation Hooks

- Parser support should already carry the full parent prefix on object
  property array append targets used by Lane 1691 and Lane 1692. Confirm that
  direct named and dynamic forms keep both `"outer"` and `"inner"` and that
  non-direct named and dynamic forms preserve the same prefix through the
  temporary-holder path.
- Keep this branch distinct from the empty `offsetSet(null, $value)` append
  store path, Lane 1692's one-key `offsetGet()` bucket append path, and the
  magic plain-array append helpers.
- Reuse the visible public `__get($name)` lookup route and temporary holder
  machinery from the direct and non-direct magic-property lanes.
- Reuse the exact public by-reference `offsetGet($offset) { return
  $this->property[$offset]; }` bridge to bind the first selected backing
  bucket.
- After the first bucket is selected, apply the remaining parent-key suffix as
  plain-array storage, then pass the full destination prefix including the
  actual appended key into the reference-slot propagation hooks.
- Ensure by-value `offsetGet()` is detected before mutation and routed to the
  indirect-modification notice/no-op behavior.
- Native lowering rejection strings in `compiler/src/codegen.rs` should remain
  broad enough to reject the new interpreter-only shape.

## Unsupported Boundaries

- Empty magic-property `ArrayAccess` append stores remain the Lane 1687 and
  Lane 1688 `offsetSet(null, $value)` behavior.
- One-key magic-property `ArrayAccess` nested append stores remain Lane 1692.
- By-value `offsetGet()` mutation. This lane should test the notice/no-op
  behavior, not turn it into a write-through path.
- Magic `__get()` bodies that return properties, offsets, method calls,
  object fields, newly constructed objects, or arbitrary expressions outside
  the bounded root-object shape.
- `offsetGet()` bodies beyond the exact public
  `return $this->property[$offset];` bridge, including side-effecting,
  guarded, private/protected-only, nested, computed-property, or expression
  return bodies.
- Mixed nested `ArrayAccess` chains where the selected `"outer"` bucket or a
  later parent such as `"inner"` contains another `ArrayAccess` object that
  would require another dispatch.
- Deeper `ArrayAccess` parent paths beyond the focused first
  `offsetGet()` bucket plus plain-array suffix, including arbitrary recursive
  `ArrayAccess` descent.
- Method-return or factory holder roots such as
  `$registry->holder()->missing["outer"]["inner"][] = $array` and
  `make_holder()->missing["outer"]["inner"][] = $array`.
- Dynamic, append, object, or side-effecting parent index expressions beyond
  the parser/evaluator subset explicitly tested for this lane.
- Scalar existing selected buckets or scalar plain-array parents unless the
  implementation intentionally matches PHP's exact overwrite/error behavior
  and proves it with system PHP fixtures.
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
  missing-or-null suffix materialization if covered, by-value `offsetGet()`
  no-op, empty `offsetSet(null)` append-store regression, Lane 1692 one-key
  append regression, plain-array deeper append regression,
  reference-source regression, evaluate-once, and `__set()` guard cases.
- `cargo run -q -p phpc -- test --compare-php tests/fixtures/milestone1693`
- Adjacent fixture comparisons for `tests/fixtures/milestone1687`,
  `tests/fixtures/milestone1688`, `tests/fixtures/milestone1691`, and
  `tests/fixtures/milestone1692`.
- Native rejection checks for representative direct and non-direct deeper
  magic `ArrayAccess` nested append stores with both `--emit-ir` and
  `--emit-asm`.
- Broader verification if implementation touches shared parser
  assignment-target, magic-property, temporary-holder, `ArrayAccess`
  reference-source, nested append-alias, or reference-slot propagation
  helpers.
