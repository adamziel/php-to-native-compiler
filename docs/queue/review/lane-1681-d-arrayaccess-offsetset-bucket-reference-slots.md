# Lane 1681-D Review: ArrayAccess `offsetSet()` Bucket Reference Slots

This note tracks the focused COW lane landed in the shared worktree.

## Integrated Slice

- Direct keyed `ArrayAccess::offsetSet($offset, $value)` stores are recognized
  when the public method body is exactly
  `$this->property[$offset] = $value;`.
- If the stored value is an array literal with reference slots, those
  reference slots are bound onto the backing bucket.
- If the stored value is a copied array with alias metadata, those aliases are
  mirrored onto the backing bucket.
- Later exact `offsetGet($offset) { return $this->property[$offset]; }`
  bucket copies preserve the nested reference slots through public or
  method-context private/protected backing properties.

## Focused Proof

- `compiler/tests/functions_and_scopes.rs` covers the runtime behavior with a
  direct regression and a system-PHP comparison helper.
- `tests/fixtures/milestone1681/arrayaccess_offsetset_bucket_arbitrary_reference_slot_cow.php`
  provides the CLI fixture.

## Still Unsupported

- Append `offsetSet(null)` stored-bucket reference slots.
- Non-direct receiver `offsetSet()` storage.
- Side-effecting or broader `offsetSet()`/`offsetGet()` bodies.
- Broader mixed `ArrayAccess` chains.
- Full references/COW, native reference lowering, and exact alias destruction
  or destructor ordering.
