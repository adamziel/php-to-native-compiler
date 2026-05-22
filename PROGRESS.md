# PHP Native Compiler Progress

Updated: 2026-05-22 15:29 CEST
Evaluation marker: `20260522T132951Z`
Primary HEAD: `764cf014 runtime: add symbol-table nested write ABI`
Current primary semantic baseline: `764cf014 runtime: add symbol-table nested write ABI`

Percentages are candid engineering estimates, not test-suite pass rates. Lane-local candidate work and unstaged primary diffs are not counted as product capability until integrated into `master`, gated, committed, and pushed.

## Executive Read

Overall estimated progress toward the current generalized native-compiler roadmap: **74%** `[###############-----]`

Momentum remains positive, but the remaining blockers are structural. Primary now has stronger request/symbol storage, direct request-root assignment/snapshots, direct keyed request-superglobal read/write/unset/`isset()`/`empty()` storage, direct request-root value/reference storage, symbol-table nested write-by-path runtime storage, selected generated-C array/lvalue/string/native-value consumers, natural array sort execution through the lvalue sort ABI, a runtime ABI for nested array reference paths over owned value handles, and source-location metadata on shared native diagnostic handles. This is real generalized infrastructure. It is not broad PHP compatibility yet: executable `$GLOBALS`, nested/path superglobal lowering beyond direct keyed slots, generated reference assignment, full references/COW, user calls, objects, structured control flow, and exact diagnostic attachment/ordering remain major blockers.

## Roadmap Position

| Area | Estimate | Bar | Primary-integrated read |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | 96% | `[###################-]` | Strong shared surfaces exist for values, arrays, strings, comparisons, diagnostics, diagnostic source locations, symbol tables, request state, request-root direct values/references, reference slots, request/superglobal mutation, nested array reference paths, and array sort result families. |
| Selected compiler/backend consumers | 76% | `[###############-----]` | Generated-C consumers exist for selected scalar/string/array/lvalue/request-root behavior, including direct request-root assignment, direct keyed request storage, keyed request `empty()`, and lvalue-backed sort families. Symbols, nested/path request storage, calls, objects, references, and control-flow cleanup remain partial or absent. |
| Executable generalized PHP semantics | 65% | `[#############-------]` | Selected scalar, string, array, lvalue, symbol-runtime, request-runtime, direct request-root value/reference storage, direct root assignment, snapshots, keyed request storage including `empty()`, null-callback array value behavior, and natural sort execution works. Broad PHP programs still hit structural blockers. |
| Arrays, references, COW, lvalues | 71% | `[##############------]` | Primary has selected array/lvalue execution plus natural sort execution and a runtime nested reference-path ABI. Generated PHP reference assignment, arbitrary roots, owner/value/reference slots, by-reference foreach, and full COW remain open. |
| Symbols, globals, request state | 54% | `[###########---------]` | Runtime symbol/request roots can snapshot, mutate, store direct scalar/null/object/resource root values, store direct root reference cells, clear stale keyed slots, re-enter keyed storage where safe, write nested symbol-table paths through a shared runtime ABI, and generated C can assign direct request roots plus direct keyed request slots and keyed `empty()` through request-state ABIs. Compiler-level `$GLOBALS`, nested/path superglobal lowering beyond direct keyed slots, request lifetime, and frame propagation are not integrated. |
| Calls, functions, frames | 25% | `[#####---------------]` | Runtime call contracts and promising lane-local source-call/value-frame consumers exist, but primary still lacks broad executable user function/method/closure frames and result consumers. |
| Objects, properties, methods | 11% | `[##------------------]` | Mostly blockers, metadata, and lane-local scaffolds. Real allocation/property/method behavior remains largely absent. |
| Diagnostics and control-flow cleanup | 29% | `[######--------------]` | Shared diagnostic/status surfaces exist, diagnostics can carry source-location metadata, and request missing-key value reads report through request result carriers. Generated source-span attachment, exact ordering, recovery, loops/switch/goto/finally/exceptions, and cleanup stacks are not primary-integrated. |
| Broad integrated verification | 75% | `[###############-----]` | Focused gates and linked tests are useful, including request-root snapshots, direct request-root assignments, direct keyed request storage including `empty()`, null-callback array builtins, sort-family lvalues including natural sorts, and runtime reference-path storage-root coverage. Broad differential composition coverage remains thin. |

## Done / In Progress / Not Done

- [x] Shared runtime/value foundations for selected scalar, string, array, comparison, diagnostic, symbol, request, and reference-slot operations.
- [x] Generated-native C consumers for selected scalar/string/array/lvalue behaviors with linked executable coverage.
- [x] Request-state runtime snapshot/rebuild and symbol-table population ABIs.
- [x] Symbol-table and request-root reference-cell sharing at runtime.
- [x] Runtime request missing-key value-read diagnostics through the shared request operation-result carrier.
- [x] Runtime request/superglobal path mutation through shared request-state storage.
- [x] Generated-native C shell-escape calls through the shared native string-result ABI.
- [x] Generated-native C comparator-free array sort builtins over tracked native array owners and nested owner paths.
- [x] Runtime/generated-native C natural array sort execution for `natsort()` and `natcasesort()` through the shared array lvalue sort ABI.
- [x] Generated-native C null-callback `array_filter()` and `array_map()` through shared native value handles and the native array callback result ABI.
- [x] Generated-native C direct request-superglobal root snapshots through the request-state ABI, including root `isset()`, root `empty()`, type-name, and output consumers.
- [x] Generated-native C direct request-superglobal root assignments through the request-state replace-value ABI, including scalar, bool, array, and native string-result RHS values across multiple request bags.
- [x] Generated-native C direct keyed request-superglobal reads, writes, unsets, `isset()`, and `empty()` through request-state operation/mutation ABIs with arbitrary key expressions, native truthiness conversion, and linked executable coverage.
- [x] Runtime direct request-superglobal root values, including scalar/null/object/resource roots, stale keyed-slot clearing, keyed write re-entry, and scalar-root write rejection.
- [x] Runtime direct request-superglobal root references, including shared root reference cells across `_GET`, `_POST`, and `_REQUEST`, stale keyed-slot clearing, snapshot visibility after reference updates, and blocked keyed mutation while the root is reference-backed.
- [x] Runtime symbol-table nested write-by-path ABI over native value key paths, including missing/null/false parent materialization, scalar-parent diagnostics, and reference-backed root slots.
- [x] Runtime nested array reference-path ABI over owned value handles, with root writeback coverage for direct values, symbol-table values, and request-superglobal slots.
- [x] Runtime source-location metadata on shared native diagnostic handles, with clone/query ABI coverage across direct, conversion, and request diagnostics.
- [ ] Compiler-lowered nested/path request-superglobal reads, writes, unsets, assignment-expression values, and `$GLOBALS` request aliases in committed primary.
- [ ] Generated PHP reference assignment over the path-reference ABI.
- [ ] Executable PHP-level `$GLOBALS`, request lifetime threading, and frame propagation.
- [ ] General references/COW, owner-slot/value-slot/reference-slot materialization, by-reference arguments/returns, and alias-visible mutation barriers.
- [ ] Real object allocation, properties, methods, magic hooks, visibility, ArrayAccess, and resource offset behavior.
- [ ] General dynamic calls, user function frames, variadics/spreads, cleanup, and exact diagnostic ordering across control flow.

## Recent Primary Progress

Recent integrated semantic commits:

- `764cf014 runtime: add symbol-table nested write ABI`
- `ed2d9031 runtime: add array reference path ABI`
- `4d4158f3 runtime: store direct request root values`
- `a72df84f runtime: store direct request root references`
- `5600ff8a codegen: route request root assignments through state ABI`
- `7fc1d7ef codegen: route request keyed storage through state ABI`
- `e3a458bb runtime: execute natural array sorts through lvalue ABI`
- `625b98d0 codegen: route request keyed empty through state ABI`
- `42934793 runtime: attach source locations to diagnostics`

Current primary state:

- `master` has semantic baseline `764cf014`; this progress update is management-only on top.
- Live primary still has the preserved unstaged `runtime/src/lib.rs` null-slot increment/decrement behavior hunk; it is not counted as product progress until separately accepted or rejected with focused gates.
- The latest committed semantic addition is the runtime symbol-table nested write-by-path ABI. Generated compiler consumers for `$GLOBALS`/symbol paths, generated PHP source-span attachment, nested/path request operations, `$GLOBALS` reconciliation, request-root reference assignment, LLVM parity, PHP reference assignment, arbitrary writable roots, and full references/COW still need primary compiler consumers.

## Lane-Local Candidate Work

These are active or completed candidates, not integrated capability:

- `impl-array-linked-exec`: temporary array-expression reference roots now route through a shared value-root reference-result boundary, but value-root reference promotion still runtime-blocks. Useful as a centralization candidate; not executable reference semantics.
- `impl-array-lowering`: lazy RHS direct-variable storage creation is isolated behind a clearer blocker for skipped `??=` branches. Useful blocker hygiene; not direct variable storage execution.
- `impl-array-value-runtime`: byte-aware dynamic symbol names, extension metadata bytes, `call_user_func_array()` named-argument byte blockers, and call/value-frame surfaces. Valuable but large and conflict-prone; needs compact transplant notes before primary consideration.
- `impl-native-call-semantics`: generated-C/source-call value-vector work remains high-value if sliced into real by-value source-call argument/frame consumers, not just descriptor vocabulary.
- `impl-native-control-flow-seed`: control-flow candidates are strategically important only when they land executable branch/loop/cleanup behavior with state and cleanup evidence.
- `impl-native-reference-cell-runtime`: owner-cell scaffolds across `$GLOBALS`, object/static property arrays, reference returns, and destructuring remain promising, but need concrete backend builders and alias-visible mutation before counting.
- `impl-symbol-integrator`: LLVM supported named-call producer cleanup/blocker routing improves backend hygiene but does not yet provide dynamic call dispatch or user frames.

## Active Roadmap Items

| Active item | Estimate | Status | Next useful primary shape |
| --- | ---: | --- | --- |
| Array/lvalue execution | 71% | In progress | Consume the path-reference ABI in generated reference assignment, arbitrary-root writeback, by-reference foreach, `??=`/RMW, owner/value/reference-slot materialization, or LLVM parity. |
| Symbols/request/globals | 54% | In progress | Extend beyond runtime symbol-table nested writes and direct keyed request storage into compiler-lowered symbol paths, nested/path request reads/writes/unsets/`empty()`, `$GLOBALS` aliasing, request-root reference consumers, assignment-expression values, or request lifetime threading. |
| References/COW | 31% | In progress | Narrow owner/reference slot materialization with alias-visible mutation and executable evidence, not just runtime ABI vocabulary. |
| Calls/functions | 25% | In progress | Real declaration descriptor/callable table population with generated body callbacks, by-value source-call argument vectors, and result consumers. |
| Objects/properties/methods | 11% | Early | Allocation/property/method behavior through shared carriers, not metadata-only blockers. |
| Control flow/cleanup/diagnostics | 28% | Early | One real structured branch/loop/transfer cleanup path with exact ordering evidence. |
| Broad verification | 74% | In progress | Composition checks crossing request, symbols, arrays, references, calls, diagnostics, and control flow after each primary consumer lands. |

## Steering Notes

The next best primary slice should move beyond direct keyed request slots, diagnostic metadata, and sort-family breadth: nested/path superglobal reads, writes, unsets, `empty()`, `$GLOBALS` alias reconciliation, request-root reference assignment, request lifetime/frame threading, generated reference assignment over proven array/request parents, arbitrary-root/owner-slot/reference-slot lvalue materialization, or a narrow real call/control-flow execution slice. Avoid another standalone builtin-family batch unless it crosses references/COW, request state, function frames, object/ArrayAccess, resource behavior, or structured cleanup.

The live dirty `runtime/src/lib.rs` hunk should be explicitly classified by its owner. It is not part of the symbol-table nested write ABI batch and should not stay ambiguous background state.
