# PHP Native Compiler Progress

Updated: 2026-05-24 12:00 CEST
Evaluation marker: `20260524T100021Z`

Latest counted primary semantic/test baseline:
`b099039e codegen: route declared object property unset`

Latest primary head before this progress update:
`bc43f7c0 docs: update progress after object property unset`

Only pushed primary work counts here. Dirty WIP, lane-local candidates, parked diffs, exact-shape fixtures, and status-file claims are not product capability until selected, gated, committed, and pushed through primary.

## Executive Read

Overall estimated progress: **70%** `[##############------]`

Executable PHP semantics: **67%** `[#############-------]`

Primary is making real integrated progress in selected native PHP execution islands. The strongest recent momentum is the generated-C object path: declared class allocation, public property read/write/`isset`/`empty`/`unset`, named `instanceof`, public instance methods with `$this`, and supported public constructors now share runtime/frame/property ABIs and have focused source/link proof.

This is not general PHP yet. The remaining cliffs are still large: full callable lookup/invocation, closures, dynamic/static/non-public methods, inheritance/interfaces/traits, contextual `self`/`parent`/`static`, dynamic/static properties, magic methods, references/COW identity, by-reference returns, request/global alias parity, includes, variable variables, exact/source-ordered diagnostics, cleanup/unwind/finally/destructors/output-buffer shutdown, and backend parity.

Current live primary has uncommitted `compiler/src/codegen.rs`, `compiler/tests/native_link.rs`, and `compiler/tests/native_function_call_boundary.rs` WIP that appears to target declared static method calls. It is **not counted** here.

## Primary-Integrated Capability

- [x] Runtime/value foundations: shared native value, array, string, comparison, truthiness, diagnostic, request, reference, symbol, call-frame, output-buffer, object, object-property, and method-diagnostic ABI surfaces exist for selected paths.
- [x] Generated-C execution: direct variables, arrays/lvalues, selected dynamic calls, function frames, function-scope globals, `$GLOBALS` self-imports, by-reference parameter writes, assignment expressions, direct-variable compound assignments, output buffers, finalizer transfer slices, and the bounded declared-object family are executable.
- [x] LLVM consumers: selected string, predicate, search, integer/string helper, primitive assignment-expression, primitive compound-assignment, output-buffer, and native value-operation arithmetic paths consume shared ABIs.
- [x] C assembly fallback: selected unary string-result and two-operand string-predicate helpers consume shared runtime ABIs.
- [x] Object slice: generated-C can allocate supported declared class objects, preserve type/class identity, execute public properties including `unset`, evaluate named `instanceof`, call supported public instance methods with `$this`, and run supported public constructors.
- [ ] Not complete: full PHP callable, object, reference/COW, cleanup/unwind, diagnostic, request/global, include, variable-variable, and backend-parity behavior.

## Current Primary State

- [x] Primary `master` and `origin/master` were live-checked synced at `bc43f7c0`.
- [x] Latest counted semantic commit is `b099039e`.
- [ ] Uncounted dirty WIP exists in `compiler/src/codegen.rs`, `compiler/tests/native_link.rs`, and `compiler/tests/native_function_call_boundary.rs` with a declared static-method-call direction. It must remain uncounted until gated, committed, and pushed.
- [ ] After this review, `PROGRESS.md` is expected to be dirty for the wrapper commit; unrelated product diffs should remain untouched.

## Lane-Local Candidate Work

Fresh worker statuses show broad candidate inventory, including static-property blockers, contextual `self`/`parent`/`static` receiver carriers, callable result egress dispatch, non-local symbol candidate sequencing, request/global diagnostics, parameter-aware conversion diagnostics, reference/COW metadata, and cleanup/CFG transfer planning.

These are useful inputs, not product capability. Prefer candidates that land as primary source/link proof or remove a real shared execution blocker. Treat blocker-only or metadata-only slices as lower priority unless they unlock an immediate executable consumer.

## Roadmap Position

| Workstream | Estimate | Bar | Current read |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | **88%** | `[##################--]` | Strong shared surfaces exist, now consumed by output buffers and the declared-object/property/method family; some surfaces remain scaffolding until broader consumers land. |
| Compiler/backend consumers | **88%** | `[##################--]` | Generated-C is broad in selected areas; LLVM and C assembly consume selected ABI families. Direct assembly and many nested/object consumers remain blocked. |
| Executable PHP semantics | **67%** | `[#############-------]` | Many focused linked programs run, but semantics are still selected islands rather than general PHP. |
| Arrays, lvalues, references, COW | **64%** | `[#############-------]` | Good selected lvalue/reference paths; full COW, arbitrary roots, foreach, object joins, and alias composition remain open. |
| Symbols, globals, request state | **72%** | `[##############------]` | Function globals and `$GLOBALS` self-imports improved; request superglobal imports, includes, variable variables, and exact unset behavior remain incomplete. |
| Calls, functions, frames | **59%** | `[############--------]` | Bounded functions, public instance methods, and constructors work for selected generated-C paths; closures, callable arrays/objects, named/unpacked args, static methods, and by-reference returns remain open. |
| Objects, properties, methods | **33%** | `[#######-------------]` | Primary now has a useful declared-object subset; dynamic/static/non-public methods, inheritance/interfaces, magic methods, dynamic/static properties, visibility contexts, references/COW, and backend parity remain open. |
| Control flow, cleanup, diagnostics | **49%** | `[##########----------]` | Selected branches, loops, transfers, finalizers, output buffers, and diagnostics exist; broad unwind, handlers, destructors, shutdown flushing, and exact ordering remain open. |
| Broad integrated verification | **63%** | `[#############-------]` | Focused gates are strong; cross-feature linked programs, backend parity, and full end-to-end PHP proof lag. |

## Done / In Progress / Not Done

- [x] Done in primary: bounded generated-C direct variables, arrays/lvalues, selected dynamic calls, function globals, `$GLOBALS` self-imports, by-reference parameter writes, assignment expressions, direct-variable compound assignments, output buffers, finalizer transfer slices, declared-class allocation, public constructors, public instance methods, public properties including unset, and named `instanceof`.
- [x] Done in primary: LLVM value-operation arithmetic, primitive direct-variable assignment-expression and compound-assignment paths, lowerable output-buffer calls, and C assembly fallback string-result/string-predicate slices.
- [ ] In progress but uncounted: dirty primary static-method-call WIP plus lane-local candidates for callable dispatch, object/static property metadata, frame contracts, symbol/reference transport, diagnostics, and cleanup boundaries.
- [ ] Not done: general object model, dynamic/static/non-public methods, general constructor semantics, contextual class names, closures, callable-array/object invocation, complete references/COW, by-reference returns, complete mutation/unset, full diagnostics, full cleanup/unwind, includes, variable variables, request/global parity, and direct assembly parity.

## Recent Primary-Integrated Work

- `b099039e`: generated-C named public object-property `unset` routes through the shared object-property ABI, with runtime/source/link proof for direct, chained, missing-property, post-unset `isset`/`empty`, reassignment, visibility diagnostics, and non-object diagnostics.
- `c00780c3`: generated-C `new NamedClass(...)` runs supported public constructors after allocation through the declared-method frame boundary, with `$this` binding and constructor argument/default cleanup proof.
- `a792e8b5`: generated-C public declared instance-method calls dispatch over declared-class method candidates, bind `$this`, reuse argument/default/return cleanup, and report runtime method misses through shared diagnostics.
- `06f699f8`: generated-C named `instanceof` checks consume the shared object/class relation ABI.
- `9d637923`: generated-C public property read/write/`isset`/`empty` consume the shared object-property ABI.
- `07516bc3`: generated-C registers supported top-level classes and allocates no-argument declared objects through the runtime object ABI.
- `f6d9ad0a`: LLVM and generated-C lowerable output-buffer calls consume a shared runtime output-buffer stack ABI.

## Current Review Notes

- The supervisor dashboard tail is stale relative to current primary; it still centers on the earlier formatter-diagnostics era and should be refreshed before strategic steering relies on it.
- `/dev/shm` live check: 22G total, 15G used, 7.4G available, 67% used. Use disk-backed `/tmp` targets for broad gates if it approaches the 6G floor, and only reclaim shared-memory targets after owner checks.
- `/home` live check: 459G total, 210G used, 231G available, 48% used. `du -sh /home` reported about 110G but exited nonzero with stderr suppressed.
- Do not raise percentages for the current dirty static-method WIP or lane-local status claims until primary lands gated source/link proof.
