# PHP Native Compiler Progress

Updated: 2026-05-26 04:15 CEST
Evaluation marker: `20260526T020844Z`

Accounting rule: only generalized, tested, committed, and pushed primary work
counts as integrated capability. Dirty WIP, candidate worktrees, lane-local
claims, review-only work, failed prep proofs, probe-only commits, architecture
notes, and dashboard-only commits are excluded.

## Executive Read

Overall supervised-roadmap progress: **95%** `[###################-]`

Selected executable native-compiler islands: **95%** `[###################-]`

Full generalized PHP semantics reality check: **61%** `[############--------]`

Primary `HEAD` is aligned with `origin/master` at:

`90a9204a docs: account RMW lvalue diagnostics`.

Latest primary-integrated source capability baseline:

`b918d3b1 native: add RMW lvalue operand-list diagnostics`.

Recent primary progress is real and well scoped. `b918d3b1` integrates an RMW
lvalue operand-list diagnostic boundary with runtime operation tag `8`, operand
tag `21`, shared `AssignTarget` operand enumeration, runtime ABI coverage, and
compiler classifier tests. It is not executable RMW semantics: owner/writeback,
references/COW, object/static property storage, ArrayAccess dispatch,
missing-key recovery, cleanup ownership, and exact PHP diagnostic ordering
remain open.

The prior Object/ArrayAccess work at `513dbf21` is still blocker
classification only. It does not execute `offsetSet()` or `offsetUnset()`.
The high supervised-roadmap percentage reflects selected native compiler
surfaces, not full PHP language parity.

## Grand Roadmap Position

| Workstream | Estimate | Bar | Current read |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | **99%** | `[####################]` | Strong selected-path value, byte-string, array, reference, symbol, callable table, callable-value dispatch, call-frame/result, request-state, diagnostic operation/operand-list, reference-binding, assignment-lvalue, RMW-lvalue, and object allocation-risk surfaces. Remaining gaps include broader callable lookup parity, namespace fallback, autoload, magic calls, constructors, closure frame handoff, and cleanup/unwind parity. |
| Compiler/backend consumers | **99%** | `[####################]` | Generated C has the freshest executable consumers. Direct and dynamic generated-C callable paths consume shared runtime callable ABI surfaces. Object/ArrayAccess and RMW paths now have shared blocker/diagnostic boundaries. LLVM and direct assembly still lag some recent semantics. |
| Selected executable PHP semantics | **95%** | `[###################-]` | Many selected executable islands exist, but major PHP semantics remain open: full assignment/RMW/writeback, references/COW, executable object/ArrayAccess writes, cleanup/unwind/finally/destructors, exact diagnostics, and backend parity. |
| Full generalized PHP semantics | **61%** | `[############--------]` | Shared boundaries are broad, but the hardest execution cliffs remain: mutation/writeback, alias identity, object/method dispatch, ArrayAccess execution, unwinding/finally/destructors, source-ordered diagnostics, and backend parity. |
| Strings and byte semantics | **62%** | `[############--------]` | Byte-backed values and byte-preserving selected string-array slots are integrated. Binary source bytes, byte-exact interpreter/session/debug output, `mb_str_split()`, request/global byte keys, and exact diagnostics remain open. |
| Arrays, lvalues, references, COW | **83%** | `[#################---]` | Selected reference-source/lvalue extraction, closure capture from reference-backed slots, reference-binding diagnostics, assignment-lvalue diagnostics, RMW-lvalue diagnostics, and Object/ArrayAccess write blockers are integrated. Executable assignment, RMW, broad writeback, arbitrary alias roots, foreach, static/magic/non-public properties, ArrayAccess execution, and full COW remain incomplete. |
| Symbols, globals, request state | **75%** | `[###############-----]` | Selected globals, root-symbol consumers, active symbol-table consumers, request-key blockers, append-shaped symbol reference-source materialization, direct generated-C request-state frame handoff, and generated-C dynamic user-function handoff proof exist. `$GLOBALS` self-cells, closure request-state handoff, request/global alias parity, request writeback, includes, variable variables, and exact unset/global behavior remain incomplete. |
| Calls, functions, frames | **92%** | `[##################--]` | Runtime callable table/value dispatch, call arguments/frame/result ABI, direct generated-C user-function consumers, generated-C dynamic callable-value consumers, by-reference argument transport, descriptor closures, closure returns, and direct/dynamic generated-C request-state frame handoff are integrated. Object/method callable parity, callable array validation parity, `Class::method` strings, namespace fallback, autoload, magic calls, named/spread breadth, return references, constructors, closure frame-environment handoff, and cleanup/unwind parity remain open. |
| Objects, properties, methods | **54%** | `[###########---------]` | Public object-property reference-source extraction, object-property reference-slot mutation, declared-class allocation cleanup-risk metadata, and Object/ArrayAccess write-operation blocker classification exist for selected paths. Full visibility, magic, dynamic/static/typed properties, destructor execution, interfaces/traits execution, references/COW, constructors, and actual `ArrayAccess` execution remain open. |
| Control flow, cleanup, diagnostics | **53%** | `[###########---------]` | Selected branches, loops, transfers, finalizers, output buffers, diagnostics, try-body call-boundary preflight, generic operand-list blockers, reference-binding blockers, assignment-lvalue blockers, RMW-lvalue blockers, and Object/ArrayAccess write blockers exist. Broad unwind/finally/destructor/shutdown execution, cleanup ownership, executable writeback/reference binding, and source-ordered diagnostics remain open. |
| Broad integrated verification | **92%** | `[##################--]` | Recent focused gates are strong and nonzero. The full `native_runtime_abi` baseline still has known current-primary failures, and broad verification remains constrained by lane extraction cost, stale candidate expectations, swap pressure, and backend parity gaps. |

## Recent Primary-Integrated Work

- `b918d3b1`: added generalized RMW lvalue operand-list diagnostics for
  compound assignment, null-coalesce assignment, increment/decrement, and
  `for` RMW target operands. The packet uses operation tag `8`, operand tag
  `21`, shared `AssignTarget` operand enumeration, and the generic
  `NativeDiagnosticOperandRequirement` operation-list ABI. This is diagnostic
  boundary progress only; no executable RMW writeback or COW/reference
  semantics were added.
- `90a9204a`: accounted `b918d3b1` in `PROGRESS.md` only. It made no
  compiler/runtime/test source change.
- `513dbf21`: added generalized object/property ArrayAccess write-operation
  blocker classification over shared `AssignTarget` and `UnsetTarget`
  operation results. It covers write, append, compound update,
  null-coalesce write, and unset operation families through shared LLVM and
  generated-C blocker boundaries. No executable `offsetSet()` or
  `offsetUnset()` dispatch was added.
- `6a73b186`: routed generated-C dynamic callee expression lowering through
  the shared runtime callable-value lookup/invocation ABI and runtime builtin
  dispatch. The remaining compiler-side dynamic callable builtin ladder is
  cleanup debt, not a path to extend.
- `81c15cd5`: added assignment-lvalue operand-list requirement diagnostics
  over the shared runtime/compiler operation-list ABI. Assignment/RMW
  execution and writeback remain open.
- `b27bbb20`: replaced root-symbol-only generated user-function frame metadata
  with `CFrameEnvironmentRequirement { root_symbols, request_state }` and
  propagated those requirements through direct generated-C user-function
  callable/frame handoff.
- `b4b21937`: extended the shared operation-list diagnostic ABI to
  reference-binding target/source and by-reference array item requirements.
- `a544daa8`, `dae1b44c`, `5abf8525`, and `f0cc17c1`: built the generic
  diagnostic operation-list boundary, runtime callable-value dispatch, and
  direct generated-C callable ABI consumers that later packets now consume.

## Active Roadmap Items

Primary-integrated capability and lane-local work are separated explicitly.

| Item | Primary Integrated | Candidate Readiness | Toward Full Feature | Status |
| --- | ---: | ---: | ---: | --- |
| RMW-lvalue operand-list diagnostics | **100%** `[####################]` | **100%** `[####################]` | **16%** `[###-----------------]` | Integrated at `b918d3b1`. Useful shared diagnostic boundary for RMW target operands, but still no executable RMW writeback, COW/reference identity, object/static storage, or ArrayAccess dispatch. |
| Object/ArrayAccess write-side blockers | **100%** `[####################]` | **100%** `[####################]` | **25%** `[#####---------------]` | Integrated at `513dbf21`. Useful shared blocker boundary for write/unset families, but still no executable `offsetSet()`/`offsetUnset()` behavior. |
| Assignment-lvalue operand-list diagnostics | **100%** `[####################]` | **100%** `[####################]` | **15%** `[###-----------------]` | Integrated at `81c15cd5`. Generalized blocker vocabulary exists; assignment/RMW execution and writeback do not. |
| Direct request-state frame handoff for generated-C user functions | **100%** `[####################]` | **100%** `[####################]` | **40%** `[########------------]` | Integrated at `b27bbb20`; extended by `6a73b186` for generated-C dynamic user-function handoff proof. Closure handoff remains open. |
| Reference-binding operand-list diagnostics | **100%** `[####################]` | **100%** `[####################]` | **15%** `[###-----------------]` | Integrated at `b4b21937`. Diagnostic ABI progress only; executable reference binding remains open. |
| Runtime callable ABI, callable-value dispatch, and direct/dynamic generated-C consumers | **100%** `[####################]` | **100%** `[####################]` | **66%** `[#############-------]` | Integrated through `6a73b186`. Direct and dynamic generated-C callable paths consume the shared runtime lookup/invocation ABI and runtime builtin dispatch. Constructors, object/method callable parity, callable array validation parity, broader lookup parity, return references, named/spread breadth, and cleanup/unwind remain open. |
| Cleanup/unwind requirement boundary | **0%** `[--------------------]` | **55%** `[###########---------]` | **18%** `[####----------------]` | Lane-local. After Object/ArrayAccess it was review-clean, but the conflict map after RMW says it needs refresh because diagnostic import/constant lists conflict. Not approved for primary as-is. |
| Trait effective-method metadata | **0%** `[--------------------]` | **30%** `[######--------------]` | **20%** `[####----------------]` | Lane-local/stale relative to current primary. Metadata-only until trait-composed method execution and cleanup consumers exist. |
| Dead dynamic callable compiler ladder cleanup | **0%** `[--------------------]` | **20%** `[####----------------]` | **5%** `[#-------------------]` | Cleanup debt. Useful after `6a73b186`, but not a substitute for new executable semantic coverage. |
| Executable RMW/writeback | **0%** `[--------------------]` | **15%** `[###-----------------]` | **8%** `[##------------------]` | Not implemented. Needs ordered read/write evaluation, owner/writeback contracts, COW/reference identity, object/property/ArrayAccess storage, cleanup, and exact diagnostics. |
| Runtime ArrayAccess execution | **0%** `[--------------------]` | **20%** `[####----------------]` | **10%** `[##------------------]` | Not integrated. Current primary blocks/classifies Object/ArrayAccess write operations; execution of `offsetGet`, `offsetExists`, `offsetSet`, and `offsetUnset` remains open. |
| Broad dirty lane extraction backlog | **0%** `[--------------------]` | **35%** `[#######-------------]` | **36%** `[#######-------------]` | Dirty call, diagnostic, object, control-flow, symbol, byte/string, and array lanes remain evidence pools only. |

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
- [x] RMW-lvalue operand-list requirement blockers.
- [x] Object/ArrayAccess write-operation blocker classification.
- [x] Declared-class allocation cleanup-risk metadata.
- [x] Try/catch/finally body call-boundary preflight diagnostics.
- [x] Selected reference-source/lvalue extraction and reference-backed closure
  capture materialization.
- [x] Closure value/reference return ABI for descriptor closures.
- [x] Byte-backed PHP string values and byte-preserving selected string-array
  slots.

In progress but not counted as primary capability:

- [ ] RMW post-integration shadow audit and dashboard refresh after accounting
  and pages consistency.
- [ ] Cleanup/unwind requirement boundary refresh after `b918d3b1`.
- [ ] Trait effective-method metadata refresh, stale and metadata-only.
- [ ] Dead dynamic callable ladder cleanup.
- [ ] Broad dirty call, diagnostic, object, symbol, byte/string, control-flow,
  and array lanes as evidence pools only.

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

- [x] Primary is clean and synced at `90a9204a`; latest source capability is
  `b918d3b1`.
- [x] Latest source capability is `b918d3b1`.
- [x] `b918d3b1` is accounted as generalized RMW lvalue operand-list
  diagnostic progress only.
- [x] `513dbf21` remains accounted as generalized Object/ArrayAccess
  write-operation blocker progress only.
- [x] Overall supervised-roadmap and selected-executable percentages remain
  flat for this review; lvalue/diagnostic sub-reads nudge slightly.

Lane-local and follow-up state:

- [ ] RMW progress accounting is committed at `90a9204a`, and pages
  consistency is clean.
- [ ] RMW post-integration shadow audit was not yet observed at live
  verification.
- [ ] Supervisor dashboard refresh after the completed RMW accounting/pages
  state was not yet observed at live verification.
- [ ] Cleanup/unwind has useful candidate evidence, but it must refresh after
  RMW because the conflict map reports diagnostic import/constant conflicts.
- [ ] Broad worker lanes are active evidence pools, not primary capability.
- [ ] The supervisor dashboard was stale at live verification: it still
  described RMW as running even though `b918d3b1` had landed.

Resource posture:

- `/dev/shm`: 40G total, 24G used, 17G available, 58% used.
- `/home`: 459G total, 243G used, 197G available, 56% used.
- Memory available is about 36Gi, but swap remains high at 23Gi/29Gi used.
- Continue disk-backed `/tmp` target dirs, `umask 0007`,
  `CARGO_BUILD_JOBS=1`, `CARGO_INCREMENTAL=0`, and focused nonzero gates.

## Non-Repeat Guard

Future RMW work must not repeat the same operation-list diagnostic tag,
operand tag, shared `AssignTarget` operand enumeration, or focused
`native_rmw` diagnostic proof as standalone progress. Distinct next progress
should implement or unblock executable RMW semantics: ordered read/write,
owner/writeback, references/COW, object/static property storage,
ArrayAccess dispatch, cleanup, and exact diagnostics.

Future Object/ArrayAccess work must not repeat the same write-operation blocker
classification proof as standalone progress. Distinct next progress should
implement a new runtime/compiler semantic boundary such as `offsetSet()` /
`offsetUnset()` dispatch, writeback/reference/COW handling, object
handle/visibility/magic property behavior, or exact diagnostics.

Future callable work should not extend the old compiler-side finite dynamic
callable/builtin-name ladder. After `6a73b186`, the route forward is shared
runtime callable-value lookup/invocation semantics or behavior-preserving
cleanup of dead compiler code.

## Next Steering Read

Best next action:

- Complete RMW follow-through: shadow audit and dashboard refresh after
  `90a9204a`.
- If cleanup/unwind is next, refresh it after `b918d3b1`, union the diagnostic
  imports/constants, and rerun focused cleanup/RMW-adjacent gates before any
  primary route.
- Prefer the next source integration that removes an executable semantic cliff:
  RMW writeback, reference/COW alias identity, executable ArrayAccess writes,
  object/method callable parity, or real cleanup/unwind execution.

Avoid:

- Recounting RMW operand-list diagnostics; they are already in primary at
  `b918d3b1`.
- Recounting Object/ArrayAccess write-operation blockers; they are already in
  primary at `513dbf21`.
- Routing cleanup/unwind or trait metadata without current-head reconcile and
  fresh review.
- More diagnostic/metadata-only packets unless they remove a concrete blocker
  and feed an imminent executable consumer.
- Directly importing broad dirty lanes.
