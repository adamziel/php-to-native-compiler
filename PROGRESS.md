# PHP Native Compiler Progress

Updated: 2026-05-22 05:28 CEST
Evaluation marker: 20260522T032800Z-plus-0568ebb1

This is a high-level supervisor dashboard. Percentages are candid engineering estimates, not test-suite pass rates. Primary-integrated capability means committed on `master`; lane-local or uncommitted primary work is candidate material until selected, gated, committed, and pushed.

## Overall Status

Estimated progress toward a broadly usable generalized PHP native compiler: **50%**

```
Generalized runtime/ABI foundations      [##################--] 89%
Compiler/backend consumers               [###################-] 95%
Executable generalized PHP semantics     [############--------] 60%
Arrays, references, COW, lvalues         [#######-------------] 35%
Objects, properties, methods             [##------------------] 11%
Diagnostics/control-flow composition     [#####---------------] 25%
Broad integrated verification            [##########----------] 50%
```

## Current Primary State

- Current primary HEAD at review: `0568ebb1 codegen: route array appends through value mutation ABI`.
- Current committed semantic baseline at review: `0568ebb1 codegen: route array appends through value mutation ABI`.
- Latest semantic batch routes generated-native C direct `$array[] = $value` assignments through the shared `phpc_native_value_offset_mutation_operation_with_diagnostic(...)` value-offset mutation ABI using append operation tag `1`, a null offset handle, diagnostic reporting, and returned-array rematerialization through `phpc_native_value_array_clone(...)`. This is direct owner-slot append execution, not nested append/writeback, assignment-expression append values, references/COW, LLVM parity, or object/ArrayAccess/resource offset support.
- Previous semantic batch routes LLVM lowering for lowerable string offset reads, plus string-offset `isset()`/`empty()` probes, through the shared `phpc_native_value_offset_operation_with_diagnostic(...)` value-offset ABI and native bool/string materialization consumers. It replaces an LLVM-side string-offset bypass with the same value-offset semantic boundary used by generated-native C surfaces instead of adding source-shape handling. LLVM array-offset reads/probes remain blocked behind missing owner/value-slot/lvalue contracts.
- Previous semantic batch routes generated-native C `unset(...)` statements with multiple direct array-index operands through the same shared value-offset mutation path used by single direct array-offset unset. It sequences each target through the reusable mutation helper instead of adding a fixture-specific recognizer.
- Previous semantic batch routes generated-native C direct `unset($array[$offset])` through the shared `phpc_native_value_offset_mutation_operation_with_diagnostic(...)` ABI using operation tag `2`, then rematerializes the returned array through `phpc_native_value_array_clone(...)`. The same helper now serves direct array-offset writes, unsets, and direct appends.
- Previous semantic batch routed LLVM lowering for `strcasecmp()`, `strcmp()`, `strncmp()`, `strncasecmp()`, `substr_count()`, `ord()`, and `crc32()` through `phpc_native_value_string_int_operation_with_diagnostic(...)`.
- Recent primary-integrated semantic progress in this evaluation window: generated-C byte string compare builtins through the string-int ABI; generated-C array/string offset presence through the value-offset ABI; generated-C string-offset writes through the value-offset mutation ABI; generated-C direct array-offset writes, direct array-offset unsets, multi-operand direct array-offset unset sequencing, and direct array appends through the value-offset mutation ABI; LLVM string-int backend parity through the same string-int ABI; LLVM string-offset reads and probes through the shared value-offset ABI.
- Resource note: `/dev/shm` remains usable but volatile. Live check for this review showed 22G total, 13G used, and 9.9G free by `df`; dashboard evidence says it saturated to 100% during concurrent gates in this window. `/home` has about 227G free by `df`; `du -sh /home` reported about 195G used with permission warnings under container overlay directories.

## Grand Roadmap Position

The compiler is steadily replacing backend-local decisions and direct scalar/string handling with reusable runtime/ABI contracts plus selected backend/runtime consumers. Recent committed progress is strongest in generated-C value-offset presence and mutation, LLVM string-offset read/probe routing through the shared value-offset ABI, direct array-offset writes and unsets, multi-operand direct array-offset unset sequencing, direct array appends, LLVM/generated-C string-int ABI consumers, string-offset reads/bool probes/writes, primitive arithmetic conversion, unary string-result execution, comparison relation routing, scalar output, selected type predicates, bitwise/shift operations, and focused verification gates.

The product is still far from full generalized PHP. The largest missing regions remain references/COW, executable lvalues, user calls and frames, object/class/property semantics, mutable globals/superglobals, include/require, exceptions/finally, exact diagnostics, and cleanup across real control flow.

## Done / In Progress / Not Done

- [x] Supervised parallel lanes and a primary integration gate are established.
- [x] Shared runtime ABI surfaces exist for strings, byte buffers, comparisons, numeric-string classification, array key/value operations, value-offset read/presence/mutation operations, request-state snapshots, selected conversions, selected diagnostics, branch-decision status/abort handling, native value bitwise/shift operations, native value type predicates, runtime string-byte materialization, typed bool extraction, diagnostic report/free, and string-offset read/write/bool results.
- [x] LLVM/generated-C consumers exist for selected primitive arithmetic, selected unary string-result builtins, selected generated-C array/string offset presence, selected LLVM string-offset reads and probes through the shared value-offset ABI, selected generated-C array-offset writes, direct array appends, direct and multi-operand direct array-offset unsets, plus string-offset writes through value-offset mutation, selected generated-C string-offset reads/bool probes, selected LLVM/generated-C string-int builtins, selected generated-C string-distance/path builtins, array key/value operations, array-handle value operands, array append diagnostics, comparison relation results, comparison abort guards, strict array/object identity in selected array-search builtins, native value bitwise/shift operations, scalar value echo/print output, print value-result output, native value type predicates, casts/type-name output, selected filesystem/cache blockers, and centralized call diagnostics.
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
| String conversion, truthiness, byte buffers | 79% | 88% | Primary has shared value string-form semantics, generated-C unary string-result execution, LLVM/generated-C string-int consumers through the string-int ABI, generated-C value-offset presence for string offsets, generated-C string-offset reads, LLVM string-offset reads and probes through the shared value-offset ABI, `isset`/`empty` bool probes, string-offset writes through the shared value-offset mutation ABI, warning continuation, scalar value output, generated-C print output, comparison byte materialization, runtime string-byte materialization, and raw-buffer writes. Lane-local work adds more LLVM string/debug/formatter parity, but it is not counted until integrated. |
| Call operation cleanup and ownership | 43% | 68% | Primary routes many call-result contexts, function declaration fallbacks, and backend call diagnostics through common contracts. Lane-local required-lvalue/discarded-result cleanup is broader; real frames, binding, returns, by-ref semantics, and dispatch remain mostly non-executable in primary. |
| Comparison and conversion semantics | 75% | 84% | Primary has reusable comparison validation, relation-result/result/branch/free/decision/status/abort ABIs, generated-C relation-result consumers, public operand routing, recursive-array blocker classification, string-handle operands, LLVM/generated-C byte string compare builtin consumers, native object/resource strict identity, primitive arithmetic conversion for known operands, scalar casts, bitwise/shift consumers, value-operation output, type predicates, unary string-result output, and string-offset execution slices. Lane-local conversion work is advancing concat/source-result paths; dynamic arithmetic, division/modulo warning parity, executable recursive array comparison, object property comparison, resource loose comparison, reference dereference comparison, and backend parity remain open. |
| Arrays, lvalues, references, COW | 35% | 85% | Primary has array-key materialization, array value-operation result ABI, generated-C array/string offset presence through the value-offset ABI, LLVM string-offset reads and probes through the same value-offset ABI, runtime value-offset mutation for write/append/unset families, generated-C direct array-offset writes, direct array appends, direct and multi-operand direct array-offset unsets, plus string-offset writes through that mutation ABI, array-value cloning back to owned handles, array-entry snapshots, array-handle comparisons, append diagnostics, native array handles as owned value operands, cloned-literal cleanup, and selected string-offset probes outside full array/lvalue semantics. Nested append/writeback, append expression values, LLVM array-offset reads/probes, full executable lvalues, references, and COW are not integrated. Lane-local lvalue/foreach/reference candidates are much stronger. |
| Symbols, globals, request state | 24% | 68% | Primary has symbol ABI helpers, request/superglobal snapshot ABI, and selected expression-result boundaries. Lane-local expression-result and symbol-cell work is promising, but primary still lacks real generalized locals, frames, globals, imports, request mutation, and reference assignment lowering. |
| Objects, properties, methods | 11% | 50% | Primary has native object handle strict-identity relation results and loose-comparison blockers through the shared comparison path. Lane-local object/class/property blockers and operation plans continue improving, but executable object/property/method behavior is still largely absent. |
| Diagnostics and control-flow cleanup | 25% | 70% | Primary has selected severity/blocker surfaces, diagnostic array append behavior, a shared native diagnostic report consumer, centralized call diagnostics, call-boundary cleanup routing, comparison branch abort handling, string-offset bool diagnostics, and warning-capable string-offset write continuation. Lane-local diagnostic sinks and terminating/control-flow cleanup boundaries are broader, but most loop/switch/goto/finally/exception behavior remains blocker/model work. |
| Broad composition verification | 49% | 50% | Focused runtime/native-link/native-runtime-ABI gates cover the newest LLVM string-offset value-offset read/probe consumer, multi-operand direct array-offset unset sequencing, direct array-offset unset consumer, LLVM string-int consumer, generated-C array-offset write consumer, value-offset mutation string-write consumer, value-offset array/string presence path, byte string compare builtin path, string-offset read path, shared diagnostic report consumer, string-offset warning continuation, string-offset write path, string-offset bool path, unary string-result executable path, comparison relation-result path, value-operation, scalar echo, print output, bitwise, type-predicate, array-value, string-byte, and diagnostic paths. Broad differential PHP composition coverage remains thin. |

## Recent Primary-Integrated Work

- `0568ebb1 codegen: route array appends through value mutation ABI`
  - Routes generated-native C direct `$array[] = $value` assignments through the shared value-offset mutation ABI with append operation tag `1`, null offset handles, diagnostic reporting, and array clone rematerialization. Focused native-link and runtime gates cover repeated appends, variable and literal replacements, follow-up offset reads, `isset()` composition, source routing, and linked executable behavior.
- `3b537955 codegen: route LLVM offset reads through value ABI`
  - Routes lowerable LLVM string offset reads and `isset()`/`empty()` probes through `phpc_native_value_offset_operation_with_diagnostic(...)`, then consumes returned native values through the shared bool and string/native-value materialization paths with cleanup. Focused native-runtime-ABI tests cover read/probe routing, string consumers, and assembly-backend reachability. LLVM array-offset reads/probes remain blocked.
- `c3573793 codegen: sequence multi-operand array unsets`
  - Routes generated-native C `unset(...)` statements with multiple direct array-index operands through repeated use of the shared array-offset mutation helper. Focused gates covered source routing, a linked executable with multiple dynamic keys, adjacent `array_offset` native-link regression tests, `cargo check -p phpc`, formatting, and diff checks.
- `df634f81 codegen: route array offset unset through value mutation ABI`
  - Routes generated-native C direct array-offset unsets through the shared value-offset mutation ABI with unset operation tag `2`, then clones the returned array value back into an owned array handle. Focused gates cover source routing, linked executable behavior with dynamic keys, missing-key no-op, follow-up rewrite, adjacent direct array-offset writes, runtime value-offset mutation, string-offset mutation regression, package checks, formatting, and diff checks.
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

Recent primary work is directionally sound because it turns shared ABI surfaces into concrete backend/runtime consumers and removes duplicated backend-local handling. The value-offset mutation path is the most strategically useful recent thread because it now reaches array writes, direct array-offset unsets, multi-operand direct array-offset unset sequencing, and direct array appends, pushing toward executable lvalue semantics; the LLVM string-int slice is useful backend parity for an already-landed runtime/generated-C surface.

The project remains boundary-heavy. More result/blocker vocabulary without immediate executable consumers will not move the product much. The `0568ebb1` slice is directionally useful because it consumes the already-landed value-offset mutation ABI from another generated-C array family and removes a direct append bypass, but it should not be overread as nested lvalue, assignment-expression, reference/COW, or backend parity support. The next valuable batches should each answer: what PHP behavior now runs that did not run before, what shared runtime materialization path now has real consumers, or what backend-local bypass has been removed?

## Near-Term Steering

1. Treat `0568ebb1` as the latest committed semantic baseline; `960babca`, `b9015b2d`, and evaluator dashboard commits are management artifacts.
2. Use precise steering language: generated-C direct array appends landed through the value-offset mutation ABI; nested append/writeback, append expression values, references/COW, LLVM parity, and object/ArrayAccess/resource offsets did not.
3. Avoid extending scalar/string work unless it removes another real backend-local bypass with executable gates.
4. Prefer small executable generated-C/LLVM/runtime consumers of existing ABI surfaces over more standalone vocabulary.
5. Strong next candidates: nested append/writeback or append assignment-expression values on the value-offset mutation boundary, a narrow reference/COW symbol-cell consumer, mutable request/superglobal storage through a shared root operation, generated-C/LLVM parity for value-offset mutation/unset beyond the landed read/probe surface, or a control-flow cleanup consumer that emits before a real terminal transfer.
6. Avoid whole-lane merges. Current lane evidence is broad and conflict-prone even when technically useful.
7. Keep resource checks explicit before broad gates; `/dev/shm` saturated to 100% during this evaluation window and required repeated process-aware cleanup.
