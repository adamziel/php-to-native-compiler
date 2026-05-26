# PHP Native Compiler Progress

Updated: 2026-05-26 04:34 CEST
Evaluation marker: `20260526T023213Z`

Accounting rule: only generalized, tested, committed, and pushed primary work
counts as integrated capability. Dirty WIP, candidate worktrees, lane-local
claims, review-only work, failed prep proofs, probe-only commits, architecture
notes, and dashboard-only commits are excluded.

## Executive Read

Overall supervised-roadmap progress: **95%** `[###################-]`

Selected executable PHP semantics: **95%** `[###################-]`

Primary was clean and synced with `origin/master` at review time:
`90a9204a docs: account RMW lvalue diagnostics`.

Latest primary-integrated source capability baseline:
`b918d3b1 native: add RMW lvalue operand-list diagnostics`.

Recent primary progress is real, but it is still selected-path infrastructure
and diagnostics rather than full PHP parity. `6a73b186` routed generated-C
dynamic callees through the shared runtime callable-value ABI. `513dbf21`
added generalized Object/ArrayAccess write-operation blockers. `b918d3b1`
added a generalized RMW-lvalue operand-list diagnostic boundary for compound
assignment, null-coalesce assignment, and increment/decrement targets.

The next useful progress should execute missing behavior, not repeat blocker
classification. The best live candidate signals are lane-local Object/ArrayAccess
runtime write dispatch after ownership repair, and an RMW executable writeback
plan for array-lvalue owners. Neither is primary-integrated yet.

Full PHP callable, object, lvalue, reference/COW, cleanup, diagnostic, and
backend parity remain incomplete. Percentages remain flat for this review.

## Grand Roadmap Position

| Workstream | Estimate | Bar | Current read |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | **99%** | `[####################]` | Strong selected-path value, byte-string, array, reference, symbol, callable table, callable-value dispatch, call-frame/result, request-state, diagnostic operation/operand-list, reference-binding, assignment-lvalue, RMW-lvalue, and object allocation-risk surfaces. Remaining gaps include broader callable lookup parity, namespace fallback, autoload, magic calls, constructors, closure frame handoff, and cleanup/unwind parity. |
| Compiler/backend consumers | **99%** | `[####################]` | Generated C has the freshest executable consumers. Direct and dynamic generated-C callable paths consume shared runtime callable ABI surfaces. LLVM and direct assembly still lag recent semantics. |
| Executable PHP semantics | **95%** | `[###################-]` | Many selected executable islands exist, but major PHP semantics remain open: full assignment/RMW/writeback, references/COW, executable object/ArrayAccess writes, cleanup/unwind/finally/destructors, exact diagnostics, and backend parity. |
| Strings and byte semantics | **62%** | `[############--------]` | Byte-backed values and byte-preserving selected string-array slots are integrated. Binary source bytes, byte-exact interpreter/session/debug output, `mb_str_split()`, request/global byte keys, and exact diagnostics remain open. |
| Arrays, lvalues, references, COW | **82%** | `[################----]` | Selected reference-source/lvalue extraction, closure capture from reference-backed slots, reference-binding operand diagnostics, assignment-lvalue operand diagnostics, RMW-lvalue operand diagnostics, and Object/ArrayAccess write blockers are integrated. Executable assignment, RMW, broad writeback, arbitrary alias roots, foreach, static/magic/non-public properties, ArrayAccess execution, and full COW remain incomplete. |
| Symbols, globals, request state | **75%** | `[###############-----]` | Selected globals, root-symbol consumers, active symbol-table consumers, request-key blockers, append-shaped symbol reference-source materialization, direct generated-C request-state frame handoff, and generated-C dynamic user-function handoff proof exist. `$GLOBALS` self-cells, closure request-state handoff, request/global alias parity, request writeback, includes, variable variables, and exact unset/global behavior remain incomplete. |
| Calls, functions, frames | **92%** | `[##################--]` | Runtime callable table/value dispatch, call arguments/frame/result ABI, direct generated-C user-function consumers, generated-C dynamic callable-value consumers, by-reference argument transport, descriptor closures, closure returns, and direct/dynamic generated-C request-state frame handoff are integrated. Object/method callable parity, callable array validation parity, `Class::method` strings, namespace fallback, autoload, magic calls, named/spread breadth, return references, constructors, closure frame-environment handoff, and cleanup/unwind parity remain open. |
| Objects, properties, methods | **54%** | `[###########---------]` | Public object-property reference-source extraction, object-property reference-slot mutation, declared-class allocation cleanup-risk metadata, and Object/ArrayAccess write-operation blocker classification exist for selected paths. Actual Object/ArrayAccess write dispatch is lane-local only. Full visibility, magic, dynamic/static/typed properties, destructor execution, interfaces/traits execution, references/COW, constructors, and compiler-consumed `ArrayAccess` execution remain open. |
| Control flow, cleanup, diagnostics | **52%** | `[##########----------]` | Selected branches, loops, transfers, finalizers, output buffers, diagnostics, try-body call-boundary preflight, generic operand-list blockers, reference-binding blockers, assignment-lvalue blockers, RMW-lvalue blockers, and Object/ArrayAccess write blockers exist. Broad unwind/finally/destructor/shutdown execution, cleanup ownership, executable writeback/reference binding, and source-ordered diagnostics remain open. |
| Broad integrated verification | **92%** | `[##################--]` | Recent focused gates are strong and nonzero. The full `native_runtime_abi` baseline still has known current-primary failures, and broad verification remains constrained by lane extraction cost, stale candidate expectations, swap pressure, and backend parity gaps. |

## Recent Primary-Integrated Work

- `b918d3b1`: generalized RMW-lvalue operand-list diagnostics over compound
  assignment, null-coalesce assignment, and increment/decrement target
  operands. This adds operation tag `8`, operand tag `21`, shared
  `AssignTarget` operand enumeration, and
  `NativeDiagnosticOperandRequirement` operation-list diagnostics. It does not
  execute RMW owner/writeback, references/COW, object/static property storage,
  ArrayAccess dispatch, cleanup ownership, or exact diagnostic ordering.
- `513dbf21`: generalized Object/ArrayAccess write-operation blocker
  classification over shared `AssignTarget` and `UnsetTarget` operation
  results. It classifies write, append, compound update, null-coalesce write,
  and unset operation families through shared backend blocker boundaries. It
  does not execute `offsetSet()` or `offsetUnset()`.
- `6a73b186`: generated-C dynamic callee expression lowering consumes
  `phpc_native_callable_lookup_value_or_closure_with_context_diagnostic` and
  `phpc_native_callable_value_invoke_value_with_diagnostic_and_free`, including
  dynamic string callable and request/global dynamic user-function proof.
- `81c15cd5`: assignment-lvalue operand-list requirement diagnostics.
- `b27bbb20`: direct generated-C user-function request-state frame handoff.
- `b4b21937`: reference-binding operand-list requirement diagnostics.
- `a544daa8`: generic diagnostic operation/operand-list blocker boundary.

## Active Roadmap Items

Primary-integrated capability and lane-local work are separated explicitly.

| Item | Primary Integrated | Candidate Readiness | Toward Full Feature | Status |
| --- | ---: | ---: | ---: | --- |
| Runtime callable ABI and generated-C direct/dynamic consumers | **100%** `[####################]` | **100%** `[####################]` | **66%** `[#############-------]` | Integrated through `6a73b186`. Object/method callable parity, callable array validation, `Class::method` strings, constructors, return references, named/spread breadth, cleanup/unwind, and backend parity remain open. |
| Object/ArrayAccess write-side blockers | **100%** `[####################]` | **100%** `[####################]` | **25%** `[#####---------------]` | Integrated at `513dbf21`. Useful shared blocker boundary, but no executable `offsetSet()`/`offsetUnset()` behavior. |
| Assignment-lvalue operand-list diagnostics | **100%** `[####################]` | **100%** `[####################]` | **15%** `[###-----------------]` | Integrated at `81c15cd5`. Diagnostic vocabulary exists; assignment/RMW execution and writeback do not. |
| Reference-binding operand-list diagnostics | **100%** `[####################]` | **100%** `[####################]` | **15%** `[###-----------------]` | Integrated at `b4b21937`. Diagnostic ABI progress only; executable reference binding remains open. |
| RMW-lvalue operand-list diagnostics | **100%** `[####################]` | **100%** `[####################]` | **14%** `[###-----------------]` | Integrated at `b918d3b1`. Do not recount this as executable RMW progress. |
| Direct request-state frame handoff for generated-C user functions | **100%** `[####################]` | **100%** `[####################]` | **40%** `[########------------]` | Integrated at `b27bbb20`; dynamic generated-C user-function handoff proof exists through `6a73b186`. Closure handoff remains open. |
| Object/ArrayAccess runtime write dispatch ABI | **0%** `[--------------------]` | **85%** `[#################---]` | **35%** `[#######-------------]` | Lane-local after RMW. Prep added runtime `offsetSet`/append/`offsetUnset` dispatch through callable-value ABI; shadow found ownership leaks; repair is ready for shadow review. No compiler lowering yet. |
| RMW executable array-lvalue owner/writeback | **0%** `[--------------------]` | **20%** `[####----------------]` | **28%** `[######--------------]` | Scout plan only. Proposed packet should materialize generated-C array-lvalue owners for local arrays and active symbol/global-import reference slots across multiple RMW families. |
| Cleanup/unwind requirement boundary | **0%** `[--------------------]` | **45%** `[#########-----------]` | **18%** `[####----------------]` | Lane-local refresh in progress after RMW. It found and patched a real RMW diagnostic composition gap, but final audit/gates are pending. |
| Generated declared-method callable-table registration | **0%** `[--------------------]` | **20%** `[####----------------]` | **45%** `[#########-----------]` | Scout plan only. Would publish generated methods into the runtime callable table and bridge method wrappers to `NativeCallFrame`; not implemented. |
| Dead dynamic callable compiler ladder cleanup | **0%** `[--------------------]` | **80%** `[################----]` | **5%** `[#-------------------]` | Approved-but-deferred cleanup. Useful for reducing dead exact-shape debt, but not capability progress. |
| Broad dirty lane/candidate backlog | **0%** `[--------------------]` | **35%** `[#######-------------]` | **36%** `[#######-------------]` | Evidence pools only. Snapshot inventory had 98 candidate/lane worktrees, 86 dirty, with heavy overlap in central compiler/runtime files. |

## Done / In Progress / Not Done

Primary-integrated capability:

- [x] Runtime callable table plus call arguments/frame/result ABI.
- [x] Runtime callable-value dispatch for selected string/binary function
  names, callable arrays, descriptor closures, inherited methods, bound
  receivers, and object `__invoke`.
- [x] Direct generated-C user-function calls through runtime callable lookup,
  arguments, frame wrappers, and value results.
- [x] Direct generated-C user-function root/request frame-environment handoff.
- [x] Generated-C dynamic callee expressions through the shared callable-value
  lookup/invocation ABI.
- [x] Shared diagnostic operation/operand-list blocker boundary.
- [x] Reference-binding operand-list requirement blockers.
- [x] Assignment-lvalue operand-list requirement blockers.
- [x] RMW-lvalue operand-list requirement blockers.
- [x] Object/ArrayAccess write-operation blocker classification.
- [x] Declared-class allocation cleanup-risk metadata.
- [x] Try/catch/finally body call-boundary preflight diagnostics.
- [x] Selected reference-source/lvalue extraction and reference-backed closure
  capture materialization.
- [x] Closure value/reference return ABI for descriptor closures.
- [x] Byte-backed PHP string values and byte-preserving selected string-array
  slots.

In progress but not counted:

- [ ] Object/ArrayAccess runtime write dispatch ABI after ownership repair and
  shadow review.
- [ ] RMW executable array-lvalue owner/writeback packet.
- [ ] Cleanup/unwind requirement boundary refresh after RMW.
- [ ] Generated declared-method callable-table registration and wrapper frames.
- [ ] Dead dynamic callable ladder cleanup.
- [ ] Broad dirty call, diagnostic, object, symbol, byte/string, control-flow,
  and array lanes as evidence pools only.

Not done:

- [ ] Full executable assignment and RMW semantics, including expression
  results, writeback, ordered key coercion, and property/array storage.
- [ ] Full PHP reference binding, references/COW identity, arbitrary alias
  roots, and alias-preserving write-through.
- [ ] Full callable parity beyond generated-C direct/dynamic callable consumers:
  object/method callable parity, callable array validation parity,
  `Class::method` strings, namespace fallback, autoload, magic calls,
  named/spread arguments, by-reference/default/variadic edge cases, return
  references, constructors, source-ordered diagnostics, closure frame handoff,
  and backend parity.
- [ ] Request storage/writeback, `$GLOBALS` self-cells, request/global alias
  parity, request foreach, and mutation-during-iteration behavior.
- [ ] Includes, variable variables, and dynamic symbol behavior.
- [ ] Compiler-consumed runtime `ArrayAccess` execution for `offsetGet`,
  `offsetExists`, `offsetSet`, and `offsetUnset`.
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

- [x] Primary source is clean and synced at `90a9204a`.
- [x] Latest source capability is `b918d3b1`.
- [x] `b918d3b1` is accounted as RMW-lvalue operand-list diagnostic progress
  only, not executable RMW owner/writeback or ArrayAccess dispatch.
- [x] `513dbf21` is accounted as Object/ArrayAccess write-operation blocker
  progress only, not executable `offsetSet()`/`offsetUnset()`.
- [x] Overall and selected-executable percentages remain flat for this review.

Lane-local:

- [ ] Object/ArrayAccess runtime write dispatch repair is ready for shadow
  review, still runtime-only and unintegrated.
- [ ] RMW executable writeback is a scout plan, not a candidate implementation.
- [ ] Cleanup/unwind refresh is in progress and pending final audit.
- [ ] Dead dynamic callable ladder cleanup is approved-but-deferred and should
  not preempt stronger semantic packets.

Resource posture:

- `/dev/shm`: 40G total, 24G used, 17G available, 58% used.
- `/home`: 459G total, 255G used, 185G available, 58% used.
- Memory available is about 37Gi, but swap remains high at 23Gi/29Gi used.
- Largest `/dev/shm` targets: `phpc-target-native-call-semantics` 8.9G,
  `phpc-target-native-object-seed` 5.6G, and
  `phpc-target-native-diagnostics` 3.0G.
- Continue disk-backed `/tmp` target dirs, `umask 0007`,
  `CARGO_BUILD_JOBS=1`, `CARGO_INCREMENTAL=0`, and focused nonzero gates.

## Non-Repeat Guard

Future dynamic-callable work must not repeat the same callable-value
lookup/invocation ABI consumption, dynamic string callable generated-C proof,
request/global dynamic user-function handoff proof, or exact focused gate set
as standalone progress. Distinct next progress should remove dead compiler
ladder debt or broaden a new callable semantic family such as generated method
callable-table registration.

Future Object/ArrayAccess work must not repeat write-operation blocker
classification as standalone progress. Distinct next progress should execute a
new runtime/compiler semantic boundary such as `offsetSet()` / `offsetUnset()`
dispatch, writeback/reference/COW handling, object handle/visibility/magic
property behavior, or exact diagnostics.

Future RMW work must not repeat the RMW-lvalue diagnostic boundary/tag proof as
standalone progress. Operation tag `8`, operand tag `21`, shared
`AssignTarget` operand enumeration, and
`NativeDiagnosticOperandRequirement` operation-list diagnostics are already
accounted. Distinct next progress should implement owner/writeback,
references/COW, object/static property storage, ArrayAccess dispatch, cleanup
ownership, or exact diagnostic ordering.

## Next Steering Read

Best next action:

- Consider integrating the repaired Object/ArrayAccess runtime write-dispatch
  ABI only after shadow review passes, then steer immediately toward a compiler
  assignment/RMW/unset consumer.
- Consider the RMW executable owner/writeback packet if it can stay narrowly
  scoped to generated-C array-lvalue owners and prove multiple operation
  families across local and reference-slot owners.
- Keep cleanup/unwind on current-head reconcile plus final gates.

Avoid:

- Recounting Object/ArrayAccess write-operation blockers; they are already in
  primary at `513dbf21`.
- Recounting RMW-lvalue operand-list diagnostics; they are already in primary
  at `b918d3b1`.
- Treating runtime-only ArrayAccess dispatch as full compiler/execution parity.
- Routing stale cleanup/unwind, trait metadata, or broad dirty lane diffs
  without current-head reconcile and fresh review.
- Letting dead-code cleanup displace a ready executable semantic packet.
