# PHP Native Compiler Progress

Updated: 2026-05-22 17:52 CEST
Evaluation marker: `20260522T151651Z`
Primary HEAD: `3574e350 codegen: route GLOBALS path appends through ABI`
Current pushed semantic baseline: `3574e350 codegen: route GLOBALS path appends through ABI`

These percentages are candid engineering estimates toward generalized PHP
semantics in the native compiler. They are not test pass rates. Lane-local work
and unstaged primary diffs do not count until reviewed, gated, committed to
`master`, and pushed.

## Executive Read

Overall estimated progress: **80%** `[################----]`

Primary now has one new generalized semantic slice after the marker refresh:
generated-C `$GLOBALS[$expr]...[]... = value` executes through a root-inclusive
symbol-table path append ABI, including suffix-key wrapping and assignment
expression values. This extends the persistent root symbol-table path boundary
from `$GLOBALS` reads/probes/writes/unsets into keyed path appends without
adding a fixture-shaped recognizer.

Recent integrated work is still bounded: request-state paths and dynamic
`$GLOBALS[...]` read/probe/write/unset/append paths now execute through shared
runtime/compiler ABIs. That is real movement toward generalized PHP semantics,
but it is not full global/request/reference behavior. Direct no-key
`$GLOBALS[]`, request aliases, frames, references, COW, exact diagnostics,
object/property semantics, and broad control-flow cleanup remain substantial
open systems.

## Roadmap Position

| Roadmap item | Estimate | Visual | Primary-integrated status |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | 96% | `[###################-]` | Strong shared ABI base; avoid standalone vocabulary without immediate compiler consumers. |
| Compiler/backend consumers | 85% | `[#################---]` | Good for selected request/array/string/`$GLOBALS` read/write/unset/append paths; uneven across calls, objects, control flow, and LLVM/C parity. |
| Executable generalized PHP semantics | 72% | `[##############------]` | Improving, but many real PHP compositions still block. |
| Arrays, lvalues, references, COW | 73% | `[###############-----]` | Arrays/lvalues advanced; full references/COW and arbitrary writable roots remain large. |
| Symbols, globals, request state | 72% | `[##############------]` | Request paths and `$GLOBALS` reads/writes/probes/unsets/appends are stronger; direct root append, aliases, frames, and self-reference remain incomplete. |
| Calls, functions, frames | 25% | `[#####---------------]` | Early; call/frame candidates exist but broad executable semantics are not primary yet. |
| Objects, properties, methods | 11% | `[##------------------]` | Early; runtime candidates exist, but general compiled object/property/method execution remains missing. |
| Diagnostics and control flow | 29% | `[######--------------]` | Useful focused work, but exact diagnostic ordering and structured cleanup are not generalized. |
| Broad integrated verification | 80% | `[################----]` | Focused gates are useful; cross-feature and backend-composition coverage remains thin. |

## Done / In Progress / Not Done

- [x] Runtime/value foundations for selected scalar, string, array, comparison,
  diagnostic, symbol-table, request-state, reference-slot, and native-value
  operations.
- [x] Generated-C consumers for selected scalar/string/array/lvalue behavior,
  including tracked array owner mutations and natural sort families.
- [x] Request-state root, keyed, and nested/path reads, writes, unsets,
  `isset()`, `empty()`, and assignment-expression values through shared request
  ABIs.
- [x] Direct `$GLOBALS` root snapshots and runtime symbol-table nested
  write/read/probe ABIs.
- [x] Compiler-lowered `$GLOBALS[$expr]` and nested `$GLOBALS[...]`
  read/`isset`/`empty` paths through the symbol-table path ABI in generated C.
- [x] Compiler-lowered `$GLOBALS[$expr]` and nested `$GLOBALS[...]` writes
  through a persistent root symbol-table path ABI in generated C.
- [x] Compiler-lowered `$GLOBALS[$expr]` and nested `$GLOBALS[...]` unsets
  through the persistent root symbol-table path ABI in generated C.
- [x] Compiler-lowered `$GLOBALS[$expr]...[]...` appends through the persistent
  root symbol-table path ABI in generated C, including suffix-key wrapping and
  assignment-expression values.
- [ ] Direct no-key `$GLOBALS[]` root append and request-root alias
  reconciliation.
- [ ] Request-root append/write behavior reconciled with `$GLOBALS` and
  request-state storage.
- [ ] Generated PHP reference assignment over proven array/request/symbol
  reference boundaries.
- [ ] Full references/COW, arbitrary writable roots, owner/value/reference
  slots, by-reference args/returns, and by-reference foreach parity.
- [ ] User function/method/closure frames, dynamic calls, variadics/spreads, and
  cleanup ownership across calls.
- [ ] Real object/property/method semantics, `ArrayAccess`, resource offsets,
  and PHP-compatible diagnostics around those features.
- [ ] Structured control-flow cleanup, branch joins, loop/switch transfer, and
  source-ordered warnings/errors at broad scale.

## Recent Primary-Integrated Work

Recent semantic commits on primary:

- `3574e350 codegen: route GLOBALS path appends through ABI`
- `af1511d3 codegen: route GLOBALS path unsets through ABI`
- `6ded95bc codegen: route GLOBALS symbol writes through ABI`
- `633c8713 codegen: route GLOBALS symbol paths through ABI`
- `39586978 runtime: add symbol-table nested read probes`
- `8c13b871 codegen: return request assignment values`
- `f88a624d codegen: route request path reads through state ABI`
- `15657b95 codegen: route request path mutations through state ABI`
- `3bda4f51 codegen: route array mutation builtins through lvalue ABI`
- `d7fc807d codegen: materialize direct $GLOBALS snapshots`
- `764cf014 runtime: add symbol-table nested write ABI`
- `ed2d9031 runtime: add array reference path ABI`

Primary-integrated capability now includes strong request-superglobal path
execution through shared request-state ABIs and generated-C `$GLOBALS[...]`
read/probe/write/unset/append lowering through shared symbol-table path ABIs.
The append slice intentionally leaves direct no-key `$GLOBALS[]`, request-root
alias reconciliation, self-reference behavior, frames, references/COW, and
LLVM/C assembly parity blocked.

## Lane-Local And Active Candidate Work

Lane-local candidates, not counted:

- `impl-native-integration-batch`: generated-C direct value-slot array-lvalue
  owner consumer for pointer/cursor, sort, and mutation families.
- `impl-native-diagnostics`: request-backed `array_key_exists()` / `key_exists()`
  path-presence lowering through request-state path operations.
- `impl-global-symbols`: request-root append ABI, frame-slot nested mutation,
  and `$GLOBALS` request alias candidates. Coordinate this with primary before
  merging.
- `impl-array-value-runtime`: dynamic byte-string query value-frame dispatch for
  `strcasecmp()`, `strpos()`, `substr()`, and `substr_count()`.
- Other fresh lanes continue around calls, control flow, exit/termination,
  objects/properties, references, comparison, type conversion, binary strings,
  diagnostics, and arrays.

## Current Steering

The next integration batches should favor small executable slices:

- Keep semantic progress tied to executable primary commits, not lane-local
  status or management-only dashboard refreshes.
- Build directly on the current global/request work: request/global alias
  reconciliation, direct no-key `$GLOBALS[]`, or request-root append/write
  behavior.
- Keep one source of truth for `$GLOBALS`, request roots, symbol-table roots,
  and aliases before importing dynamic request-root dispatch.
- Consider the fresh value-slot array-lvalue owner consumer or request
  `array_key_exists()` path-presence slice only if they remain narrow and
  primary-compatible.
- Defer broad byte-string, call-frame, object, and control-flow stacks until a
  single executable consumer can be extracted without importing full-lane churn.

Rejected distractions:

- Exact-shape lowering for one fixture or one PHP snippet.
- Standalone blocker/status vocabulary without a near-term consumer.
- Large wholesale lane merges.
- Progress percentage bumps from lane-local work alone.
- Documentation churn beyond the required evaluator marker update.

## Live Notes

Primary currently has one preserved unstaged implementation diff:
`runtime/src/lib.rs` null-slot increment/decrement behavior. It is not counted
as progress and still needs explicit classification, focused tests, and a
separate commit or rejection before any runtime staging.

Resource snapshot: `/dev/shm` has about 15G free of 22G; `/home` has about
198G free. Keep focused gates and avoid broad simultaneous cargo waves.

Evaluator cadence: one candid strategy/progress evaluation every 45 minutes,
feeding advisory steering back to the supervisor. This marker was refreshed
even though percentages did not change, so the completed review is observable.
