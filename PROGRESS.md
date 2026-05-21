# PHP to Native Compiler Progress

Updated: 2026-05-21 14:27 CEST  
Primary state: `master` clean and pushed at `f17b1352 native: route lvalue operand call results`  
Progress estimate: **23% toward broad, generalized native PHP semantics**

This percentage is a steering estimate, not a release metric. It weights merged primary capability higher than lane-local work. Lane-local patches are counted as promising candidates only after they are merged, tested, committed, and pushed to this repo.

## Executive View

The project is moving in the right direction: recent work is replacing one-off backend rejections with shared semantic-family contracts for calls, comparison results, symbol tables, diagnostics, arrays, object properties, references, and ownership cleanup. The most important primary wins are now executable comparison-branch ABI consumption and broader call-result operand routing through shared cleanup/blocker boundaries.

The candid caveat: much of the current progress is still boundary and ABI infrastructure. It makes unsupported PHP surfaces fail at the right shared semantic boundary and prepares codegen/runtime consumers, but it is not yet full PHP execution for calls, arrays, references, objects, closures, methods, or diagnostics.

## Roadmap Snapshot

```
Overall generalized PHP semantics       [#####---------------] 23%
Runtime value/ABI foundation            [########------------] 42%
Compiler/backend ABI consumption        [######--------------] 30%
Function calls and frames               [####----------------] 20%
Arrays, lvalues, references, COW        [####----------------] 20%
Objects and properties                  [###-----------------] 15%
Diagnostics/control-flow cleanup        [####----------------] 20%
Broad PHP compatibility surface         [#-------------------]  6%
```

## Recently Integrated In Primary

- [x] Runtime symbol table ABI seed and helper convergence.
- [x] Shared scalar string boundary in runtime.
- [x] Centralized native call operation blockers in compiler codegen.
- [x] Runtime comparison result ABI.
- [x] Diagnostic severity tagging ABI.
- [x] Constructor and closure call expressions routed through the shared call boundary.
- [x] Call-result lvalue targets routed through shared call-operation ownership blockers.
- [x] Executable native C scalar comparisons now consume the comparison branch ABI.
- [x] Direct call argument results routed through shared cleanup boundaries.
- [x] Constant-table call argument results routed through shared cleanup boundaries.
- [x] Unsupported direct builtin argument results routed through shared cleanup boundaries.
- [x] Lvalue operand call results routed through shared call-result cleanup/blocker paths.

## In Progress

- [~] **Call semantics** `[####----------------] 20%`
  - Done: shared call-operation blockers, call-result lvalue routing, argument-result cleanup preflight, and lvalue operand call-result routing across multiple direct-call families.
  - Missing: real callable lookup, frames, argument binding, by-reference args, variadics, named/default/spread args, callbacks, recursion, exact call diagnostics.

- [~] **Comparison semantics** `[########------------] 40%`
  - Done: runtime comparison result ABI and primary executable branch consumer for scalar comparisons.
  - Missing: full PHP loose/strict comparison parity across arrays, objects, resources, references, conversion ordering, diagnostics, and backend parity beyond the currently integrated slice.

- [~] **Arrays and lvalues** `[####----------------] 20%`
  - Done: broad lane-local contracts for array lvalue owners, read-modify-write, `??=`, `isset`, `empty`, generated-C linkage, and shared owner/result blockers.
  - Missing: primary integration of most lane work, dynamic string offsets, ArrayAccess, foreach mutation, COW/reference separation, exact key diagnostics, arbitrary native-value propagation.

- [~] **Symbol environments** `[#####---------------] 25%`
  - Done: runtime symbol table ABI and lane-local symbol/root work.
  - Missing: generalized local/global/frame symbol flow through generated code, request/superglobal semantics, unset effects, conditional state merging.

- [~] **References and owner cells** `[###-----------------] 18%`
  - Done: lane-local owner-cell handle/component/result/borrow scaffolding with focused runtime/compiler ABI tests.
  - Missing: production codegen emission of borrow/result branches, alias visibility, COW detach/write barriers, by-reference call outputs, cleanup dominance.

- [~] **Objects and properties** `[###-----------------] 15%`
  - Done: bounded object/property blockers and lane-local property operation result surfaces.
  - Missing: method dispatch, visibility, declared/static properties, dynamic property policy, magic hooks, object allocation, ArrayAccess, property references/COW, exact diagnostics.

- [~] **Diagnostics and control flow** `[####----------------] 20%`
  - Done: diagnostic severity ABI and many centralized blocker families; lane-local cleanup/control-flow effect tracking.
  - Missing: exact PHP diagnostic text/severity/recovery ordering, exception/termination unwinding, full cleanup-stack merge semantics.

## Not Yet Done

- [ ] Full PHP function/method/constructor execution.
- [ ] Closures, captures, dynamic callable dispatch, callbacks.
- [ ] Named/default/spread arguments, variadics, by-reference parameters and returns.
- [ ] Generalized arrays with ArrayAccess, references, COW, foreach mutation, append holes, exact key conversion diagnostics.
- [ ] Object allocation, class metadata, static members, visibility, magic methods/properties.
- [ ] Request state, superglobals, include/require, resources, streams, sessions, headers.
- [ ] Exact PHP diagnostics, warning/recovery order, fatal/exception/termination behavior.
- [ ] Broad generated-C and LLVM parity across all supported semantic families.
- [ ] Large compatibility test corpus proving real PHP programs, not only focused boundary slices.

## Current Steering

Prioritize small primary commits that connect landed ABI/result surfaces to real compiler, generated-C, LLVM, or runtime consumers with composition tests. Deprioritize standalone blocker vocabulary unless it immediately unlocks an executable generalized consumer.

Best next integration candidates:

- comparison ABI consumed by more branch/control-flow surfaces;
- call-frame argument/result ownership where lane work is already mature;
- symbol-table variable flow in linked native execution;
- array lvalue/value ownership where generated-C linkage is already tested lane-locally;
- owner-cell/reference surfaces only when production codegen can consume the ABI, not just declare it.

## Reading The Percentages

- **0-10%**: mostly design, blockers, or isolated probes.
- **10-30%**: shared semantic contracts exist; some focused tests; limited primary consumers.
- **30-60%**: real executable primary consumers across multiple PHP surfaces.
- **60-85%**: broad feature-family behavior with cleanup, diagnostics, and backend parity.
- **85-100%**: PHP-compatible edge cases and corpus-level confidence.
