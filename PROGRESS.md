# PHP Native Compiler Progress

Updated: 2026-05-26 05:24 CEST
Evaluation marker: `20260526T032451Z`

Accounting rule: only generalized, tested, committed, and pushed primary work
counts as integrated capability. Dirty WIP, candidate worktrees, lane-local
claims, review-only work, failed prep proofs, probe-only commits, architecture
notes, and dashboard-only commits are excluded.

## Executive Read

Overall supervised-roadmap progress: **95%** `[###################-]`

Selected executable PHP semantics: **95%** `[###################-]`

Primary `HEAD` is aligned with `origin/master` at
`ccb16eb0 native: add cleanup unwind requirement boundary`.

Latest primary-integrated source capability baseline:
`ccb16eb0 native: add cleanup unwind requirement boundary`.

Recent source progress is real but still selected-path rather than full PHP
parity. `ccb16eb0` adds the cleanup/unwind requirement boundary as
diagnostic/preflight infrastructure for statement, lvalue, value, call,
transfer, nested-container, and destructor-observable cleanup requirement
families. It composes with the generated-C RMW array-lvalue owner/writeback
path from `e17998ef`, which covers local native array handles and
active-symbol/global-import reference-slot owners across compound assignment,
null-coalesce assignment, and increment/decrement.

This complements `0f4a8603` runtime ArrayAccess read/exists dispatch,
`682f3aef` runtime write/unset dispatch, and `d2adc130` generated
declared-method callable-table publication. It does not add actual stack
unwinding, `Throwable` construction/propagation, catch matching/binding,
finally execution, destructor execution, object lifetime cleanup, exact
diagnostic ordering, object/static property storage, ArrayAccess RMW dispatch,
broad alias roots, exact reference/COW parity, LLVM consumers, or backend
parity.

Full PHP callable, object, lvalue, cleanup, diagnostic, and backend parity
remain incomplete. Overall and selected-executable percentages remain flat.

Current lane-local signals are explicitly not counted. ArrayAccess read/isset
compiler-consumer work is active candidate work, pages remain blocked by
gh-pages generated-output deletions pending explicit approval to
restore/regenerate, and dead dynamic-callable ladder cleanup remains
approved-but-deferred cleanup debt.

## Grand Roadmap Position

| Workstream | Estimate | Bar | Current read |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | **99%** | `[####################]` | Strong selected-path value, byte-string, array, reference, symbol, callable table, callable-value dispatch, call-frame/result, request-state, diagnostics, lvalue, and ArrayAccess read/write dispatch surfaces. Remaining gaps include broader callable lookup parity, namespace fallback, autoload, magic calls, constructors, closure frame handoff, reference-return ArrayAccess, and cleanup/unwind parity. |
| Compiler/backend consumers | **99%** | `[####################]` | Generated C has the freshest executable consumers for direct/dynamic callables, declared methods, and selected RMW array-lvalue owner/writeback. Recent ArrayAccess read/write dispatch is runtime-only; generated-C, LLVM, and direct assembly still need consumers for object offset operations and broader lvalue/runtime ABIs. |
| Executable PHP semantics | **95%** | `[###################-]` | Many selected executable islands exist, including selected generated-C RMW array-lvalue owner/writeback, but major PHP semantics remain open: full assignment/RMW/writeback, references/COW, executable object/ArrayAccess operations, cleanup/unwind/finally/destructors, exact diagnostics, and backend parity. |
| Strings and byte semantics | **62%** | `[############--------]` | Byte-backed values and byte-preserving selected string-array slots are integrated. Binary source bytes, byte-exact interpreter/session/debug output, `mb_str_split()`, request/global byte keys, and exact diagnostics remain open. |
| Arrays, lvalues, references, COW | **83%** | `[#################---]` | Selected reference-source/lvalue extraction, closure capture from reference-backed slots, reference-binding diagnostics, assignment/RMW-lvalue diagnostics, generated-C RMW array-lvalue owner/writeback for local arrays and reference-slot owners, and Object/ArrayAccess blocker/runtime dispatch pieces are integrated. Object/static property storage, ArrayAccess RMW dispatch, arbitrary alias roots, foreach, broader writeback, and full COW remain incomplete. |
| Symbols, globals, request state | **75%** | `[###############-----]` | Selected globals, root-symbol consumers, active symbol-table consumers, request-key blockers, append-shaped symbol reference-source materialization, direct generated-C request-state frame handoff, and generated-C dynamic user-function handoff proof exist. `$GLOBALS` self-cells, closure request-state handoff, request/global alias parity, request writeback, includes, variable variables, and exact unset/global behavior remain incomplete. |
| Calls, functions, frames | **93%** | `[###################-]` | Runtime callable table/value dispatch, call arguments/frame/result ABI, direct generated-C user-function consumers, generated-C dynamic callable-value consumers, generated declared-method callable registration/wrapper frames, by-reference argument transport, descriptor closures, closure returns, and direct/dynamic generated-C request-state frame handoff are integrated. Full object/method callable parity, callable array validation parity, `Class::method` strings, namespace fallback, autoload, magic calls beyond declared `__invoke`, named/spread breadth, return references, constructors, closure frame-environment handoff, cleanup/unwind parity, and backend parity remain open. |
| Objects, properties, methods | **57%** | `[###########---------]` | Public object-property reference-source extraction, object-property reference-slot mutation, declared-class allocation cleanup-risk metadata, Object/ArrayAccess write blockers, runtime ArrayAccess write/read/exists dispatch, and generated declared-method callable-table publication exist for selected paths. Compiler-consumed ArrayAccess lowering, non-public visibility parity, magic, dynamic/static/typed properties, destructors, interfaces/traits execution, references/COW, constructors, and backend parity remain open. |
| Control flow, cleanup, diagnostics | **54%** | `[###########---------]` | Selected branches, loops, transfers, finalizers, output buffers, diagnostics, try-body call-boundary preflight, generic operand-list blockers, reference-binding blockers, assignment/RMW-lvalue blockers, Object/ArrayAccess write blockers, and cleanup/unwind requirement preflight exist. Broad unwind/finally/destructor/shutdown execution, cleanup ownership, executable reference binding, and source-ordered diagnostics remain open. |
| Broad integrated verification | **92%** | `[##################--]` | Recent focused gates are strong and nonzero. The full `native_runtime_abi` baseline still has known current-primary failures, and broad verification remains constrained by lane extraction cost, stale candidate expectations, swap pressure, and backend parity gaps. |

## Recent Primary-Integrated Work

- `ccb16eb0`: added a generalized cleanup/unwind requirement boundary for
  statement, lvalue, value, call, transfer, nested-container, and
  destructor-observable cleanup requirement families. This is
  diagnostic/preflight infrastructure; actual unwinding, `Throwable`
  construction/propagation, catch matching/binding, finally execution,
  destructor execution, object lifetime cleanup, exact diagnostic ordering,
  and backend parity remain open.
- `e17998ef`: added generated-C native array-lvalue owner materialization and
  writeback for RMW families over local native array handles and
  active-symbol/global-import reference-slot owners. Compound assignment,
  null-coalesce assignment, and increment/decrement now consume the shared
  owner/writeback path for selected array lvalues. Object/static property
  storage, ArrayAccess RMW dispatch, broad alias roots, exact reference/COW
  parity, cleanup/unwind, exact diagnostics, LLVM consumers, and backend
  parity remain open.
- `0f4a8603`: added runtime ABI
  `phpc_native_value_arrayaccess_offset_read_operation_with_diagnostic` for
  `ArrayAccess::offsetGet($offset)` and
  `ArrayAccess::offsetExists($offset)`. Dispatch uses runtime object/interface
  metadata, callable-table method lookup, bound receiver invocation, and the
  shared native call-result ABI. This is runtime-only progress; compiler
  lowering, `isset`/`empty`/null-coalesce sequencing, reference-return
  `offsetGet`, cleanup/unwind, exact diagnostics, and backend parity remain
  open.
- `d2adc130`: registered generated declared methods into the runtime callable
  table and generated method wrapper frames that bridge `NativeCallFrame`
  receiver/value/reference arguments into declared method frames, returning
  through the shared native call-result ABI. This is generated-C declared-method
  callable-table publication progress only; constructors, magic calls beyond
  declared `__invoke`, autoload, namespace fallback, `Class::method` strings,
  named/spread breadth, return references, cleanup/unwind, exact diagnostics,
  non-public visibility parity, and backend parity remain open.
- `682f3aef`: added runtime ABI
  `phpc_native_value_arrayaccess_offset_write_operation_with_diagnostic` for
  `ArrayAccess::offsetSet($offset, $value)`,
  `ArrayAccess::offsetSet(null, $value)`, and
  `ArrayAccess::offsetUnset($offset)` dispatch through the shared
  callable-value ABI. This is runtime ABI progress only; compiler lowering,
  assignment/RMW owner writeback, cleanup/unwind parity, `offsetGet`,
  `offsetExists`, reference/COW, magic, constructor, visibility, exact
  diagnostics, and backend parity remain open.
- `b918d3b1`: added a generalized RMW-lvalue operand-list diagnostic boundary
  over compound assignment, null-coalesce assignment, and increment/decrement
  target operands. The integrated packet adds diagnostic operation tag `8`,
  diagnostic operand tag `21`, and routes shared `AssignTarget` operand
  enumeration through `NativeDiagnosticOperandRequirement` operation-list
  diagnostics. This is a diagnostic boundary only; executable RMW
  owner/writeback, references/COW, object/static property storage, ArrayAccess
  dispatch, cleanup ownership, and exact diagnostic ordering remain open.
- `513dbf21`: added generalized object/property ArrayAccess write-operation
  blocker classification over shared `AssignTarget` and `UnsetTarget`
  operation results. The integrated packet classifies object/property
  ArrayAccess write, append, compound update, null-coalesce write, and unset
  operation families through shared LLVM/generated-C backend blocker
  boundaries. No executable `offsetSet()` or `offsetUnset()` dispatch was
  added.
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

Primary-integrated capability and lane-local work are separated explicitly.

| Item | Primary Integrated | Candidate Readiness | Toward Full Feature | Status |
| --- | ---: | ---: | ---: | --- |
| Object/ArrayAccess write-side blockers | **100%** `[####################]` | **100%** `[####################]` | **25%** `[#####---------------]` | Integrated at `513dbf21`. Useful shared blocker boundary for write/unset families, but still no executable `offsetSet()`/`offsetUnset()` behavior. |
| Assignment-lvalue operand-list diagnostics | **100%** `[####################]` | **100%** `[####################]` | **15%** `[###-----------------]` | Integrated at `81c15cd5`. Generalized blocker vocabulary exists; assignment/RMW execution and writeback do not. |
| Direct request-state frame handoff for generated-C user functions | **100%** `[####################]` | **100%** `[####################]` | **40%** `[########------------]` | Integrated at `b27bbb20`; extended by `6a73b186` for generated-C dynamic user-function handoff proof. Closure handoff remains open. |
| Reference-binding operand-list diagnostics | **100%** `[####################]` | **100%** `[####################]` | **15%** `[###-----------------]` | Integrated at `b4b21937`. Diagnostic ABI progress only; executable reference binding remains open. |
| Runtime callable ABI, callable-value dispatch, and direct/dynamic generated-C consumers | **100%** `[####################]` | **100%** `[####################]` | **68%** `[##############------]` | Integrated through `6a73b186` and `d2adc130`. Direct/dynamic generated-C callable paths consume the shared runtime lookup/invocation ABI, and generated declared methods are published through callable-table method descriptors and wrapper frames. Constructors, full object/method callable parity, callable array validation parity, broader lookup parity, return references, named/spread breadth, and cleanup/unwind remain open. |
| Dynamic callable compiler consumer plus builtin string callable repair | **100%** `[####################]` | **100%** `[####################]` | **60%** `[############--------]` | Integrated at `6a73b186`. Dynamic callee expression lowering routes through the shared callable-value lookup/invocation ABI instead of a compiler-side finite builtin/name ladder. Full callable semantics remain open. |
| RMW-lvalue operand-list diagnostics | **100%** `[####################]` | **100%** `[####################]` | **14%** `[###-----------------]` | Integrated at `b918d3b1`. Generalized diagnostic boundary uses operation tag `8`, operand tag `21`, shared `AssignTarget` operand enumeration, and `NativeDiagnosticOperandRequirement` operation-list diagnostics. Executable RMW writeback/reference/COW behavior remains open. |
| Object/ArrayAccess runtime read/exists dispatch ABI | **100%** `[####################]` | **100%** `[####################]` | **40%** `[########------------]` | Integrated at `0f4a8603`. Runtime `offsetGet`/`offsetExists` dispatch exists, including truthiness conversion for `offsetExists`; no compiler lowering, `isset`/`empty`/null-coalesce sequencing, or reference/COW parity yet. |
| Object/ArrayAccess runtime write dispatch ABI | **100%** `[####################]` | **100%** `[####################]` | **35%** `[#######-------------]` | Integrated at `682f3aef`. Runtime dispatch exists for keyed `offsetSet`, append `offsetSet(null, value)`, and `offsetUnset`; no compiler lowering or owner/writeback consumer yet. |
| RMW executable array-lvalue owner/writeback | **100%** `[####################]` | **100%** `[####################]` | **36%** `[#######-------------]` | Integrated at `e17998ef`. Generated-C RMW owner materialization/writeback now covers local native array handles and active-symbol/global-import reference-slot owners across compound assignment, null-coalesce assignment, and increment/decrement. Object/static property storage, ArrayAccess RMW dispatch, broad alias roots, exact reference/COW parity, cleanup/unwind, exact diagnostics, LLVM consumers, and backend parity remain open. |
| Cleanup/unwind requirement boundary | **100%** `[####################]` | **100%** `[####################]` | **25%** `[#####---------------]` | Integrated at `ccb16eb0`. Generalized diagnostic/preflight cleanup requirement boundary now covers statement, lvalue, value, call, transfer, nested-container, and destructor-observable requirement families; actual unwinding/finally/destructor execution remains open. |
| Generated declared-method callable-table registration | **100%** `[####################]` | **100%** `[####################]` | **45%** `[#########-----------]` | Integrated at `d2adc130`. Generated declared methods are registered in the runtime callable table and wrapper frames bridge `NativeCallFrame` receiver/value/reference arguments into declared method frames. Full callable/object parity remains open. |
| `Class::method` string callable resolution | **0%** `[--------------------]` | **25%** `[#####---------------]` | **28%** `[######--------------]` | Scout plan recommends consuming `d2adc130` method descriptors through runtime callable-value string parsing. No candidate implementation yet. |
| Trait effective-method metadata | **0%** `[--------------------]` | **30%** `[######--------------]` | **20%** `[####----------------]` | Lane-local/stale relative to current primary. Metadata-only until trait-composed method execution and cleanup consumers exist. |
| Dead dynamic callable compiler ladder cleanup | **0%** `[--------------------]` | **80%** `[################----]` | **5%** `[#-------------------]` | Approved-but-deferred cleanup. Useful to remove after `6a73b186`, but not a substitute for new executable semantic coverage. |
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
- [x] Generated-C declared-method callable-table registration and method
  wrapper frames through the shared runtime callable table/call-frame/result
  ABI.
- [x] Shared diagnostic operation/operand-list blocker boundary.
- [x] Reference-binding operand-list requirement blockers.
- [x] Assignment-lvalue operand-list requirement blockers.
- [x] RMW-lvalue operand-list requirement blockers.
- [x] Generated-C RMW array-lvalue owner materialization/writeback for local
  native array handles and active-symbol/global-import reference-slot owners
  across compound assignment, null-coalesce assignment, and increment/
  decrement.
- [x] Cleanup/unwind requirement boundary diagnostics/preflight over statement,
  lvalue, value, call, transfer, nested-container, and destructor-observable
  cleanup requirement families.
- [x] Object/ArrayAccess write-operation blocker classification.
- [x] Runtime-only Object/ArrayAccess `offsetSet`/append/`offsetUnset`
  dispatch ABI through callable-value invocation.
- [x] Runtime-only Object/ArrayAccess `offsetGet`/`offsetExists` dispatch ABI
  through callable-value invocation.
- [x] Declared-class allocation cleanup-risk metadata.
- [x] Try/catch/finally body call-boundary preflight diagnostics.
- [x] Selected reference-source/lvalue extraction and reference-backed closure
  capture materialization.
- [x] Closure value/reference return ABI for descriptor closures.
- [x] Byte-backed PHP string values and byte-preserving selected string-array
  slots.

In progress but not counted:

- [ ] `Class::method` string callable resolution candidate based on the scout
  plan.
- [ ] Compiler-generated-C/LLVM ArrayAccess read/write/isset/empty lowering
  that consumes the runtime read/write dispatch ABIs.
- [ ] Trait effective-method metadata refresh, stale and metadata-only.
- [ ] Dead dynamic callable ladder cleanup.
- [ ] Broad dirty call, diagnostic, object, symbol, byte/string, control-flow,
  and array lanes as evidence pools only.

Not done:

- [ ] Full broad executable assignment and RMW semantics, including all
  expression results, ordered key coercion, property/static storage,
  ArrayAccess dispatch, arbitrary alias roots, and COW-preserving writeback.
- [ ] Full PHP reference binding, references/COW identity, arbitrary alias
  roots, and alias-preserving write-through.
- [ ] Full callable parity beyond the integrated generated-C dynamic callable
  consumer and declared-method wrapper publication: object/method callable
  parity, callable array validation parity,
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

- [x] Primary source is clean and synced at `ccb16eb0`.
- [x] Latest source capability is `ccb16eb0`.
- [x] `ccb16eb0` is accounted as generalized cleanup/unwind requirement
  boundary diagnostics/preflight over statement, lvalue, value, call,
  transfer, nested-container, and destructor-observable cleanup requirement
  families. It is not counted as actual stack unwinding, `Throwable`
  construction/propagation, catch matching/binding, finally execution,
  destructor execution, object lifetime cleanup, exact diagnostic ordering,
  LLVM/backend parity, or full PHP cleanup parity.
- [x] `e17998ef` is accounted as generated-C RMW array-lvalue owner
  materialization/writeback for local native array handles and active-symbol/
  global-import reference-slot owners across compound assignment,
  null-coalesce assignment, and increment/decrement. It is not counted as
  object/static property storage, ArrayAccess RMW dispatch, broad alias roots,
  exact reference/COW parity, cleanup/unwind, exact diagnostics, LLVM/backend
  parity, or full PHP RMW parity.
- [x] `0f4a8603` is accounted as runtime-only Object/ArrayAccess
  `offsetGet`/`offsetExists` dispatch ABI progress, not compiler lowering,
  `isset`/`empty`/null-coalesce sequencing, reference/COW identity,
  reference-return `offsetGet`, cleanup/unwind, exact diagnostics, or backend
  parity.
- [x] `d2adc130` is accounted as generated-C declared-method callable-table
  registration and method wrapper-frame publication through the shared runtime
  callable table/call-frame ABI, not full callable/object parity.
- [x] `682f3aef` is accounted as runtime-only Object/ArrayAccess offset write
  dispatch ABI progress, not compiler lowering or full ArrayAccess parity.
- [x] `6a73b186` is accounted as executable generated-C dynamic callable
  progress through the shared runtime callable-value ABI.
- [x] `513dbf21` is accounted as generalized Object/ArrayAccess write-operation
  blocker progress only, not executable `offsetSet()`/`offsetUnset()` behavior.
- [x] `b918d3b1` is accounted as generalized RMW-lvalue operand-list diagnostic
  progress only, not executable RMW owner/writeback, references/COW,
  object/static property storage, ArrayAccess dispatch, cleanup ownership, or
  exact diagnostic ordering.
- [x] Overall and selected-executable percentages remain flat for this
  accounting; the control-flow/cleanup workstream and cleanup/unwind item now
  reflect the integrated requirement-boundary diagnostic/preflight work.

Lane-local:

- [ ] `Class::method` string callable work is a plan, not an implementation.
- [ ] ArrayAccess read/isset compiler-consumer work is boundary-plan-ready, not
  implementation-ready.
- [ ] Dead dynamic callable ladder cleanup is approved-but-deferred and should
  not preempt stronger semantic packets.
- [ ] Trait metadata still needs current-head review before any primary route.
- [ ] Broad dirty lanes remain evidence repositories, not integration units.

Resource posture:

- `/dev/shm`: 40G total, 24G used, 17G available, 58% used.
- `/home`: 459G total, 239G used, 202G available, 55% used.
- Memory available is about 36Gi, but swap remains high at 23Gi/29Gi used.
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

Future object/method callable work must not recount generated declared-method
callable-table registration or wrapper-frame publication from `d2adc130` as
standalone progress. Distinct next progress should broaden a new callable
semantic family such as constructors, `Class::method` strings, namespace
fallback, autoload, named/spread arguments, return references, cleanup/unwind,
visibility parity, or backend parity.

Future Object/ArrayAccess work must not repeat the same write-operation blocker
classification proof, runtime `offsetSet`/append/`offsetUnset` dispatch from
`682f3aef`, or runtime `offsetGet`/`offsetExists` dispatch from `0f4a8603` as
standalone progress. Distinct next progress should implement compiler
lowering/sequencing, RMW integration, writeback/reference/COW handling,
cleanup/unwind ownership, object handle/visibility/magic property behavior,
exact diagnostics, or backend parity through a shared semantic boundary.

Future RMW work must not repeat the same RMW-lvalue diagnostic boundary/tag
proof or the selected generated-C array-lvalue owner/writeback path from
`e17998ef` as standalone progress. Operation tag `8`, operand tag `21`, shared
`AssignTarget` operand enumeration,
`NativeDiagnosticOperandRequirement` operation-list diagnostics, local array
handle owners, active-symbol/global-import reference-slot owners, and
compound/null-coalesce/increment/decrement generated-C owner writeback are
already accounted. Distinct next progress should implement broader
references/COW, object/static property storage, ArrayAccess dispatch, broad
alias roots, cleanup ownership, exact diagnostic ordering, or backend parity.

Future cleanup/unwind work must not recount the `ccb16eb0` requirement-boundary
diagnostic/preflight surface as standalone progress. Statement, lvalue, value,
call, transfer, nested-container, destructor-observable cleanup requirement
families, try-body call-boundary preflight, and cleanup requirement
classification are already accounted. Distinct next progress should implement
actual unwinding, `Throwable` construction/propagation, catch matching/binding,
finally execution, destructor execution, object lifetime cleanup, cleanup
ownership, exact diagnostic ordering, or backend parity.

## Next Steering Read

Best next action:

- Treat `d2adc130` as integrated generated-C declared-method callable-table
  registration progress and steer follow-up method-callable work toward a new
  callable semantic family rather than republishing the same descriptors.
- Treat `682f3aef` and `0f4a8603` as integrated runtime ABI progress and steer
  immediately toward compiler consumers for ArrayAccess reads/writes,
  `isset`/`empty`/null-coalesce sequencing, assignment/RMW owner writeback, or
  reference/COW behavior.
- Treat `e17998ef` as integrated generated-C RMW array-lvalue
  owner/writeback progress for local array handles and active-symbol/global-
  import reference-slot owners; steer follow-up RMW work toward broader
  references/COW, object/static property storage, ArrayAccess dispatch, broad
  alias roots, cleanup ownership, exact diagnostic ordering, or backend parity.
- Push the next object/ArrayAccess work toward compiler-consumed
  `offsetSet()` / `offsetUnset()` owner writeback, `offsetGet()` /
  `offsetExists()`, or reference/COW-aware writeback rather than more blocker
  classification.

Avoid:

- Recounting Object/ArrayAccess write-operation blockers; they are already in
  primary at `513dbf21`.
- Recounting runtime ArrayAccess write dispatch from `682f3aef` or read/exists
  dispatch from `0f4a8603`.
- Recounting RMW-lvalue operand-list diagnostics; they are already in primary
  at `b918d3b1`.
- Recounting selected generated-C RMW array-lvalue owner/writeback; it is
  already in primary at `e17998ef`.
- Recounting cleanup/unwind requirement diagnostics/preflight; it is already in
  primary at `ccb16eb0`.
- Treating runtime-only ArrayAccess dispatch as full compiler/execution parity.
- Recounting generated declared-method callable-table registration or wrapper
  frames; they are already in primary at `d2adc130`.
- Routing more cleanup/unwind or trait metadata without a deliberate
  current-head integration decision and fresh review.
- Re-integrating the dynamic callable compiler consumer; it is already in
  primary at `6a73b186`.
- More diagnostic/metadata-only packets unless they remove a concrete blocker
  and feed an imminent executable consumer.
- Directly importing broad dirty lanes.
- Reintroducing or extending compiler-side finite dynamic-call or builtin-name
  ladders instead of consuming shared runtime callable-value semantics.
