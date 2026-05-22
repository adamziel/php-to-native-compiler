# PHP Native Compiler Progress

Updated: 2026-05-22 13:19 CEST
Evaluation marker: `20260522T105253Z` plus post-evaluator semantic refresh through `161a7006`
Primary HEAD: current progress-only update on top of `161a7006`
Current primary semantic baseline: `161a7006 codegen: route null array callbacks through value ABI`

Percentages are candid engineering estimates, not test-suite pass rates. Lane-local and primary-local candidate work is not counted as product capability until it is integrated into `master`, gated, committed, and pushed.

## Executive Read

Overall estimated progress toward the current generalized native-compiler roadmap: **67%** `[#############-------]`

Momentum is positive, but still uneven. The primary branch recently landed real request/symbol runtime storage, generated-C array/lvalue consumers, request missing-key diagnostics, runtime request/superglobal path mutation, shell-escape string-result routing, comparator-free array sort-family routing over the array-lvalue ABI, and now generated-C null-callback `array_filter()` / `array_map()` execution through the native value-result ABI. Ordinary broad PHP program support still has major blockers in `$GLOBALS`, compiler-lowered superglobals, references/COW, calls, objects, control flow, exact diagnostics, and broad composition.

## Roadmap Position

| Area | Estimate | Bar | Primary-integrated read |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | 93% | `[###################-]` | Strong shared surfaces exist for values, arrays, strings, comparisons, diagnostics, symbol tables, request state, reference slots, and request/superglobal path mutation. |
| Selected compiler/backend consumers | 71% | `[##############------]` | Many generated-C consumers exist, including null-callback array value-result consumers, but important consumers for symbols, request state, calls, objects, references, and control-flow cleanup remain absent or partial. |
| Executable generalized PHP semantics | 58% | `[############--------]` | Selected scalar, string, array, lvalue, symbol-runtime, request-runtime, and null-callback array value behavior works. Broad PHP programs still hit structural blockers. |
| Arrays, references, COW, lvalues | 68% | `[##############------]` | Selected generated-C array/lvalue/value execution works, including pointer/cursor builtins, direct value-offset writes, comparator-free sort-family mutation, and null-callback `array_filter()` / `array_map()` over native value handles. Arbitrary roots, full references/COW, by-reference foreach, and ArrayAccess/resource offsets remain open. |
| Symbols, globals, request state | 38% | `[########------------]` | Runtime symbol/request roots can snapshot, populate, share reference cells, report missing-key value-read diagnostics, and mutate request/superglobal paths. Compiler-level `$GLOBALS`, superglobal expression lowering, request lifetime, and frame propagation are not done. |
| Calls, functions, frames | 25% | `[#####---------------]` | Runtime call contracts and lane-local source-call consumers exist, but primary still lacks broad executable user function/method/closure frames and result consumers. |
| Objects, properties, methods | 11% | `[##------------------]` | Mostly blockers and metadata. Real allocation/property/method behavior remains largely absent. |
| Diagnostics and control-flow cleanup | 28% | `[######--------------]` | Shared diagnostic/status surfaces exist and request missing-key value reads now report through the request result carrier. Exact ordering, recovery, loops/switch/goto/finally/exceptions, and cleanup stacks are not integrated. |
| Broad integrated verification | 69% | `[##############------]` | Focused gates and linked tests are useful, including linked executable coverage for null-callback array builtins. Broad differential composition coverage remains thin. |

## Done / In Progress / Not Done

- [x] Shared runtime/value foundations for selected scalar, string, array, comparison, diagnostic, symbol, request, and reference-slot operations.
- [x] Generated-native C consumers for selected scalar/string/array/lvalue behaviors with linked executable coverage.
- [x] Request-state runtime snapshot/rebuild and symbol-table population ABIs.
- [x] Symbol-table and request-root reference-cell sharing at runtime.
- [x] Generated-native C pointer/cursor builtins over tracked native array owners and nested owner paths.
- [x] Generated-native C direct value-offset writes over missing/null/false/scalar roots through the shared mutation ABI.
- [x] Runtime request missing-key value-read diagnostics through the shared request operation-result carrier.
- [x] Runtime request/superglobal path mutation through shared request-state storage.
- [x] Generated-native C shell-escape calls through the shared native string-result ABI with runtime diagnostics and linked executable coverage.
- [x] Generated-native C comparator-free array sort builtins over tracked native array owners and nested owner paths through the array-lvalue result ABI.
- [x] Generated-native C null-callback `array_filter()` and `array_map()` through shared native value handles and the native array callback result ABI.
- [ ] Executable PHP-level `$GLOBALS`, compiler-lowered superglobal reads/writes/`isset()`/`empty()`, request lifetime threading, and frame propagation.
- [ ] General references/COW, owner-slot/value-slot/reference-slot materialization, by-reference arguments/returns, and alias-visible mutation barriers.
- [ ] Real object allocation, properties, methods, magic hooks, visibility, ArrayAccess, and resource offset behavior.
- [ ] General dynamic calls, user function frames, variadics/spreads, cleanup, and exact diagnostic ordering across control flow.

## Recent Primary Progress

Integrated and pushed in the latest evaluator window:

- `f0f785e5 runtime: report request missing-key value diagnostics`
- `dbc6b13c runtime: mutate request superglobal paths`
- `01db5099 codegen: route shell escapes through string-result ABI`
- `f3c0f574 codegen: route array sort builtins through lvalue ABI`
- `161a7006 codegen: route null array callbacks through value ABI`

Earlier recent integrated work in this run includes request-root population, symbol/request reference-slot sharing, array pointer builtins, and direct value-offset writes.

Current product repo state:

- `master` has semantic progress through `161a7006`; this dashboard update is progress-only.
- Null-callback array value-result routing for `array_filter()` and `array_map()` is integrated in `compiler/src/codegen.rs`, `compiler/tests/native_link.rs`, and `runtime/src/lib.rs`.
- Preserved unrelated unstaged runtime hunk remains in `runtime/src/lib.rs`.
- The preserved hunk is not counted as product progress here.

## Lane-Local Candidate Work

These are active or completed candidates, not integrated capability:

- `impl-native-integration-batch`: array sort-family owner/path mutation and null-callback array value-result consumers have been sliced into primary; broader lane work remains excluded.
- `impl-native-call-semantics`: generated-C echo/print statement-operand call consumers over the shared runtime call result handle.
- `impl-array-linked-exec`: callback-free `array_filter()` through the native value-result callback-family ABI, plus adjacent string-transform/sort-family surfaces.
- `impl-array-lowering`: centralized conditional RHS state-merge blocker for lazy `??=` operation families.
- `impl-binary-string-runtime`: include-path, constants, stream/filesystem, and shell/string-result follow-ons in lane-local history.
- `impl-native-type-conversion`, `impl-native-diagnostics`, `impl-native-control-flow-seed`, `impl-native-exit-seed`, object, reference-cell, and symbol lanes: useful generalized surfaces remain mostly lane-local or conflict-heavy until sliced.

## Active Roadmap Items

| Active item | Estimate | Status | Next useful primary shape |
| --- | ---: | --- | --- |
| Array/lvalue execution | 68% | In progress | Prefer arbitrary-root writes, by-reference foreach, `??=`/RMW, owner/value/reference-slot materialization, callback/user-frame execution, LLVM parity, or deeper cleanup/diagnostic ordering over another null-callback/builtin breadth slice. |
| Symbols/request/globals | 38% | In progress | Whole-bag request/superglobal reads, compiler-lowered mutations, `isset()`/`empty()`, `$GLOBALS` aliasing, or request lifetime threading. |
| References/COW | 28% | In progress | Narrow owner/reference slot materialization with alias-visible mutation and executable evidence. |
| Calls/functions | 25% | In progress | Real declaration descriptor/callable table population, dynamic call lookup, or user-frame value propagation beyond statement-only consumers. |
| Objects/properties/methods | 11% | Early | Allocation/property/method behavior through shared carriers, not metadata-only blockers. |
| Control flow/cleanup/diagnostics | 28% | Early | One real structured transfer or cleanup path with exact ordering evidence, plus request diagnostic consumers in generated backends. |
| Broad verification | 67% | In progress | Composition checks crossing request, symbols, arrays, references, calls, diagnostics, and control flow after each primary consumer lands. |

## Steering Notes

The next best primary slices should turn landed runtime/storage surfaces into executable compiler behavior, especially superglobal reads/writes/`isset()`/`empty()`, `$GLOBALS` aliasing, request lifetime threading, or reference/COW materialization. Sort-family and null-callback array routing are counted as real but narrow generated-C array/value consumers, not a reason to keep prioritizing builtin breadth over deeper symbol/request/reference/call/control-flow work. Whole-lane merges, fixture-shaped production lowering, generated-source substring-only progress, formatter spillover, and docs-only churn outside this progress dashboard remain rejected.
