# PHP Native Compiler Progress

Updated: 2026-05-25 11:03 CEST
Evaluation marker: `20260525T090330Z`

Accounting rule: only generalized, tested, committed, and pushed primary work
counts as integrated capability. Dirty WIP, lane-local candidates, broad
worktree claims, probe-only commits, and dashboard-only commits are excluded.
Classifier/blocker commits are listed as integrated infrastructure, but they do
not count as executable PHP feature support unless they execute PHP semantics.

## Current Primary State

- Primary head before this dashboard edit:
  `62b78f18 docs: update progress dashboard`.
- Primary sync at evaluation start: clean and aligned with `origin/master`.
- Latest integrated executable semantic baseline:
  `bfbc62c4 native: route object property reference slots`.
- Latest integrated non-executable classifier:
  `deaf52ca codegen: classify object ArrayAccess receivers`.
- Current evaluation read:
  no executable semantic code landed after `bfbc62c4`. The best current
  movement is a reviewed scalar/resource offset-read source-result prerequisite
  candidate. It is `go-for-primary-integrator`, but it is not counted until a
  dedicated integrator applies, gates, commits, and pushes it on primary.

## Executive Read

Overall estimated progress: **86%** `[#################---]`

Executable PHP semantics: **86%** `[#################---]`

Primary still has a focused executable object-property/reference-slot step, not
full object/reference semantics. Covered object-property assignment and unset
mutation route operands through shared value-or-reference slot handling before
runtime dereference, validation, mutation, diagnostics, and cleanup.

This is meaningful product capability, but full generalized PHP remains blocked
on callable/userland frames, references/COW identity, request and `$GLOBALS`
parity, includes, variable variables, full object semantics, real `ArrayAccess`
dispatch, cleanup/unwind/destructor/shutdown ordering, exact diagnostics, and
backend parity.

The freshest movement is candidate readiness, not integrated product support.
The scalar/resource prerequisite packet now has candidate prep plus independent
primary review, with a concrete hash, current-primary apply proof, and nonzero
focused gates. Count it as the next primary-integration opportunity, not as
landed capability.

## Grand Roadmap Position

| Workstream | Estimate | Bar | Current read |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | **97%** | `[###################-]` | Strong selected-path value, array, string, diagnostic, reference, symbol, call-frame, object, comparison, conversion, owner-cell, and request-state surfaces. Candidate offset-read source-result ABI is reviewed but not integrated. |
| Compiler/backend consumers | **98%** | `[####################]` | Generated C and LLVM consume many shared ABIs. Recent executable primary progress is generated-C/native-link object-property reference-slot mutation; broad parity remains incomplete. |
| Executable PHP semantics | **86%** | `[#################---]` | Primary has closure/callable/object islands, bounded preg callbacks, and focused object-property reference-slot mutation. No new executable semantic commit landed in this review window. |
| Arrays, lvalues, references, COW | **69%** | `[##############------]` | Reference-backed object-property mutation is integrated. Offset-read source-result work is reviewed for integration but not counted yet. Full COW, arbitrary roots, foreach, property references, and alias composition remain open. |
| Symbols, globals, request state | **72%** | `[##############------]` | Selected function globals and root-symbol surfaces exist. `$GLOBALS` self-cells, request/global alias parity, includes, variable variables, and exact unset/global behavior remain incomplete. |
| Calls, functions, frames | **82%** | `[################----]` | Bounded closures, callable arrays/objects, public method frames, and constructors exist in selected paths. Named/unpacked/by-reference/userland frame breadth remains incomplete. |
| Objects, properties, methods | **50%** | `[##########----------]` | Object-property reference-slot mutation and diagnostic classifiers are integrated. Full visibility, magic, dynamic/static/typed properties, destructors, references/COW, and `ArrayAccess` execution remain open. |
| Control flow, cleanup, diagnostics | **50%** | `[##########----------]` | Selected branches, loops, transfers, finalizers, output buffers, and diagnostics exist. Broad unwind/finally/destructor/shutdown and exact source ordering remain open. |
| Broad integrated verification | **85%** | `[#################---]` | Focused gates are strong. Broad gates remain constrained by lane extraction cost, high swap, and backend parity gaps. |

## Active Roadmap Items

| Item | Toward Primary Integration | Toward Full Feature | Status |
| --- | ---: | ---: | --- |
| Object-property reference-slot mutation | **100%** `[####################]` | **39%** `[########------------]` | Integrated at `bfbc62c4`. Executable generated-C/native-link support for covered assignment/unset mutation operands. Full object/property/reference semantics remain open. |
| Bounded `preg_replace_callback()` string callbacks | **100%** `[####################]` | **32%** `[######--------------]` | Integrated at `6aca392d`. Full PCRE, broader captures/modifiers, non-string callables, `limit`/`count`/`flags`, and legacy recognizer cleanup remain open. |
| Object-offset `ArrayAccess` diagnostic classifier | **100%** `[####################]` | **12%** `[##------------------]` | Integrated at `deaf52ca`. Diagnostic routing only; no `offsetGet`, `offsetExists`, `offsetSet`, or `offsetUnset` execution. |
| Scalar/resource offset-read source-result prerequisite | **86%** `[#################---]` | **42%** `[########------------]` | Candidate is ready and independently reviewed as `go-for-primary-integrator`. Hash `5694f62a...058d3`, files `runtime/src/lib.rs`, `compiler/src/codegen.rs`, `compiler/tests/native_runtime_abi.rs`. Awaiting dedicated primary integration and rerun gates. |
| Static-property comparison operand ABI | **35%** `[#######-------------]` | **37%** `[#######-------------]` | Prior extraction says `needs-split`: source lane is too broad and entangled; candidate remained empty. Split metadata/operand prerequisites first. |
| Array-key value/reference-slot ABI | **60%** `[############--------]` | **38%** `[########------------]` | Lane-local generated-C/runtime packet has compact proof and is the recommended fallback if scalar/resource integration fails. Needs fresh current-primary extraction. |
| Strict identity and bitwise shared value ABI consolidation | **54%** `[###########---------]` | **37%** `[#######-------------]` | Lane-local integration-batch work routes more backend operations through shared value ABIs. Useful, but not extracted to primary and generated-C/LLVM parity remains incomplete. |
| Post-spread call argument planning | **48%** `[##########----------]` | **42%** `[########------------]` | Lane-local call planner evidence is focused but high sensitivity; extract only with current-primary hash/apply proof and narrow gates. |
| Diagnostics, request, and cleanup boundaries | **59%** `[############--------]` | **39%** `[########------------]` | Lane-local request handle, writeback, branch cleanup, try/catch/finally preflight, and result-boundary work is useful infrastructure. Exact Zend ordering and real handler/exceptions execution remain open. |
| Broad lane extraction backlog | **32%** `[######--------------]` | **32%** `[######--------------]` | Recent failed extractions show the backlog is real. Treat lanes as sources for compact prerequisite packets, not integration units. |

## Done / In Progress / Not Done

Primary-integrated executable capability:

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

Primary-integrated non-executable infrastructure:

- [x] Object-offset `ArrayAccess` receiver diagnostic classifier for read,
  append-read, null-coalesce, `isset`, `empty`, and error-control forms.
- [x] Symbol-table ABI probe is pushed, but remains probe-only until real
  assignment/readback consumers land.

In progress but lane-local or not yet executable primary support:

- [ ] Scalar/resource offset-read prerequisite is ready for integrator review,
  but not yet committed or pushed on primary.
- [ ] Static-property comparison operands need a smaller prerequisite split
  before primary review.
- [ ] Array-key value/reference-slot ABI is a promising fallback but needs
  fresh extraction from current primary.
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

- `62b78f18`: progress-dashboard commit only. No executable compiler/runtime
  semantic code changed.
- `d5450544`: progress-dashboard commit only. No executable compiler/runtime
  semantic code changed.
- `d589bf7d`: progress-dashboard commit only. No executable compiler/runtime
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

- [x] Primary was clean and synced at `62b78f18` at evaluation start.
- [x] Counted executable semantic baseline remains `bfbc62c4`.
- [x] Object-property reference-slot mutation remains the latest integrated
  executable support.
- [x] Object-offset `ArrayAccess` receiver classifier remains integrated at
  `deaf52ca` as diagnostic infrastructure only.
- [x] Overall and executable-semantics estimates remain **86%**.

Best next candidate supply:

- [ ] `scalar-resource-offset-read-prereq`: ready for dedicated primary
  integrator. Reviewed candidate hash
  `5694f62aafc2dcc94aebb7a97a10f3e3a6b8d2587c4266493ed5e766141058d3`;
  current-primary apply proof and focused gates passed in prep and review.
- [ ] `main:34 primary-integrator`: should start only after the evaluator
  wrapper commits/pushes this dashboard update, then recover true primary head
  and rerun the reviewed gates before committing.
- [ ] `impl-native-integration-batch`: array-key value/reference-slot ABI has
  compact lane-local executable proof and remains the recommended fallback.
- [ ] `static-property-comparison-extract`: `needs-split`; do not integrate
  the current broad lane hunk as-is.
- [ ] Older `scalar-resource-offset-read-extract`: `needs-split`; direct
  extraction had empty diff and zero-test gates. Use the reviewed prerequisite
  packet instead.
- [ ] Broad symbol/control-flow/diagnostic/reference/binary-string/type-
  conversion lanes continue producing useful surfaces that are not yet product
  capability.

## Review Notes

Resource pressure is usable but guarded. `/dev/shm` is `40G` total, `24G`
used, `17G` available (`58%`), and `du -sh /dev/shm` reports `24G`. The
`/home` filesystem is `459G` total, `215G` used, `226G` available (`49%`) by
`df`; `du -sh /home` reports `126G` with unreadable overlay-directory
permission warnings. Memory has about `38Gi` available, but swap remains high
at `23Gi` used of `29Gi`.

Advisory steering read: keep accounting at primary head `62b78f18` with
executable baseline `bfbc62c4` until a new semantic commit is pushed. The
scalar/resource prerequisite is now the clear next integrator packet, but the
integrator must revalidate against the true post-dashboard primary head and
rerun focused gates. If it conflicts, broadens, or fails gates, pivot to a
fresh current-primary array-key value/reference-slot ABI extraction rather than
importing broad lane work.
