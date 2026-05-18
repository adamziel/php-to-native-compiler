# Lane 1677-D Review: Append By-Value `ArrayAccess::offsetGet(null)` Reference Sources

This is review/triage only. Do not treat append-source support as complete
unless implementation code, Rust coverage, PHP-comparable fixtures, and a CLI
exercise are all present. Runtime files were intentionally not edited by this
review.

## Current Worktree Baseline

- There is existing uncommitted Lane 1677 WIP in `compiler/src/interpreter.rs`,
  `compiler/tests/functions_and_scopes.rs`, supported docs/progress docs, and
  `tests/fixtures/milestone1677*`. This review did not create or modify those
  files.
- The WIP adds a detached write helper and routes direct, visible
  property-held, and visible dynamic-property-held append reference sources
  such as `$alias =& $bag[]`, `$alias =& $holder->bag[]`, and
  `$alias =& $holder->{$property}[]` through the existing by-value exact
  `ArrayAccess` bridge before the by-reference alias bridge.
- Focused local verification of the current WIP passed:
  `CARGO_TARGET_DIR=/tmp/phpc-target-1677d-review CARGO_BUILD_JOBS=1
  CARGO_INCREMENTAL=0 cargo test -q -p phpc --test functions_and_scopes
  array_access_append_source_by_value -- --test-threads=1` ran 2 tests and
  passed.
- A direct `phpc run` smoke test for `$alias =& $bag[]` with by-value
  `offsetGet($offset) { return $this->items[$offset]; }` produced the expected
  detached target value, left `$bag->items[""]` unchanged, and emitted the
  bounded indirect-modification notice.
- A local target-detachment probe also matched PHP for the important reassignment
  shape where `$target` was first bound to `$items["slot"]`, then reassigned
  from `$target =& $bag[]`; the later write stayed detached from both the old
  array slot and the `ArrayAccess` backing slot.

## Observed PHP 8.2.29 Behavior

Local PHP 8.2.29 shows that append-reference sources on by-value
`ArrayAccess::offsetGet()` use `offsetGet(null)`, not `offsetSet()`:

- `$alias =& $bag[]` calls `offsetGet(null)`.
- If `offsetGet()` returns by value, PHP emits `E_NOTICE` level `8` with
  `Indirect modification of overloaded element of Bag has no effect`.
- The alias receives a detached local value. Later writes to the alias do not
  mutate the backing storage and do not append a new numeric element.
- In the exact bridge body `return $this->items[$offset];`, PHP's `null` array
  key maps to the empty-string key, so a seeded `$this->items[""]` is the value
  copied into the detached alias.
- `$alias =& $holder->bag[]` and `$alias =& $holder->{$property}[]` have the
  same detached notice/no-op behavior for direct visible holder properties.
- If the backing empty-string key is absent, PHP first reports
  `Undefined array key ""`, then reports the indirect-modification notice, and
  the detached alias starts as `null`. Current phpc WIP only reports the
  indirect-modification notice for that missing-key shape, so do not claim
  exact missing-key warning parity.
- If `offsetGet()` returns by reference, `$alias =& $bag[]` aliases the
  empty-string backing slot for the exact bridge. The by-value no-op path must
  continue to run before only by-value methods and must not weaken the existing
  by-reference alias behavior.

## Smallest Safe Runtime Shape

The safe runtime change is the same bounded detached-value path used for
non-append by-value reference sources, with the append key supplied as
`array_access_append_reference_key()`:

- For only the direct object root and direct visible property-held roots being
  claimed, detect an `ArrayAccess` object whose public non-static
  `offsetGet($offset)` returns by value and has the exact
  `return $this->property[$offset];` body.
- Use the existing synthetic append key that represents PHP's
  `offsetGet(null)` call as the empty-string backing array key in this exact
  bridge.
- Emit the bounded indirect-modification `E_NOTICE` through the existing
  notice path so `set_error_handler()` and `error_reporting()` keep working.
- Write the target variable with a detached value, not
  `ArrayOffsetAlias` metadata. If the target name was already an alias, detach
  it before installing the local value.
- Fall through to the existing by-reference alias bridge only when the method
  returns by reference. Keep broader `offsetGet()` bodies rejected instead of
  pretending to evaluate side effects.

Do not broaden this lane to non-direct holder append roots, magic-property
append roots, nested append under an `ArrayAccess` read such as
`$bag["outer"][]`, stored callback argument-array append sources, or native
lowering unless separate probes and tests prove those paths.

## Risks To Watch

- Missing empty-string backing keys have an extra PHP warning before the
  notice. The current bounded bridge can reasonably leave that unnamed, but
  support docs should avoid exact warning-order claims.
- The detached write helper changes the target-variable behavior for all
  by-value `ArrayAccess` reference-source no-op paths in the WIP, not only
  append. This is probably correct for targets that were previously aliases,
  but the local reassignment probe should be committed as a regression so this
  does not drift.
- Dynamic property names must still be evaluated exactly once before the
  by-value/no-op and alias fallbacks.
- By-reference `offsetGet()` append behavior must stay covered by the existing
  alias tests so the by-value no-op route does not steal true reference
  sources.
- Returned arrays containing nested reference elements remain a broader COW
  nuance. This append lane should claim only detached outer value/no backing
  append unless nested-reference-slot parity is explicitly tested.

## Recommended Tests

- Keep the current direct append and direct visible property-held append Rust
  tests.
- Add or keep a PHP-comparable fixture for the same direct,
  property-held, and dynamic-property-held append forms, run through
  `cargo run -q -p phpc -- test --compare-php tests/fixtures/milestone1677`.
- Add a guard for by-reference `offsetGet()` append sources proving
  `$alias =& $bag[]` still writes through to `$bag->items[""]` for the exact
  bridge.
- Add a target-detachment regression where `$target` is first a real array-slot
  reference, then `$target =& $bag[]` with by-value `offsetGet()`, then a write
  to `$target` must not mutate the old array slot or the `ArrayAccess` backing
  slot.
- If missing-key parity is desired, add a separate probe documenting the extra
  PHP `Undefined array key ""` warning before deciding whether to implement it
  in this bounded bridge.
