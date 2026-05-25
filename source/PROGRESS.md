# PHP Native Compiler Progress

Updated: 2026-05-25 07:00 CEST
Evaluation marker: `20260525T045836Z`

Accounting rule: only generalized, tested, committed, and pushed primary work
counts as product capability. Dirty primary WIP, probe-only commits,
dashboard-only commits, lane-local candidates, historical worktrees,
blocker-only classifiers, and status-file claims are excluded until selected,
gated, committed, pushed, and reflected here as semantic product progress.

Current live pushed primary head observed by evaluator:
`94ee1a7c docs: update progress dashboard`

Latest counted semantic/test baseline:
`b217e2b4 codegen: block destructor-observable native allocation`

Latest pushed but uncounted code work:
`2967110c codegen: expose symbol table abi probe`

## Executive Read

Overall estimated progress: **85%** `[#################---]`

Executable PHP semantics: **85%** `[#################---]`

Primary semantic capability did not move in this review. Primary is synced
with `origin/master` at `94ee1a7c`; recent pushed primary commits after the
destructor blocker are dashboard/progress updates plus the uncounted symbol
table ABI probe. The latest counted semantic baseline remains
`b217e2b4`.

The live primary checkout is dirty only in the two `preg_replace_callback()`
files. The repair remains ready for primary review at hash
`52973e3185c8874b67c38e245e6b0c6497c2117ac826fcb46d092f4ea655e8b5`.
Packet gate, independent regression audit, shape audit, focused preg tests,
milestone841 PHP comparison, `cargo check`, scoped rustfmt, and scoped diff
checks support review. It remains uncommitted and unpushed, so it is not
counted here.

Lane-local work is active but uncounted. Fresh statuses report candidates
around callable `sscanf()` output references, object/closure-to-bool loose
comparison, interpreter byte-keyed dynamic-property reads, dynamic byte-string
ternary sources, `ignore_user_abort()`, formatted/stream output, root-symbol
truthy/numeric flags, nested symbol-operation metadata, object/property
ArrayAccess admission, owner-cell by-reference foreach ABIs, and switch/loop
target-state materialization. These are candidate supplies, not product
capability, until one compact slice is extracted, rechecked on current
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
behavior, runtime `ArrayAccess` method execution, userland
method/constructor/function frames, cleanup/unwind/finally/destructor/output
buffer ordering, exact diagnostics, and backend parity.

## Grand Roadmap Position

| Workstream | Estimate | Bar | Current read |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | **97%** | `[###################-]` | Strong shared value, array, string, diagnostic, reference, symbol, call-frame, object, method, callable-array, descriptor-closure, comparison, and conversion surfaces exist for selected paths. Many lane ABI expansions remain uncounted. |
| Compiler/backend consumers | **97%** | `[###################-]` | Generated C consumes many shared ABIs; LLVM and C assembly consume selected ABI families. Object lowering, assembly parity, and nested consumers remain blocked. |
| Executable PHP semantics | **85%** | `[#################---]` | Focused linked/runtime programs cover closure/callable/object islands. Dirty preg work and lane-local callback/ArrayAccess/symbol/control-flow/string progress are not counted yet. |
| Arrays, lvalues, references, COW | **67%** | `[#############-------]` | Useful selected lvalue/reference paths feed closures and lane-local mutation/foreach work. Full COW, arbitrary roots, foreach, property references, and alias composition remain open. |
| Symbols, globals, request state | **72%** | `[##############------]` | Function globals and `$GLOBALS` self-import blockers improved. Executable symbol-table storage, request imports, includes, variable variables, exact unset/global alias behavior, and reconciliation remain incomplete. |
| Calls, functions, frames | **82%** | `[################----]` | Bounded functions, descriptor closures, callable arrays, callable objects, public method frames, and constructor frames work in selected generated-C cases. Direct-call/frame extraction and broad callback execution remain lane-local. |
| Objects, properties, methods | **45%** | `[#########-----------]` | Useful public declared-object subset exists and supported public `__invoke` frames are callable. Non-public/contextual visibility, overrides, interfaces/traits, magic methods, dynamic/static/typed properties, destructors, references/COW, and backend parity remain open. |
| Control flow, cleanup, diagnostics | **49%** | `[##########----------]` | Selected branches, loops, transfers, finalizers, output buffers, diagnostics, and destructor blockers exist. Broad unwind, handlers, destructor execution, shutdown flushing, and source-ordered diagnostics remain open. |
| Broad integrated verification | **84%** | `[#################---]` | Focused gates are strong for recent counted slices. Broad dirty-checkout gates remain constrained by preg WIP, stale dashboard state, lane extraction cost, and backend parity gaps. |

## Active Roadmap Items

| Item | Toward Primary Integration | Toward Full Feature | Status |
| --- | ---: | ---: | --- |
| Dirty `preg_replace_callback()` repair | **96%** `[###################-]` | **32%** `[######--------------]` | Ready for primary review at hash `52973e3185c8874b67c38e245e6b0c6497c2117ac826fcb46d092f4ea655e8b5`. Executes string callbacks over a bounded slash-delimited regex subset. Still uncounted until committed and pushed. |
| Compact `impl-native-integration-batch` candidates | **66%** `[#############-------]` | **25%** `[#####---------------]` | Fresh lane candidates include callable `sscanf()` output references, object-like loose comparison, interpreter byte-keyed dynamic-property reads, and type-predicate value-result materialization. Extract one clean slice; do not import the accumulated worktree. |
| Callable, scanner, stream, and request-state builtins | **58%** `[############--------]` | **39%** `[########------------]` | Lane-local callable `sscanf()`, `fscanf()`, `fprintf()`/`vfprintf()`, `ignore_user_abort()`, hash, bitwise, and formatted-output work is active. Good candidate supply, but broad callable/userland/method dispatch remains incomplete. |
| Symbol-table storage and request/global reconciliation | **34%** `[#######-------------]` | **28%** `[######--------------]` | Root-symbol truthy/numeric flags, nested operation metadata, and linked scalar symbol-table routing are active lane work. Generalized PHP variable storage, includes, request/global aliases, and `$GLOBALS` parity are not counted. |
| Direct calls, callbacks, and frame extraction | **57%** `[###########---------]` | **43%** `[#########-----------]` | Shared call argument/reference/result-consumer and constructor allocation boundaries are stronger. Broad PHP callable/function/method body execution remains unintegrated. |
| Object/ArrayAccess/method dispatch boundaries | **61%** `[############--------]` | **30%** `[######--------------]` | Shared object-offset, stable property receiver, dynamic-property policy, and reference-assignment boundaries improved. Runtime `offsetGet`, `offsetExists`, `offsetSet`, and `offsetUnset` method-frame execution is still not integrated. |
| Object-offset `ArrayAccess` error-control classifier | **88%** `[##################--]` | **10%** `[##------------------]` | Focused one-file compiler classifier candidate. Improves unsupported object-offset diagnostic routing; it is not runtime `ArrayAccess`. Recheck hash/apply state after the preg decision. |
| Object-property reference-slot mutation | **78%** `[################----]` | **35%** `[#######-------------]` | Strong executable object mutation candidate at hash `c60ed1c30dc1d979da1fc44641ae4378a3629e45c42da0d9621faf10519e56e8`. Needs formatting repair and fresh current-primary gates before integration. |
| Reference/foreach owner-cell ABI candidates | **50%** `[##########----------]` | **34%** `[#######-------------]` | Lane-local owner-cell by-reference foreach cursor/element, borrow/apply, branch cleanup, and mutation-effect cleanup ABIs are richer. Production foreach/source provider lowering and alias/COW execution remain blocked. |
| Conditional/control-flow value consumers | **54%** `[###########---------]` | **43%** `[#########-----------]` | Lane-local switch/loop/goto target-state, nested transfer, label cleanup, and condition-reference blocker work has focused gates. Mutation-capable state, full cleanup/unwind, and general expression merges remain open. |
| Type-conversion, binary string, regex, and formatted output | **56%** `[###########---------]` | **38%** `[########------------]` | Lane-local dynamic byte strings, dynamic string comparison, formatted-string parsing, internal callbacks, object casts, regex cleanup blockers, request conversion, and byte/string/serialization work continues. Extract one focused executable candidate at a time. |
| Broad lane extraction backlog | **28%** `[######--------------]` | **30%** `[######--------------]` | Many lanes report generalized progress, but several carry huge conflict-heavy worktrees. Treat them as mines for compact slices, not as patches to import. |

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

- [ ] Dirty bounded `preg_replace_callback()` callback-execution repair; ready for primary review and gated at hash `52973e3185c8874b67c38e245e6b0c6497c2117ac826fcb46d092f4ea655e8b5`.
- [ ] Pushed symbol-table ABI probe `2967110c`; useful visibility, not generalized PHP symbol storage.
- [ ] Lane-local callable scanner/reference-output, formatted stream-output, request-state, bitwise, and dynamic byte-string/string-comparison candidates.
- [ ] Lane-local generated-C type-predicate value-result materialization, runtime object-like loose comparison, and byte-keyed dynamic-property read candidates.
- [ ] Lane-local root-symbol truthy/numeric flags, nested symbol-operation metadata, scalar linked-symbol routing, and request/global diagnostic sequencing.
- [ ] Lane-local object/ArrayAccess/method-dispatch policy boundaries; promising but still not full method-frame or `ArrayAccess` execution.
- [ ] Lane-local owner-cell by-reference foreach and mutation-effect cleanup ABIs; production source-provider lowering remains blocked.
- [ ] Lane-local `object-arrayaccess-error-control-retry` diagnostic classifier candidate; recheck hash/apply state after the preg decision.
- [ ] Lane-local `object-property-reference-slots` mutation/reference-slot candidate; repair formatting and refresh current-primary apply proof and gates before integration.

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
- [x] Live `HEAD` and `origin/master` are synced at `94ee1a7c`.

Pushed but uncounted:

- [ ] `2967110c codegen: expose symbol table abi probe` exposes helper declarations/probe calls but does not execute generalized PHP symbol storage.
- [ ] Dashboard/progress commits through `94ee1a7c` update observability only.

Dirty primary but uncounted:

- [ ] `preg_replace_callback()` WIP is ready for primary review, but remains dirty and uncounted.
- [ ] Current evidence diff hash is `52973e3185c8874b67c38e245e6b0c6497c2117ac826fcb46d092f4ea655e8b5`.
- [ ] Dirty files remain exactly `compiler/src/interpreter.rs` and `compiler/tests/preg_replace_callback_builtin.rs`.
- [ ] Packet gate, regression audit, shape audit, `cargo check`, scoped rustfmt, scoped diff checks, and milestone841 PHP comparison support review at that hash.

Lane-local but uncounted:

- [ ] `impl-native-integration-batch` has multiple compact candidates, but its accumulated worktree should not be imported wholesale.
- [ ] Object/ArrayAccess/method-dispatch policy boundaries have fresh evidence, but remain lane-local and not full method-frame or `ArrayAccess` execution.
- [ ] Request/root metadata, linked diagnostics, dynamic internal callback execution, receiver-method lookup preflight, switch/goto/loop cleanup-state work, JSON/resource/INI/object-cast/class-property surfaces, scanner/reference-output work, and direct-variable probe consolidation are useful lane evidence but not primary-integrated.
- [ ] `object-arrayaccess-error-control-retry` remains a low-risk classifier candidate after live hash/apply recheck.
- [ ] `object-property-reference-slots` remains the stronger executable object candidate after formatting repair, current-primary recheck, and rerun gates.

## Review Notes

Resource pressure is serviceable but guarded. Live `/dev/shm` is `40G` total,
`24G` used, `17G` available, `58%` used; `du -sh /dev/shm` reports `24G`.
The `/home` filesystem has `459G` total, `197G` used, `244G` available,
`45%` used by `df`; `du -sh /home` reported `123G` but exited nonzero because
of unreadable container overlay paths, so use it only as a lower-bound sample.
Swap remains high: `23Gi` used out of `29Gi`.

The supervisor dashboard is stale relative to live worker statuses: dashboard
last update is `2026-05-25 01:56 CEST`, while fresh worker status mtimes are
around `06:48-06:58 CEST`. Use current git state, worker artifacts, and
evaluator reports for steering until the dashboard is refreshed.

Advisory steering read: resolve the dirty preg decision first. Then integrate
only one narrow current-primary candidate at a time, with live hash/apply
verification and focused disk-backed gates. Treat fresh lane reports as
candidate supply, not counted product capability. Keep broad gates disk-backed
while swap remains high.
