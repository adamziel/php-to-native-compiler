# PHP Native Compiler Progress

Updated: 2026-05-25 09:04 CEST
Evaluation marker: `20260525T070422Z`

Accounting rule: only generalized, tested, committed, and pushed primary work
counts as product capability. Dirty primary WIP, probe-only commits,
dashboard-only commits, lane-local candidates, historical worktrees,
blocker-only classifiers, and status-file claims are excluded until selected,
gated, committed, pushed, and reflected here as semantic product progress.

Current live pushed primary head observed by evaluator:
`84450ef6 docs: update progress dashboard`

Latest counted semantic/test baseline:
`b217e2b4 codegen: block destructor-observable native allocation`

Latest pushed but uncounted code work:
`2967110c codegen: expose symbol table abi probe`

## Executive Read

Overall estimated progress: **85%** `[#################---]`

Executable PHP semantics: **85%** `[#################---]`

Primary semantic capability did not move in this review. Primary is synced
with `origin/master` at `84450ef6`; the newest pushed primary work is another
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

Lane-local work is active but uncounted. Fresh evidence includes root-symbol
string/cast/comparison consumers, source-call reference-return argument
carriers, call-frame result cleanup contracts, control-flow transfer-state
materialization, object-property ArrayAccess dispatch blockers, nested trait
metadata, contextual class-name resolution, owner-cell foreach/source-report
ABIs, scanner/string/resource/shutdown surfaces, and diagnostic sequencing.
These are candidate supplies, not product capability, until one compact slice
is extracted, rechecked on current primary, gated, committed, and pushed.

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
| Runtime and ABI foundations | **97%** | `[###################-]` | Strong shared value, array, string, diagnostic, reference, symbol, call-frame, object, method, callable-array, descriptor-closure, comparison, conversion, owner-cell, and request-state surfaces exist for selected paths. Many ABI expansions remain lane-local. |
| Compiler/backend consumers | **97%** | `[###################-]` | Generated C consumes many shared ABIs; LLVM and C assembly consume selected ABI families. Fresh root-symbol, call-frame, object/property, and control-flow consumers are lane-local until primary integration. |
| Executable PHP semantics | **85%** | `[#################---]` | Focused linked/runtime programs cover closure/callable/object islands. Dirty preg work and lane-local callback/ArrayAccess/symbol/control-flow/string progress are not counted yet. |
| Arrays, lvalues, references, COW | **67%** | `[#############-------]` | Useful selected lvalue/reference paths feed closures and lane-local mutation/foreach work. Full COW, arbitrary roots, foreach production lowering, property references, and alias composition remain open. |
| Symbols, globals, request state | **72%** | `[##############------]` | Function globals and `$GLOBALS` self-import blockers improved. Lane work has root string/cast/comparison/result consumers and activation/request metadata. Generalized symbol storage, includes, variable variables, exact unset/global alias behavior, and reconciliation remain incomplete. |
| Calls, functions, frames | **82%** | `[################----]` | Bounded functions, descriptor closures, callable arrays, callable objects, public method frames, and constructor frames work in selected generated-C cases. Call-frame cleanup and by-reference call carriers remain lane-local. |
| Objects, properties, methods | **45%** | `[#########-----------]` | Useful public declared-object subset exists and supported public `__invoke` frames are callable. Lane work adds trait/contextual-class/property dispatch blockers. Non-public/contextual visibility, overrides, interfaces/traits execution, magic methods, dynamic/static/typed properties, destructors, references/COW, and backend parity remain open. |
| Control flow, cleanup, diagnostics | **49%** | `[##########----------]` | Selected branches, loops, transfers, finalizers, output buffers, diagnostics, and destructor blockers exist. Transfer-state, diagnostic sequencing, and cleanup metadata lanes are active but not integrated. |
| Broad integrated verification | **84%** | `[#################---]` | Focused gates are strong for recent counted slices. Broad dirty-checkout gates remain constrained by preg WIP, stale dashboard state, lane extraction cost, high swap, and backend parity gaps. |

## Active Roadmap Items

| Item | Toward Primary Integration | Toward Full Feature | Status |
| --- | ---: | ---: | --- |
| Dirty `preg_replace_callback()` repair | **96%** `[###################-]` | **32%** `[######--------------]` | Ready for primary review at hash `52973e3185c8874b67c38e245e6b0c6497c2117ac826fcb46d092f4ea655e8b5`. Executes string callbacks over a bounded slash-delimited regex subset. Still uncounted until committed and pushed. |
| Root symbols, globals, request state | **74%** `[###############-----]` | **34%** `[#######-------------]` | Lane-local root-symbol exists/string/scalar/array/comparison consumers are promising. Dynamic mixed roots, `$GLOBALS` self-cells, request alias cells, function imports, includes, references/COW, and exact diagnostics remain open. |
| Callable, call-frame, and argument cleanup | **78%** `[################----]` | **52%** `[##########----------]` | Lane-local source-call reference-return carriers and call-frame result cleanup contracts are useful. Full userland/method invocation, by-reference returns, named/unpacked args, recursion, and exact cleanup remain incomplete. |
| Object/property/ArrayAccess boundaries | **66%** `[#############-------]` | **33%** `[#######-------------]` | Lane-local object-property ArrayAccess dispatch classifiers, nested trait metadata, and contextual class-name boundaries improved. Real `offsetGet/Exists/Set/Unset`, magic/visibility, typed/dynamic/static properties, and method frames are not integrated. |
| Control-flow state and cleanup | **58%** `[############--------]` | **45%** `[#########-----------]` | Lane-local loop/switch/goto transfer-state materialization is substantial. Mutation-capable loops, full phis, references/objects/resources, finally/throw/return/destructor ordering, and exact diagnostics remain open. |
| References, owner cells, foreach | **55%** `[###########---------]` | **36%** `[#######-------------]` | Owner-cell source-report cleanup and visibility-aware by-reference foreach cursor ABIs are active. Production foreach/source-provider lowering and full alias/COW execution remain blocked. |
| String, stream, scanner, and conversion surfaces | **67%** `[#############-------]` | **44%** `[#########-----------]` | Lane-local scanner output, formatted strings, byte strings, resource/string offsets, shutdown callbacks, and conversion-result boundaries are active. Broad PHP string/resource/PCRE/callback parity remains incomplete. |
| Diagnostics and error handling | **56%** `[###########---------]` | **38%** `[########------------]` | Lane-local diagnostic sequencing, prepared-argument consumers, clear sinks, and branch-state merges improved. Real custom handler execution, exact suppression/order, and cleanup through all control flow remain open. |
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
- [ ] Lane-local root-symbol string/cast/comparison/result consumers across LLVM/generated C.
- [ ] Lane-local source-call reference-return carriers and call-frame result cleanup contracts.
- [ ] Lane-local loop/switch/goto transfer-state and cleanup metadata.
- [ ] Lane-local object/property, trait, contextual class-name, and ArrayAccess dispatch boundaries.
- [ ] Lane-local owner-cell by-reference foreach/source-report cleanup ABIs.
- [ ] Lane-local scanner/string/stream/resource/shutdown/formatted-output and conversion candidates.
- [ ] Lane-local diagnostic sequencing, clear-sink, prepared-argument, and branch-state merge boundaries.

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

- `b217e2b4`: generated-C declared-object allocation now blocks destructor-observable native allocation before emitting allocation branches. Destructor declarations are recorded as declared-class metadata, inherited through class hierarchy lookup, and checked against runtime string-valued dynamic class-name facts. This is cleanup/unwind safety only.
- `7679dc0e`: generated-C runtime dynamic calls can invoke supported declared callable objects through public `__invoke` method frames.
- `53c8a283`: supported non-static regular closures and arrows created inside active object method/constructor frames bind `$this` through the shared descriptor capture/callback path.
- `8f5d8fb3` and `79496862`: supported static arrow and static anonymous descriptor closures reuse the shared descriptor closure stack.
- `7a43e1ac`: runtime dynamic calls can invoke syntax-valid callable arrays that resolve to supported generated public static/object method frames.

## Current Work Snapshot

Primary-integrated and counted:

- [x] Counted semantic baseline remains `b217e2b4`.
- [x] Overall and executable-semantics estimates remain 85%.
- [x] Live `HEAD` and `origin/master` are synced at `84450ef6`.

Dirty primary but uncounted:

- [ ] `preg_replace_callback()` WIP remains ready for primary review, but remains dirty and uncounted.
- [ ] Current verified dirty diff hash is `52973e3185c8874b67c38e245e6b0c6497c2117ac826fcb46d092f4ea655e8b5`.
- [ ] Dirty files remain exactly `compiler/src/interpreter.rs` and `compiler/tests/preg_replace_callback_builtin.rs`.

Lane-local but uncounted:

- [ ] Fresh statuses reach 2026-05-25 09:11 CEST, while the supervisor dashboard is stale at 2026-05-25 01:56 CEST.
- [ ] `impl-native-integration-batch`, `impl-native-diagnostics`, `impl-native-call-semantics`, `impl-function-frame-seed`, `impl-global-symbols`, `impl-native-control-flow-seed`, `impl-native-object-property-runtime`, `impl-native-object-seed`, `impl-native-reference-cell-runtime`, `impl-binary-string-runtime`, `impl-native-type-conversion`, and `impl-array-lowering` all have candidate supply. None is counted until extracted and integrated.

## Resource And Steering Notes

Resource pressure is serviceable but guarded. Live `/dev/shm` is `40G` total,
`24G` used, `17G` available, `58%` used; `du -sh /dev/shm` reports `24G`.
The `/home` filesystem has `459G` total, `213G` used, `228G` available,
`49%` used by `df`; `du -sh /home` reported `124G` but exited nonzero, likely
because some entries were unreadable. Memory has `43Gi` total, `36Gi`
available through cache, and only `601Mi` free; swap remains high at `23Gi`
used of `29Gi`.

Advisory steering read: resolve the dirty preg decision first. Then integrate
only one narrow current-primary candidate at a time, with live hash/apply
verification and focused disk-backed gates. Treat fresh lane reports as
candidate supply, not counted product capability. Keep broad gates disk-backed
while swap remains high.
