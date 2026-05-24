# PHP Native Compiler Progress

Updated: 2026-05-24 14:31 CEST
Evaluation marker: `20260524T114058Z`

Latest counted primary semantic/test baseline:
`d5e0e60f codegen: lower descriptor closure calls`

Latest primary head before this progress update:
`d5e0e60f codegen: lower descriptor closure calls`

Only pushed primary work counts here. Dirty WIP, lane-local candidates, parked diffs, exact-shape fixtures, and status-file claims are not product capability until selected, gated, committed, and pushed through primary.

## Executive Read

Overall estimated progress: **73%** `[###############-----]`

Executable PHP semantics: **73%** `[###############-----]`

Primary is making real integrated progress in selected generated-C execution islands. The current strongest area is declared objects/classes: supported declared allocation, ancestor-aware child allocation, inherited public properties, named `instanceof`, public instance methods, runtime string-valued public dynamic instance methods, supported public/inherited constructors for named and runtime string-valued `new`, public static methods, object static-receiver methods, and runtime string-valued constructorless declared-class `new`.

Generated-C now also has the first executable closure/callable frame slice: no-capture by-value fixed-parameter closures lower to descriptor-backed closure values, preserve descriptors through ordinary by-value frame slots, and invoke later through the shared dynamic-call path. Linked proof covers immediate closure calls, direct function-frame transfer, nested relay, dynamic named dispatch to a closure-consuming function, static method frames, inline closure arguments, and discarded closure calls.

This is still not general PHP. The remaining cliffs are large: full callable lookup/invocation, captured/arrow/by-reference/default/variadic closures, callable arrays/objects, arbitrary class-name expressions for `new`, non-public methods, override compatibility, interfaces/traits, contextual `self`/`parent`/`static`, magic methods, dynamic/static properties, references/COW identity, by-reference returns, request/global alias parity, includes, variable variables, exact/source-ordered diagnostics, cleanup/unwind/finally/destructors/output-buffer shutdown, and backend parity.

## Primary-Integrated Capability

- [x] Runtime/value foundations: shared native value, array, string, comparison, truthiness, diagnostic, request, reference, symbol, call-frame, output-buffer, object, object-property, method-diagnostic, and descriptor-closure ABI surfaces exist for selected paths.
- [x] Generated-C execution: direct variables, arrays/lvalues, selected dynamic calls, function frames, descriptor-backed no-capture closure frames, function-scope globals, `$GLOBALS` self-imports, by-reference parameter writes, assignment expressions, direct-variable compound assignments, output buffers, finalizer transfer slices, runtime string-valued declared-class `new` for constructorless and supported public-constructor classes, and the bounded declared-object family are executable.
- [x] LLVM consumers: selected string, predicate, search, integer/string helper, primitive assignment-expression, primitive compound-assignment, output-buffer, and native value-operation arithmetic paths consume shared ABIs.
- [x] C assembly fallback: selected unary string-result and two-operand string-predicate helpers consume shared runtime ABIs.
- [x] Object slice: generated-C can allocate supported declared class objects and declared child objects with ancestor metadata, instantiate declared classes through runtime string-valued class names when the generated class is constructorless or has a supported public constructor, preserve type/class identity across supported ancestor chains, execute public properties including inherited public slots and `unset`, evaluate named `instanceof`, call supported public/inherited instance methods with `$this`, call runtime string-valued public dynamic instance methods, run supported public/inherited constructors for named and runtime string-valued `new`, call supported named public static methods, and call supported public static methods through object receivers.
- [ ] Not complete: full PHP callable/closure, object, reference/COW, cleanup/unwind, diagnostic, request/global, include, variable-variable, and backend-parity behavior.

## Current Primary State

- [x] Primary semantic baseline is committed as `d5e0e60f`.
- [x] Primary head before this progress update is `d5e0e60f`.
- [x] Latest counted semantic work is generated-C descriptor-backed no-capture closure values and dynamic invocation through closure frame callbacks.
- [x] No uncounted dirty primary WIP was present before this progress update.
- [x] This progress update is expected to be committed as a wrapper docs commit; unrelated product diffs should remain untouched.

## Lane-Local Candidate Work

The `candidate-closure-callable-frames` lane produced the descriptor-backed closure frame slice that landed as `d5e0e60f`.

The `candidate-dynamic-constructor-new` lane has been integrated through primary as `6360acdf`.

Historical lane: `candidate-object-instantiation` produced the constructorless dynamic declared-class `new` slice that landed as `e2d20f3`. Do not repeat it.

Other active lanes contain useful but uncounted material around broader callable/frame flow, function-frame result-vector binding, ArrayAccess method-policy slots, readonly property metadata, request/global symbol operations, object/reference cleanup classification, reference/COW live-generator conformance, and diagnostic scanner boundaries. Prefer candidates that land as primary source/link proof or remove a real shared execution blocker.

## Roadmap Position

| Workstream | Estimate | Bar | Current read |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | **91%** | `[##################--]` | Strong shared surfaces exist and are consumed by output buffers, descriptor-backed closure values, and the declared-object/property/method family; some newer surfaces remain scaffolding until executable consumers land. |
| Compiler/backend consumers | **94%** | `[###################-]` | Generated-C is broad in selected areas, now including descriptor-backed closure values/calls and runtime string-valued declared-class `new` with constructorless and supported public-constructor dispatch; LLVM and C assembly consume selected ABI families. Direct assembly, LLVM object lowering, and many nested/object consumers remain blocked. |
| Executable PHP semantics | **73%** | `[###############-----]` | Many focused linked programs run, including descriptor-backed no-capture closure invocation across frame-transfer families, runtime string-valued declared-class `new` with supported public constructor dispatch, constructorless dynamic declared-class `new`, and inherited public object/method/static/constructor paths, but execution is still selected islands rather than general PHP. |
| Arrays, lvalues, references, COW | **64%** | `[#############-------]` | Good selected lvalue/reference paths; full COW, arbitrary roots, foreach, object joins, and alias composition remain open. |
| Symbols, globals, request state | **72%** | `[##############------]` | Function globals and `$GLOBALS` self-imports improved; request superglobal imports, includes, variable variables, and exact unset behavior remain incomplete. |
| Calls, functions, frames | **68%** | `[##############------]` | Bounded functions, descriptor-backed no-capture closure frames, and public declared object/method/constructor call paths work in selected generated-C cases, including runtime string-valued dynamic constructor dispatch for generated declared classes; captured/arrow/by-reference/default/variadic closures, callable arrays/objects, named/unpacked args, arbitrary class-name expressions, non-public contexts, and by-reference returns remain open. |
| Objects, properties, methods | **43%** | `[#########-----------]` | Useful declared-object subset exists, including runtime string-valued declared-class `new` for constructorless and supported public-constructor classes; non-public methods, overrides, interfaces/traits, magic methods, dynamic/static properties, visibility contexts, references/COW, and backend parity remain open. |
| Control flow, cleanup, diagnostics | **49%** | `[##########----------]` | Selected branches, loops, transfers, finalizers, output buffers, and diagnostics exist; broad unwind, handlers, destructors, shutdown flushing, and exact ordering remain open. |
| Broad integrated verification | **70%** | `[##############------]` | Focused gates are strong for recent slices, including descriptor-backed closure frame transfer and dynamic public-constructor declared-class `new`; cross-feature linked programs, backend parity, and full end-to-end PHP proof lag. |

## Done / In Progress / Not Done

- [x] Done in primary: bounded generated-C direct variables, arrays/lvalues, selected dynamic calls, function globals, `$GLOBALS` self-imports, descriptor-backed no-capture by-value closure frames and dynamic invocation, by-reference parameter writes, assignment expressions, direct-variable compound assignments, output buffers, finalizer transfer slices, declared-class allocation including runtime string-valued declared-class `new` for constructorless and supported public-constructor classes, supported child allocation with ancestor metadata, constructorless argument evaluation, public/inherited public constructors for named and runtime string-valued `new`, public/inherited public instance/static methods, runtime string-valued public dynamic instance methods, public object static-receiver methods, public properties including inherited public slots and unset, and named `instanceof`.
- [x] Done in primary: LLVM value-operation arithmetic, primitive direct-variable assignment-expression and compound-assignment paths, lowerable output-buffer calls, and C assembly fallback string-result/string-predicate slices.
- [ ] In progress but uncounted: lane-local candidates for broader callable dispatch, object/static property metadata, frame contracts, symbol/reference transport, diagnostics, and cleanup boundaries.
- [ ] Not done: general object model, arbitrary dynamic class-name expressions, contextual class names, non-public methods, overrides, interfaces/traits, magic methods, captured/arrow/by-reference/default/variadic closures, callable-array/object invocation, complete references/COW, by-reference returns, complete mutation/unset, full diagnostics, full cleanup/unwind, includes, variable variables, request/global parity, and direct assembly parity.

## Recent Primary-Integrated Work

- `d5e0e60f`: generated-C no-capture by-value fixed-parameter closures lower to descriptor-backed closure values with generated frame callbacks. Runtime closure descriptor ABI materializes, identifies, and invokes descriptor closures through the shared dynamic-call path, including values cloned through ordinary by-value function/static-method frames. Source/link proof covers immediate invocation, direct function-frame transfer, nested relay, dynamic named dispatch to a closure-consuming function, static method frames, inline closure arguments, discarded closure calls, runtime arity/no-descriptor diagnostics, and unsupported capture/reference/default/variadic/global-root closure shapes staying on the shared closure blocker. Dynamic function-call blocker wording now distinguishes descriptor-backed closures from non-descriptor closures.
- `6360acdf`: generated-C `new $class(...)` supports runtime string-valued direct-variable class names for generated declared classes that require supported public constructor dispatch. The dynamic class-name value is matched against all registered declared-class candidates through the shared dynamic-name helper; matched candidates allocate through existing declared-class or ancestor-aware allocation helpers, then call the existing declared constructor frame boundary with `$this`, defaults, argument materialization, status handling, and cleanup ownership. Source/link proof covers multiple constructor classes, case-insensitive runtime class names, default and explicit constructor arguments, argument side effects, declared-child allocation using an inherited public constructor, property/debug-type/`instanceof` consumers, constructorless dynamic `new` regressions, and private/unsupported constructor shapes staying blocked.
- `e2d20f3`: generated-C `new $class(...)` supports runtime string-valued direct-variable class names for generated constructorless declared classes. Dynamic class names are materialized as native values, matched case-insensitively against registered declared-class candidates through the shared dynamic-name helper, and allocated through existing declared-class or ancestor-aware helpers. Constructorless named and dynamic `new` now evaluate and clean up argument expressions. Source/link proof covers named constructorless arguments, case-insensitive runtime class names, multiple declared classes, declared child allocation through ancestor metadata, inherited public property writes/reads, named ancestor `instanceof`, `is_object()`, `gettype()`, `get_debug_type()`, and dynamic class names requiring constructor dispatch staying blocked.
- `de7ee6d2`: generated-C declared child objects allocate through ancestor-aware metadata, preserving inherited public property/method/static/constructor behavior and ancestor relation checks.
- `99b0fa3f`: generated-C runtime string-valued public dynamic instance-method calls dispatch through shared dynamic-call name matching and object/class relation checks.
- `5e5ab57c`: generated-C public object static-receiver calls dispatch across declared public static method candidates through shared object/class relation checks.
- `de8e9634`: generated-C named public static method calls execute through receiverless declared-method frames.
- `b099039e`, `c00780c3`, `a792e8b5`, `06f699f8`, `9d637923`, `07516bc3`, and `f6d9ad0a`: earlier object/property/method/constructor/instanceof/allocation/output-buffer foundations remain part of the counted primary baseline.

## Current Review Notes

- Primary semantic baseline is `d5e0e60f`; primary head was clean at `d5e0e60f` before this progress update.
- Focused gates for the latest descriptor-backed closure slice passed using disk-backed `/tmp` targets: `cargo check -q -p phpc`, `cargo test -q -p php_runtime native_descriptor_closure_values_invoke_through_frame_callback`, `cargo test -q -p phpc --test native_link native_executable_c_source_routes_descriptor_closures_through_shared_runtime_abi`, `cargo test -q -p phpc --test native_link emit_exe_links_and_runs_immediate_descriptor_closure_invocation`, `cargo test -q -p phpc --test native_link emit_exe_links_and_runs_descriptor_closure_after_by_value_frame_transfer`, `cargo test -q -p phpc --test native_link native_executable_c_source_keeps_unsupported_closure_shapes_on_shared_blocker`, `cargo test -q -p phpc --test native_link emit_exe_links_and_runs_runtime_dynamic_user_function_call_program`, `cargo test -q -p phpc --test native_link emit_exe_reports_runtime_dynamic_user_function_call_failures`, `cargo test -q -p phpc --test native_function_call_boundary native_executable_c_source_routes_call_operation_blockers_across_call_families`, `cargo test -q -p phpc native_call_diagnostics_centralizes_backend_recovery_across_call_families`, `rustfmt --edition 2021 --check compiler/src/codegen.rs compiler/tests/native_link.rs compiler/tests/native_function_call_boundary.rs runtime/src/lib.rs`, and `git diff --check`.
- The broader full `cargo test -q -p phpc --test native_function_call_boundary` gate was not re-run for this slice; the previous known unrelated direct-call column expectation failures remain the last full-test observation.
- Live `/dev/shm` check: 22G total, 15G used, 7.8G available, 65% used. Use disk-backed `/tmp` targets for broad gates and owner-check before cleaning shared-memory targets.
- Live `/home` check: 459G total, 290G used, 150G available, 66% used.
