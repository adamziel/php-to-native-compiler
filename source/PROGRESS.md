# PHP Native Compiler Progress

Updated: 2026-05-22 09:25 CEST
Evaluation marker: 20260522T072500Z-primary-b713c089

This is a high-level supervisor dashboard. Percentages are candid engineering estimates, not test-suite pass rates. Primary-integrated capability means committed on `master`; lane-local and uncommitted primary work are candidate material until selected, gated, committed, and pushed.

## Overall Status

Estimated progress toward a broadly usable generalized PHP native compiler: **61%**

```
Generalized runtime/ABI foundations      [##################--] 89%
Compiler/backend consumers               [###################-] 97%
Executable generalized PHP semantics     [###############-----] 73%
Arrays, references, COW, lvalues         [############--------] 57%
Objects, properties, methods             [##------------------] 11%
Diagnostics/control-flow composition     [#####---------------] 26%
Broad integrated verification            [############--------] 59%
```

## Current Primary State

- Latest committed semantic compiler/backend baseline: `b713c089 runtime: centralize null lvalue increment defaults`.
- Latest semantic batch centralizes PHP null increment/decrement defaults for tracked array-owner lvalue updates so missing final keyed slots and existing null slots share the same runtime helper and linked generated-C coverage. This is a narrow follow-up to the missing-slot recovery batch; it does not change the high-level completion estimate.
- The previous semantic batch recovers missing final keyed slots reached by the array-lvalue increment/decrement update ABI. Direct and nested generated-C tracked array-owner paths now use PHP null defaults, write the recovered slot through the same owner/path update boundary, return the right pre/post expression value, and carry the undefined-key diagnostic as a recoverable value result.
- The previous semantic batch routes generated-C append-path pre/post increment and decrement over tracked native array owners through the existing array-lvalue owner/path update ABI. It adds parser admission for append-path increment/decrement targets, runtime append-path update behavior for null slots, and linked executable coverage while still blocking arbitrary roots, compound append RMW, append suffix RMW, references/COW, object/ArrayAccess/resource offsets, owner/value/reference-slot materialization, and LLVM/C assembly parity.
- The batch before that routes generated-C by-reference `foreach` forms to a dedicated reference-slot/cursor-binding blocker instead of the generic array-lowering rejection. The blocker names the missing generalized boundary: reference-slot symbol storage, foreach cursor reference binding, owner/path value-reference acquisition, and loop-body cleanup ownership across direct array, nested array, and array-literal/value-root iterable families. It does not implement executable by-reference foreach.
- Recent integrated array/value-offset progress also includes generated-C array reads, direct appends, direct keyed/append assignment-expression values, direct array writes, generated-C offset presence, generated-C string-offset writes, LLVM string-offset reads/probes, owned native value-result variable storage, direct `??=`, direct compound assignment, direct increment/decrement, nested writes/appends/reads/assignment values, selected read recovery, unsets, and LLVM/generated-C string-int consumers.
- Resource note at this review: `/dev/shm` is 22G total with about 8.4G free by `df`; `/home` has about 220G free by `df`. Broad gates should keep using explicit resource checks and should fall back to a non-shm target when the shared tmpfs is tight.

## Grand Roadmap Position

The project is making its best progress when shared runtime/ABI contracts gain real backend consumers. The value-offset and array-lvalue families now form a meaningful executable spine for selected generated-C array/string offset reads, presence, writes, unsets, appends, assignment-expression values, direct `??=`, direct compound assignment, direct increment/decrement, nested compound assignment, nested increment/decrement, nested writes/appends, nested assignment-expression values, nested reads, by-value foreach over tracked native array owners, and selected missing/scalar read recovery, with LLVM parity for string-offset reads/probes.

The compiler is still not close to full generalized PHP semantics. The major unfinished regions are full executable lvalues/writeback, references/COW, symbol environments, mutable globals/superglobals, functions/method frames, by-ref calls/returns, object/property/method behavior, include/require, exceptions/finally, exact diagnostics, and cleanup across real control flow.

## Done / In Progress / Not Done

- [x] Supervised parallel lanes and a primary integration gate are established.
- [x] Shared runtime ABI surfaces exist for strings, byte buffers, comparisons, numeric-string classification, selected conversions, array key/value operations, value-offset read/presence/mutation operations, diagnostics, branch decisions, native value output, type predicates, bitwise/shift operations, array snapshots, and selected array-owner lvalue operations.
- [x] Primary has selected LLVM/generated-C consumers for primitive arithmetic, unary string-result builtins, string-int builtins, generated-C array/string offset presence/read/write/append/null-coalesce slices, generated-C direct array-offset `??=`, compound-assignment, and increment/decrement statements/expression values, generated-C nested array-owner compound-assignment and increment/decrement paths, missing-slot increment/decrement recovery through the array-lvalue update ABI, generated-C append-path increment/decrement over tracked array owners, generated-C by-value foreach over tracked array owners, a dedicated generated-C by-reference foreach reference-slot blocker, generated-C array-owner lvalue unset paths, generated-C nested array-owner keyed write and append statement paths, generated-C nested array-owner assignment-expression values and reads, selected generated-C array read recovery, LLVM string-offset reads/probes, scalar output, type predicates, bitwise/shift, comparison relation results, casts/type-name output, and focused diagnostic paths.
- [x] Selected generated-C direct-variable storage and cleanup for owned native value-result handles is integrated for already-lowerable value-result families.
- [ ] In progress: array lvalue/RMW/reference/COW work. Primary has useful foundations and selected generated-C consumers; lane-local and uncommitted primary candidates are broader than integrated capability.
- [ ] In progress: symbol/request/global value-flow and request-state work. Primary has surfaces, not full mutable PHP symbol behavior.
- [ ] In progress: control-flow cleanup and termination modeling. Lane-local work is useful, but broad loop/switch/goto/finally/exception behavior is not integrated.
- [ ] Not done: generalized function/method frames, argument binding, by-ref args/returns, variadics/spreads, callbacks, recursion, closures, and dynamic dispatch.
- [ ] Not done: object/class/property semantics including allocation, visibility, magic hooks, `stdClass`, dynamic names, ArrayAccess, references/COW, and exact diagnostics.
- [ ] Not done: broad differential composition coverage across real PHP programs.

## Active Roadmap Estimates

| Active item | Primary-integrated | Lane-local/candidate maturity | Current read |
| --- | ---: | ---: | --- |
| String conversion, truthiness, byte buffers | 79% | 88% | Primary has strong selected string/value surfaces, generated-C and LLVM string-offset consumers, string-int consumers, stdout/materialization paths, and byte-buffer helpers. Lane-local formatter, binary string, stream/resource, and tracked-byte-length work is broader but not fully integrated. |
| Call operation cleanup and ownership | 43% | 68% | Primary has common call diagnostics and selected cleanup routing. Real frames, binding, return semantics, by-ref calls, and dynamic dispatch remain mostly missing. |
| Comparison and conversion semantics | 75% | 84% | Primary has shared comparison/conversion surfaces and selected backend consumers. Dynamic arithmetic, warning parity, recursive arrays, object/resource/reference comparisons, and full backend parity remain open. |
| Arrays, lvalues, references, COW | 57% | 89% | Primary has selected generated-C value-offset read/presence/mutation/null-coalesce execution, direct array-offset `??=`, compound-assignment, and increment/decrement statements/expression values, nested array-owner compound-assignment and increment/decrement paths, missing-slot and existing-null increment/decrement recovery through the array-lvalue update ABI, append-path increment/decrement over tracked array owners, by-value foreach over tracked native array owners and array literals, a dedicated by-reference foreach reference-slot blocker, direct/nested array-owner lvalue unset paths, nested array-owner keyed write and append statement paths, nested array-owner assignment-expression values, nested array-owner reads, selected missing/scalar read recovery, owned native value-result variable storage, plus LLVM string-offset parity. Executable by-reference foreach, foreach body mutation/storage, nested/arbitrary-root `??=`, compound append RMW, append suffix RMW, references, COW, object/ArrayAccess/resource offsets, and LLVM array-offset parity remain open. |
| Symbols, globals, request state | 25% | 70% | Primary can persist selected owned native value-result handles in generated-C direct variables with clone/overwrite cleanup. Lane-local request-root presence, request-superglobal, storage-root, and symbol/frame work is broader. Real generalized locals, frames, imports, globals, superglobals, undefined slots, mutation, and reference assignment lowering are not integrated. |
| Objects, properties, methods | 11% | 52% | Primary has strict-identity/object comparison blockers and plans. Lane-local declared-property, dynamic-property, and lowerable `stdClass` diagnostic ABI work exists, but executable object allocation/property/method behavior is largely absent. |
| Diagnostics and control-flow cleanup | 26% | 72% | Primary has selected diagnostic/reporting and cleanup paths, including recoverable array-read diagnostics that can continue through owned native values. Lane-local callable-shape, terminal, request/string recovery, and cleanup models are broader. Full warning ordering, recovery, terminal cleanup, loop/switch/goto/finally/exception behavior, and broad composition remain missing. |
| Broad composition verification | 59% | 52% | Focused runtime/native-link/native-runtime-ABI gates cover recent slices, including missing-slot update recovery, append-path increment/decrement, by-value foreach linked execution, nested RMW source/link behavior, and read recovery through output, storage/copy, and probe consumers. Broad PHP differential coverage is still thin. |

## Recent Primary-Integrated Work

- `b713c089 runtime: centralize null lvalue increment defaults`
  - Centralizes PHP null increment/decrement defaults for tracked array-owner lvalue updates so existing null slots and recovered missing final keyed slots share the same helper. Linked coverage now proves direct missing keys, nested missing leaves, missing decrement slots, direct existing-null slots, and nested existing-null slots through the same update ABI. This is a correctness follow-up to `87e52301`; it preserves the broader roadmap estimate.
- `87e52301 runtime: recover missing array lvalue update slots`
  - Recovers final missing keyed slots for generated-C tracked array-owner increment/decrement updates through the shared array-lvalue owner/path ABI. Runtime/source/link gates cover direct missing keys, nested missing keys, existing null slots, pre/post result values, recoverable undefined-key diagnostics, and the full `native_link` sweep at 99 tests. This still does not implement arbitrary roots, compound append RMW, append suffix RMW, references/COW, object/ArrayAccess/resource offsets, or LLVM/C assembly parity.
- `f505f727 codegen: route append increments through lvalue ABI`
  - Routes generated-C append-path pre/post increment and decrement over tracked native array owners through the shared array-lvalue owner/path update ABI. Runtime helpers materialize append-path null slots for increment/decrement, and linked native executable coverage plus the full `native_link` sweep prove the path composes with current array-lvalue consumers. This does not implement arbitrary writable roots, references/COW, object/ArrayAccess/resource offsets, or LLVM/C assembly parity.
- `78dcf1b1 codegen: classify by-reference foreach blocker`
  - Routes generated-C by-reference `foreach` to a dedicated reference-slot/cursor-binding blocker across direct array variables, nested tracked array lvalue iterables, and array-literal/value-root iterables. This removes the generic array-lowering rejection for the semantic family while keeping executable by-reference foreach, reference cursor binding, alias-preserving symbol storage, references/COW, loop-body cleanup, and exact diagnostics blocked.
- `eaedbc7f codegen: route foreach through array lvalue ABI`
  - Adds a by-value foreach iterable family to the array-owner lvalue ABI and routes generated-C foreach over tracked native array owners, nested owner paths, and array literals through owned iterable snapshots. Runtime/source/link gates cover key/value result carriers, string-result consumers in loop bodies, nested paths, and literal arrays, while by-reference foreach, prior-symbol overwrite, body storage mutation, symbol/reference storage, COW, object/ArrayAccess/resource iteration, and LLVM parity remain blocked.
- `802665ee codegen: route nested array RMW through lvalue ABI`
  - Admits nested array-index RMW assignment targets in the parser and routes generated-C nested compound assignment plus pre/post increment/decrement over tracked native array owners through the shared array-owner path/update/read/write boundaries. Focused and full `native_link` gates passed, including 92 broad native-link tests after the slice.
- `bce89074 tests: restore native link shared-boundary gate`
  - Maintenance only: restores broad `native_link` regression value with current shared-boundary expectations and conditional-lowering blocker coverage. It does not change production compiler/runtime semantics and does not move the semantic completion estimates.
- `e96e35fe codegen: route array increment updates through lvalue ABI`
  - Adds an operation-labeled update family to the array-owner lvalue path/result ABI and routes generated-C direct array-offset increment/decrement statements and expression values over tracked native array owners through that shared runtime boundary.
- `946301d0 codegen: route array compound assignments through lvalue ABI`
  - Routes generated-C direct array-offset compound assignment statements and expression values over tracked native array owners through shared array-owner lvalue read/write operations plus the native value binary result ABI.
- `106ede04 codegen: route array offset null coalesce assignment`
  - Routes generated-C direct array-offset `??=` over tracked native array owners through shared value-offset isset/read/mutation operations, preserving lazy RHS evaluation and assigned/existing result values.
- `329a7933 runtime: recover array reads through diagnostic values`
  - Routes generated-C direct array missing-key reads and nested array-lvalue missing/scalar read paths through recoverable owned null values with shared diagnostics.

## Candidate Work Not Yet Counted

- Lane-local array/lvalue candidates include executable by-reference foreach work, missing string-offset read recovery, reference-source span refinements, value-root/reference blockers, dynamic key diagnostics, scalar-offset foreach recovery, false-to-array diagnostics, owner-slot/value-slot/reference-slot materialization, and wider string/array lvalue behavior.
- Lane-local symbol/request candidates include direct request-root presence, request-superglobal storage, storage-root helpers, root/frame/imported alias binding, undefined-slot tracking, request-state operation contracts, and early function return-slot activation.
- Lane-local object/call/control/diagnostic candidates include object-property diagnostic ABI routing, declared/dynamic property policy work, class-policy and receiver blockers, call-frame/value boundaries, callable-shape diagnostic result carriers, request/string recovery, and termination/control-flow cleanup models.

Lane-local and uncommitted primary work is useful source material, but it is not product capability until selected into a small primary batch, gated, committed, and pushed.

## Steering Read

Recent primary work is directionally sound because it turns shared ABI surfaces into executable generated-C/LLVM consumers. The supervisor should still keep language precise: generated-C direct variables can store selected owned native value-result handles, and tracked native array owners can execute selected lvalue operations, but this is not a full PHP symbol table, reference/COW cell model, arbitrary expression root model, or generalized local/global/request variable implementation.

The by-reference foreach blocker is only a shared missing-boundary classification, not shipped by-reference execution. The next best primary work should stay executable and narrow: missing-slot RMW recovery, nested/arbitrary-root `??=`, append RMW forms, LLVM array-offset parity, a narrow reference/value-slot materialization consumer, mutable request/superglobal storage, or concrete cleanup before terminal control transfer.
