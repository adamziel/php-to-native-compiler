# PHP Native Compiler Progress

Updated: 2026-05-22 09:30 CEST
Evaluation marker: 20260522T073015Z-primary-5ab507cd

This is a high-level supervisor dashboard. Percentages are candid engineering estimates, not test-suite pass rates. Primary-integrated capability means committed on `master`; lane-local and uncommitted primary work are candidate material until selected, gated, committed, and pushed.

## Overall Status

Estimated progress toward a broadly usable generalized PHP native compiler: **61%**

```
Generalized runtime/ABI foundations      [##################--] 89%
Compiler/backend consumers               [###################-] 97%
Executable generalized PHP semantics     [###############-----] 74%
Arrays, references, COW, lvalues         [############--------] 58%
Objects, properties, methods             [##------------------] 11%
Diagnostics/control-flow composition     [#####---------------] 26%
Broad integrated verification            [############--------] 60%
```

## Current Primary State

- Primary HEAD at review: `5ab507cd codegen: route nested array ??= through lvalue ABI`, synced with `origin/master`.
- Latest committed semantic compiler/runtime batch: `5ab507cd codegen: route nested array ??= through lvalue ABI`.
- Recent committed semantic progress is concentrated in generated-C array-owner lvalues: by-value foreach over tracked native owners, a by-reference foreach reference-slot blocker, append-path increment/decrement, missing final keyed increment/decrement recovery, shared null increment/decrement defaults, and nested tracked-owner `??=`.
- One pre-existing unstaged runtime cleanup hunk remains preserved in `runtime/src/lib.rs`; it is not counted as integrated capability.
- Resource note: `/dev/shm` is 22G total with about 7.5G free; `/home` has about 218G free. Broad gates should keep explicit resource checks and use `/home` target dirs when tmpfs is tight.

## Grand Roadmap Position

The project is strongest when shared runtime/ABI contracts gain real backend consumers. The committed value-offset and array-lvalue spine now covers selected generated-C array/string offset reads, presence, writes, unsets, appends, assignment-expression values, direct and nested tracked-owner `??=`, direct and nested compound assignment, direct and nested increment/decrement, by-value foreach over tracked native owners, append increment/decrement, and selected read/update recovery.

The compiler is still not close to full generalized PHP semantics. Major unfinished regions remain: arbitrary writable roots, reference/COW cells, symbol environments, mutable globals/superglobals, function/method frames, by-ref calls/returns, object/property/method behavior, include/require, exceptions/finally, exact diagnostics, and cleanup across real control flow.

## Done / In Progress / Not Done

- [x] Supervised parallel lanes and primary integration gate are established.
- [x] Shared runtime ABI surfaces exist for strings, byte buffers, comparisons, selected conversions, array key/value operations, value-offset operations, diagnostics, branch decisions, native value output, snapshots, and selected array-owner lvalue operations.
- [x] Primary has selected LLVM/generated-C consumers for primitive arithmetic, string/value-offset families, generated-C direct/nested tracked array-owner lvalues, direct and nested tracked-owner `??=`, compound assignment, increment/decrement, by-value foreach, unsets, selected recovery, direct-variable native value storage, scalar output, type predicates, bitwise/shift, casts/type-name output, and focused diagnostics.
- [ ] In progress: array lvalue/RMW/reference/COW work. Primary has useful committed consumers; lane-local and staged primary candidates are broader than integrated capability.
- [ ] In progress: symbol/request/global value flow. Primary has surfaces and selected direct-variable storage, not a generalized PHP symbol table or mutable request/global model.
- [ ] In progress: call/frame/control-flow cleanup. Lane-local work is active, but production frames, by-ref calls, returns, exceptions/finally, and broad cleanup are not integrated.
- [ ] Not done: generalized object/class/property/method semantics, including allocation, visibility, magic hooks, `stdClass`, dynamic names, ArrayAccess, references/COW, and exact diagnostics.
- [ ] Not done: broad differential composition coverage across ordinary PHP programs.

## Active Roadmap Estimates

| Active item | Primary-integrated | Lane-local/candidate maturity | Current read |
| --- | ---: | ---: | --- |
| String conversion, truthiness, byte buffers | 79% | 88% | Primary has strong selected string/value surfaces, generated-C and LLVM string-offset consumers, string-int consumers, stdout/materialization paths, and byte-buffer helpers. Broader formatter, binary string, stream/resource, and tracked-byte-length work remains candidate material. |
| Call operation cleanup and ownership | 43% | 69% | Primary has common call diagnostics and selected cleanup routing. Lane-local dynamic callee-name and argument cleanup helpers are broader, but real frames, binding, returns, by-ref calls, variadics/spreads, and dynamic dispatch remain mostly missing. |
| Comparison and conversion semantics | 75% | 84% | Primary has shared comparison/conversion surfaces and selected backend consumers. Dynamic arithmetic, warning parity, recursive arrays, object/resource/reference comparisons, and full backend parity remain open. |
| Arrays, lvalues, references, COW | 58% | 90% | Primary has selected generated-C value-offset and array-owner lvalue execution, including direct/nested `??=`, direct/nested RMW, append increment/decrement, by-value foreach, unsets, selected recovery, and native value storage. Lane-local candidates cover more, including by-reference foreach materialization, but executable by-reference foreach, arbitrary roots, append RMW, references, COW, ArrayAccess/resource offsets, and LLVM array parity remain open. |
| Symbols, globals, request state | 25% | 70% | Primary can persist selected owned native value-result handles in generated-C direct variables. Lane-local request-root, request-superglobal, storage-root, frame/import, and symbol operation work is broader but not integrated. |
| Objects, properties, methods | 11% | 52% | Primary has blockers/plans and selected comparison identity handling. Lane-local object/property policies and `stdClass` diagnostic ABI work exist, but executable object allocation/property/method behavior is largely absent. |
| Diagnostics and control-flow cleanup | 26% | 72% | Primary has selected diagnostic/reporting and cleanup paths, including recoverable array-read/update diagnostics. Lane-local terminal/control-flow and diagnostic models are broader; full warning ordering, terminal cleanup, loop/switch/goto/finally/exception behavior, and broad composition remain missing. |
| Broad composition verification | 59% | 52% | Focused runtime/native-link/native-runtime-ABI gates cover recent slices, including missing-slot update recovery, append-path increment/decrement, by-value foreach, nested RMW, and read recovery. Broad PHP differential coverage is still thin. |

## Recent Primary-Integrated Work

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

- Lane-local array/lvalue candidates: executable by-reference foreach materialization, native array builtin value-result propagation, owner/value/reference-slot materialization, value-root/reference blockers, dynamic key diagnostics, false/null/scalar recovery, and wider string/array lvalue behavior.
- Lane-local symbol/request candidates: request-root presence, request-superglobal storage, storage-root helpers, root/frame/imported alias binding, undefined-slot tracking, and request-state operation contracts.
- Lane-local object/call/control/diagnostic candidates: object-property policy/result routing, declared/dynamic property work, call-frame/value boundaries, dynamic callable cleanup, diagnostic result carriers, and termination/control-flow cleanup models.

Lane-local and uncommitted primary work is useful source material, but it is not product capability until selected into a small primary batch, gated, committed, and pushed.

## Steering Read

Recent primary work is directionally sound because it turns shared ABI surfaces into executable generated-C consumers. The supervisor should keep language precise: tracked native array owners can execute selected lvalue operations, and direct variables can store selected owned native value-result handles, but this is not a full PHP symbol table, reference/COW cell model, arbitrary writable-root model, or generalized request/global/frame implementation.

Next best primary work should stay executable and narrow: prefer owner/value/reference-slot materialization, mutable request/superglobal storage, append RMW forms, LLVM array-offset parity, exact diagnostic/recovery ordering, or concrete cleanup before terminal control transfer.
