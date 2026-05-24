# PHP Native Compiler Progress

Updated: 2026-05-24 15:28 CEST
Evaluation marker: `20260524T132405Z`

Latest counted primary semantic/test baseline:
`deabcd6d codegen: lower by-reference closure parameters`

Latest primary head before this progress update:
`deabcd6d codegen: lower by-reference closure parameters`

Latest observed `origin/master` during this review:
`f0188cf9 docs: update progress after closure captures`

Accounting rule: only generalized, tested, committed, and pushed primary work counts. Dirty primary WIP, lane-local candidates, parked diffs, blocker-only classifiers, and status-file claims are not product capability until selected, gated, committed, and pushed through primary.

Accounting history: the earlier **88%** headline is retired because it counted lane-local candidates, scaffolding, and ABI surface area too generously before they became executable product behavior. The later **50%** figure was a conservative strict-rubric rebaseline, not a code rollback. The current source-of-truth estimate is **75%** under the stricter rule.

## Executive Read

Overall estimated progress: **75%** `[###############-----]`

Executable PHP semantics: **75%** `[###############-----]`

Primary has real integrated progress in selected generated-C execution islands. The current counted baseline includes descriptor-backed closure invocation, direct by-value closure captures, untyped by-reference descriptor closure parameters, same-family aggregate equality, runtime string-valued declared-class `new` for constructorless and supported public-constructor classes, and a bounded public declared-object family.

Generated-C descriptor closures now carry a shared native closure-argument carrier that can hold either an owned value or a reference handle. Runtime descriptor metadata marks untyped by-reference parameters; dynamic closure invocation materializes supported direct-variable and nested array lvalue arguments through the shared symbol/reference ABI, while by-value parameters and direct by-value captures continue through ordinary value handles.

This is still not general PHP. The remaining cliffs are large: full callable lookup/invocation, arrow/static/default/variadic/typed closures, by-reference captures and broader returns, callable arrays/objects, arbitrary class-name expressions for `new`, non-public methods, overrides, interfaces/traits, contextual `self`/`parent`/`static`, magic methods, dynamic/static properties, references/COW identity, request/global alias parity, includes, variable variables, exact/source-ordered diagnostics, cleanup/unwind/finally/destructors/output-buffer shutdown, and backend parity.

## Primary-Integrated Capability

- [x] Runtime/value foundations: shared native value, array, string, comparison, truthiness, diagnostic, request, reference, symbol, call-frame, output-buffer, object, object-property, method-diagnostic, and descriptor-closure ABI surfaces exist for selected paths, including capture-aware and by-reference-argument descriptor closures.
- [x] Generated-C execution: direct variables, arrays/lvalues, selected dynamic calls, function frames, descriptor-backed by-value closure frames with direct by-value captures and untyped by-reference closure parameters, function globals, `$GLOBALS` self-imports, selected references, assignment expressions, output buffers, finalizer transfer slices, runtime string-valued declared-class `new`, and the bounded public declared-object family are executable.
- [x] Runtime comparison: arrays, native array handles, objects/closures, and resources now have same-family loose equality through one recursive comparison context.
- [x] LLVM and C assembly consumers: selected string, predicate, search, integer/string helper, primitive assignment/compound-assignment, output-buffer, value-operation arithmetic, and string-result/string-predicate helper paths consume shared ABIs.
- [ ] Not complete: full PHP callable/closure, object, reference/COW, cleanup/unwind, diagnostic, request/global, include, variable-variable, and backend-parity behavior.

## Grand Roadmap Position

| Workstream | Estimate | Bar | Current read |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | **94%** | `[###################-]` | Strong shared surfaces exist and are consumed by output buffers, capture-aware and by-reference-argument descriptor closures, aggregate equality, and declared-object paths; newer surfaces still need executable consumers. |
| Compiler/backend consumers | **96%** | `[###################-]` | Generated-C is broad in selected areas; LLVM and C assembly consume selected ABI families. Direct assembly, LLVM object lowering, and many nested/object consumers remain blocked. |
| Executable PHP semantics | **75%** | `[###############-----]` | Many focused linked/runtime programs run, including by-reference descriptor closure parameter execution, but execution remains selected islands rather than general PHP. |
| Arrays, lvalues, references, COW | **65%** | `[#############-------]` | Good selected lvalue/reference paths now feed descriptor closure parameters; full COW, arbitrary roots, foreach, object joins, and alias composition remain open. |
| Symbols, globals, request state | **72%** | `[##############------]` | Function globals and `$GLOBALS` self-imports improved; request superglobal imports, includes, variable variables, and exact unset behavior remain incomplete. |
| Calls, functions, frames | **70%** | `[##############------]` | Bounded functions, descriptor-backed by-value closure frames/captures, untyped by-reference closure parameters, and public object/method/constructor call paths work in selected generated-C cases. |
| Objects, properties, methods | **43%** | `[#########-----------]` | Useful public declared-object subset exists; non-public methods, overrides, interfaces/traits, magic methods, dynamic/static properties, visibility contexts, references/COW, and backend parity remain open. |
| Control flow, cleanup, diagnostics | **49%** | `[##########----------]` | Selected branches, loops, transfers, finalizers, output buffers, and diagnostics exist; broad unwind, handlers, destructors, shutdown flushing, and exact ordering remain open. |
| Broad integrated verification | **73%** | `[###############-----]` | Focused gates are strong for recent slices, including runtime/source/link proof for by-reference descriptor closure parameters; cross-feature linked programs, backend parity, and full end-to-end PHP proof lag. |

## Done / In Progress / Not Done

- [x] Done in primary: descriptor-backed by-value closure invocation, direct by-value closure captures, untyped by-reference descriptor closure parameters, runtime same-family aggregate equality, runtime string-valued declared-class `new` for constructorless and supported public-constructor classes, public declared properties/methods/statics/constructors, inherited public slots, named `instanceof`, output buffers, selected function globals, assignment expressions, compound assignments, selected references, and selected LLVM/C assembly ABI consumers.
- [ ] In progress but uncounted: lane-local work around by-reference returns, arrow captures, method-table metadata, ArrayAccess/object offsets, root-symbol selection, callback/reference-slot cleanup, and blocker classification.
- [ ] Not done: general object model, arbitrary dynamic class-name expressions, contextual class names, non-public methods, overrides, interfaces/traits, magic methods, arrow/static/default/variadic closures, callable-array/object invocation, complete references/COW, by-reference captures, broad by-reference returns, complete mutation/unset, full diagnostics, full cleanup/unwind, includes, variable variables, request/global parity, and backend parity.

## Recent Primary-Integrated Work

- `deabcd6d`: generated-C descriptor closures support untyped by-reference parameters through shared descriptor parameter metadata, `phpc_NativeClosureArgument` value/reference carriers, runtime reference-argument diagnostics, and the existing dynamic-call path. Linked proof covers direct closure calls, user-function relay, runtime dynamic relay, direct variable lvalues, and nested array lvalues while by-reference captures, typed/default/variadic closure parameters, arrow/static closures, root/global frame handoff, callable arrays/objects, and backend parity remain blocked.
- `2f306cea`: generated-C descriptor-ready closures support direct by-value `use (...)` captures. Capture values are copied into closure descriptor payloads, rebound as closure-frame locals during invocation, and proven through stored closures, immediate closures, repeated calls, function-frame relay, and outer-variable mutation isolation.
- `6dda705d`: native runtime value comparison executes same-family loose equality for arrays, native array handles, declared objects/closures, and resources through a shared comparison context.
- `d5e0e60f`: generated-C no-capture by-value fixed-parameter closures lower to descriptor-backed closure values and invoke through the shared dynamic-call path, including ordinary by-value frame transfer.
- `6360acdf`: generated-C `new $class(...)` supports runtime string-valued direct-variable class names for generated declared classes requiring supported public constructor dispatch.
- `e2d20f3`: generated-C `new $class(...)` supports runtime string-valued direct-variable class names for generated constructorless declared classes.

## Lane-Local Candidate Work

- The by-reference descriptor closure parameter candidate has landed through primary as `deabcd6d`; treat the previous candidate lane as historical unless it is explicitly rebased/repurposed from current primary.
- `impl-native-call-semantics` has relevant lane-local material around by-reference frame-local returns and arrow implicit captures, but it remains broad relative to current primary.
- `impl-native-object-property-runtime` has ArrayAccess object-offset mutation dispatch evidence, but real method dispatch, references/COW, nested offsets, visibility/magic policy, and exact diagnostics remain blocked.
- `impl-function-frame-seed` and `impl-native-integration-batch` continue producing useful ABI/metadata surfaces, but many are not executable product semantics until primary consumers land.
- Several lanes are producing blocker-classifier and reference-slot cleanup work. Prefer candidates that remove a shared execution blocker or provide cross-feature linked proof.

## Current Review Notes

- Primary semantic work is committed at `deabcd6d`; this progress update is the separate documentation wrapper before push.
- Focused gates for the latest by-reference closure-parameter slice passed using disk-backed `/tmp` targets: `cargo check -q -p phpc`, `cargo test -q -p php_runtime native_descriptor_closure_values_bind_reference_arguments_through_frame_callback`, `cargo test -q -p phpc --test native_link native_executable_c_source_binds_descriptor_closure_by_reference_parameters`, `cargo test -q -p phpc --test native_link emit_exe_links_and_runs_descriptor_closure_by_reference_parameter_program`, `cargo test -q -p phpc --test native_link descriptor_closure`, `cargo test -q -p phpc --test native_link emit_exe_links_and_runs_by_value_captured_descriptor_closure_invocation`, `cargo test -q -p phpc --test native_link native_executable_c_source_keeps_unsupported_closure_shapes_on_shared_blocker`, `cargo test -q -p phpc --test native_function_call_boundary native_executable_c_source_routes_call_operation_blockers_across_call_families`, full `cargo test -q -p php_runtime -- --test-threads=1`, `cargo test -q -p phpc native_function_frame_blocker_classifies_parameter_and_return_families`, `cargo test -q -p phpc native_call_diagnostics_centralizes_backend_recovery_across_call_families`, `rustfmt --edition 2021 --check compiler/src/codegen.rs compiler/tests/native_link.rs compiler/tests/native_function_call_boundary.rs runtime/src/lib.rs`, and `git diff --check`.
- Live resource check: `/dev/shm` 22G total, 8.6G used, 14G available, 39% used; `/home` 459G total, 297G used, 143G available, 68% used. Use disk-backed targets or owner-aware cleanup for broad gates.
