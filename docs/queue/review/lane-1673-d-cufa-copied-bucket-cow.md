# Lane 1673-D Review: Stored `call_user_func_array()` Copied-Bucket COW

This is wording prep only. Do not move any of this into supported docs until
implementation code, Rust coverage, PHP-comparable fixtures, and a CLI exercise
prove the behavior.

## Current Baseline

- Milestone 1672 supports copied `ArrayAccess` bucket provenance through
  by-value direct user-function parameters, direct closure parameters,
  string-user-function `call_user_func()` parameters, and positional literal
  `call_user_func_array()` parameters.
- `docs/PROGRESS.md`, `docs/SUPPORT.md`, `docs/ARCHITECTURE.md`,
  `docs/NEXT_TASKS.md`, and `docs/LOOP_MEMORY.md` still name stored
  `call_user_func_array()` argument arrays as the next missing copied-bucket
  propagation target.
- Existing stored `call_user_func_array()` reference-argument support is a
  separate alias/writeback path. The next COW slice should not describe that
  older by-reference path as new support.

## Support Wording After Implementation

Use this shape only after tests prove it:

> Stored `call_user_func_array()` argument arrays now carry the focused
> `ArrayAccess` copied-bucket provenance into by-value callback parameters.
> When a stored positional argument slot contains a bucket copied from one of
> the covered exact-bridge `ArrayAccess` sources, callback-local writes through
> mirrored nested public-property reference slots reach the original referenced
> callback variable and backing bucket, while ordinary copied fields remain
> detached.

Keep the covered source list explicit:

- direct object bucket copies such as `$bucket = $hook[10]`
- direct visible property-held or dynamic-property-held copies such as
  `$bucket = $holder->hook[10]` or `$bucket = $holder->{$name}[10]`
- non-direct holder and expression-root holder copies already covered by
  Milestone 1672, such as `$bucket = $holders["box"]->hook[10]` and
  `$bucket = make_holder($hook)->hook[10]`

Keep the argument-array claim narrow unless broader tests land:

- stored direct argument-array variables such as
  `$args = [$bucket, "label"]; call_user_func_array("mutate", $args)`
- direct visible named object-property stored argument arrays only if the
  implementation and tests cover that path
- string-keyed/named stored arrays only if explicit tests prove preserved
  copied-bucket provenance through name mapping
- public array-callable methods or closure callbacks only if explicit tests
  cover those callback forms; otherwise say string user-function callbacks

## Progress Snippet

Draft for `docs/PROGRESS.md` after code lands:

> Added Milestone 1673 stored `call_user_func_array()` copied-bucket
> propagation for the focused `ArrayAccess` COW shape. Stored positional
> argument arrays can now preserve copied-bucket provenance when the selected
> argument slot contains a bucket copied from the covered exact-bridge
> `ArrayAccess` sources. Writes through mirrored nested public-property
> reference slots inside the callback reach the original referenced callback
> variable and backing bucket, while ordinary copied fields remain detached.
> This does not add broader stored-array expression support, untested
> string-keyed/named argument propagation, side-effecting or broader
> `offsetGet()` bodies, by-value `offsetGet()` reference-source
> indirect-modification notice/no-op fidelity, arbitrary nested reference slots
> stored inside `ArrayAccess` buckets, broader mixed `ArrayAccess` chains,
> general PHP reference containers, broad COW identity, native reference
> lowering, or exact alias destruction/destructor ordering.

## Remaining Blockers To Keep Named

- exact alias detachment after replacing variables or stored argument slots
  that previously mirrored copied-bucket reference slots
- by-value `offsetGet()` used as a reference source: PHP notice/no-op fidelity
  remains a boundary
- non-public backing properties for copied-bucket provenance
- side-effecting, guarded, computed, or broader `offsetGet()` bodies beyond the
  exact `return $this->property[$offset];` bridge
- arbitrary nested reference slots stored inside `ArrayAccess` buckets
- broader mixed `ArrayAccess` chains beyond the documented bounded bridge
- broad PHP reference containers and same-container COW identity
- native reference/COW lowering
- exact alias destruction and destructor ordering

## Suggested Test Proof

- Rust regression for stored direct variable argument arrays with a copied
  direct-object bucket.
- Rust or fixture coverage for one holder path, preferably property-held or
  expression-root, to prove stored propagation reuses Milestone 1672 source
  provenance.
- PHP-comparable fixture exercising `phpc run` with a stored positional
  argument array. The expected output should show the nested referenced
  callback slot writing through and a plain copied field staying detached.
- Add property-stored, string-keyed, array-callable, or closure callback tests
  only if those shapes are documented as supported.
