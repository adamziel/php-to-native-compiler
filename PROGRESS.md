# PHP Native Compiler Progress

Updated: 2026-05-24 12:07 CEST
Evaluation marker: `20260524T100021Z`

Latest counted primary semantic/test baseline:
`de8e9634 codegen: lower declared static methods`

Latest primary head before this progress update:
`de8e9634 codegen: lower declared static methods`

Only pushed primary work counts here. Dirty WIP, lane-local candidates, parked diffs, exact-shape fixtures, and status-file claims are not product capability until selected, gated, committed, and pushed through primary.

## Executive Read

Overall estimated progress: **70%** `[##############------]`

Executable PHP semantics: **68%** `[##############------]`

Primary is making real integrated progress in selected native PHP execution islands. The strongest recent momentum is the generated-C object path: declared class allocation, public property read/write/`isset`/`empty`/`unset`, named `instanceof`, public instance methods with `$this`, supported public constructors, and named public static methods now share runtime/frame/property or declared-method frame boundaries and have focused source/link proof.

This is not general PHP yet. The remaining cliffs are still large: full callable lookup/invocation, closures, dynamic/non-public methods, object static-receiver calls, inheritance/interfaces/traits, contextual `self`/`parent`/`static`, dynamic/static properties, magic methods, references/COW identity, by-reference returns, request/global alias parity, includes, variable variables, exact/source-ordered diagnostics, cleanup/unwind/finally/destructors/output-buffer shutdown, and backend parity.

The latest method slice extends generated-C declared method frames to named public static methods. `ClassName::method(...)` resolves against declared-class static method metadata, emits receiverless frames, reuses the same argument/default/variadic materialization and result cleanup path as instance methods and constructors, and composes as a value expression, nested argument, assignment RHS, and discard statement. Dynamic method names, object static-receiver calls, `self::`/`parent::`/`static::`, non-public static visibility contexts, inheritance/autoload/late-static binding, exact diagnostics, LLVM/direct assembly object lowering, and broad object/reference/COW semantics remain blocked.

## Primary-Integrated Capability

- [x] Runtime/value foundations: shared native value, array, string, comparison, truthiness, diagnostic, request, reference, symbol, call-frame, output-buffer, object, object-property, and method-diagnostic ABI surfaces exist for selected paths.
- [x] Generated-C execution: direct variables, arrays/lvalues, selected dynamic calls, function frames, function-scope globals, `$GLOBALS` self-imports, by-reference parameter writes, assignment expressions, direct-variable compound assignments, output buffers, finalizer transfer slices, and the bounded declared-object family are executable.
- [x] LLVM consumers: selected string, predicate, search, integer/string helper, primitive assignment-expression, primitive compound-assignment, output-buffer, and native value-operation arithmetic paths consume shared ABIs.
- [x] C assembly fallback: selected unary string-result and two-operand string-predicate helpers consume shared runtime ABIs.
- [x] Object slice: generated-C can allocate supported declared class objects, preserve type/class identity, execute public properties including `unset`, evaluate named `instanceof`, call supported public instance methods with `$this`, run supported public constructors, and call supported named public static methods without `$this`.
- [ ] Not complete: full PHP callable, object, reference/COW, cleanup/unwind, diagnostic, request/global, include, variable-variable, and backend-parity behavior.

## Current Primary State

- [x] Primary semantic work is committed locally as `de8e9634`.
- [x] Latest counted semantic commit is `de8e9634`.
- [x] No uncounted dirty primary WIP remains from the declared static-method-call slice before this progress update.
- [ ] After this review, `PROGRESS.md` is expected to be dirty for the wrapper commit; unrelated product diffs should remain untouched.

## Lane-Local Candidate Work

Fresh worker statuses show broad candidate inventory, including static-property blockers, contextual `self`/`parent`/`static` receiver carriers, callable result egress dispatch, non-local symbol candidate sequencing, request/global diagnostics, parameter-aware conversion diagnostics, reference/COW metadata, and cleanup/CFG transfer planning.

These are useful inputs, not product capability. Prefer candidates that land as primary source/link proof or remove a real shared execution blocker. Treat blocker-only or metadata-only slices as lower priority unless they unlock an immediate executable consumer.

## Roadmap Position

| Workstream | Estimate | Bar | Current read |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | **88%** | `[##################--]` | Strong shared surfaces exist, now consumed by output buffers and the declared-object/property/method family; some surfaces remain scaffolding until broader consumers land. |
| Compiler/backend consumers | **89%** | `[##################--]` | Generated-C is broad in selected areas, now including named public static declared methods; LLVM and C assembly consume selected ABI families. Direct assembly and many nested/object consumers remain blocked. |
| Executable PHP semantics | **68%** | `[##############------]` | Many focused linked programs run, now including named public static methods, but semantics are still selected islands rather than general PHP. |
| Arrays, lvalues, references, COW | **64%** | `[#############-------]` | Good selected lvalue/reference paths; full COW, arbitrary roots, foreach, object joins, and alias composition remain open. |
| Symbols, globals, request state | **72%** | `[##############------]` | Function globals and `$GLOBALS` self-imports improved; request superglobal imports, includes, variable variables, and exact unset behavior remain incomplete. |
| Calls, functions, frames | **60%** | `[############--------]` | Bounded functions, public instance methods, constructors, and named public static methods work for selected generated-C paths; closures, callable arrays/objects, named/unpacked args, object static-receiver calls, and by-reference returns remain open. |
| Objects, properties, methods | **35%** | `[#######-------------]` | Primary now has a useful declared-object subset with public instance/static method execution; dynamic/non-public methods, object static receivers, inheritance/interfaces, magic methods, dynamic/static properties, visibility contexts, references/COW, and backend parity remain open. |
| Control flow, cleanup, diagnostics | **49%** | `[##########----------]` | Selected branches, loops, transfers, finalizers, output buffers, and diagnostics exist; broad unwind, handlers, destructors, shutdown flushing, and exact ordering remain open. |
| Broad integrated verification | **64%** | `[#############-------]` | Focused gates are strong, including source/link proof for named public static methods; cross-feature linked programs, backend parity, and full end-to-end PHP proof lag. |

## Done / In Progress / Not Done

- [x] Done in primary: bounded generated-C direct variables, arrays/lvalues, selected dynamic calls, function globals, `$GLOBALS` self-imports, by-reference parameter writes, assignment expressions, direct-variable compound assignments, output buffers, finalizer transfer slices, declared-class allocation, public constructors, public instance/static methods, public properties including unset, and named `instanceof`.
- [x] Done in primary: LLVM value-operation arithmetic, primitive direct-variable assignment-expression and compound-assignment paths, lowerable output-buffer calls, and C assembly fallback string-result/string-predicate slices.
- [ ] In progress but uncounted: lane-local candidates for callable dispatch, object/static property metadata, frame contracts, symbol/reference transport, diagnostics, and cleanup boundaries.
- [ ] Not done: general object model, dynamic/non-public methods, object static-receiver calls, general constructor semantics, contextual class names, closures, callable-array/object invocation, complete references/COW, by-reference returns, complete mutation/unset, full diagnostics, full cleanup/unwind, includes, variable variables, request/global parity, and direct assembly parity.

## Recent Primary-Integrated Work

- `de8e9634`: generated-C named public static method calls now resolve declared-class static method metadata, emit receiverless declared-method frames, and reuse the same argument/default/variadic materialization, call-depth/status handling, return ownership, and cleanup machinery used by public instance methods and constructors. Source/link proof covers defaults, explicit arguments, multiple classes, nested static-call arguments, assignment RHS values, discard statements with method-body output, and unsupported object static-receiver shapes staying blocked.
- `b099039e`: generated-C named public object-property `unset` routes through the shared object-property ABI, with runtime/source/link proof for direct, chained, missing-property, post-unset `isset`/`empty`, reassignment, visibility diagnostics, and non-object diagnostics.
- `c00780c3`: generated-C `new NamedClass(...)` runs supported public constructors after allocation through the declared-method frame boundary, with `$this` binding and constructor argument/default cleanup proof.
- `a792e8b5`: generated-C public declared instance-method calls dispatch over declared-class method candidates, bind `$this`, reuse argument/default/return cleanup, and report runtime method misses through shared diagnostics.
- `06f699f8`: generated-C named `instanceof` checks consume the shared object/class relation ABI.
- `9d637923`: generated-C public property read/write/`isset`/`empty` consume the shared object-property ABI.
- `07516bc3`: generated-C registers supported top-level classes and allocates no-argument declared objects through the runtime object ABI.
- `f6d9ad0a`: LLVM and generated-C lowerable output-buffer calls consume a shared runtime output-buffer stack ABI.

## Current Review Notes

- Primary semantic baseline is `de8e9634`; the progress wrapper should be the only dirty primary file before the docs commit.
- The supervisor dashboard tail is stale relative to current primary; it still centers on the earlier formatter-diagnostics era and should be refreshed before strategic steering relies on it.
- `/dev/shm` live check: 22G total, 15G used, 7.4G available, 67% used. Use disk-backed `/tmp` targets for broad gates if it approaches the 6G floor, and only reclaim shared-memory targets after owner checks.
- `/home` live check: 459G total, 210G used, 231G available, 48% used. `du -sh /home` reported about 110G but exited nonzero with stderr suppressed.
- Lane-local status claims still do not count until selected, gated, committed, and pushed through primary.
