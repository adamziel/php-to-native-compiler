# PHP Native Compiler Progress

Updated: 2026-05-22 17:14 CEST
Evaluation marker: `20260522T150931Z`
Primary HEAD: `633c8713 codegen: route GLOBALS symbol paths through ABI`
Current pushed semantic baseline: `633c8713 codegen: route GLOBALS symbol paths through ABI`

These percentages are candid engineering estimates toward generalized PHP
semantics in the native compiler. They are not test pass rates. Lane-local work
and unstaged primary diffs do not count until reviewed, gated, committed to
`master`, and pushed.

## Executive Read

Overall estimated progress: **80%** `[################----]`

The project is still moving in the right direction. Recent pushed primary work
has converted request-state infrastructure into executable generated-C behavior
for nested/path reads, writes, unsets, probes, `empty()`, and assignment
expression values. The newest pushed compiler slice now also routes dynamic and
nested `$GLOBALS[...]` reads, `isset()`, and `empty()` through the symbol-table
path ABI with linked executable coverage.

This is real generalized symbol/global progress, but it is not full `$GLOBALS`
semantics. Writes, unsets, alias reconciliation, self-reference behavior,
reference/COW-preserving nested cells, function-frame visibility, LLVM/C
assembly parity, and exact diagnostics still remain open.

## Roadmap Position

| Roadmap item | Estimate | Visual | Status |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | 96% | `[###################-]` | Strong, but avoid more vocabulary-only layers without immediate consumers. |
| Compiler/backend consumers | 83% | `[#################---]` | Good for selected request/array/string/`$GLOBALS` read paths; uneven elsewhere. |
| Executable generalized PHP semantics | 71% | `[##############------]` | Partial; many real PHP compositions still block. |
| Arrays, lvalues, references, COW | 73% | `[###############-----]` | Arrays/lvalues improved; references/COW and arbitrary writable roots remain large. |
| Symbols, globals, request state | 67% | `[#############-------]` | Request paths and `$GLOBALS` reads/probes are stronger; writes, aliases, frames, and self-reference remain incomplete. |
| Calls, functions, frames | 25% | `[#####---------------]` | Early; frame/call execution is still mostly candidate or bounded work. |
| Objects, properties, methods | 11% | `[##------------------]` | Early; real object allocation/property/method semantics remain missing. |
| Diagnostics and control flow | 29% | `[######--------------]` | Early; cleanup ordering and exact PHP diagnostic order are not generalized. |
| Broad integrated verification | 80% | `[################----]` | Useful focused gates, including linked `$GLOBALS` path coverage, but cross-feature composition remains thin. |

## Done / In Progress / Not Done

- [x] Runtime/value foundations for selected scalar, string, array, comparison,
  diagnostic, symbol-table, request-state, reference-slot, and native-value
  operations.
- [x] Generated-C consumers for selected scalar/string/array/lvalue behavior,
  including tracked array owner mutations and natural sort families.
- [x] Request-state root, keyed, and nested/path reads, writes, unsets,
  `isset()`, `empty()`, and assignment-expression values through shared request
  ABIs.
- [x] Direct `$GLOBALS` root snapshots and runtime symbol-table nested
  write/read/probe ABIs.
- [x] Compiler-lowered `$GLOBALS[$expr]` and nested `$GLOBALS[...]`
  read/`isset`/`empty` paths through the symbol-table path ABI in generated C.
- [ ] `$GLOBALS` symbol path writes/unsets and request-root alias
  reconciliation.
- [ ] Generated PHP reference assignment over proven array/request/symbol
  reference boundaries.
- [ ] Full references/COW, arbitrary writable roots, owner/value/reference
  slots, by-reference args/returns, and by-reference foreach parity.
- [ ] User function/method/closure frames, dynamic calls, variadics/spreads, and
  cleanup ownership across calls.
- [ ] Real object/property/method semantics, `ArrayAccess`, resource offsets,
  and PHP-compatible diagnostics around those features.
- [ ] Structured control-flow cleanup, branch joins, loop/switch transfer, and
  source-ordered warnings/errors at broad scale.

## Recent Primary-Integrated Work

Recent semantic commits on primary:

- `633c8713 codegen: route GLOBALS symbol paths through ABI`
- `39586978 runtime: add symbol-table nested read probes`
- `8c13b871 codegen: return request assignment values`
- `f88a624d codegen: route request path reads through state ABI`
- `15657b95 codegen: route request path mutations through state ABI`
- `3bda4f51 codegen: route array mutation builtins through lvalue ABI`
- `d7fc807d codegen: materialize direct $GLOBALS snapshots`
- `764cf014 runtime: add symbol-table nested write ABI`
- `ed2d9031 runtime: add array reference path ABI`

Primary-integrated capability now includes strong request-superglobal path
execution through shared request-state ABIs and generated-C `$GLOBALS[...]`
read/probe lowering through shared symbol-table path ABIs. `$GLOBALS` path
writes/unsets, alias reconciliation, self-reference behavior, and frame/request
lifetime remain incomplete.

## Lane-Local And Active Candidate Work

Lane-local candidates, not counted:

- `impl-global-symbols`: dynamic `$GLOBALS[$expr]` nested dispatch, dynamic
  request slot names, and function-frame nested slot work.
- `impl-native-integration-batch`: tracked array mutation owner/path and
  by-reference foreach owner/path candidates.
- `impl-symbol-integrator`: runtime value-content inspection and expression
  result continuation work such as `strlen()` and `is_numeric()`.
- Other fresh lanes are active around control flow, calls, objects, diagnostics,
  comparison, type conversion, reference cells, binary strings, and arrays.

## Current Steering

The next integration batches should favor small executable slices:

- Follow the pushed `$GLOBALS` read/probe slice with `$GLOBALS` writes/unsets and
  request/global alias reconciliation, not another standalone runtime helper.
- Move generated reference assignment, request/global frame lifetime, and
  owner/value/reference-slot materialization closer to primary.
- Take one narrow call/frame or control-flow cleanup consumer only when it
  proves real executable behavior and cleanup ordering.
- Coordinate `impl-global-symbols` with primary before importing dynamic
  request-root dispatch, so the alias model does not split.

Rejected distractions:

- Exact-shape lowering for one fixture or one PHP snippet.
- Standalone blocker/status vocabulary without a near-term consumer.
- Large wholesale lane merges.
- Documentation churn that does not improve steering or integration clarity.

## Live Notes

Primary currently has one preserved unstaged implementation diff:
`runtime/src/lib.rs` null-slot increment/decrement behavior. It is not counted
as progress and still needs explicit classification, focused tests, and a
separate commit or rejection before any runtime staging.

Resource snapshot after the latest push: `/dev/shm` is tight but above the
dispatcher floor, around 6.5G free of 22G. Keep focused gates and avoid broad
simultaneous cargo waves while tmpfs usage remains volatile.

Evaluator cadence: one candid strategy/progress evaluation every 45 minutes,
feeding advisory steering back to the supervisor.
