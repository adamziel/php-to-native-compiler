# PHP Native Compiler Progress

Updated: 2026-05-24 16:30 CEST
Evaluation marker: `20260524T142625Z`

Latest counted primary semantic/test baseline:
`ff1d8ee3 codegen: lower typed default closure parameters`

Latest primary head before this progress update:
`ff1d8ee3 codegen: lower typed default closure parameters`

Latest observed `origin/master` during this review:
`ff1d8ee3 codegen: lower typed default closure parameters`

Accounting rule: only generalized, tested, committed, and pushed primary work counts. Current primary is synced to `origin/master`; dirty primary WIP, lane-local candidates, parked diffs, blocker-only classifiers, and status-file claims are not product capability until selected, gated, committed, and pushed through primary.

Accounting history: the earlier **88%** headline is retired because it counted lane-local candidates, scaffolding, and ABI surface area too generously before they became executable product behavior. The later **50%** figure was a conservative strict-rubric rebaseline, not a code rollback. The current source-of-truth estimate is **77%** under the stricter rule.

## Executive Read

Overall estimated progress: **77%** `[###############-----]`

Executable PHP semantics: **77%** `[###############-----]`

Primary has real integrated progress in selected generated-C execution islands. The current counted baseline includes descriptor-backed closure invocation, direct by-value closure captures, non-static arrow closures with implicit by-value captures, typed/default by-value descriptor closure parameters, untyped by-reference descriptor closure parameters, same-family aggregate equality, runtime string-valued declared-class `new` for constructorless and supported public-constructor classes, and a bounded public declared-object family.

Generated-C descriptor closures now carry a shared native closure-argument carrier that can hold either an owned value or a reference handle. Runtime descriptor metadata marks untyped by-reference parameters and descriptor arity distinguishes required and total parameters. Closure-frame binding can fill missing supported by-value parameters from parsed defaults and route supported scalar/array/mixed parameter types through the shared call-frame type ABI. Dynamic closure invocation materializes supported direct-variable and nested array lvalue arguments through the shared symbol/reference ABI. Non-static arrow closures synthesize by-value captures from generalized AST variable use and feed those captures through the same descriptor capture ABI, including nested arrow propagation, nested regular closures with explicit `use (...)`, array-key captures, and composition with untyped by-reference arrow parameters.

This is still not general PHP. The remaining cliffs are large: full callable lookup/invocation, static closures, variadic closure parameters, unsupported typed/default closure cases, by-reference captures and broader returns, callable arrays/objects, arbitrary class-name expressions for `new`, non-public methods, overrides, interfaces/traits, contextual `self`/`parent`/`static`, magic methods, dynamic/static properties, references/COW identity, request/global alias parity, includes, variable variables, exact/source-ordered diagnostics, cleanup/unwind/finally/destructors/output-buffer shutdown, and backend parity.

## Primary-Integrated Capability

- [x] Runtime/value foundations: shared native value, array, string, comparison, truthiness, diagnostic, request, reference, symbol, call-frame, output-buffer, object, object-property, method-diagnostic, and descriptor-closure ABI surfaces exist for selected paths, including capture-aware and by-reference-argument descriptor closures.
- [x] Generated-C execution: direct variables, arrays/lvalues, selected dynamic calls, function frames, descriptor-backed by-value closure frames with direct by-value captures, non-static arrow implicit by-value captures, typed/default by-value closure parameters, and untyped by-reference closure parameters, function globals, `$GLOBALS` self-imports, selected references, assignment expressions, output buffers, finalizer transfer slices, runtime string-valued declared-class `new`, and the bounded public declared-object family are executable.
- [x] Runtime comparison: arrays, native array handles, objects/closures, and resources now have same-family loose equality through one recursive comparison context.
- [x] LLVM and C assembly consumers: selected string, predicate, search, integer/string helper, primitive assignment/compound-assignment, output-buffer, value-operation arithmetic, and string-result/string-predicate helper paths consume shared ABIs.
- [ ] Not complete: full PHP callable/closure, object, reference/COW, cleanup/unwind, diagnostic, request/global, include, variable-variable, and backend-parity behavior.

## Grand Roadmap Position

| Workstream | Estimate | Bar | Current read |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | **94%** | `[###################-]` | Strong shared surfaces exist and are consumed by output buffers, capture-aware and by-reference-argument descriptor closures, aggregate equality, and declared-object paths; newer surfaces still need executable consumers. |
| Compiler/backend consumers | **96%** | `[###################-]` | Generated-C is broad in selected areas; LLVM and C assembly consume selected ABI families. Direct assembly, LLVM object lowering, and many nested/object consumers remain blocked. |
| Executable PHP semantics | **77%** | `[###############-----]` | Many focused linked/runtime programs run, including typed/default by-value descriptor closure parameters, by-reference descriptor closure parameters, and arrow implicit capture execution, but execution remains selected islands rather than general PHP. |
| Arrays, lvalues, references, COW | **65%** | `[#############-------]` | Good selected lvalue/reference paths now feed descriptor closure parameters; full COW, arbitrary roots, foreach, object joins, and alias composition remain open. |
| Symbols, globals, request state | **72%** | `[##############------]` | Function globals and `$GLOBALS` self-imports improved; request superglobal imports, includes, variable variables, and exact unset behavior remain incomplete. |
| Calls, functions, frames | **72%** | `[##############------]` | Bounded functions, descriptor-backed by-value closure frames/captures, non-static arrow implicit captures, typed/default by-value closure parameters, untyped by-reference closure parameters, and public object/method/constructor call paths work in selected generated-C cases. |
| Objects, properties, methods | **43%** | `[#########-----------]` | Useful public declared-object subset exists; non-public methods, overrides, interfaces/traits, magic methods, dynamic/static properties, visibility contexts, references/COW, and backend parity remain open. |
| Control flow, cleanup, diagnostics | **49%** | `[##########----------]` | Selected branches, loops, transfers, finalizers, output buffers, and diagnostics exist; broad unwind, handlers, destructors, shutdown flushing, and exact ordering remain open. |
| Broad integrated verification | **75%** | `[###############-----]` | Focused gates are strong for recent slices, including source/link proof for typed/default closure parameters, arrow implicit captures, and descriptor closure regressions; cross-feature linked programs, backend parity, and full end-to-end PHP proof lag. |

## Done / In Progress / Not Done

- [x] Done in primary: descriptor-backed by-value closure invocation, direct by-value closure captures, non-static arrow implicit by-value captures, typed/default by-value descriptor closure parameters, untyped by-reference descriptor closure parameters, runtime same-family aggregate equality, runtime string-valued declared-class `new` for constructorless and supported public-constructor classes, public declared properties/methods/statics/constructors, inherited public slots, named `instanceof`, output buffers, selected function globals, assignment expressions, compound assignments, selected references, and selected LLVM/C assembly ABI consumers.
- [ ] In progress but uncounted: lane-local work around by-reference returns, method-table metadata, ArrayAccess/object offsets, root-symbol selection, callback/reference-slot cleanup, and blocker classification.
- [ ] Not done: general object model, arbitrary dynamic class-name expressions, contextual class names, non-public methods, overrides, interfaces/traits, magic methods, static closures, variadic closure parameters, unsupported typed/default closure cases, callable-array/object invocation, complete references/COW, by-reference captures, broad by-reference returns, complete mutation/unset, full diagnostics, full cleanup/unwind, includes, variable variables, request/global parity, and backend parity.

## Recent Primary-Integrated Work

- `ff1d8ee3`: generated-C descriptor closures support supported typed/default by-value parameters. Descriptor metadata now publishes required arg count separately from total param count, closure callbacks bind missing by-value parameters from parsed default expressions, typed closure parameters use the shared call-frame type coercion ABI, and captures start after the actual runtime call-arg prefix. Source and linked proof cover direct closure calls, user-function relay, public static method relay, arrows with defaults, captured closures with defaults, and descriptor/by-reference/arrow/user-function regressions while variadic closure parameters, by-reference captures, by-reference returns, static arrows, callable arrays/objects, named/unpacked args, request/global separation, references/COW, exact diagnostics, cleanup/unwind, and backend parity remain blocked.
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
- `impl-native-call-semantics` has relevant lane-local material around by-reference frame-local returns and broader callable behavior, but it remains broad relative to current primary.
- `impl-native-object-property-runtime` has ArrayAccess object-offset mutation dispatch evidence, but real method dispatch, references/COW, nested offsets, visibility/magic policy, and exact diagnostics remain blocked.
- `impl-function-frame-seed` and `impl-native-integration-batch` continue producing useful ABI/metadata surfaces, but many are not executable product semantics until primary consumers land.
- Several lanes are producing blocker-classifier and reference-slot cleanup work. Prefer candidates that remove a shared execution blocker or provide cross-feature linked proof.

## Current Review Notes

- Primary semantic work is committed and pushed at `ff1d8ee3`; this progress update is the separate documentation wrapper before push.
- Focused gates for the latest typed/default closure-parameter slice passed using disk-backed `/tmp/phpc-target-primary-closure-param-integration`: `cargo check -q -p phpc -p php_runtime`, `cargo test -q -p phpc --test native_link closure_parameter`, `cargo test -q -p phpc --test native_link descriptor_closure`, `cargo test -q -p phpc --test native_link native_executable_c_source_keeps_unsupported_closure_shapes_on_shared_blocker`, `cargo test -q -p phpc --test native_function_call_boundary native_executable_c_source_routes_call_operation_blockers_across_call_families`, `cargo test -q -p phpc --test native_link emit_exe_links_and_runs_descriptor_closure_by_reference_parameter_program`, `cargo test -q -p phpc --test native_link emit_exe_links_and_runs_arrow_closure_implicit_capture_program`, `cargo test -q -p phpc --test native_link emit_exe_links_and_runs_typed_user_function_frame_program`, scoped `rustfmt --edition 2021 --check compiler/src/codegen.rs compiler/tests/native_link.rs compiler/tests/native_function_call_boundary.rs`, `git diff --check`, and `git diff --cached --check`.
- Live resource check: `/dev/shm` 22G total, 15G used, 7.7G available, 65% used; `/home` 459G total, 334G used, 106G available, 76% used. Use disk-backed targets or owner-aware cleanup for broad gates.
