# PHP Native Compiler Progress

Updated: 2026-05-21 21:05 CEST
Evaluation marker: 20260521T185117Z
Final refresh: 20260521T190500Z

This is a high-level supervisor dashboard. Percentages are candid engineering estimates, not test-suite pass rates. Primary-integrated capability means committed on `master`; lane-local and dirty-worktree work is candidate material until selected, gated, committed, and pushed.

## Overall Status

Estimated progress toward a broadly usable generalized PHP native compiler: **27%**

```
Generalized runtime/ABI foundations      [##############------] 70%
Compiler/backend consumers               [#############-------] 63%
Executable generalized PHP semantics     [######--------------] 30%
Arrays, references, COW, lvalues         [####----------------] 20%
Objects, properties, methods             [##------------------] 10%
Diagnostics/control-flow composition     [###-----------------] 17%
Broad integrated verification            [####----------------] 19%
```

## Current Primary State

- Primary semantic HEAD at final review check: `811cd281 native: route defined interpolation through expression boundary`.
- Latest primary-integrated semantic commit: `811cd281 native: route defined interpolation through expression boundary`.
- Progress metadata caveat: this `PROGRESS.md` refresh is a management artifact committed separately from semantic compiler progress.
- Review race caveat: the bounded snapshot caught the comparison branch-decision work dirty while the dashboard still said primary was clean. A final recheck found it landed and pushed as `70872c2e`, so it is counted as integrated semantic progress.
- Resource caveat: `/dev/shm` is fluctuating with active lane builds but was around 6-13G free during this refresh; use disk-backed primary targets when broad gates run.

## Grand Roadmap Position

The project has moved from scattered backend rejection paths toward reusable runtime/ABI families and selected generated-C consumers. That is the right direction. The product is still much stronger at centralized blockers, ownership shapes, and diagnostics than at executing arbitrary PHP correctly across references, COW, objects, user calls, request/global state, cleanup, and control flow.

## Done / In Progress / Not Done

- [x] Supervised parallel lanes and primary integration gate established.
- [x] Shared runtime ABI surfaces for strings, comparisons, numeric-string classification, array keys/value operations, request-state snapshots, and selected conversion helpers.
- [x] Generated-C consumers for selected string builtins, array key/value operations, comparison operands/results, strict array/object identity in array-search builtins, and array-handle comparisons.
- [x] Reusable array-entry snapshot ABI for future foreach/lvalue/reference consumers.
- [~] Replace shared blockers with executable semantics one family at a time.
- [~] Generalize value/result ownership across returns, call args, conditions, branch joins, stdout, discarded temporaries, and cleanup.
- [~] Array lvalue/RMW/reference/COW work: primary has foundations; lane-local candidates are stronger than integrated capability.
- [ ] Full symbol environment semantics for locals/imports/globals/superglobals, undefined slots, repeated calls, writes/unset, and request/global separation.
- [ ] Function/method frames, argument binding, by-ref args/returns, variadics/spreads, callbacks, constructors, and dynamic dispatch.
- [ ] Object/class/property semantics including visibility, magic hooks, stdClass, dynamic names, references/COW, and exact diagnostics.
- [ ] Full control-flow cleanup, loops/switch/break/continue/goto/finally, exceptions, and broad differential composition coverage.

## Active Roadmap Estimates

| Active item | Primary-integrated | Candidate maturity | Current read |
| --- | ---: | ---: | --- |
| String conversion, truthiness, byte buffers | 47% | 72% | Primary has shared value string-form semantics, numeric-string classification, selected generated-C string builtin consumers, and comparison byte materialization. Lane-local work adds `PhpStringByteSource`, byte tokenizer/parser/string-result candidates, and stronger binary-byte surfaces. |
| Call operation cleanup and ownership | 37% | 57% | Primary routes many call-result contexts through shared blockers. Lanes have callable-signature and sequence/consumer contracts, but real frames, binding, by-ref args/returns, dynamic calls, and return ownership remain mostly non-executable. |
| Comparison and conversion semantics | 50% | 64% | Primary has reusable comparison operation validation, comparison operands, branch/free ABI, strict array/object identity in selected array-search builtins, generated-C array-handle comparisons, and a centralized comparison branch-decision ABI consumed by generated-C scalar/string and array-handle branch paths. Loose array/object comparison, warning order, LLVM parity, and broader conversion consumers remain open. |
| Arrays, lvalues, references, COW | 20% | 64% | Primary has array-key materialization, array value-operation result ABI, array-entry snapshots, and array-handle comparison consumers. Lanes have stronger RMW, `??=`, owner-slot, foreach, reference-operation, and generated-C lvalue candidates, but full executable lvalues/references/COW are not integrated. |
| Symbols, globals, request state | 24% | 50% | Primary has symbol ABI helpers, request/superglobal snapshot ABI, and now routes `defined()` interpolated-string operands through the shared expression-result boundary instead of a one-shape global-constant rejection. Lanes have expression-result consumer and slot/readiness contracts. Mutable globals/superglobals, writes/unset, repeated calls, references/COW, and exact diagnostics remain early. |
| Objects, properties, methods | 10% | 39% | Lane-local object/class/property dependency boundaries are improving. Primary still has little broad executable object/property/method behavior. |
| Diagnostics and control-flow cleanup | 17% | 52% | Primary has selected severity/blocker surfaces. Lanes have diagnostic/result carriers and termination cleanup models, but exact warning/recovery order and executable cleanup across control flow are still broad blockers. |
| Filesystem/path builtins and request state | 15% | 34% | Primary centralizes filesystem path/state blockers and request-state snapshots. Real stream/stat/cache/current-directory behavior and mutable request/global state are still not implemented. |
| Broad composition verification | 19% | 32% | Focused native-link/runtime gates now include comparison branch-decision, full native-link, runtime comparison, and dynamic string comparison coverage. Broad differential PHP composition coverage remains thin, and broad `phpc --tests` still has a known pre-existing `builtin_exception_class` failure. |

## Recent Primary-Integrated Work

- `811cd281 native: route defined interpolation through expression boundary`
  - Removes the `defined(<interpolated string>)` one-shape gate in both LLVM and generated-C/assembly paths so interpolated operands reach the shared expression-result/value-consumer boundary. This does not add arbitrary `defined()` execution; it replaces a narrow source-shape rejection with the existing generalized blocker path. Focused `native_global_constant_boundary`, cargo check, rustfmt, and diff checks passed.
- `70872c2e native: centralize comparison branch decisions`
  - Adds a reusable runtime `NativeComparisonBranchDecision` ABI and routes generated-C scalar/string comparison operand branches plus array-handle comparison branches through one branch-decision projection for exit/truth handling. Runtime, native-link, dynamic string comparison, cargo-check, rustfmt, and diff gates passed per `primary-integrator.status.md`.
- `238a6303 native: route array handle comparisons through branch ABI`
  - Generated-C array-handle comparisons now call the shared runtime array comparison branch ABI instead of rejecting array-vs-array handles before runtime semantics. Strict array identity/non-identity is executable through linked tests; loose array comparison still centralizes blockers.
- `1f4c2e2f runtime: route array search through comparison contract`
  - `array_keys()`, `in_array()`, and `array_search()` share comparison-operation matching for strict array/object identity. Loose array/object comparison remains blocked.
- `ac2096e2 runtime: add array entry snapshot ABI`
  - Adds array-entry snapshot/key/value/reference helper infrastructure for future foreach, lvalue, and reference/COW consumers. Not yet full foreach or shared mutation execution.
- `cf6f8d21 native: validate comparisons through operation ABI`
  - Routes scalar, native-value, operand, branch, and array comparison helpers through one comparison operation validation path.
- `a93f3bb5 runtime: centralize value string semantics`
  - Shares full-value PHP string-form analysis across echo/scalar/native string conversion consumers.
- `150e3733 native: add array compare branch/free ABI`
  - Adds shared array comparison result/branch/free helpers over the comparison outcome contract.
- `b05ed08b native: add request-state value snapshot ABI`
  - Adds reusable request/superglobal snapshot result/cleanup/probe surfaces, but not mutable superglobal semantics.

## Lane-Local Candidate Work Not Yet Counted

- Array/reference candidates: operation-tagged array item references and by-reference foreach cursor references, keyed RMW operation boundaries, null-coalescing assignment/lvalue result paths, owner-slot classifiers, reference-cell preflight, and generated-C array lvalue consumers.
- String/conversion candidates: public string-byte source boundaries, byte tokenizer/parser/string-result execution, broader binary-safe string surfaces, source-policy conversion result families, and pair-conversion result consolidation.
- Symbol/request/call candidates: expression-result sequence consumers, root write/slot readiness contracts, callable-signature contracts, request/global/superglobal operation boundaries, and call cleanup/diagnostic families.
- Object/control/diagnostic candidates: object-property dependency boundaries, class-policy/object receiver contracts, diagnostic result carriers, and termination/control-flow cleanup models.

Lane-local work is useful source material, but it is not product capability until selected into a small primary batch, gated, committed, and pushed.

## Candid Assessment

Recent primary work is directionally sound because it turns shared ABI surfaces into generated-C consumers with focused runtime/native-link tests. The best current pattern is small and concrete: remove backend-local handling, route through a shared runtime semantic contract, and prove executable behavior.

The project remains boundary-heavy. More result/blocker vocabulary without immediate executable consumers will not move the product much. The next valuable batches should each answer: what PHP behavior now runs that did not run before?

## Near-Term Steering

1. Treat `811cd281` as the latest integrated semantic baseline.
2. Prefer small executable generated-C/LLVM consumers of existing ABI surfaces over more standalone vocabulary.
3. Strong next candidates: array lvalue/reference operation consumers, request-state snapshot generated-C consumers, LLVM parity for comparison branch-decision/array comparison branch-free, value string semantics consumers, narrow string-result execution, or broader composition gates around comparison/array/string/diagnostic families.
4. Keep `/dev/shm` closer to 10-12G free before broad native-link or workspace gates; use disk-backed primary targets while lane builds are active.
