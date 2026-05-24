# PHP Native Compiler Progress

Updated: 2026-05-24 17:26 CEST
Evaluation marker: `20260524T145851Z`

Latest counted primary semantic/test baseline:
`7a43e1ac codegen: invoke callable arrays through method frames`

Latest primary head before this progress update:
`7a43e1ac codegen: invoke callable arrays through method frames`

Latest observed `origin/master` during this review:
`8e0b9192 docs: update progress after local ref captures`

Accounting rule: only generalized, tested, committed, and pushed primary work counts. Current primary is synced to `origin/master`; dirty primary WIP, lane-local candidates, parked diffs, blocker-only classifiers, and status-file claims are not product capability until selected, gated, committed, and pushed through primary.

Accounting history: the earlier **88%** headline is retired because it counted lane-local candidates, scaffolding, and ABI surface area too generously before they became executable product behavior. The later **50%** figure was a conservative strict-rubric rebaseline, not a code rollback. The current source-of-truth estimate is **81%** under the stricter rule.

## Executive Read

Overall estimated progress: **81%** `[################----]`

Executable PHP semantics: **81%** `[################----]`

Primary has real integrated progress in selected generated-C execution islands. The current counted baseline includes descriptor-backed closure invocation, direct by-value closure captures, supported by-reference closure captures through root symbol/reference handles and promoted function-frame locals, non-static arrow closures with implicit by-value captures, typed/default/variadic by-value descriptor closure parameters, untyped by-reference descriptor closure parameters, callable-array invocation for supported public static and object method frames, same-family aggregate equality, runtime string-valued declared-class `new` for constructorless and supported public-constructor classes, and a bounded public declared-object family.

Generated-C descriptor closures now carry a shared native closure-argument carrier that can hold either an owned value or a reference handle. Runtime descriptor metadata marks untyped by-reference parameters and by-value variadic parameters; descriptor arity distinguishes required, total, and variadic-unbounded call ranges. Closure construction can store by-reference captures by preserving the same runtime reference cell, and can now promote supported in-scope function-frame locals and by-value parameters into runtime reference cells before capture. Closure-frame binding rehydrates those captures as reference handles during callback execution. Closure-frame binding can also fill missing supported by-value parameters from parsed defaults, pack surplus by-value closure arguments into native arrays, and route supported scalar/array/mixed parameter types through the shared call-frame type ABI. Dynamic closure invocation materializes supported direct-variable and nested array lvalue arguments through the shared symbol/reference ABI. Non-static arrow closures synthesize by-value captures from generalized AST variable use and feed those captures through the same descriptor capture ABI, including nested arrow propagation, nested regular closures with explicit `use (...)`, array-key captures, and composition with untyped by-reference arrow parameters.

Runtime dynamic calls now also decompose syntax-valid callable arrays through a shared callable-array parts ABI. Generated-C dispatch routes class-string public static method arrays and object public instance method arrays through the existing runtime name matching, object/class relation checks, method-frame argument planner, by-reference argument materialization, generated method frames, and cleanup/result ownership. The proof includes direct callable arrays, user-function relay, by-reference argument relay, and inherited public static/instance method lookup.

This is still not general PHP. The remaining cliffs are large: full callable lookup/invocation, static closures, request/`$GLOBALS` and missing-local by-reference capture roots, by-reference variadic/default/typed closure parameter combinations, unsupported typed/default closure cases, broader by-reference returns, callable objects and unsupported callable-array forms, arbitrary class-name expressions for `new`, non-public methods, overrides, interfaces/traits, contextual `self`/`parent`/`static`, magic methods, dynamic/static properties, references/COW identity, request/global alias parity, includes, variable variables, exact/source-ordered diagnostics, cleanup/unwind/finally/destructors/output-buffer shutdown, and backend parity.

## Primary-Integrated Capability

- [x] Runtime/value foundations: shared native value, array, string, comparison, truthiness, diagnostic, request, reference, symbol, call-frame, output-buffer, object, object-property, method-diagnostic, callable-array parts, and descriptor-closure ABI surfaces exist for selected paths, including capture-aware, by-reference-capture, by-reference-argument, and variadic descriptor closures.
- [x] Generated-C execution: direct variables, arrays/lvalues, selected dynamic calls, function frames, callable-array public method-frame dispatch, descriptor-backed by-value closure frames with direct by-value captures, supported by-reference captures, non-static arrow implicit by-value captures, typed/default/variadic by-value closure parameters, and untyped by-reference closure parameters, function globals, `$GLOBALS` self-imports, selected references, assignment expressions, output buffers, finalizer transfer slices, runtime string-valued declared-class `new`, and the bounded public declared-object family are executable.
- [x] Runtime comparison: arrays, native array handles, objects/closures, and resources now have same-family loose equality through one recursive comparison context.
- [x] LLVM and C assembly consumers: selected string, predicate, search, integer/string helper, primitive assignment/compound-assignment, output-buffer, value-operation arithmetic, and string-result/string-predicate helper paths consume shared ABIs.
- [ ] Not complete: full PHP callable/closure, object, reference/COW, cleanup/unwind, diagnostic, request/global, include, variable-variable, and backend-parity behavior.

## Grand Roadmap Position

| Workstream | Estimate | Bar | Current read |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | **97%** | `[###################-]` | Strong shared surfaces exist and are consumed by output buffers, capture-aware/by-reference-capture/by-reference-argument/variadic descriptor closures, callable-array parts, frame-local reference promotion, aggregate equality, and declared-object paths; newer surfaces still need executable consumers. |
| Compiler/backend consumers | **97%** | `[###################-]` | Generated-C is broad in selected areas and now routes supported callable arrays into method frames; LLVM and C assembly consume selected ABI families. Direct assembly, LLVM object lowering, and many nested/object consumers remain blocked. |
| Executable PHP semantics | **81%** | `[################----]` | Many focused linked/runtime programs run, including typed/default/variadic by-value descriptor closure parameters, supported root/reference and frame-local by-reference captures, by-reference descriptor closure parameters, arrow implicit capture execution, and callable-array public method-frame invocation, but execution remains selected islands rather than general PHP. |
| Arrays, lvalues, references, COW | **67%** | `[#############-------]` | Good selected lvalue/reference paths now feed descriptor closure parameters, supported capture aliases, and frame-local reference promotion; full COW, arbitrary roots, foreach, object joins, and alias composition remain open. |
| Symbols, globals, request state | **72%** | `[##############------]` | Function globals and `$GLOBALS` self-imports improved; request superglobal imports, includes, variable variables, and exact unset behavior remain incomplete. |
| Calls, functions, frames | **78%** | `[################----]` | Bounded functions, descriptor-backed by-value and supported by-reference closure captures including promoted frame locals, non-static arrow implicit captures, typed/default/variadic by-value closure parameters, untyped by-reference closure parameters, callable-array public method frames, and public object/method/constructor call paths work in selected generated-C cases. |
| Objects, properties, methods | **44%** | `[#########-----------]` | Useful public declared-object subset exists and callable arrays can invoke supported public static/object methods; non-public methods, overrides, interfaces/traits, magic methods, dynamic/static properties, visibility contexts, references/COW, and backend parity remain open. |
| Control flow, cleanup, diagnostics | **49%** | `[##########----------]` | Selected branches, loops, transfers, finalizers, output buffers, and diagnostics exist; broad unwind, handlers, destructors, shutdown flushing, and exact ordering remain open. |
| Broad integrated verification | **80%** | `[################----]` | Focused gates are strong for recent slices, including source/link proof for callable-array method frames, frame-local by-reference captures, root/reference by-reference captures, typed/default/variadic closure parameters, arrow implicit captures, and descriptor closure regressions; broad `native_link`/call-boundary suites still expose unrelated baseline failures, and backend parity/full end-to-end PHP proof lag. |

## Done / In Progress / Not Done

- [x] Done in primary: descriptor-backed by-value closure invocation, direct by-value closure captures, supported by-reference closure captures through root symbol/reference handles and promoted function-frame locals, non-static arrow implicit by-value captures, typed/default/variadic by-value descriptor closure parameters, untyped by-reference descriptor closure parameters, callable-array invocation for supported public static/object method frames, runtime same-family aggregate equality, runtime string-valued declared-class `new` for constructorless and supported public-constructor classes, public declared properties/methods/statics/constructors, inherited public slots, named `instanceof`, output buffers, selected function globals, assignment expressions, compound assignments, selected references, and selected LLVM/C assembly ABI consumers.
- [ ] In progress but uncounted: lane-local work around by-reference returns, method-table metadata, ArrayAccess/object offsets, root-symbol selection, callback/reference-slot cleanup, and blocker classification.
- [ ] Not done: general object model, arbitrary dynamic class-name expressions, contextual class names, non-public methods, overrides, interfaces/traits, magic methods, static closures, by-reference variadic/default/typed closure parameter combinations, unsupported typed/default closure cases, callable objects and unsupported callable-array forms, complete references/COW, request/`$GLOBALS` and missing-local by-reference capture roots, broad by-reference returns, complete mutation/unset, full diagnostics, full cleanup/unwind, includes, variable variables, request/global parity, and backend parity.

## Recent Primary-Integrated Work

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
- `impl-native-call-semantics` has relevant lane-local material around by-reference frame-local returns and broader callable behavior, but it remains broad relative to current primary.
- `impl-native-object-property-runtime` has ArrayAccess object-offset mutation dispatch evidence, but real method dispatch, references/COW, nested offsets, visibility/magic policy, and exact diagnostics remain blocked.
- `impl-function-frame-seed` and `impl-native-integration-batch` continue producing useful ABI/metadata surfaces, but many are not executable product semantics until primary consumers land.
- Several lanes are producing blocker-classifier and reference-slot cleanup work. Prefer candidates that remove a shared execution blocker or provide cross-feature linked proof. By-reference closure captures have now landed for supported root/reference paths and promoted function-frame locals; do not repeat that surface without a distinct request/`$GLOBALS`/missing-root, return, COW, or cleanup boundary.

## Current Review Notes

- Primary semantic work is committed at `7a43e1ac`; this progress update is the separate documentation wrapper before push.
- Focused gates for the latest callable-array method-frame slice passed using disk-backed `/tmp/phpc-primary-target`: `cargo check -q -p phpc -p php_runtime`, `cargo test -q -p php_runtime native_callable_array_parts_extract_string_and_object_targets`, `cargo test -q -p php_runtime native_callable_syntax_helpers_cover_array_and_value_families`, `cargo test -q -p phpc --test native_link native_executable_c_source_invokes_callable_arrays_through_method_frames`, `cargo test -q -p phpc --test native_link emit_exe_links_and_runs_callable_array_invocation_program`, `cargo test -q -p phpc --test native_link callable_array`, adjacent runtime string/fixed mixed dynamic-call source gates, callable-array syntax-only source/link gates, descriptor closure source gate, frame-local by-reference capture source/link gates, by-reference capture link gate, typed/default and variadic closure source gates, focused unsupported-shape and call-boundary gates, full `cargo test -q -p php_runtime`, scoped `rustfmt --edition 2021 --check compiler/src/codegen.rs compiler/tests/native_link.rs runtime/src/lib.rs`, and `git diff --check`.
- Broad attempted gates on the same `/tmp` target are still not fully clean on this baseline: full `native_link` reported 394 passed and 8 failed in unrelated string-offset/global-path/diagnostic tests plus one brittle by-value closure generated-source substring assertion; full `native_function_call_boundary` reported 38 passed and 2 failed in unsupported direct-call argument result count assertions.
