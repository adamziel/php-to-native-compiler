# PHP Native Compiler Progress

Updated: 2026-05-25 09:28 CEST
Evaluation marker: `20260525T072856Z`

Accounting rule: only generalized, tested, committed, and pushed primary work
counts as product capability. Dirty WIP, lane-local candidates, probe-only
commits, dashboard-only commits, blocker-only classifiers, and status-file
claims are excluded until selected, gated, committed, pushed, and reflected
here as semantic product progress.

## Current Primary State

- Primary head: `3cf5707c docs: update progress for bounded preg callbacks`.
- Primary sync: clean and aligned with `origin/master`.
- Latest counted semantic baseline:
  `6aca392d interpreter: execute bounded preg callbacks`.
- Pushed but uncounted code work:
  `2967110c codegen: expose symbol table abi probe`.
- Current primary-ready review result: GO for
  `object-arrayaccess-error-control-retry`, explicitly as a diagnostic
  blocker/classifier, not as executable `ArrayAccess` support.

## Executive Read

Overall estimated progress: **85%** `[#################---]`

Executable PHP semantics: **85%** `[#################---]`

Primary semantic capability is flat since the bounded preg callback slice.
That slice is now cleanly accounted: supported string callbacks execute for a
bounded slash-delimited `preg_replace_callback()` subset. It is still not full
PCRE and not generalized callable semantics.

The freshest primary decision is about integration hygiene, not feature
breadth. `object-arrayaccess-error-control-retry` is ready for a dedicated
integrator as a blocker/classifier. It improves unsupported object-offset
diagnostic routing, but it must not move the headline feature percentage.

Lane-local work is active and useful, especially object-property reference
slots, scalar/resource offset-read continuations, static-property comparison
operands, root-symbol consumers, control-flow cleanup, diagnostics, and
call-frame cleanup contracts. These remain candidate supply until extracted
and integrated.

This compiler is still selected PHP execution. Full generalized PHP remains
blocked on callable/userland frames, references/COW identity, request and
`$GLOBALS` parity, includes, variable variables, full object semantics, real
`ArrayAccess` dispatch, cleanup/unwind/destructor/shutdown ordering, exact
diagnostics, and backend parity.

## Grand Roadmap Position

| Workstream | Estimate | Bar | Current read |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | **97%** | `[###################-]` | Strong selected-path value, array, string, diagnostic, reference, symbol, call-frame, object, comparison, conversion, owner-cell, and request-state surfaces. Many expansions remain lane-local. |
| Compiler/backend consumers | **97%** | `[###################-]` | Generated C consumes many shared ABIs; LLVM and C assembly consume selected families. Recent root-symbol, static-property, and diagnostic consumers remain uncounted until integrated. |
| Executable PHP semantics | **85%** | `[#################---]` | Primary has closure/callable/object islands plus bounded preg string-callback execution. The next GO candidate is classifier-only, so semantic percentage is flat. |
| Arrays, lvalues, references, COW | **67%** | `[#############-------]` | Useful selected lvalue/reference paths exist. Full COW, arbitrary roots, foreach, property references, and alias composition remain open. |
| Symbols, globals, request state | **72%** | `[##############------]` | Selected function globals and root-symbol surfaces exist. `$GLOBALS` self-cells, request/global alias parity, includes, variable variables, and exact unset/global behavior remain incomplete. |
| Calls, functions, frames | **82%** | `[################----]` | Bounded closures, callable arrays/objects, public method frames, and constructors exist in selected paths. Named/unpacked/by-reference/userland frame breadth remains incomplete. |
| Objects, properties, methods | **45%** | `[#########-----------]` | Public declared-object subsets and `__invoke` islands exist. Full visibility, magic, dynamic/static/typed properties, destructors, references/COW, and `ArrayAccess` execution remain open. |
| Control flow, cleanup, diagnostics | **49%** | `[##########----------]` | Selected branches, loops, transfers, finalizers, output buffers, and diagnostics exist. Broad unwind/finally/destructor/shutdown and exact source ordering remain open. |
| Broad integrated verification | **84%** | `[#################---]` | Focused gates are strong. Broad gates remain constrained by lane extraction cost, high swap, and backend parity gaps. |

## Active Roadmap Items

| Item | Toward Primary Integration | Toward Full Feature | Status |
| --- | ---: | ---: | --- |
| Bounded `preg_replace_callback()` string callbacks | **100%** `[####################]` | **32%** `[######--------------]` | Integrated at `6aca392d`. Full PCRE, broader captures/modifiers, non-string callables, `limit`/`count`/`flags`, and legacy recognizer cleanup remain open. |
| Object-offset `ArrayAccess` error-control classifier | **91%** `[##################--]` | **10%** `[##------------------]` | Primary review says GO. One-file classifier with current-primary apply proof. It is diagnostic routing only, not `ArrayAccess` method execution. |
| Object-property reference-slot mutation | **82%** `[################----]` | **35%** `[#######-------------]` | Strong executable candidate with apply proof, but scoped rustfmt still fails on formatting-only diffs. Needs repair, refreshed hash, and gates. |
| Scalar/resource offset-read continuations | **64%** `[#############-------]` | **40%** `[########------------]` | Lane-local shared warning-plus-null continuation across runtime, LLVM, generated C, and consumers. Needs compact extraction. |
| Static-property comparison operand ABI | **62%** `[############--------]` | **37%** `[#######-------------]` | Lane-local public static-property comparison operand and uninitialized typed-property fatal operand. Extract narrowly. |
| Array owner value-operation boundary | **58%** `[############--------]` | **36%** `[#######-------------]` | Lane-local read/update/unset/RMW/foreach validation boundary. Useful but high overlap with array/reference work and one adjacent focused failure. |
| Root-symbol result consumers | **58%** `[############--------]` | **36%** `[#######-------------]` | Lane-local count, selected-value, comparison, string, and metadata consumers are active. Full `$GLOBALS`/request alias parity remains open. |
| Diagnostics and read/report boundaries | **55%** `[###########---------]` | **37%** `[#######-------------]` | Lane-local diagnostic report outcome, read-presence, callable resolution, and entry sequencing boundaries. Exact Zend ordering and handler execution remain open. |
| Call/frame cleanup and metadata | **76%** `[###############-----]` | **51%** `[##########----------]` | Lane-local method static-local metadata, reference-source callable dispatch, and argument-aware result cleanup are strong infrastructure. Full executable frames remain open. |
| Broad lane extraction backlog | **30%** `[######--------------]` | **31%** `[######--------------]` | Many lanes have useful work, but broad worktrees are conflict-heavy. Treat them as sources for compact extraction packets. |

## Done / In Progress / Not Done

Primary-integrated and counted:

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

In progress but uncounted:

- [ ] `object-arrayaccess-error-control-retry` classifier is primary-ready but not integrated, and is not executable `ArrayAccess`.
- [ ] `object-property-reference-slots` executable candidate needs formatting repair and fresh gates.
- [ ] Scalar/resource offset-read continuation, static-property comparison operand, and array owner value-operation candidates need compact extraction.
- [ ] Root-symbol count/selected/comparison/string consumers and request/global metadata remain lane-local.
- [ ] Diagnostic report/read-presence/callable-resolution boundaries remain lane-local.
- [ ] Control-flow loop/switch/goto cleanup-state advances remain lane-local.
- [ ] Binary-string scanner, stream, error-handler, text-byte, and request-state surfaces remain lane-local.

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

- `6aca392d`: interpreter `preg_replace_callback()` executes supported string
  callbacks over a bounded slash-delimited pattern subset.
- `b217e2b4`: generated-C declared-object allocation blocks
  destructor-observable native allocation before emitting allocation branches.
- `7679dc0e`: generated-C runtime dynamic calls can invoke supported declared
  callable objects through public `__invoke` method frames.
- `53c8a283`: non-static regular closures and arrows created inside active
  object frames bind `$this` through the descriptor capture/callback path.
- `7a43e1ac`: runtime dynamic calls can invoke syntax-valid callable arrays
  that resolve to supported generated public static/object method frames.

## Current Work Snapshot

Primary-integrated and counted:

- [x] Primary clean and synced at `3cf5707c`.
- [x] Counted semantic baseline remains `6aca392d`.
- [x] Overall and executable-semantics estimates remain **85%**.

Primary-ready but unintegrated:

- [ ] `object-arrayaccess-error-control-retry` has GO from read-only review.
  It should be integrated only as a classifier/blocker and should not change
  feature percentages.

Lane-local candidate supply:

- [ ] `object-property-reference-slots`: executable object mutation/reference
  candidate blocked by formatting-only rustfmt failures.
- [ ] `impl-native-type-conversion`: scalar/resource offset-read continuation
  candidate needs extraction.
- [ ] `impl-native-comparison-semantics`: static-property comparison operand
  candidate needs extraction.
- [ ] `impl-array-linked-exec`: owner value-operation boundary candidate is
  useful but high-overlap.
- [ ] `impl-native-diagnostics`, `impl-native-error-diagnostic-semantics`,
  `impl-global-symbols`, `impl-function-frame-seed`,
  `impl-native-control-flow-seed`, `impl-binary-string-runtime`, and related
  lanes continue producing useful ABI/diagnostic/control-flow/call-frame
  surfaces that are not yet product capability.

## Review Notes

Resource pressure is serviceable but guarded. `/dev/shm` is `40G` total,
`24G` used, `17G` available (`58%`), and `du -sh /dev/shm` reports `24G`.
`/home` is `459G` total, `219G` used, `221G` available (`50%`); bounded
`du -sh /home` returned `125G` with nonzero exit, likely due unreadable
entries. Memory has about `37Gi` available, but swap remains high at
`23Gi` used of `29Gi`.

Advisory steering read: let the dedicated integrator consume one clean GO at a
time. If `object-arrayaccess-error-control-retry` lands, keep the message
strictly classifier/blocker. For executable progress, next favor formatting
repair and gating of `object-property-reference-slots`, or extract the
scalar/resource offset-read continuation slice. Keep broad gates disk-backed
and single-job while swap remains high.
