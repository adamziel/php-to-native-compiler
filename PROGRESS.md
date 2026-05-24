# PHP Native Compiler Progress

Updated: 2026-05-24 12:51 CEST
Evaluation marker: `20260524T105131Z`

Latest counted primary semantic/test baseline:
`de7ee6d2 codegen: lower declared class inheritance`

Latest primary head before this progress update:
`e1ee1314 docs: update progress after declared inheritance`

Only pushed primary work counts here. Dirty WIP, lane-local candidates, parked diffs, exact-shape fixtures, and status-file claims are not product capability until selected, gated, committed, and pushed through primary.

## Progress Accounting Note

The current headline number is **overall compiler completion**, estimated against generalized, end-to-end PHP semantics. The Runtime/ABI foundations number below is a workstream score, not the whole compiler. Older headline estimates near 88% used a looser rubric that counted selected generated-C execution islands too heavily; they are retired and not comparable with the current overall estimate.

## Executive Read

Overall estimated progress: **72%** `[##############------]`

Executable PHP semantics: **70%** `[##############------]`

Primary is making real integrated progress in selected native PHP execution islands. The strongest recent momentum is the generated-C object path: declared class allocation, declared child allocation with ancestor metadata, public property read/write/`isset`/`empty`/`unset` including inherited public slots, named `instanceof` across supported ancestor chains, public instance methods with `$this`, runtime string-valued public dynamic instance methods, supported public and inherited constructors, named public static methods including inherited lookup, and public object static-receiver method calls now share runtime/frame/property, object/class relation, dynamic-call name, or declared-method frame boundaries and have focused source/link proof.

This is not general PHP yet. The remaining cliffs are still large: full callable lookup/invocation, closures, dynamic method forms outside the declared public instance subset, non-public methods, full inheritance/interfaces/traits, contextual `self`/`parent`/`static`, method/property override compatibility, dynamic/static properties, magic methods, references/COW identity, by-reference returns, request/global alias parity, includes, variable variables, exact/source-ordered diagnostics, cleanup/unwind/finally/destructors/output-buffer shutdown, and backend parity.

The latest inheritance slice extends generated-C declared objects through ancestor-aware allocation metadata. Supported child objects now preserve ancestor class names and inherited property declaring-class metadata, so named `instanceof`, public property operations, public inherited instance-method dispatch, runtime string-valued inherited dynamic instance-method dispatch, inherited public constructor dispatch, named inherited public static calls, and public object static-receiver calls reuse the existing object/class relation ABI and declared-method frame boundary. Non-public visibility execution, method/property overrides, interfaces/traits, magic methods, contextual `self`/`parent`/`static`, callable arrays/objects/closures, exact diagnostics, LLVM/direct assembly object lowering, and broad object/reference/COW semantics remain blocked.

## Primary-Integrated Capability

- [x] Runtime/value foundations: shared native value, array, string, comparison, truthiness, diagnostic, request, reference, symbol, call-frame, output-buffer, object, object-property, and method-diagnostic ABI surfaces exist for selected paths.
- [x] Generated-C execution: direct variables, arrays/lvalues, selected dynamic calls, function frames, function-scope globals, `$GLOBALS` self-imports, by-reference parameter writes, assignment expressions, direct-variable compound assignments, output buffers, finalizer transfer slices, and the bounded declared-object family are executable.
- [x] LLVM consumers: selected string, predicate, search, integer/string helper, primitive assignment-expression, primitive compound-assignment, output-buffer, and native value-operation arithmetic paths consume shared ABIs.
- [x] C assembly fallback: selected unary string-result and two-operand string-predicate helpers consume shared runtime ABIs.
- [x] Object slice: generated-C can allocate supported declared class objects and declared child objects with ancestor metadata, preserve type/class identity across supported ancestor chains, execute public properties including inherited public slots and `unset`, evaluate named `instanceof`, call supported public and inherited instance methods with `$this` using static or runtime string method names, run supported public and inherited constructors, call supported named public static methods without `$this`, and call supported public static methods through object receivers.
- [ ] Not complete: full PHP callable, object, reference/COW, cleanup/unwind, diagnostic, request/global, include, variable-variable, and backend-parity behavior.

## Current Primary State

- [x] Primary semantic work is committed as `de7ee6d2`.
- [x] Latest counted semantic commit is `de7ee6d2`.
- [x] Primary head before this evaluator progress update is synced with `origin/master` at `e1ee1314`.
- [x] No uncounted dirty primary WIP was present before this evaluator progress update.
- [x] This evaluator progress update is expected to be committed as a wrapper docs commit; unrelated product diffs should remain untouched.

## Lane-Local Candidate Work

Fresh worker statuses show broad candidate inventory, including callable/closure frame and by-reference parameter/result contracts, linked local symbol-table routing, request/diagnostic snapshots and custom-handler blockers, object-property reference dispatch boundaries, reference/COW live-generator conformance gates, control-flow cleanup and target-state planning, and runtime object-instantiation candidate ABIs.

These are useful inputs, not product capability. Prefer candidates that land as primary source/link proof or remove a real shared execution blocker. Treat blocker-only or metadata-only slices as lower priority unless they unlock an immediate executable consumer.

## Roadmap Position

| Workstream | Estimate | Bar | Current read |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | **90%** | `[##################--]` | Strong shared surfaces exist, now consumed by output buffers and the declared-object/property/method family, including ancestor-aware declared object allocation and dynamic method-name miss diagnostics; some surfaces remain scaffolding until broader consumers land. |
| Compiler/backend consumers | **91%** | `[##################--]` | Generated-C is broad in selected areas, now including supported declared inheritance allocation, inherited public properties, inherited public constructors, and named/runtime public instance/static method paths; LLVM and C assembly consume selected ABI families. Direct assembly and many nested/object consumers remain blocked. |
| Executable PHP semantics | **70%** | `[##############------]` | Many focused linked programs run, now including supported declared child objects, inherited public property/method/static/constructor execution, public object static-receiver calls, and runtime string-valued public dynamic instance methods, but semantics are still selected islands rather than general PHP. |
| Arrays, lvalues, references, COW | **64%** | `[#############-------]` | Good selected lvalue/reference paths; full COW, arbitrary roots, foreach, object joins, and alias composition remain open. |
| Symbols, globals, request state | **72%** | `[##############------]` | Function globals and `$GLOBALS` self-imports improved; request superglobal imports, includes, variable variables, and exact unset behavior remain incomplete. |
| Calls, functions, frames | **64%** | `[#############-------]` | Bounded functions, public and inherited public instance methods, runtime string-valued dynamic public instance methods, public and inherited constructors, named public static methods, and public object static-receiver calls work for selected generated-C paths; closures, callable arrays/objects, named/unpacked args, non-public contexts, and by-reference returns remain open. |
| Objects, properties, methods | **41%** | `[########------------]` | Primary now has a useful declared-object subset with supported inheritance metadata, inherited public properties, public instance/static method execution, public dynamic instance-method names, inherited public constructors, and public object static-receiver dispatch; non-public methods, overrides, interfaces/traits, magic methods, dynamic/static properties, visibility contexts, references/COW, and backend parity remain open. |
| Control flow, cleanup, diagnostics | **49%** | `[##########----------]` | Selected branches, loops, transfers, finalizers, output buffers, and diagnostics exist; broad unwind, handlers, destructors, shutdown flushing, and exact ordering remain open. |
| Broad integrated verification | **67%** | `[#############-------]` | Focused gates are strong, including source/link proof for declared inheritance, named public static methods, object static-receiver dispatch, and runtime string-valued public dynamic instance methods; cross-feature linked programs, backend parity, and full end-to-end PHP proof lag. |

## Done / In Progress / Not Done

- [x] Done in primary: bounded generated-C direct variables, arrays/lvalues, selected dynamic calls, function globals, `$GLOBALS` self-imports, by-reference parameter writes, assignment expressions, direct-variable compound assignments, output buffers, finalizer transfer slices, declared-class allocation including supported child allocation with ancestor metadata, public and inherited public constructors, public and inherited public instance/static methods including runtime string-valued public dynamic instance methods, public object static-receiver methods, public properties including inherited public slots and unset, and named `instanceof`.
- [x] Done in primary: LLVM value-operation arithmetic, primitive direct-variable assignment-expression and compound-assignment paths, lowerable output-buffer calls, and C assembly fallback string-result/string-predicate slices.
- [ ] In progress but uncounted: lane-local candidates for callable dispatch, object/static property metadata, frame contracts, symbol/reference transport, diagnostics, and cleanup boundaries.
- [ ] Not done: general object model, dynamic method forms outside declared public instance dispatch, non-public methods, overrides, interfaces/traits, unsupported contextual class names, closures, callable-array/object invocation, complete references/COW, by-reference returns, complete mutation/unset, full diagnostics, full cleanup/unwind, includes, variable variables, request/global parity, and direct assembly parity.

## Recent Primary-Integrated Work

- `de7ee6d2`: generated-C declared child objects now allocate through an ancestor-aware object ABI when supported inheritance metadata is present. The compiler preserves ancestor names and property declaring-class metadata, inherited public properties flow through the existing public object-property ABI, named `instanceof` and receiver checks consume the shared object/class relation ABI, inherited public instance methods and runtime string-valued dynamic instance methods reuse declared frames with `$this`, inherited public constructors run after child allocation, and named/object-receiver static calls can resolve inherited public static frames. Source/link/runtime proof covers multi-level inheritance, inherited public property read/write/`empty`, named ancestor `instanceof`, inherited instance and dynamic method calls, named inherited static calls, object static-receiver inherited calls, inherited constructor execution, multiple class families, and unsupported missing/final parent shapes staying blocked.
- `99b0fa3f`: generated-C runtime string-valued public dynamic instance-method calls now materialize the receiver and method expression as native values, compare method names through the shared dynamic-call name ABI, validate receiver class identity through the shared object/class relation ABI, and invoke declared instance-method frames with `$this` plus the existing argument/default/variadic/result cleanup path. Source/link/runtime proof covers multiple receiver classes sharing a method name, multiple method names, default and explicit arguments, ternary method-name expressions, assignment/value/discard consumers, method-body property writes/reads, dynamic miss diagnostics, and unsupported static/non-public method shapes staying blocked.
- `5e5ab57c`: generated-C public object static-receiver calls now evaluate the receiver as a native object value, dispatch over declared public static method candidates by method name, validate receiver class identity through the shared object/class relation ABI, and invoke receiverless declared-method frames with the existing argument/default/variadic/result cleanup path. Source/link proof covers multiple receiver classes sharing a method name, default and explicit arguments, nested object-static call arguments, assignment RHS values, ordinary value consumers, discard statements with method-body output, and non-static object static-receiver shapes staying blocked.
- `de8e9634`: generated-C named public static method calls now resolve declared-class static method metadata, emit receiverless declared-method frames, and reuse the same argument/default/variadic materialization, call-depth/status handling, return ownership, and cleanup machinery used by public instance methods and constructors. Source/link proof covers defaults, explicit arguments, multiple classes, nested static-call arguments, assignment RHS values, and discard statements with method-body output.
- `b099039e`: generated-C named public object-property `unset` routes through the shared object-property ABI, with runtime/source/link proof for direct, chained, missing-property, post-unset `isset`/`empty`, reassignment, visibility diagnostics, and non-object diagnostics.
- `c00780c3`: generated-C `new NamedClass(...)` runs supported public constructors after allocation through the declared-method frame boundary, with `$this` binding and constructor argument/default cleanup proof.
- `a792e8b5`: generated-C public declared instance-method calls dispatch over declared-class method candidates, bind `$this`, reuse argument/default/return cleanup, and report runtime method misses through shared diagnostics.
- `06f699f8`: generated-C named `instanceof` checks consume the shared object/class relation ABI.
- `9d637923`: generated-C public property read/write/`isset`/`empty` consume the shared object-property ABI.
- `07516bc3`: generated-C registers supported top-level classes and allocates no-argument declared objects through the runtime object ABI.
- `f6d9ad0a`: LLVM and generated-C lowerable output-buffer calls consume a shared runtime output-buffer stack ABI.

## Current Review Notes

- Primary semantic baseline is `de7ee6d2`; primary head was clean and synced at `e1ee1314` before this evaluator progress update.
- Focused gates for the latest inheritance slice passed using disk-backed `/tmp` targets: `cargo check -q -p phpc -p php_runtime`, `cargo test -q -p php_runtime native_declared_class_inheritance_allocation_preserves_ancestors_and_slots`, `cargo test -q -p php_runtime native_declared_class`, `cargo test -q -p phpc --test native_link native_executable_c_source_routes_declared_inheritance_through_ancestor_metadata`, `cargo test -q -p phpc --test native_link emit_exe_links_and_runs_declared_class_inheritance_program`, `cargo test -q -p phpc --test native_link declared_class`, `cargo test -q -p phpc --test native_link object_static`, `cargo test -q -p phpc --test native_object_class_boundary`, and `cargo test -q -p phpc --test native_link native_executable_c_source_keeps_unsupported_method_shapes_blocked`.
- The broader `cargo test -q -p phpc --test native_function_call_boundary` gate was not re-run for this inheritance slice; the previous unrelated direct-call column expectation failures in `emit_ir_routes_unsupported_direct_call_argument_results_through_call_boundary` and `native_executable_c_source_routes_unsupported_direct_call_argument_results_through_call_boundary` remain noted until that boundary is refreshed.
- The supervisor dashboard is current as of 2026-05-24 12:50 CEST and matches primary head `e1ee1314` / semantic baseline `de7ee6d2`.
- `/dev/shm` live check: 22G total, 15G used, 7.9G available, 65% used. Use disk-backed `/tmp` targets for broad primary gates, and only reclaim shared-memory targets after owner checks.
- `/home` live check: 459G total, 224G used, 217G available, 51% used. Disk-backed `/tmp/php-to-native-compiler-target*` targets include one large 11G target plus several 256-264M focused targets.
- Lane-local status claims still do not count until selected, gated, committed, and pushed through primary.
