# PHP Native Compiler Progress

Updated: 2026-05-25 12:55 CEST
Evaluation marker: `20260525T105552Z`

Accounting rule: only generalized, tested, committed, and pushed primary work
counts as integrated capability. Dirty WIP, lane-local candidates, broad
worktree claims, probe-only commits, and dashboard-only commits are excluded.

## Executive Read

Overall estimated progress: **86%** `[#################---]`

Executable PHP semantics: **86%** `[#################---]`

Primary is clean and aligned with `origin/master` at
`ecf3ba12 docs: update progress dashboard` before this dashboard edit. No new
compiler/runtime semantic commit landed after
`9022eb9e native: add array key reference slot ABI`, so the integrated product
percentage does not move.

The latest integrated capability is still the shared runtime/codegen
value/reference-slot ABI for array-key materialization. It lets generated-native
reference-backed variable operands and active symbol-table variable references
feed array-key conversion through one checked slot boundary. This is real
primary progress, but it is not full array-key, reference/COW,
object/resource/Stringable, or `ArrayAccess` parity.

Current candidate supply improved: the repaired reference-slot type/int
consumer candidate has independent review approval for primary-integrator
handoff with exact hash
`5b5aedd6863ee1ee210dedd3c449d3e669f7479d0f77a3862e114b217f1cb5c7`. It is
still lane-local until primary applies it, gates it, commits it, and pushes it.

Full generalized PHP remains blocked on callable/userland frame breadth,
references/COW identity, request and `$GLOBALS` parity, includes, variable
variables, full object semantics, real `ArrayAccess` dispatch, cleanup/unwind/
destructor/shutdown ordering, exact diagnostics/error handlers, and backend
parity.

## Current Primary State

- Current primary head before this dashboard edit:
  `ecf3ba12 docs: update progress dashboard`.
- Primary sync at evaluation verification: clean and aligned with
  `origin/master`.
- Latest integrated executable/prerequisite semantic baseline:
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
| Runtime and ABI foundations | **98%** | `[####################]` | Strong selected-path value, array, string, diagnostic, reference, symbol, call-frame, object, comparison, conversion, owner-cell, request-state, offset-read, and array-key slot surfaces. |
| Compiler/backend consumers | **98%** | `[####################]` | Generated C and LLVM consume many shared ABIs. Recent array-key slot routing is integrated for generated C; broader LLVM/backend parity remains incomplete. |
| Executable PHP semantics | **86%** | `[#################---]` | Primary has closure/callable/object islands, bounded preg callbacks, object-property reference-slot mutation, offset-read continuation proof, and reference-backed array-key conversion. Broad semantics remain incomplete. |
| Arrays, lvalues, references, COW | **72%** | `[##############------]` | Array-key value/reference-slot ABI is integrated. Full COW, arbitrary roots, foreach, property references, broad expression reference slots, and alias composition remain open. |
| Symbols, globals, request state | **72%** | `[##############------]` | Selected function globals, root-symbol surfaces, and active symbol-table reference consumers exist. `$GLOBALS` self-cells, request/global alias parity, includes, variable variables, and exact unset/global behavior remain incomplete. |
| Calls, functions, frames | **82%** | `[################----]` | Bounded closures, callable arrays/objects, public method frames, and constructors exist in selected paths. Named/unpacked/by-reference/userland frame breadth remains incomplete. |
| Objects, properties, methods | **50%** | `[##########----------]` | Object-property reference-slot mutation and diagnostic classifiers are integrated. Full visibility, magic, dynamic/static/typed properties, destructors, references/COW, and `ArrayAccess` execution remain open. |
| Control flow, cleanup, diagnostics | **50%** | `[##########----------]` | Selected branches, loops, transfers, finalizers, output buffers, and diagnostics exist. Broad unwind/finally/destructor/shutdown and exact source ordering remain open. |
| Broad integrated verification | **85%** | `[#################---]` | Focused gates are strong. Broad gates remain constrained by lane extraction cost, high swap, stale lane expectations, and backend parity gaps. |

## Active Roadmap Items

| Item | Toward Primary Integration | Toward Full Feature | Status |
| --- | ---: | ---: | --- |
| Array-key value/reference-slot ABI | **100%** `[####################]` | **42%** `[########------------]` | Integrated at `9022eb9e`. Adds shared checked slot ABI for value/reference-backed array-key operands with runtime, generated-C source, and linked executable proof. Broader expression lvalues, COW identity, object/resource/Stringable keys, and direct `ArrayAccess` remain open. |
| Scalar/resource offset-read source-result prerequisite | **100%** `[####################]` | **44%** `[#########-----------]` | Integrated at `cc7efc2d`. Adds shared runtime/codegen source-result ABI and executable continuation proof. Direct object `ArrayAccess`, object/resource materialization, and LLVM error-status cleanup remain open. |
| Object-property reference-slot mutation | **100%** `[####################]` | **39%** `[########------------]` | Integrated at `bfbc62c4`. Executable generated-C/native-link support for covered assignment/unset mutation operands. Full object/property/reference semantics remain open. |
| Bounded `preg_replace_callback()` string callbacks | **100%** `[####################]` | **32%** `[######--------------]` | Integrated at `6aca392d`. Full PCRE, broader captures/modifiers, non-string callables, `limit`/`count`/`flags`, and legacy recognizer cleanup remain open. |
| Object-offset `ArrayAccess` diagnostic classifier | **100%** `[####################]` | **12%** `[##------------------]` | Integrated at `deaf52ca`. Diagnostic routing only; no `offsetGet`, `offsetExists`, `offsetSet`, or `offsetUnset` execution. |
| Reference-slot type/int consumer packet | **90%** `[##################--]` | **43%** `[#########-----------]` | Reviewed `go-for-primary-integrator` with exact four-file hash and nonzero gates. Still not product capability until applied, gated, committed, and pushed in primary. |
| String-array/string-position/string-result consumer prerequisite | **25%** `[#####---------------]` | **35%** `[#######-------------]` | Useful lane-local work exists, but it must remain separate from the type/int packet and needs compact current-primary extraction. |
| Text-membership/reference text-byte conversion | **25%** `[#####---------------]` | **34%** `[#######-------------]` | Explicitly excluded from the array-key and type/int packets. Needs separate current-primary proof. |
| Broader lvalue/reference-slot materializer | **25%** `[#####---------------]` | **38%** `[########------------]` | Needed after `9022eb9e` so non-variable expression families that can carry references can enter shared array-key and consumer slot ABIs safely. |
| Object/resource source materialization for shared conversion sources | **25%** `[#####---------------]` | **30%** `[######--------------]` | Explicit blocker left by the offset-read ABI. Needs a general value reconstruction boundary before generic object/resource consumers are safe. |
| LLVM offset-read/error-status cleanup | **25%** `[#####---------------]` | **30%** `[######--------------]` | Offset-read diagnostics exist, but LLVM still needs a generalized control-flow/error-exit status boundary for failed conversion results. |
| Static-property comparison operand ABI | **35%** `[#######-------------]` | **37%** `[#######-------------]` | Prior extraction says `needs-split`: source lane is too broad and entangled. Split metadata/operand prerequisites first. |
| Callable/dynamic constructor candidates | **60%** `[############--------]` | **42%** `[########------------]` | May 24 lane-local candidates look useful but are stale relative to current primary `9022eb9e`; refresh before any primary review. |
| Diagnostics, request, and cleanup boundaries | **60%** `[############--------]` | **40%** `[########------------]` | Lane-local request handle, writeback, branch cleanup, try/catch/finally preflight, stateful-call cleanup, and result-boundary work is useful infrastructure. Exact Zend ordering and real handler/exceptions execution remain open. |
| Broad lane extraction backlog | **32%** `[######--------------]` | **32%** `[######--------------]` | Broad dirty lanes continue producing useful surfaces, but many are blocker/preflight-only and not directly integrable. Treat lanes as packet sources, not integration units. |

## Done / In Progress / Not Done

Primary-integrated executable or executable-prerequisite capability:

- [x] Descriptor-backed by-value closure invocation.
- [x] Direct by-value closure captures and non-static arrow implicit captures.
- [x] Untyped by-reference descriptor closure parameters.
- [x] Supported root/reference and promoted frame-local by-reference captures.
- [x] Typed/default/variadic by-value descriptor closure parameters.
- [x] Static anonymous descriptor closures and static arrow closures.
- [x] Non-static closure `$this` binding inside active object frames.
- [x] Callable-array invocation for supported public static/object method frames.
- [x] Callable-object invocation through supported public `__invoke` frames.
- [x] Runtime string-valued declared-class `new` for selected declared classes.
- [x] Destructor-observable declared-class allocation is blocked before unsafe generated-C native allocation.
- [x] Bounded public declared-object properties, methods, statics, constructors, named `instanceof`, and same-family aggregate equality.
- [x] Bounded `preg_replace_callback()` string-callback execution over supported slash-delimited patterns.
- [x] Object-property assignment/unset mutation for covered reference-backed operands through generated-C/native-link shared slot boundaries.
- [x] Shared offset-read source-result ABI for scalar/resource warning continuations, arrays, byte strings, references, and object-property offset-source composition.
- [x] Shared array-key value/reference-slot ABI for generated-native reference-backed variable operands and active symbol-table variable references.

Primary-integrated non-executable infrastructure:

- [x] Object-offset `ArrayAccess` receiver diagnostic classifier for read,
  append-read, null-coalesce, `isset`, `empty`, and error-control forms.
- [x] Symbol-table ABI probe is pushed, but remains probe-only until real
  assignment/readback consumers land.

In progress but lane-local or not yet executable primary support:

- [ ] Reference-held type-name/type-predicate/int conversion slot consumer
  candidate is reviewed for primary-integrator handoff, but not integrated.
- [ ] String-array/string-position/string-result consumer family must be split
  from the type/int packet and proven separately.
- [ ] Text-membership/reference text-byte conversion still needs separate
  proof.
- [ ] Direct object `ArrayAccess` method dispatch remains blocked behind
  diagnostic-only classifier support.
- [ ] Broader expression-family lvalue/reference-slot materialization is
  needed beyond variable-backed operands.
- [ ] Reference-slot comparison, scanner, and diagnostic consumers remain
  lane-local.
- [ ] Object/resource source materialization for generic conversion sources
  remains blocked.
- [ ] LLVM offset-read error-status cleanup needs a generalized control-flow
  boundary.
- [ ] Static-property comparison operands need a smaller prerequisite split
  before primary review.
- [ ] Callable-object and dynamic-constructor candidates need current-primary
  refresh before review.
- [ ] Function-frame signature-table blockers, method-table lookup blockers,
  and object string/Countable preflight work remain lane-local.
- [ ] Symbol/control-flow try/catch/finally preflight and rejecting-statement
  result boundaries remain lane-local infrastructure.
- [ ] Request-backed throw/clone/instanceof blockers and diagnostic
  writeback/selection boundaries remain lane-local.
- [ ] Binary-string scanner, error-handler dispatch, stream, and class-alias
  surfaces remain lane-local.
- [ ] Control-flow loop/switch/goto cleanup-state advances remain lane-local.

Not done:

- [ ] Full callable lookup and invocation, including non-string preg callbacks,
  closures, arrays, invokable objects, magic/visibility, and rebinding rules.
- [ ] Runtime `ArrayAccess` method dispatch for `offsetGet`, `offsetExists`,
  `offsetSet`, and `offsetUnset`.
- [ ] Full references/COW identity and arbitrary alias roots.
- [ ] Request and `$GLOBALS` parity, includes, variable variables, and dynamic
  symbol behavior.
- [ ] Full PCRE behavior beyond the bounded slash-delimited subset.
- [ ] Retirement or reframing of unrelated legacy WordPress-named preg/database
  recognizers behind generalized PHP semantic boundaries.
- [ ] General object model: non-public methods, overrides, interfaces/traits
  execution, magic methods, dynamic/static/typed properties, destructors.
- [ ] Complete cleanup/unwind/finally/destructor/output-buffer shutdown behavior.
- [ ] Exact/source-ordered diagnostics, custom handler execution,
  warning/error continuation, and suppression parity.
- [ ] LLVM/direct assembly parity for recent generated-C semantics.

## Recent Primary-Integrated Work

- `ecf3ba12`: progress-dashboard commit only. No executable compiler/runtime
  semantic code changed.
- `28016b44`: progress-dashboard commit only. No executable compiler/runtime
  semantic code changed.
- `7b476083`: progress-dashboard commit only. No executable compiler/runtime
  semantic code changed.
- `9022eb9e`: array-key value/reference-slot ABI. Integrated files:
  `runtime/src/lib.rs`, `compiler/src/codegen.rs`, and
  `compiler/tests/native_link.rs`. Focused runtime, generated-C source,
  linked executable, `cargo check`, rustfmt, and diff hygiene gates passed.
- `cc7efc2d`: scalar/resource offset-read source-result ABI. Integrated files:
  `runtime/src/lib.rs`, `compiler/src/codegen.rs`, and
  `compiler/tests/native_runtime_abi.rs`. Focused runtime, generated-source,
  executable continuation, object-property composition, ArrayAccess rejection,
  `cargo check`, diff hygiene, and rustfmt gates passed.
- `bfbc62c4`: generated-C/native-link object-property assignment and unset
  mutation route subject, property, and replacement operands through shared
  value-or-reference slot handling with runtime dereference boundaries.
- `deaf52ca`: compiler classifies unsupported object-offset `ArrayAccess`
  receiver diagnostics through a shared operation result across covered
  read/probe/error-control families. This is not executable `ArrayAccess`.
- `6aca392d`: interpreter `preg_replace_callback()` executes supported string
  callbacks over a bounded slash-delimited pattern subset.

## Current Work Snapshot

Primary-integrated:

- [x] Primary is clean and synced at `ecf3ba12` before this dashboard edit.
- [x] Latest counted semantic/prerequisite commit remains `9022eb9e`.
- [x] Array-key value/reference-slot support is integrated for the reviewed
  generated-native variable/reference-backed operand family.
- [x] Offset-read source/result and object-property reference-slot mutation
  remain the adjacent integrated reference/lvalue prerequisites.
- [x] Overall and executable-semantics estimates remain **86%** because no new
  semantic packet landed after the previous marker.

Best next candidate supply:

- [ ] Consider launching the reviewed reference-slot type/int consumer through
  a dedicated primary-integrator pass, using exact hash
  `5b5aedd6863ee1ee210dedd3c449d3e669f7479d0f77a3862e114b217f1cb5c7`.
- [ ] Keep that packet limited to reference-held type-name/type-predicate/int
  consumers and the centralized LLVM post-alias write-through rejection.
- [ ] Reject or split any string-array/text-byte, scanner, comparison,
  request/SAPI, parser/lexer/AST, fixture, or broad reference/COW collateral.
- [ ] If type/int integration fails, pivot to a compact post-`9022eb9e`
  lvalue/reference-slot materializer that widens array-key/reference-slot
  ingress beyond variable-backed operands.
- [ ] Direct `ArrayAccess` execution should not be counted until real
  `offsetGet`/`offsetExists`/`offsetSet`/`offsetUnset` dispatch lands with
  reference/COW and diagnostic proof.
- [ ] `static-property-comparison-extract`: `needs-split`; do not integrate
  the current broad lane hunk as-is.
- [ ] Callable-object and dynamic-constructor candidate packets need a fresh
  current-primary review because they predate `9022eb9e`.
- [ ] Broad symbol/control-flow/diagnostic/reference/binary-string/type-
  conversion lanes continue producing useful surfaces that are not yet product
  capability.

## Review Notes

Resource pressure is usable but guarded. Live verification for this review
showed `/dev/shm` at `40G` total, `24G` used, `17G` available (`58%`) and the
filesystem backing `/home` at `459G` total, `187G` used, `254G` available
(`43%`). `du -sh /dev/shm` reports `24G`; `du -sh /home` reports `127G` with
container-overlay permission warnings. Memory has about `38Gi` available, but
swap remains high at `23Gi` used of `29Gi`.

Advisory steering read: the top-level supervisor should consider the reviewed
reference-slot type/int consumer as the next primary-integrator handoff, at its
discretion, but only with strict current-primary recovery, exact hash/file
guards, focused nonzero gates, and no broad lane imports. Keep counting
lane-local work as candidate supply until it is integrated, committed, and
pushed in primary.
