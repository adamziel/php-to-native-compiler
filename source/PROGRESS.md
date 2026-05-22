# PHP Native Compiler Progress

Updated: 2026-05-22 17:00 CEST
Primary HEAD: `39586978 runtime: add symbol-table nested read probes`
Current pushed semantic baseline: `39586978`

These percentages are candid engineering estimates toward the current goal:
generalized PHP semantics in the native compiler. They are not test pass rates.
Lane-local work and unstaged diffs do not count until reviewed, gated, committed
to `master`, and pushed.

## Executive Read

Overall estimated progress: **79%** `[################----]`

The project is moving in the right direction: recent primary commits have landed
shared request-state, symbol-table, lvalue, array, diagnostic, comparison, and
native-value ABI surfaces, plus several generated-C consumers. The remaining
gap is still large because many of those surfaces are foundations, not complete
PHP behavior across arbitrary programs.

The current priority is to convert shared runtime/compiler infrastructure into
executable generalized semantics: `$GLOBALS[$expr]` and symbol paths, reference
assignment, COW/reference-visible mutation, calls/frames, objects/properties,
control-flow cleanup, and diagnostics that compose across those features.

## High-Level Roadmap

| Roadmap item | Estimate | Status | What still has to become true |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | 96% | Strong | Keep ABI surfaces stable while adding consumers; avoid more vocabulary-only layers unless they unblock immediate lowering. |
| Compiler/backend consumers | 82% | Good but uneven | Wire symbol paths, reference assignment, calls, objects, control flow, and diagnostics through generated native code and LLVM/C parity paths. |
| Executable generalized PHP semantics | 70% | Partial | Make broad PHP source shapes run through the shared machinery, not only selected request/array/string/lvalue cases. |
| Arrays, lvalues, references, COW | 73% | Partial | Finish writable roots, owner/value/reference slots, by-reference foreach, reference assignment, alias-visible mutation, and COW separation. |
| Symbols, globals, request state | 64% | In progress | Extend from direct request roots and direct `$GLOBALS` snapshots to `$GLOBALS[$expr]`, symbol path mutation/read/probe, alias reconciliation, and request/frame lifetime. |
| Calls, functions, frames | 25% | Early | Add callable tables, generated body callbacks, argument/return ownership, by-ref args, variadics/spreads, and dynamic call dispatch. |
| Objects, properties, methods | 11% | Early | Implement allocation, property storage, visibility/magic hooks, method calls, `stdClass`, `ArrayAccess`, and resource offset behavior. |
| Diagnostics and control flow | 29% | Early | Compose source spans, severity, warning/error ordering, cleanup stacks, loops, switch, break/continue, goto, and branch joins. |
| Broad integrated verification | 79% | Useful but thin | Add composition tests crossing symbols, request state, arrays, references, calls, objects, diagnostics, and control flow after every integration batch. |

## Current Primary Capability

Integrated and pushed primary now includes:

- Native runtime/value foundations for selected scalar, string, array,
  comparison, diagnostic, symbol-table, request-state, reference-slot, and
  native-value operations.
- Generated-native C consumers for selected scalar/string/array/lvalue behavior,
  including tracked array owner mutations and natural sort families.
- Request-state root, keyed, and nested/path reads, writes, unsets, `isset()`,
  `empty()`, and assignment-expression values through shared request ABIs.
- Direct request-root value/reference storage at runtime.
- Direct `$GLOBALS` root snapshots through the symbol-table snapshot ABI.
- Runtime symbol-table nested write-by-path and nested read/probe ABIs over
  generalized key paths.
- Source-location metadata on shared native diagnostic handles.

Not counted as complete yet:

- Full `$GLOBALS` aliasing and mutation reconciliation.
- Compiler-lowered `$GLOBALS[$expr]` and general symbol path consumers.
- Generated PHP reference assignment over proven path/reference ABIs.
- Full references/COW, arbitrary writable roots, owner/value/reference slots,
  by-reference args/returns, and by-reference foreach.
- User function/method/closure frames, dynamic calls, variadics/spreads, and
  cleanup ownership across calls.
- Real object/property/method semantics, `ArrayAccess`, resource offsets, and
  PHP-compatible diagnostics around those features.
- Structured control-flow cleanup and branch-join behavior at broad scale.

## Recent Integrated Work

Recent semantic commits on primary:

- `39586978 runtime: add symbol-table nested read probes`
- `8c13b871 codegen: return request assignment values`
- `f88a624d codegen: route request path reads through state ABI`
- `15657b95 codegen: route request path mutations through state ABI`
- `3bda4f51 codegen: route array mutation builtins through lvalue ABI`
- `d7fc807d codegen: materialize direct $GLOBALS snapshots`
- `764cf014 runtime: add symbol-table nested write ABI`
- `ed2d9031 runtime: add array reference path ABI`

The latest runtime commit adds generalized symbol-table nested read, `isset()`,
and `empty()` probes over value-handle key paths, with diagnostics for invalid
handles, missing roots/keys, invalid keys, and scalar parents. It is useful
infrastructure, but it should be followed quickly by compiler/generated-C symbol
path consumers so it does not remain runtime-only scaffolding.

## Active Steering

The next integration batches should favor small, generalized slices that unlock
actual behavior:

- `$GLOBALS[$expr]` and arbitrary symbol path read/write/probe lowering.
- Generated reference assignment using existing array/request/symbol reference
  path boundaries.
- Request/global lifetime through function frames and repeated calls.
- Source-call/user-frame execution with real argument and return ownership.
- One narrow object/property slice with allocation plus read/write/probe
  behavior, not metadata-only blockers.
- One control-flow cleanup slice that proves branch/loop state joins and
  diagnostic ordering.

Rejected distractions:

- Exact-shape lowering for one fixture or one PHP snippet.
- Standalone blocker/status vocabulary without a near-term consumer.
- Large wholesale lane merges.
- Documentation churn that does not improve steering or integration clarity.

## Live Notes

Primary currently has one preserved unstaged runtime hunk:

`runtime/src/lib.rs`: array lvalue null-slot increment/decrement behavior.

That hunk is not counted as progress. It needs explicit owner classification,
focused tests, and a separate commit or rejection.

Evaluator cadence: one candid strategy/progress evaluation every 45 minutes,
feeding back into worker steering and integration priorities.
