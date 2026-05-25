# PHP Native Compiler Progress

Updated: 2026-05-25 03:13 CEST
Evaluation marker: `20260525T011111Z`

Accounting rule: only generalized, tested, committed, and pushed primary work
counts as product capability. Dirty primary WIP, probe-only commits,
dashboard-only commits, lane-local candidates, historical worktrees,
blocker-only classifiers, and status-file claims are excluded until selected,
gated, committed, pushed, and reflected here as semantic product progress.

Current live pushed primary head:
`570c69ca docs: update progress dashboard`

Latest counted semantic/test baseline:
`b217e2b4 codegen: block destructor-observable native allocation`

## Executive Read

Overall estimated progress: **85%** `[#################---]`

Executable PHP semantics: **85%** `[#################---]`

Primary capability did not move this review. The live primary head advanced
only through dashboard/progress commits. The important primary-local work is
still the dirty, gated `preg_replace_callback()` repair at hash
`52973e3185c8874b67c38e245e6b0c6497c2117ac826fcb46d092f4ea655e8b5`; it remains
uncommitted and unpushed, so it is not counted.

The integrated compiler has real islands: descriptor closures, direct and
implicit captures, selected by-reference captures and parameters,
typed/default/variadic by-value closure parameters, static anonymous closures,
static arrows, non-static closure `$this` binding, callable-array public method
frames, callable-object public `__invoke` frames, selected runtime
string-valued declared-class construction, and destructor-observable allocation
blocking for declared objects.

This is still selected PHP execution, not general PHP. The hard cliffs remain
full callable lookup/invocation, closure rebinding APIs, full PCRE behavior,
references/COW identity, request and `$GLOBALS` alias parity, includes,
variable variables, object visibility/magic/dynamic/static property behavior,
runtime `ArrayAccess` method execution, cleanup/unwind/finally/destructor
execution, exact diagnostics, and backend parity.

## Grand Roadmap Position

| Workstream | Estimate | Bar | Current read |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | **97%** | `[###################-]` | Strong shared value, array, string, diagnostic, reference, symbol, call-frame, object, method, callable-array, and descriptor-closure surfaces exist for selected paths. Symbol-table ABI visibility exists, but executable symbol storage is not generalized. |
| Compiler/backend consumers | **97%** | `[###################-]` | Generated-C consumes many shared ABIs; LLVM and C assembly consume selected ABI families. Object lowering, direct assembly parity, and many nested consumers remain blocked. |
| Executable PHP semantics | **85%** | `[#################---]` | Focused linked/runtime programs cover many closure/callable/object islands, but execution is selected rather than general PHP. Dirty preg work is not counted yet. |
| Arrays, lvalues, references, COW | **67%** | `[#############-------]` | Useful selected lvalue/reference paths feed closures and some lane-local mutation work; full COW, arbitrary roots, foreach, property references, and alias composition remain open. |
| Symbols, globals, request state | **72%** | `[##############------]` | Function globals and `$GLOBALS` self-imports improved. Request imports, includes, variable variables, exact unset/global alias behavior, and executable symbol-table lowering remain incomplete. |
| Calls, functions, frames | **82%** | `[################----]` | Bounded functions, descriptor closures, callable arrays, callable objects, public method frames, and constructor frames work in selected generated-C cases. Direct-call/frame extraction remains lane-local. |
| Objects, properties, methods | **45%** | `[#########-----------]` | Useful public declared-object subset exists and supported public `__invoke` frames are callable. Non-public/contextual visibility, overrides, interfaces/traits, magic methods, dynamic/static properties, destructors, references/COW, and backend parity remain open. |
| Control flow, cleanup, diagnostics | **49%** | `[##########----------]` | Selected branches, loops, transfers, finalizers, output buffers, diagnostics, and destructor blockers exist; broad unwind, handlers, destructor execution, shutdown flushing, and source-ordered diagnostics remain open. |
| Broad integrated verification | **84%** | `[#################---]` | Focused gates are strong for recent counted slices. Broad dirty-checkout gates remain constrained by preg WIP, sequencing, and backend parity gaps. |

## Active Roadmap Items

| Item | Toward Primary Integration | Toward Full Feature | Status |
| --- | ---: | ---: | --- |
| Dirty `preg_replace_callback()` repair | **92%** `[##################--]` | **30%** `[######--------------]` | Ready for primary review at hash `52973e3185c8874b67c38e245e6b0c6497c2117ac826fcb46d092f4ea655e8b5`. Packet gate, regression audit, shape audit, `cargo check`, rustfmt, and milestone841 PHP comparison passed. Still uncounted until committed and pushed. |
| Lane-local object-offset `ArrayAccess` dispatch ABI | **45%** `[#########-----------]` | **20%** `[####----------------]` | Fresh lane-local work routes object offset reads, liveness, writes, appends, unsets, null-coalesce operations, compound assignment, and inc/dec through a shared dispatch ABI with focused gates. It still needs compact current-primary extraction and does not execute offset method frames yet. |
| Object-offset ArrayAccess error-control classifier | **88%** `[##################--]` | **10%** `[##------------------]` | Best isolated non-preg candidate if the supervisor wants a low-risk diagnostic/blocker slice. This is a one-file compiler classifier, not runtime `ArrayAccess`. Recheck hash/apply state before integration. |
| Object-property reference-slot mutation | **78%** `[################----]` | **35%** `[#######-------------]` | Strong executable object mutation candidate at hash `c60ed1c30dc1d979da1fc44641ae4378a3629e45c42da0d9621faf10519e56e8`, but evidence still has a formatting/current-primary disagreement. Needs refreshed hash and focused `/tmp` gates. |
| Symbol-table executable storage | **20%** `[####----------------]` | **20%** `[####----------------]` | ABI/runtime helper visibility exists through `2967110c`; generalized PHP variable assignment/readback through symbol tables is still not implemented in counted primary. |
| Direct user-function call/frame extraction | **45%** `[#########-----------]` | **35%** `[#######-------------]` | Lane-local direct-call/user-frame work looks semantically useful, but it is cumulative lane work. Extract a compact current-primary candidate with stable hash and apply proof before review. |
| String-result reference-slot extraction | **45%** `[#########-----------]` | **25%** `[#####---------------]` | Lane-local integration-batch work routes lowerable generated-C string-result operations through value/reference slots and has focused gates. It still needs compact extraction and overlap audit before primary review. |
| `http_build_query()` string-result extraction | **40%** `[########------------]` | **25%** `[#####---------------]` | Lane-local runtime/compiler/backend work has executable evidence, but needs a focused candidate and overlap audit against adjacent string, JSON, request, output-buffer, and callable work. |
| JSON/type-conversion/debug-output extraction | **40%** `[########------------]` | **25%** `[#####---------------]` | Lane-local type-conversion work has useful debug-output, resource, JSON, and conversion-result surfaces. Extract one focused executable candidate at a time. |
| Control-flow/cleanup target-state extraction | **35%** `[#######-------------]` | **45%** `[#########-----------]` | Lane-local switch/loop/exception/request blocker and target-state work has useful gates, but needs narrow current-primary extraction. |
| Broad lane extraction backlog | **30%** `[######--------------]` | **30%** `[######--------------]` | Many lanes report generalized progress, but most remain broad dirty worktrees or status-only evidence until sliced, applied, gated, and pushed. |

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

- [ ] Pushed symbol-table ABI probe `2967110c`; useful visibility, not generalized PHP symbol storage.
- [ ] Dirty bounded `preg_replace_callback()` callback-execution repair; ready for primary review and gated at hash `52973e3185c8874b67c38e245e6b0c6497c2117ac826fcb46d092f4ea655e8b5`.
- [ ] Lane-local object-offset `ArrayAccess` dispatch ABI candidate; promising but not extracted to current primary and still not executable offset method dispatch.
- [ ] Lane-local `object-arrayaccess-error-control-retry` diagnostic classifier candidate; recheck hash/apply state before integration.
- [ ] Lane-local `object-property-reference-slots` mutation/reference-slot candidate; resolve formatting/current-primary disagreement before integration.
- [ ] Lane-local direct-call/frame, string-result reference-slot, `http_build_query()`, JSON/type-conversion/debug-output, constructor/call preflight, control-flow, cleanup-contract, diagnostic sequencing, comparison, object-policy, symbol/global, and array/reference candidates that need clean extraction.

Not done:

- [ ] Full callable lookup and invocation beyond selected strings, closures, arrays, and public `__invoke` objects, including magic/visibility/rebinding rules.
- [ ] Runtime `ArrayAccess` method dispatch for `offsetGet`, `offsetExists`, `offsetSet`, and `offsetUnset`.
- [ ] Full references/COW identity and arbitrary alias roots.
- [ ] Request and `$GLOBALS` parity, includes, variable variables, and dynamic symbol behavior.
- [ ] General object model: non-public methods, overrides, interfaces/traits, magic methods, dynamic/static properties, typed properties, destructors.
- [ ] Complete cleanup/unwind/finally/destructor/output-buffer shutdown behavior.
- [ ] Exact/source-ordered diagnostics and warning/error continuation.
- [ ] LLVM/direct assembly parity for recent generated-C semantics.

## Recent Primary-Integrated Work

- `b217e2b4`: generated-C declared-object allocation now blocks
  destructor-observable native allocation before emitting allocation branches.
  Destructor declarations are recorded as declared-class metadata, inherited
  through class hierarchy lookup, and checked against runtime string-valued
  dynamic class-name facts. This is cleanup/unwind safety only.
- `7679dc0e`: generated-C runtime dynamic calls can invoke supported declared
  callable objects through public `__invoke` method frames.
- `53c8a283`: supported non-static regular closures and arrows created inside
  active object method/constructor frames bind `$this` through the shared
  descriptor capture/callback path.
- `8f5d8fb3` and `79496862`: supported static arrow and static anonymous
  descriptor closures reuse the shared descriptor closure stack.
- `7a43e1ac`: runtime dynamic calls can invoke syntax-valid callable arrays
  that resolve to supported generated public static/object method frames.

## Current Work Snapshot

Primary-integrated and counted:

- [x] Counted semantic baseline remains `b217e2b4`.
- [x] Overall and executable-semantics estimates remain 85%.

Pushed but uncounted:

- [ ] `28bb7423 docs: update progress after destructor blocker` records progress metadata.
- [ ] `2967110c codegen: expose symbol table abi probe` exposes helper declarations/probe calls but does not execute generalized PHP symbol storage.
- [ ] `59343e11`, `16209c55`, `12fe04ea`, `9417fcfa`, `4aa6c24c`, `e4ad066f`, `1171f1e0`, `c745dd47`, `d2656ff5`, and `570c69ca` are dashboard/progress metadata only.

Dirty primary but uncounted:

- [ ] `preg_replace_callback()` WIP is ready for primary review, but remains dirty and uncounted.
- [ ] Current live diff hash is `52973e3185c8874b67c38e245e6b0c6497c2117ac826fcb46d092f4ea655e8b5`.
- [ ] Dirty files remain exactly `compiler/src/interpreter.rs` and `compiler/tests/preg_replace_callback_builtin.rs`.
- [ ] Packet gate, regression audit, and shape audit all support review at that hash.

Lane-local but uncounted:

- [ ] Fresh object-offset `ArrayAccess` dispatch ABI work has useful runtime/compiler/linked gates, but it is still lane-local and not full method-frame execution.
- [ ] `object-arrayaccess-error-control-retry` remains a low-risk classifier candidate after hash/apply recheck.
- [ ] `object-property-reference-slots` remains the stronger executable object candidate after formatting repair and rerun gates.
- [ ] Fresh lanes report useful generalized surfaces including string-result reference slots, request try/loop blockers, owner-cell diagnostic source conformance, native debug-output resource formatting, callable/string/numeric reference-slot routing, and symbol/global work. None count until compactly extracted, gated, committed, and pushed.

## Review Notes

Resource pressure is serviceable but still relevant. `/dev/shm` is `40G`
total, `24G` used, `17G` available, `58%` used; `du -sh /dev/shm` reports
`24G`. The `/home` filesystem has `459G` total, `189G` used, `251G`
available, `43%` used. `du -sh /home` returned `121G` with permission
warnings, so `df` is the better pressure signal.

Advisory steering read: resolve the dirty preg decision first. Then integrate
only one narrow current-primary candidate at a time, with live hash/apply
verification and focused disk-backed gates. The supervisor dashboard content is
stale relative to current worker statuses, so use fresh status artifacts and
live git checks for steering.
