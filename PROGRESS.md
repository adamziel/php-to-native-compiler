# PHP Native Compiler Progress

Updated: 2026-05-24 13:40 CEST
Evaluation marker: `20260524T114058Z`

Latest counted primary semantic/test baseline:
`e2d20f3 codegen: lower dynamic declared class new`

Latest primary head before this progress update:
`f843578e docs: update progress after dynamic declared new`

Only pushed primary work counts here. Dirty WIP, lane-local candidates, parked diffs, exact-shape fixtures, and status-file claims are not product capability until selected, gated, committed, and pushed through primary.

## Executive Read

Overall estimated progress: **72%** `[##############------]`

Executable PHP semantics: **71%** `[##############------]`

Primary is making real integrated progress in selected generated-C execution islands. The current strongest area is declared objects/classes: supported declared allocation, ancestor-aware child allocation, inherited public properties, named `instanceof`, public instance methods, runtime string-valued public dynamic instance methods, supported public/inherited constructors for named `new`, public static methods, object static-receiver methods, and runtime string-valued constructorless declared-class `new`.

This is still not general PHP. The remaining cliffs are large: full callable lookup/invocation, closures, callable arrays/objects, non-public methods, override compatibility, interfaces/traits, contextual `self`/`parent`/`static`, magic methods, dynamic/static properties, references/COW identity, by-reference returns, request/global alias parity, includes, variable variables, exact/source-ordered diagnostics, cleanup/unwind/finally/destructors/output-buffer shutdown, and backend parity.

Live review note: the primary worktree contains uncommitted implementation WIP in `compiler/src/codegen.rs`, `compiler/tests/native_function_call_boundary.rs`, and `compiler/tests/native_link.rs`. It appears related to dynamic declared-class `new` with public constructor dispatch. It is not counted in these percentages.

## Primary-Integrated Capability

- [x] Runtime/value foundations: shared native value, array, string, comparison, truthiness, diagnostic, request, reference, symbol, call-frame, output-buffer, object, object-property, and method-diagnostic ABI surfaces exist for selected paths.
- [x] Generated-C execution: direct variables, arrays/lvalues, selected dynamic calls, function frames, function-scope globals, `$GLOBALS` self-imports, by-reference parameter writes, assignment expressions, direct-variable compound assignments, output buffers, finalizer transfer slices, runtime string-valued constructorless declared-class `new`, and the bounded declared-object family are executable.
- [x] LLVM consumers: selected string, predicate, search, integer/string helper, primitive assignment-expression, primitive compound-assignment, output-buffer, and native value-operation arithmetic paths consume shared ABIs.
- [x] C assembly fallback: selected unary string-result and two-operand string-predicate helpers consume shared runtime ABIs.
- [x] Object slice: generated-C can allocate supported declared class objects and declared child objects with ancestor metadata, instantiate constructorless declared classes through runtime string-valued class names, preserve type/class identity across supported ancestor chains, execute public properties including inherited public slots and `unset`, evaluate named `instanceof`, call supported public/inherited instance methods with `$this`, call runtime string-valued public dynamic instance methods, run supported public/inherited constructors for named `new`, call supported named public static methods, and call supported public static methods through object receivers.
- [ ] Not complete: full PHP callable, object, reference/COW, cleanup/unwind, diagnostic, request/global, include, variable-variable, and backend-parity behavior.

## Current Primary State

- [x] Primary semantic baseline is committed as `e2d20f3`.
- [x] Primary docs head is `f843578e` and was reported synced with `origin/master`.
- [x] Latest counted semantic work is generated-C runtime string-valued constructorless declared-class `new`.
- [ ] Clean primary worktree before this evaluator update: live check showed uncommitted implementation WIP in three product files. This is uncounted and should be handled only by the primary integrator.
- [x] This evaluator progress update should be committed by the wrapper as a docs-only `PROGRESS.md` update; unrelated product diffs should remain untouched.

## Lane-Local Candidate Work

Most relevant candidate now: `candidate-dynamic-constructor-new` reports a compact generated-C slice for `new $class(...)` over generated declared classes requiring supported public constructor dispatch. It reuses existing declared allocation helpers, constructor frame dispatch, `$this` binding, argument/default cleanup, and inherited constructor lookup. It is ready for primary-integrator review but is not counted until committed and pushed through primary.

Historical lane: `candidate-object-instantiation` produced the constructorless dynamic declared-class `new` slice that landed as `e2d20f3`. Do not repeat it.

Other active lanes contain useful but uncounted material around closure descriptor invocation, function-frame result-vector binding, ArrayAccess method-policy slots, readonly property metadata, request/global symbol operations, object/reference cleanup classification, reference/COW live-generator conformance, and diagnostic scanner boundaries. Prefer candidates that land as primary source/link proof or remove a real shared execution blocker.

## Roadmap Position

| Workstream | Estimate | Bar | Current read |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | **90%** | `[##################--]` | Strong shared surfaces exist and are consumed by output buffers and the declared-object/property/method family; some newer surfaces remain scaffolding until executable consumers land. |
| Compiler/backend consumers | **92%** | `[##################--]` | Generated-C is broad in selected areas; LLVM and C assembly consume selected ABI families. Direct assembly, LLVM object lowering, and many nested/object consumers remain blocked. |
| Executable PHP semantics | **71%** | `[##############------]` | Many focused linked programs run, including constructorless dynamic declared-class `new` and inherited public object/method/static/constructor paths, but execution is still selected islands rather than general PHP. |
| Arrays, lvalues, references, COW | **64%** | `[#############-------]` | Good selected lvalue/reference paths; full COW, arbitrary roots, foreach, object joins, and alias composition remain open. |
| Symbols, globals, request state | **72%** | `[##############------]` | Function globals and `$GLOBALS` self-imports improved; request superglobal imports, includes, variable variables, and exact unset behavior remain incomplete. |
| Calls, functions, frames | **65%** | `[#############-------]` | Bounded functions and public declared object/method/constructor call paths work in selected generated-C cases; closures, callable arrays/objects, named/unpacked args, dynamic constructor dispatch, non-public contexts, and by-reference returns remain open. |
| Objects, properties, methods | **42%** | `[########------------]` | Useful declared-object subset exists, including runtime string-valued constructorless `new`; non-public methods, overrides, interfaces/traits, magic methods, dynamic/static properties, visibility contexts, references/COW, and backend parity remain open. |
| Control flow, cleanup, diagnostics | **49%** | `[##########----------]` | Selected branches, loops, transfers, finalizers, output buffers, and diagnostics exist; broad unwind, handlers, destructors, shutdown flushing, and exact ordering remain open. |
| Broad integrated verification | **68%** | `[##############------]` | Focused gates are strong for recent slices; cross-feature linked programs, backend parity, and full end-to-end PHP proof lag. |

## Done / In Progress / Not Done

- [x] Done in primary: bounded generated-C direct variables, arrays/lvalues, selected dynamic calls, function globals, `$GLOBALS` self-imports, by-reference parameter writes, assignment expressions, direct-variable compound assignments, output buffers, finalizer transfer slices, declared-class allocation including runtime string-valued constructorless declared-class `new`, supported child allocation with ancestor metadata, constructorless argument evaluation, public/inherited public constructors for named `new`, public/inherited public instance/static methods, runtime string-valued public dynamic instance methods, public object static-receiver methods, public properties including inherited public slots and unset, and named `instanceof`.
- [x] Done in primary: LLVM value-operation arithmetic, primitive direct-variable assignment-expression and compound-assignment paths, lowerable output-buffer calls, and C assembly fallback string-result/string-predicate slices.
- [ ] In progress but uncounted: dirty primary/candidate work for dynamic declared-class `new` with public constructor dispatch; lane-local candidates for closure/callable dispatch, object/static property metadata, frame contracts, symbol/reference transport, diagnostics, and cleanup boundaries.
- [ ] Not done: general object model, arbitrary dynamic class-name expressions, contextual class names, non-public methods, overrides, interfaces/traits, magic methods, closures, callable-array/object invocation, complete references/COW, by-reference returns, complete mutation/unset, full diagnostics, full cleanup/unwind, includes, variable variables, request/global parity, and direct assembly parity.

## Recent Primary-Integrated Work

- `e2d20f3`: generated-C `new $class(...)` supports runtime string-valued direct-variable class names for generated constructorless declared classes. Dynamic class names are materialized as native values, matched case-insensitively against registered declared-class candidates through the shared dynamic-name helper, and allocated through existing declared-class or ancestor-aware helpers. Constructorless named and dynamic `new` now evaluate and clean up argument expressions. Source/link proof covers named constructorless arguments, case-insensitive runtime class names, multiple declared classes, declared child allocation through ancestor metadata, inherited public property writes/reads, named ancestor `instanceof`, `is_object()`, `gettype()`, `get_debug_type()`, and dynamic class names requiring constructor dispatch staying blocked.
- `de7ee6d2`: generated-C declared child objects allocate through ancestor-aware metadata, preserving inherited public property/method/static/constructor behavior and ancestor relation checks.
- `99b0fa3f`: generated-C runtime string-valued public dynamic instance-method calls dispatch through shared dynamic-call name matching and object/class relation checks.
- `5e5ab57c`: generated-C public object static-receiver calls dispatch across declared public static method candidates through shared object/class relation checks.
- `de8e9634`: generated-C named public static method calls execute through receiverless declared-method frames.
- `b099039e`, `c00780c3`, `a792e8b5`, `06f699f8`, `9d637923`, `07516bc3`, and `f6d9ad0a`: earlier object/property/method/constructor/instanceof/allocation/output-buffer foundations remain part of the counted primary baseline.

## Current Review Notes

- Primary semantic baseline remains `e2d20f3`; docs head before this evaluator update is `f843578e`.
- Current percentages stay flat because no new pushed primary semantic commit has landed since the last progress update.
- The live primary worktree has uncommitted implementation diffs; these are deliberately excluded from progress accounting.
- `candidate-dynamic-constructor-new` is the most review-ready lane-local candidate, but its capability should be counted only after primary integration and focused gates.
- Live `/dev/shm` check: 22G total, 16G used, 6.6G available, 71% used. Use disk-backed `/tmp` targets for broad gates and owner-check before cleaning shared-memory targets.
- Live `/home` check: 459G total, 245G used, 196G available, 56% used.
