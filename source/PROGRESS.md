# PHP Native Compiler Progress

Updated: 2026-05-25 05:35 CEST
Evaluation marker: `20260525T033541Z`

Accounting rule: only generalized, tested, committed, and pushed primary work
counts as product capability. Dirty primary WIP, probe-only commits,
dashboard-only commits, lane-local candidates, historical worktrees,
blocker-only classifiers, and status-file claims are excluded until selected,
gated, committed, pushed, and reflected here as semantic product progress.

Current live pushed primary head observed by evaluator:
`348f232b docs: update progress dashboard`

Latest counted semantic/test baseline:
`b217e2b4 codegen: block destructor-observable native allocation`

Latest pushed but uncounted code work:
`2967110c codegen: expose symbol table abi probe`

## Executive Read

Overall estimated progress: **85%** `[#################---]`

Executable PHP semantics: **85%** `[#################---]`

Primary semantic capability did not move this review. Primary is synced with
`origin/master` at `348f232b`, and every pushed commit after the symbol-table
ABI probe is a progress/dashboard commit. The latest counted semantic baseline
remains the destructor-observable allocation blocker at `b217e2b4`.

The live primary checkout is dirty only in the two `preg_replace_callback()`
files. The dirty repair remains ready for primary review at hash
`52973e3185c8874b67c38e245e6b0c6497c2117ac826fcb46d092f4ea655e8b5`.
Focused preg tests, milestone841 PHP comparison, `cargo check`, scoped
rustfmt, scoped diff checks, regression audit, and shape audit support review.
It is still uncommitted and unpushed, so it is not counted here.

Lane-local progress remains real but uncounted. Fresh worker statuses sampled
around this review report request/root symbol reconciliation, dynamic
`$GLOBALS[$expr]` request-root routing, dynamic internal callback execution,
runtime-selected by-reference call planning, switch/goto/label cleanup-state
work, object/ArrayAccess diagnostic classification, object-property and
reference-cell mutation boundaries, request-state unset/payload conversion,
dynamic string-comparison conversion, parameter-binding diagnostic
classification, SPL exception metadata, and more. These are candidate
supplies, not product capability, until extracted, rechecked on current
primary, gated, committed, and pushed.

The integrated compiler has useful islands: descriptor closures, direct and
implicit captures, selected by-reference captures and parameters,
typed/default/variadic by-value closure parameters, static anonymous closures,
static arrows, non-static closure `$this` binding, callable-array public method
frames, callable-object public `__invoke` frames, selected runtime
string-valued declared-class construction, bounded public object features, and
destructor-observable allocation blocking for declared objects.

This is still selected PHP execution, not general PHP. The hard cliffs remain
full callable lookup/invocation, closure rebinding APIs, full PCRE behavior,
references/COW identity, request and `$GLOBALS` alias parity, includes,
variable variables, object visibility/magic/dynamic/static/typed property
behavior, runtime `ArrayAccess` method execution, userland method/constructor
frames, cleanup/unwind/finally/destructor execution, exact diagnostics, and
backend parity.

## Grand Roadmap Position

| Workstream | Estimate | Bar | Current read |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | **97%** | `[###################-]` | Strong shared value, array, string, diagnostic, reference, symbol, call-frame, object, method, callable-array, and descriptor-closure surfaces exist for selected paths. Symbol-table ABI visibility exists, but executable symbol storage is not generalized in counted primary. |
| Compiler/backend consumers | **97%** | `[###################-]` | Generated C consumes many shared ABIs; LLVM and C assembly consume selected ABI families. Object lowering, direct assembly parity, and many nested consumers remain blocked. |
| Executable PHP semantics | **85%** | `[#################---]` | Focused linked/runtime programs cover closure/callable/object islands. Dirty preg work and lane-local callback/ArrayAccess/symbol/control-flow progress are not counted yet. |
| Arrays, lvalues, references, COW | **67%** | `[#############-------]` | Useful selected lvalue/reference paths feed closures and lane-local mutation work; full COW, arbitrary roots, foreach, property references, and alias composition remain open. |
| Symbols, globals, request state | **72%** | `[##############------]` | Function globals and `$GLOBALS` self-import blockers improved. Executable symbol-table storage, request imports, includes, variable variables, exact unset/global alias behavior, and reconciliation remain incomplete. |
| Calls, functions, frames | **82%** | `[################----]` | Bounded functions, descriptor closures, callable arrays, callable objects, public method frames, and constructor frames work in selected generated-C cases. Direct-call/frame extraction and broad callback execution remain lane-local. |
| Objects, properties, methods | **45%** | `[#########-----------]` | Useful public declared-object subset exists and supported public `__invoke` frames are callable. Non-public/contextual visibility, overrides, interfaces/traits, magic methods, dynamic/static/typed properties, destructors, references/COW, and backend parity remain open. |
| Control flow, cleanup, diagnostics | **49%** | `[##########----------]` | Selected branches, loops, transfers, finalizers, output buffers, diagnostics, and destructor blockers exist; broad unwind, handlers, destructor execution, shutdown flushing, and source-ordered diagnostics remain open. |
| Broad integrated verification | **84%** | `[#################---]` | Focused gates are strong for recent counted slices. Broad dirty-checkout gates remain constrained by preg WIP, stale management artifacts, lane extraction cost, and backend parity gaps. |

## Active Roadmap Items

| Item | Toward Primary Integration | Toward Full Feature | Status |
| --- | ---: | ---: | --- |
| Dirty `preg_replace_callback()` repair | **95%** `[###################-]` | **32%** `[######--------------]` | Ready for primary review at hash `52973e3185c8874b67c38e245e6b0c6497c2117ac826fcb46d092f4ea655e8b5`. It executes string callbacks over a bounded slash-delimited regex subset. Still uncounted until committed and pushed. |
| Lane-local object/ArrayAccess/method dispatch policy boundaries | **58%** `[############--------]` | **28%** `[######--------------]` | Fresh lane evidence reports operation-discriminated ArrayAccess diagnostics, method-dispatch policy, constructor-dispatch policy, and `$this` context blockers. It still does not execute `offsetGet`, `offsetExists`, `offsetSet`, `offsetUnset`, userland methods, or constructors. |
| Object-offset `ArrayAccess` error-control classifier | **88%** `[##################--]` | **10%** `[##------------------]` | Focused one-file compiler classifier candidate. It improves unsupported object-offset diagnostic routing; it is not runtime `ArrayAccess`. Recheck hash/apply state after the preg decision. |
| Object-property reference-slot mutation | **78%** `[################----]` | **35%** `[#######-------------]` | Strong executable object mutation candidate at hash `c60ed1c30dc1d979da1fc44641ae4378a3629e45c42da0d9621faf10519e56e8`. Latest triage still calls out formatting repair and fresh current-primary gates before integration. |
| Symbol-table storage and request/global reconciliation | **30%** `[######--------------]` | **25%** `[#####---------------]` | ABI/runtime helper visibility exists through `2967110c`; lane-local request/root reconciliation is promising, but generalized PHP variable assignment/readback through symbol tables is still not implemented in counted primary. |
| Direct calls, callbacks, and frame extraction | **52%** `[##########----------]` | **40%** `[########------------]` | Lane-local direct-call classification, runtime-selected by-reference planning, constructor lookup ordering, callable registry diagnostics, and internal callback execution are semantically useful. Real PHP function/method body execution and broad by-reference/return propagation remain unintegrated. |
| Conditional/control-flow value consumers | **45%** `[#########-----------]` | **38%** `[########------------]` | Lane-local switch/goto/loop cleanup-state work and short-ternary/probe work have focused gates. Mutation-capable state, full cleanup/unwind, and general expression merges remain open. |
| Type-conversion, regex, string, request, and formatted-output work | **52%** `[##########----------]` | **34%** `[#######-------------]` | Lane-local surfaces include JSON flags, resource lifecycle, INI state, object casts, class-property metadata, dynamic internal callbacks, regex parameter validation, request connection-state builtins, and byte/string/serialization work. Extract one focused executable candidate at a time. |
| Broad lane extraction backlog | **30%** `[######--------------]` | **30%** `[######--------------]` | Many lanes report generalized progress, but several carry huge conflict-heavy worktrees. Treat them as mines for compact slices, not as patches to import. |

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
- [ ] Lane-local object/ArrayAccess/method-dispatch policy boundaries; promising but still blocker/policy infrastructure, not executable userland methods.
- [ ] Lane-local `object-arrayaccess-error-control-retry` diagnostic classifier candidate; recheck hash/apply state after the preg decision.
- [ ] Lane-local `object-property-reference-slots` mutation/reference-slot candidate; repair formatting and refresh current-primary apply proof and gates before integration.
- [ ] Lane-local request/root symbol reconciliation, dynamic internal callbacks, direct-call/frame, JSON/type-conversion/resource/INI/object-cast/class-property, regex, control-flow, cleanup-contract, diagnostic sequencing, comparison, object-policy, symbol/global, and array/reference candidates that need clean extraction.

Not done:

- [ ] Full callable lookup and invocation beyond selected strings, closures, arrays, and public `__invoke` objects, including magic/visibility/rebinding rules.
- [ ] Runtime `ArrayAccess` method dispatch for `offsetGet`, `offsetExists`, `offsetSet`, and `offsetUnset`.
- [ ] Full references/COW identity and arbitrary alias roots.
- [ ] Request and `$GLOBALS` parity, includes, variable variables, and dynamic symbol behavior.
- [ ] General object model: non-public methods, overrides, interfaces/traits, magic methods, dynamic/static/typed properties, destructors.
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
- [x] Live `HEAD` and `origin/master` are synced at `348f232b`.

Pushed but uncounted:

- [ ] `2967110c codegen: expose symbol table abi probe` exposes helper declarations/probe calls but does not execute generalized PHP symbol storage.
- [ ] Dashboard/progress commits through `348f232b` update observability only.

Dirty primary but uncounted:

- [ ] `preg_replace_callback()` WIP is ready for primary review, but remains dirty and uncounted.
- [ ] Current evidence diff hash is `52973e3185c8874b67c38e245e6b0c6497c2117ac826fcb46d092f4ea655e8b5`.
- [ ] Dirty files remain exactly `compiler/src/interpreter.rs` and `compiler/tests/preg_replace_callback_builtin.rs`.
- [ ] Packet gate, regression audit, shape audit, `cargo check`, scoped rustfmt, scoped diff checks, and milestone841 PHP comparison support review at that hash.

Lane-local but uncounted:

- [ ] Object/ArrayAccess/method-dispatch policy boundaries have fresh multi-operation evidence, but remain lane-local and not full method-frame or `ArrayAccess` execution.
- [ ] Request/root symbol reconciliation, dynamic internal callback execution, runtime-selected by-reference call planning, switch/goto/loop cleanup-state work, JSON/resource/INI/object-cast/class-property surfaces, and direct-variable probe consolidation are useful lane evidence but not primary-integrated.
- [ ] `object-arrayaccess-error-control-retry` remains a low-risk classifier candidate after live hash/apply recheck.
- [ ] `object-property-reference-slots` remains the stronger executable object candidate after formatting repair, current-primary recheck, and rerun gates.

## Review Notes

Resource pressure is serviceable but guarded. Live `/dev/shm` is `40G` total,
`24G` used, `17G` available, `58%` used; `du -sh /dev/shm` reports `24G`.
The `/home` filesystem has `459G` total, `222G` used, `219G` available,
`51%` used by `df`; bounded `du -sh /home` timed out and emitted permission
noise under container overlay paths, so `df` is the better current pressure
signal. Swap remains high: `23Gi` used out of `29Gi`.

Advisory steering read: resolve the dirty preg decision first. Then integrate
only one narrow current-primary candidate at a time, with live hash/apply
verification and focused disk-backed gates. Treat fresh lane reports as
candidate supply, not counted product capability, and refresh stale management
artifacts before using them for steering.
