# PHP Native Compiler Progress

Updated: 2026-05-22 12:04 CEST
Evaluation marker: `20260522T100443Z`
Primary HEAD: `8865eb9c docs: update progress after value-offset writes`
Current primary semantic baseline: `90d8af3f codegen: route direct value-offset writes through mutation ABI`

Percentages are candid engineering estimates, not test-suite pass rates. Lane-local work is not counted as product capability until it is integrated into `master`, gated, committed, and pushed.

## Executive Read

Overall estimated progress toward a broadly usable generalized PHP native compiler: **65%** `[#############-------]`

Momentum is positive: primary has recently landed real generalized request/symbol runtime storage and generated-C array/lvalue consumers. The compiler is still not close to broad PHP completeness because executable `$GLOBALS`, mutable superglobals, references/COW, objects, calls, control flow, exact diagnostics, and broad composition remain major gaps.

## Roadmap Position

| Area | Estimate | Bar | Current primary-integrated read |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | 92% | `[##################--]` | Strong shared surfaces exist for values, arrays, strings, comparisons, diagnostics, symbol tables, request state, and reference slots. |
| Compiler/backend consumers | 97% | `[###################-]` | Many generated-C and LLVM paths consume shared ABIs; missing consumers now concentrate in symbols, calls, objects, references, and control flow. |
| Executable generalized PHP semantics | 79% | `[################----]` | Selected scalar, string, array, lvalue, symbol, and request runtime behavior works; ordinary broad PHP programs still hit major blockers. |
| Arrays, references, COW, lvalues | 66% | `[#############-------]` | Selected generated-C array/lvalue execution works, including pointer/cursor builtins and direct value-offset writes; arbitrary roots, full references/COW, by-reference foreach, and ArrayAccess/resource offsets remain open. |
| Symbols, globals, request state | 33% | `[#######-------------]` | Runtime symbol/request roots can snapshot, populate, and share reference cells; compiler-level `$GLOBALS`, superglobal mutation, request lifetime, and frame propagation are not done. |
| Objects, properties, methods | 11% | `[##------------------]` | Mostly blockers and metadata. Real allocation/property/method behavior remains largely absent. |
| Diagnostics and control-flow cleanup | 26% | `[#####---------------]` | Shared diagnostic/status surfaces exist, but exact ordering, recovery, loops/switch/goto/finally/exceptions, and cleanup stacks are not integrated. |
| Broad integrated verification | 67% | `[#############-------]` | Focused gates and linked tests are useful; broad differential composition coverage remains thin. |

## Primary-Integrated Capability

- [x] Shared runtime/value foundations for selected scalar, string, array, comparison, diagnostic, symbol, request, and reference-slot operations.
- [x] Generated-native C consumers for selected scalar/string/array/lvalue behaviors with linked executable coverage.
- [x] Request-state runtime snapshot/rebuild and symbol-table population ABIs.
- [x] Symbol-table and request-root reference-cell sharing at runtime.
- [x] Generated-native C pointer/cursor builtins over tracked native array owners and nested owner paths.
- [x] Generated-native C direct value-offset writes over missing/null/false/scalar roots through the shared mutation ABI.
- [ ] Executable PHP-level `$GLOBALS`, mutable superglobals, request lifetime threading, and frame propagation.
- [ ] General references/COW, owner-slot/value-slot/reference-slot materialization, by-reference arguments/returns, and alias-visible mutation barriers.
- [ ] Real object allocation, properties, methods, magic hooks, visibility, ArrayAccess, and resource offset behavior.
- [ ] General dynamic calls, user function frames, variadics/spreads, cleanup, and exact diagnostic ordering across control flow.

## Recent Primary Progress

Since the prior evaluator window, primary integrated and pushed:

- `9ca31007 runtime: populate request roots from symbol tables`
- `4aef9974 runtime: share symbol and request reference slots`
- `0f123f2d codegen: route array pointer builtins through lvalue ABI`
- `90d8af3f codegen: route direct value-offset writes through mutation ABI`

The current worktree is synced with `origin/master` at `8865eb9c`. The only expected dirty primary diff is the preserved unstaged `runtime/src/lib.rs` append/null-slot cleanup hunk; it is not counted as product progress.

## Lane-Local Candidate Work

These are active candidates, not integrated capability:

- `impl-array-lowering`: lazy `??=` RHS owner-cell storage-effect handling.
- `impl-array-value-runtime`: generated-C and LLVM single-known dynamic call value propagation.
- `impl-binary-string-runtime`: include-path execution and array replacement string operations.
- `impl-native-control-flow-seed`: shared control-flow target-block allocation.
- `impl-function-frame-seed`: typed parameter frame-slot binding contract.
- `impl-native-error-diagnostic-semantics`: diagnostic result/family contract expansion.
- `impl-native-reference-cell-runtime`: owner-cell production-realizer contract.
- `impl-native-object-seed`: declared class-name constant metadata routing.

## Active Roadmap Items

| Active item | Estimate | Status | Next useful primary shape |
| --- | ---: | --- | --- |
| Array/lvalue execution | 66% | In progress | Nested/arbitrary-root writes, `??=`, append/RMW, owner-slot/value-slot/reference-slot materialization, or LLVM parity. |
| Symbols/request/globals | 33% | In progress | Whole-bag request/superglobal reads, `isset()`/`empty()`, mutable storage, `$GLOBALS` aliasing, or request lifetime threading. |
| References/COW | 28% | In progress | Narrow owner/reference slot materialization with alias-visible mutation and executable evidence. |
| Calls/functions | 24% | In progress | Dynamic call lookup or user-frame value propagation beyond one known callable. |
| Objects/properties/methods | 11% | Early | Allocation/property/method behavior through shared carriers, not metadata-only blockers. |
| Control flow/cleanup/diagnostics | 26% | Early | One real structured transfer or cleanup path with exact ordering evidence. |
| Broad verification | 67% | In progress | Composition checks crossing semantic families after each primary consumer lands. |

## Steering Notes

The next best primary slices should turn landed runtime/storage surfaces into executable behavior. More standalone ABI vocabulary is lower value unless it directly unlocks a compiled PHP consumer. Whole-lane merges, fixture-shaped production lowering, generated-source substring-only progress, formatter spillover, and docs-only churn outside this progress dashboard should remain rejected.
