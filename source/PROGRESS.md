# PHP Native Compiler Progress

Updated: 2026-05-25 11:18 CEST
Evaluation marker: `20260525T091858Z`

Accounting rule: only generalized, tested, committed, and pushed primary work
counts as integrated capability. Dirty WIP, lane-local candidates, broad
worktree claims, probe-only commits, and dashboard-only commits are excluded.
Classifier/blocker commits are listed as integrated infrastructure, but they do
not count as executable PHP feature support unless they execute PHP semantics.

## Executive Read

Overall estimated progress: **86%** `[#################---]`

Executable PHP semantics: **86%** `[#################---]`

Primary advanced this review window. The latest pushed primary head is
`cc7efc2d native: add offset read source result ABI`, clean and aligned with
`origin/master`. This lands the scalar/resource offset-read source-result
prerequisite that was previously only a reviewed candidate.

The new ABI is real progress because generated C/LLVM and runtime offset-read
consumers now share a source/result boundary for arrays, byte strings,
scalar/null/resource warning continuations, references, and object-property
offset-source composition. It is still a prerequisite, not full PHP offset or
`ArrayAccess` semantics.

Full generalized PHP remains blocked on callable/userland frame breadth,
references/COW identity, request and `$GLOBALS` parity, includes, variable
variables, full object semantics, real `ArrayAccess` dispatch, cleanup/unwind/
destructor/shutdown ordering, exact diagnostics, and backend parity.

## Current Primary State

- Current primary head before this dashboard edit:
  `cc7efc2d native: add offset read source result ABI`.
- Primary sync at evaluation verification: clean and aligned with
  `origin/master`.
- Latest integrated executable/prerequisite semantic baseline:
  `cc7efc2d native: add offset read source result ABI`.
- Prior integrated executable feature baseline:
  `bfbc62c4 native: route object property reference slots`.
- Latest integrated non-executable classifier:
  `deaf52ca codegen: classify object ArrayAccess receivers`.
- Current read:
  the scalar/resource offset-read source-result ABI is now primary-integrated
  and pushed. Direct object `ArrayAccess`, object/resource source
  materialization, full reference/COW behavior, and LLVM error-status cleanup
  remain open.

## Grand Roadmap Position

| Workstream | Estimate | Bar | Current read |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | **98%** | `[####################]` | Strong selected-path value, array, string, diagnostic, reference, symbol, call-frame, object, comparison, conversion, owner-cell, request-state, and now offset-read source/result surfaces. |
| Compiler/backend consumers | **98%** | `[####################]` | Generated C and LLVM consume many shared ABIs. Offset-read source-result calls landed; backend parity and LLVM error-status behavior remain incomplete. |
| Executable PHP semantics | **86%** | `[#################---]` | Primary has closure/callable/object islands, bounded preg callbacks, object-property reference-slot mutation, and offset-read source-result continuation proof. Broad semantics remain incomplete. |
| Arrays, lvalues, references, COW | **70%** | `[##############------]` | Offset-read source-result ABI and reference-backed object-property mutation are integrated. Full COW, arbitrary roots, foreach, property references, array-key slots, and alias composition remain open. |
| Symbols, globals, request state | **72%** | `[##############------]` | Selected function globals and root-symbol surfaces exist. `$GLOBALS` self-cells, request/global alias parity, includes, variable variables, and exact unset/global behavior remain incomplete. |
| Calls, functions, frames | **82%** | `[################----]` | Bounded closures, callable arrays/objects, public method frames, and constructors exist in selected paths. Named/unpacked/by-reference/userland frame breadth remains incomplete. |
| Objects, properties, methods | **50%** | `[##########----------]` | Object-property reference-slot mutation and diagnostic classifiers are integrated. Full visibility, magic, dynamic/static/typed properties, destructors, references/COW, and `ArrayAccess` execution remain open. |
| Control flow, cleanup, diagnostics | **50%** | `[##########----------]` | Selected branches, loops, transfers, finalizers, output buffers, and diagnostics exist. Broad unwind/finally/destructor/shutdown and exact source ordering remain open. |
| Broad integrated verification | **85%** | `[#################---]` | Focused gates are strong. Broad gates remain constrained by lane extraction cost, high swap, and backend parity gaps. |

## Active Roadmap Items

| Item | Toward Primary Integration | Toward Full Feature | Status |
| --- | ---: | ---: | --- |
| Scalar/resource offset-read source-result prerequisite | **100%** `[####################]` | **44%** `[#########-----------]` | Integrated at `cc7efc2d`. Adds shared runtime/codegen source-result ABI and executable continuation proof. Direct object `ArrayAccess`, object/resource materialization, and LLVM error-status cleanup remain open. |
| Object-property reference-slot mutation | **100%** `[####################]` | **39%** `[########------------]` | Integrated at `bfbc62c4`. Executable generated-C/native-link support for covered assignment/unset mutation operands. Full object/property/reference semantics remain open. |
| Bounded `preg_replace_callback()` string callbacks | **100%** `[####################]` | **32%** `[######--------------]` | Integrated at `6aca392d`. Full PCRE, broader captures/modifiers, non-string callables, `limit`/`count`/`flags`, and legacy recognizer cleanup remain open. |
| Object-offset `ArrayAccess` diagnostic classifier | **100%** `[####################]` | **12%** `[##------------------]` | Integrated at `deaf52ca`. Diagnostic routing only; no `offsetGet`, `offsetExists`, `offsetSet`, or `offsetUnset` execution. |
| Array-key value/reference-slot ABI | **60%** `[############--------]` | **38%** `[########------------]` | Lane-local generated-C/runtime packet has compact proof and is now the strongest follow-up candidate if freshly extracted against `cc7efc2d`. |
| Object/resource source materialization for shared conversion sources | **25%** `[#####---------------]` | **30%** `[######--------------]` | Explicit blocker left by the new offset-read ABI. Needs a general value reconstruction boundary before generic object/resource consumers are safe. |
| LLVM offset-read error-status cleanup | **25%** `[#####---------------]` | **30%** `[######--------------]` | New ABI reports diagnostics, but LLVM still needs a generalized control-flow/error-exit status boundary for failed conversion results. |
| Static-property comparison operand ABI | **35%** `[#######-------------]` | **37%** `[#######-------------]` | Prior extraction says `needs-split`: source lane is too broad and entangled; candidate remained empty. Split metadata/operand prerequisites first. |
| Callable/dynamic constructor candidates | **65%** `[#############-------]` | **42%** `[########------------]` | May 24 lane-local candidates look useful but are stale relative to `cc7efc2d`; refresh before any primary review. |
| Diagnostics, request, and cleanup boundaries | **59%** `[############--------]` | **39%** `[########------------]` | Lane-local request handle, writeback, branch cleanup, try/catch/finally preflight, and result-boundary work is useful infrastructure. Exact Zend ordering and real handler/exceptions execution remain open. |
| Broad lane extraction backlog | **32%** `[######--------------]` | **32%** `[######--------------]` | Recent failed extractions show the backlog is real. Treat lanes as sources for compact prerequisite or consumer packets, not integration units. |

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

Primary-integrated non-executable infrastructure:

- [x] Object-offset `ArrayAccess` receiver diagnostic classifier for read,
  append-read, null-coalesce, `isset`, `empty`, and error-control forms.
- [x] Symbol-table ABI probe is pushed, but remains probe-only until real
  assignment/readback consumers land.

In progress but lane-local or not yet executable primary support:

- [ ] Array-key value/reference-slot ABI needs fresh current-primary extraction
  after `cc7efc2d`.
- [ ] Direct object `ArrayAccess` method dispatch remains blocked behind
  diagnostic-only classifier support.
- [ ] Object/resource source materialization for generic conversion sources
  remains blocked.
- [ ] LLVM offset-read error-status cleanup needs a generalized control-flow
  boundary.
- [ ] Static-property comparison operands need a smaller prerequisite split
  before primary review.
- [ ] Callable-object/dynamic-constructor candidates need current-primary
  refresh before review.
- [ ] Strict identity, LLVM bitwise, string predicate callback dispatch,
  handler/autoload blockers, and date/time blockers remain lane-local.
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

- `cc7efc2d`: scalar/resource offset-read source-result ABI. Integrated files:
  `runtime/src/lib.rs`, `compiler/src/codegen.rs`, and
  `compiler/tests/native_runtime_abi.rs`. Focused runtime, generated-source,
  executable continuation, object-property composition, ArrayAccess rejection,
  `cargo check`, diff hygiene, and rustfmt gates passed.
- `eea2c2a1`: progress-dashboard commit only. No executable compiler/runtime
  semantic code changed.
- `62b78f18`: progress-dashboard commit only. No executable compiler/runtime
  semantic code changed.
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

- [x] Primary is clean and synced at `cc7efc2d`.
- [x] Scalar/resource offset-read source-result ABI is now integrated and
  counted.
- [x] Object-property reference-slot mutation remains the latest focused
  object/reference executable feature.
- [x] Object-offset `ArrayAccess` receiver classifier remains integrated as
  diagnostic infrastructure only.
- [x] Overall and executable-semantics estimates remain **86%** because the new
  commit improves a prerequisite boundary but does not close a major remaining
  semantic cliff by itself.

Best next candidate supply:

- [ ] `impl-native-integration-batch`: array-key value/reference-slot ABI has
  compact lane-local executable proof and is the recommended next extraction
  target if it applies cleanly to `cc7efc2d`.
- [ ] Direct `ArrayAccess` execution should not be counted until real
  `offsetGet`/`offsetExists`/`offsetSet`/`offsetUnset` dispatch lands with
  reference/COW and diagnostic proof.
- [ ] `static-property-comparison-extract`: `needs-split`; do not integrate
  the current broad lane hunk as-is.
- [ ] Callable-object and dynamic-constructor candidate packets need a fresh
  current-primary review because they predate `cc7efc2d`.
- [ ] Broad symbol/control-flow/diagnostic/reference/binary-string/type-
  conversion lanes continue producing useful surfaces that are not yet product
  capability.

## Review Notes

Resource pressure is usable but guarded. `/dev/shm` is `40G` total, `24G`
used, `17G` available (`58%`), and `du -sh /dev/shm` reports `24G`. The
filesystem backing `/home` is `459G` total, `245G` used, `196G` available
(`56%`) by `df`; `du -sh /home` reports `126G` but exits with overlay
permission warnings. Memory has about `38Gi` available, but swap remains high
at `23Gi` used of `29Gi`.

Advisory steering read: update supervisor state around `cc7efc2d`, then move
from the landed offset-read prerequisite to one compact executable consumer.
The best next target on current evidence is a fresh current-primary
array-key value/reference-slot ABI extraction. Avoid importing broad lanes or
counting diagnostic-only `ArrayAccess` support as runtime method dispatch.
