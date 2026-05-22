# PHP Native Compiler Progress

Updated: 2026-05-22 12:52 CEST
Evaluation marker: `20260522T105253Z`
Primary HEAD: `aaf3e263 docs: update progress after shell escapes`
Current primary semantic baseline: `01db5099 codegen: route shell escapes through string-result ABI`

Percentages are candid engineering estimates, not test-suite pass rates. Lane-local and primary-local candidate work is not counted as product capability until it is integrated into `master`, gated, committed, and pushed.

## Executive Read

Overall estimated progress toward the current generalized native-compiler roadmap: **66%** `[#############-------]`

Momentum is positive, but still uneven. The primary branch recently landed real request/symbol runtime storage, generated-C array/lvalue consumers, request missing-key diagnostics, runtime request/superglobal path mutation, and a narrow generated-C shell-escape consumer over the shared string-result ABI. Ordinary broad PHP program support still has major blockers in `$GLOBALS`, compiler-lowered superglobals, references/COW, calls, objects, control flow, exact diagnostics, and broad composition.

Current primary has an active staged array sort-family candidate. It is not counted below until it lands and is pushed.

## Roadmap Position

| Area | Estimate | Bar | Primary-integrated read |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | 93% | `[###################-]` | Strong shared surfaces exist for values, arrays, strings, comparisons, diagnostics, symbol tables, request state, reference slots, and request/superglobal path mutation. |
| Selected compiler/backend consumers | 70% | `[##############------]` | Many generated-C consumers exist, but important consumers for symbols, request state, calls, objects, references, and control-flow cleanup remain absent or partial. |
| Executable generalized PHP semantics | 56% | `[###########---------]` | Selected scalar, string, array, lvalue, symbol-runtime, and request-runtime behavior works. Broad PHP programs still hit structural blockers. |
| Arrays, references, COW, lvalues | 66% | `[#############-------]` | Selected generated-C array/lvalue execution works, including pointer/cursor builtins and direct value-offset writes. Arbitrary roots, full references/COW, by-reference foreach, ArrayAccess/resource offsets, and sort-family work are not fully landed. |
| Symbols, globals, request state | 38% | `[########------------]` | Runtime symbol/request roots can snapshot, populate, share reference cells, report missing-key value-read diagnostics, and mutate request/superglobal paths. Compiler-level `$GLOBALS`, superglobal expression lowering, request lifetime, and frame propagation are not done. |
| Calls, functions, frames | 25% | `[#####---------------]` | Runtime call contracts and lane-local source-call consumers exist, but primary still lacks broad executable user function/method/closure frames and result consumers. |
| Objects, properties, methods | 11% | `[##------------------]` | Mostly blockers and metadata. Real allocation/property/method behavior remains largely absent. |
| Diagnostics and control-flow cleanup | 28% | `[######--------------]` | Shared diagnostic/status surfaces exist and request missing-key value reads now report through the request result carrier. Exact ordering, recovery, loops/switch/goto/finally/exceptions, and cleanup stacks are not integrated. |
| Broad integrated verification | 67% | `[#############-------]` | Focused gates and linked tests are useful. Broad differential composition coverage remains thin. |

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
- [ ] Executable PHP-level `$GLOBALS`, compiler-lowered superglobal reads/writes/`isset()`/`empty()`, request lifetime threading, and frame propagation.
- [ ] General references/COW, owner-slot/value-slot/reference-slot materialization, by-reference arguments/returns, and alias-visible mutation barriers.
- [ ] Real object allocation, properties, methods, magic hooks, visibility, ArrayAccess, and resource offset behavior.
- [ ] General dynamic calls, user function frames, variadics/spreads, cleanup, and exact diagnostic ordering across control flow.

## Recent Primary Progress

Integrated and pushed in the latest evaluator window:

- `f0f785e5 runtime: report request missing-key value diagnostics`
- `dbc6b13c runtime: mutate request superglobal paths`
- `01db5099 codegen: route shell escapes through string-result ABI`

Earlier recent integrated work in this run includes request-root population, symbol/request reference-slot sharing, array pointer builtins, and direct value-offset writes.

Current product repo state:

- `master` and `origin/master` are synced at `aaf3e263`.
- Active staged primary-local candidate: array sort-family owner/path mutation ABI and generated-C consumer in `compiler/src/codegen.rs`, `compiler/tests/native_link.rs`, and `runtime/src/lib.rs`.
- Preserved unrelated unstaged runtime hunk remains in `runtime/src/lib.rs`.
- Neither the active candidate nor the preserved hunk is counted as product progress here.

## Lane-Local Candidate Work

These are active or completed candidates, not integrated capability:

- `impl-native-integration-batch`: array sort-family owner/path mutation ABI and generated-C consumer, currently visible as active primary-local candidate work.
- `impl-native-call-semantics`: generated-C echo/print statement-operand call consumers over the shared runtime call result handle.
- `impl-array-linked-exec`: callback-free `array_filter()` through the native value-result callback-family ABI, plus adjacent string-transform/sort-family surfaces.
- `impl-array-lowering`: centralized conditional RHS state-merge blocker for lazy `??=` operation families.
- `impl-binary-string-runtime`: include-path, constants, stream/filesystem, and shell/string-result follow-ons in lane-local history.
- `impl-native-type-conversion`, `impl-native-diagnostics`, `impl-native-control-flow-seed`, `impl-native-exit-seed`, object, reference-cell, and symbol lanes: useful generalized surfaces remain mostly lane-local or conflict-heavy until sliced.

## Active Roadmap Items

| Active item | Estimate | Status | Next useful primary shape |
| --- | ---: | --- | --- |
| Array/lvalue execution | 66% | In progress | Finish or reject the active sort-family candidate cleanly; then prefer arbitrary-root writes, by-reference foreach, `??=`/RMW, owner/value/reference-slot materialization, or LLVM parity. |
| Symbols/request/globals | 38% | In progress | Whole-bag request/superglobal reads, compiler-lowered mutations, `isset()`/`empty()`, `$GLOBALS` aliasing, or request lifetime threading. |
| References/COW | 28% | In progress | Narrow owner/reference slot materialization with alias-visible mutation and executable evidence. |
| Calls/functions | 25% | In progress | Real declaration descriptor/callable table population, dynamic call lookup, or user-frame value propagation beyond statement-only consumers. |
| Objects/properties/methods | 11% | Early | Allocation/property/method behavior through shared carriers, not metadata-only blockers. |
| Control flow/cleanup/diagnostics | 28% | Early | One real structured transfer or cleanup path with exact ordering evidence, plus request diagnostic consumers in generated backends. |
| Broad verification | 67% | In progress | Composition checks crossing request, symbols, arrays, references, calls, diagnostics, and control flow after each primary consumer lands. |

## Steering Notes

The next best primary slices should turn landed runtime/storage surfaces into executable compiler behavior, especially superglobal reads/writes/`isset()`/`empty()`, `$GLOBALS` aliasing, request lifetime threading, or reference/COW materialization. The active sort-family candidate can be useful if it stays operation-tag driven and cleanly verified, but it should not displace deeper symbol/request/reference/call/control-flow work for another full cycle. Whole-lane merges, fixture-shaped production lowering, generated-source substring-only progress, formatter spillover, and docs-only churn outside this progress dashboard remain rejected.
