# PHP Native Compiler Progress

Updated: 2026-05-22 02:16 CEST
Evaluation marker: 20260522T001600Z-plus-7471b54c

This is a high-level supervisor dashboard. Percentages are candid engineering estimates, not test-suite pass rates. Primary-integrated capability means committed on `master`; lane-local work is candidate material until selected, gated, committed, and pushed.

## Overall Status

Estimated progress toward a broadly usable generalized PHP native compiler: **35%**

```
Generalized runtime/ABI foundations      [#################---] 83%
Compiler/backend consumers               [################----] 81%
Executable generalized PHP semantics     [#########-----------] 46%
Arrays, references, COW, lvalues         [#####---------------] 24%
Objects, properties, methods             [##------------------] 11%
Diagnostics/control-flow composition     [####----------------] 21%
Broad integrated verification            [#######-------------] 35%
```

## Current Primary State

- Product HEAD before this progress update: `7471b54c codegen: lower primitive arithmetic semantics`.
- Latest committed semantic baseline: `7471b54c codegen: lower primitive arithmetic semantics`.
- Latest semantic batch adds a shared primitive arithmetic conversion result consumed by LLVM and generated-C primitive `+`, `-`, `*`, and unary `-` lowering for statically known primitive operands including null, booleans, ints, floats, and numeric strings. Division, modulo, dynamic operands, references/COW, and warning/recovery parity remain explicitly blocked.
- Resource note: `/dev/shm` was tight but usable during the focused gates. Continue using isolated target dirs and low job counts for primary integration because concurrent lane builds remain volatile.

## Grand Roadmap Position

The compiler is steadily replacing backend-local decisions and direct scalar/string handling with reusable runtime/ABI contracts and selected LLVM/generated-C/runtime consumers. Recent primary progress is strongest in primitive arithmetic conversion, comparison relation routing, public operand comparison consumers, recursive-array blocker classification, native object/resource strict-identity relation results, generated-C value output for `echo`/`print`, type predicates, bitwise/shift operations, scalar casts, string byte materialization, array-handle value operands, selected call diagnostics, and focused verification gates.

The product is still far from full generalized PHP. The largest missing regions remain references/COW, executable lvalues, user calls and frames, object/class/property semantics, mutable globals/superglobals, include/require, exceptions/finally, exact diagnostics, and cleanup across real control flow.

## Done / In Progress / Not Done

- [x] Supervised parallel lanes and a primary integration gate are established.
- [x] Shared runtime ABI surfaces exist for strings, byte buffers, comparisons, numeric-string classification, array key/value operations, request-state snapshots, selected conversions, selected filesystem/cache blockers, selected diagnostics, branch-decision status/abort handling, native value bitwise/shift operations, native value type predicates, and runtime string-byte materialization.
- [x] LLVM/generated-C consumers exist for selected primitive arithmetic, selected string builtins, array key/value operations, array-handle value operands, array append diagnostics, comparison relation results, comparison abort guards, strict array/object identity in selected array-search builtins, native value bitwise/shift operations, unary/binary/native value operation output, scalar value echo/print output, print value-result output, native value type predicates, casts/type-name output, selected filesystem/cache blockers, and centralized call diagnostics.
- [x] Reusable array-entry snapshot ABI exists for future foreach/lvalue/reference consumers.
- [ ] In progress: replace shared blockers with executable semantics one family at a time.
- [ ] In progress: generalize value/result/source ownership across returns, call args, conditions, branch joins, stdout, discarded temporaries, comparisons, casts, string-byte materialization, and cleanup.
- [ ] In progress: array lvalue/RMW/reference/COW work. Primary has foundations and selected consumers; lane-local candidates are much stronger than integrated capability.
- [ ] In progress: statement termination and control-flow cleanup candidates. Lane-local work is useful, but broad recursive loop/switch/goto/finally behavior is not integrated.
- [ ] Not done: full symbol environment semantics for locals/imports/globals/superglobals, undefined slots, repeated calls, writes/unset, references, and request/global separation.
- [ ] Not done: function/method frames, argument binding, by-ref args/returns, variadics/spreads, callbacks, constructors, recursion, closures, and dynamic dispatch.
- [ ] Not done: object/class/property semantics including allocation, visibility, magic hooks, `stdClass`, dynamic names, references/COW, and exact diagnostics.
- [ ] Not done: full control-flow cleanup, loops/switch/break/continue/goto/finally, exceptions, shutdown/destructors, and broad differential composition coverage.

## Active Roadmap Estimates

| Active item | Primary-integrated | Lane-local candidate maturity | Current read |
| --- | ---: | ---: | --- |
| String conversion, truthiness, byte buffers | 55% | 84% | Primary has shared value string-form semantics, scalar value output, generated-C print output, comparison byte materialization, runtime string-byte materialization, and raw-buffer writes. Lane-local formatter stdout/byte-buffer work for `var_dump`, `print_r`, `serialize`, and `strlen` is stronger but not counted until integrated. |
| Call operation cleanup and ownership | 43% | 67% | Primary routes many call-result contexts, function declaration fallbacks, and backend call diagnostics through common contracts. Lane-local required-lvalue/discarded-result cleanup is mostly blocker routing; real frames, binding, returns, by-ref semantics, and dispatch remain mostly non-executable. |
| Comparison and conversion semantics | 72% | 79% | Primary has reusable comparison validation, relation-result/result/branch/free/decision/status/abort ABIs, generated-C relation-result consumers, public operand routing, recursive-array blocker classification, string-handle operands, native object/resource strict identity, primitive arithmetic conversion for known operands, scalar casts, bitwise/shift consumers, value-operation output, and type predicates. Division/modulo warning parity, dynamic arithmetic, executable recursive array comparison, object property comparison, resource loose comparison, reference dereference comparison, and backend parity remain open. |
| Arrays, lvalues, references, COW | 24% | 80% | Primary has array-key materialization, array value-operation result ABI, array-entry snapshots, array-handle comparisons, append diagnostics, native array handles as owned value operands, and cloned-literal cleanup. Lane-local current-read/RMW/null-aware/foreach/reference/lvalue candidates are much stronger; full executable lvalues/references/COW are not integrated. |
| Symbols, globals, request state | 24% | 61% | Primary has symbol ABI helpers, request/superglobal snapshot ABI, and selected expression-result boundaries. Lane-local scalar symbol-table execution and reference/COW slot contracts are promising but do not yet provide real generalized locals, frames, globals, imports, or request mutation in primary. |
| Objects, properties, methods | 11% | 48% | Primary has native object handle strict-identity relation results and loose-comparison blockers through the shared comparison path. Lane-local object/class/property blockers and operation plans continue improving, but executable object/property/method behavior is still largely absent. |
| Diagnostics and control-flow cleanup | 21% | 66% | Primary has selected severity/blocker surfaces, diagnostic array append behavior, centralized call diagnostics, call-boundary cleanup routing, and comparison branch abort handling. Lane-local diagnostic-result and CFG/termination cleanup boundaries are broader, but much of it remains blocker/model work. |
| Broad composition verification | 35% | 44% | Focused runtime/native-link gates cover the newest comparison relation-result path, value-operation, scalar echo, print output, bitwise, type-predicate, array-value, string-byte, and diagnostic paths. Broad differential PHP composition coverage remains thin, and broad `phpc --tests` still has known pre-existing gaps. |

## Recent Primary-Integrated Work

- `7471b54c codegen: lower primitive arithmetic semantics`
  - Adds shared primitive arithmetic conversion/result handling and consumes it from LLVM and generated-C lowering for known primitive `+`, `-`, `*`, and unary `-` operands. Focused coverage proves null/bool/int/float/numeric-string arithmetic, mixed numeric primitive arithmetic, overflow/non-finite blockers, division/modulo blockers, and adjacent runtime arithmetic behavior. This is real executable scalar arithmetic progress, not full PHP arithmetic: dynamic operands, references/COW, arrays/objects/resources, exact warning/recovery ordering, and division/modulo parity remain blocked.
- `23df69e6 runtime: route arrays through recursive comparison blockers`
  - Classifies loose array comparisons as a recursive-array comparison blocker through the shared runtime comparison family. Coverage proves the same blocker across direct value comparisons, native array-handle comparisons, owned operand relation results, branch results, and branch decisions. This is a centralized blocker/diagnostic boundary, not executable recursive array comparison.
- `45d48d75 runtime: route native handles through strict identity relations`
  - Routes native object and resource handle strict identity/non-identity through `NativeComparisonRelationResult`. This removes another comparison bypass, but it does not implement object property comparison, resource loose comparison, reference dereference comparison, generated backend object/resource consumers, or full PHP object/resource diagnostics.
- `61abb76d codegen: route print values through native result output`
  - Routes generated-native C `print` output through existing native value result/output helpers and links executable coverage for integers, floats, string bytes including NUL, type-name output, and `strlen()` composition. It does not complete expression-position `print`, dynamic formatter dispatch, LLVM/assembly parity, object/resource/reference formatting, or nontrivial cleanup.
- `68b17030 codegen: share function declaration fallback diagnostics`
  - Routes LLVM and generated-C/native function declaration fallback diagnostics through one helper while preserving static-local precedence. This removes duplicated fallback selection; function declarations still do not execute natively.
- `b1d7d0b9 runtime: route operand comparisons through relation result`
  - Routes public owned comparison operand result, branch, and decision consumers through `NativeComparisonRelationResult` instead of a second direct operand path. Value/array/object/resource/reference consumers, LLVM/assembly parity, recursive array comparison, and object/resource/reference semantics remain open.
- `81b46b66 codegen: route comparisons through relation result`
  - Adds generated-C relation-result branch consumers and focused runtime/native-link coverage. It strengthens comparison result composition, but broad expression storage, recursive arrays, objects/resources/references, exact diagnostics, and LLVM parity remain open.
- `f6c5b0fe codegen: echo scalars through value ABI`
  - Routes generated-C scalar `echo`/`print` output for null, bool, int, and float through native value handles and runtime stdout output instead of backend-local scalar formatting.
- `efeaf043 codegen: route type predicates through value ABI`
  - Adds runtime-backed `is_*` predicate consumption over materializable native value-result expressions. This is a concrete generated-C consumer, not full call-result propagation or native-value storage.

## Current Lane-Local Candidate Work Not Yet Counted

- Array/reference candidates: generated-C owner/value/reference operation wrappers, current-read/RMW/null-aware paths, foreach iterable/reference setup, assignment-expression value results, reference-cell transfers, shared writable path builders, and generated-C lvalue consumers.
- String/conversion candidates: formatter-tag stdout/raw-buffer/text-byte ABIs, formatter expression byte results for `print_r`/`serialize`, `strlen` byte-length consumers, byte-source/view/result boundaries, interpreter byte-output sinks, string offset warning blockers, and broader binary-safe string surfaces.
- Termination/control-flow candidates: normal-resume cleanup plans, statement-level native `exit`/`die` value ABI work, and recursive CFG/effect/cleanup blocker work for loops/switch/goto/break/continue.
- Symbol/request/call candidates: scalar linked symbol-table execution, reference/COW slot contracts, request-derived assignment expression consumers, expression-result sequence consumers, callable operation/preflight/source contracts, direct call frame/argument/return ownership, and executable call cleanup families.
- Object/diagnostic/comparison candidates: object-property operation plans, class-policy/object receiver blockers, diagnostic result carriers and sinks, broader comparison relation-result cleanup, and LLVM/generated-C comparison parity work.

Lane-local work is useful source material, but it is not product capability until selected into a small primary batch, gated, committed, and pushed.

## Candid Assessment

Recent primary work is directionally sound because it turns shared ABI surfaces into generated-C or runtime consumers and removes duplicated backend-local handling. The strongest pattern remains small and concrete: remove a backend-local bypass, route through a shared semantic contract, and prove executable or ABI behavior.

The project remains boundary-heavy. More result/blocker vocabulary without immediate executable consumers will not move the product much. The next valuable batches should each answer: what PHP behavior now runs that did not run before, what shared runtime materialization path now has real consumers, or what backend-local bypass has been removed?

## Near-Term Steering

1. Treat `7471b54c` as the latest integrated semantic baseline; `281a4cca`, `44654896`, and `1d6c8c70` are progress artifacts.
2. Prefer small executable generated-C/LLVM consumers of existing ABI surfaces over more standalone vocabulary.
3. Strong next candidates: isolate one narrow array-lvalue current-read/RMW consumer from `impl-array-linked-exec` or `impl-native-integration-batch`; integrate a formatter byte-buffer backend consumer only if it clearly generalizes value/string output; or select a minimal scalar symbol-table ABI consumer if it avoids linked-C fixture plumbing.
4. Avoid whole-lane merges. Current patch-probe evidence shows broad lane stacks are large and conflict-prone.
5. Require call/control-flow/object/diagnostic candidates to emit or link something real, or to remove a duplicated production blocker, before they receive primary integration time.
6. Keep resource checks explicit before broad gates; `/dev/shm` is tight at this review, and isolated target directories remain preferred to avoid target-dir races.
