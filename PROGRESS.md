# PHP Native Compiler Progress

Updated: 2026-05-25 10:00 CEST
Evaluation marker: `20260525T080015Z`

Accounting rule: only generalized, tested, committed, and pushed primary work
counts as integrated capability. Dirty WIP, lane-local candidates, broad
worktree claims, probe-only commits, and dashboard-only commits are excluded.
Classifier/blocker commits are listed as integrated infrastructure, but they do
not count as executable PHP feature support unless they execute PHP semantics.

## Current Primary State

- Primary head:
  `bfbc62c4 native: route object property reference slots`.
- Primary sync: clean and aligned with `origin/master`.
- Latest integrated executable semantic baseline:
  `bfbc62c4` object-property mutation with reference-backed operands.
- Latest integrated non-executable classifier:
  `deaf52ca` object-offset `ArrayAccess` receiver diagnostics.
- Pushed but still probe-only:
  `2967110c codegen: expose symbol table abi probe`.

## Executive Read

Overall estimated progress: **86%** `[#################---]`

Executable PHP semantics: **86%** `[#################---]`

Primary made a real executable step after the last dashboard: `bfbc62c4`
routes object-property assignment and unset mutation through shared
value-or-reference slot handling. Reference-backed subject/property/replacement
operands now dereference through runtime boundaries before object validation,
property-name conversion, mutation, diagnostics, and cleanup. The focused
runtime, generated-C route, linked executable, and object-property mutation
gates passed before push.

This is meaningful object/property/reference progress, but it is a focused
slice. It is not full object semantics, full references/COW identity, magic or
typed or visibility-sensitive properties, destructor cleanup, exact diagnostics,
or broad backend parity.

Lane-local supply remains strong. The best next product moves are compact
extractions from static-property comparison operands, scalar/resource offset
read continuations, or root-symbol result-to-result comparisons. Broad lane
worktrees are not product progress until extracted, gated, committed, and
pushed.

This compiler still executes selected PHP islands. Full generalized PHP remains
blocked on callable/userland frames, references/COW identity, request and
`$GLOBALS` parity, includes, variable variables, full object semantics, real
`ArrayAccess` dispatch, cleanup/unwind/destructor/shutdown ordering, exact
diagnostics, and backend parity.

## Grand Roadmap Position

| Workstream | Estimate | Bar | Current read |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | **97%** | `[###################-]` | Strong selected-path value, array, string, diagnostic, reference, symbol, call-frame, object, comparison, conversion, owner-cell, and request-state surfaces. Many expansions remain lane-local. |
| Compiler/backend consumers | **98%** | `[####################]` | Generated C and LLVM consume many shared ABIs. `bfbc62c4` adds generated-C object-property reference-slot mutation routing; broad parity remains incomplete. |
| Executable PHP semantics | **86%** | `[#################---]` | Primary has closure/callable/object islands, bounded preg callbacks, and focused object-property reference-slot mutation. |
| Arrays, lvalues, references, COW | **69%** | `[##############------]` | Reference-backed object-property mutation is integrated. Full COW, arbitrary roots, foreach, property references, and alias composition remain open. |
| Symbols, globals, request state | **72%** | `[##############------]` | Selected function globals and root-symbol surfaces exist. `$GLOBALS` self-cells, request/global alias parity, includes, variable variables, and exact unset/global behavior remain incomplete. |
| Calls, functions, frames | **82%** | `[################----]` | Bounded closures, callable arrays/objects, public method frames, and constructors exist in selected paths. Named/unpacked/by-reference/userland frame breadth remains incomplete. |
| Objects, properties, methods | **50%** | `[##########----------]` | `bfbc62c4` improves mutation/reference-slot execution for object properties. Full visibility, magic, dynamic/static/typed properties, destructors, references/COW, and `ArrayAccess` execution remain open. |
| Control flow, cleanup, diagnostics | **50%** | `[##########----------]` | Selected branches, loops, transfers, finalizers, output buffers, and diagnostics exist. Broad unwind/finally/destructor/shutdown and exact source ordering remain open. |
| Broad integrated verification | **85%** | `[#################---]` | Focused gates are strong. Broad gates remain constrained by lane extraction cost, high swap, and backend parity gaps. |

## Active Roadmap Items

| Item | Toward Primary Integration | Toward Full Feature | Status |
| --- | ---: | ---: | --- |
| Object-property reference-slot mutation | **100%** `[####################]` | **39%** `[########------------]` | Integrated at `bfbc62c4`. Executable generated-C/native-link support for covered assignment/unset mutation operands. Full object/property/reference semantics remain open. |
| Bounded `preg_replace_callback()` string callbacks | **100%** `[####################]` | **32%** `[######--------------]` | Integrated at `6aca392d`. Full PCRE, broader captures/modifiers, non-string callables, `limit`/`count`/`flags`, and legacy recognizer cleanup remain open. |
| Object-offset `ArrayAccess` diagnostic classifier | **100%** `[####################]` | **12%** `[##------------------]` | Integrated at `deaf52ca`. Diagnostic routing only; no `offsetGet`, `offsetExists`, `offsetSet`, or `offsetUnset` execution. |
| Scalar/resource offset-read continuations | **64%** `[#############-------]` | **40%** `[########------------]` | Lane-local shared warning-plus-null continuation across runtime, LLVM, generated C, and consumers. Needs compact extraction. |
| Static-property comparison operand ABI | **62%** `[############--------]` | **37%** `[#######-------------]` | Lane-local public static-property comparison operands and uninitialized typed-property fatal operands. Extract narrowly. |
| Root-symbol result comparison consumers | **55%** `[###########---------]` | **35%** `[#######-------------]` | Fresh lane-local result-to-result comparison ABI and LLVM/generated-C consumers. Broad lane has conflict-heavy carryover. |
| Array owner and value-result boundaries | **60%** `[############--------]` | **38%** `[########------------]` | Lane-local null-coalesce, probe, update-value, and owner-operation work. Useful but broad, with reference/COW and computed-root gaps still open. |
| Formatted-string, stream, and numeric conversion continuations | **54%** `[###########---------]` | **36%** `[#######-------------]` | Lane-local dynamic width/precision, `fopen()` recovery, and numeric-unary work. Extract only one coherent slice at a time. |
| Diagnostics and read/report/writeback boundaries | **58%** `[############--------]` | **39%** `[########------------]` | Lane-local selected-result, required-value writeback, custom-handler, request throw/clone/instanceof blocker, and diagnostic-handle surfaces. Exact Zend ordering and real handler execution remain open. |
| Call/frame cleanup and metadata | **76%** `[###############-----]` | **51%** `[##########----------]` | Lane-local method static-local metadata, reference-source callable dispatch, call-stack metadata, and argument-aware result cleanup are strong infrastructure. Full executable frames remain open. |
| Broad lane extraction backlog | **33%** `[#######-------------]` | **32%** `[######--------------]` | Many lanes have useful work, but broad worktrees are conflict-heavy. Treat them as sources for compact extraction packets. |

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

- [ ] Static-property comparison operands and scalar/resource offset-read continuations need compact extraction.
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
- `53c8a283`: non-static regular closures and arrows created inside active
  object frames bind `$this` through the descriptor capture/callback path.

## Current Work Snapshot

Primary-integrated:

- [x] Primary clean and synced at `bfbc62c4`.
- [x] Counted executable semantic baseline is now `bfbc62c4`.
- [x] Object-property reference-slot mutation is integrated as focused
  executable support.
- [x] Object-offset `ArrayAccess` receiver classifier is integrated at
  `deaf52ca` as diagnostic infrastructure only.
- [x] Overall and executable-semantics estimates move to **86%**.

Best next candidate supply:

- [ ] `impl-native-comparison-semantics`: static-property comparison operand
  candidate needs compact extraction.
- [ ] `impl-native-type-conversion`: scalar/resource offset-read continuation
  candidate needs compact extraction; avoid bundling later stream/math/format
  work into the same import.
- [ ] `impl-native-integration-batch`: root-symbol result-to-result comparison
  has fresh proof, but broad lane conflict state needs a small packet.
- [ ] `impl-array-linked-exec`: null-coalesce/probe/update-value surfaces are
  useful but broad and overlap array/reference cleanup work.
- [ ] `impl-native-diagnostics`, `impl-native-error-diagnostic-semantics`,
  `impl-global-symbols`, `impl-function-frame-seed`,
  `impl-native-control-flow-seed`, `impl-binary-string-runtime`, and related
  lanes continue producing useful ABI/diagnostic/control-flow/call-frame
  surfaces that are not yet product capability.

## Review Notes

Resource pressure is serviceable but guarded. `/dev/shm` is `40G` total,
`24G` used, `17G` available (`58%`), and `du -sh /dev/shm` reports `24G`.
The `/home` filesystem is `459G` total, `237G` used, `203G` available (`54%`).
A live full `du -sh /home` did not finish within the bounded check and hit
overlay permission warnings; the bounded snapshot reported `125G` with those
warnings. Memory has about `36Gi` available, but swap remains high at `23Gi`
used of `29Gi`.

Advisory steering read: refresh supervisor state to current head `bfbc62c4`
and count it as a focused executable object-property/reference-slot slice. For
the next primary step, favor a compact executable extraction from
static-property comparison operands, scalar/resource offset-read continuations,
or root-symbol result-to-result comparisons. Keep broad gates disk-backed and
single-job while swap remains high.
