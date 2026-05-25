# PHP Native Compiler Progress

Updated: 2026-05-25 16:36 CEST
Evaluation marker: `20260525T143621Z`

Accounting rule: only generalized, tested, committed, and pushed primary work
counts as integrated capability. Dirty WIP, candidate worktrees, lane-local
claims, probe-only commits, and dashboard-only commits are excluded.

## Executive Read

Overall estimated progress: **90%** `[##################--]`

Executable PHP semantics: **90%** `[##################--]`

Primary was clean and aligned with `origin/master` at
`a501c4d1 native: block request-backed array keys` before this dashboard edit.
This review counts the request-backed array-key/RMW blocker as a
primary-integrated executable-prerequisite packet. It does **not** count as
executable request storage, request writeback, `$GLOBALS` parity, or broad
request/global semantics.

The useful movement since the prior dashboard is primary-integrated, not merely
lane-local: LLVM and generated C now share request-backed ordinary array-key
blocker classification across read, assignment, unset, reference assignment,
for-action assignment/RMW, compound assignment, null-coalescing assignment, and
increment/decrement consumers.

Full generalized PHP remains blocked on references/COW identity, arbitrary
lvalues, request/global parity, includes, variable variables, broad userland
frames, real `ArrayAccess`, object/magic/visibility/destructor semantics,
cleanup/unwind/finally/shutdown ordering, exact diagnostics/error handlers, and
backend parity.

## Primary-Integrated Baseline

- Current primary head before this dashboard edit:
  `a501c4d1 native: block request-backed array keys`.
- Current head type: semantic/prerequisite blocker packet.
- Latest integrated executable/prerequisite semantic baseline:
  `a501c4d1 native: block request-backed array keys`.
- Prior integrated prerequisite:
  `24ec4a10 native: route reference truthiness slots`.
- Prior integrated prerequisite:
  `146c2d64 native: route reference comparison slots`.
- Prior integrated prerequisite:
  `9f373b25 native: route reference text membership slots`.
- Prior integrated prerequisite:
  `8f6266ce native: route reference slot type and int consumers`.
- Prior integrated prerequisite:
  `9022eb9e native: add array key reference slot ABI`.
- Prior integrated prerequisite:
  `cc7efc2d native: add offset read source result ABI`.
- Prior integrated executable object/reference feature:
  `bfbc62c4 native: route object property reference slots`.
- Latest integrated non-executable classifier:
  `deaf52ca codegen: classify object ArrayAccess receivers`.

## Grand Roadmap Position

| Workstream | Estimate | Bar | Current read |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | **99%** | `[####################]` | Strong selected-path value, array, string, diagnostic, reference, symbol, call-frame, object, comparison, conversion, owner-cell, request-state, offset-read, array-key, type/int, text-membership, comparison, and truthiness slot surfaces. |
| Compiler/backend consumers | **99%** | `[####################]` | Generated C and LLVM consume many shared ABIs and now share request-backed array-key blocker classification. Direct assembly and recent generated-C-only semantics still lag. |
| Executable PHP semantics | **90%** | `[##################--]` | Primary has closure/callable/object islands, bounded preg callbacks, object-property reference-slot mutation, offset-read continuation proof, reference-backed array-key conversion, type/int, text-membership, comparison, truthiness consumers, and request-key blocker parity. |
| Arrays, lvalues, references, COW | **77%** | `[###############-----]` | Value/reference slot ABI reuse is expanding and request-backed unsafe key materialization is now blocked. Full COW, arbitrary roots, foreach, property references, broader expression reference slots, and alias composition remain open. |
| Symbols, globals, request state | **73%** | `[###############-----]` | Selected function globals, root-symbol surfaces, active symbol-table reference consumers, and request-backed key blockers exist. `$GLOBALS` self-cells, request/global alias parity, request writeback, includes, variable variables, and exact unset/global behavior remain incomplete. |
| Calls, functions, frames | **83%** | `[#################---]` | Selected direct/callable/function-table surfaces exist. Named/unpacked/by-reference/userland frame breadth remains incomplete. |
| Objects, properties, methods | **50%** | `[##########----------]` | Object-property reference-slot mutation and diagnostic classifiers are integrated. Full visibility, magic, dynamic/static/typed properties, destructors, references/COW, and `ArrayAccess` execution remain open. |
| Control flow, cleanup, diagnostics | **51%** | `[##########----------]` | Selected branches, loops, transfers, finalizers, output buffers, diagnostics, and truthiness consumers exist. Broad unwind/finally/destructor/shutdown and exact source ordering remain open. |
| Broad integrated verification | **89%** | `[##################--]` | Focused gates are strong for recent packets, including request-backed blocker review/integration. Broad gates remain constrained by lane extraction cost, high swap, stale lane expectations, and backend parity gaps. |

## Active Roadmap Items

| Item | Toward Primary Integration | Toward Full Feature | Status |
| --- | ---: | ---: | --- |
| Request-backed array-key/RMW blocker parity | **100%** `[####################]` | **37%** `[#######-------------]` | Integrated at `a501c4d1`. LLVM and generated C share blocker classification for request-backed ordinary array-key consumers across read, assignment, unset, reference assignment, for-action assignment/RMW, compound assignment, `??=`, and increment/decrement. Blocker-only: no request storage/writeback or `$GLOBALS` parity. |
| Truthiness value/reference-slot consumers | **100%** `[####################]` | **45%** `[#########-----------]` | Integrated at `24ec4a10`. Runtime, LLVM, and generated C route covered unary/logical truthiness, scalar/static `empty()`, reference-held operands, and native-value variable `isset`/`empty` proof paths through a shared value/reference-slot diagnostic ABI. |
| Reference-slot comparison consumers | **100%** `[####################]` | **45%** `[#########-----------]` | Integrated at `146c2d64`. Runtime, LLVM, and generated C route covered native value/reference comparison operands through shared diagnostic comparison slot ABIs. |
| Text-membership/reference text-byte conversion | **100%** `[####################]` | **44%** `[#########-----------]` | Integrated at `9f373b25`. Shared runtime value/reference text slot feeds selected `function_exists()` and `extension_loaded()` consumers. |
| Reference-slot type/int consumer ABI | **100%** `[####################]` | **45%** `[#########-----------]` | Integrated at `8f6266ce`. Runtime, LLVM, and generated C route reference-held type names, type predicates, and supported int operands through shared value/reference slots. |
| Array-key value/reference-slot ABI | **100%** `[####################]` | **42%** `[########------------]` | Integrated at `9022eb9e`. Broader expression lvalues, COW identity, object/resource/Stringable keys, request/global execution, and direct `ArrayAccess` remain open. |
| Scalar/resource offset-read source-result prerequisite | **100%** `[####################]` | **44%** `[#########-----------]` | Integrated at `cc7efc2d`. Direct object `ArrayAccess`, object/resource materialization, and LLVM error-status cleanup remain open. |
| Object-property reference-slot mutation | **100%** `[####################]` | **39%** `[########------------]` | Integrated at `bfbc62c4`. Covered assignment/unset operands use shared value/reference slot handling. |
| Bounded `preg_replace_callback()` string callbacks | **100%** `[####################]` | **32%** `[######--------------]` | Integrated at `6aca392d`. Full PCRE, broader captures/modifiers, non-string callables, `limit`/`count`/`flags`, and legacy recognizer cleanup remain open. |
| String operation-family slot consumers | **45%** `[#########-----------]` | **39%** `[########------------]` | Lane evidence shows value/reference-slot emission for string-result/array/int/position/parser/distance families. Best next packet only if one tight family is split with runtime/backend proof and no builtin accumulation. |
| Broader lvalue/reference-slot materializer | **30%** `[######--------------]` | **39%** `[########------------]` | Needed so non-variable expression families that can carry references can enter shared array-key and consumer slot ABIs safely. Should follow a narrow extraction brief, not broad foreach/ArrayAccess execution. |
| Throw/source blocker and exception boundary | **38%** `[########------------]` | **28%** `[######--------------]` | Lane-local in `impl-native-type-conversion`: generated native `throw` operands materialize through one source blocker before a noncontinuable Throwable boundary. Real exception objects, unwind, catch/finally, and handlers remain open. |
| Object/resource source materialization | **25%** `[#####---------------]` | **30%** `[######--------------]` | Explicit blocker left by the offset-read ABI. Needs a general value reconstruction boundary before generic object/resource consumers are safe. |
| LLVM offset-read/error-status cleanup | **25%** `[#####---------------]` | **30%** `[######--------------]` | Offset-read diagnostics exist, but LLVM still needs a generalized control-flow/error-exit status boundary for failed conversion results. |
| Static-property comparison operand ABI | **35%** `[#######-------------]` | **37%** `[#######-------------]` | Prior extraction says `needs-split`: source lane is too broad and entangled. Split metadata/operand prerequisites first. |
| Callable-object/dynamic-constructor candidates | **52%** `[##########----------]` | **42%** `[########------------]` | May 24 candidates still look useful but are stale relative to current primary and the May 25 slot/request integrations. Refresh from `a501c4d1` before review and do not combine them. |
| Linked symbol-table value carriers | **35%** `[#######-------------]` | **35%** `[#######-------------]` | `impl-symbol-integrator` has useful lane-local result-carrier evidence, but the lane is broad and dirty. Extract only compact packets. |
| Diagnostics, request, and cleanup boundaries | **61%** `[############--------]` | **41%** `[########------------]` | Lane-local request handle, writeback, branch cleanup, destructuring, RMW read-source, stateful-call cleanup, and result-boundary work is useful infrastructure. Exact Zend ordering and real handler/exception execution remain open. |
| Broad lane extraction backlog | **34%** `[#######-------------]` | **35%** `[#######-------------]` | Broad dirty lanes continue producing useful surfaces, but several were checkpointed, parked, paused, or stopped for stale cadence and broad conflict-heavy probing. Treat lanes as packet sources, not integration units. |

## Done / In Progress / Not Done

Primary-integrated executable or executable-prerequisite capability:

- [x] Descriptor-backed closures, selected captures, selected by-reference parameters, and selected callable-array/object invocation.
- [x] Runtime string-valued declared-class `new` for selected declared classes, with destructor-observable allocation blocked before unsafe native allocation.
- [x] Bounded public declared-object properties, methods, statics, constructors, named `instanceof`, and same-family aggregate equality.
- [x] Bounded `preg_replace_callback()` string-callback execution over supported slash-delimited patterns.
- [x] Object-property assignment/unset mutation for covered reference-backed operands through generated-C/native-link shared slot boundaries.
- [x] Shared offset-read source-result ABI for scalar/resource warning continuations, arrays, byte strings, references, and object-property offset-source composition.
- [x] Shared array-key value/reference-slot ABI for generated-native reference-backed variable operands and active symbol-table variable references.
- [x] Shared reference-slot type-name/type-predicate/int consumer ABI for runtime, LLVM, and generated C.
- [x] Shared reference-slot text-byte/text-membership ABI for runtime, LLVM, and generated C selected paths.
- [x] Shared reference-slot comparison ABI for covered runtime, LLVM, and generated-C native value comparison consumers.
- [x] Shared reference-slot truthiness ABI for covered runtime, LLVM, and generated-C native value truthiness consumers.
- [x] Shared request-backed ordinary array-key/RMW blocker classification for selected LLVM and generated-C consumers.

Primary-integrated non-executable infrastructure:

- [x] Object-offset `ArrayAccess` receiver diagnostic classifier for read, append-read, null-coalesce, `isset`, `empty`, and error-control forms.
- [x] Symbol-table ABI probe is pushed, but remains probe-only until real assignment/readback consumers land.

In progress but lane-local or not yet executable primary support:

- [ ] One string operation-family value/reference-slot split is the leading compact candidate if it avoids unrelated builtin breadth.
- [ ] Broader expression-family lvalue/reference-slot materialization is needed beyond variable-backed operands.
- [ ] Direct object `ArrayAccess` method dispatch remains blocked behind diagnostic-only classifier support.
- [ ] Alias-aware LLVM direct-root write-through after `=&` remains blocked for both statement assignment and assignment expressions.
- [ ] Object/resource source materialization for generic conversion sources remains blocked.
- [ ] LLVM offset-read error-status cleanup needs a generalized control-flow boundary.
- [ ] Callable-object and dynamic-constructor candidates need current-primary refresh before review.
- [ ] Function-frame, method-table, request-state, object visibility, cleanup, and diagnostic boundaries remain lane-local infrastructure.
- [ ] Binary-string, stream, PCRE, callback, pathinfo, filesystem, and broad internal-callback surfaces remain lane-local until extracted into compact semantic packets.

Not done:

- [ ] Executable request storage/writeback, `$GLOBALS` self-cells, request/global alias parity, request foreach, and mutation-during-iteration behavior.
- [ ] Full callable lookup and invocation, including non-string preg callbacks, closures, arrays, invokable objects, magic/visibility, and rebinding rules.
- [ ] Runtime `ArrayAccess` method dispatch for `offsetGet`, `offsetExists`, `offsetSet`, and `offsetUnset`.
- [ ] Full references/COW identity and arbitrary alias roots.
- [ ] Includes, variable variables, and dynamic symbol behavior.
- [ ] Full PCRE behavior beyond the bounded slash-delimited subset.
- [ ] Retirement or reframing of unrelated legacy WordPress-named preg/database recognizers behind generalized PHP semantic boundaries.
- [ ] General object model: non-public methods, overrides, interfaces/traits execution, magic methods, dynamic/static/typed properties, destructors.
- [ ] Complete cleanup/unwind/finally/destructor/output-buffer shutdown behavior.
- [ ] Exact/source-ordered diagnostics, custom handler execution, warning/error continuation, and suppression parity.
- [ ] LLVM/direct assembly parity for recent generated-C semantics.

## Recent Primary-Integrated Work

- `a501c4d1`: request-backed array-key/RMW blocker parity. Integrated files:
  `compiler/src/codegen.rs`, `compiler/tests/native_runtime_abi.rs`, and
  `compiler/tests/superglobals.rs`. Review/integration proof included exact
  hash/scope/apply checks, two nonzero focused gates, `cargo check`,
  `cargo fmt --check`, `git diff --check`, push proof, and clean post-push
  state. No `PROGRESS.md` edit was included in the semantic commit.
- `ccafa180`: progress-dashboard commit only. No executable compiler/runtime
  semantic code changed.
- `24ec4a10`: reference truthiness slots. Integrated files:
  `runtime/src/lib.rs`, `compiler/src/codegen.rs`,
  `compiler/tests/native_runtime_abi.rs`, and `compiler/tests/native_link.rs`.
  Focused runtime/IR/generated-C/link gates, exact one-test matches, `cargo
  check`, `cargo fmt --check`, `git diff --check`, apply/hash proof, and push
  proof passed.
- `146c2d64`: reference comparison slots. Integrated files:
  `runtime/src/lib.rs`, `compiler/src/codegen.rs`,
  `compiler/tests/native_runtime_abi.rs`, and `compiler/tests/native_link.rs`.
- `9f373b25`: reference text-membership slots. Integrated files:
  `runtime/src/lib.rs`, `compiler/src/codegen.rs`,
  `compiler/tests/native_runtime_abi.rs`, and `compiler/tests/native_link.rs`.
- `8f6266ce`: reference-slot type/int consumers. Integrated files:
  `runtime/src/lib.rs`, `compiler/src/codegen.rs`,
  `compiler/tests/native_runtime_abi.rs`, and `compiler/tests/native_link.rs`.
- `9022eb9e`: array-key value/reference-slot ABI. Integrated files:
  `runtime/src/lib.rs`, `compiler/src/codegen.rs`, and
  `compiler/tests/native_link.rs`.

## Current Work Snapshot

Primary-integrated:

- [x] Primary was clean and synced at `a501c4d1` before this `PROGRESS.md`
  edit.
- [x] Latest counted semantic/prerequisite commit is `a501c4d1`.
- [x] Request-backed ordinary array-key/RMW consumers now share blocker
  classification across selected LLVM and generated-C paths.
- [x] Reference-held truthiness, comparison, text-membership, type/int,
  array-key value/reference slots, and offset-read source-result support remain
  integrated for reviewed selected paths.
- [x] No uncommitted primary implementation diffs were present before this
  `PROGRESS.md` edit.

Lane-local:

- [ ] The request-backed prep/review/integration lane is consumed. Do not keep
  counting it as pending lane-local work.
- [ ] One string operation-family split is the best next compact packet if it
  stays to one semantic family and proves runtime plus backend consumers.
- [ ] A broader lvalue/reference-slot materializer remains important but needs
  a narrower extraction brief before primary review.
- [ ] Throw/source blocker work remains fallback blocker-only evidence; real
  exception execution is not present.
- [ ] `impl-native-type-conversion`, `impl-native-error-diagnostic-semantics`,
  `impl-function-frame-seed`, `impl-symbol-integrator`,
  `impl-array-linked-exec`, `impl-array-lowering`, and
  `impl-array-value-runtime` continue to contain useful but broad lane-local
  evidence. Extract compact packets only.
- [ ] Callable-object and dynamic-constructor candidates need refresh from
  `a501c4d1` before review.

Resource posture:

- `/dev/shm`: live df `40G` total, `24G` used, `17G` available; live `du`
  reports `24G`.
- `/home`: live df `459G` total, `202G` used, `238G` available; live `du -sh
  /home` reports about `130G` after permission-denied overlay warnings.
- Live memory has about `37Gi` available.
- Live swap remains high at `23Gi/29Gi`; use disk-backed target dirs,
  `umask 0007`, `CARGO_BUILD_JOBS=1`, `CARGO_INCREMENTAL=0`, and focused
  nonzero gates.

## Next Steering Read

The top-level queue should now move past request-backed array-key/RMW blocker
prep. The packet landed and should be treated as integrated blocker parity
only; it should not claim executable request storage, `$GLOBALS` self-cells,
request/global alias parity, request writeback, or request foreach.

Best next compact packets to consider:

- one string operation-family slot-consumer split, if it includes runtime/IR/
  generated-C proof and excludes adjacent builtin accretion;
- a broader lvalue/reference materializer prerequisite, if it stays focused on
  enabling shared slot ABIs rather than adding foreach/ArrayAccess execution;
- blocker-only throw/source boundary, if the first two cannot stay narrow;
- callable-object or dynamic-constructor refresh only after rebasing/reviewing
  from `a501c4d1`, and never combined as one integration.

Do not count lane-local triage, stopped dirty lanes, broad source-lane work,
stale May 24 candidates, callback/filesystem-family accumulation,
blocker-only metadata, or docs-only commits as product capability.
