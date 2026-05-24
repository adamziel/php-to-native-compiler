# PHP Native Compiler Progress

Updated: 2026-05-24 19:17 CEST
Evaluation marker: `20260524T1717Z`

Accounting rule: only generalized, tested, committed, and pushed primary work
counts as product capability. Dirty primary WIP, lane-local candidates,
historical worktrees, blocker-only classifiers, and status-file claims are
excluded until selected, gated, committed, pushed, and reflected here.

Current counted primary head:
`b217e2b4 codegen: block destructor-observable native allocation`

Latest counted semantic/test baseline:
`b217e2b4 codegen: block destructor-observable native allocation`

Current uncounted primary WIP:
None in primary.

## Executive Read

Overall estimated progress: **85%** `[#################---]`

Executable PHP semantics: **85%** `[#################---]`

Primary has made real recent progress in selected generated-C execution
islands: descriptor closures, direct and implicit captures, supported
by-reference closure parameters and captures, typed/default/variadic by-value
closure parameters, static anonymous closures, static arrows, callable-array
public method-frame dispatch, supported non-static closure `$this` binding, and
callable-object `__invoke` dispatch through supported public method frames.

This is still not general PHP. The hardest remaining cliffs are full callable
lookup/invocation beyond the selected generated-C callable families, closure
rebinding APIs, references/COW identity, request/`$GLOBALS` alias parity,
object visibility/magic/dynamic/static property behavior, cleanup/unwind/finally
and destructors, exact diagnostics, includes, variable variables, and backend
parity.

## Primary-Integrated Capability

- [x] Descriptor-backed by-value closure invocation.
- [x] Direct by-value closure captures and non-static arrow implicit captures.
- [x] Untyped by-reference descriptor closure parameters.
- [x] Supported root/reference and promoted frame-local by-reference captures.
- [x] Typed/default/variadic by-value descriptor closure parameters.
- [x] Supported static anonymous descriptor closures and static arrow closures.
- [x] Supported non-static closure `$this` binding inside active object frames.
- [x] Callable-array invocation for supported public static/object method frames.
- [x] Callable-object invocation through supported public `__invoke` method frames.
- [x] Runtime string-valued declared-class `new` for constructorless and supported public-constructor classes.
- [x] Destructor-observable declared-class allocation is blocked before generated-C native allocation through declared-class metadata, hierarchy lookup, and dynamic class-name facts.
- [x] Bounded public declared-object properties, methods, statics, constructors, named `instanceof`, and same-family aggregate equality.
- [ ] Not complete: general callable/closure/object/reference/COW/request/global/cleanup/diagnostic/include/backend semantics.

## Grand Roadmap Position

| Workstream | Estimate | Bar | Current read |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | **97%** | `[###################-]` | Strong shared value, array, string, diagnostic, reference, symbol, call-frame, object, method, callable-array, and descriptor-closure surfaces exist for selected paths. |
| Compiler/backend consumers | **97%** | `[###################-]` | Generated-C consumes many shared ABIs; LLVM and C assembly consume selected ABI families. Object lowering, direct assembly parity, and many nested consumers remain blocked. |
| Executable PHP semantics | **85%** | `[#################---]` | Focused linked/runtime programs cover many closure/callable islands, now including supported callable objects, but execution is still selected rather than general PHP. |
| Arrays, lvalues, references, COW | **67%** | `[#############-------]` | Useful selected lvalue/reference paths feed closure parameters and captures; full COW, arbitrary roots, foreach, object joins, and alias composition remain open. |
| Symbols, globals, request state | **72%** | `[##############------]` | Function globals and `$GLOBALS` self-imports improved; request superglobal imports, includes, variable variables, and exact unset/global alias behavior remain incomplete. |
| Calls, functions, frames | **82%** | `[################----]` | Bounded functions, descriptor closures, callable arrays, callable objects, public method frames, and constructor frames work in selected generated-C cases. |
| Objects, properties, methods | **45%** | `[#########-----------]` | Useful public declared-object subset exists and supported public `__invoke` frames are callable. Non-public/contextual visibility, overrides, interfaces/traits, broader magic methods, dynamic/static properties, destructors, references/COW, and backend parity remain open. |
| Control flow, cleanup, diagnostics | **49%** | `[##########----------]` | Selected branches, loops, transfers, finalizers, output buffers, and diagnostics exist; broad unwind, handlers, destructors, shutdown flushing, and source-ordered diagnostics remain open. |
| Broad integrated verification | **84%** | `[#################---]` | Focused gates are strong for recent slices, including callable-object source/link proof; broad `native_link` and call-boundary suites still carry unrelated baseline failures and backend parity gaps. |

## Recent Primary-Integrated Work

- `b217e2b4`: generated-C declared-object allocation now blocks
  destructor-observable native allocation before emitting allocation branches.
  Destructor declarations are recorded as declared-class metadata, inherited
  through class hierarchy lookup, and checked against runtime string-valued
  dynamic class-name facts. The slice treats this as cleanup/unwind safety, not
  destructor execution: destructor-free dynamic constructors stay on the
  bounded declared-class path, while direct, inherited, finite known dynamic,
  unknown dynamic, and nested constructor-argument destructor-risk allocations
  fail through the shared constructor call-boundary blocker.
- `7679dc0e`: generated-C runtime dynamic calls can invoke supported declared
  callable objects through public `__invoke` method frames. Dispatch is gated
  by runtime object type, object/class relation checks, and the declared-method
  candidate table rather than by one source shape. It reuses method-frame
  argument planning, by-reference argument materialization, static/user
  function relay paths, method `$this` binding, diagnostics, and cleanup
  ownership. Source and linked proof cover direct object calls, user-function
  relay, public static-method relay, method self-call through `$this(...)`,
  inherited `__invoke`, and by-reference `__invoke` arguments.
- `53c8a283`: supported non-static regular closures and arrows created inside active object method/constructor frames bind `$this` through the shared descriptor capture/callback path.
- `8f5d8fb3`: parser and generated-C path now admit supported static arrow descriptor closures while preserving static no-`$this` behavior.
- `79496862`: supported static anonymous closures reuse descriptor closure creation, invocation, diagnostics, and cleanup ownership.
- `7a43e1ac`: runtime dynamic calls can invoke syntax-valid callable arrays that resolve to supported generated public static/object method frames.
- `c9172ca6` and `1aaaac30`: supported by-reference closure captures now preserve root/reference cells and promoted function-frame local cells.
- `103c0a4e`, `ff1d8ee3`, and `deabcd6d`: descriptor closures support selected variadic, typed/default by-value, and untyped by-reference parameters.

## Current Work Snapshot

Primary-integrated and counted:

- [x] Previous synced baseline was `adc1785d`.
- [x] Local counted semantic baseline is `b217e2b4`; this progress wrapper and push complete the destructor-observable allocation-blocker accounting batch.
- [x] Progress accounting clarifies that the current source-of-truth estimate is 85%, not older lane-inflated or strict-rebaseline figures.

In progress but uncounted:

- [ ] Lane-local ArrayAccess/object-offset diagnostics and blocker precision.
- [ ] Lane-local byte-keyed global/symbol storage and request/global work.
- [ ] Lane-local reference/COW metadata, cleanup/control-flow, callback, and object/property candidates.

Not done:

- [ ] Full callable lookup and invocation beyond selected strings, closures, arrays, and public `__invoke` objects, including magic/visibility/rebinding rules.
- [ ] Full references/COW identity and arbitrary alias roots.
- [ ] Request/`$GLOBALS` parity, includes, variable variables, and dynamic symbol behavior.
- [ ] General object model: non-public methods, overrides, interfaces/traits, magic methods, dynamic/static properties, destructors.
- [ ] Complete cleanup/unwind/finally/destructor/output-buffer shutdown behavior.
- [ ] Exact/source-ordered diagnostics and warning/error continuation.
- [ ] LLVM/direct assembly parity for recent generated-C semantics.

## Lane-Local Versus Primary

Lane-local work is useful only when it feeds a distinct primary integration.
Current likely-useful evidence includes destructor cleanup blockers,
ArrayAccess operation-family diagnostics, byte-keyed globals, reference/COW
owner-slot work, object/property runtime blockers, and control-flow cleanup
candidates.

Historical or already-landed surfaces should not be repeated: callable-array
public method-frame invocation, callable-object public `__invoke` dispatch,
static anonymous descriptor closures, static arrow descriptor closures,
non-static closure `$this` binding, and dynamic declared-class `new` variants
unless they are rebased around a new unresolved boundary.

## Review Notes

Focused destructor-observable allocation-blocker gates passed on disk-backed
`/tmp/phpc-primary-destructor-blocker`: `cargo check -q -p phpc -p
php_runtime`, the `native_function_call_boundary` destructor filter, adjacent
call-boundary routing regressions, declared-object and callable-object
generated-C regressions, scoped `rustfmt --edition 2021 --check`, and `git diff
--check` / `git diff --cached --check`. Whole-repo `cargo fmt --check` still
reports unrelated existing formatting drift in `compiler/src/interpreter.rs`;
the touched Rust files pass scoped rustfmt. This blocker remains cleanup/unwind
safety only, not destructor execution.
