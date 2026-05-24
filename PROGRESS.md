# PHP Native Compiler Progress

Updated: 2026-05-24 09:56 CEST
Evaluation marker: `20260524T072649Z`

Latest primary semantic/test baseline:
`a27bb444 codegen: allow globals self imports in frames`

Latest primary head before this progress update:
`a27bb444 codegen: allow globals self imports in frames`

Only pushed primary work counts here. Dirty WIP, lane-local candidates, parked diffs, exact-shape fixtures, and status-file claims are not counted until selected, gated, committed, and pushed through primary.

## Executive Read

Overall estimated progress: **66%** `[#############-------]`

Executable PHP semantics: **63%** `[#############-------]`

The primary branch has made solid integrated progress on selected native PHP execution islands. Since the last durable progress marker, primary landed leading-numeric arithmetic recovery through the shared native value-operation result ABI, LLVM consumption of the same arithmetic result path, direct-variable assignment expressions through assignment-target semantics, LLVM direct-variable compound assignments through the existing primitive binary lowering path, C assembly fallback consumption of the shared unary string-result and two-operand string-predicate ABIs, and generated-C `$GLOBALS` self-imports in user-function frames through the shared root symbol table.

The product is still not close to "general PHP." The remaining work is concentrated in the real semantic cliffs: full callable lookup/invocation, closures, methods, objects/properties, `$this`, named/unpacked arguments, typed/default/variadic by-reference binding, by-reference returns, reference/COW identity, request/global alias parity, source-ordered diagnostics, cleanup/unwind/finally/destructors/output buffers, and backend parity.

The latest `$GLOBALS` self-import slice improves generated-C request/global alias parity for compiled user-function frames. It does not implement request-superglobal imports, includes, variable variables, exact unset aliasing, or broader call/object/reference/COW cliffs.

## Primary-Integrated Capability

- [x] Shared runtime value, array, string, comparison, truthiness, diagnostic, request, reference, symbol, and call-frame ABI foundations exist for many selected paths.
- [x] Generated-C can execute many focused programs across direct variables, arrays, function frames, dynamic calls, selected globals, by-reference parameters, direct-variable compound assignment, assignment expressions, leading-numeric arithmetic warnings, and bounded finalizer transfers.
- [x] LLVM consumes selected shared ABIs for strings, predicates, searches, integer/string helpers, primitive direct-variable assignment expressions, and value-operation arithmetic that cannot be primitive-folded safely.
- [x] The C assembly fallback consumes shared string-result and string-predicate ABIs for lowerable direct and nested operands.
- [x] LLVM lowers lowerable primitive direct-variable compound-assignment statements and expressions through direct variable storage and existing binary operator semantics.
- [x] Function-scope ordinary `global $name` imports and `$GLOBALS` self-imports work through generated-C frames for direct calls, transitive wrapper calls, and runtime string-valued dynamic calls.
- [x] Direct-variable assignment expressions update lowerable primitive LLVM locals and generated-C ordinary/native-value/reference-backed/active-symbol direct variables.
- [ ] Full PHP callable, object, reference/COW, cleanup/unwind, diagnostic, request/global, include, variable-variable, and backend-parity behavior remains incomplete.

## Current Dirty Primary WIP

- [x] No counted dirty primary WIP at this review point.

## Lane-Local Candidate Work

Recent worker-status updates show active candidate inventory, including interface-member cleanup blockers, static-property storage blockers, callable numeric dispatch through existing numeric ABI, reference-held array key/value operation slots, function-frame contract-result blockers, and expression-result transport for includes, closures, and selection conditions.

These are not product capability yet. Treat them as a queue of possible integration inputs.

## Roadmap Position

| Workstream | Estimate | Bar | Current read |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | **83%** | `[#################---]` | Strong shared surfaces exist, but some are still scaffolding until consumed end to end. |
| Compiler/backend consumers | **83%** | `[#################---]` | Generated-C is broad in selected areas; LLVM covers primitive direct-variable compound assignments, and the C assembly fallback now covers selected string-result and string-predicate ABI consumers; direct assembly and many nested consumers remain blocked. |
| Executable PHP semantics | **63%** | `[#############-------]` | Many focused linked programs run, but behavior is still selected islands. |
| Arrays, lvalues, references, COW | **64%** | `[#############-------]` | Good selected lvalue/reference paths; full COW, arbitrary roots, foreach, object joins, and wider alias composition remain open. |
| Symbols, globals, request state | **72%** | `[##############------]` | Ordinary function-scope globals and `$GLOBALS` self-imports are much better; request superglobal imports, includes, variable variables, and exact unset behavior remain incomplete. |
| Calls, functions, frames | **57%** | `[###########---------]` | Bounded generated-C frames and selected dynamic calls work; closures, methods, callable arrays/objects, named/unpacked args, and by-reference returns remain open. |
| Objects, properties, methods | **10%** | `[##------------------]` | Mostly lane-local/runtime candidate work; primary lacks general compiled object/property/method execution. |
| Control flow, cleanup, diagnostics | **48%** | `[##########----------]` | Selected branches, loops, transfers, finalizers, and diagnostics exist; broad unwind, handlers, destructors, buffers, and exact ordering remain open. |
| Broad integrated verification | **56%** | `[###########---------]` | Focused gates are strong, now including LLVM direct-variable compound-assignment arithmetic/bitwise/shift/modulo/expression proof; cross-feature linked programs, backend parity, and full end-to-end PHP proof lag. |

## Done / In Progress / Not Done

- [x] Bounded generated-C direct variables, array/lvalue paths, selected dynamic calls, function globals, `$GLOBALS` self-imports, by-reference parameter writes, assignment expressions, direct-variable compound assignment, and finalizer transfer slices.
- [x] LLVM value-operation arithmetic, primitive direct-variable assignment-expression, primitive direct-variable compound-assignment, and C assembly fallback string-result/string-predicate slices.
- [ ] In progress: lane-local candidates for callable dispatch, object/property metadata, frame contracts, symbol/reference transport, diagnostics, and cleanup boundaries.
- [ ] Not done: general object model, methods, `$this`, closures, callable-array/object invocation, complete references/COW, by-reference returns, complete mutation/unset, full diagnostics, full cleanup/unwind, includes, variable variables, request/global parity, and direct assembly parity.

## Recent Primary-Integrated Work

- `a27bb444`: generated-C user-function frames allow `global $GLOBALS` as the PHP self-import case. Direct and runtime string-valued dynamic calls pass the caller root symbol table into those frames, `$GLOBALS[...]` reads and writes stay on the shared symbol-path ABI, and request superglobal imports remain blocked until request state is threaded through frames.
- `b1f3c546`: C assembly fallback lowering now routes lowerable `str_starts_with(...)`, `str_contains(...)`, and `str_ends_with(...)` calls through `phpc_native_value_string_predicate_with_diagnostic(...)`, including nested lowerable value-result operands and arity blockers.
- `91bc2f4a`: C assembly fallback lowering now routes lowerable unary string-result builtins through `phpc_native_value_string_result_operation_with_diagnostic(...)` for direct and nested operands, preserving owned result cleanup and arity blockers. This covers the existing string-result runtime/compiler family, not broader direct assembly parity.
- `b3625c8a`: LLVM direct-variable compound assignments now lower through `AssignTarget::Variable` storage and existing binary operator semantics for lowerable primitive arithmetic, bitwise, shift, modulo, and expression-result forms. Undefined direct variables, native-owned result storage, non-direct lvalues, request/global roots, object/static properties, `??=`, increment/decrement, unset, references/COW, exact mutation diagnostics, and direct assembly parity remain blocked.
- `ea6ebcf2`: direct-variable assignment expressions lower through assignment-target semantics. LLVM supports lowerable primitive direct variables and chained direct assignment expressions. Generated-C supports ordinary scalar, native-value result, reference-backed, and active symbol-table direct-variable owners.
- `c7e35c50`: LLVM routes scalar/native-value arithmetic that cannot be primitive-folded safely through the shared native value-operation result ABI, including diagnostics and owned result handling.
- `40838cef`: native value binary/unary arithmetic result ABIs recover leading-numeric string operands and carry warning diagnostics through generated-C consumers.
- `84d33e6f`, `f0b22da2`, `482e7c76`: generated-C function-scope ordinary globals now compose through direct frames, transitive wrapper frames, and runtime string-valued dynamic calls.
- `e9ba63d0`: generated-C `break`/`continue` through supported active `finally` scopes execute the finalizers they leave.
- `a7ffdcd2`: generated-C direct-variable compound assignment writes back through ordinary local, reference-backed, and active symbol-table owners.

## Current Review Notes

- Primary repo had just landed `b1f3c546` when this dashboard was refreshed.
- `/dev/shm` at review was 22G total, 15G used, 7.6G available. This is above the 6G floor but tight for broad concurrent link/test waves.
- The largest visible target was `/dev/shm/phpc-target-native-object-property-runtime` at about 8.8G. Reclaim only after owner checks.
- Dashboard evidence is stale relative to current primary and worker status files; live state should be preferred for steering.
