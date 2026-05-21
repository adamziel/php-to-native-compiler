# PHP Native Compiler Progress

Updated: 2026-05-22 01:26 CEST
Evaluation marker: 20260521T232600Z-plus-b1d7d0b9

This is a high-level supervisor dashboard. Percentages are candid engineering estimates, not test-suite pass rates. Primary-integrated capability means committed on `master`; lane-local work is candidate material until selected, gated, committed, and pushed.

## Overall Status

Estimated progress toward a broadly usable generalized PHP native compiler: **34%**

```
Generalized runtime/ABI foundations      [################----] 82%
Compiler/backend consumers               [################----] 80%
Executable generalized PHP semantics     [#########-----------] 45%
Arrays, references, COW, lvalues         [#####---------------] 24%
Objects, properties, methods             [##------------------] 10%
Diagnostics/control-flow composition     [####----------------] 21%
Broad integrated verification            [#######-------------] 35%
```

## Current Primary State

- Product HEAD before this progress update: `b1d7d0b9 runtime: route operand comparisons through relation result`.
- Latest committed semantic baseline: `b1d7d0b9 runtime: route operand comparisons through relation result`.
- Latest semantic batch routes public owned comparison operand result, branch, and decision consumers through the existing relation-result boundary, replacing the remaining direct operand-to-result path for scalar, string, array-blocker, materialization-failure, and invalid-opcode families.
- Resource note: live `/dev/shm` is under pressure at about 6.7G free. The largest target dir is active under `impl-native-error-diagnostic-semantics`, so broad primary gates should remain isolated, single-job, and resource-aware until that build wave settles.

## Grand Roadmap Position

The compiler is steadily replacing backend-local decisions and local byte/value handling with reusable runtime/ABI contracts and selected executable consumers. Recent primary progress is strongest in comparison routing, public operand comparison consumer routing, string-handle comparison operands, nested comparison decision rematerialization, comparison branch-decision abort handling, native value bitwise/shift operations, echoing unary/binary/native value operations and direct scalar output through runtime value ABIs, runtime-backed `is_*` type predicates over native value handles, numeric-string classifier sharing, value-result materialization, scalar value-cast ABI consumers, array-handle value operands and cleanup, selected filesystem/cache ABI routing, call-boundary diagnostics, shared runtime string-byte source materialization, and shared raw-buffer writes for PHP string sources.

The product is still far from full generalized PHP. The largest missing regions remain references/COW, executable lvalues, user calls and frames, object/class/property semantics, mutable globals/superglobals, include/require, exceptions/finally, exact diagnostics, and cleanup across real control flow.

## Done / In Progress / Not Done

- [x] Supervised parallel lanes and primary integration gate established.
- [x] Shared runtime ABI surfaces for strings, raw byte-buffer writes, comparisons, numeric-string classification, array keys/value operations, request-state snapshots, selected conversion helpers, selected filesystem/cache operation blockers, selected diagnostic carriers, branch-decision status/abort handling, native value bitwise/shift operations, native value type predicates, string-handle comparison operands, and runtime string-byte source materialization.
- [x] Generated-C consumers for selected string builtins, array key/value operations, array-handle value operands, array append diagnostics, string-handle comparison operands/results/decisions, nested comparison decision operands, strict array/object identity in selected array-search builtins, array-handle comparisons, comparison abort guards, native value bitwise/shift operations, unary/binary/native value operation echo paths, direct scalar value echo/print output, native value type-predicate builtins, cast/type-name echo value results, scalar value-cast operations, selected realpath-cache filesystem operation blockers, and centralized call diagnostics.
- [x] Reusable array-entry snapshot ABI for future foreach/lvalue/reference consumers.
- [ ] In progress: replace shared blockers with executable semantics one family at a time.
- [ ] In progress: generalize value/result/source ownership across returns, call args, conditions, branch joins, stdout, discarded temporaries, comparisons, casts, string-byte materialization, and cleanup.
- [ ] In progress: array lvalue/RMW/reference/COW work. Primary has foundations, append diagnostics, comparison consumers, array-handle value operands, and cloned-literal cleanup; lane-local candidates are much stronger than integrated capability.
- [ ] In progress: narrow termination/control-flow candidates. Lane-local statement-level `exit`/`die` value ABI work looks useful; broad recursive branch/loop cleanup is still not integrated.
- [ ] Not done: full symbol environment semantics for locals/imports/globals/superglobals, undefined slots, repeated calls, writes/unset, references, and request/global separation.
- [ ] Not done: function/method frames, argument binding, by-ref args/returns, variadics/spreads, callbacks, constructors, and dynamic dispatch.
- [ ] Not done: object/class/property semantics including visibility, magic hooks, stdClass, dynamic names, references/COW, and exact diagnostics.
- [ ] Not done: full control-flow cleanup, loops/switch/break/continue/goto/finally, exceptions, and broad differential composition coverage.

## Active Roadmap Estimates

| Active item | Primary-integrated | Lane-local candidate maturity | Current read |
| --- | ---: | ---: | --- |
| String conversion, truthiness, byte buffers | 54% | 82% | Primary has shared value string-form semantics, numeric-string classification, selected generated-C string builtin consumers, cast/type-name echo value-result consumers, scalar value-cast generated-C consumers, scalar-to-native-value echo/print output, comparison byte materialization, shared runtime string-byte source/materialization, and shared raw-buffer writes. Lane-local formatter/stdout work is stronger but not integrated. |
| Call operation cleanup and ownership | 42% | 66% | Primary routes many call-result contexts and shared backend call diagnostics through common call-boundary contracts. Lane-local work is mostly blocker/diagnostic cleanup; the latest reference-assignment pass explicitly adds no executable call behavior. Real frames, binding, by-ref args/returns, dynamic calls, and return execution remain mostly non-executable. |
| Comparison and conversion semantics | 69% | 78% | Primary has reusable comparison operation validation, relation-result/result/branch/free/decision/status/abort ABIs, public owned operand result/branch/decision consumers routed through relation results, generated-C relation-result branch consumers, generated-C abort-code guards, string-handle operands, numeric-string pair classification, comparison decision rematerialization, array-handle comparison consumers, scalar casts, bitwise/shift ABI consumers, value-operation echo, and type-predicate consumers. Recursive array/object/resource/reference comparison semantics and broad backend parity remain open. |
| Arrays, lvalues, references, COW | 24% | 78% | Primary has array-key materialization, array value-operation result ABI, array-entry snapshots, array-handle comparisons, diagnostic append consumers, native array handles as owned value operands, and cloned-literal source cleanup. Lane-local owner/value/reference operation wrappers, null-coalescing value operands, foreach/reference paths, and generated-C lvalue candidates are much stronger; full executable lvalues/references/COW are not integrated. |
| Symbols, globals, request state | 24% | 58% | Primary has symbol ABI helpers, request/superglobal snapshot ABI, and `defined()` interpolation routed through expression-result boundaries. Lane-local request assignment expression work preserves derived request-native values, but mutable globals/superglobals and repeated-call state remain early. |
| Objects, properties, methods | 10% | 46% | Lane-local object/class/property blockers and operation plans continue improving. Primary still has little broad executable object/property/method behavior. |
| Diagnostics and control-flow cleanup | 21% | 64% | Primary has selected severity/blocker surfaces, diagnostic array append behavior, centralized call diagnostic subjects, call-boundary cleanup routing, and comparison branch abort handling. Lane-local diagnostic result carriers and control-flow rejection boundaries are broader, but most control-flow remains blocker or non-emitting model work. |
| Filesystem/path builtins and request state | 18% | 43% | Primary centralizes filesystem/path/request blockers and snapshots, and routes `realpath_cache_get()`/`realpath_cache_size()` through the shared filesystem path/cache operation ABI. Real stream/stat/cache/current-directory/request mutation behavior is not implemented. |
| Broad composition verification | 35% | 43% | Focused runtime/native-link gates cover the newest comparison relation-result path, value-operation, scalar echo, bitwise, type-predicate, array-value, string-byte, and diagnostic paths. Broad differential PHP composition coverage remains thin, and broad `phpc --tests` still has known pre-existing gaps. |

## Recent Primary-Integrated Work

- `b1d7d0b9 runtime: route operand comparisons through relation result`
  - Routes public owned comparison operand result, branch, and decision consumers through the existing `NativeComparisonRelationResult` path instead of maintaining a second direct operand result/materialization path. Runtime coverage proves matching relation-result behavior across loose equality, loose ordering, strict non-identity, array comparison blockers, string materialization blockers, and invalid opcodes; focused runtime comparison filters, package check, native-link comparison filter, rustfmt check, and diff checks passed. This removes one runtime bypass, but value/array/object/resource/reference public comparison consumers, LLVM/assembly parity, recursive array comparison, and object/resource/reference semantics remain open.
- `81b46b66 codegen: route comparisons through relation result`
  - Adds `NativeComparisonRelationResult` and runtime helpers for owned operand comparison relation results, then routes runtime-linked generated-C comparison branch consumers through relation-result-to-decision/reporting helpers instead of the older direct decision ABI. Gates included focused runtime relation-result coverage, generated-C source assertions, dynamic string operand source coverage, native-link comparison subset, full `native_link`, `php_runtime native_comparison`, `phpc --lib comparison`, `php_runtime comparison`, package check, rustfmt check, and diff checks. This strengthens comparison result composition and backend consumer discipline, but recursive array/object/resource/reference comparisons, exact diagnostics, LLVM parity, and broad expression storage remain open.
- `f6c5b0fe codegen: echo scalars through value ABI`
  - Routes generated-native C scalar `echo`/`print` output for null, bool, int, and float values through `phpc_native_value_from_scalar(...)` and `phpc_native_value_echo_stdout(...)` instead of backend-local C `printf` scalar formatting. Gates included source-level generated-C assertions, a linked executable scalar output program, runtime scalar-to-value echo coverage, full `native_link`, runtime `native_value_`, `cargo check -q -p phpc -p php_runtime`, rustfmt checks, and diff checks. This is a narrow generated-C consumer of an existing value ABI; LLVM scalar stdout, arbitrary value storage, references/COW, object/resource formatting, exact diagnostics, and nontrivial cleanup remain open.
- `efeaf043 codegen: route type predicates through value ABI`
  - Adds runtime `phpc_native_value_type_predicate(...)` for PHP `is_*` predicate families over borrowed native value handles. Generated-C `is_null`, `is_bool`, int/float aliases, `is_string`, `is_array`, `is_scalar`, `is_numeric`, `is_countable`, `is_iterable`, and `is_object` now consume materializable native value-result expressions through that ABI. This is a concrete compiler/backend consumer, not broad call-result propagation, native-value storage, references/COW, object/resource metadata, exact diagnostics, or LLVM/assembly parity.
- `b1ecabdf codegen: echo value operations through runtime ABI`
  - Routes generated-C echoing of unary, binary, concat, bitwise, and shift native value-operation expressions through existing runtime value-result/value-operation ABIs. This expands executable generated-C operation semantics but does not complete arbitrary expression materialization, references/COW, object/resource conversion, exact warning order, or LLVM parity.
- `7296fdca codegen: route bitwise values through runtime ABI`
  - Adds and consumes a diagnostic-bearing runtime ABI for native value bitwise/shift operations. This is executable generated-C bitwise/shift semantics; warning parity, references/COW, object/resource conversion, and broad backend parity remain open.
- `35a20bfa codegen: route comparison aborts through decision ABI`
  - Routes generated-C comparison guards through shared branch-decision abort-code accessors instead of duplicating status plus exit-code handling.
- `56bc236c codegen: release cloned array literal values`
  - Releases temporary source array handles after cloning array literals into owned native value handles, tightening generated-C value ownership/cleanup.
- `5ccefbcf runtime: share PHP string raw buffer writes`
  - Routes scalar echo raw-buffer writes through the shared PHP string byte-source/view boundary.
- `b214ccc5 runtime: share PHP string byte source materialization`
  - Adds a shared runtime string byte view/source boundary consumed by native strings, diagnostics, array keys, array-column keys, and PHP string values.

## Current Lane-Local Candidate Work Not Yet Counted

- Array/reference candidates: generated-C owner/value/reference operation wrappers, null-coalescing native-value operands, foreach iterable/reference setup, assignment-expression value results, RMW paths, reference-cell transfers, shared writable path builders, and generated-C lvalue consumers.
- String/conversion candidates: formatter-tag stdout/raw-buffer/text-byte ABIs, scalar-to-native-value formatter consumers, byte-source/view/result boundaries, interpreter byte-output sinks, string offset warning blockers, and broader binary-safe string surfaces.
- Termination/control-flow candidates: statement-level native `exit`/`die` value ABI in LLVM and generated C, plus broader recursive CFG/effect/cleanup blocker work for loops/switch/goto/break/continue.
- Symbol/request/call candidates: request-derived assignment expression consumers, expression-result sequence consumers, root slot transition contracts, scalar linked symbol-table execution, callable operation/preflight/source contracts, and call cleanup/diagnostic families.
- Object/diagnostic/comparison candidates: object-property operation plans, class-policy/object receiver blockers, diagnostic result carriers and sinks, broader comparison relation-result public-consumer cleanup, and LLVM/generated-C comparison parity work.

Lane-local work is useful source material, but it is not product capability until selected into a small primary batch, gated, committed, and pushed.

## Candid Assessment

Recent primary work is directionally sound because it turns shared ABI surfaces into generated-C or runtime consumers and removes duplicated backend-local handling. The strongest pattern remains small and concrete: remove a backend-local bypass, route through a shared runtime semantic contract, and prove executable or ABI behavior.

The project remains boundary-heavy. More result/blocker vocabulary without immediate executable consumers will not move the product much. The next valuable batches should each answer: what PHP behavior now runs that did not run before, what shared runtime materialization path now has real consumers, or what backend-local bypass has been removed?

## Near-Term Steering

1. Treat `b1d7d0b9` as the latest integrated semantic baseline.
2. Prefer small executable generated-C/LLVM consumers of existing ABI surfaces over more standalone vocabulary.
3. Strong next candidates: isolate one narrow array-lvalue consumer from `impl-array-linked-exec` or `impl-native-integration-batch`; integrate the statement-level `exit`/`die` value ABI slice if it stays dependency-minimal; pursue LLVM/generated-C parity for already-landed comparison relation-result behavior; or route the generic formatter stdout boundary through a real backend path after the scalar echo value consumer.
4. Avoid whole-lane merges. Several lanes contain broad, conflict-prone, or non-executable contract work.
5. Require call/control-flow/object candidates to emit/link something real or remove a duplicated production blocker before they receive primary integration time.
6. Keep resource checks explicit before broad gates; `/dev/shm` is healthy at this review but volatile under build waves, and isolated target directories remain preferred to avoid target-dir races.
