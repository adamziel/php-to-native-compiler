# PHP Native Compiler Progress

Updated: 2026-05-22 16:19 CEST
Evaluation marker: `20260522T141900Z`
Primary HEAD: progress update on top of `15657b95 codegen: route request path mutations through state ABI`
Current pushed semantic baseline: `15657b95 codegen: route request path mutations through state ABI`

Percentages are candid engineering estimates, not test-suite pass rates. Lane-local candidate work and unstaged primary diffs are not counted as product capability until integrated into `master`, gated, committed, and pushed.

## Executive Read

Overall estimated progress toward the current generalized native-compiler roadmap: **77%** `[###############-----]`

Momentum remains positive, but the remaining blockers are structural. Pushed primary now has stronger request/symbol storage, direct `$GLOBALS` root snapshots, direct request-root assignment/snapshots, direct keyed request-superglobal read/write/unset/`isset()`/`empty()` storage, generated-C nested/path request-superglobal writes and unsets, direct request-root value/reference storage, runtime symbol-table nested write-by-path storage, selected generated-C array/lvalue/string/native-value consumers, lvalue-backed sort and array mutation builtin families, a runtime ABI for nested array reference paths over owned value handles, and source-location metadata on shared native diagnostic handles. This is real generalized infrastructure, but not broad PHP compatibility yet: full `$GLOBALS` aliasing/mutation, nested/path superglobal reads/probes/`empty()`, generated reference assignment, full references/COW, user calls, objects, structured control flow, and exact diagnostic attachment/ordering remain major blockers.

## Roadmap Position

| Area | Estimate | Bar | Primary-integrated read |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | 96% | `[###################-]` | Strong pushed surfaces exist for values, arrays, strings, comparisons, diagnostics, diagnostic source locations, symbol tables, symbol-table nested write paths, request state, request-root direct values/references, reference slots, request/superglobal mutation, nested array reference paths, and array sort result families. |
| Selected compiler/backend consumers | 79% | `[################----]` | Generated-C consumers exist for selected scalar/string/array/lvalue/request-root behavior, including direct `$GLOBALS` root snapshots, direct request-root assignment, direct keyed request storage, keyed request `empty()`, nested/path request writes and unsets, lvalue-backed sort families, and lvalue-backed `array_push`/`array_pop`/`array_shift`/`array_unshift`. Symbol paths, nested/path request reads/probes, calls, objects, references, and control-flow cleanup remain partial or absent. |
| Executable generalized PHP semantics | 68% | `[##############------]` | Selected scalar, string, array, lvalue, direct `$GLOBALS` root snapshots, symbol-runtime, request-runtime, direct request-root value/reference storage, direct root assignment, snapshots, keyed request storage including `empty()`, nested/path request writes/unsets, null-callback array value behavior, natural sort execution, and array mutation builtin execution works. Broad PHP programs still hit structural blockers. |
| Arrays, references, COW, lvalues | 73% | `[###############-----]` | Primary has selected array/lvalue execution plus natural sort and array mutation execution through shared owner/path ABIs, and a runtime nested reference-path ABI. Generated PHP reference assignment, arbitrary roots, owner/value/reference slots, by-reference foreach, and full COW remain open. |
| Symbols, globals, request state | 59% | `[############--------]` | Pushed runtime symbol/request roots can snapshot, mutate, store direct scalar/null/object/resource root values, store direct root reference cells, clear stale keyed slots, re-enter keyed storage where safe, write nested symbol-table paths through a shared runtime ABI, and generated C can materialize direct `$GLOBALS` root snapshots, assign direct request roots, route direct keyed request slots plus keyed `empty()`, and execute nested/path request-superglobal writes/unsets through request-state ABIs. `$GLOBALS` aliasing/mutation, symbol paths, nested/path superglobal reads/probes, request lifetime, and frame propagation are not integrated. |
| Calls, functions, frames | 25% | `[#####---------------]` | Runtime call contracts and promising lane-local source-call/value-frame consumers exist, but primary still lacks broad executable user function/method/closure frames and result consumers. |
| Objects, properties, methods | 11% | `[##------------------]` | Mostly blockers, metadata, and lane-local scaffolds. Real allocation/property/method behavior remains largely absent. |
| Diagnostics and control-flow cleanup | 29% | `[######--------------]` | Shared diagnostic/status surfaces exist, diagnostics can carry source-location metadata, and request missing-key value reads report through request result carriers. Generated source-span attachment, exact ordering, recovery, loops/switch/goto/finally/exceptions, and cleanup stacks are not primary-integrated. |
| Broad integrated verification | 77% | `[###############-----]` | Focused gates and linked tests are useful, including request-root snapshots, direct request-root assignments, direct keyed request storage including `empty()`, nested/path request writes/unsets, null-callback array builtins, sort-family lvalues including natural sorts, array mutation lvalue consumers, and runtime reference-path storage-root coverage. Broad differential composition coverage remains thin. |

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
- [x] Runtime/generated-native C `array_push()`, `array_pop()`, `array_shift()`, and `array_unshift()` over tracked native array owners and nested owner paths through the shared array lvalue owner/path result ABI.
- [x] Generated-native C null-callback `array_filter()` and `array_map()` through shared native value handles and the native array callback result ABI.
- [x] Generated-native C direct request-superglobal root snapshots through the request-state ABI, including root `isset()`, root `empty()`, type-name, and output consumers.
- [x] Generated-native C direct request-superglobal root assignments through the request-state replace-value ABI, including scalar, bool, array, and native string-result RHS values across multiple request bags.
- [x] Generated-native C direct keyed request-superglobal reads, writes, unsets, `isset()`, and `empty()` through request-state operation/mutation ABIs with arbitrary key expressions, native truthiness conversion, and linked executable coverage.
- [x] Generated-native C nested/path request-superglobal writes and unsets through the request-state path mutation ABI with arbitrary key expressions, shared key materialization/status, diagnostics, and linked executable coverage.
- [x] Runtime direct request-superglobal root values, including scalar/null/object/resource roots, stale keyed-slot clearing, keyed write re-entry, and scalar-root write rejection.
- [x] Runtime direct request-superglobal root references, including shared root reference cells across `_GET`, `_POST`, and `_REQUEST`, stale keyed-slot clearing, snapshot visibility after reference updates, and blocked keyed mutation while the root is reference-backed.
- [x] Runtime nested array reference-path ABI over owned value handles, with root writeback coverage for direct values, symbol-table values, and request-superglobal slots.
- [x] Runtime source-location metadata on shared native diagnostic handles, with clone/query ABI coverage across direct, conversion, and request diagnostics.
- [x] Runtime symbol-table nested write-by-path ABI over native value key paths, including missing/null/false parent materialization, scalar-parent diagnostics, invalid path/key diagnostics, and reference-backed root slots.
- [x] Generated-native C direct `$GLOBALS` root snapshots through the symbol-table snapshot ABI, with current root variable materialization, owned snapshot storage, direct value consumers, and linked executable coverage.
- [ ] Compiler consumers for runtime symbol-table nested write paths, including `$GLOBALS[$expr]`/symbol path lowering.
- [ ] Compiler-lowered nested/path request-superglobal reads, probes, `empty()`, assignment-expression values, and `$GLOBALS` request aliases in committed/pushed primary.
- [ ] Generated PHP reference assignment over the path-reference ABI.
- [ ] Full PHP-level `$GLOBALS` aliasing/mutation, request lifetime threading, and frame propagation.
- [ ] General references/COW, owner-slot/value-slot/reference-slot materialization, by-reference arguments/returns, and alias-visible mutation barriers.
- [ ] Real object allocation, properties, methods, magic hooks, visibility, ArrayAccess, and resource offset behavior.
- [ ] General dynamic calls, user function frames, variadics/spreads, cleanup, and exact diagnostic ordering across control flow.

## Recent Primary Progress

Recent pushed semantic commits:

- `15657b95 codegen: route request path mutations through state ABI`
- `3bda4f51 codegen: route array mutation builtins through lvalue ABI`
- `d7fc807d codegen: materialize direct $GLOBALS snapshots`
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

- `master` is pushed through the generated-C request path mutation commit, with this progress update as a docs-only follow-up.
- The latest pushed semantic baseline after this batch is `15657b95`.
- Live primary currently has only the preserved unstaged `runtime/src/lib.rs` null-slot increment/decrement hunk. That hunk is not counted as product progress until classified, gated, committed, and pushed.
- The latest semantic addition is generated-native C nested/path request-superglobal write and unset execution through the request-state path mutation ABI across arbitrary key expressions and multiple request bags. Generated compiler consumers for `$GLOBALS[$expr]`/symbol paths, `$GLOBALS` alias reconciliation and mutation, generated PHP source-span attachment, nested/path request reads/probes/`empty()`, request-root reference assignment, LLVM parity, PHP reference assignment, arbitrary writable roots, owner/value/reference slots, object/ArrayAccess/resource offsets, and full references/COW still need primary compiler consumers.

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
| Array/lvalue execution | 73% | In progress | Consume the path-reference ABI in generated reference assignment, arbitrary-root writeback, by-reference foreach, `??=`/RMW beyond tracked owners, owner/value/reference-slot materialization, object/ArrayAccess/resource offsets, or LLVM parity. |
| Symbols/request/globals | 59% | In progress | Extend beyond direct `$GLOBALS` root snapshots, runtime symbol-table nested writes, direct keyed request storage, and nested/path request writes/unsets into compiler-lowered symbol paths, nested/path request reads/probes/`empty()`, `$GLOBALS` aliasing/mutation, request-root reference consumers, assignment-expression values, or request lifetime threading. |
| References/COW | 31% | In progress | Narrow owner/reference slot materialization with alias-visible mutation and executable evidence, not just runtime ABI vocabulary. |
| Calls/functions | 25% | In progress | Real declaration descriptor/callable table population with generated body callbacks, by-value source-call argument vectors, and result consumers. |
| Objects/properties/methods | 11% | Early | Allocation/property/method behavior through shared carriers, not metadata-only blockers. |
| Control flow/cleanup/diagnostics | 28% | Early | One real structured branch/loop/transfer cleanup path with exact ordering evidence. |
| Broad verification | 77% | In progress | Composition checks crossing request, symbols, arrays, references, calls, diagnostics, and control flow after each primary consumer lands. |

## Steering Notes

The next best primary slice should move beyond direct keyed request slots, nested/path request writes/unsets, direct `$GLOBALS` root snapshots, diagnostic metadata, sort-family breadth, and tracked-owner array mutation builtins: `$GLOBALS[$expr]`/symbol path mutation, nested/path superglobal reads/probes/`empty()`, `$GLOBALS` alias reconciliation, request-root reference assignment, request lifetime/frame threading, generated reference assignment over proven array/request parents, arbitrary-root/owner-slot/reference-slot lvalue materialization, object/ArrayAccess/resource offsets, LLVM array/lvalue parity, or a narrow real call/control-flow execution slice. Avoid another standalone builtin-family batch unless it crosses references/COW, request state, function frames, object/ArrayAccess, resource behavior, or structured cleanup.

The live dirty `runtime/src/lib.rs` null-slot hunk should be explicitly classified by its owner. It is not part of the pushed symbol-table nested write ABI batch and should not stay ambiguous background state.
