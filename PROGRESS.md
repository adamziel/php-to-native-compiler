# PHP Native Compiler Progress

Updated: 2026-05-21 21:41 CEST
Evaluation marker: 20260521T194119Z

This is a high-level supervisor dashboard. Percentages are candid engineering estimates, not test-suite pass rates. Primary-integrated capability means committed on `master`; lane-local work is candidate material until selected, gated, committed, and pushed.

## Overall Status

Estimated progress toward a broadly usable generalized PHP native compiler: **29%**

```
Generalized runtime/ABI foundations      [##############------] 71%
Compiler/backend consumers               [#############-------] 66%
Executable generalized PHP semantics     [#######-------------] 33%
Arrays, references, COW, lvalues         [####----------------] 21%
Objects, properties, methods             [##------------------] 10%
Diagnostics/control-flow composition     [####----------------] 18%
Broad integrated verification            [####----------------] 21%
```

## Current Primary State

- Primary semantic HEAD at this review: `a9823290 native: route array appends through diagnostic ABI`.
- Primary worktree status at live check: clean and synced with `origin/master`.
- Latest integrated progress is semantic product work, not only metadata: generated-C array literal append lowering now uses a diagnostic-carrying runtime ABI.
- Resource caveat: `/dev/shm` was around 7.5-7.6G free during this review; broad primary gates should continue using disk-backed targets until headroom improves.

## Grand Roadmap Position

The compiler is steadily replacing backend-local rejections and ad hoc generated-C decisions with reusable runtime/ABI contracts and selected executable consumers. Recent progress is strongest in comparisons, value-result materialization, array append diagnostics, and array/key/value foundations.

The product is still far from full generalized PHP. The largest missing regions remain references/COW, executable lvalues, user calls and frames, object/class/property semantics, mutable globals/superglobals, include/require, exceptions/finally, exact diagnostics, and cleanup across real control flow.

## Done / In Progress / Not Done

- [x] Supervised parallel lanes and primary integration gate established.
- [x] Shared runtime ABI surfaces for strings, comparisons, numeric-string classification, array keys/value operations, request-state snapshots, selected conversion helpers, and selected diagnostic carriers.
- [x] Generated-C consumers for selected string builtins, array key/value operations, array append diagnostics, comparison operands/results, strict array/object identity in selected array-search builtins, and array-handle comparisons.
- [x] Reusable array-entry snapshot ABI for future foreach/lvalue/reference consumers.
- [~] Replace shared blockers with executable semantics one family at a time.
- [~] Generalize value/result ownership across returns, call args, conditions, branch joins, stdout, discarded temporaries, and cleanup.
- [~] Array lvalue/RMW/reference/COW work: primary has foundations and append diagnostics; lane-local candidates are much stronger than integrated capability.
- [ ] Full symbol environment semantics for locals/imports/globals/superglobals, undefined slots, repeated calls, writes/unset, references, and request/global separation.
- [ ] Function/method frames, argument binding, by-ref args/returns, variadics/spreads, callbacks, constructors, and dynamic dispatch.
- [ ] Object/class/property semantics including visibility, magic hooks, stdClass, dynamic names, references/COW, and exact diagnostics.
- [ ] Full control-flow cleanup, loops/switch/break/continue/goto/finally, exceptions, and broad differential composition coverage.

## Active Roadmap Estimates

| Active item | Primary-integrated | Lane-local candidate maturity | Current read |
| --- | ---: | ---: | --- |
| String conversion, truthiness, byte buffers | 47% | 76% | Primary has shared value string-form semantics, numeric-string classification, selected generated-C string builtin consumers, and comparison byte materialization. Lanes add byte-source/view/result boundaries, tokenizer/parser/string-result execution, and interpreter output byte sinks. |
| Call operation cleanup and ownership | 37% | 59% | Primary routes many call-result contexts through shared blockers. Lanes add callable-signature, sequence, direct-special-form preflight, and recovery contracts, but real frames, binding, by-ref args/returns, dynamic calls, and return ownership remain mostly non-executable. |
| Comparison and conversion semantics | 54% | 68% | Primary has reusable comparison operation validation, branch/free/decision ABIs, direct operand-decision consumers, array-handle comparison consumers, and compare/cast/type-name value-result consumers. Loose array/object/resource/reference comparison, warning order, and LLVM parity remain open. |
| Arrays, lvalues, references, COW | 21% | 69% | Primary has array-key materialization, array value-operation result ABI, array-entry snapshots, array-handle comparisons, and diagnostic array append consumers. Lanes have stronger RMW, `??=`, owner-slot, foreach, reference-operation, and generated-C lvalue candidates; full executable lvalues/references/COW are not integrated. |
| Symbols, globals, request state | 24% | 53% | Primary has symbol ABI helpers, request/superglobal snapshot ABI, and `defined()` interpolation routed through expression-result boundaries. Lanes add slot transition contracts, scalar linked symbol-table execution, and request-state blocker/presence consolidation. Mutable globals/superglobals and repeated-call state remain early. |
| Objects, properties, methods | 10% | 43% | Lane-local object/class/property blockers and operation plans are improving. Primary still has little broad executable object/property/method behavior. |
| Diagnostics and control-flow cleanup | 18% | 58% | Primary has selected severity/blocker surfaces and diagnostic array append behavior. Lanes have richer diagnostic-result carriers, sinks, termination handoffs, and CFG/control-flow rows; most control-flow rows are still non-emitting. |
| Filesystem/path builtins and request state | 16% | 42% | Primary centralizes filesystem/path/request blockers and snapshots. Lanes route more request/configuration/runtime-state builtins through shared blockers, but real stream/stat/cache/current-directory/request mutation behavior is not implemented. |
| Broad composition verification | 21% | 36% | Focused runtime/native-link gates now cover comparison decision paths, value-result consumers, array-handle comparisons, array append diagnostics, and full `native_link` for selected batches. Broad differential PHP composition coverage remains thin, and broad `phpc --tests` still has known pre-existing gaps. |

## Recent Primary-Integrated Work

- `a9823290 native: route array appends through diagnostic ABI`
  - Adds a diagnostic-carrying native array append ABI and routes generated-C array literal appends through it. Runtime/source/linked executable/full `native_link`/`native_array`/cargo-check/rustfmt/diff gates passed. Remaining blockers include spreads, by-reference items, references/COW, exact diagnostics, nontrivial cleanup, and LLVM parity.
- `edbb031b native: route comparison operands to decision ABI`
  - Routes generated-C scalar/string comparison conditions directly through the shared owned operand decision ABI while preserving array branch-result handling. Runtime comparison, dynamic string comparison, linked native comparison, full `native_link`, cargo-check, rustfmt, and diff gates passed.
- `fed5f01c native: route compare cast type-name value results`
  - Extends the native value-operation result path to generated-C loose comparison, scalar cast, and type-name consumers. Non-integral float array keys, arbitrary nested calls, broader LLVM parity, and full conversion warning/recovery remain blocked.
- `811cd281 native: route defined interpolation through expression boundary`
  - Removes the `defined(<interpolated string>)` one-shape rejection so interpolated operands reach the shared expression-result/value-consumer boundary.
- `70872c2e native: centralize comparison branch decisions`
  - Adds `NativeComparisonBranchDecision` and routes generated-C scalar/string comparison operand branches plus array-handle comparison branches through one branch-decision projection.

## Lane-Local Candidate Work Not Yet Counted

- Array/reference candidates: operation-tagged array lvalue value and reference update boundaries, by-reference foreach cursor references, keyed RMW operation boundaries, null-coalescing assignment paths, owner-cell descriptors, reference-cell transfers, and generated-C lvalue consumers.
- String/conversion candidates: byte-source/view/result boundaries, tokenizer/parser/string-result execution, interpreter byte-output sinks, string offset warning blockers, and broader binary-safe string surfaces.
- Symbol/request/call candidates: expression-result sequence consumers, root slot transition contracts, scalar linked symbol-table execution, callable operation/preflight contracts, request/global/superglobal operation boundaries, and call cleanup/diagnostic families.
- Object/control/diagnostic candidates: object-property operation plans, class-policy/object receiver blockers, diagnostic result carriers and sinks, termination handoffs, and non-emitting CFG/control-flow readiness rows.

Lane-local work is useful source material, but it is not product capability until selected into a small primary batch, gated, committed, and pushed.

## Candid Assessment

Recent primary work is directionally sound because it turns shared ABI surfaces into generated-C consumers with focused runtime/native-link tests. The best current pattern remains small and concrete: remove backend-local handling, route through a shared runtime semantic contract, and prove executable behavior.

The project remains boundary-heavy. More result/blocker vocabulary without immediate executable consumers will not move the product much. The next valuable batches should each answer: what PHP behavior now runs that did not run before?

## Near-Term Steering

1. Treat `a9823290` as the latest integrated semantic baseline.
2. Prefer small executable generated-C/LLVM consumers of existing ABI surfaces over more standalone vocabulary.
3. Strong next candidates: narrow array lvalue/reference operation consumers, LLVM parity for comparison or array append diagnostic boundaries, diagnostic-result producers feeding real generated-C reads/offsets, narrow binary string-result execution, or request-state consumers that replace real backend fallbacks.
4. Avoid whole-lane merges; several lanes contain broad, conflict-prone, or non-executable contract work.
5. Keep broad primary gates disk-backed while `/dev/shm` is below roughly 10-12G free.
