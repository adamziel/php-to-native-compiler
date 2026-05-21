# PHP Native Compiler Progress

Updated: 2026-05-21 22:51 CEST
Evaluation marker: 20260521T205106Z

This is a high-level supervisor dashboard. Percentages are candid engineering estimates, not test-suite pass rates. Primary-integrated capability means committed on `master`; lane-local work is candidate material until selected, gated, committed, and pushed.

## Overall Status

Estimated progress toward a broadly usable generalized PHP native compiler: **30%**

```
Generalized runtime/ABI foundations      [###############-----] 75%
Compiler/backend consumers               [##############------] 70%
Executable generalized PHP semantics     [#######-------------] 36%
Arrays, references, COW, lvalues         [####----------------] 21%
Objects, properties, methods             [##------------------] 10%
Diagnostics/control-flow composition     [####----------------] 19%
Broad integrated verification            [####----------------] 22%
```

## Current Primary State

- Primary semantic HEAD at this review: `e0280cee comparison: share string numeric-pair classifier`.
- Primary worktree status at live check: semantic slice committed; this dashboard is committed separately per the primary progress-reporting exception.
- Latest integrated progress is a small generalized runtime/compiler slice: runtime string comparison plus LLVM/C known-string safety decisions now use the shared `php_strings_use_numeric_comparison(left, right)` boundary instead of a compiler-local first-byte numeric-looking gate.
- Resource caveat: `/dev/shm` is tight at about 6.2G free and 16G used; broad primary gates should continue using disk-backed targets until headroom improves.

## Grand Roadmap Position

The compiler is steadily replacing backend-local rejections and ad hoc generated-C decisions with reusable runtime/ABI contracts and selected executable consumers. Recent primary progress is strongest in comparison routing, value-result materialization, call-boundary cleanup ordering, array append diagnostics, and selected generated-C consumers.

The product is still far from full generalized PHP. The largest missing regions remain references/COW, executable lvalues, user calls and frames, object/class/property semantics, mutable globals/superglobals, include/require, exceptions/finally, exact diagnostics, and cleanup across real control flow.

## Done / In Progress / Not Done

- [x] Supervised parallel lanes and primary integration gate established.
- [x] Shared runtime ABI surfaces for strings, comparisons, numeric-string classification, array keys/value operations, request-state snapshots, selected conversion helpers, selected diagnostic carriers, branch-decision status, and string-handle comparison operands.
- [x] Generated-C consumers for selected string builtins, array key/value operations, array append diagnostics, string-handle comparison operands/results/decisions, strict array/object identity in selected array-search builtins, and array-handle comparisons.
- [x] Reusable array-entry snapshot ABI for future foreach/lvalue/reference consumers.
- [~] Replace shared blockers with executable semantics one family at a time.
- [~] Generalize value/result ownership across returns, call args, conditions, branch joins, stdout, discarded temporaries, comparisons, and cleanup.
- [~] Array lvalue/RMW/reference/COW work: primary has foundations, append diagnostics, and comparison consumers; lane-local candidates are much stronger than integrated capability.
- [ ] Full symbol environment semantics for locals/imports/globals/superglobals, undefined slots, repeated calls, writes/unset, references, and request/global separation.
- [ ] Function/method frames, argument binding, by-ref args/returns, variadics/spreads, callbacks, constructors, and dynamic dispatch.
- [ ] Object/class/property semantics including visibility, magic hooks, stdClass, dynamic names, references/COW, and exact diagnostics.
- [ ] Full control-flow cleanup, loops/switch/break/continue/goto/finally, exceptions, and broad differential composition coverage.

## Active Roadmap Estimates

| Active item | Primary-integrated | Lane-local candidate maturity | Current read |
| --- | ---: | ---: | --- |
| String conversion, truthiness, byte buffers | 49% | 77% | Primary has shared value string-form semantics, numeric-string classification, selected generated-C string builtin consumers, cast/type-name echo value-result consumers, and comparison byte materialization. Lanes add formatter-tag, byte-source/view/result, tokenizer/parser/string-result, and interpreter output byte-sink candidates. |
| Call operation cleanup and ownership | 40% | 60% | Primary routes many call-result contexts, termination-construct argument expressions, and direct special-form argument/arity failures through shared call-boundary blockers. Lanes add callable-signature, sequence, direct-special-form preflight, and recovery contracts, but real frames, binding, by-ref args/returns, dynamic calls, and return ownership remain mostly non-executable. |
| Comparison and conversion semantics | 59% | 70% | Primary has reusable comparison operation validation, branch/free/decision/status ABIs, direct operand-decision consumers, generated-C status guards, generated-C static/dynamic string-handle comparison operands, shared numeric-string pair classification for runtime and LLVM/C known-string safety, materialized value/diagnostic operand comparison entry points, array-handle comparison consumers, compare/cast/type-name value-result consumers, generated-C cast/type-name echo consumers, and operand-side/value-family/operation-aware blockers. Loose array/object/resource/reference execution, warning order, arbitrary expression materialization, and broader LLVM/generated-C parity remain open. |
| Arrays, lvalues, references, COW | 21% | 70% | Primary has array-key materialization, array value-operation result ABI, array-entry snapshots, array-handle comparisons, and diagnostic array append consumers. Lanes have stronger RMW, `??=`, owner-slot, foreach, reference-operation, and generated-C lvalue candidates; full executable lvalues/references/COW are not integrated. |
| Symbols, globals, request state | 24% | 54% | Primary has symbol ABI helpers, request/superglobal snapshot ABI, and `defined()` interpolation routed through expression-result boundaries. Lanes add expression-result consumer consolidation, slot transition contracts, scalar linked symbol-table execution, and request-state blocker/presence consolidation. Mutable globals/superglobals and repeated-call state remain early. |
| Objects, properties, methods | 10% | 44% | Lane-local object/class/property blockers and operation plans are improving. Primary still has little broad executable object/property/method behavior. |
| Diagnostics and control-flow cleanup | 19% | 59% | Primary has selected severity/blocker surfaces, diagnostic array append behavior, call-boundary cleanup routing, and comparison status/exit handling. Lanes have richer diagnostic-result carriers, sinks, termination handoffs, and CFG/control-flow rows; most control-flow rows are still non-emitting. |
| Filesystem/path builtins and request state | 16% | 42% | Primary centralizes filesystem/path/request blockers and snapshots. Lanes route more request/configuration/runtime-state builtins through shared blockers, but real stream/stat/cache/current-directory/request mutation behavior is not implemented. |
| Broad composition verification | 22% | 37% | Focused runtime/native-link gates now cover comparison decision/status paths, value-result consumers, array-handle comparisons, array append diagnostics, and full `native_link` filters for selected batches. Broad differential PHP composition coverage remains thin, and broad `phpc --tests` still has known pre-existing gaps. |

## Recent Primary-Integrated Work

- `e0280cee comparison: share string numeric-pair classifier`
  - Adds shared `php_strings_use_numeric_comparison(left, right)` over the runtime numeric-string classifier, routes runtime string comparison through it, and replaces LLVM/C known-string native comparison safety checks with pairwise PHP semantics instead of a local first-byte recognizer. Focused runtime, LLVM boundary, native-link comparison, cargo-check, rustfmt, and diff gates passed. This removes a backend-local blocker decision; arbitrary dynamic expression comparison, binary/non-UTF-8 ownership, references/COW, and object/resource semantics remain blocked.
- `315c03c3 codegen: route string comparisons through handles`
  - Wires the new string-handle comparison operand ABI into generated-C static and dynamic string comparison operands. Source and linked executable tests prove dynamic binary strings keep tracked byte lengths and feed the shared operand-decision ABI through `phpc_native_string_from_bytes(...)` and `phpc_native_comparison_operand_from_string_and_free(...)`, rather than bypassing via raw byte operand construction.
- `fb99b703 runtime: compare native string handles as operands`
  - Adds borrowed and owned `NativeStringHandle` comparison operand ABI entry points and routes raw string-byte operands through the same value/diagnostic materialization helper. Focused runtime tests cover numeric-string ordering, strict identity, binary string ordering, empty-string equality, null handles, invalid UTF-8 diagnostics, and the broader runtime comparison suite plus package check, rustfmt, and diff gates.
- `327e1770 native: route comparison guards through decision status`
  - Adds `phpc_native_comparison_branch_decision_status(...)` and routes generated executable C scalar/string and array comparison guards through that shared status ABI before consuming exit-code and truth accessors. Focused runtime, generated-C source, native-link comparison, cargo-check, rustfmt, and diff gates passed. This improves comparison status/cleanup consistency; it is not loose array/object/resource/reference comparison execution.
- `1dee564f codegen: echo cast value results through ABI`
  - Routes generated-C echo of cast expressions and `gettype()` through the existing `NativeValueOperationResult` materialization, `phpc_native_value_echo_stdout`, and cleanup path instead of hand-formatted cast output. Focused generated-C source and linked executable tests cover string, int, float, bool casts and type-name composition, with runtime value-result and comparison source-contract gates plus rustfmt/diff checks passing.
- `72298edf codegen: route special forms through call boundary`
  - Routes `defined()`, `isset()`, and `empty()` argument/arity failures and nested call-shaped operands through the shared direct-call argument cleanup blocker in both LLVM and generated-C codegen. Focused direct-special-form call-boundary tests, the shared direct-call argument cleanup unit test, cargo check, rustfmt check, and diff checks passed. This is blocker/cleanup routing only; it does not add executable native call or special-form semantics.
- `ba67c7e9 codegen: route termination args through call boundary`
  - Removes a backend-local termination-argument bypass by routing `exit()`/`die()` argument expressions through the shared direct-named call argument boundary in both LLVM and generated-C codegen. Focused direct/dynamic/method/constructor-shaped call-argument boundary tests, the shared direct-call argument cleanup unit test, cargo check, rustfmt check, and diff checks passed. Termination execution remains blocked.
- `b9ae1632 runtime: compare materialized value pairs`
  - Adds runtime ABI wrappers for already-materialized `NativeValueHandle` plus `NativeDiagnosticHandle` comparison operands and routes them through the shared `NativeComparisonOperand` result, branch, operation-branch, decision, and operation-decision contracts. Focused materialized-comparison tests, neighboring comparison/materialization/decision tests, runtime comparison suite, package check, rustfmt, and diff checks passed.
- `09f97b8f runtime: track comparison blocker operand sides`
  - Replaces coarse array/object/resource comparison blockers with operand-side, value-family, and operation-family-aware blockers across loose comparison families. Focused runtime comparison blocker tests, runtime comparison suite, native-link comparison tests, package check, rustfmt, and diff checks passed. This improves generalized diagnostics/blocker routing across comparison families; it is still not loose array/object/resource/reference comparison execution.
- `a9823290 native: route array appends through diagnostic ABI`
  - Adds a diagnostic-carrying native array append ABI and routes generated-C array literal appends through it. Runtime/source/linked executable/full `native_link`/`native_array`/cargo-check/rustfmt/diff gates passed. Remaining blockers include spreads, by-reference items, references/COW, exact diagnostics, nontrivial cleanup, and LLVM parity.
- `edbb031b native: route comparison operands to decision ABI`
  - Routes generated-C scalar/string comparison conditions directly through the shared owned operand decision ABI while preserving array branch-result handling. Runtime comparison, dynamic string comparison, linked native comparison, full `native_link`, cargo-check, rustfmt, and diff gates passed.
- `fed5f01c native: route compare cast type-name value results`
  - Extends the native value-operation result path to generated-C loose comparison, scalar cast, and type-name consumers. Non-integral float array keys, arbitrary nested calls, broader LLVM parity, and full conversion warning/recovery remain blocked.

## Lane-Local Candidate Work Not Yet Counted

- Array/reference candidates: operation-tagged array lvalue value and reference update boundaries, by-reference foreach cursor references, keyed RMW operation boundaries, null-coalescing assignment paths, owner-cell descriptors, reference-cell transfers, generated-C lvalue consumers, and one-descriptor mutation update emitters.
- String/conversion candidates: formatter-tag byte ABI, byte-source/view/result boundaries, tokenizer/parser/string-result execution, interpreter byte-output sinks, string offset warning blockers, and broader binary-safe string surfaces.
- Symbol/request/call candidates: expression-result sequence consumers, root slot transition contracts, scalar linked symbol-table execution, callable operation/preflight contracts, request/global/superglobal operation boundaries, and call cleanup/diagnostic families.
- Object/control/diagnostic candidates: object-property operation plans, class-policy/object receiver blockers, diagnostic result carriers and sinks, termination handoffs, and non-emitting CFG/control-flow readiness rows.

Lane-local work is useful source material, but it is not product capability until selected into a small primary batch, gated, committed, and pushed.

## Candid Assessment

Recent primary work is directionally sound because it turns shared ABI surfaces into generated-C consumers with focused runtime/native-link tests. The best current pattern remains small and concrete: remove backend-local handling, route through a shared runtime semantic contract, and prove executable behavior.

The project remains boundary-heavy. More result/blocker vocabulary without immediate executable consumers will not move the product much. The next valuable batches should each answer: what PHP behavior now runs that did not run before, or what real backend-local bypass has been removed?

## Near-Term Steering

1. Treat `e0280cee` as the latest integrated semantic baseline.
2. Prefer small executable generated-C/LLVM consumers of existing ABI surfaces over more standalone vocabulary.
3. Strong next candidates: narrow array lvalue/reference operation consumers, LLVM parity for comparison or array append diagnostic boundaries, diagnostic-result producers feeding real generated-C reads/offsets, narrow formatter/string-result execution, or request-state consumers that replace real backend fallbacks.
4. Avoid whole-lane merges; several lanes contain broad, conflict-prone, or non-executable contract work.
5. Keep broad primary gates disk-backed while `/dev/shm` is below roughly 10-12G free.
