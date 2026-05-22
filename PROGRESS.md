# PHP Native Compiler Progress

Updated: 2026-05-22 11:13 CEST
Evaluation marker: 20260522T091328Z-primary-after-b6e271e6

This is a high-level supervisor dashboard. Percentages are candid engineering estimates, not test-suite pass rates. Primary-integrated capability means committed on `master`; lane-local and uncommitted primary work are candidate material until selected, gated, committed, and pushed.

## Overall Status

Estimated progress toward a broadly usable generalized PHP native compiler: **62%**

```
Generalized runtime/ABI foundations      [##################--] 91%
Compiler/backend consumers               [###################-] 97%
Executable generalized PHP semantics     [###############-----] 76%
Arrays, references, COW, lvalues         [############--------] 62%
Objects, properties, methods             [##------------------] 11%
Diagnostics/control-flow composition     [#####---------------] 26%
Broad integrated verification            [#############-------] 64%
```

The overall estimate stays flat. The newest primary semantic commit is useful, but it is a runtime request-state snapshot/rebuild boundary, not executable generalized superglobal semantics.

## Current Primary State

- Primary branch at review: `b6e271e6 runtime: snapshot request superglobal storage`, synced with `origin/master`.
- Latest committed semantic compiler/runtime batch: `b6e271e6`.
- New primary capability since the prior dashboard: runtime request-state helpers can snapshot backed request superglobal arrays as owned native values and rebuild `$_REQUEST` from `$_GET`, `$_POST`, and `$_COOKIE` using request-order / variables-order style policy. Runtime probe declarations were updated for both pointer widths.
- Recent committed primary progress is still concentrated in generated-C array/value-offset lvalues plus narrow symbol/request runtime surfaces: by-value foreach over tracked native owners, by-reference foreach classification, append-path increment/decrement, final keyed increment/decrement recovery, nested tracked-owner `??=`, direct value append assignment over native value handles, nested value-root path writes/appends/unsets, nested value-root assignment-expression results, owned symbol-table snapshots, and request-superglobal snapshot/rebuild storage.
- Final verification showed uncommitted `runtime/src/lib.rs` product diffs in the primary worktree. The review evidence began from the known preserved runtime cleanup hunk, but any current uncommitted runtime work is not counted as integrated capability.
- Resource note from this review: `/dev/shm` is 22G total with about 14G free and about 8.4G used; `/home` has about 214G free. Memory has about 29Gi available. Broad gates should still stay resource-aware because recent supervisor samples required `/dev/shm` cleanup.

## Grand Roadmap Position

The strongest committed line of progress is the shared value/lvalue/runtime ABI spine with real generated-C consumers. Primary now covers selected generated-C array/string offset reads, presence, writes, unsets, appends, direct value append assignment over null/false/scalar native value handles, nested value-root path writes/appends/unsets, assignment-expression values for selected array-owner/value-offset forms, direct and nested tracked-owner `??=`, direct and nested compound assignment, direct and nested increment/decrement, by-value foreach over tracked native owners, append increment/decrement, selected read/update recovery, owned symbol-table snapshots, and request-state superglobal snapshot/rebuild runtime APIs.

The compiler is still not close to full generalized PHP semantics. Major unfinished regions remain: arbitrary writable roots, reference/COW cells, mutable symbol environments, real globals/superglobals, function/method frames, by-ref calls/returns, object/property/method behavior, include/require, exceptions/finally, exact diagnostics, and cleanup across real control flow.

## Done / In Progress / Not Done

- [x] Supervised parallel lanes and primary integration gate are established.
- [x] Shared runtime ABI surfaces exist for strings, byte buffers, comparisons, selected conversions, array key/value operations, value-offset operations, diagnostics, branch decisions, native value output, array/symbol snapshots, request-superglobal snapshots/rebuild, and selected array-owner lvalue operations.
- [x] Primary has selected LLVM/generated-C consumers for primitive arithmetic, string/value-offset families, generated-C direct/nested tracked array-owner lvalues, direct and nested tracked-owner `??=`, compound assignment, increment/decrement, by-value foreach, unsets, direct value append assignment over native value storage, nested non-string value-root path writes/appends/unsets and assignment-expression results, selected recovery, direct-variable native value storage, scalar output, type predicates, bitwise/shift, casts/type-name output, and focused diagnostics.
- [ ] In progress: array lvalue/RMW/reference/COW work. Primary has useful committed consumers; lane-local candidates are broader than integrated capability.
- [ ] In progress: symbol/request/global value flow. Primary has selected symbol-table storage, owned symbol snapshots, request-superglobal snapshot/rebuild APIs, and selected direct-variable native value storage, not a generalized PHP symbol table or mutable request/global model.
- [ ] In progress: call/frame/control-flow cleanup. Lane-local work is active, but production frames, by-ref calls, returns, exceptions/finally, and broad cleanup are not integrated.
- [ ] Not done: generalized object/class/property/method semantics, including allocation, visibility, magic hooks, `stdClass`, dynamic names, ArrayAccess, references/COW, and exact diagnostics.
- [ ] Not done: broad differential composition coverage across ordinary PHP programs.

## Active Roadmap Estimates

| Active item | Primary-integrated | Lane-local/candidate maturity | Current read |
| --- | ---: | ---: | --- |
| String conversion, truthiness, byte buffers | 79% | 91% | Primary has strong selected string/value surfaces, generated-C and LLVM string-offset consumers, string-int consumers, stdout/materialization paths, and byte-buffer helpers. Lane-local work has more string-family/interpreter byte-boundary material, but exact diagnostics, object/stringable dispatch, PCRE/stream/resource behavior, and cleanup remain open. |
| Call operation cleanup and ownership | 43% | 72% | Primary has common call diagnostics and selected cleanup routing. Lane-local work has owned argument/frame/depth/cleanup contracts, but real source-level call dispatch, frame callback generation, by-ref parameters, returns, variadics/spreads, dynamic dispatch, and exact call diagnostics remain mostly missing. |
| Comparison and conversion semantics | 75% | 88% | Primary has shared comparison/conversion surfaces and selected backend consumers. Lane-local work now includes request-superglobal direct values/probes, undefined-variable continuations, array-read recovery, constants in comparisons, and a `define()` side-effect blocker. Recursive arrays, object/resource/reference comparisons, runtime constant mutation, warning parity, and full backend parity remain open. |
| Arrays, lvalues, references, COW | 62% | 93% | Primary has selected generated-C value-offset and array-owner lvalue execution, including direct/nested `??=`, direct/nested RMW, append increment/decrement, direct native-value append assignment, nested non-string value-root path writes/appends/unsets with assignment-expression results, by-value foreach, tracked-owner unsets, selected recovery, and native value storage. Lane-local candidates cover array builtins, pointer/cursor functions, value-result/reference material, and broader blockers, but executable by-reference foreach, arbitrary roots, references, COW, ArrayAccess/resource offsets, and LLVM array parity remain open. |
| Symbols, globals, request state | 29% | 78% | Primary has runtime symbol-table handles, read/write clone storage, owned symbol-table snapshots, request-superglobal snapshot/rebuild runtime APIs, and selected generated-C direct-variable native value storage. Lane-local work covers direct request values/probes, `$GLOBALS` slot routing, request-root aliases, undefined-slot tracking, and request-state operation contracts, but not a generalized mutable symbol table, request model, aliases, or global/import reconciliation. |
| Objects, properties, methods | 11% | 56% | Primary has blockers/plans and selected comparison identity handling. Lane-local work has property/runtime ABIs, inherited static-property metadata, method/constructor metadata preflights, and object-property blockers, but executable object allocation/property/method behavior is still largely absent. |
| Diagnostics and control-flow cleanup | 26% | 75% | Primary has selected diagnostic/reporting and cleanup paths, including recoverable array-read/update diagnostics. Lane-local work has richer request diagnostic blockers, structured-control-flow, and owner-cell cleanup/diagnostic execution contracts, but real loop/switch/goto/finally/exception lowering, warning ordering, and terminal cleanup are not integrated. |
| Broad composition verification | 64% | 55% | Focused runtime/native-link/native-runtime-ABI gates cover recent primary slices, including value-root path mutations, symbol snapshots, and request-state snapshot/rebuild APIs. Broad PHP differential coverage remains thin, and many lane-local claims are validated in isolated worktrees rather than primary. |

## Primary-Integrated Capability

- `b6e271e6 runtime: snapshot request superglobal storage`
  - Runtime request state can snapshot backed superglobal arrays as owned native values and rebuild `$_REQUEST` from backed `$_GET`, `$_POST`, and `$_COOKIE` using order policy. Probe ABI declarations were updated. No executable PHP superglobal lowering was added.
- `ebaceb64 runtime: materialize symbol table snapshots`
  - Runtime symbol-table handles preserve root slot insertion order and can materialize an owned PHP array snapshot through `phpc_native_symbol_table_snapshot_value`, with focused runtime coverage.
- `9e4c5fd codegen: route value path unsets through offset ABI`
  - Generated-C direct and nested non-string value-root `unset(...)` targets reuse a shared value-offset path unset ABI and write the selected result back through native value storage.
- `a68693bd codegen: return value-root path assignment values`
  - Generated-C nested non-string value-root keyed and append assignment expressions reuse shared path mutation helpers while returning RHS values to output/storage consumers.
- `030cb4ac codegen: route nested value offsets through path ABI`
  - Generated-C nested non-string value-root writes and appends route through shared path-level value-offset mutation ABIs.
- `d468cd17 codegen: route value appends through offset mutation ABI`
  - Generated-C direct value append assignments over null, false, and scalar native value handles route through the shared offset mutation ABI.
- `5ab507cd codegen: route nested array ??= through lvalue ABI`
  - Nested tracked native array-owner null-coalescing assignment uses owner/path null-aware probe plus existing lvalue read/write operations.
- `eaedbc7f codegen: route foreach through array lvalue ABI`
  - Generated-C by-value foreach over tracked native array owners, nested owner paths, and array literals executes through owned iterable snapshots.
- `802665ee codegen: route nested array RMW through lvalue ABI`
  - Nested keyed compound assignment plus pre/post increment/decrement over tracked native array owners route through shared owner/path read/write/update boundaries.

## Candidate Work Not Yet Counted

- Lane-local request/symbol candidates: generated LLVM/C direct request superglobal values, request offset and whole-bag `isset()` / `empty()` probes, request warning continuations, `$GLOBALS["GLOBALS"]` root-slot routing, request-root alias handling, request dynamic-path ordered diagnostic blockers, and mutable request/storage operation contracts.
- Lane-local array/lvalue candidates: array builtin native value-result families, `current()` / `next()` pointer/cursor propagation through lvalue owners, executable by-reference foreach materialization, owner/value/reference-slot materialization, value-root/reference blockers, dynamic key diagnostics, broader false/null/scalar recovery, and wider string/array lvalue behavior.
- Lane-local object/call/control/diagnostic candidates: inherited static-property metadata blockers, object-property array-offset ABIs, constructor/method metadata preflights, call-frame/value boundaries, dynamic callable cleanup, diagnostic result carriers, owner-cell reference-return/result-branch execution planning, and termination/control-flow cleanup models.

Lane-local and uncommitted primary work is useful source material, but it is not product capability until selected into a small primary batch, gated, committed, and pushed.

## Steering Read

Recent primary work is directionally sound because it continues to turn shared ABI surfaces into executable generated-C consumers and now adds request-state runtime storage needed by future superglobal consumers. The supervisor should keep language precise: primary can snapshot symbol tables and backed request superglobal storage, but this is not a full PHP symbol table, mutable request/global model, reference/COW cell model, arbitrary writable-root model, or generalized frame/call implementation.

Next best primary work should stay executable and narrow. The most natural follow-up is a small request/superglobal consumer over the newly integrated request-state runtime boundary, or another slice that replaces a real blocker with executable behavior across a semantic family. Whole-lane merges, fixture-shaped lowering, and blocker-only churn should remain excluded.
