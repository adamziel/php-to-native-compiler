# PHP Native Compiler Progress

Updated: 2026-05-22 03:36 CEST
Evaluation marker: 20260522T013047Z-plus-ff938b0a

This is a high-level supervisor dashboard. Percentages are candid engineering estimates, not test-suite pass rates. Primary-integrated capability means committed on `master`; lane-local work is candidate material until selected, gated, committed, and pushed.

## Overall Status

Estimated progress toward a broadly usable generalized PHP native compiler: **42%**

```
Generalized runtime/ABI foundations      [#################---] 86%
Compiler/backend consumers               [#################---] 87%
Executable generalized PHP semantics     [##########----------] 52%
Arrays, references, COW, lvalues         [#####---------------] 24%
Objects, properties, methods             [##------------------] 11%
Diagnostics/control-flow composition     [#####---------------] 25%
Broad integrated verification            [########------------] 42%
```

## Current Primary State

- Current committed primary HEAD at progress update: `ff938b0a native: route byte string compares through string-int ABI`; `origin/master` matches for the semantic code push.
- Latest committed semantic baseline: `ff938b0a native: route byte string compares through string-int ABI`.
- Latest semantic batch routes generated-native C `strcmp()`, `strncmp()`, and `strncasecmp()` through the shared string-int runtime ABI, reusing byte-preserving value-to-string conversion, diagnostics, length handling, and generated-C linked execution. It removed backend-local rejection for these lowerable byte string compare builtins without claiming full PHP string semantics.
- Previous semantic batch routes generated-native C string-offset reads through `phpc_native_value_string_offset_operation_with_diagnostic(...)`, clones returned one-byte string values through the shared byte-buffer boundary, reports diagnostics through the shared native diagnostic reporter, tracks byte lengths, and frees owned byte buffers on normal/error exits.
- Earlier batches in this window routed generated-C string-offset assignment through shared write/byte-buffer helpers, kept supported warning-capable writes running after diagnostics, routed string-offset `isset()`/`empty()` probes through shared bool-result paths, and shared owned diagnostic report/free consumption.
- Explicit remaining blockers for the string-offset family: string-offset unsets/references, nested/append/compound writes, scalar subject warning parity, negative offsets, out-of-range read recovery, invalid-key parity, non-UTF-8 byte string results, array/object/ArrayAccess/resource offset behavior, references/COW mutation, arbitrary dynamic byte-buffer subjects, LLVM parity, and C assembly fallback lowering.
- Resource note: `/dev/shm` is currently healthy for focused gates at about 19G free, but recent test waves drove it near full. Continue isolated target dirs and low job counts for primary integration.

## Grand Roadmap Position

The compiler is steadily replacing backend-local decisions and direct scalar/string handling with reusable runtime/ABI contracts and selected generated-C/runtime consumers. Recent primary progress is strongest in generated-C string-offset reads/bool probes/writes, warning-capable write continuation, byte string compare builtins through the string-int ABI, primitive arithmetic conversion, unary string-result execution, comparison relation routing, public operand comparison consumers, recursive-array blocker classification, native object/resource strict-identity relation results, generated-C value output for `echo`/`print`, type predicates, bitwise/shift operations, scalar casts, string byte materialization, array-handle value operands, selected call diagnostics, and focused verification gates.

The product is still far from full generalized PHP. The largest missing regions remain references/COW, executable lvalues, user calls and frames, object/class/property semantics, mutable globals/superglobals, include/require, exceptions/finally, exact diagnostics, and cleanup across real control flow.

## Done / In Progress / Not Done

- [x] Supervised parallel lanes and a primary integration gate are established.
- [x] Shared runtime ABI surfaces exist for strings, byte buffers, comparisons, numeric-string classification, array key/value operations, request-state snapshots, selected conversions, selected diagnostics, branch-decision status/abort handling, native value bitwise/shift operations, native value type predicates, runtime string-byte materialization, typed bool extraction, diagnostic report/free, and string-offset read/write/bool results.
- [x] LLVM/generated-C consumers exist for selected primitive arithmetic, selected unary string-result builtins, selected generated-C string-offset reads/bool probes/writes, selected string-int/distance/path builtins, array key/value operations, array-handle value operands, array append diagnostics, comparison relation results, comparison abort guards, strict array/object identity in selected array-search builtins, native value bitwise/shift operations, scalar value echo/print output, print value-result output, native value type predicates, casts/type-name output, selected filesystem/cache blockers, and centralized call diagnostics.
- [x] Reusable array-entry snapshot ABI exists for future foreach/lvalue/reference consumers.
- [ ] In progress: replace shared blockers with executable semantics one family at a time.
- [ ] In progress: generalize value/result/source ownership across returns, call args, conditions, branch joins, stdout, discarded temporaries, comparisons, casts, string-byte materialization, bool probes, diagnostics, and cleanup.
- [ ] In progress: array lvalue/RMW/reference/COW work. Primary has foundations and selected consumers; lane-local candidates are much stronger than integrated capability.
- [ ] In progress: statement termination and control-flow cleanup candidates. Lane-local work is useful, but broad recursive loop/switch/goto/finally behavior is not integrated.
- [ ] Not done: full symbol environment semantics for locals/imports/globals/superglobals, undefined slots, repeated calls, writes/unset, references, and request/global separation.
- [ ] Not done: function/method frames, argument binding, by-ref args/returns, variadics/spreads, callbacks, constructors, recursion, closures, and dynamic dispatch.
- [ ] Not done: object/class/property semantics including allocation, visibility, magic hooks, `stdClass`, dynamic names, references/COW, and exact diagnostics.
- [ ] Not done: full control-flow cleanup, loops/switch/break/continue/goto/finally, exceptions, shutdown/destructors, and broad differential composition coverage.

## Active Roadmap Estimates

| Active item | Primary-integrated | Lane-local candidate maturity | Current read |
| --- | ---: | ---: | --- |
| String conversion, truthiness, byte buffers | 74% | 87% | Primary has shared value string-form semantics, generated-C unary string-result execution, generated-C byte string compare builtins through the string-int ABI, generated-C string-offset reads, generated-C string-offset `isset`/`empty` bool probes, generated-C string-offset writes with byte-buffer rematerialization, warning-capable multi-byte replacement truncation, scalar value output, generated-C print output, comparison byte materialization, runtime string-byte materialization, and raw-buffer writes. Lane-local string-offset/lvalue blockers and formatter/string-byte work are broader but not counted until integrated. |
| Call operation cleanup and ownership | 43% | 68% | Primary routes many call-result contexts, function declaration fallbacks, and backend call diagnostics through common contracts. Lane-local required-lvalue/discarded-result cleanup is broader; real frames, binding, returns, by-ref semantics, and dispatch remain mostly non-executable in primary. |
| Comparison and conversion semantics | 74% | 83% | Primary has reusable comparison validation, relation-result/result/branch/free/decision/status/abort ABIs, generated-C relation-result consumers, public operand routing, recursive-array blocker classification, string-handle operands, byte string compare builtin consumers, native object/resource strict identity, primitive arithmetic conversion for known operands, scalar casts, bitwise/shift consumers, value-operation output, type predicates, unary string-result output, and string-offset execution slices. Dynamic arithmetic, division/modulo warning parity, executable recursive array comparison, object property comparison, resource loose comparison, reference dereference comparison, and backend parity remain open. |
| Arrays, lvalues, references, COW | 24% | 84% | Primary has array-key materialization, array value-operation result ABI, array-entry snapshots, array-handle comparisons, append diagnostics, native array handles as owned value operands, cloned-literal cleanup, and selected string-offset probes outside full array semantics. Lane-local value-offset presence, dynamic string-offset lvalue read/probe, foreach/root-aware blockers, and reference/COW owner-cell candidates are much stronger; full executable lvalues/references/COW are not integrated. |
| Symbols, globals, request state | 24% | 67% | Primary has symbol ABI helpers, request/superglobal snapshot ABI, and selected expression-result boundaries. Lane-local reference/COW-aware symbol cells and request-superglobal backing storage are promising, but primary still lacks real generalized locals, frames, globals, imports, request mutation, and reference assignment lowering. |
| Objects, properties, methods | 11% | 50% | Primary has native object handle strict-identity relation results and loose-comparison blockers through the shared comparison path. Lane-local object/class/property blockers and operation plans continue improving, but executable object/property/method behavior is still largely absent. |
| Diagnostics and control-flow cleanup | 25% | 69% | Primary has selected severity/blocker surfaces, diagnostic array append behavior, a shared native diagnostic report consumer, centralized call diagnostics, call-boundary cleanup routing, comparison branch abort handling, string-offset bool diagnostics, and warning-capable string-offset write continuation. Lane-local diagnostic-result and terminating-arm cleanup boundaries are broader, but most loop/switch/goto/finally/exception behavior remains blocker/model work. |
| Broad composition verification | 42% | 48% | Focused runtime/native-link gates cover the newest byte string compare builtin path, string-offset read path, shared diagnostic report consumer, string-offset warning continuation, string-offset write path, string-offset bool path, unary string-result executable path, comparison relation-result path, value-operation, scalar echo, print output, bitwise, type-predicate, array-value, string-byte, and diagnostic paths. Broad differential PHP composition coverage remains thin. |

## Recent Primary-Integrated Work

- `ff938b0a native: route byte string compares through string-int ABI`
  - Routes generated-native C `strcmp()`, `strncmp()`, and `strncasecmp()` through the shared string-int runtime ABI, with focused runtime, generated-source, linked-executable, string-int filter, package check, rustfmt, and diff gates passing.
- `4291db4d codegen: route string offset reads through native ABI`
  - Routes generated-native C string offset reads through shared runtime string-offset operation and byte-buffer clone ABIs, with linked executable coverage for echo, `strlen`, and array key/value materialization consumers.
- `a74194cf native: share diagnostic report consumer`
  - Adds one shared runtime/backend diagnostic-report consumer so diagnostic handles are reported and freed through a reusable path in runtime and generated backend declarations/consumers.
- `990ec6a5 runtime: continue string offset writes after warnings`
  - Extends the generated-C/runtime string-offset write path so warning-capable writes do not collapse into hard failure. Multi-byte replacement strings truncate to the first byte, return the updated value, emit the PHP warning diagnostic, and preserve the existing byte-buffer materialization path.
- `6e07d95f codegen: route string offset writes through native ABI`
  - Routes generated-C string-offset assignment through shared runtime write and byte-buffer clone ABIs, then emits written string bytes with runtime lengths instead of C `strlen` or local string formatting.
- `886fd131 codegen: route string offset bool probes through native ABI`
  - Routes generated-C string-offset `isset()` and `empty()` probes through shared runtime string-offset and bool-result ABIs with linked executable coverage.
- `1f8e92ef codegen: route unary string results through native ABI`
  - Routes lowerable generated-C `strrev()`, `str_rot13()`, `bin2hex()`, `strtolower()`, `strtoupper()`, `ucfirst()`, and `lcfirst()` through the shared string-result runtime ABI.
- `7471b54c codegen: lower primitive arithmetic semantics`
  - Adds shared primitive arithmetic conversion/result handling and consumes it from LLVM and generated-C lowering for known primitive `+`, `-`, `*`, and unary `-` operands.

## Current Lane-Local Candidate Work Not Yet Counted

- Integration-batch candidate: generated-C array/string offset presence through the shared value-offset ABI.
- Array/lvalue candidates: dynamic string-offset read/probe execution through lvalue results, generated-C owner/value/reference operation wrappers, current-read/RMW/null-aware/foreach/reference/lvalue candidates, dynamic key source-span diagnostics, and root-aware by-reference foreach blockers.
- Reference/COW candidates: owner-cell alias-visibility, mutation-barrier, cleanup-insertion, and call-emission scaffolds; useful planning, but still not emitted execution in primary.
- Symbol/request candidates: reference/COW-aware native symbol cells, request-superglobal backing storage, root/frame/imported alias reference binding, undefined-slot tracking, and request-state operation contracts.
- Conversion/string candidates: conversion-result stdout/free ABI, primitive comparison/bitwise/division/modulo conversion result work, formatter/string-byte/string-offset surfaces, and broader binary-safe string operation boundaries.
- Termination/control-flow candidates: terminating-arm cleanup insertion and structured control-transfer cleanup plans.
- Object/call/diagnostic candidates: object-property operation plans, class-policy/object receiver blockers, diagnostic result carriers and sinks, call-frame/value-boundary work, and cleanup carriers.

Lane-local work is useful source material, but it is not product capability until selected into a small primary batch, gated, committed, and pushed.

## Candid Assessment

Recent primary work is directionally sound because it turns shared ABI surfaces into generated-C/runtime consumers and removes duplicated backend-local handling. The best pattern in this window was narrow executable string-offset semantics with focused linked coverage.

The project remains boundary-heavy. More result/blocker vocabulary without immediate executable consumers will not move the product much. The next valuable batches should each answer: what PHP behavior now runs that did not run before, what shared runtime materialization path now has real consumers, or what backend-local bypass has been removed?

## Near-Term Steering

1. Treat `ff938b0a` as the latest committed semantic baseline; progress-only commits are management artifacts.
2. Avoid extending scalar/string work unless it removes another real backend-local bypass with executable gates; otherwise steer the next primary slice toward lvalues/references/symbols/control-flow.
3. Prefer small executable generated-C/LLVM/runtime consumers of existing ABI surfaces over more standalone vocabulary.
4. Strong next candidates: shared value-offset array/string presence, dynamic string-offset lvalue read/probe, a narrow reference/COW symbol-cell consumer, or a control-flow cleanup consumer that emits before a real terminal transfer.
5. Avoid whole-lane merges. Current lane evidence is broad and conflict-prone even when technically useful.
6. Refresh the supervisor dashboard; its current tail is stale versus live git/status evidence.
7. Keep resource checks explicit before broad gates; `/dev/shm` is currently healthy but has recently been near full under concurrent lane builds.
