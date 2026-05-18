# Lane 1675-C Review: By-Value `offsetGet()` Reference-Source Notice/No-op

This is wording prep only. Do not move any completion language into supported
docs until implementation code, Rust coverage, PHP-comparable fixtures, and a
CLI exercise prove the behavior.

## Current Baseline

- The current exact `ArrayAccess` bridge supports reference sources only when
  public `offsetGet($offset)` returns by reference and has the bounded
  `return $this->property[$offset];` body. It covers direct object roots,
  visible property-held roots, dynamic property-held roots, selected
  method-context non-public holder properties, and the documented nested child
  suffixes.
- Direct bucket-copy reads through the same exact body can preserve copied
  nested public-property reference-slot provenance for both by-value and
  by-reference `offsetGet()` declarations. That is copy provenance, not a
  reference-source alias for by-value `offsetGet()`.
- System PHP behavior for a reference source such as
  `$alias =& $bag[$key]` or `$alias =& $bag["outer"]["slot"]` when
  `offsetGet()` returns by value is narrower than a real alias: PHP emits an
  indirect-modification `E_NOTICE`, gives the alias a detached local value, and
  leaves the backing `ArrayAccess` storage unchanged when the alias or its
  children are later mutated.
- `docs/SUPPORT.md` and `docs/ARCHITECTURE.md` currently keep that shape as a
  hard runtime boundary for phpc until notice/no-op fidelity exists.

## Support Wording After Implementation

Use this shape only after tests prove it:

> By-value `ArrayAccess::offsetGet()` reference sources now model PHP's
> indirect-modification notice/no-op path for the bounded exact bridge. For
> direct object roots, visible property-held roots, dynamic property-held
> roots, and the covered nested child suffixes where public
> `offsetGet($offset)` returns by value with exactly
> `return $this->property[$offset];`, statement-form reference assignment such
> as `$alias =& $bag[$key]` snapshots the returned value into a detached local
> alias, emits the bounded indirect-modification `E_NOTICE`, and does not bind
> the alias to the backing property array slot. Later writes through the alias
> or through children of the alias remain local; the backing `ArrayAccess`
> storage keeps its original value.

Keep the scope narrow:

- This does not change by-reference `offsetGet()` reference-source support.
- This does not turn by-value `offsetGet()` into an aliasing source; it is a
  PHP-compatible notice/no-op plus detached local alias/value.
- This does not broaden copied-bucket provenance through ordinary by-value
  reads or callback/helper parameters.
- This does not add reference assignment to `ArrayAccess` offset targets such
  as `$bag[$key] =& $value`; those remain the separate PHP-parity target
  boundary.

## Architecture Wording After Implementation

Draft replacement for the current boundary paragraph in `docs/ARCHITECTURE.md`:

> By-value `offsetGet()` used as a reference source follows PHP's bounded
> indirect-modification path for the same exact bridge instead of creating a
> backing alias. The source expression is evaluated through the covered
> direct/property-held/dynamic-property-held `ArrayAccess` root, the returned
> value is placed in a detached local alias cell, and the runtime emits the
> bounded indirect-modification `E_NOTICE`. Writes through that detached alias
> do not update the backing property array slot, and later backing-slot writes
> do not reattach the alias. This remains separate from copied-bucket
> provenance, where by-value reads may mirror nested reference slots into a
> copied array variable without making the outer read a reference source.

## Progress Snippet After Implementation

Draft for `docs/PROGRESS.md` after code lands:

> Added Lane 1675-C notice/no-op fidelity for by-value
> `ArrayAccess::offsetGet()` reference sources in the focused exact-bridge
> slice. Statement-form reference assignment from a by-value exact-bridge
> source such as `$alias =& $bag[$key]` or a covered nested child source now
> emits the bounded indirect-modification `E_NOTICE`, initializes the alias
> from the returned value as a detached local cell, and leaves the backing
> `ArrayAccess` storage unchanged when the alias is later written. This does
> not add by-value `offsetGet()` aliasing, reference assignment to
> `ArrayAccess` offset targets, side-effecting or broader `offsetGet()`
> bodies, non-public copied-bucket provenance, arbitrary nested reference slots
> stored inside `ArrayAccess` buckets, broader mixed `ArrayAccess` chains,
> general PHP reference containers, broad COW identity, native reference
> lowering, or exact alias destruction/destructor ordering.

## Suggested Test Proof

- PHP-comparable fixture for direct `$alias =& $bag[$key]` with by-value
  exact-bridge `offsetGet()`: the expected output should show the detached
  alias changed and the backing element unchanged, with the bounded notice
  emitted.
- PHP-comparable fixture for nested `$alias =& $bag["outer"]["slot"]` proving
  the detached child alias can change without mutating the backing nested
  slot.
- Rust regression for at least one property-held or dynamic property-held
  root if the support docs claim those roots in the completed slice.
- CLI exercise through `cargo run -q -p phpc -- test --compare-php` over the
  fixture directory so the notice/no-op behavior is proven against system PHP.

## Remaining Blockers To Keep Named

- side-effecting, guarded, computed, or broader `offsetGet()` bodies beyond
  the exact `return $this->property[$offset];` bridge
- append-source by-value `offsetGet(null)` reference-source behavior unless
  explicitly implemented and tested
- non-public copied-bucket provenance and any non-public backing-property
  paths not covered by the completed by-value notice/no-op slice
- arbitrary nested reference slots stored inside `ArrayAccess` buckets
- broader stored-array expression support and untested named-argument
  propagation
- broader mixed `ArrayAccess` chains beyond the documented bounded bridge
- real PHP reference containers and same-container COW identity
- native reference/COW lowering
- exact alias destruction and destructor ordering
