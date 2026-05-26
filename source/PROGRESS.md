# PHP Native Compiler Progress

Updated: 2026-05-26 03:38 CEST
Evaluation marker: `20260526T012306Z`

Accounting rule: only generalized, tested, committed, and pushed primary work
counts as integrated capability. Dirty WIP, candidate worktrees, lane-local
claims, review-only work, failed prep proofs, probe-only commits, architecture
notes, and dashboard-only commits are excluded.

## Executive Read

Overall estimated progress: **95%** `[###################-]`

Executable PHP semantics: **95%** `[###################-]`

Primary is clean and aligned with `origin/master` at
`6a73b186 native: route dynamic callables through callable-value ABI`.

Latest primary-integrated source capability baseline:
`6a73b186 native: route dynamic callables through callable-value ABI`.

Current momentum is still positive, but the recent primary source progress is
still bounded. The latest source packet adds real executable generated-C
dynamic callable progress by routing dynamic callee expression lowering through
the shared runtime callable-value ABI and runtime builtin dispatch. Full PHP
callable semantics remain incomplete: object/method callable parity, callable
array validation parity, by-reference/default/variadic edge cases,
source-ordered diagnostics, namespace fallback, autoload, magic calls,
constructors, closure frame handoff, and backend parity remain open. Overall
and executable percentages remain flat.

## Grand Roadmap Position

| Workstream | Estimate | Bar | Current read |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | **99%** | `[####################]` | Strong selected-path value, byte-string, array, reference, symbol, callable table, callable-value dispatch, call-frame/result, request-state, diagnostic operation/operand-list, reference-binding operand-list, assignment-lvalue operand-list, and object allocation-risk metadata surfaces. Remaining gaps include broader callable lookup parity, namespace fallback, autoload, magic calls, constructors, closure frame handoff, and cleanup/unwind parity. |
| Compiler/backend consumers | **99%** | `[####################]` | Generated C has the freshest executable consumers. LLVM and direct assembly lag recent semantics. Direct generated-C user-function calls consume the runtime callable ABI and direct root/request frame handoff; generated-C dynamic callable expressions now consume the callable-value lookup/invocation ABI. |
| Executable PHP semantics | **95%** | `[###################-]` | Many selected executable islands exist, but major PHP semantics remain open: full assignment/RMW/writeback, references/COW, dynamic callables, objects/properties/ArrayAccess, cleanup/unwind/finally/destructors, exact diagnostics, and backend parity. |
| Strings and byte semantics | **62%** | `[############--------]` | Byte-backed values and byte-preserving `explode()` / `str_split()` slots are integrated. Binary source bytes, byte-exact interpreter/session/debug output, `mb_str_split()`, request/global byte keys, and exact diagnostics remain open. |
| Arrays, lvalues, references, COW | **82%** | `[################----]` | Selected reference-source/lvalue extraction, closure capture from reference-backed slots, reference-binding operand diagnostics, and assignment-lvalue operand diagnostics are integrated. Executable assignment, RMW, broad writeback, arbitrary alias roots, foreach, static/magic/non-public properties, ArrayAccess, and full COW remain incomplete. |
| Symbols, globals, request state | **75%** | `[###############-----]` | Selected globals, root-symbol consumers, active symbol-table consumers, request-key blockers, append-shaped symbol reference-source materialization, direct generated-C request-state frame handoff, and generated-C dynamic user-function handoff proof exist. `$GLOBALS` self-cells, closure request-state handoff, request/global alias parity, request writeback, includes, variable variables, and exact unset/global behavior remain incomplete. |
| Calls, functions, frames | **92%** | `[##################--]` | Runtime callable table/value dispatch, call arguments/frame/result ABI, direct generated-C user-function consumers, generated-C dynamic callable-value consumers, by-reference argument transport, descriptor closures, closure returns, and direct/dynamic generated-C request-state frame handoff are integrated. Object/method callable parity, callable array validation parity, `Class::method` strings, namespace fallback, autoload, magic calls, named/spread breadth, return references, constructors, closure frame-environment handoff, and cleanup/unwind parity remain open. |
| Objects, properties, methods | **53%** | `[###########---------]` | Public object-property reference-source extraction, object-property reference-slot mutation, and declared-class allocation cleanup-risk metadata exist for selected paths. Full visibility, magic, dynamic/static/typed properties, destructor execution, interfaces/traits execution, references/COW, constructors, and `ArrayAccess` execution remain open. |
| Control flow, cleanup, diagnostics | **51%** | `[##########----------]` | Selected branches, loops, transfers, finalizers, output buffers, diagnostics, try-body call-boundary preflight, generic operand-list blockers, reference-binding operand blockers, and assignment-lvalue operand blockers exist. Broad unwind/finally/destructor/shutdown execution, cleanup ownership, executable writeback/reference binding, and source-ordered diagnostics remain open. |
| Broad integrated verification | **92%** | `[##################--]` | Recent focused gates are strong and nonzero. The full `native_runtime_abi` baseline still has known current-primary failures, and broad verification remains constrained by lane extraction cost, stale candidate expectations, swap pressure, and backend parity gaps. |

## Recent Primary-Integrated Work

- `6a73b186`: routed generated-C dynamic callee expression lowering through
  the shared runtime callable-value ABI and runtime builtin dispatch. Dynamic
  callable lookup now uses
  `phpc_native_callable_lookup_value_or_closure_with_context_diagnostic`, and
  invocation uses
  `phpc_native_callable_value_invoke_value_with_diagnostic_and_free`, including
  dynamic string callable generated-C proof and request/global dynamic
  user-function handoff proof. The remaining compiler-side callable builtin
  ladder is dead cleanup debt, not a path to extend.
- `81c15cd5`: added assignment-lvalue operand-list requirement diagnostics
  over the shared runtime/compiler operation-list ABI. Assignment targets now
  report receiver, dynamic-property, and key-expression evaluation
  requirements through `AssignTarget` semantic families. This is a diagnostic
  blocker boundary only; executable assignment and writeback remain open.
- `b27bbb20`: replaced root-symbol-only generated user-function frame metadata
  with `CFrameEnvironmentRequirement { root_symbols, request_state }` and
  propagated those requirements through direct generated-C user-function
  callable/frame handoff. Dynamic callable and closure frame-environment
  handoff remain open.
- `b4b21937`: extended the shared operation-list diagnostic ABI to
  reference-binding target/source and by-reference array item requirements.
  Executable reference binding, alias/COW identity, writeback, and cleanup
  ordering remain open.
- `a544daa8`: introduced the generic diagnostic operation/operand-list blocker
  boundary for call-argument and lvalue diagnostic families.
- `dae1b44c` and `5abf8525`: integrated and repaired runtime callable-value
  dispatch over the callable table and call arguments/frame/result ABI.
- `f0cc17c1`: routed direct generated-C user-function calls through the
  runtime callable table, lookup, arguments, frame, and result ABI.

## Active Roadmap Items

Primary-integrated capability and lane-local candidate work are separated
explicitly.

| Item | Toward Primary Integration | Toward Full Feature | Status |
| --- | ---: | ---: | --- |
| Assignment-lvalue operand-list diagnostics | **100%** `[####################]` | **15%** `[###-----------------]` | Integrated at `81c15cd5`. Generalized blocker vocabulary exists; assignment/RMW execution and writeback do not. |
| Direct request-state frame handoff for generated-C user functions | **100%** `[####################]` | **40%** `[########------------]` | Integrated at `b27bbb20`; extended by `6a73b186` for generated-C dynamic user-function handoff proof. Closure handoff remains open. |
| Reference-binding operand-list diagnostics | **100%** `[####################]` | **15%** `[###-----------------]` | Integrated at `b4b21937`. Diagnostic ABI progress only; executable reference binding remains open. |
| Runtime callable ABI, callable-value dispatch, and direct/dynamic generated-C consumers | **100%** `[####################]` | **66%** `[#############-------]` | Integrated through `6a73b186`. Direct and dynamic generated-C callable paths consume the shared runtime lookup/invocation ABI and runtime builtin dispatch. Constructors, object/method callable parity, callable array validation parity, broader lookup parity, return references, named/spread breadth, and cleanup/unwind remain open. |
| Dynamic callable compiler consumer plus builtin string callable repair | **100%** `[####################]` | **60%** `[############--------]` | Integrated at `6a73b186`. Dynamic callee expression lowering routes through the shared callable-value lookup/invocation ABI instead of a compiler-side finite builtin/name ladder. Full callable semantics remain open. |
| RMW-lvalue operand-list diagnostics | **0%** `[--------------------]` | **14%** `[###-----------------]` | Lane-local. Must rebase and renumber after `81c15cd5` because assignment claimed operation tag `7` and operand tags `18..20`. |
| Object/ArrayAccess write-side blockers | **0%** `[--------------------]` | **24%** `[#####---------------]` | Lane-local. Useful classifier evidence, but no executable `ArrayAccess` write/unset behavior yet. |
| Cleanup/unwind requirement boundary | **0%** `[--------------------]` | **18%** `[####----------------]` | Lane-local. Diagnostic/blocker candidate only; no finally/destructor/unwind execution. |
| Trait effective-method metadata | **0%** `[--------------------]` | **20%** `[####----------------]` | Lane-local/stale relative to current primary. Needs refresh before any integration; trait method execution remains open. |
| Broad lane extraction backlog | **35%** `[#######-------------]` | **36%** `[#######-------------]` | Broad dirty lanes remain useful evidence repositories, not integration units. |

## Done / In Progress / Not Done

Primary-integrated capability:

- [x] Runtime callable table plus call arguments/frame/result ABI.
- [x] Runtime callable-value dispatch for string/binary function names,
  callable arrays, descriptor closures, inherited methods, bound receivers,
  and object `__invoke`.
- [x] Direct generated-C user-function calls through runtime callable lookup,
  arguments, frame wrappers, and value results.
- [x] Direct generated-C user-function root/request frame-environment handoff.
- [x] Generated-C dynamic callee expressions through
  `phpc_native_callable_lookup_value_or_closure_with_context_diagnostic` and
  `phpc_native_callable_value_invoke_value_with_diagnostic_and_free`.
- [x] Dynamic string callable generated-C proof and request/global dynamic
  user-function handoff proof through the shared callable-value ABI.
- [x] Shared diagnostic operation/operand-list blocker boundary.
- [x] Reference-binding operand-list requirement blockers.
- [x] Assignment-lvalue operand-list requirement blockers.
- [x] Declared-class allocation cleanup-risk metadata.
- [x] Try/catch/finally body call-boundary preflight diagnostics.
- [x] Selected reference-source/lvalue extraction and reference-backed closure
  capture materialization.
- [x] Closure value/reference return ABI for descriptor closures.
- [x] Byte-backed PHP string values and byte-preserving selected string-array
  slots.

In progress but lane-local:

- [ ] RMW-lvalue diagnostic packet after assignment tag reconciliation.
- [ ] Object/ArrayAccess write-side blocker boundary.
- [ ] Cleanup/unwind requirement boundary.
- [ ] Trait effective-method metadata refresh.
- [ ] Broad dirty call, diagnostic, object, symbol, byte/string, and array lanes
  as evidence pools only.

Not done:

- [ ] Full executable assignment and RMW semantics, including expression
  results, writeback, ordered key coercion, and property/array storage.
- [ ] Full PHP reference binding, references/COW identity, arbitrary alias
  roots, and alias-preserving write-through.
- [ ] Full callable parity beyond the integrated generated-C dynamic callable
  consumer: object/method callable parity, callable array validation parity,
  `Class::method` strings, namespace fallback, autoload, magic calls,
  named/spread arguments, by-reference/default/variadic edge cases, return
  references, constructors, source-ordered diagnostics, closure frame handoff,
  and backend parity.
- [ ] Request storage/writeback, `$GLOBALS` self-cells, request/global alias
  parity, request foreach, and mutation-during-iteration behavior.
- [ ] Includes, variable variables, and dynamic symbol behavior.
- [ ] Runtime `ArrayAccess` execution for `offsetGet`, `offsetExists`,
  `offsetSet`, and `offsetUnset`.
- [ ] General object model: non-public methods, overrides, interfaces/traits
  execution, magic methods, dynamic/static/typed properties, destructors, and
  object lifetime cleanup.
- [ ] Complete cleanup/unwind/finally/destructor/output-buffer shutdown
  behavior.
- [ ] Exact/source-ordered diagnostics, custom handler execution,
  warning/error continuation, and suppression parity.
- [ ] LLVM/direct assembly parity for recent generated-C semantics.
- [ ] Known current-primary full `native_runtime_abi` baseline failures.

## Current Work Snapshot

Primary-integrated:

- [x] Primary is clean and synced at `6a73b186`.
- [x] Latest source capability is `6a73b186`.
- [x] `6a73b186` is accounted as executable generated-C dynamic callable
  progress through the shared runtime callable-value ABI.
- [x] Percentages remain flat for this review.

Lane-local:

- [ ] RMW-lvalue work must rebase and renumber after assignment integration.
- [ ] Object/ArrayAccess, cleanup/unwind, trait metadata, and symbol/global
  proof candidates need fresh current-primary routing before integration.

Resource posture:

- `/dev/shm`: 40G total, 24G used, 17G available, 58% used.
- `/home`: 459G total, 261G used, 180G available, 60% used.
- Memory available is about 39Gi, but swap remains high at 23Gi/29Gi used.
- Largest `/dev/shm` targets: `phpc-target-native-call-semantics` 8.9G,
  `phpc-target-native-object-seed` 5.6G, and
  `phpc-target-native-diagnostics` 3.0G.
- Continue disk-backed `/tmp` target dirs, `umask 0007`,
  `CARGO_BUILD_JOBS=1`, `CARGO_INCREMENTAL=0`, and focused nonzero gates.

## Non-Repeat Guard

Future dynamic-callable work must not repeat the same callable-value
lookup/invocation ABI consumption, dynamic string callable generated-C proof,
request/global dynamic user-function handoff proof, or exact focused gate set
as standalone progress. Distinct next progress should remove the remaining
dead compiler callable ladder or broaden a new callable semantic family with
focused proof.

## Next Steering Read

Best next action:

- Remove the remaining dead compiler callable ladder helpers, or target a
  distinct callable semantic family such as object/method callable parity,
  callable array validation parity, namespace fallback, autoload, magic calls,
  constructors, closure frame handoff, or backend parity.

Avoid:

- Counting lane-local candidate work before primary integration.
- More diagnostic/metadata-only packets unless they remove a concrete
  exact-shape gate and feed an imminent executable consumer.
- Integrating RMW-lvalue unchanged after assignment; reconcile diagnostic tags
  first.
- Directly importing broad dirty lanes.
- Repeating the `6a73b186` callable-value ABI consumption and focused proof set
  as standalone progress.
- Reintroducing or extending compiler-side finite dynamic-call or builtin-name
  ladders instead of consuming shared runtime callable-value semantics.
