# PHP Native Compiler Progress

Updated: 2026-05-22 03:23 CEST
Evaluation marker: 20260522T012300Z-plus-4291db4d

This is a high-level supervisor dashboard. Percentages are candid engineering estimates, not test-suite pass rates. Primary-integrated capability means committed on `master`; lane-local work is candidate material until selected, gated, committed, and pushed.

## Overall Status

Estimated progress toward a broadly usable generalized PHP native compiler: **41%**

```
Generalized runtime/ABI foundations      [#################---] 86%
Compiler/backend consumers               [#################---] 86%
Executable generalized PHP semantics     [##########----------] 51%
Arrays, references, COW, lvalues         [#####---------------] 24%
Objects, properties, methods             [##------------------] 11%
Diagnostics/control-flow composition     [#####---------------] 25%
Broad integrated verification            [########------------] 41%
```

## Current Primary State

- Current primary HEAD at review: `4291db4d codegen: route string offset reads through native ABI`.
- Latest committed semantic baseline: `4291db4d codegen: route string offset reads through native ABI`.
- Latest semantic batch routes generated-native C string-offset reads through `phpc_native_value_string_offset_operation_with_diagnostic(...)`, clones returned one-byte string values through the shared byte-buffer boundary, reports diagnostics through the shared native diagnostic reporter, tracks byte lengths, and frees owned byte buffers on normal/error exits.
- Previous string-offset batches keep supported generated-C string-offset assignments running when the runtime emits a non-fatal warning, route generated-C string-offset assignment through shared write/byte-buffer helpers, and route generated-C `isset($string[$offset])` / `empty($string[$offset])` probes through shared string-offset and bool-result ABIs.
- Explicit remaining blockers for that family: string-offset unsets/references, nested/append writes, negative offset parity, out-of-range warning recovery, invalid-key parity, non-UTF-8 byte string results, array/object/ArrayAccess/resource offset behavior, references/COW mutation, arbitrary dynamic byte-buffer subjects, LLVM parity, and C assembly fallback lowering.
- Resource note: `/dev/shm` is usable for focused gates but can swing sharply under concurrent lane builds. Current supervisor sample saw it recover from below the 6G dispatcher floor to over 20G free after active-process-aware cleanup. Continue using isolated target dirs and low job counts for primary integration.

## Grand Roadmap Position

The compiler is steadily replacing backend-local decisions and direct scalar/string handling with reusable runtime/ABI contracts and selected LLVM/generated-C/runtime consumers. Recent primary progress is strongest in primitive arithmetic conversion, generated-C unary string-result execution, generated-C string-offset reads/bool probes/writes, comparison relation routing, public operand comparison consumers, recursive-array blocker classification, native object/resource strict-identity relation results, generated-C value output for `echo`/`print`, type predicates, bitwise/shift operations, scalar casts, string byte materialization, array-handle value operands, selected call diagnostics, and focused verification gates.

The product is still far from full generalized PHP. The largest missing regions remain references/COW, executable lvalues, user calls and frames, object/class/property semantics, mutable globals/superglobals, include/require, exceptions/finally, exact diagnostics, and cleanup across real control flow.

## Done / In Progress / Not Done

- [x] Supervised parallel lanes and a primary integration gate are established.
- [x] Shared runtime ABI surfaces exist for strings, byte buffers, comparisons, numeric-string classification, array key/value operations, request-state snapshots, selected conversions, selected filesystem/cache blockers, selected diagnostics, branch-decision status/abort handling, native value bitwise/shift operations, native value type predicates, runtime string-byte materialization, typed bool extraction, and string-offset write results.
- [x] LLVM/generated-C consumers exist for selected primitive arithmetic, selected unary string-result builtins, selected generated-C string-offset reads/bool probes/writes, selected string-int/distance/path builtins, array key/value operations, array-handle value operands, array append diagnostics, comparison relation results, comparison abort guards, strict array/object identity in selected array-search builtins, native value bitwise/shift operations, unary/binary/native value operation output, scalar value echo/print output, print value-result output, native value type predicates, casts/type-name output, selected filesystem/cache blockers, and centralized call diagnostics.
- [x] Reusable array-entry snapshot ABI exists for future foreach/lvalue/reference consumers.
- [ ] In progress: replace shared blockers with executable semantics one family at a time.
- [ ] In progress: generalize value/result/source ownership across returns, call args, conditions, branch joins, stdout, discarded temporaries, comparisons, casts, string-byte materialization, bool probes, and cleanup.
- [ ] In progress: array lvalue/RMW/reference/COW work. Primary has foundations and selected consumers; lane-local candidates are much stronger than integrated capability.
- [ ] In progress: statement termination and control-flow cleanup candidates. Lane-local work is useful, but broad recursive loop/switch/goto/finally behavior is not integrated.
- [ ] Not done: full symbol environment semantics for locals/imports/globals/superglobals, undefined slots, repeated calls, writes/unset, references, and request/global separation.
- [ ] Not done: function/method frames, argument binding, by-ref args/returns, variadics/spreads, callbacks, constructors, recursion, closures, and dynamic dispatch.
- [ ] Not done: object/class/property semantics including allocation, visibility, magic hooks, `stdClass`, dynamic names, references/COW, and exact diagnostics.
- [ ] Not done: full control-flow cleanup, loops/switch/break/continue/goto/finally, exceptions, shutdown/destructors, and broad differential composition coverage.

## Active Roadmap Estimates

| Active item | Primary-integrated | Lane-local candidate maturity | Current read |
| --- | ---: | ---: | --- |
| String conversion, truthiness, byte buffers | 72% | 86% | Primary has shared value string-form semantics, generated-C unary string-result execution, generated-C string-offset reads, generated-C string-offset `isset`/`empty` bool probes, generated-C string-offset writes with byte-buffer rematerialization, warning-capable multi-byte replacement truncation, scalar value output, generated-C print output, comparison byte materialization, runtime string-byte materialization, and raw-buffer writes. Lane-local formatter stdout/byte-buffer/string-offset work is stronger but not counted until integrated. |
| Call operation cleanup and ownership | 43% | 68% | Primary routes many call-result contexts, function declaration fallbacks, and backend call diagnostics through common contracts. Lane-local required-lvalue/discarded-result cleanup is broader; real frames, binding, returns, by-ref semantics, and dispatch remain mostly non-executable in primary. |
| Comparison and conversion semantics | 73% | 82% | Primary has reusable comparison validation, relation-result/result/branch/free/decision/status/abort ABIs, generated-C relation-result consumers, public operand routing, recursive-array blocker classification, string-handle operands, native object/resource strict identity, primitive arithmetic conversion for known operands, scalar casts, bitwise/shift consumers, value-operation output, type predicates, unary string-result output, string-offset reads/bool probes, and string-offset write byte rematerialization. Dynamic arithmetic, division/modulo warning parity, executable recursive array comparison, object property comparison, resource loose comparison, reference dereference comparison, and backend parity remain open. |
| Arrays, lvalues, references, COW | 24% | 82% | Primary has array-key materialization, array value-operation result ABI, array-entry snapshots, array-handle comparisons, append diagnostics, native array handles as owned value operands, cloned-literal cleanup, and selected string-offset bool probes outside array semantics. Lane-local current-read/RMW/null-aware/foreach/reference/lvalue and reference/COW symbol-cell candidates are much stronger; full executable lvalues/references/COW are not integrated. |
| Symbols, globals, request state | 24% | 67% | Primary has symbol ABI helpers, request/superglobal snapshot ABI, and selected expression-result boundaries. Lane-local reference/COW-aware symbol cells and request-superglobal backing storage are promising, but primary still lacks real generalized locals, frames, globals, imports, request mutation, and reference assignment lowering. |
| Objects, properties, methods | 11% | 50% | Primary has native object handle strict-identity relation results and loose-comparison blockers through the shared comparison path. Lane-local object/class/property blockers and operation plans continue improving, but executable object/property/method behavior is still largely absent. |
| Diagnostics and control-flow cleanup | 25% | 69% | Primary has selected severity/blocker surfaces, diagnostic array append behavior, a shared native diagnostic report consumer, centralized call diagnostics, call-boundary cleanup routing, comparison branch abort handling, string-offset bool diagnostics, and warning-capable string-offset write continuation. Lane-local diagnostic-result and terminating-arm cleanup boundaries are broader, but most loop/switch/goto/finally/exception behavior remains blocker/model work. |
| Broad composition verification | 41% | 47% | Focused runtime/native-link gates cover the newest string-offset read path, shared diagnostic report consumer, string-offset warning continuation, string-offset write path, string-offset bool path, unary string-result executable path, comparison relation-result path, value-operation, scalar echo, print output, bitwise, type-predicate, array-value, string-byte, and diagnostic paths. Broad differential PHP composition coverage remains thin, and exploratory full `native_link` still has two stale unrelated failures. |

## Recent Primary-Integrated Work

- `4291db4d codegen: route string offset reads through native ABI`
  - Routes generated-native C string offset reads through shared runtime string-offset operation and byte-buffer clone ABIs, with linked executable coverage for echo, `strlen`, and array key/value materialization consumers. This is real generated-C read execution for supported string-valued subjects, not full PHP offset semantics: scalar-subject warning behavior, negative offsets, out-of-range warning-plus-empty recovery, invalid key TypeError parity, non-UTF-8 byte results, unsets/references/COW, nested/append/compound offsets, array/object/ArrayAccess/resource subjects, LLVM parity, and C assembly fallback remain blocked.
- `a74194cf native: share diagnostic report consumer`
  - Adds one shared runtime/backend diagnostic-report consumer so diagnostic handles are reported and freed through a reusable path in runtime and generated backend declarations/consumers. Linked coverage proves the shared reporter is used from generated-native diagnostic paths instead of duplicating report/free handling. This is cleanup/reporting infrastructure, not complete diagnostic semantics: exact warning ordering, lvalue diagnostics, branch/control-flow threading, request/object/callable producers, and LLVM/assembly parity for every diagnostic-producing expression remain open.
- `990ec6a5 runtime: continue string offset writes after warnings`
  - Extends the generated-C/runtime string-offset write path so warning-capable writes do not collapse into hard failure. Multi-byte replacement strings truncate to the first byte, return the updated value, emit the PHP warning diagnostic, and preserve the existing byte-buffer materialization path. This is still limited to supported string-offset assignment shapes: reads, unsets, references/COW, nested writes, append writes, negative offsets, array/object/ArrayAccess offsets, LLVM parity, and exact diagnostic ordering outside this warning case remain blocked.
- `6e07d95f codegen: route string offset writes through native ABI`
  - Routes generated-C string-offset assignment through shared runtime write and byte-buffer clone ABIs, then emits written string bytes with runtime lengths instead of C `strlen` or local string formatting. Linked executable coverage proves dynamic offset/replacement handling, NUL byte preservation, array-key composition, cleanup, and diagnostics. This is real string-offset write execution for supported string variables, not full PHP offset semantics: reads, unsets, references/COW, nested writes, append writes, negative offsets, exact warning/recovery, array/object/ArrayAccess offsets, LLVM parity, and arbitrary byte-buffer subjects remain blocked.
- `886fd131 codegen: route string offset bool probes through native ABI`
  - Routes generated-C string-offset `isset()` and `empty()` probes through shared runtime string-offset and bool-result ABIs with linked executable coverage. This is real generated-C bool-probe execution, not full string/array offset semantics: reads/writes/unsets/references, negative offsets, exact warnings, arrays/objects/ArrayAccess, references/COW, LLVM parity, and arbitrary byte-buffer subjects remain blocked.
- `1f8e92ef codegen: route unary string results through native ABI`
  - Routes lowerable generated-C `strrev()`, `str_rot13()`, `bin2hex()`, `strtolower()`, `strtoupper()`, `ucfirst()`, and `lcfirst()` through the shared string-result runtime ABI instead of rejecting or formatting locally. Runtime and linked executable coverage prove binary-byte preservation where current UTF-8 value storage permits it, scalar conversion, stdout consumption, diagnostics, cleanup, and shared operation tags.
- `7471b54c codegen: lower primitive arithmetic semantics`
  - Adds shared primitive arithmetic conversion/result handling and consumes it from LLVM and generated-C lowering for known primitive `+`, `-`, `*`, and unary `-` operands. Dynamic operands, references/COW, arrays/objects/resources, exact warning/recovery ordering, and division/modulo parity remain blocked in primary.
- `23df69e6 runtime: route arrays through recursive comparison blockers`
  - Classifies loose array comparisons as a recursive-array comparison blocker through the shared runtime comparison family. This is a centralized blocker/diagnostic boundary, not executable recursive array comparison.
- `45d48d75 runtime: route native handles through strict identity relations`
  - Routes native object and resource handle strict identity/non-identity through `NativeComparisonRelationResult`. It does not implement object property comparison, resource loose comparison, reference dereference comparison, generated backend object/resource consumers, or full PHP object/resource diagnostics.
- `61abb76d codegen: route print values through native result output`
  - Routes generated-native C `print` output through existing native value result/output helpers and links executable coverage for integers, floats, string bytes including NUL, type-name output, and `strlen()` composition.
- `68b17030 codegen: share function declaration fallback diagnostics`
  - Routes LLVM and generated-C/native function declaration fallback diagnostics through one helper while preserving static-local precedence. Function declarations still do not execute natively.

## Current Lane-Local Candidate Work Not Yet Counted

- Symbol/reference candidates: reference/COW-aware native symbol cells, request-superglobal backing storage, root/frame/imported alias reference binding, undefined-slot tracking, and request-state operation contracts.
- Conversion/string candidates: conversion-result stdout/free ABI, primitive comparison/bitwise/division/modulo conversion result work, formatter/string-byte/string-offset surfaces, and broader binary-safe string operation boundaries.
- Array/lvalue candidates: generated-C owner/value/reference operation wrappers, current-read/RMW/null-aware/foreach/reference/lvalue candidates, dynamic key source-span diagnostics, and native-value bridge blocker consolidation.
- Comparison candidates: owned reference comparison cleanup, concrete reference-handle dereference comparison, and backend comparison parity surfaces.
- Termination/control-flow candidates: terminating-arm cleanup insertion and structured control-transfer cleanup plans.
- Object/call/diagnostic candidates: object-property operation plans, class-policy/object receiver blockers, diagnostic result carriers and sinks, call-frame/value-boundary work, and cleanup carriers.

Lane-local work is useful source material, but it is not product capability until selected into a small primary batch, gated, committed, and pushed.

## Candid Assessment

Recent primary work is directionally sound because it turns shared ABI surfaces into generated-C or runtime consumers and removes duplicated backend-local handling. The newest diagnostic-report consumer is small and reusable, but it is still infrastructure. The next batches should keep biasing toward executable behavior, not just more result vocabulary.

The project remains boundary-heavy. More result/blocker vocabulary without immediate executable consumers will not move the product much. The next valuable batches should each answer: what PHP behavior now runs that did not run before, what shared runtime materialization path now has real consumers, or what backend-local bypass has been removed?

## Near-Term Steering

1. Treat `4291db4d` as the latest integrated semantic baseline; progress-only commits are management artifacts.
2. Prefer small executable generated-C/LLVM/runtime consumers of existing ABI surfaces over more standalone vocabulary.
3. Strong next candidates: isolate one narrow reference/COW symbol-cell or array-lvalue current-read/RMW primary slice; integrate a conversion-result stdout/string-offset follow-up only if it removes a real backend bypass; or select one control-flow cleanup consumer that emits before a real terminal transfer.
4. Avoid whole-lane merges. Current lane evidence is broad and conflict-prone even when technically useful.
5. Require call/control-flow/object/diagnostic candidates to emit or link something real, or to remove a duplicated production blocker, before they receive primary integration time.
6. Keep resource checks explicit before broad gates; `/dev/shm` is below 10G free at this review, and isolated target directories remain preferred.
