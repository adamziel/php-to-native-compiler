# Lane 1674-C Review: Helper-Parameter Replacement Detachment COW

This is wording prep only. Do not move any of this into supported docs until
implementation code, Rust coverage, PHP-comparable fixtures, and a CLI exercise
prove the behavior.

## Current Baseline

- Milestone 1672/1673 support the focused `ArrayAccess` copied-bucket
  provenance through by-value direct user-function parameters, direct closure
  parameters, string-user-function `call_user_func()` parameters, positional
  literal `call_user_func_array()` parameters, and direct stored positional
  `call_user_func_array()` argument-array variables.
- Lane 1673-C probes classify the remaining replacement nuance. Replacing the
  outer copied `$bucket` variable already detaches from the original callback
  slot, and lingering by-reference foreach callback-variable reuse matches the
  observed PHP control.
- The remaining mismatch is narrower: when a by-value helper parameter has
  imported copied-bucket provenance and the helper later assigns a replacement
  array to that same parameter name, current phpc keeps the old mirrored alias
  attached. The replacement array's nested write can still reach the original
  callback slot, while PHP leaves the original callback slot at the earlier
  write and keeps the replacement local to the helper.
- `docs/SUPPORT.md` currently names this as unsupported:
  exact alias detachment when a by-value helper parameter holding copied
  bucket provenance is replaced.

## Missing Wording To Keep Until Fixed

Use this boundary in support/progress docs while the probe remains phpc-only:

> Exact alias detachment when a by-value helper parameter holding copied
> `ArrayAccess` bucket provenance is replaced remains unsupported. Current
> phpc may keep the mirrored nested reference-slot provenance attached to the
> helper parameter name after whole-parameter reassignment, so a later nested
> write through the replacement array can still update the original callback
> slot where PHP keeps that replacement local.

Keep the scope narrow:

- This is not a rollback of copied-bucket propagation through supported
  by-value parameters or stored positional `call_user_func_array()` arrays.
- This is specifically whole-parameter replacement after the parameter already
  carried copied-bucket provenance.
- Do not generalize the wording to all copied bucket variable replacement;
  the existing comparable controls cover direct copied `$bucket` replacement
  and lingering foreach callback-variable reuse separately.

## Support Wording After Implementation

Use this shape only after tests prove it:

> By-value helper parameters that import the focused `ArrayAccess`
> copied-bucket provenance now detach that mirrored provenance when the
> parameter variable is replaced as a whole. Writes through the copied
> bucket's mirrored nested public-property reference slots still reach the
> original callback slot before replacement, but later nested writes through a
> replacement array assigned to the helper parameter remain local to that
> replacement, matching PHP.

Keep the covered source list aligned with the already-supported provenance
sources:

- direct object bucket copies such as `$bucket = $hook[10]`
- direct visible property-held or dynamic-property-held copies such as
  `$bucket = $holder->hook[10]` or `$bucket = $holder->{$name}[10]`
- non-direct holder and expression-root holder copies such as
  `$bucket = $holders["box"]->hook[10]` and
  `$bucket = make_holder($hook)->hook[10]`
- direct stored positional `call_user_func_array()` argument-array variables
  only if the fix proves detachment after callback/helper parameter replacement
  on that path too

## Progress Snippet After Implementation

Draft for `docs/PROGRESS.md` after code lands:

> Fixed the helper-parameter replacement detachment gap for the focused
> `ArrayAccess` copied-bucket COW shape. A by-value helper parameter can still
> receive covered copied-bucket provenance and write through mirrored nested
> public-property reference slots before replacement, but assigning a new array
> to that parameter name now detaches the old mirrored provenance before later
> nested writes. This matches PHP's behavior where the original callback slot
> remains at the pre-replacement write while the replacement array stays local
> to the helper. This does not add broader stored-array expression support,
> untested string-keyed/named argument propagation, side-effecting or broader
> `offsetGet()` bodies, by-value `offsetGet()` reference-source
> indirect-modification notice/no-op fidelity, arbitrary nested reference slots
> stored inside `ArrayAccess` buckets, broader mixed `ArrayAccess` chains,
> general PHP reference containers, broad COW identity, native reference
> lowering, or exact alias destruction/destructor ordering.

## Suggested Test Proof

- Convert the current phpc-only helper-parameter replacement probe into a
  PHP-comparable fixture once behavior matches system PHP.
- Add a Rust regression for direct by-value helper parameters that replace the
  parameter after one mirrored nested reference-slot write.
- Add coverage for the stored positional `call_user_func_array()` path if the
  public docs claim that path also detaches on callback/helper parameter
  replacement.
- Exercise `phpc run --compare-php` over the fixture so the CLI path proves
  both the pre-replacement write-through and post-replacement detachment.

## Remaining Blockers To Keep Named

- by-value `offsetGet()` used as a reference source: PHP notice/no-op fidelity
  remains a boundary
- non-public backing properties for copied-bucket provenance
- side-effecting, guarded, computed, or broader `offsetGet()` bodies beyond
  the exact `return $this->property[$offset];` bridge
- arbitrary nested reference slots stored inside `ArrayAccess` buckets
- broader stored-array expression support and untested named-argument
  propagation
- broader mixed `ArrayAccess` chains beyond the documented bounded bridge
- broad PHP reference containers and same-container COW identity
- native reference/COW lowering
- exact alias destruction and destructor ordering
