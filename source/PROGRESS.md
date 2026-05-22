# PHP Native Compiler Progress

Updated: 2026-05-22 10:25 CEST
Evaluation marker: 20260522T082500Z-primary-after-a68693bd

This is a high-level supervisor dashboard. Percentages are candid engineering estimates, not test-suite pass rates. Primary-integrated capability means committed on `master`; lane-local and uncommitted primary work are candidate material until selected, gated, committed, and pushed.

## Overall Status

Estimated progress toward a broadly usable generalized PHP native compiler: **62%**

```
Generalized runtime/ABI foundations      [##################--] 89%
Compiler/backend consumers               [###################-] 97%
Executable generalized PHP semantics     [###############-----] 76%
Arrays, references, COW, lvalues         [############--------] 61%
Objects, properties, methods             [##------------------] 11%
Diagnostics/control-flow composition     [#####---------------] 26%
Broad integrated verification            [############--------] 62%
```

The estimate moved only in arrays/lvalues because the new primary semantic commit is a narrow generated-C value-offset consumer.

## Current Primary State

- Primary branch at review: `e3a97c14 docs: update progress after value-root assignment values`; latest semantic capability is `a68693bd`.
- Latest committed semantic compiler/runtime batch: `a68693bd codegen: return value-root path assignment values`.
- Recent committed semantic progress is concentrated in generated-C array/value-offset lvalues: by-value foreach over tracked native owners, a by-reference foreach reference-slot blocker, append-path increment/decrement, missing final keyed increment/decrement recovery, shared null increment/decrement defaults, nested tracked-owner `??=`, direct value append assignment over stored native value handles, nested non-string value-root path writes/appends, and nested value-root assignment-expression values.
- One preserved unstaged runtime cleanup hunk remains in `runtime/src/lib.rs`; it is not counted as integrated capability.
- Resource note: `/dev/shm` is 22G total with about 8.5G free and about 14G used; `/home` has about 216G free and about 206G used. The largest sampled tmpfs target is `phpc-target-native-call-semantics` at about 8.2G, so broad gates still need resource checks.

## Grand Roadmap Position

The strongest committed line of progress is the shared value/lvalue ABI spine with real generated-C consumers. Primary now covers selected generated-C array/string offset reads, presence, writes, unsets, appends, direct value append assignment over null/false/scalar native value handles, nested value-root path writes/appends, assignment-expression values for selected array-owner/value-offset forms including nested value-root path assignment expressions, direct and nested tracked-owner `??=`, direct and nested compound assignment, direct and nested increment/decrement, by-value foreach over tracked native owners, append increment/decrement, and selected read/update recovery.

The compiler is still not close to full generalized PHP semantics. Major unfinished regions remain: arbitrary writable roots, reference/COW cells, symbol environments, mutable globals/superglobals, function/method frames, by-ref calls/returns, object/property/method behavior, include/require, exceptions/finally, exact diagnostics, and cleanup across real control flow.

## Done / In Progress / Not Done

- [x] Supervised parallel lanes and primary integration gate are established.
- [x] Shared runtime ABI surfaces exist for strings, byte buffers, comparisons, selected conversions, array key/value operations, value-offset operations, diagnostics, branch decisions, native value output, snapshots, and selected array-owner lvalue operations.
- [x] Primary has selected LLVM/generated-C consumers for primitive arithmetic, string/value-offset families, generated-C direct/nested tracked array-owner lvalues, direct and nested tracked-owner `??=`, compound assignment, increment/decrement, by-value foreach, unsets, direct value append assignment over native value storage, nested non-string value-root path writes/appends and assignment-expression results, selected recovery, direct-variable native value storage, scalar output, type predicates, bitwise/shift, casts/type-name output, and focused diagnostics.
- [ ] In progress: array lvalue/RMW/reference/COW work. Primary has useful committed consumers; lane-local candidates are broader than integrated capability.
- [ ] In progress: symbol/request/global value flow. Primary has surfaces and selected direct-variable storage, not a generalized PHP symbol table or mutable request/global model.
- [ ] In progress: call/frame/control-flow cleanup. Lane-local work is active, but production frames, by-ref calls, returns, exceptions/finally, and broad cleanup are not integrated.
- [ ] Not done: generalized object/class/property/method semantics, including allocation, visibility, magic hooks, `stdClass`, dynamic names, ArrayAccess, references/COW, and exact diagnostics.
- [ ] Not done: broad differential composition coverage across ordinary PHP programs.

## Active Roadmap Estimates

| Active item | Primary-integrated | Lane-local/candidate maturity | Current read |
| --- | ---: | ---: | --- |
| String conversion, truthiness, byte buffers | 79% | 91% | Primary has strong selected string/value surfaces, generated-C and LLVM string-offset consumers, string-int consumers, stdout/materialization paths, and byte-buffer helpers. Lane-local work has many more string-family ABIs, but exact diagnostics, object/stringable dispatch, PCRE/stream/resource behavior, and cleanup remain open. |
| Call operation cleanup and ownership | 43% | 72% | Primary has common call diagnostics and selected cleanup routing. Lane-local work has owned argument/frame/depth/cleanup contracts, but real source-level call dispatch, frame callback generation, by-ref parameters, returns, variadics/spreads, dynamic dispatch, and exact call diagnostics remain mostly missing. |
| Comparison and conversion semantics | 75% | 86% | Primary has shared comparison/conversion surfaces and selected backend consumers. Lane-local work adds more constant, magic constant, undefined-variable, array-read, warning-continuation, and conversion-result paths. Recursive arrays, object/resource/reference comparisons, warning parity, and full backend parity remain open. |
| Arrays, lvalues, references, COW | 61% | 92% | Primary has selected generated-C value-offset and array-owner lvalue execution, including direct/nested `??=`, direct/nested RMW, append increment/decrement, direct native-value append assignment, nested non-string value-root path writes/appends with assignment-expression results, by-value foreach, unsets, selected recovery, and native value storage. Lane-local candidates cover much more array builtin/value-result/reference material, but executable by-reference foreach, arbitrary roots, append RMW, references, COW, ArrayAccess/resource offsets, and LLVM array parity remain open. |
| Symbols, globals, request state | 26% | 74% | Primary can persist selected owned native value-result handles in generated-C direct variables and compose that storage with selected value-offset writeback. Lane-local work covers known-array `$GLOBALS`/request-superglobal nested append/unset and request foreach mutation blockers, but not a generalized symbol table, mutable request model, aliases, or global/import reconciliation. |
| Objects, properties, methods | 11% | 55% | Primary has blockers/plans and selected comparison identity handling. Lane-local work has property/runtime ABIs and method/constructor metadata preflights, but executable object allocation/property/method behavior is still largely absent. |
| Diagnostics and control-flow cleanup | 26% | 74% | Primary has selected diagnostic/reporting and cleanup paths, including recoverable array-read/update diagnostics. Lane-local work has richer structured-control-flow and owner-cell cleanup/diagnostic execution contracts, but real loop/switch/goto/finally/exception lowering, warning ordering, and terminal cleanup are not integrated. |
| Broad composition verification | 62% | 55% | Focused runtime/native-link/native-runtime-ABI gates cover recent primary slices, including nested value-root assignment-expression results. Broad PHP differential coverage remains thin, and many lane-local claims are validated in isolated worktrees rather than primary. |

## Primary-Integrated Capability

- `a68693bd codegen: return value-root path assignment values`
  - Generated-C nested non-string value-root keyed and append assignment expressions now reuse the shared path mutation helpers while returning RHS values to output/storage consumers, including native value-result RHS handles.
- `030cb4ac codegen: route nested value offsets through path ABI`
  - Generated-C nested non-string value-root writes and appends now route through shared path-level value-offset mutation ABIs, preserving null/false root materialization, scalar-root diagnostics, native value storage write-back, and read/probe composition.
- `d468cd17 codegen: route value appends through offset mutation ABI`
  - Generated-C direct value append assignments over null, false, and scalar native value handles now route through `phpc_native_value_offset_mutation_operation_with_diagnostic`, write the selected result back through native value storage, and compose with value-offset reads.
- `5ab507cd codegen: route nested array ??= through lvalue ABI`
  - Nested tracked native array-owner null-coalescing assignment now uses an owner/path null-aware probe plus existing lvalue read/write operations, preserving lazy RHS evaluation and expression-result ownership.
- `b713c089 runtime: centralize null lvalue increment defaults`
  - Existing null slots and recovered missing final keyed slots now share the same PHP null increment/decrement helper through the tracked array-owner lvalue update ABI.
- `87e52301 runtime: recover missing array lvalue update slots`
  - Final missing keyed slots for generated-C tracked array-owner increment/decrement recover through the shared owner/path update result, including direct/nested paths and recoverable undefined-key diagnostics.
- `f505f727 codegen: route append increments through lvalue ABI`
  - Final append pre/post increment and decrement over tracked native array owners route through the shared array-lvalue update ABI and linked executable coverage.
- `78dcf1b1 codegen: classify by-reference foreach blocker`
  - By-reference foreach now reaches a dedicated reference-slot/cursor-binding blocker. This clarifies the missing boundary but does not implement executable by-reference foreach.
- `eaedbc7f codegen: route foreach through array lvalue ABI`
  - Generated-C by-value foreach over tracked native array owners, nested owner paths, and array literals executes through owned iterable snapshots.
- `802665ee codegen: route nested array RMW through lvalue ABI`
  - Nested keyed compound assignment plus pre/post increment/decrement over tracked native array owners route through shared owner/path read/write/update boundaries.

## Candidate Work Not Yet Counted

- Lane-local array/lvalue candidates: array builtin native value-result families, executable by-reference foreach materialization, owner/value/reference-slot materialization, value-root/reference blockers, dynamic key diagnostics, broader false/null/scalar recovery, and wider string/array lvalue behavior.
- Lane-local symbol/request candidates: known-array local/`$GLOBALS[$expr]`/request-superglobal nested append/unset, request foreach mutation blockers, request-root presence, request-superglobal storage, storage-root helpers, root/frame/imported alias binding, undefined-slot tracking, and request-state operation contracts.
- Lane-local object/call/control/diagnostic candidates: object-property array-offset runtime ABIs, constructor/method metadata preflights, call-frame/value boundaries, dynamic callable cleanup, diagnostic result carriers, owner-cell cleanup/diagnostic execution planning, and termination/control-flow cleanup models.

Lane-local and uncommitted primary work is useful source material, but it is not product capability until selected into a small primary batch, gated, committed, and pushed.

## Steering Read

Recent primary work is directionally sound because it turns shared ABI surfaces into executable generated-C consumers. The supervisor should keep language precise: tracked native array owners can execute selected lvalue operations, and direct variables can store and mutate selected owned native value-result handles, but this is not a full PHP symbol table, reference/COW cell model, arbitrary writable-root model, or generalized request/global/frame implementation.

Next best primary work should stay executable and narrow. The broader project needs more work in owner/value/reference-slot materialization, mutable request/superglobal storage, LLVM array/value-offset parity, exact diagnostic/recovery ordering, concrete call/frame execution, or cleanup before terminal control transfer.
