# PHP Native Compiler Progress

Updated: 2026-05-25 08:41 CEST
Evaluation marker: `20260525T064144Z`

Accounting rule: only generalized, tested, committed, and pushed primary work
counts as product capability. Dirty primary WIP, probe-only commits,
dashboard-only commits, lane-local candidates, historical worktrees,
blocker-only classifiers, and status-file claims are excluded until selected,
gated, committed, pushed, and reflected here as semantic product progress.

Current live pushed primary head observed by evaluator:
`34922783 docs: update progress dashboard`

Latest counted semantic/test baseline:
`b217e2b4 codegen: block destructor-observable native allocation`

Latest pushed but uncounted code work:
`2967110c codegen: expose symbol table abi probe`

## Executive Read

Overall estimated progress: **85%** `[#################---]`

Executable PHP semantics: **85%** `[#################---]`

Primary semantic capability did not move in this review. Primary is synced
with `origin/master` at `34922783`; the newest pushed primary commit is another
progress/dashboard update. The latest counted semantic baseline remains
`b217e2b4`.

The live primary checkout is dirty only in the two `preg_replace_callback()`
files. The repair remains ready for primary review at hash
`52973e3185c8874b67c38e245e6b0c6497c2117ac826fcb46d092f4ea655e8b5`, with
current diffstat `2 files changed, 1367 insertions(+), 312 deletions(-)`.
Packet gate, independent regression audit, shape audit, focused preg tests,
milestone841 PHP comparison, `cargo check`, scoped rustfmt, and scoped diff
checks support review. It remains uncommitted and unpushed, so it is not
counted here.

Lane-local work is active but uncounted. Fresh statuses include call-argument
ordering and by-reference capture cleanup, control-flow value-slot and cleanup
blockers, array/reference cleanup preflights, type conversion, root-symbol and
`global` root-reference candidates, strict identity consumers, object/
ArrayAccess receiver classifiers, contextual `self` typed-property handling,
object-valued `instanceof`, stream/filesystem resources, shutdown callbacks,
and diagnostic boundaries. These are candidate supplies, not product
capability, until one compact slice is extracted, rechecked on current primary,
gated, committed, and pushed.

This is still selected PHP execution, not general PHP. The hard cliffs remain
full callable lookup/invocation, closure rebinding APIs, full PCRE behavior,
references/COW identity, request and `$GLOBALS` alias parity, includes,
variable variables, object visibility/magic/dynamic/static/typed property
behavior, runtime `ArrayAccess` method execution, userland method/constructor/
function frames, cleanup/unwind/finally/destructor/output-buffer ordering,
exact diagnostics, and backend parity.

## Grand Roadmap Position

| Workstream | Estimate | Bar | Current read |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | **97%** | `[###################-]` | Strong shared value, array, string, diagnostic, reference, symbol, call-frame, object, method, callable-array, descriptor-closure, comparison, conversion, owner-cell, and request-state surfaces exist for selected paths. Many fresh lane ABI expansions remain uncounted. |
| Compiler/backend consumers | **97%** | `[###################-]` | Generated C consumes many shared ABIs; LLVM and C assembly consume selected ABI families. Fresh root-symbol exists/global-import/strict-identity consumers remain lane-local until primary integration. |
| Executable PHP semantics | **85%** | `[#################---]` | Focused linked/runtime programs cover closure/callable/object islands. Dirty preg work and lane-local callback/ArrayAccess/symbol/control-flow/string progress are not counted yet. |
| Arrays, lvalues, references, COW | **67%** | `[#############-------]` | Useful selected lvalue/reference paths feed closures and lane-local mutation/foreach work. Full COW, arbitrary roots, foreach, property references, and alias composition remain open. |
| Symbols, globals, request state | **72%** | `[##############------]` | Function globals and `$GLOBALS` self-import blockers improved. Lane work has root-reference import, root-symbol exists, activation diagnostics, dynamic globals append shape, POST body SAPI population, and request-state candidates. Generalized symbol storage, includes, variable variables, exact unset/global alias behavior, and reconciliation remain incomplete. |
| Calls, functions, frames | **82%** | `[################----]` | Bounded functions, descriptor closures, callable arrays, callable objects, public method frames, and constructor frames work in selected generated-C cases. Spread/named/by-reference planning and structured signature metadata remain lane-local. |
| Objects, properties, methods | **45%** | `[#########-----------]` | Useful public declared-object subset exists and supported public `__invoke` frames are callable. Lane work adds contextual `self` typed properties, object-valued `instanceof`, and receiver blockers. Non-public/contextual visibility, overrides, interfaces/traits execution, magic methods, dynamic/static/typed properties, destructors, references/COW, and backend parity remain open. |
| Control flow, cleanup, diagnostics | **49%** | `[##########----------]` | Selected branches, loops, transfers, finalizers, output buffers, diagnostics, and destructor blockers exist. Request cleanup and diagnostic boundary lanes are active, but broad unwind, handlers, destructor execution, shutdown ordering, and source-ordered diagnostics remain open. |
| Broad integrated verification | **84%** | `[#################---]` | Focused gates are strong for recent counted slices. Broad dirty-checkout gates remain constrained by preg WIP, stale dashboard state, lane extraction cost, high swap, and backend parity gaps. |

## Active Roadmap Items

| Item | Toward Primary Integration | Toward Full Feature | Status |
| --- | ---: | ---: | --- |
| Dirty `preg_replace_callback()` repair | **96%** `[###################-]` | **32%** `[######--------------]` | Ready for primary review at hash `52973e3185c8874b67c38e245e6b0c6497c2117ac826fcb46d092f4ea655e8b5`. Executes string callbacks over a bounded slash-delimited regex subset. Still uncounted until committed and pushed. |
| Compact `impl-native-integration-batch` candidates | **73%** `[###############-----]` | **31%** `[######--------------]` | Lane work adds root-symbol exists, strict identity string/native-value comparison consumers, and generated-C `global` root-reference import. Promising, but conflict-heavy; extract one slice only. |
| Callable, scanner, stream, and request-state builtins | **66%** `[#############-------]` | **43%** `[#########-----------]` | Lane-local callable recovery values, stream/filesystem paths, shutdown callbacks, echo/print array conversion, class aliases, scanner/reference-output, and request-state candidates are active. Broad callable/userland/method dispatch remains incomplete. |
| Closure and call-frame correctness | **76%** `[###############-----]` | **51%** `[##########----------]` | Lane-local spread/named/by-reference capture work plus structured function/method signature metadata are strong candidates. Exact fatal timing, cleanup, request/global frames, and broad callability remain open. |
| Symbol-table storage and request/global reconciliation | **40%** `[########------------]` | **32%** `[######--------------]` | Root-symbol exists/import-reference, activation diagnostic handles, dynamic globals append shape, POST body SAPI population, and request cleanup candidates are active. Generalized PHP variable storage, includes, request/global aliases, and `$GLOBALS` parity are not counted. |
| Direct calls, callbacks, and frame extraction | **60%** `[############--------]` | **45%** `[#########-----------]` | Shared call argument/reference/result-consumer and constructor allocation boundaries are stronger. Broad PHP callable/function/method body execution remains unintegrated. |
| Object/ArrayAccess/method dispatch boundaries | **64%** `[#############-------]` | **32%** `[######--------------]` | Shared object-offset dispatch planning, backend-neutral ArrayAccess receiver blockers, contextual `self` properties, and object-valued `instanceof` carriers improved. Runtime `offsetGet`, `offsetExists`, `offsetSet`, and `offsetUnset` method-frame execution is still not integrated. |
| Object-offset `ArrayAccess` error-control classifier | **88%** `[##################--]` | **10%** `[##------------------]` | Focused one-file compiler classifier candidate. Improves unsupported object-offset diagnostic routing; it is not runtime `ArrayAccess`. Recheck hash/apply state after the preg decision. |
| Object-property reference-slot mutation | **78%** `[################----]` | **35%** `[#######-------------]` | Strong executable object mutation candidate at hash `c60ed1c30dc1d979da1fc44641ae4378a3629e45c42da0d9621faf10519e56e8`. Needs fresh current-primary apply proof and gates before integration. |
| Reference/foreach owner-cell ABI candidates | **53%** `[###########---------]` | **35%** `[#######-------------]` | Lane-local owner-cell component, transfer, mutation-effect sink, source-report, foreach cursor/element, borrow/apply, and cleanup ABIs are richer. Production foreach/provider lowering and alias/COW execution remain blocked. |
| Conditional/control-flow value consumers | **56%** `[###########---------]` | **44%** `[#########-----------]` | Lane-local request ternary, switch/loop/goto/throw target-state, value-slot predeclaration, and cleanup-state blockers have focused gates. Mutation-capable state, full cleanup/unwind, and general expression merges remain open. |
| Diagnostics, error handling, and source ordering | **54%** `[###########---------]` | **37%** `[#######-------------]` | Lane-local property/list/custom-handler/suppression bridges, activation diagnostic handles, request diagnostics, and source-report cleanup ABIs are improving. Exact Zend ordering, custom handler execution, suppression, and cleanup through real control flow remain open. |
| Broad lane extraction backlog | **30%** `[######--------------]` | **31%** `[######--------------]` | Many lanes report generalized progress, but several carry huge conflict-heavy worktrees. Treat them as mines for compact slices, not as patches to import. |

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
- [ ] Lane-local root-symbol exists, strict identity comparison consumer, and generated-C `global` root-reference import candidates.
- [ ] Lane-local callable scanner/reference-output, formatted stream-output, stream-context/resource/filesystem, shutdown, class alias, request-state, bitwise, echo/print array conversion, and dynamic byte-string/string-comparison candidates.
- [ ] Lane-local spread/named argument binding cleanup, closure by-value/by-reference capture behavior, activation signature metadata, and class-context lookup/invoke ABI exposure.
- [ ] Lane-local request/global reconciliation freshness, activation diagnostics, POST body SAPI population, nested diagnostics, and root-symbol metadata.
- [ ] Lane-local object/ArrayAccess dispatch-plan, backend-neutral receiver blocker, `self` typed-property, and object-valued `instanceof` candidates; promising but still not full method-frame or `ArrayAccess` execution.
- [ ] Lane-local owner-cell by-reference foreach, component-policy, mutation-effect cleanup, source-report, integer-argument diagnostic-slot, and reference-assignment offset-dispatch ABIs; production source-provider lowering remains blocked.
- [ ] Lane-local `object-arrayaccess-error-control-retry` diagnostic classifier candidate; recheck hash/apply state after the preg decision.
- [ ] Lane-local `object-property-reference-slots` mutation/reference-slot candidate; refresh current-primary apply proof and gates before integration.

Not done:

- [ ] Full callable lookup and invocation beyond selected strings, closures, arrays, and public `__invoke` objects, including magic/visibility/rebinding rules.
- [ ] Runtime `ArrayAccess` method dispatch for `offsetGet`, `offsetExists`, `offsetSet`, and `offsetUnset`.
- [ ] Full references/COW identity and arbitrary alias roots.
- [ ] Request and `$GLOBALS` parity, includes, variable variables, and dynamic symbol behavior.
- [ ] Full PCRE behavior beyond bounded literal/prefix/suffix and selected exact regex families.
- [ ] General object model: non-public methods, overrides, interfaces/traits execution, magic methods, dynamic/static/typed properties, destructors.
- [ ] Complete cleanup/unwind/finally/destructor/output-buffer shutdown behavior.
- [ ] Exact/source-ordered diagnostics, custom handler execution, warning/error continuation, and suppression parity.
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
- [x] Live `HEAD` and `origin/master` are synced at `34922783`.

Pushed but uncounted:

- [ ] `2967110c codegen: expose symbol table abi probe` exposes helper declarations/probe calls but does not execute generalized PHP symbol storage.
- [ ] Dashboard/progress commits through `34922783` update observability only.

Dirty primary but uncounted:

- [ ] `preg_replace_callback()` WIP is ready for primary review, but remains dirty and uncounted.
- [ ] Current verified dirty diff hash is `52973e3185c8874b67c38e245e6b0c6497c2117ac826fcb46d092f4ea655e8b5`.
- [ ] Dirty files remain exactly `compiler/src/interpreter.rs` and `compiler/tests/preg_replace_callback_builtin.rs`.
- [ ] Packet gate, regression audit, shape audit, `cargo check`, scoped rustfmt, scoped diff checks, and milestone841 PHP comparison support review at that hash.

Lane-local but uncounted:

- [ ] `impl-native-integration-batch` has root-symbol exists, strict identity comparison, and global-root-reference consumer candidates, but its accumulated worktree should not be imported wholesale.
- [ ] `impl-binary-string-runtime` has echo/print array conversion, comparison, concat, callable warning recovery, shutdown callback, filesystem/stream, object-stringable, request-state, bitwise, and diagnostic-recovery candidates.
- [ ] `impl-native-call-semantics` has fresh 08:35 CEST status around argument-ordering, spread/named binding, by-reference capture, and class-context direct lookup/invoke ABI candidates.
- [ ] `impl-native-control-flow-seed` has fresh 08:34 CEST status around nested value-slot and cleanup-state blockers.
- [ ] `impl-function-frame-seed` has structured function/method signature metadata and call-frame result cleanup candidates.
- [ ] `impl-global-symbols` has activation diagnostic, dynamic globals append, POST body SAPI, and request/symbol metadata candidates.
- [ ] Object/ArrayAccess/method-dispatch policy boundaries have fresh evidence, but remain lane-local and not full method-frame or `ArrayAccess` execution.
- [ ] Request/root metadata, linked diagnostics, receiver-method lookup preflight, switch/goto/loop/throw cleanup-state work, JSON/resource/INI/object-cast/class-property surfaces, and direct-variable probe consolidation are useful lane evidence but not primary-integrated.

## Review Notes

Resource pressure is serviceable but guarded. Live `/dev/shm` is `40G` total,
`24G` used, `17G` available, `58%` used; `du -sh /dev/shm` reports `24G`.
The `/home` filesystem has `459G` total, `206G` used, `235G` available,
`47%` used by `df`. A bounded `du -sh /home` attempt timed out after 20
seconds, so no fresh complete `/home` `du` size is available from this run.
Swap remains high at `23Gi` used of `29Gi`.

The supervisor dashboard is stale relative to live worker statuses: dashboard
last update remains `2026-05-25 01:56 CEST`, while fresh worker statuses reach
`2026-05-25 08:35 CEST`. Use current git state, worker artifacts, preg packet
evidence, and evaluator reports for steering until the dashboard is refreshed.

Advisory steering read: resolve the dirty preg decision first. Then integrate
only one narrow current-primary candidate at a time, with live hash/apply
verification and focused disk-backed gates. Treat fresh lane reports as
candidate supply, not counted product capability. Keep broad gates disk-backed
while swap remains high.
