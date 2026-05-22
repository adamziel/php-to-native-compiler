# PHP Native Compiler Progress

Updated: 2026-05-22 06:08 CEST
Evaluation marker: 20260522T040802Z-primary-e3af4261

This is a high-level supervisor dashboard. Percentages are candid engineering estimates, not test-suite pass rates. Primary-integrated capability means committed on `master`; lane-local and uncommitted primary work are candidate material until selected, gated, committed, and pushed.

## Overall Status

Estimated progress toward a broadly usable generalized PHP native compiler: **52%**

```
Generalized runtime/ABI foundations      [##################--] 89%
Compiler/backend consumers               [###################-] 96%
Executable generalized PHP semantics     [#############-------] 63%
Arrays, references, COW, lvalues         [########------------] 40%
Objects, properties, methods             [##------------------] 11%
Diagnostics/control-flow composition     [#####---------------] 25%
Broad integrated verification            [##########----------] 52%
```

## Current Primary State

- Current primary HEAD at review: `e3af4261 codegen: store native value results in variables`.
- Latest committed semantic compiler/runtime baseline: `e3af4261 codegen: store native value results in variables`.
- Latest integrated semantic batch adds the `phpc_native_value_clone(...)` ABI and generated-C direct-variable storage/cleanup for owned native value-result handles. Lowerable array offset reads, offset `??`, casts, and string-result operations can now persist in variables, clone on variable copy and downstream native-value consumers, and clean up on overwrite/error/program exit.
- Previous semantic batch routes generated-C lowerable array/string offset `??` through the shared value-offset presence/read ABI, preserves RHS laziness, reports diagnostics through the native diagnostic path, and returns an owned native value for downstream consumers such as output and native string-result calls.
- Recent integrated array/value-offset progress also includes generated-C array reads, direct appends, direct keyed/append assignment-expression values, direct/multi-operand array unsets, direct array writes, generated-C offset presence, generated-C string-offset writes, LLVM string-offset reads/probes, and LLVM/generated-C string-int consumers.
- Resource note at this review: `/dev/shm` is 22G total, 15G used, 7.2G free, 68% full by `df`; `/home` has 225G free by `df` and about 197G used by `du`. Broad gates should keep using explicit resource checks.

## Grand Roadmap Position

The project is making its best progress when shared runtime/ABI contracts gain real backend consumers. The value-offset family is now a meaningful executable spine for selected generated-C array/string offset reads, presence, writes, unsets, appends, assignment-expression values, and null-coalescing, with LLVM parity for string-offset reads/probes.

The compiler is still not close to full generalized PHP semantics. The major unfinished regions are full executable lvalues/writeback, references/COW, symbol environments, mutable globals/superglobals, functions/method frames, by-ref calls/returns, object/property/method behavior, include/require, exceptions/finally, exact diagnostics, and cleanup across real control flow.

## Done / In Progress / Not Done

- [x] Supervised parallel lanes and a primary integration gate are established.
- [x] Shared runtime ABI surfaces exist for strings, byte buffers, comparisons, numeric-string classification, selected conversions, array key/value operations, value-offset read/presence/mutation operations, diagnostics, branch decisions, native value output, type predicates, bitwise/shift operations, and array snapshots.
- [x] Primary has selected LLVM/generated-C consumers for primitive arithmetic, unary string-result builtins, string-int builtins, generated-C array/string offset presence/read/write/unset/append/null-coalesce slices, LLVM string-offset reads/probes, scalar output, type predicates, bitwise/shift, comparison relation results, casts/type-name output, and focused diagnostic paths.
- [x] Selected generated-C direct-variable storage and cleanup for owned native value-result handles is integrated for already-lowerable value-result families.
- [ ] In progress: array lvalue/RMW/reference/COW work. Primary has useful foundations and selected direct consumers; lane-local candidates are much broader than integrated capability.
- [ ] In progress: symbol/request/global value-flow and request-state work. Primary has surfaces, not full mutable PHP symbol behavior.
- [ ] In progress: control-flow cleanup and termination modeling. Lane-local work is useful, but broad loop/switch/goto/finally/exception behavior is not integrated.
- [ ] Not done: generalized function/method frames, argument binding, by-ref args/returns, variadics/spreads, callbacks, recursion, closures, and dynamic dispatch.
- [ ] Not done: object/class/property semantics including allocation, visibility, magic hooks, `stdClass`, dynamic names, ArrayAccess, references/COW, and exact diagnostics.
- [ ] Not done: broad differential composition coverage across real PHP programs.

## Active Roadmap Estimates

| Active item | Primary-integrated | Lane-local/candidate maturity | Current read |
| --- | ---: | ---: | --- |
| String conversion, truthiness, byte buffers | 79% | 88% | Primary has strong selected string/value surfaces, generated-C and LLVM string-offset consumers, string-int consumers, stdout/materialization paths, and byte-buffer helpers. Lane-local formatter/binary string work is broader but not fully integrated. |
| Call operation cleanup and ownership | 43% | 68% | Primary has common call diagnostics and selected cleanup routing. Real frames, binding, return semantics, by-ref calls, and dynamic dispatch remain mostly missing. |
| Comparison and conversion semantics | 75% | 84% | Primary has shared comparison/conversion surfaces and selected backend consumers. Dynamic arithmetic, warning parity, recursive arrays, object/resource/reference comparisons, and full backend parity remain open. |
| Arrays, lvalues, references, COW | 40% | 85% | Primary has selected generated-C value-offset read/presence/mutation/null-coalesce execution, owned native value-result variable storage, plus LLVM string-offset parity. Nested writeback, `??=`, RMW, references, COW, arbitrary roots, object/ArrayAccess/resource offsets, and LLVM array-offset parity remain open. |
| Symbols, globals, request state | 25% | 68% | Primary can now persist selected owned native value-result handles in generated-C direct variables with clone/overwrite cleanup. Real generalized locals, frames, imports, globals, superglobals, undefined slots, mutation, and reference assignment lowering are not integrated. |
| Objects, properties, methods | 11% | 50% | Primary has strict-identity/object comparison blockers and plans. Executable object allocation/property/method behavior is largely absent. |
| Diagnostics and control-flow cleanup | 25% | 70% | Primary has selected diagnostic/reporting and cleanup paths. Full warning ordering, recovery, terminal cleanup, loop/switch/goto/finally/exception behavior, and broad composition remain missing. |
| Broad composition verification | 52% | 50% | Focused runtime/native-link/native-runtime-ABI gates cover recent slices. Broad PHP differential coverage is still thin. |

## Recent Primary-Integrated Work

- `e3af4261 codegen: store native value results in variables`
  - Adds native value cloning plus generated-C direct-variable storage for owned native value-result handles. Source and linked executable gates cover array read values, variable copies, offset null-coalesce values, cast values, string-result values, array-append consumers, overwrite cleanup, and runtime clone independence.
- `40941ddb codegen: route offset null coalesce through value ABI`
  - Routes generated-C lowerable array/string offset `??` through shared value-offset presence/read operations with lazy RHS behavior and owned native-value result consumption.
- `c9999be0 codegen: route array reads through value offset ABI`
  - Routes generated-C lowerable array offset reads through the shared value-offset read ABI and lets owned read results feed output, mutation replacements, and string-result consumers.
- `d059565d codegen: return array assignment expression values`
  - Routes direct generated-C keyed and append array assignment expressions through the shared mutation ABI and returns the assigned value.
- `0568ebb1 codegen: route array appends through value mutation ABI`
  - Routes direct generated-C array append statements through the shared value-offset mutation ABI with diagnostic reporting and array rematerialization.
- `3b537955 codegen: route LLVM offset reads through value ABI`
  - Routes LLVM lowerable string offset reads and `isset()`/`empty()` probes through the shared value-offset ABI. LLVM array-offset reads/probes remain blocked.

## Candidate Work Not Yet Counted

- Lane-local array/lvalue candidates: nested/current-read/RMW/null-aware/foreach/reference/lvalue work, body-storage blockers, dynamic key diagnostics, and reference-source span refinements.
- Lane-local symbol/request candidates: reference/COW-aware symbol cells, request-superglobal storage, root/frame/imported alias binding, undefined-slot tracking, and request-state operation contracts.
- Lane-local conversion/string candidates: dynamic concat/source-result conversion, formatter/raw byte surfaces, primitive comparison/bitwise/division/modulo conversion work, and broader binary-safe string boundaries.
- Lane-local object/call/control/diagnostic candidates: object-property operation plans, class-policy and receiver blockers, call-frame/value boundaries, diagnostic result carriers, and termination/control-flow cleanup models.

Lane-local work is useful source material, but it is not product capability until selected into a small primary batch, gated, committed, and pushed.

## Steering Read

Recent primary work is directionally sound because it turns shared ABI surfaces into executable generated-C/LLVM consumers. The supervisor should still keep language precise: generated-C direct variables can now store selected owned native value-result handles, but this is not a full PHP symbol table, reference/COW cell model, arbitrary expression root model, or generalized local/global/request variable implementation.

The next best primary work should stay executable and narrow: select nested value-offset writeback, LLVM array-offset parity, a narrow reference/COW symbol-cell consumer, mutable request/superglobal storage, or concrete cleanup before terminal control transfer. Avoid whole-lane merges and standalone vocabulary.
