# Lane 1675-D Review: By-Value `ArrayAccess::offsetGet()` Reference Source

This is review/triage only. Do not move any of this into supported docs until
implementation code, Rust coverage, PHP-comparable fixtures, and a CLI exercise
prove the behavior.

## Current Baseline

- `phpc run` supports direct and property-held `ArrayAccess` reference sources
  only when public `offsetGet($offset)` returns by reference and has the exact
  `return $this->property[$offset];` body.
- Direct by-value or by-reference `offsetGet()` bucket-copy reads are a
  separate copied-array provenance path. Those reads intentionally do not make
  by-value `offsetGet()` a reference source.
- The current by-value reference-source fixture
  `tests/fixtures/milestone1092/reference_assignment_array_access_source_boundary.php`
  is phpc-only and reports a structured runtime boundary before execution.
- `docs/ARCHITECTURE.md` already names the intended PHP behavior: by-value
  `offsetGet()` used as a reference source emits an indirect-modification
  notice, creates only a detached local alias/value, and leaves the backing
  element unchanged.

## Observed PHP 8.2.29 Behavior

Using local CLI PHP 8.2.29:

- `$alias =& $bag["name"];` calls by-value `offsetGet("name")`.
- PHP emits `E_NOTICE` level `8` with message
  `Indirect modification of overloaded element of Bag has no effect`.
- The process exits `0`; this is recoverable, not a fatal error.
- Later `$alias = "changed";` does not mutate `$bag->items["name"]`.
- Nested `$alias =& $bag["outer"]["slot"];` has the same no-op backing
  behavior after the single `offsetGet("outer")` read.
- `error_reporting(0)` suppresses stderr output.
- `set_error_handler()` receives the notice before stderr fallback; returning
  `true` suppresses stderr.

Only PHP 8.2.29 is installed in this environment, so exact stderr text should
not be treated as stable across all PHP versions. Older/newer PHP versions may
format the prefix, file label, or notice/fatal classification differently.

## Smallest Safe Runtime Shape

Keep the existing by-reference exact-bridge path unchanged. For the smallest
safe by-value `offsetGet()` reference-source slice, stay inside the same exact
public `return $this->property[$offset];` bridge already used by copied-bucket
provenance:

- read the selected backing slot once through the existing exact-bridge
  property/key analysis;
- emit a bounded `E_NOTICE` through `emit_notice()` with the PHP-observed
  indirect-modification message;
- write the read value into the target variable as a plain detached value;
- do not bind the target to `ArrayOffsetAlias` metadata and do not mirror
  copied-bucket provenance from this reference-assignment path;
- leave by-reference `offsetGet()` methods outside the exact bridge rejected,
  because a detached no-op would be wrong when PHP expects an actual returned
  reference;
- leave side-effecting, guarded, computed, or broader by-value `offsetGet()`
  bodies rejected until a later lane intentionally proves normal method-call
  evaluation plus detached-result behavior.

The natural implementation seam is around
`evaluate_array_access_reference_source_alias_for_object()` and its direct /
property-held callers. That function currently returns only an alias/value pair
or errors; adding a small internal outcome enum such as `Alias(...)` versus
`DetachedValue(...)` would avoid overloading `Option` and would let
`execute_reference_assignment()` bind only the alias case.

## Suggested Test Proof

- Convert or supplement the existing Milestone 1092 phpc-only boundary with a
  PHP-comparable fixture that installs `set_error_handler()`, captures the
  notice level/message in stdout, assigns through the detached target, and
  proves the backing element remains unchanged with empty stderr.
- Add a Rust regression for direct `$alias =& $bag["name"]` with by-value
  `offsetGet()`: target receives the value, target writes stay local, backing
  storage is unchanged, exit code is `0`, and stderr contains the bounded
  notice when no handler is installed.
- Add a Rust or fixture regression for nested `$alias =& $bag["outer"]["slot"]`
  to prove the normal read happens once and later target writes remain
  detached.
- Add property-held coverage only if the implementation claims
  `$alias =& $holder->bag["name"]` parity in the same lane.
- Keep by-reference non-exact `offsetGet()` as a rejection regression.

## Stderr And Version Risks

- Avoid byte-for-byte system-PHP stderr comparison for this lane unless the
  runner normalizes PHP versions and notice prefixes. Prefer an error-handler
  fixture for PHP comparison and a phpc-only stderr snapshot for the bounded
  fallback text.
- Current phpc diagnostic formatting includes a function/context label in
  runtime notices. Exact PHP does not include a function name in the observed
  notice text for this case, so the fallback stderr string may intentionally
  differ even when level and no-op behavior are correct.
- Notice routing must respect the existing `error_reporting()` mask and
  `set_error_handler()` stack. Calling `emit_notice()` after the `offsetGet()`
  read should reuse that behavior.
- The fix should not broaden copied-bucket COW support. Reference assignment
  from by-value `offsetGet()` should stay detached even when the returned
  array contains reference slots.

## Remaining Boundaries To Keep Named

- broad PHP reference containers and same-container COW identity
- by-reference `offsetGet()` bodies outside the exact
  `return $this->property[$offset];` bridge
- exact stderr byte-for-byte parity across PHP versions
- arbitrary mixed `ArrayAccess` chains beyond documented covered paths
- native reference/COW lowering
- exact alias destruction and destructor ordering
