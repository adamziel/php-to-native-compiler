# PHP Native Compiler Progress

Updated: 2026-05-22 13:45 CEST
Evaluation marker: `20260522T114515Z`
Primary HEAD: `ed2d9031 runtime: add array reference path ABI`
Current primary semantic baseline: `ed2d9031 runtime: add array reference path ABI`

Percentages are candid engineering estimates, not test-suite pass rates. Lane-local candidate work is not counted as product capability until it is integrated into `master`, gated, committed, and pushed.

## Executive Read

Overall estimated progress toward the current generalized native-compiler roadmap: **69%** `[##############------]`

Momentum remains positive and still uneven. Primary now has stronger request/symbol storage, generated-C array/lvalue consumers, direct request-root snapshot consumers, null-callback array value consumers, and a new runtime ABI for nested array reference paths over owned value handles. This is real generalized infrastructure. It is not yet broad PHP compatibility: executable `$GLOBALS`, keyed/path superglobal lowering, generated reference assignment, references/COW, user calls, objects, structured control flow, and exact diagnostics remain the major blockers.

## Roadmap Position

| Area | Estimate | Bar | Primary-integrated read |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | 94% | `[###################-]` | Strong shared surfaces exist for values, arrays, strings, comparisons, diagnostics, symbol tables, request state, reference slots, request/superglobal mutation, and nested array reference paths. |
| Selected compiler/backend consumers | 72% | `[##############------]` | Generated-C consumers exist for selected scalar/string/array/lvalue/request-root behavior, but symbols, keyed/path request storage, calls, objects, references, and control-flow cleanup remain partial or absent. |
| Executable generalized PHP semantics | 59% | `[############--------]` | Selected scalar, string, array, lvalue, symbol-runtime, request-runtime, request-root snapshot, and null-callback array value behavior works. Broad PHP programs still hit structural blockers. |
| Arrays, references, COW, lvalues | 70% | `[##############------]` | Primary has selected array/lvalue execution plus a runtime nested reference-path ABI. Generated PHP reference assignment, arbitrary roots, owner/value/reference slots, by-reference foreach, and full COW remain open. |
| Symbols, globals, request state | 42% | `[########------------]` | Runtime symbol/request roots can snapshot, mutate, share reference cells, and participate in reference-path ABI tests. Compiler-level `$GLOBALS`, keyed/path superglobal lowering, request lifetime, and frame propagation are not done. |
| Calls, functions, frames | 25% | `[#####---------------]` | Runtime call contracts and promising lane-local source-call consumers exist, but primary still lacks broad executable user function/method/closure frames and result consumers. |
| Objects, properties, methods | 11% | `[##------------------]` | Mostly blockers and metadata. Real allocation/property/method behavior remains largely absent. |
| Diagnostics and control-flow cleanup | 28% | `[######--------------]` | Shared diagnostic/status surfaces exist and request missing-key value reads report through request result carriers. Exact ordering, recovery, loops/switch/goto/finally/exceptions, and cleanup stacks are not primary-integrated. |
| Broad integrated verification | 71% | `[##############------]` | Focused gates and linked tests are useful, including request-root snapshots, null-callback array builtins, sort-family lvalues, and runtime reference-path storage-root coverage. Broad differential composition coverage remains thin. |

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
- [x] Runtime nested array reference-path ABI over owned value handles, with root writeback coverage for direct values, symbol-table values, and request-superglobal slots.
- [ ] Generated PHP reference assignment over the new path-reference ABI.
- [ ] Executable PHP-level `$GLOBALS`, compiler-lowered keyed/path superglobal reads/writes/`isset()`/`empty()`, request lifetime threading, and frame propagation.
- [ ] General references/COW, owner-slot/value-slot/reference-slot materialization, by-reference arguments/returns, and alias-visible mutation barriers.
- [ ] Real object allocation, properties, methods, magic hooks, visibility, ArrayAccess, and resource offset behavior.
- [ ] General dynamic calls, user function frames, variadics/spreads, cleanup, and exact diagnostic ordering across control flow.

## Recent Primary Progress

Recent integrated semantic commits:

- `f3c0f574 codegen: route array sort builtins through lvalue ABI`
- `161a7006 codegen: route null array callbacks through value ABI`
- `b21c5c42 codegen: route request roots through snapshot ABI`
- `ed2d9031 runtime: add array reference path ABI`

Current primary state:

- `master` and `origin/master` are synced at `ed2d9031`.
- The product dashboard update is progress-only on top of that semantic baseline.
- The preserved unrelated unstaged runtime hunk remains in `runtime/src/lib.rs` and is not counted as product progress.
- The latest semantic addition is runtime-only reference-path infrastructure; generated-C/LLVM PHP reference assignment still needs a primary compiler consumer.

## Lane-Local Candidate Work

These are active or completed candidates, not integrated capability:

- `impl-global-symbols`: nested array reference source lowering through the path-reference ABI and request/symbol writeback contracts; promising next-step material, still lane-local.
- `impl-native-call-semantics`: generated native C frame callbacks for descriptor-ready top-level by-value functions with straight-line supported bodies; high value if sliced narrowly.
- `impl-native-control-flow-seed`: executable LLVM loop condition/backedge lowering for state-stable loop forms; still missing loop-carried state, transfer cleanup, switch/goto, and phis.
- `impl-native-diagnostics`: `$GLOBALS` static-string request-root aliases route to a shared request-state symbol blocker; not executable reconciliation yet.
- `impl-array-linked-exec`: additional generated-C array/value builtin consumers and by-reference foreach scalar-offset recovery; useful but lower priority than structural semantics.
- `impl-array-value-runtime`, `impl-binary-string-runtime`, and `impl-native-type-conversion`: broad callable, byte-string, regex, constant, predicate, and conversion work; needs small transplant notes before primary selection.

## Active Roadmap Items

| Active item | Estimate | Status | Next useful primary shape |
| --- | ---: | --- | --- |
| Array/lvalue execution | 70% | In progress | Consume the path-reference ABI in generated reference assignment, arbitrary-root writeback, by-reference foreach, `??=`/RMW, owner/value/reference-slot materialization, or LLVM parity. |
| Symbols/request/globals | 42% | In progress | Keyed/path request/superglobal reads and probes, compiler-lowered mutations, `$GLOBALS` aliasing, request-root replacement, or request lifetime threading. |
| References/COW | 30% | In progress | Narrow owner/reference slot materialization with alias-visible mutation and executable evidence, not just runtime ABI vocabulary. |
| Calls/functions | 25% | In progress | Real declaration descriptor/callable table population with generated body callbacks and result consumers. |
| Objects/properties/methods | 11% | Early | Allocation/property/method behavior through shared carriers, not metadata-only blockers. |
| Control flow/cleanup/diagnostics | 28% | Early | One real structured branch/loop/transfer cleanup path with exact ordering evidence. |
| Broad verification | 71% | In progress | Composition checks crossing request, symbols, arrays, references, calls, diagnostics, and control flow after each primary consumer lands. |

## Steering Notes

The next best primary slice should turn the new reference-path runtime ABI or the request/symbol storage surfaces into executable compiler behavior. Prefer generated reference assignment over proven array parents, keyed/path superglobal reads/writes/probes, `$GLOBALS` alias reconciliation, request lifetime threading, or a narrow real call/control-flow execution slice. Avoid another standalone builtin-family batch unless it crosses references/COW, request state, function frames, object/ArrayAccess, or structured cleanup. Whole-lane merges, fixture-shaped production lowering, generated-source substring-only proof, formatter spillover, and docs-only churn outside this dashboard remain rejected.
