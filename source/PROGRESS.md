# PHP Native Compiler Progress

Updated: 2026-05-22 14:38 CEST
Evaluation marker: `20260522T123802Z`
Primary HEAD: `7fc1d7ef codegen: route request keyed storage through state ABI`
Current primary semantic baseline: `7fc1d7ef codegen: route request keyed storage through state ABI`

Percentages are candid engineering estimates, not test-suite pass rates. Lane-local candidate work and unstaged primary diffs are not counted as product capability until integrated into `master`, gated, committed, and pushed.

## Executive Read

Overall estimated progress toward the current generalized native-compiler roadmap: **72%** `[##############------]`

Momentum is positive, but the remaining blockers are structural. Primary now has stronger request/symbol storage, generated-C direct request-root assignment, direct request-root snapshots, direct keyed request-superglobal read/write/unset/`isset()` storage, direct request-root value/reference storage, selected generated-C array/lvalue/string/native-value consumers, and a runtime ABI for nested array reference paths over owned value handles. This is real generalized infrastructure. It is not broad PHP compatibility yet: executable `$GLOBALS`, nested/path superglobal lowering, request `empty()`, generated reference assignment, full references/COW, user calls, objects, structured control flow, and exact diagnostics remain major blockers.

## Roadmap Position

| Area | Estimate | Bar | Primary-integrated read |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | 96% | `[###################-]` | Strong shared surfaces exist for values, arrays, strings, comparisons, diagnostics, symbol tables, request state, request-root direct values/references, reference slots, request/superglobal mutation, and nested array reference paths. |
| Selected compiler/backend consumers | 74% | `[###############-----]` | Generated-C consumers exist for selected scalar/string/array/lvalue/request-root behavior, including direct request-root assignment and direct keyed request storage. Symbols, nested/path request storage, calls, objects, references, and control-flow cleanup remain partial or absent. |
| Executable generalized PHP semantics | 63% | `[#############-------]` | Selected scalar, string, array, lvalue, symbol-runtime, request-runtime, direct request-root value/reference storage, direct root assignment, snapshots, keyed request storage, and null-callback array value behavior works. Broad PHP programs still hit structural blockers. |
| Arrays, references, COW, lvalues | 70% | `[##############------]` | Primary has selected array/lvalue execution plus a runtime nested reference-path ABI. Generated PHP reference assignment, arbitrary roots, owner/value/reference slots, by-reference foreach, and full COW remain open. |
| Symbols, globals, request state | 51% | `[##########----------]` | Runtime symbol/request roots can snapshot, mutate, store direct scalar/null/object/resource root values, store direct root reference cells, clear stale keyed slots, re-enter keyed storage where safe, and generated C can assign direct request roots plus direct keyed request slots through request-state ABIs. Compiler-level `$GLOBALS`, nested/path superglobal lowering, request lifetime, and frame propagation are not integrated. |
| Calls, functions, frames | 25% | `[#####---------------]` | Runtime call contracts and promising lane-local source-call consumers exist, but primary still lacks broad executable user function/method/closure frames and result consumers. |
| Objects, properties, methods | 11% | `[##------------------]` | Mostly blockers, metadata, and lane-local scaffolds. Real allocation/property/method behavior remains largely absent. |
| Diagnostics and control-flow cleanup | 28% | `[######--------------]` | Shared diagnostic/status surfaces exist and request missing-key value reads report through request result carriers. Exact ordering, recovery, loops/switch/goto/finally/exceptions, and cleanup stacks are not primary-integrated. |
| Broad integrated verification | 73% | `[###############-----]` | Focused gates and linked tests are useful, including request-root snapshots, direct request-root assignments, direct keyed request storage, null-callback array builtins, sort-family lvalues, and runtime reference-path storage-root coverage. Broad differential composition coverage remains thin. |

## Done / In Progress / Not Done

- [x] Shared runtime/value foundations for selected scalar, string, array, comparison, diagnostic, symbol, request, and reference-slot operations.
- [x] Generated-native C consumers for selected scalar/string/array/lvalue behaviors with linked executable coverage.
- [x] Request-state runtime snapshot/rebuild and symbol-table population ABIs.
- [x] Symbol-table and request-root reference-cell sharing at runtime.
- [x] Runtime request missing-key value-read diagnostics through the shared request operation-result carrier.
- [x] Runtime request/superglobal path mutation through shared request-state storage.
- [x] Generated-native C shell-escape calls through the shared native string-result ABI.
- [x] Generated-native C comparator-free array sort builtins over tracked native array owners and nested owner paths.
- [x] Generated-native C null-callback `array_filter()` and `array_map()` through shared native value handles and the native array callback result ABI.
- [x] Generated-native C direct request-superglobal root snapshots through the request-state ABI, including root `isset()`, root `empty()`, type-name, and output consumers.
- [x] Generated-native C direct request-superglobal root assignments through the request-state replace-value ABI, including scalar, bool, array, and native string-result RHS values across multiple request bags.
- [x] Generated-native C direct keyed request-superglobal reads, writes, unsets, and `isset()` through request-state operation/mutation ABIs with arbitrary key expressions and linked executable coverage.
- [x] Runtime direct request-superglobal root values, including scalar/null/object/resource roots, stale keyed-slot clearing, keyed write re-entry, and scalar-root write rejection.
- [x] Runtime direct request-superglobal root references, including shared root reference cells across `_GET`, `_POST`, and `_REQUEST`, stale keyed-slot clearing, snapshot visibility after reference updates, and blocked keyed mutation while the root is reference-backed.
- [x] Runtime nested array reference-path ABI over owned value handles, with root writeback coverage for direct values, symbol-table values, and request-superglobal slots.
- [ ] Compiler-lowered nested/path request-superglobal reads, writes, unsets, `empty()`, assignment-expression values, and `$GLOBALS` request aliases in committed primary.
- [ ] Generated PHP reference assignment over the path-reference ABI.
- [ ] Executable PHP-level `$GLOBALS`, request lifetime threading, and frame propagation.
- [ ] General references/COW, owner-slot/value-slot/reference-slot materialization, by-reference arguments/returns, and alias-visible mutation barriers.
- [ ] Real object allocation, properties, methods, magic hooks, visibility, ArrayAccess, and resource offset behavior.
- [ ] General dynamic calls, user function frames, variadics/spreads, cleanup, and exact diagnostic ordering across control flow.

## Recent Primary Progress

Recent integrated semantic commits:

- `ed2d9031 runtime: add array reference path ABI`
- `4d4158f3 runtime: store direct request root values`
- `a72df84f runtime: store direct request root references`
- `5600ff8a codegen: route request root assignments through state ABI`
- `7fc1d7ef codegen: route request keyed storage through state ABI`

Current primary state:

- `master` and `origin/master` are synced at `7fc1d7ef`.
- This progress update is docs-only on top of semantic baseline `7fc1d7ef`.
- Active dirty implementation state is the preserved unrelated unstaged runtime hunk in `runtime/src/lib.rs`; it is not counted as product progress until gated, committed, and pushed.
- The latest committed semantic addition is generated-C direct keyed request-superglobal read/write/unset/`isset()` storage through request-state operation and mutation ABIs. Nested/path request operations, keyed `empty()`, `$GLOBALS` reconciliation, request-root reference assignment, LLVM parity, and PHP reference assignment still need primary compiler consumers.

## Lane-Local Candidate Work

These are active or completed candidates, not integrated capability:

- `impl-native-call-semantics`: generated-C source-call argument vectors through `phpc_native_call_arguments_from_values_with_diagnostic_and_free(...)`; high value if sliced narrowly into primary.
- `impl-native-control-flow-seed`: executable LLVM loop condition/backedge lowering for state-stable loops; still missing loop-carried state, phis, transfer cleanup, switch/goto, and break/continue.
- `impl-native-reference-cell-runtime`: owner-cell backend scaffolds across `$GLOBALS`, object/static property arrays, reference returns, and destructuring; still waiting on concrete backend builders and operand providers.
- `impl-array-value-runtime`: `call_user_func_array()` value-frame ABI and byte/PCRE surfaces; useful but large and conflict-prone without compact transplant notes.
- `impl-native-integration-batch` and `impl-array-linked-exec`: additional array/string/native-value builtin consumers; useful, but lower priority than structural request/reference/call/control-flow work unless they cross those boundaries.

## Active Roadmap Items

| Active item | Estimate | Status | Next useful primary shape |
| --- | ---: | --- | --- |
| Array/lvalue execution | 70% | In progress | Consume the path-reference ABI in generated reference assignment, arbitrary-root writeback, by-reference foreach, `??=`/RMW, owner/value/reference-slot materialization, or LLVM parity. |
| Symbols/request/globals | 51% | In progress | Extend beyond direct keyed request storage into nested/path request reads/writes/unsets, keyed `empty()`, `$GLOBALS` aliasing, request-root reference consumers, or request lifetime threading. |
| References/COW | 31% | In progress | Narrow owner/reference slot materialization with alias-visible mutation and executable evidence, not just runtime ABI vocabulary. |
| Calls/functions | 25% | In progress | Real declaration descriptor/callable table population with generated body callbacks, by-value source-call argument vectors, and result consumers. |
| Objects/properties/methods | 11% | Early | Allocation/property/method behavior through shared carriers, not metadata-only blockers. |
| Control flow/cleanup/diagnostics | 28% | Early | One real structured branch/loop/transfer cleanup path with exact ordering evidence. |
| Broad verification | 73% | In progress | Composition checks crossing request, symbols, arrays, references, calls, diagnostics, and control flow after each primary consumer lands. |

## Steering Notes

The next best primary slice should move beyond direct keyed request slots: nested/path superglobal reads, writes, unsets, keyed `empty()`, `$GLOBALS` alias reconciliation, request-root reference assignment, request lifetime/frame threading, generated reference assignment over proven array/request parents, or a narrow real call/control-flow execution slice. Avoid another standalone builtin-family batch unless it crosses references/COW, request state, function frames, object/ArrayAccess, or structured cleanup. Whole-lane merges, fixture-shaped production lowering, generated-source substring-only proof, formatter spillover, and docs-only churn outside this dashboard remain rejected.
