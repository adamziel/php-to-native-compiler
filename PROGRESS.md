# PHP Native Compiler Progress

Updated: 2026-05-25 11:53 CEST
Evaluation marker: `20260525T095317Z`

Accounting rule: only generalized, tested, committed, and pushed primary work
counts as integrated capability. Dirty WIP, lane-local candidates, broad
worktree claims, probe-only commits, and dashboard-only commits are excluded.
Classifier/blocker commits are listed as integrated infrastructure, but they do
not count as executable PHP feature support unless they execute PHP semantics.

## Executive Read

Overall estimated progress: **86%** `[#################---]`

Executable PHP semantics: **86%** `[#################---]`

Primary gained one new semantic/prerequisite commit in this review window:
`9022eb9e native: add array key reference slot ABI`. Current pushed primary is
clean and aligned with `origin/master` at `9022eb9e`.

The new integrated surface is a shared runtime/codegen value/reference-slot ABI
for array-key materialization. It lets generated-native reference-backed
variable operands and active symbol-table variable references feed array-key
conversion through one checked slot boundary. This is real primary progress,
but it is still a narrow lvalue/reference slice, not full array-key,
reference/COW, object/resource/Stringable, or `ArrayAccess` parity.

Full generalized PHP remains blocked on callable/userland frame breadth,
references/COW identity, request and `$GLOBALS` parity, includes, variable
variables, full object semantics, real `ArrayAccess` dispatch, cleanup/unwind/
destructor/shutdown ordering, exact diagnostics/error handlers, and backend
parity.

## Current Primary State

- Current primary head before this dashboard edit:
  `9022eb9e native: add array key reference slot ABI`.
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
| Compiler/backend consumers | **98%** | `[####################]` | Generated C and LLVM consume many shared ABIs. Array-key slot routing is integrated for generated C; broader LLVM/backend parity remains incomplete. |
| Executable PHP semantics | **86%** | `[#################---]` | Primary has closure/callable/object islands, bounded preg callbacks, object-property reference-slot mutation, offset-read continuation proof, and reference-backed array-key conversion. Broad semantics remain incomplete. |
| Arrays, lvalues, references, COW | **72%** | `[##############------]` | Array-key value/reference-slot ABI is now integrated. Full COW, arbitrary roots, foreach, property references, broad expression reference slots, and alias composition remain open. |
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
| Broader lvalue/reference-slot materializer | **25%** `[#####---------------]` | **38%** `[########------------]` | Needed after `9022eb9e` so non-variable expression families that can carry references can enter shared array-key and consumer slot ABIs safely. |
| Reference-slot consumer families | **45%** `[#########-----------]` | **40%** `[########------------]` | Lane-local binary-string/reference work reports comparison, type-name, type-predicate, int conversion, scanner, and diagnostic surfaces. Extract one compact family at a time before primary review. |
| Object/resource source materialization for shared conversion sources | **25%** `[#####---------------]` | **30%** `[######--------------]` | Explicit blocker left by the offset-read ABI. Needs a general value reconstruction boundary before generic object/resource consumers are safe. |
| LLVM offset-read error-status cleanup | **25%** `[#####---------------]` | **30%** `[######--------------]` | Offset-read diagnostics exist, but LLVM still needs a generalized control-flow/error-exit status boundary for failed conversion results. |
| Static-property comparison operand ABI | **35%** `[#######-------------]` | **37%** `[#######-------------]` | Prior extraction says `needs-split`: source lane is too broad and entangled; candidate remained empty. Split metadata/operand prerequisites first. |
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

- [ ] Text-membership/reference text-byte conversion was explicitly excluded
  from the array-key slot packet and still needs separate proof.
- [ ] Direct object `ArrayAccess` method dispatch remains blocked behind
  diagnostic-only classifier support.
- [ ] Broader expression-family lvalue/reference-slot materialization is
  needed beyond variable-backed operands.
- [ ] Reference-slot comparison, type-name, type-predicate, int conversion,
  scanner, and diagnostic consumers remain lane-local.
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
- [ ] Binary-string scanner, text-byte slot, error-handler dispatch, stream,
  and class-alias surfaces remain lane-local.
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

- `9022eb9e`: array-key value/reference-slot ABI. Integrated files:
  `runtime/src/lib.rs`, `compiler/src/codegen.rs`, and
  `compiler/tests/native_link.rs`. Focused runtime, generated-C source,
  linked executable, `cargo check`, rustfmt, and diff hygiene gates passed.
- `23f110c3`: progress-dashboard commit only. No executable compiler/runtime
  semantic code changed.
- `de93017d`: progress-dashboard commit only. No executable compiler/runtime
  semantic code changed.
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

- [x] Primary is clean and synced at `9022eb9e`.
- [x] Latest counted semantic/prerequisite commit is `9022eb9e`.
- [x] Array-key value/reference-slot support is integrated for the reviewed
  generated-native variable/reference-backed operand family.
- [x] Offset-read source/result and object-property reference-slot mutation
  remain the adjacent integrated reference/lvalue prerequisites.
- [x] Overall and executable-semantics estimates remain **86%** because the new
  slice is important but too narrow to move the rounded whole-project number.

Best next candidate supply:

- [ ] Extract a compact post-`9022eb9e` lvalue/reference-slot materializer that
  widens array-key/reference-slot ingress beyond variable-backed operands.
- [ ] Extract one small reference-slot consumer family from
  `impl-binary-string-runtime` only if exact files, hash, apply proof, and
  nonzero runtime/codegen/link gates can be shown.
- [ ] Keep text-membership/reference text-byte conversion separate from
  array-key materialization.
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

Resource pressure is usable but guarded. Current `df` shows `/dev/shm` at
`40G` total, `24G` used, `17G` available (`58%`) and the filesystem backing
`/home` at `459G` total, `236G` used, `205G` available (`54%`). `du -sh
/dev/shm` reports `24G`. Raw `du -sh /home` hit overlay permission warnings
and timed out in this review; `du -sh --exclude=/home/claude/.local/share/containers/storage/overlay /home`
reports `124G`. Memory has about `39Gi` available, but swap remains high at
`23Gi` used of `29Gi`.

Advisory steering read: the top-level supervisor should consider treating
`9022eb9e` as the new integrated baseline, then route the next candidate
through the same small-packet flow that worked here: exact file scope, stable
binary hash, independent primary review, apply proof against current primary,
focused nonzero runtime/codegen/link gates, and a dedicated integrator. Avoid
importing broad binary-string, text-byte, request, callable, or cleanup surfaces
as collateral.
