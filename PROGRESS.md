# PHP Native Compiler Progress

Updated: 2026-05-21 23:29 CEST
Evaluation marker: 20260521T212433Z-plus-d3287ff6

This is a high-level supervisor dashboard. Percentages are candid engineering estimates, not test-suite pass rates. Primary-integrated capability means committed on `master`; lane-local work is candidate material until selected, gated, committed, and pushed.

## Overall Status

Estimated progress toward a broadly usable generalized PHP native compiler: **30%**

```
Generalized runtime/ABI foundations      [###############-----] 75%
Compiler/backend consumers               [##############------] 71%
Executable generalized PHP semantics     [#######-------------] 36%
Arrays, references, COW, lvalues         [####----------------] 21%
Objects, properties, methods             [##------------------] 10%
Diagnostics/control-flow composition     [####----------------] 20%
Broad integrated verification            [#####---------------] 23%
```

## Current Primary State

- Product HEAD at this review: `d3287ff6 codegen: consume scalar value cast ABI`, synced with `origin/master`.
- Latest committed semantic baseline: `d3287ff6 codegen: consume scalar value cast ABI`.
- Live worktree status: clean and synced after the scalar value-cast ABI consumer commit.
- Resource note: `/dev/shm` is healthy at about 15G free and 7.6G used; `/home` has about 252G free. Prior windows still hit severe `/dev/shm` pressure, so broad primary gates should remain isolated or disk-backed when headroom falls.

## Grand Roadmap Position

The compiler is steadily replacing backend-local rejections and ad hoc generated-C decisions with reusable runtime/ABI contracts and selected executable consumers. Recent primary progress is strongest in comparison routing, string-handle comparison operands, numeric-string classifier sharing, value-result materialization, scalar value-cast ABI consumers, call-boundary cleanup ordering, and centralized native call diagnostics.

The product is still far from full generalized PHP. The largest missing regions remain references/COW, executable lvalues, user calls and frames, object/class/property semantics, mutable globals/superglobals, include/require, exceptions/finally, exact diagnostics, and cleanup across real control flow.

## Done / In Progress / Not Done

- [x] Supervised parallel lanes and primary integration gate established.
- [x] Shared runtime ABI surfaces for strings, comparisons, numeric-string classification, array keys/value operations, request-state snapshots, selected conversion helpers, selected diagnostic carriers, branch-decision status, and string-handle comparison operands.
- [x] Generated-C consumers for selected string builtins, array key/value operations, array append diagnostics, string-handle comparison operands/results/decisions, strict array/object identity in selected array-search builtins, array-handle comparisons, comparison status guards, cast/type-name echo value results, scalar value-cast operations, and centralized call diagnostics.
- [x] Reusable array-entry snapshot ABI for future foreach/lvalue/reference consumers.
- [ ] In progress: replace shared blockers with executable semantics one family at a time.
- [ ] In progress: generalize value/result ownership across returns, call args, conditions, branch joins, stdout, discarded temporaries, comparisons, casts, and cleanup.
- [ ] In progress: array lvalue/RMW/reference/COW work. Primary has foundations, append diagnostics, and comparison consumers; lane-local candidates are much stronger than integrated capability.
- [ ] Not done: full symbol environment semantics for locals/imports/globals/superglobals, undefined slots, repeated calls, writes/unset, references, and request/global separation.
- [ ] Not done: function/method frames, argument binding, by-ref args/returns, variadics/spreads, callbacks, constructors, and dynamic dispatch.
- [ ] Not done: object/class/property semantics including visibility, magic hooks, stdClass, dynamic names, references/COW, and exact diagnostics.
- [ ] Not done: full control-flow cleanup, loops/switch/break/continue/goto/finally, exceptions, and broad differential composition coverage.

## Active Roadmap Estimates

| Active item | Primary-integrated | Lane-local candidate maturity | Current read |
| --- | ---: | ---: | --- |
| String conversion, truthiness, byte buffers | 50% | 78% | Primary has shared value string-form semantics, numeric-string classification, selected generated-C string builtin consumers, cast/type-name echo value-result consumers, scalar value-cast generated-C consumers, and comparison byte materialization. |
| Call operation cleanup and ownership | 42% | 64% | Primary routes many call-result contexts, termination-construct argument expressions, direct special-form argument/arity failures, and shared backend call diagnostics through common call-boundary contracts. Lanes add callable-signature, sequence, direct-special-form preflight, termination/user-function blockers, and recovery contracts, but real frames, binding, by-ref args/returns, dynamic calls, and return execution remain mostly non-executable. |
| Comparison and conversion semantics | 60% | 72% | Primary has reusable comparison operation validation, branch/free/decision/status ABIs, generated-C status guards, string-handle comparison operands, shared numeric-string pair classification, materialized value comparison entry points, array-handle comparison consumers, compare/cast/type-name value-result consumers, scalar value-cast ABI consumers, and operand-side/value-family blockers. Loose array/object/resource/reference execution, warning order, arbitrary expression materialization, and broader LLVM/generated-C parity remain open. |
| Arrays, lvalues, references, COW | 21% | 73% | Primary has array-key materialization, array value-operation result ABI, array-entry snapshots, array-handle comparisons, and diagnostic array append consumers. Lanes have stronger owner-root, foreach, assignment-statement/value-result, RMW, `??=`, reference-operation, and generated-C lvalue candidates; full executable lvalues/references/COW are not integrated. |
| Symbols, globals, request state | 24% | 55% | Primary has symbol ABI helpers, request/superglobal snapshot ABI, and `defined()` interpolation routed through expression-result boundaries. Lanes add expression-result consumer consolidation, slot transition contracts, scalar linked symbol-table execution, and request-state blocker/presence consolidation. Mutable globals/superglobals and repeated-call state remain early. |
| Objects, properties, methods | 10% | 45% | Lane-local object/class/property blockers and operation plans are improving. Primary still has little broad executable object/property/method behavior. |
| Diagnostics and control-flow cleanup | 20% | 60% | Primary has selected severity/blocker surfaces, diagnostic array append behavior, centralized call diagnostic subjects, call-boundary cleanup routing, and comparison status/exit handling. Lanes have richer diagnostic-result carriers, sinks, termination handoffs, and CFG/control-flow rows; most control-flow rows are still non-emitting. |
| Filesystem/path builtins and request state | 16% | 43% | Primary centralizes filesystem/path/request blockers and snapshots. Lanes route more request/configuration/runtime-state builtins through shared blockers, but real stream/stat/cache/current-directory/request mutation behavior is not implemented. |
| Broad composition verification | 23% | 39% | Focused runtime/native-link gates cover comparison decision/status paths, value-result consumers, scalar value-cast consumers, array-handle comparisons, array append diagnostics, call diagnostics, and selected full `native_link` batches. Broad differential PHP composition coverage remains thin, and broad `phpc --tests` still has known pre-existing gaps. |

## Recent Primary-Integrated Work

- `d3287ff6 codegen: consume scalar value cast ABI`
  - Routes scalar cast operations through the shared native value-cast ABI across runtime, generated-C lowering, native link coverage, ABI probes, scalar-cast tests, and cast boundary fixtures. This converts the previously in-flight scalar-cast builtin/value-cast diagnostics slice into committed primary capability. It does not complete arbitrary expression materialization, object/resource/Stringable conversion hooks, reference/COW behavior, full warning/recovery ordering, or broad LLVM parity.
- `325c9c7d codegen: centralize native call diagnostics`
  - Adds shared `NativeCallDiagnostics` and `NativeCallDiagnosticSubject` handling for existing native call diagnostics across direct, dynamic, method, constructor, closure, function-frame handoff, return-value ownership, and explicit call-operation blockers. Generated-C echo recovery now continues far enough to report later call-family operand blockers instead of stopping at an earlier local echo-consumer blocker. This removes duplicated backend-local diagnostic construction; it does not add callable lookup, real frames, argument binding, by-ref/variadic ownership, closure/object dispatch, or return execution.
- `cc50fb91 tests: prove string comparison classifier consumers`
  - Adds proof coverage that the shared numeric-string pair classifier is consumed by LLVM, generated-C fallback, runtime comparison, and linked executable behavior. This is verification-only: no production lowering or string-pair special case was added.
- `e0280cee comparison: share string numeric-pair classifier`
  - Adds shared `php_strings_use_numeric_comparison(left, right)` over the runtime numeric-string classifier, routes runtime string comparison through it, and replaces LLVM/C known-string native comparison safety checks with pairwise PHP semantics instead of a local first-byte recognizer. Focused runtime, LLVM boundary, native-link comparison, cargo-check, rustfmt, and diff gates passed. This removes a backend-local blocker decision; arbitrary dynamic expression comparison, binary/non-UTF-8 ownership, references/COW, and object/resource semantics remain blocked.
- `315c03c3 codegen: route string comparisons through handles`
  - Wires the string-handle comparison operand ABI into generated-C static and dynamic string comparison operands. Source and linked executable tests prove dynamic binary strings keep tracked byte lengths and feed the shared operand-decision ABI rather than bypassing via raw byte operand construction.
- `fb99b703 runtime: compare native string handles as operands`
  - Adds borrowed and owned `NativeStringHandle` comparison operand ABI entry points and routes raw string-byte operands through the same value/diagnostic materialization helper. Focused runtime tests cover numeric-string ordering, strict identity, binary string ordering, empty-string equality, null handles, invalid UTF-8 diagnostics, and the broader runtime comparison suite plus package check, rustfmt, and diff gates.
- `327e1770 native: route comparison guards through decision status`
  - Adds `phpc_native_comparison_branch_decision_status(...)` and routes generated executable C scalar/string and array comparison guards through that shared status ABI before consuming exit-code and truth accessors. This improves comparison status/cleanup consistency; it is not loose array/object/resource/reference comparison execution.
- `1dee564f codegen: echo cast value results through ABI`
  - Routes generated-C echo of cast expressions and `gettype()` through the existing `NativeValueOperationResult` materialization, `phpc_native_value_echo_stdout`, and cleanup path instead of hand-formatted cast output.
- `72298edf codegen: route special forms through call boundary`
  - Routes `defined()`, `isset()`, and `empty()` argument/arity failures and nested call-shaped operands through the shared direct-call argument cleanup blocker in both LLVM and generated-C codegen. This is blocker/cleanup routing only; it does not add executable native call or special-form semantics.
- `ba67c7e9 codegen: route termination args through call boundary`
  - Removes a backend-local termination-argument bypass by routing `exit()`/`die()` argument expressions through the shared direct-named call argument boundary in both LLVM and generated-C codegen. Termination execution remains blocked.
- `b9ae1632 runtime: compare materialized value pairs`
  - Adds runtime ABI wrappers for already-materialized `NativeValueHandle` plus `NativeDiagnosticHandle` comparison operands and routes them through the shared comparison operand/result/branch/decision contracts.
- `09f97b8f runtime: track comparison blocker operand sides`
  - Replaces coarse array/object/resource comparison blockers with operand-side, value-family, and operation-family-aware blockers across loose comparison families. This improves diagnostics/blocker routing but is still not loose array/object/resource/reference comparison execution.
- `a9823290 native: route array appends through diagnostic ABI`
  - Adds a diagnostic-carrying native array append ABI and routes generated-C array literal appends through it. Remaining blockers include spreads, by-reference items, references/COW, exact diagnostics, nontrivial cleanup, and LLVM parity.

## Current Lane-Local Candidate Work Not Yet Counted

- Array/reference candidates: shared generated-C owner-root builder, foreach iterable setup, assignment-statement update tag, operation-tagged array lvalue value/reference update boundaries, keyed RMW operation boundaries, null-coalescing assignment paths, owner-cell descriptors, reference-cell transfers, and generated-C lvalue consumers.
- String/conversion candidates: formatter-tag byte ABI, byte-source/view/result boundaries, tokenizer/parser/string-result execution, interpreter byte-output sinks, scalar-cast builtin routing, string offset warning blockers, and broader binary-safe string surfaces.
- Symbol/request/call candidates: expression-result sequence consumers, root slot transition contracts, scalar linked symbol-table execution, callable operation/preflight contracts, request/global/superglobal operation boundaries, user-function/termination blockers, and call cleanup/diagnostic families.
- Object/control/diagnostic candidates: object-property operation plans, class-policy/object receiver blockers, diagnostic result carriers and sinks, termination handoffs, and non-emitting CFG/control-flow readiness rows.

Lane-local work is useful source material, but it is not product capability until selected into a small primary batch, gated, committed, and pushed.

## Candid Assessment

Recent primary work is directionally sound because it turns shared ABI surfaces into generated-C or LLVM-visible consumers with focused runtime/native-link tests. The strongest pattern remains small and concrete: remove backend-local handling, route through a shared runtime semantic contract, and prove executable behavior.

The project remains boundary-heavy. More result/blocker vocabulary without immediate executable consumers will not move the product much. The next valuable batches should each answer: what PHP behavior now runs that did not run before, or what real backend-local bypass has been removed?

## Near-Term Steering

1. Treat `d3287ff6` as the latest integrated semantic baseline.
2. Prefer small executable generated-C/LLVM consumers of existing ABI surfaces over more standalone vocabulary.
3. Strong next candidates: isolate one narrow array-lvalue consumer from `impl-array-linked-exec`; add LLVM parity for comparison or array append diagnostic boundaries; connect diagnostic/result producers to real generated-C reads/offsets; or consume another existing conversion/comparison ABI in a backend path.
4. Avoid whole-lane merges. Several lanes contain broad, conflict-prone, or non-executable contract work.
5. Be skeptical of scalar/local linked-symbol helper paths unless they connect to generalized frame/symbol environment semantics.
6. Keep resource checks explicit before broad gates; `/dev/shm` is healthy at this review, but isolated target directories remain preferred for primary integration to avoid target-dir races.
