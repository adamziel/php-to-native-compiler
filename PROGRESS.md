# PHP Native Compiler Progress

Updated: 2026-05-24 20:41 CEST
Evaluation marker: `20260524T184153Z`

Accounting rule: only generalized, tested, committed, and pushed primary work
counts as product capability. Dirty primary WIP, unpushed local commits,
lane-local candidates, historical worktrees, probe-only changes, blocker-only
classifiers, and status-file claims are excluded until selected, gated,
committed, pushed, and reflected here.

Current counted pushed primary head:
`28bb7423 docs: update progress after destructor blocker`

Latest counted semantic/test baseline:
`b217e2b4 codegen: block destructor-observable native allocation`

Current uncounted primary state:

- Local primary `HEAD` is `2967110c codegen: expose symbol table abi probe`,
  one commit ahead of `origin/master`. This is ABI/probe visibility, not
  generalized PHP variable storage semantics, and is not counted here while
  unpushed.
- Dirty primary WIP remains in `compiler/src/interpreter.rs` and
  `compiler/tests/preg_replace_callback_builtin.rs`. Audit says it needs edits
  because bounded `preg_replace_callback()` callback execution still consumes
  exact `WordPressNonAsciiByte` pattern behavior.

## Executive Read

Overall estimated progress: **85%** `[#################---]`

Executable PHP semantics: **85%** `[#################---]`

Primary has made real recent progress in selected generated-C execution
islands: descriptor closures, direct and implicit captures, supported
by-reference closure parameters and captures, typed/default/variadic by-value
closure parameters, static anonymous closures, static arrows, callable-array
public method-frame dispatch, supported non-static closure `$this` binding,
callable-object `__invoke` dispatch through supported public method frames,
runtime string-valued declared-class construction for selected cases, and
destructor-observable allocation blocking for declared objects.

This is still not general PHP. The hardest remaining cliffs are full callable
lookup/invocation beyond selected generated-C families, closure rebinding APIs,
references/COW identity, request/`$GLOBALS` alias parity, includes, variable
variables, object visibility/magic/dynamic/static property behavior,
cleanup/unwind/finally and destructor execution, exact diagnostics, and backend
parity.

## Grand Roadmap Position

| Workstream | Estimate | Bar | Current read |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | **97%** | `[###################-]` | Strong shared value, array, string, diagnostic, reference, symbol, call-frame, object, method, callable-array, and descriptor-closure surfaces exist for selected paths. Local symbol-table ABI probe work is not counted as semantics. |
| Compiler/backend consumers | **97%** | `[###################-]` | Generated-C consumes many shared ABIs; LLVM and C assembly consume selected ABI families. Object lowering, direct assembly parity, and many nested consumers remain blocked. |
| Executable PHP semantics | **85%** | `[#################---]` | Focused linked/runtime programs cover many closure/callable islands, including supported callable objects, but execution is still selected rather than general PHP. |
| Arrays, lvalues, references, COW | **67%** | `[#############-------]` | Useful selected lvalue/reference paths feed closure parameters and captures; full COW, arbitrary roots, foreach, object joins, property references, and alias composition remain open. |
| Symbols, globals, request state | **72%** | `[##############------]` | Function globals and `$GLOBALS` self-imports improved; request superglobal imports, includes, variable variables, exact unset/global alias behavior, and executable symbol-table lowering remain incomplete. |
| Calls, functions, frames | **82%** | `[################----]` | Bounded functions, descriptor closures, callable arrays, callable objects, public method frames, and constructor frames work in selected generated-C cases. |
| Objects, properties, methods | **45%** | `[#########-----------]` | Useful public declared-object subset exists and supported public `__invoke` frames are callable. Non-public/contextual visibility, overrides, interfaces/traits, broader magic methods, dynamic/static properties, destructors, references/COW, and backend parity remain open. |
| Control flow, cleanup, diagnostics | **49%** | `[##########----------]` | Selected branches, loops, transfers, finalizers, output buffers, and diagnostics exist; broad unwind, handlers, destructor execution, shutdown flushing, and source-ordered diagnostics remain open. |
| Broad integrated verification | **84%** | `[#################---]` | Focused gates are strong for recent slices. Broad `native_link`, call-boundary, and dirty-checkout gates still carry unrelated failures and backend parity gaps. |

## Done / In Progress / Not Done

Primary-integrated and counted:

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

In progress but uncounted:

- [ ] Local primary symbol-table ABI probe commit `2967110c`; not pushed and not generalized PHP symbol storage.
- [ ] Dirty bounded `preg_replace_callback()` callback-execution WIP; needs regex-boundary cleanup or exclusion of exact `WordPressNonAsciiByte` behavior.
- [ ] Lane-local object-property mutation value/reference-slot candidate at hash `c60ed1c30dc1d979da1fc44641ae4378a3629e45c42da0d9621faf10519e56e8`.
- [ ] Lane-local object-offset ArrayAccess error-control classifier candidate; useful blocker progress only, hash must be rechecked.
- [ ] Lane-local extension metadata, object-property liveness/unset, post-append reference assignment, comparison, diagnostics, and symbol/control-flow candidates that need clean extraction.

Not done:

- [ ] Full callable lookup and invocation beyond selected strings, closures, arrays, and public `__invoke` objects, including magic/visibility/rebinding rules.
- [ ] Full references/COW identity and arbitrary alias roots.
- [ ] Request/`$GLOBALS` parity, includes, variable variables, and dynamic symbol behavior.
- [ ] General object model: non-public methods, overrides, interfaces/traits, magic methods, dynamic/static properties, typed properties, destructors.
- [ ] Complete cleanup/unwind/finally/destructor/output-buffer shutdown behavior.
- [ ] Exact/source-ordered diagnostics and warning/error continuation.
- [ ] LLVM/direct assembly parity for recent generated-C semantics.

## Recent Primary-Integrated Work

- `b217e2b4`: generated-C declared-object allocation now blocks
  destructor-observable native allocation before emitting allocation branches.
  Destructor declarations are recorded as declared-class metadata, inherited
  through class hierarchy lookup, and checked against runtime string-valued
  dynamic class-name facts. This is cleanup/unwind safety only: destructor-free
  dynamic constructors stay on the bounded declared-class path, while direct,
  inherited, finite known dynamic, unknown dynamic, and nested constructor-
  argument destructor-risk allocations fail through the shared constructor
  call-boundary blocker.
- `7679dc0e`: generated-C runtime dynamic calls can invoke supported declared
  callable objects through public `__invoke` method frames. Dispatch is gated
  by runtime object type, object/class relation checks, and the declared-method
  candidate table. Source and linked proof cover direct object calls,
  user-function relay, public static-method relay, method self-call through
  `$this(...)`, inherited `__invoke`, and by-reference `__invoke` arguments.
- `53c8a283`: supported non-static regular closures and arrows created inside
  active object method/constructor frames bind `$this` through the shared
  descriptor capture/callback path.
- `8f5d8fb3`: parser and generated-C path now admit supported static arrow
  descriptor closures while preserving static no-`$this` behavior.
- `79496862`: supported static anonymous closures reuse descriptor closure
  creation, invocation, diagnostics, and cleanup ownership.
- `7a43e1ac`: runtime dynamic calls can invoke syntax-valid callable arrays
  that resolve to supported generated public static/object method frames.
- `c9172ca6` and `1aaaac30`: supported by-reference closure captures now
  preserve root/reference cells and promoted function-frame local cells.
- `103c0a4e`, `ff1d8ee3`, and `deabcd6d`: descriptor closures support selected
  variadic, typed/default by-value, and untyped by-reference parameters.

## Current Work Snapshot

Primary-integrated and counted:

- [x] Pushed primary progress remains wrapped by
  `28bb7423 docs: update progress after destructor blocker`.
- [x] Counted semantic baseline remains `b217e2b4`.
- [x] Overall and executable-semantics estimates remain 85%.

Local primary but uncounted:

- [ ] `2967110c codegen: expose symbol table abi probe` is local and unpushed.
  It exposes ABI helper declarations/probe calls, not generalized PHP
  variable assignment/readback.
- [ ] Dirty `preg_replace_callback()` WIP is not ready because the non-ASCII
  regex path is still exact-pattern production behavior.

Lane-local but uncounted:

- [ ] `object-property-reference-slots` is the strongest next candidate:
  executable object-property assignment/unset mutation through value/reference
  slots, from clean `28bb7423`, with `/tmp` gates and stable final hash.
- [ ] `object-arrayaccess-error-control-retry` is a fallback blocker/
  diagnostic classifier candidate, not executable `ArrayAccess`.
- [ ] Broad active lanes should be extracted into narrow candidates before
  primary review.

## Lane-Local Versus Primary

Lane-local work is useful only when it feeds a distinct primary integration.
Current likely-useful evidence includes object-property mutation reference-slot
runtime/compiler boundaries, object-offset ArrayAccess blocker classification,
extension metadata source/result handling, object-property liveness/unset
context dispatch, post-append reference assignment, byte-keyed globals/symbol
work, and cleanup/control-flow/diagnostic candidates.

Historical or already-landed surfaces should not be repeated: callable-array
public method-frame invocation, callable-object public `__invoke` dispatch,
static anonymous descriptor closures, static arrow descriptor closures,
non-static closure `$this` binding, dynamic declared-class `new` selected
cases, and the destructor-observable allocation blocker.

## Review Notes

Resource pressure is now a steering constraint. `/dev/shm` is 22G total, 21G
used, 1.8G available, 92% used; `du -sh /dev/shm` reports 21G. `/home` has
ample filesystem headroom at 459G total, 151G used, 289G available, with
`du -sh /home` reporting 116G. New gates should use disk-backed `/tmp` targets,
`CARGO_BUILD_JOBS=1`, `CARGO_INCREMENTAL=0`, and focused filters until shared
memory recovers.

The top supervisor should be aware that committing and pushing this progress
file on the live primary branch may also publish local ancestor commit
`2967110c` unless the wrapper deliberately handles that branch state.
