# PHP Native Compiler Progress

Updated: 2026-05-25 10:43 CEST
Evaluation marker: `20260525T084349Z`

Accounting rule: only generalized, tested, committed, and pushed primary work
counts as integrated capability. Dirty WIP, lane-local candidates, broad
worktree claims, probe-only commits, and dashboard-only commits are excluded.
Classifier/blocker commits are listed as integrated infrastructure, but they do
not count as executable PHP feature support unless they execute PHP semantics.

## Current Primary State

- Primary head:
  `d5450544 docs: update progress dashboard`.
- Primary sync: clean and aligned with `origin/master`.
- Latest integrated executable semantic baseline:
  `bfbc62c4 native: route object property reference slots`.
- Latest integrated non-executable classifier:
  `deaf52ca codegen: classify object ArrayAccess receivers`.
- Current evaluation read:
  no executable semantic code landed after `bfbc62c4`. The best current
  movement is candidate triage toward a smaller scalar/resource offset-read
  prerequisite packet from true primary `d5450544`; no prerequisite candidate
  status artifact was present at this review time.

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

The freshest work is mostly lane-local supply. Candidate triage rejected the
existing static-property comparison and scalar/resource offset-read extracts as
too broad or empty, then launched a narrower scalar/resource source-result ABI
prerequisite task. Count it as promising preparation, not integrated support.

## Grand Roadmap Position

| Workstream | Estimate | Bar | Current read |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | **97%** | `[###################-]` | Strong selected-path value, array, string, diagnostic, reference, symbol, call-frame, object, comparison, conversion, owner-cell, and request-state surfaces. Many expansions remain lane-local. |
| Compiler/backend consumers | **98%** | `[####################]` | Generated C and LLVM consume many shared ABIs. Recent executable progress is generated-C/native-link object-property reference-slot mutation; broad parity remains incomplete. |
| Executable PHP semantics | **86%** | `[#################---]` | Primary has closure/callable/object islands, bounded preg callbacks, and focused object-property reference-slot mutation. No new executable semantic commit landed in this review window. |
| Arrays, lvalues, references, COW | **69%** | `[##############------]` | Reference-backed object-property mutation is integrated. Full COW, arbitrary roots, foreach, property references, and alias composition remain open. |
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
| Scalar/resource offset-read source-result prerequisite | **44%** `[#########-----------]` | **40%** `[########------------]` | Candidate triage ranks this first. A manual task is assigned to extract `NativeConversionSource`, offset-read source ABIs, generated consumers, and focused tests from true primary `d5450544`; no result artifact yet. |
| Static-property comparison operand ABI | **35%** `[#######-------------]` | **37%** `[#######-------------]` | Fresh extraction says `needs-split`: source lane is too broad and entangled; candidate remained empty. Split metadata/operand prerequisites first. |
| Array-key value/reference-slot ABI | **60%** `[############--------]` | **38%** `[########------------]` | Lane-local generated-C/runtime packet has compact proof and is the recommended fallback if scalar/resource extraction broadens. Needs fresh current-primary extraction. |
| Root-symbol result comparison consumers | **52%** `[##########----------]` | **35%** `[#######-------------]` | Lane-local result-to-result comparison ABI and LLVM/generated-C consumers remain promising, but need a compact current-primary packet. |
| Post-spread call argument planning | **48%** `[##########----------]` | **42%** `[########------------]` | Lane-local call planner evidence is focused but high sensitivity; extract only with current-primary hash/apply proof and narrow gates. |
| LLVM/static integer bitwise shared ABI | **44%** `[#########-----------]` | **34%** `[#######-------------]` | Lane-local LLVM routing removes direct integer bitwise folding through a shared value ABI. Generated-C parity and broad conversion/reference gaps remain open. |
| Diagnostics, request, and cleanup boundaries | **58%** `[############--------]` | **39%** `[########------------]` | Lane-local request handle, writeback, branch cleanup, and result-boundary work is useful infrastructure. Exact Zend ordering and real handler execution remain open. |
| Broad lane extraction backlog | **31%** `[######--------------]` | **32%** `[######--------------]` | Recent failed extractions show the backlog is real. Treat lanes as sources for compact prerequisite packets, not integration units. |

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

In progress but lane-local or not yet executable support:

- [ ] Scalar/resource offset-read prerequisite extraction is active, but no candidate status artifact was present at this review.
- [ ] Static-property comparison operands need a smaller prerequisite split before primary review.
- [ ] Array-key value/reference-slot ABI is a promising fallback but needs fresh extraction from current primary.
- [ ] Root-symbol result-to-result comparison consumers are fresh but broad-lane local.
- [ ] Array-linked null-coalesce/probe/update-value and owner-operation boundaries remain lane-local.
- [ ] Formatted-string, stream, numeric conversion, LLVM bitwise, and callback-dispatch work remain lane-local.
- [ ] Request-backed throw/clone/instanceof blockers and diagnostic writeback/selection boundaries remain lane-local.
- [ ] Binary-string scanner, text-byte slot, error-handler dispatch, stream, and class-alias surfaces remain lane-local.
- [ ] Control-flow loop/switch/goto cleanup-state advances remain lane-local.

Not done:

- [ ] Full callable lookup and invocation, including non-string preg callbacks, closures, arrays, invokable objects, magic/visibility, and rebinding rules.
- [ ] Runtime `ArrayAccess` method dispatch for `offsetGet`, `offsetExists`, `offsetSet`, and `offsetUnset`.
- [ ] Full references/COW identity and arbitrary alias roots.
- [ ] Request and `$GLOBALS` parity, includes, variable variables, and dynamic symbol behavior.
- [ ] Full PCRE behavior beyond the bounded slash-delimited subset.
- [ ] Retirement or reframing of unrelated legacy WordPress-named preg/database recognizers behind generalized PHP semantic boundaries.
- [ ] General object model: non-public methods, overrides, interfaces/traits execution, magic methods, dynamic/static/typed properties, destructors.
- [ ] Complete cleanup/unwind/finally/destructor/output-buffer shutdown behavior.
- [ ] Exact/source-ordered diagnostics, custom handler execution, warning/error continuation, and suppression parity.
- [ ] LLVM/direct assembly parity for recent generated-C semantics.

## Recent Primary-Integrated Work

- `d5450544`: progress-dashboard commit only. No executable
  compiler/runtime semantic code changed.
- `d589bf7d`: progress-dashboard commit only. No executable
  compiler/runtime semantic code changed.
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

- [x] Primary clean and synced at `d5450544`.
- [x] Counted executable semantic baseline remains `bfbc62c4`.
- [x] Object-property reference-slot mutation remains the latest integrated
  executable support.
- [x] Object-offset `ArrayAccess` receiver classifier remains integrated at
  `deaf52ca` as diagnostic infrastructure only.
- [x] Overall and executable-semantics estimates remain **86%**.

Best next candidate supply:

- [ ] `scalar-resource-offset-read-prereq`: active manual candidate-prep task; status artifact not present at review time.
- [ ] `impl-native-integration-batch`: array-key value/reference-slot ABI has compact lane-local executable proof and is the recommended fallback.
- [ ] `static-property-comparison-extract`: `needs-split`; do not integrate the current broad lane hunk as-is.
- [ ] `scalar-resource-offset-read-extract`: `needs-split`; prior direct extraction had empty diff and zero-test gates.
- [ ] `impl-native-call-semantics`: post-spread argument planning is useful but high-sensitivity.
- [ ] Broad symbol/control-flow/diagnostic/reference/binary-string/type-conversion lanes continue producing useful surfaces that are not yet product capability.

## Review Notes

Resource pressure is usable but guarded. `/dev/shm` is `40G` total, `24G`
used, `17G` available (`58%`), and `du -sh /dev/shm` reports `24G`. The
`/home` filesystem is `459G` total, `217G` used, `224G` available (`50%`) by
`df`; `du -sh /home` reports `125G` with unreadable overlay-directory
permission warnings. Memory has about `39Gi` available, but swap remains high
at `23Gi` used of `29Gi`.

Advisory steering read: keep accounting at primary head `d5450544` with
executable baseline `bfbc62c4`. Let the scalar/resource prerequisite attempt
run only as a compact current-primary candidate with real tests. If it broadens
or remains unreported, pivot to the array-key value/reference-slot ABI packet.
Keep broad gates disk-backed and single-job while swap remains high.
