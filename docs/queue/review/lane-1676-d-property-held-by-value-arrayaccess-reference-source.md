# Lane 1676-D Review: Property-Held By-Value `ArrayAccess::offsetGet()` Reference Sources

This is review/triage only. Do not move any of this into supported docs until
implementation code, Rust coverage, PHP-comparable fixtures, and a CLI exercise
prove the behavior.

## Current Runtime Shape

- In the committed Lane 1675 baseline, the detached notice/no-op path exists
  only for direct object roots. In `execute_reference_assignment()`, the direct
  `ReferenceSource::ArrayIndex` and `ReferenceSource::NestedArrayIndex`
  branches call `evaluate_direct_array_access_by_value_reference_source_value()`
  before the existing by-reference alias bridge.
- The pre-1676 property-held branches call only
  `evaluate_object_property_array_access_reference_source_alias()`, then the
  magic `__get()` bridge, then the existing reject/bind fallback. When the
  selected property holds an `ArrayAccess` object whose public
  `offsetGet($offset)` returns by value, the alias helper reaches
  `evaluate_array_access_reference_source_alias_for_object()` and rejects
  because the method does not return by reference.
- `evaluate_array_access_by_value_reference_source_value_for_object()` already
  contains the bounded detached-value behavior for the exact public
  `return $this->property[$offset];` body: it emits the indirect-modification
  notice and reads the selected backing value without binding alias metadata.
  There is no property-held wrapper that reads the holder property and delegates
  to that helper.

During this review, concurrent WIP edits appeared in the worktree adding a
property-held by-value helper and calling it from the named
`ObjectPropertyArrayIndex` and `ObjectPropertyNestedArrayIndex` branches. Those
edits were not made in this review note. The visible dynamic-property branches
still do not call the helper directly, so before documenting dynamic-property
support, verify whether parser classification already routes the intended
dynamic-property forms through the named-property branches or add the same
helper call before their alias bridge.

## Observed PHP 8.2.29 Behavior

Local PHP 8.2.29 shows the same recoverable notice/no-op behavior for
property-held roots as for direct roots:

- `$alias =& $holder->bag["name"];` calls by-value `offsetGet("name")`, emits
  `E_NOTICE` level `8` with
  `Indirect modification of overloaded element of Bag has no effect`, and later
  writes to `$alias` do not mutate `$holder->bag->items["name"]`.
- `$alias =& $holder->{$property}["name"];` has the same detached behavior.
- `$alias =& $holders["box"]->bag["name"];` has the same detached behavior for
  a non-direct holder expression.
- `$alias =& $holder->bag["outer"]["slot"];` reads the outer value once, emits
  the notice, and later writes to the detached child alias leave the backing
  nested slot unchanged.

One important COW nuance remains outside the smallest scalar/child-slot slice:
if the by-value `offsetGet()` result is an array containing a nested reference
element, PHP detaches the outer array but keeps that nested reference element
shared; plain fields remain copied. Do not claim broad returned-array
reference-slot parity unless a later implementation explicitly proves it.

## Smallest Safe Change

Keep the by-reference alias bridge unchanged and add a property-held sibling to
the existing direct detached-value helper:

- Add a helper shaped like
  `evaluate_object_property_array_access_by_value_reference_source_value()`.
  It should mirror the first half of
  `evaluate_object_property_array_access_reference_source_alias()`:
  read the direct holder object, check property visibility through the current
  context, require the property value to be an `ArrayAccess` object, create the
  same hidden object name, and delegate to
  `evaluate_array_access_by_value_reference_source_value_for_object()`.
- In `execute_reference_assignment()`, call that helper before the alias helper
  for `ObjectPropertyArrayIndex` and `ObjectPropertyNestedArrayIndex`. On
  `Some(value)`, write the target variable with `scope.write_static(name,
  value)` and return.
- Either make the same explicit call in `DynamicObjectPropertyArrayIndex` and
  `DynamicObjectPropertyNestedArrayIndex`, or keep dynamic-property support
  undocumented until tests prove the parser always classifies the claimed
  forms into the named-property branches.
- Keep append-source forms out of this lane unless `offsetGet(null)` behavior
  is separately probed and tested for property-held roots.
- Keep non-direct holder roots out of the first patch unless the lane wants to
  cover them deliberately. If included, add a separate non-direct helper that
  evaluates the holder once into a temporary object root, then delegates to the
  direct property-held detached-value helper before falling back to alias/magic
  behavior.

This preserves the existing hard rejection for by-reference `offsetGet()` bodies
outside the exact bridge and avoids changing `evaluate_storable_reference_source_alias()`,
which should continue to return only real aliasable sources.

## Risks

- Do not return `None` from the by-value helper for exact by-value methods after
  side effects have occurred; the current exact bridge avoids executing broader
  method bodies, so unsupported bodies should remain outside this path.
- Notice routing must stay after the selected exact root/key is known and must
  use the existing `emit_notice()` path so `set_error_handler()` and
  `error_reporting()` behavior remain consistent with Lane 1675.
- The helper writes a detached target value, not an `ArrayOffsetAlias`; binding
  alias metadata here would make by-value `offsetGet()` PHP-incompatible.
- Returned arrays with nested reference elements are a COW/reference-container
  nuance. Scalar and selected child-slot tests are safe; broad array-return
  parity needs explicit coverage before being documented as supported.
- Dynamic property names should be evaluated exactly once in the existing
  branch before both by-value and alias helpers.

## Tests Needed

- Rust regression for direct property-held scalar source:
  `$alias =& $holder->bag["name"]; $alias = "changed";` should emit/capture the
  indirect-modification notice, leave `$holder->bag->items["name"]` unchanged,
  produce empty stderr under `set_error_handler()`, and exit `0`.
- Rust regression for direct dynamic property-held scalar source:
  `$alias =& $holder->{$property}["name"];` with the same assertions.
- Rust or PHP-comparable fixture for nested child source:
  `$alias =& $holder->bag["outer"]["slot"];` should leave the backing child
  unchanged after writing the detached alias.
- Keep the existing by-reference property-held test
  `property_held_array_access_reference_source_survives_holder_property_rebind`
  as a guard that the alias bridge still binds real by-reference `offsetGet()`
  results.
- If non-direct holder roots are included, add a separate fixture for
  `$alias =& $holders["box"]->bag["name"];` proving the holder is evaluated once
  and the result is detached.
- Add a phpc CLI exercise through `cargo run -q -p phpc -- test --compare-php`
  for the fixture directory before updating supported docs or progress.
