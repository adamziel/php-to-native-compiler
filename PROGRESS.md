# PHP Native Compiler Progress

Updated: 2026-05-22 06:59 CEST
Evaluation marker: 20260522T045941Z-primary-f95c5a51

This is a high-level supervisor dashboard. Percentages are candid engineering estimates, not test-suite pass rates. Primary-integrated capability means committed on `master`; lane-local and uncommitted primary work are candidate material until selected, gated, committed, and pushed.

## Overall Status

Estimated progress toward a broadly usable generalized PHP native compiler: **55%**

```
Generalized runtime/ABI foundations      [##################--] 89%
Compiler/backend consumers               [###################-] 96%
Executable generalized PHP semantics     [#############-------] 67%
Arrays, references, COW, lvalues         [#########-----------] 45%
Objects, properties, methods             [##------------------] 11%
Diagnostics/control-flow composition     [#####---------------] 25%
Broad integrated verification            [###########---------] 54%
```

## Current Primary State

- Current primary HEAD at review: `f95c5a51 codegen: route nested array reads through lvalue ABI`.
- Latest committed semantic compiler/runtime baseline: `f95c5a51 codegen: route nested array reads through lvalue ABI`.
- Latest integrated semantic batch adds a read family to the existing `NativeArrayLvalueOwner`/`NativeArrayPathSegment`/`NativeArrayLvalueResult` ABI and routes generated-C nested array reads over tracked native array owners through that shared owner/path result boundary.
- Previous semantic batch routes generated-C nested array assignment expressions over tracked native array owners through the existing lvalue write boundary while returning the assigned replacement value to expression consumers.
- Earlier semantic batches added append path segments, keyed writes, and unset paths to the narrow array-owner lvalue ABI, so generated-C nested appends, nested keyed writes, and direct/nested/multi-target `unset(...)` now share operation-labeled owner/path semantics.
- Recent integrated array/value-offset progress also includes generated-C array reads, direct appends, direct keyed/append assignment-expression values, direct array writes, generated-C offset presence, generated-C string-offset writes, LLVM string-offset reads/probes, owned native value-result variable storage, and LLVM/generated-C string-int consumers.
- Resource note at this review: `/dev/shm` is 22G total, 15G used, 7.6G free, 66% full by `df`; `/home` has about 224G free. Broad gates should keep using explicit resource checks.

## Grand Roadmap Position

The project is making its best progress when shared runtime/ABI contracts gain real backend consumers. The value-offset and array-lvalue families now form a meaningful executable spine for selected generated-C array/string offset reads, presence, writes, unsets, appends, assignment-expression values, null-coalescing, nested writes/appends, nested assignment-expression values, and nested reads, with LLVM parity for string-offset reads/probes.

The compiler is still not close to full generalized PHP semantics. The major unfinished regions are full executable lvalues/writeback, references/COW, symbol environments, mutable globals/superglobals, functions/method frames, by-ref calls/returns, object/property/method behavior, include/require, exceptions/finally, exact diagnostics, and cleanup across real control flow.

## Done / In Progress / Not Done

- [x] Supervised parallel lanes and a primary integration gate are established.
- [x] Shared runtime ABI surfaces exist for strings, byte buffers, comparisons, numeric-string classification, selected conversions, array key/value operations, value-offset read/presence/mutation operations, diagnostics, branch decisions, native value output, type predicates, bitwise/shift operations, array snapshots, and selected array-owner lvalue operations.
- [x] Primary has selected LLVM/generated-C consumers for primitive arithmetic, unary string-result builtins, string-int builtins, generated-C array/string offset presence/read/write/append/null-coalesce slices, generated-C array-owner lvalue unset paths, generated-C nested array-owner keyed write/append statement paths, generated-C nested array-owner assignment-expression values and reads, LLVM string-offset reads/probes, scalar output, type predicates, bitwise/shift, comparison relation results, casts/type-name output, and focused diagnostic paths.
- [x] Selected generated-C direct-variable storage and cleanup for owned native value-result handles is integrated for already-lowerable value-result families.
- [ ] In progress: array lvalue/RMW/reference/COW work. Primary has useful foundations and selected direct consumers; lane-local and uncommitted primary candidates are broader than integrated capability.
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
| Arrays, lvalues, references, COW | 45% | 87% | Primary has selected generated-C value-offset read/presence/mutation/null-coalesce execution, direct/nested array-owner lvalue unset paths, nested array-owner keyed write and append statement paths, nested array-owner assignment-expression values, nested array-owner reads, owned native value-result variable storage, plus LLVM string-offset parity. `??=`, RMW, references, COW, arbitrary roots, object/ArrayAccess/resource offsets, and LLVM array-offset parity remain open. |
| Symbols, globals, request state | 25% | 69% | Primary can persist selected owned native value-result handles in generated-C direct variables with clone/overwrite cleanup. Lane-local storage-root and symbol work is broader. Real generalized locals, frames, imports, globals, superglobals, undefined slots, mutation, and reference assignment lowering are not integrated. |
| Objects, properties, methods | 11% | 51% | Primary has strict-identity/object comparison blockers and plans. Lane-local declared-property and property-operation carriers exist, but executable object allocation/property/method behavior is largely absent. |
| Diagnostics and control-flow cleanup | 25% | 71% | Primary has selected diagnostic/reporting and cleanup paths. Lane-local callable-shape, terminal, and cleanup models are broader. Full warning ordering, recovery, terminal cleanup, loop/switch/goto/finally/exception behavior, and broad composition remain missing. |
| Broad composition verification | 54% | 51% | Focused runtime/native-link/native-runtime-ABI gates cover recent slices. Broad PHP differential coverage is still thin. |

## Recent Primary-Integrated Work

- `f95c5a51 codegen: route nested array reads through lvalue ABI`
  - Adds a read family to the existing array-owner lvalue path/result ABI and routes generated-C nested array-index reads over tracked native array owners through the shared owner/path result boundary. Runtime/source/linked gates cover direct and nested read paths plus output, print, string-result, and array-value consumers.
- `98729ed3 codegen: return nested array assignment values`
  - Routes generated-C nested keyed and append array assignment expressions over tracked native array owners through the shared array-owner lvalue path/result write boundary, while returning the assigned replacement value to downstream expression consumers.
- `e5431f8a codegen: route nested array appends through lvalue ABI`
  - Adds append path segments to the array-owner lvalue path/result ABI and routes generated-C nested append assignments over tracked native array owners through the shared owner/path result boundary, including arbitrary prefix/suffix key materialization and centralized append-path diagnostics.
- `0a5ab20a codegen: route nested array writes through lvalue ABI`
  - Extends the array-owner lvalue path/result ABI to keyed writes and routes generated-C nested array-index assignments over tracked native array owners through the shared owner/path result boundary, including arbitrary key expression materialization and centralized scalar-intermediate blockers.
- `d53a52f6 codegen: route array unsets through lvalue ABI`
  - Adds a narrow array-owner lvalue path/result ABI and routes generated-C direct, nested, and multi-target array-index `unset(...)` through operation-labeled owner/path semantics with arbitrary key expression materialization.
- `e3af4261 codegen: store native value results in variables`
  - Adds native value cloning plus generated-C direct-variable storage for owned native value-result handles. Source and linked executable gates cover array read values, variable copies, offset null-coalesce values, cast values, string-result values, array-append consumers, overwrite cleanup, and runtime clone independence.
- `40941ddb codegen: route offset null coalesce through value ABI`
  - Routes generated-C lowerable array/string offset `??` through shared value-offset presence/read operations with lazy RHS behavior and owned native-value result consumption.
- `c9999be0 codegen: route array reads through value offset ABI`
  - Routes generated-C lowerable array offset reads through the shared value-offset read ABI and lets owned read results feed output, mutation replacements, and string-result consumers.

## Candidate Work Not Yet Counted

- Lane-local array/lvalue candidates: nested/current-read/RMW/null-aware/foreach/reference/lvalue work, body-storage blockers, dynamic key diagnostics, scalar-offset foreach recovery, false-to-array diagnostics, and reference-source span refinements.
- Lane-local symbol/request candidates: reference/COW-aware symbol cells, request-superglobal storage, storage-root helpers, root/frame/imported alias binding, undefined-slot tracking, and request-state operation contracts.
- Lane-local conversion/string candidates: dynamic concat/source-result conversion, `print_r(..., false)` formatter stdout/bool routing, formatter/raw byte surfaces, primitive comparison/bitwise/division/modulo conversion work, and broader binary-safe string boundaries.
- Lane-local object/call/control/diagnostic candidates: object-property operation plans, declared property-exists routing, class-policy and receiver blockers, call-frame/value boundaries, callable-shape diagnostic result carriers, and termination/control-flow cleanup models.

Lane-local and uncommitted primary work is useful source material, but it is not product capability until selected into a small primary batch, gated, committed, and pushed.

## Steering Read

Recent primary work is directionally sound because it turns shared ABI surfaces into executable generated-C/LLVM consumers. The supervisor should still keep language precise: generated-C direct variables can store selected owned native value-result handles, and tracked native array owners can execute selected lvalue operations, but this is not a full PHP symbol table, reference/COW cell model, arbitrary expression root model, or generalized local/global/request variable implementation.

The nested lvalue-read candidate has been finished as a small primary batch. The next best primary work should stay executable and narrow: `??=`, RMW/increment, LLVM array-offset parity, a narrow reference/COW symbol-cell consumer, mutable request/superglobal storage, or concrete cleanup before terminal control transfer. Avoid whole-lane merges and standalone vocabulary.
