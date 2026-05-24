# PHP Native Compiler Progress

Updated: 2026-05-24 18:04 CEST
Evaluation marker: `20260524T1604Z`

Latest counted primary semantic/test baseline:
`53c8a283 codegen: bind non-static closure this captures`

Latest primary head before this progress update:
`53c8a283 codegen: bind non-static closure this captures`

Latest observed `origin/master` after push:
`afdca215 docs: update progress after closure this binding`

Accounting rule: only generalized, tested, committed, and pushed primary work counts. Current primary is synced to `origin/master`; dirty primary WIP, lane-local candidates, parked diffs, blocker-only classifiers, and status-file claims are not product capability until selected, gated, committed, and pushed through primary.

Accounting history: the earlier **88%** headline is retired because it counted lane-local candidates, scaffolding, and ABI surface area too generously before they became executable product behavior. The later **50%** figure was a conservative strict-rubric rebaseline, not a code rollback. The current source-of-truth estimate is **84%**, not 50%, under the stricter primary-integrated/executable-semantics rule.

## Executive Read

Overall estimated progress: **84%** `[#################---]`

Executable PHP semantics: **84%** `[#################---]`

Primary has real integrated progress in selected generated-C execution islands. The current counted baseline includes descriptor-backed closure invocation, non-static closure `$this` binding for supported descriptor closures created in object frames, supported static anonymous descriptor closures, supported static arrow descriptor closures, direct by-value closure captures, supported by-reference closure captures through root symbol/reference handles and promoted function-frame locals, non-static arrow closures with implicit by-value captures, typed/default/variadic by-value descriptor closure parameters, untyped by-reference descriptor closure parameters, callable-array invocation for supported public static and object method frames, same-family aggregate equality, runtime string-valued declared-class `new` for constructorless and supported public-constructor classes, and a bounded public declared-object family.

Generated-C descriptor closures now carry a shared native closure-argument carrier that can hold either an owned value or a reference handle. Runtime descriptor metadata marks untyped by-reference parameters and by-value variadic parameters; descriptor arity distinguishes required, total, and variadic-unbounded call ranges. Closure construction can store by-reference captures by preserving the same runtime reference cell, and can now promote supported in-scope function-frame locals and by-value parameters into runtime reference cells before capture. Closure-frame binding rehydrates those captures as reference handles during callback execution. Closure-frame binding can also fill missing supported by-value parameters from parsed defaults, pack surplus by-value closure arguments into native arrays, and route supported scalar/array/mixed parameter types through the shared call-frame type ABI. Dynamic closure invocation materializes supported direct-variable and nested array lvalue arguments through the shared symbol/reference ABI. Supported non-static descriptor closures created in active object frames now append the frame `$this` object as an ordinary descriptor capture, so regular closures and arrows can consume `$this` through the same capture binding, public property ABI, dynamic invocation, diagnostics, and cleanup paths. Supported static anonymous closures and supported static arrow closures still reuse descriptor paths without receiving implicit `$this`.

Runtime dynamic calls now also decompose syntax-valid callable arrays through a shared callable-array parts ABI. Generated-C dispatch routes class-string public static method arrays and object public instance method arrays through the existing runtime name matching, object/class relation checks, method-frame argument planner, by-reference argument materialization, generated method frames, and cleanup/result ownership. The proof includes direct callable arrays, user-function relay, by-reference argument relay, and inherited public static/instance method lookup.

This is still not general PHP. The remaining cliffs are large: full callable lookup/invocation, full closure rebinding APIs, unsupported closure cases, request/`$GLOBALS` and missing-local by-reference capture roots, by-reference variadic/default/typed closure parameter combinations, unsupported typed/default closure cases, broader by-reference returns, callable objects and unsupported callable-array forms, arbitrary class-name expressions for `new`, non-public methods, overrides, interfaces/traits, contextual `self`/`parent`/`static`, magic methods, dynamic/static properties, references/COW identity, request/global alias parity, includes, variable variables, exact/source-ordered diagnostics, cleanup/unwind/finally/destructors/output-buffer shutdown, and backend parity.

## Primary-Integrated Capability

- [x] Runtime/value foundations: shared native value, array, string, comparison, truthiness, diagnostic, request, reference, symbol, call-frame, output-buffer, object, object-property, method-diagnostic, callable-array parts, and descriptor-closure ABI surfaces exist for selected paths, including capture-aware, by-reference-capture, by-reference-argument, and variadic descriptor closures.
- [x] Generated-C execution: direct variables, arrays/lvalues, selected dynamic calls, function frames, callable-array public method-frame dispatch, descriptor-backed by-value, supported non-static `$this`-bound closure frames, supported static anonymous closure frames, and supported static arrow closure frames with direct by-value captures, supported by-reference captures, non-static arrow implicit by-value captures, typed/default/variadic by-value closure parameters, and untyped by-reference closure parameters, function globals, `$GLOBALS` self-imports, selected references, assignment expressions, output buffers, finalizer transfer slices, runtime string-valued declared-class `new`, and the bounded public declared-object family are executable.
- [x] Runtime comparison: arrays, native array handles, objects/closures, and resources now have same-family loose equality through one recursive comparison context.
- [x] LLVM and C assembly consumers: selected string, predicate, search, integer/string helper, primitive assignment/compound-assignment, output-buffer, value-operation arithmetic, and string-result/string-predicate helper paths consume shared ABIs.
- [ ] Not complete: full PHP callable/closure, object, reference/COW, cleanup/unwind, diagnostic, request/global, include, variable-variable, and backend-parity behavior.

## Grand Roadmap Position

| Workstream | Estimate | Bar | Current read |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | **97%** | `[###################-]` | Strong shared surfaces exist and are consumed by output buffers, capture-aware/by-reference-capture/by-reference-argument/variadic descriptor closures, callable-array parts, frame-local reference promotion, aggregate equality, and declared-object paths; newer surfaces still need executable consumers. |
| Compiler/backend consumers | **97%** | `[###################-]` | Generated-C is broad in selected areas and now routes supported callable arrays into method frames; LLVM and C assembly consume selected ABI families. Direct assembly, LLVM object lowering, and many nested/object consumers remain blocked. |
| Executable PHP semantics | **84%** | `[#################---]` | Many focused linked/runtime programs run, including supported non-static `$this`-bound descriptor closures, static anonymous and static arrow descriptor closures, typed/default/variadic by-value descriptor closure parameters, supported root/reference and frame-local by-reference captures, by-reference descriptor closure parameters, arrow implicit capture execution, and callable-array public method-frame invocation, but execution remains selected islands rather than general PHP. |
| Arrays, lvalues, references, COW | **67%** | `[#############-------]` | Good selected lvalue/reference paths now feed descriptor closure parameters, supported capture aliases, and frame-local reference promotion; full COW, arbitrary roots, foreach, object joins, and alias composition remain open. |
| Symbols, globals, request state | **72%** | `[##############------]` | Function globals and `$GLOBALS` self-imports improved; request superglobal imports, includes, variable variables, and exact unset behavior remain incomplete. |
| Calls, functions, frames | **81%** | `[################----]` | Bounded functions, descriptor-backed by-value/non-static `$this`-bound/static anonymous/static arrow closures, supported by-reference closure captures including promoted frame locals, non-static arrow implicit captures, typed/default/variadic by-value closure parameters, untyped by-reference closure parameters, callable-array public method frames, and public object/method/constructor call paths work in selected generated-C cases. |
| Objects, properties, methods | **44%** | `[#########-----------]` | Useful public declared-object subset exists and callable arrays can invoke supported public static/object methods; non-public methods, overrides, interfaces/traits, magic methods, dynamic/static properties, visibility contexts, references/COW, and backend parity remain open. |
| Control flow, cleanup, diagnostics | **49%** | `[##########----------]` | Selected branches, loops, transfers, finalizers, output buffers, and diagnostics exist; broad unwind, handlers, destructors, shutdown flushing, and exact ordering remain open. |
| Broad integrated verification | **83%** | `[#################---]` | Focused gates are strong for recent slices, including source/link proof for non-static closure `$this` binding, static arrow descriptor closures, static anonymous descriptor closures, callable-array method frames, frame-local by-reference captures, root/reference by-reference captures, typed/default/variadic closure parameters, arrow implicit captures, and descriptor closure regressions; broad `native_link`/call-boundary suites still expose unrelated baseline failures, and backend parity/full end-to-end PHP proof lag. |

## Done / In Progress / Not Done

- [x] Done in primary: descriptor-backed by-value closure invocation, supported non-static closure `$this` binding, supported static anonymous descriptor closures, supported static arrow descriptor closures, direct by-value closure captures, supported by-reference closure captures through root symbol/reference handles and promoted function-frame locals, non-static arrow implicit by-value captures, typed/default/variadic by-value descriptor closure parameters, untyped by-reference descriptor closure parameters, callable-array invocation for supported public static/object method frames, runtime same-family aggregate equality, runtime string-valued declared-class `new` for constructorless and supported public-constructor classes, public declared properties/methods/statics/constructors, inherited public slots, named `instanceof`, output buffers, selected function globals, assignment expressions, compound assignments, selected references, and selected LLVM/C assembly ABI consumers.
- [ ] In progress but uncounted: lane-local work around by-reference returns, method-table metadata, ArrayAccess/object offsets, root-symbol selection, callback/reference-slot cleanup, and blocker classification.
- [ ] Not done: general object model, arbitrary dynamic class-name expressions, contextual class names, non-public methods, overrides, interfaces/traits, magic methods, unsupported closure cases, by-reference variadic/default/typed closure parameter combinations, unsupported typed/default closure cases, callable objects and unsupported callable-array forms, complete references/COW, request/`$GLOBALS` and missing-local by-reference capture roots, broad by-reference returns, complete mutation/unset, full diagnostics, full cleanup/unwind, includes, variable variables, request/global parity, and backend parity.

## Recent Primary-Integrated Work

- `53c8a283`: generated-C descriptor closures support `$this` binding for
  supported non-static regular closures and arrows created inside active object
  method/constructor frames. Closure construction now uses one descriptor
  capture selection boundary for regular closures and arrows, appending the
  active frame `$this` as an ordinary by-value capture when the closure is
  non-static and `$this` exists. The bound object is rehydrated through the
  existing closure-frame capture binder and consumed through the shared public
  property ABI, dynamic invocation, diagnostics, and cleanup paths. Source and
  linked proof cover direct invocation, user-function relay, public
  static-method relay, method-frame callback creation, regular closure
  mutation through `$this->value`, arrow reads through `$this`, static
  anonymous closure regression, static arrow regression, callable-array
  regression, descriptor closure regression, and call-boundary blocker proof
  while `Closure::bind`/`bindTo`, invalid `$this` diagnostics, references/COW,
  by-reference returns, unsupported capture roots, cleanup/unwind, and backend
  parity remain blocked.
- `8f5d8fb3`: the parser now admits `static fn (...) => ...` and marks the
  existing closure AST as both arrow and static instead of rejecting the syntax
  before lowering. Generated-C then reuses the accepted static descriptor
  closure path plus the existing arrow implicit-capture collector, descriptor
  capture ABI, callback-frame ABI, by-reference argument carriers,
  typed/default/variadic parameter handling, dynamic invocation, diagnostics,
  and cleanup ownership. Source and linked proof cover direct static arrows,
  user-function relay, public static-method relay, static arrows returned from
  instance methods, nested static arrows, array-key captures, untyped
  by-reference parameters, typed/default/variadic by-value parameters, and the
  shared `$this` variable-read blocker for static arrows while non-static
  closure `$this` auto-binding, by-reference returns, unsupported capture
  roots, exact callable diagnostics, references/COW, cleanup/unwind, and
  backend parity remain blocked.
- `79496862`: generated-C descriptor closures support supported static
  anonymous closures. The compiler no longer blocks every `static function`
  before semantic validation; instead, static closures reuse the existing
  descriptor creation/capture ABI, closure-frame callback ABI, dynamic
  invocation path, diagnostics, and cleanup ownership when their parameters,
  captures, return mode, body operations, and type metadata are already in the
  supported descriptor subset. Source and linked proof cover direct
  invocation, user-function relay, public static-method relay, returned
  instance-method closure creation, explicit by-value captures, supported
  by-reference captures, untyped by-reference parameters,
  typed/default/variadic by-value parameters, and preserved `$this` blocking
  through the shared variable-read boundary while static arrows, non-static
  closure `$this` auto-binding, by-reference returns, unsupported capture roots, exact
  callable diagnostics, references/COW, cleanup/unwind, and backend parity
  remain blocked.
- `7a43e1ac`: generated-C runtime dynamic calls support callable-array
  invocation for syntax-valid arrays that resolve to supported generated
  public method frames. Runtime exposes a shared callable-array parts ABI, and
  generated-C dispatch reuses runtime callable name matching, object/class
  relation checks, inherited declared-class lookup, method-frame argument
  planning, by-reference argument materialization, and cleanup/result
  ownership for class-string static method arrays and object instance method
  arrays. Source and linked proof cover direct invocation, by-value
  user-function relay, by-reference relay, and inherited public static/object
  method lookup while callable objects, non-public/context-sensitive
  visibility, magic `__invoke`, unsupported callable-array target/method
  forms, exact callable diagnostics, references/COW, cleanup/unwind, and
  backend parity remain blocked.
- `c9172ca6`: generated-C descriptor closures support supported
  function-frame by-reference `use (&$x)` captures for ordinary in-scope
  locals and by-value parameters. The runtime now provides a shared
  value-to-reference promotion ABI, the compiler promotes frame locals before
  descriptor capture materialization, and the existing descriptor
  reference-carrier/capture-binding path preserves alias identity across
  returned closures, repeated calls, nested regular closures, and public
  static-method relay. Missing frame locals, request/`$GLOBALS` capture roots,
  by-reference variadics/defaults/typed combinations, by-reference returns,
  static closures, callable arrays/objects, named/unpacked args,
  references/COW, exact diagnostics, cleanup/unwind, and backend parity remain
  blocked.
- `1aaaac30`: generated-C descriptor closures support supported by-reference `use (&$x)` captures through the shared native closure-argument carrier. Closure construction can now materialize supported root symbol/reference capture cells, runtime closure values preserve the same reference cell instead of cloning a by-value snapshot, and closure-frame callbacks bind captured aliases back as native reference handles. Source and linked proof cover direct invocation, user-function relay, public static-method relay, nested closures, by-reference parameter composition, and function-frame by-reference parameter factory relay while then-unsupported frame-local/request/`$GLOBALS` capture roots, by-reference variadics, by-reference returns, static closures, callable arrays/objects, named/unpacked args, request/global separation, references/COW, exact diagnostics, cleanup/unwind, and backend parity remained blocked at that slice.
- `103c0a4e`: generated-C descriptor closures support supported by-value variadic parameters. Descriptor parameter metadata now marks variadic slots, runtime descriptor invocation accepts surplus arguments only for variadic descriptors, closure callbacks pack surplus supplied arguments through the shared native array/value ABI, and typed variadic arguments reuse the existing call-frame type coercion ABI. Source and linked proof cover no-argument and surplus-argument closures, typed variadics, default-before-variadic binding, regular closures, arrows, by-value captures, user-function relay, public static-method relay, and composition with untyped by-reference descriptor parameters while by-reference variadics, by-reference captures/returns, static closures, callable arrays/objects, named/unpacked args, request/global separation, references/COW, exact diagnostics, cleanup/unwind, and backend parity remain blocked.
- `ff1d8ee3`: generated-C descriptor closures support supported typed/default by-value parameters. Descriptor metadata now publishes required arg count separately from total param count, closure callbacks bind missing by-value parameters from parsed default expressions, typed closure parameters use the shared call-frame type coercion ABI, and captures start after the actual runtime call-arg prefix. Source and linked proof cover direct closure calls, user-function relay, public static method relay, arrows with defaults, captured closures with defaults, and descriptor/by-reference/arrow/user-function regressions while then-unsupported variadic closure parameters, by-reference captures, by-reference returns, static arrows, callable arrays/objects, named/unpacked args, request/global separation, references/COW, exact diagnostics, cleanup/unwind, and backend parity remained blocked at that slice.
- `959dc8b6`: arrow capture discovery was broadened from the initial return-expression visitor into an AST-driven capture collector for statement, expression, lvalue, reference-source, unset-target, interpolated access, dynamic class-name, nested arrow, and nested regular-closure lexical-use surfaces. Focused proof now includes array-key implicit captures, nested regular closures with explicit `use (...)`, no invented capture for regular closures without `use`, composition with untyped by-reference arrow parameters, unsupported arrow-default blockers, descriptor-closure regressions, and call-boundary blockers.
- `b8029289`: generated-C descriptor-ready non-static arrow closures synthesize implicit by-value captures from arrow return-expression variable use, excluding parameters, `$this`, `$GLOBALS`, request superglobals, and unavailable native-frame symbols. The captures reuse the existing descriptor capture ABI and closure-frame binding path, and nested arrow closures propagate implicit captures through descriptor frames. Source and linked proof cover direct arrow calls, user-function relay, public static method relay, returned closures, nested arrows, and by-value isolation after outer-variable mutation while static arrows, by-reference captures, then-unsupported typed/default/variadic closure parameters, exact diagnostics, request/global frame parity, and backend parity remained blocked at that slice.
- `deabcd6d`: generated-C descriptor closures support untyped by-reference parameters through shared descriptor parameter metadata, `phpc_NativeClosureArgument` value/reference carriers, runtime reference-argument diagnostics, and the existing dynamic-call path. Linked proof covers direct closure calls, user-function relay, runtime dynamic relay, direct variable lvalues, and nested array lvalues while by-reference captures, then-unsupported typed/default/variadic closure parameters, static closure behavior, root/global frame handoff, callable arrays/objects, and backend parity remained blocked at that slice.
- `2f306cea`: generated-C descriptor-ready closures support direct by-value `use (...)` captures. Capture values are copied into closure descriptor payloads, rebound as closure-frame locals during invocation, and proven through stored closures, immediate closures, repeated calls, function-frame relay, and outer-variable mutation isolation.
- `6dda705d`: native runtime value comparison executes same-family loose equality for arrays, native array handles, declared objects/closures, and resources through a shared comparison context.
- `d5e0e60f`: generated-C no-capture by-value fixed-parameter closures lower to descriptor-backed closure values and invoke through the shared dynamic-call path, including ordinary by-value frame transfer.
- `6360acdf`: generated-C `new $class(...)` supports runtime string-valued direct-variable class names for generated declared classes requiring supported public constructor dispatch.
- `e2d20f3`: generated-C `new $class(...)` supports runtime string-valued direct-variable class names for generated constructorless declared classes.

## Lane-Local Candidate Work

- The arrow implicit by-value capture candidate has landed through primary as `b8029289` and was broadened in primary as `959dc8b6`; treat `/home/claude/phpc-candidate-closure-arrow-captures` as historical unless it is explicitly rebased/repurposed from current primary.
- The typed/default by-value closure-parameter candidate has landed through primary as `ff1d8ee3`; treat `/home/claude/phpc-candidate-closure-param-semantics` as historical unless it is explicitly rebased/repurposed from current primary.
- The by-reference descriptor closure parameter candidate has landed through primary as `deabcd6d`; treat the previous candidate lane as historical unless it is explicitly rebased/repurposed from current primary.
- The callable-array public method-frame candidate has landed through primary as `7a43e1ac`; treat `/home/claude/phpc-candidate-closure-next-callable-semantics` and `/home/claude/phpc-candidate-closure-callable-frames-generalized-20260524` as historical unless explicitly rebased/repurposed from current primary.
- The static anonymous descriptor-closure candidate has landed through primary as `79496862`; treat `/home/claude/phpc-candidate-static-descriptor-closures-current` as historical unless explicitly rebased/repurposed from current primary.
- Static arrow descriptor closures have landed through primary as `8f5d8fb3`; do not repeat parser admission, static no-`$this` descriptor routing, or the current static-arrow source/link proof.
- Non-static closure `$this` binding has landed through primary as `53c8a283`; treat `/home/claude/phpc-candidate-closure-this-binding-current-20260524` as historical unless explicitly rebased/repurposed for distinct rebinding, diagnostics, references/COW, or backend-parity work.
- `impl-native-call-semantics` has relevant lane-local material around by-reference frame-local returns and broader callable behavior, but it remains broad relative to current primary.
- `impl-native-object-property-runtime` has ArrayAccess object-offset mutation dispatch evidence, but real method dispatch, references/COW, nested offsets, visibility/magic policy, and exact diagnostics remain blocked.
- `impl-function-frame-seed` and `impl-native-integration-batch` continue producing useful ABI/metadata surfaces, but many are not executable product semantics until primary consumers land.
- Several lanes are producing blocker-classifier and reference-slot cleanup work. Prefer candidates that remove a shared execution blocker or provide cross-feature linked proof. By-reference closure captures have now landed for supported root/reference paths and promoted function-frame locals; do not repeat that surface without a distinct request/`$GLOBALS`/missing-root, return, COW, or cleanup boundary.

## Current Review Notes

- Primary semantic work is committed at `53c8a283`; this progress update is the separate documentation wrapper before push.
- Focused gates for the latest non-static closure `$this` binding slice passed using disk-backed `/tmp/phpc-primary-closure-this`: `cargo check -q -p phpc -p php_runtime`, `cargo test -q -p phpc --test native_link non_static_closure_this`, `cargo test -q -p phpc --test native_link static_arrow`, `cargo test -q -p phpc --test native_link static_descriptor_closure`, `cargo test -q -p phpc --test native_link static_closure_this_binding`, `cargo test -q -p phpc --test native_link descriptor_closure`, `cargo test -q -p phpc --test native_link arrow_closure`, `cargo test -q -p phpc --test native_link callable_array`, `cargo test -q -p phpc --test native_function_call_boundary native_executable_c_source_routes_call_operation_blockers_across_call_families`, `cargo test -q -p php_runtime --lib -- --test-threads=1` with 284 passed, scoped `rustfmt --edition 2021 --check compiler/src/codegen.rs compiler/tests/native_link.rs compiler/tests/native_function_call_boundary.rs`, `git diff --check`, and `git diff --cached --check`.
- Broad full `native_link` and `native_function_call_boundary` suites were not rerun for this small slice; previous known unrelated baseline failures remain tracked as broad verification debt.
