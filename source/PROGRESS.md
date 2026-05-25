# PHP Native Compiler Progress

Updated: 2026-05-25 10:23 CEST
Evaluation marker: `20260525T082334Z`

Accounting rule: only generalized, tested, committed, and pushed primary work
counts as integrated capability. Dirty WIP, lane-local candidates, broad
worktree claims, probe-only commits, and dashboard-only commits are excluded.
Classifier/blocker commits are listed as integrated infrastructure, but they do
not count as executable PHP feature support unless they execute PHP semantics.

## Current Primary State

- Primary head:
  `d589bf7d docs: update progress dashboard`.
- Primary sync: clean and aligned with `origin/master`.
- Latest integrated executable semantic baseline:
  `bfbc62c4 native: route object property reference slots`.
- Latest integrated non-executable classifier:
  `deaf52ca codegen: classify object ArrayAccess receivers`.
- Current evaluation read:
  no new executable semantic code landed after `bfbc62c4`; this review updates
  the progress marker and records that two attractive extraction candidates
  now need smaller prerequisite splits.

## Executive Read

Overall estimated progress: **86%** `[#################---]`

Executable PHP semantics: **86%** `[#################---]`

Primary remains on the focused executable object-property/reference-slot step
from `bfbc62c4`. Covered object-property assignment and unset mutation now
route subject, property, and replacement operands through shared
value-or-reference slot handling before runtime dereference, object validation,
property-name conversion, mutation, diagnostics, and cleanup.

That is real primary capability, but it is a bounded slice. It is not full
object semantics, full references/COW identity, magic properties, typed or
visibility-sensitive properties, destructor cleanup, exact diagnostics, or
broad backend parity.

Fresh extraction evidence is cautionary. Static-property comparison operands
and scalar/resource offset-read continuations both still look useful, but the
latest extraction artifacts found them too entangled for the requested compact
import. Treat them as lane-local supply until a smaller prerequisite packet is
isolated and gated.

This compiler still executes selected PHP islands. Full generalized PHP remains
blocked on callable/userland frames, references/COW identity, request and
`$GLOBALS` parity, includes, variable variables, full object semantics, real
`ArrayAccess` dispatch, cleanup/unwind/destructor/shutdown ordering, exact
diagnostics, and backend parity.

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
| Scalar/resource offset-read continuations | **40%** `[########------------]` | **40%** `[########------------]` | Fresh extraction says `needs-split`: missing primary `NativeConversionSource` ABI and generated offset-source consumers; attempted gates matched zero tests. |
| Static-property comparison operand ABI | **35%** `[#######-------------]` | **37%** `[#######-------------]` | Fresh extraction says `needs-split`: source lane is too broad and entangled; candidate remained empty. Split metadata/operand prerequisites first. |
| Root-symbol result comparison consumers | **52%** `[##########----------]` | **35%** `[#######-------------]` | Lane-local result-to-result comparison ABI and LLVM/generated-C consumers remain promising, but need a compact current-primary packet. |
| Array owner and value-result boundaries | **58%** `[############--------]` | **38%** `[########------------]` | Lane-local null-coalesce, probe, update-value, and owner-operation work. Useful but broad, with reference/COW and computed-root gaps still open. |
| Formatted-string, stream, and numeric conversion continuations | **52%** `[##########----------]` | **36%** `[#######-------------]` | Lane-local dynamic width/precision, `fopen()` recovery, and numeric-unary work. Extract only one coherent slice at a time. |
| Diagnostics and read/report/writeback boundaries | **58%** `[############--------]` | **39%** `[########------------]` | Lane-local selected-result, required-value writeback, custom-handler, request throw/clone/instanceof blocker, and diagnostic-handle surfaces. Exact Zend ordering and real handler execution remain open. |
| Call/frame cleanup and metadata | **76%** `[###############-----]` | **51%** `[##########----------]` | Lane-local method static-local metadata, reference-source callable dispatch, call-stack metadata, and argument-aware result cleanup are strong infrastructure. Full executable frames remain open. |
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

- [ ] Static-property comparison operands need a smaller prerequisite split before primary review.
- [ ] Scalar/resource offset-read continuations need the missing source/result ABI and generated consumers split first.
- [ ] Root-symbol result-to-result comparison consumers are fresh but broad-lane local.
- [ ] Array-linked null-coalesce/probe/update-value and owner-operation boundaries remain lane-local.
- [ ] Formatted-string dynamic width/precision, `fopen()` recovery, and numeric-unary conversion work remain lane-local.
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
- `b217e2b4`: generated-C declared-object allocation blocks
  destructor-observable native allocation before emitting allocation branches.
- `7679dc0e`: generated-C runtime dynamic calls can invoke supported declared
  callable objects through public `__invoke` method frames.

## Current Work Snapshot

Primary-integrated:

- [x] Primary clean and synced at `d589bf7d`.
- [x] Counted executable semantic baseline remains `bfbc62c4`.
- [x] Object-property reference-slot mutation remains the latest integrated
  executable support.
- [x] Object-offset `ArrayAccess` receiver classifier remains integrated at
  `deaf52ca` as diagnostic infrastructure only.
- [x] Overall and executable-semantics estimates remain **86%**.

Best next candidate supply:

- [ ] `static-property-comparison-extract`: `needs-split`; do not integrate the current broad lane hunk as-is.
- [ ] `scalar-resource-offset-read-extract`: `needs-split`; isolate the missing source/result ABI and generated offset-source consumers first.
- [ ] `impl-native-integration-batch`: root-symbol result-to-result comparison has proof, but needs a small current-primary packet.
- [ ] `impl-array-linked-exec`: null-coalesce/probe/update-value surfaces are useful but broad and overlap array/reference cleanup work.
- [ ] `impl-native-diagnostics`, `impl-native-error-diagnostic-semantics`,
  `impl-global-symbols`, `impl-function-frame-seed`,
  `impl-native-control-flow-seed`, `impl-binary-string-runtime`, and related
  lanes continue producing useful ABI/diagnostic/control-flow/call-frame
  surfaces that are not yet product capability.

## Review Notes

Resource pressure is usable but guarded. `/dev/shm` is `40G` total, `24G`
used, `17G` available (`58%`), and `du -sh /dev/shm` reports `24G`. The
`/home` filesystem is `459G` total, `208G` used, `233G` available (`48%`) by
`df`; `du -sh /home` reported `125G` with unreadable overlay-directory
permission warnings and a nonzero exit. Memory has about `38Gi` available, but
swap remains high at `23Gi` used of `29Gi`.

Advisory steering read: keep accounting at primary head `d589bf7d` with
executable baseline `bfbc62c4`. Do not hand off the static-property comparison
or scalar/resource offset-read extracts as-is; both need smaller prerequisite
splits. Prefer one compact executable current-primary packet over another
metadata-only or classifier-only commit, and keep broad gates disk-backed and
single-job while swap remains high.
