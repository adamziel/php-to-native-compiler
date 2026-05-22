# PHP Native Compiler Progress

Updated: 2026-05-22 04:27 CEST
Evaluation marker: 20260522T022700Z-plus-c56163c

This is a high-level supervisor dashboard. Percentages are candid engineering estimates, not test-suite pass rates. Primary-integrated capability means committed on `master`; lane-local or uncommitted primary work is candidate material until selected, gated, committed, and pushed.

## Overall Status

Estimated progress toward a broadly usable generalized PHP native compiler: **45%**

```
Generalized runtime/ABI foundations      [##################--] 89%
Compiler/backend consumers               [##################--] 91%
Executable generalized PHP semantics     [###########---------] 55%
Arrays, references, COW, lvalues         [######--------------] 29%
Objects, properties, methods             [##------------------] 11%
Diagnostics/control-flow composition     [#####---------------] 25%
Broad integrated verification            [#########-----------] 46%
```

## Current Primary State

- Current committed primary HEAD at review: `c56163c6 codegen: route LLVM string-int calls through native ABI`.
- Latest committed semantic baseline: `c56163c6 codegen: route LLVM string-int calls through native ABI`.
- Latest semantic batch routes LLVM lowering for `strcasecmp()`, `strcmp()`, `strncmp()`, `strncasecmp()`, `substr_count()`, `ord()`, and `crc32()` through `phpc_native_value_string_int_operation_with_diagnostic(...)`. LLVM now materializes scalar/string operands as native values, converts offset/length operands through the shared int-conversion ABI, reports diagnostics through the runtime sink, and frees owned native values.
- Previous semantic batch added `phpc_native_value_array_clone(...)` as a generalized runtime boundary for cloning array-valued native values into owned array handles, then routed generated-native C direct array-offset assignments through the shared value-offset mutation ABI.
- Recent primary-integrated semantic progress in this evaluation window: generated-C byte string compare builtins through the string-int ABI; generated-C array/string offset presence through the value-offset ABI; generated-C string-offset writes through the value-offset mutation ABI; generated-C direct array-offset writes through the value-offset mutation ABI; LLVM string-int backend parity through the same string-int ABI.
- Resource note: `/dev/shm` is usable but volatile. Live check showed about 11G free and 12G used; dashboard evidence says it recently hit 100% during concurrent gates. `/home` has about 229G free by `df`.

## Grand Roadmap Position

The compiler is steadily replacing backend-local decisions and direct scalar/string handling with reusable runtime/ABI contracts plus selected backend/runtime consumers. Recent committed progress is strongest in generated-C value-offset presence, string-offset mutation, direct array-offset writes, LLVM/generated-C string-int ABI consumers, string-offset reads/bool probes/writes, primitive arithmetic conversion, unary string-result execution, comparison relation routing, scalar output, selected type predicates, bitwise/shift operations, and focused verification gates.

The product is still far from full generalized PHP. The largest missing regions remain references/COW, executable lvalues, user calls and frames, object/class/property semantics, mutable globals/superglobals, include/require, exceptions/finally, exact diagnostics, and cleanup across real control flow.

## Done / In Progress / Not Done

- [x] Supervised parallel lanes and a primary integration gate are established.
- [x] Shared runtime ABI surfaces exist for strings, byte buffers, comparisons, numeric-string classification, array key/value operations, value-offset read/presence/mutation operations, request-state snapshots, selected conversions, selected diagnostics, branch-decision status/abort handling, native value bitwise/shift operations, native value type predicates, runtime string-byte materialization, typed bool extraction, diagnostic report/free, and string-offset read/write/bool results.
- [x] LLVM/generated-C consumers exist for selected primitive arithmetic, selected unary string-result builtins, selected generated-C array/string offset presence, selected generated-C array-offset writes and string-offset writes through value-offset mutation, selected generated-C string-offset reads/bool probes, selected LLVM/generated-C string-int builtins, selected generated-C string-distance/path builtins, array key/value operations, array-handle value operands, array append diagnostics, comparison relation results, comparison abort guards, strict array/object identity in selected array-search builtins, native value bitwise/shift operations, scalar value echo/print output, print value-result output, native value type predicates, casts/type-name output, selected filesystem/cache blockers, and centralized call diagnostics.
- [x] Reusable array-entry snapshot ABI exists for future foreach/lvalue/reference consumers.
- [ ] In progress: replace shared blockers with executable semantics one family at a time.
- [ ] In progress: generalize value/result/source ownership across returns, call args, conditions, branch joins, stdout, discarded temporaries, comparisons, casts, string-byte materialization, bool probes, diagnostics, and cleanup.
- [ ] In progress: array lvalue/RMW/reference/COW work. Primary has foundations and selected generated-C consumers; lane-local candidates are much stronger than integrated capability.
- [ ] In progress: statement termination and control-flow cleanup candidates. Lane-local work is useful, but broad recursive loop/switch/goto/finally behavior is not integrated.
- [ ] Not done: full symbol environment semantics for locals/imports/globals/superglobals, undefined slots, repeated calls, writes/unset, references, and request/global separation.
- [ ] Not done: function/method frames, argument binding, by-ref args/returns, variadics/spreads, callbacks, constructors, recursion, closures, and dynamic dispatch.
- [ ] Not done: object/class/property semantics including allocation, visibility, magic hooks, `stdClass`, dynamic names, references/COW, and exact diagnostics.
- [ ] Not done: full control-flow cleanup, loops/switch/break/continue/goto/finally, exceptions, shutdown/destructors, and broad differential composition coverage.

## Active Roadmap Estimates

| Active item | Primary-integrated | Lane-local candidate maturity | Current read |
| --- | ---: | ---: | --- |
| String conversion, truthiness, byte buffers | 77% | 88% | Primary has shared value string-form semantics, generated-C unary string-result execution, LLVM/generated-C string-int consumers through the string-int ABI, generated-C value-offset presence for string offsets, generated-C string-offset reads, `isset`/`empty` bool probes, string-offset writes through the shared value-offset mutation ABI, warning continuation, scalar value output, generated-C print output, comparison byte materialization, runtime string-byte materialization, and raw-buffer writes. Lane-local work adds more LLVM string/debug parity, but it is not counted until integrated. |
| Call operation cleanup and ownership | 43% | 68% | Primary routes many call-result contexts, function declaration fallbacks, and backend call diagnostics through common contracts. Lane-local required-lvalue/discarded-result cleanup is broader; real frames, binding, returns, by-ref semantics, and dispatch remain mostly non-executable in primary. |
| Comparison and conversion semantics | 74% | 84% | Primary has reusable comparison validation, relation-result/result/branch/free/decision/status/abort ABIs, generated-C relation-result consumers, public operand routing, recursive-array blocker classification, string-handle operands, byte string compare builtin consumers, native object/resource strict identity, primitive arithmetic conversion for known operands, scalar casts, bitwise/shift consumers, value-operation output, type predicates, unary string-result output, and string-offset execution slices. Lane-local conversion work is advancing concat/source-result paths; dynamic arithmetic, division/modulo warning parity, executable recursive array comparison, object property comparison, resource loose comparison, reference dereference comparison, and backend parity remain open. |
| Arrays, lvalues, references, COW | 29% | 85% | Primary has array-key materialization, array value-operation result ABI, generated-C array/string offset presence through the value-offset ABI, runtime value-offset mutation for write/append/unset families, generated-C direct array-offset writes and string-offset writes through that mutation ABI, array-value cloning back to owned handles, array-entry snapshots, array-handle comparisons, append diagnostics, native array handles as owned value operands, cloned-literal cleanup, and selected string-offset probes outside full array/lvalue semantics. Lane-local lvalue/foreach/reference candidates are much stronger; full executable lvalues/references/COW are not integrated. |
| Symbols, globals, request state | 24% | 68% | Primary has symbol ABI helpers, request/superglobal snapshot ABI, and selected expression-result boundaries. Lane-local expression-result and symbol-cell work is promising, but primary still lacks real generalized locals, frames, globals, imports, request mutation, and reference assignment lowering. |
| Objects, properties, methods | 11% | 50% | Primary has native object handle strict-identity relation results and loose-comparison blockers through the shared comparison path. Lane-local object/class/property blockers and operation plans continue improving, but executable object/property/method behavior is still largely absent. |
| Diagnostics and control-flow cleanup | 25% | 70% | Primary has selected severity/blocker surfaces, diagnostic array append behavior, a shared native diagnostic report consumer, centralized call diagnostics, call-boundary cleanup routing, comparison branch abort handling, string-offset bool diagnostics, and warning-capable string-offset write continuation. Lane-local diagnostic sinks and terminating/control-flow cleanup boundaries are broader, but most loop/switch/goto/finally/exception behavior remains blocker/model work. |
| Broad composition verification | 46% | 50% | Focused runtime/native-link/native-runtime-ABI gates cover the newest LLVM string-int consumer, generated-C array-offset write consumer, value-offset mutation string-write consumer, value-offset array/string presence path, byte string compare builtin path, string-offset read path, shared diagnostic report consumer, string-offset warning continuation, string-offset write path, string-offset bool path, unary string-result executable path, comparison relation-result path, value-operation, scalar echo, print output, bitwise, type-predicate, array-value, string-byte, and diagnostic paths. Broad differential PHP composition coverage remains thin. |

## Recent Primary-Integrated Work

- `c56163c6 codegen: route LLVM string-int calls through native ABI`
  - Routes lowerable LLVM `strcasecmp()`, `strcmp()`, `strncmp()`, `strncasecmp()`, `substr_count()`, `ord()`, and `crc32()` calls through the existing string-int runtime ABI with native value materialization, int conversion, diagnostics, and owned-value cleanup.
- `e3a50ece codegen: route array offset writes through value mutation ABI`
  - Routes generated-native C direct array-offset assignments through the shared value-offset mutation ABI, then clones returned array values back to owned array handles for subsequent reads/presence checks. This is executable array-write progress, but not yet nested lvalues, references/COW, append/unset, object/ArrayAccess/resource offsets, request storage, LLVM parity, or C assembly fallback parity.
- `6340cf34 native: route offset writes through value mutation ABI`
  - Adds `phpc_native_value_offset_mutation_operation_with_diagnostic(...)` for value-offset write/append/unset families and routes generated-native C string-offset assignments through the shared mutation boundary.
- `ffb158c4 codegen: route offset presence through value ABI`
  - Routes generated-native C array/string offset `isset()` and `empty()` through `phpc_native_value_offset_operation_with_diagnostic(...)` and the native bool diagnostic boundary.
- `ff938b0a native: route byte string compares through string-int ABI`
  - Routes generated-native C `strcmp()`, `strncmp()`, and `strncasecmp()` through the shared string-int runtime ABI.

## Current Work Not Yet Counted

- Lane-local array/lvalue candidates: dynamic string-offset read/probe execution through lvalue results, generated-C owner/value/reference operation wrappers, current-read/RMW/null-aware/foreach/reference/lvalue candidates, dynamic key source-span diagnostics, root-aware by-reference foreach blockers, and follow-up writes/unsets/references on top of the landed value-offset boundaries.
- Lane-local reference/COW candidates: owner-cell alias-visibility, mutation-barrier, cleanup-insertion, and call-emission scaffolds; useful planning, but still not emitted execution in primary.
- Lane-local symbol/request candidates: reference/COW-aware native symbol cells, request-superglobal backing storage, root/frame/imported alias reference binding, undefined-slot tracking, and request-state operation contracts.
- Lane-local conversion/string candidates: conversion-result stdout/free ABI, dynamic concat/source-result conversion, primitive comparison/bitwise/division/modulo conversion result work, formatter/string-byte/string-offset surfaces, and broader binary-safe string operation boundaries.
- Lane-local termination/control-flow candidates: terminating-arm cleanup insertion and structured control-transfer cleanup plans.
- Lane-local object/call/diagnostic candidates: object-property operation plans, class-policy/object receiver blockers, diagnostic result carriers and sinks, call-frame/value-boundary work, and cleanup carriers.

Lane-local work is useful source material, but it is not product capability until selected into a small primary batch, gated, committed, and pushed.

## Candid Assessment

Recent primary work is directionally sound because it turns shared ABI surfaces into concrete backend/runtime consumers and removes duplicated backend-local handling. The value-offset mutation path is the most strategically useful recent thread because it reaches array writes and starts pushing toward lvalue semantics; the LLVM string-int slice is useful backend parity for an already-landed runtime/generated-C surface.

The project remains boundary-heavy. More result/blocker vocabulary without immediate executable consumers will not move the product much. The next valuable batches should each answer: what PHP behavior now runs that did not run before, what shared runtime materialization path now has real consumers, or what backend-local bypass has been removed?

## Near-Term Steering

1. Treat `c56163c6` as the latest committed semantic baseline; progress-only commits are management artifacts.
2. Avoid extending scalar/string work unless it removes another real backend-local bypass with executable gates.
3. Prefer small executable generated-C/LLVM/runtime consumers of existing ABI surfaces over more standalone vocabulary.
4. Strong next candidates: append/unset/nested writeback on the value-offset boundary, dynamic string-offset lvalue read/probe, a narrow reference/COW symbol-cell consumer, generated-C/LLVM parity for the value-offset boundary, or a control-flow cleanup consumer that emits before a real terminal transfer.
5. Avoid whole-lane merges. Current lane evidence is broad and conflict-prone even when technically useful.
6. Keep resource checks explicit before broad gates; `/dev/shm` has recovered to about 11G free but recently reached 100% under concurrent lane builds.
