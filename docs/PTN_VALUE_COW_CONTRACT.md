# Refcounted PtnValue COW Payload Contract

Status: design-only. This document defines the ownership contract future COW
implementation work must follow. It does not claim current generated native
runtime support for PHP copy-on-write.

## Goals

PTN values need PHP-style copy-on-write for mutable string and array payloads:
ordinary assignment, by-value arguments, returns, constants, and temporaries
share payload storage until a write happens. The first write to a shared payload
detaches only the payload being written, then mutates the detached copy.

This replaces the current owned/deep-clone value model for strings and arrays
with counted heap payloads. Scalars remain immediate values.

## Runtime Shape

String and array values point at heap payload headers:

```c
typedef struct {
    size_t refcount;
    unsigned flags;
    size_t len;
    unsigned char data[];
} PtnStringPayload;

typedef struct {
    size_t refcount;
    unsigned flags;
    size_t len;
    size_t capacity;
    PtnArrayEntry *entries;
    PtnArrayIndexSlot *index_slots;
    size_t index_capacity;
    int64_t next_auto_key;
    size_t current_index;
} PtnArrayPayload;
```

The exact C layout may change, but the semantic contract may not:

- `PTN_STRING` and `PTN_ARRAY` values carry a pointer to a payload header.
- Counted payloads start with `refcount == 1`.
- Static literal payloads are immutable and effectively immortal. Retain and
  release are no-ops for them, and every write detaches first.
- `refcount == 0` is never observable on a live value. It is only the transition
  that frees a counted payload.
- A `PtnValue` that is stored in a slot owns one counted reference. Borrowed
  observations use `const PtnValue *` or equivalent and do not retain.

## Core Operations

Use these names consistently in implementation slices:

- `ptn_value_retain(PtnValue value) -> PtnValue`
  increments string/array payload refcounts unless the payload is immortal.
  Scalars are returned unchanged.
- `ptn_value_release(PtnValue *value)`
  decrements any counted string/array payload and nulls the input value. When an
  array payload reaches zero, it releases every stored entry value before
  freeing keys, index storage, entries, and the array header.
- `ptn_value_share(PtnValue value) -> PtnValue`
  is an alias for retain when a value crosses an ownership boundary such as a
  symbol-table write, array-entry write, call argument vector, constant read, or
  return value.
- `ptn_value_deep_clone(PtnValue value) -> PtnValue`
  allocates an independent string payload or array header. Array deep-clone
  duplicates keys and index storage but retains child entry values; nested
  payloads remain shared until their own write.
- `ptn_value_detach_for_write(PtnValue *slot)`
  ensures a string or array slot has a unique mutable payload. If the payload is
  counted with `refcount == 1`, no allocation happens. If `refcount > 1` or the
  payload is immutable/immortal, it deep-clones the target payload, releases the
  old payload, and stores the clone with `refcount == 1`.

Current `ptn_value_clone` performs deep copies. The COW migration must either
rename that helper or change it to the retain/share operation in one coherent
slice so call sites cannot confuse sharing with detaching.

## Borrowed And Owned Values

Every boundary must declare whether it borrows, consumes, or returns ownership:

- Runtime slots, symbol-table entries, constants, array entries, call argument
  vectors, active temporaries, and returned expression results own counted
  values.
- Runtime reads may return borrowed values only when the consumer cannot outlive
  the source slot. If a read result is stored, returned, passed into a call
  vector, or kept beyond the immediate expression helper call, retain it.
- Assignment `$b = $a`, by-value function arguments, by-value returns, and
  constant reads retain payloads instead of deep-copying them.
- A helper that consumes a value must document that it releases or transfers the
  value. A helper that only observes a value must not retain or release it.
- Moved values are passed with an explicit transfer convention. After a move,
  the source slot must be set to `null` or otherwise made impossible to release
  again.

## Detach Rules

Detach happens immediately before the first write that could mutate a PHP
payload:

- Array element assignment, append, unset, cursor-moving mutation, and in-place
  array helper operations detach the outer array first.
- String offset assignment, `.=` when implemented as append-in-place, and any
  future byte mutation helper detach the string first.
- A unique counted payload (`refcount == 1`) may be mutated in place.
- Shared counted payloads (`refcount > 1`) must allocate a replacement payload
  before the write.
- Immutable or immortal literal payloads always allocate a counted mutable copy
  before the write.
- Array detachment duplicates only the outer array storage and keys. Entry
  values are retained, so nested arrays/strings keep sharing until a write
  targets that nested value.
- Nested writes detach each level on the mutation path. For `$a["x"]["y"] = 1`,
  detach `$a` if needed, then detach the nested value at `"x"` if that nested
  value is shared.

## Symbol Tables, Constants, Temporaries, And Slots

Runtime storage owns counted values and must balance retain/release exactly:

- `ptn_symbols_set` retains or consumes the incoming value according to its API,
  then releases the overwritten slot value.
- `ptn_symbols_get` returns a borrowed value for immediate observation or a
  retained value for expression ownership. The API name must make the choice
  visible.
- `ptn_symbols_unset`, frame teardown, and runtime teardown release every stored
  value.
- Constants own their table values. A constant read returns a retained value,
  and the temporary expression result releases after its last use.
- Generated temporaries release at the end of their expression or statement in
  reverse creation order unless PHP destructor timing requires a narrower
  lifetime.
- Call argument vectors own retained values for the duration of the call and
  release them after the callee returns or throws.
- Return values transfer ownership from callee to caller. The callee must not
  release the transferred value during frame teardown.

## References

PHP references are a separate cell identity, not a replacement for payload COW:

- A `PtnReference` cell owns one `PtnValue` and has its own reference count.
- Multiple PHP aliases to the same reference cell observe the same value and do
  not detach from each other on writes through the reference.
- Copying a referenced value by value retains the payload currently inside the
  reference cell. Writes to the copied non-reference slot then use ordinary COW
  detach rules.
- If a reference cell contains an array or string, writing the payload through
  that reference detaches only when the payload is shared outside the reference
  cell.

## Destructor Behavior

Releases must happen at every ownership end:

- slot overwrite and unset;
- array entry replacement and unset;
- discarded expression and internal-call temporaries;
- call argument-vector cleanup;
- function frame teardown;
- runtime teardown;
- exception-path cleanup for all live temporaries and frames.

When a counted array payload reaches zero, destruction releases entry values
before freeing array storage. That recursively releases nested payloads whose
own counts reach zero. Recursive arrays through references will require a
separate reference-cell/GC strategy; value payload COW must not pretend ordinary
payload refcounting alone can collect reference cycles.

## Test Contract

The executable contract tests in `tests/cow_payload_contract.rs` model these
required transitions:

- assignment shares payload identity and increments refcounts;
- writes detach only when `refcount > 1` or the payload is immutable;
- outer array detachment shares nested child payloads until nested mutation;
- slot replacement and destructor paths release old payloads exactly once;
- call argument and temporary ownership retain and release around the call.

Runtime implementation tests should later expose test-only payload identity and
refcount probes under a compile-time flag, then port these same transition
checks against generated native code.
