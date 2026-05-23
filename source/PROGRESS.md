# PHP Native Compiler Progress

Updated: 2026-05-23 05:52 CEST
Evaluation marker: `20260523T034337Z`

Primary HEAD: `fe73cc3e codegen: lower loop-carried scalar state`
Latest integrated semantic baseline: `fe73cc3e codegen: lower loop-carried scalar state`
Latest evaluator report: `20260523T034337Z`

These are candid engineering estimates toward generalized PHP semantics in the
native compiler. They are not test pass rates. Only primary-integrated, pushed
work counts; lane-local candidates, dirty WIP, and exact-shape fixtures do not.

## Executive Read

Overall estimated progress: **87%** `[#################---]`

Primary has continued to land small generalized generated-C execution slices
with focused proof. Recent integrated work covers `strlen()` over value-result
producers, state-stable `while`, top-level `return`, depth-1 `while`
`break`/`continue`, state-stable `for` loops, and loop-carried scalar state.

This is still not full PHP semantics. The largest remaining gaps are
generated-native calls/frames, object/property/method execution, complete
references/COW identity, source-ordered diagnostics, cleanup/unwinding, and
LLVM/assembly parity.

Current primary cleanliness: semantic work is current through the loop-carried
scalar-state batch. The protected `runtime/src/lib.rs` null-slot hunk remains
dirty, unstaged, and uncounted.

## Roadmap Position

| State | Workstream | Estimate | Bar | Current read |
| --- | --- | ---: | --- | --- |
| [x] Mostly done | Runtime and ABI foundations | **97%** | `[###################-]` | Strong shared value, array, reference, symbol, request, comparison, truthiness, diagnostic, termination, and cleanup surfaces. |
| [x] Mostly done | Compiler/backend consumers | **98%** | `[###################-]` | Generated-C has broad selected coverage. LLVM/assembly parity remains uneven. |
| [ ] In progress | Executable PHP semantics | **87%** | `[#################---]` | Many focused linked programs run, but behavior is still selected islands rather than a complete PHP execution model. |
| [ ] In progress | Arrays, lvalues, references, COW | **88%** | `[##################--]` | Strong selected array/lvalue/reference paths. Full COW, arbitrary writable roots, and by-reference argument/foreach parity remain open. |
| [ ] In progress | Symbols, globals, request state | **96%** | `[###################-]` | Strong request and `$GLOBALS` generated-C coverage. Reconciliation across calls/requests still needs work. |
| [ ] Not done | Calls, functions, frames | **27%** | `[#####---------------]` | Runtime/interpreter metadata exists, but generated-native call/frame execution is the biggest missing block. |
| [ ] Not done | Objects, properties, methods | **11%** | `[##------------------]` | Mostly lane-local/runtime candidate work. Primary lacks general compiled object/property/method execution. |
| [ ] In progress | Control flow, cleanup, diagnostics | **45%** | `[#########-----------]` | Generated-C has bounded branches, lazy expressions, top-level return, state-stable loops, depth-1 `while` transfers, and loop-carried int/float/bool scalar state; owner/reference phis, switch/goto, multi-level transfers, unwinding, and exact ordering are still missing. |
| [ ] In progress | Broad integrated verification | **87%** | `[#################---]` | Focused gates are strong. Cross-feature composition and backend parity need broader proof. |

## Primary-Integrated Capability

- `fe73cc3e`: generated-C loop-carried int/float/bool scalar variables in
  accepted `while` and bounded `for` paths can use mutable C slots, while
  unsupported owner/reference/native-value state and kind-changing joins remain
  rejected.
- `d6d5d1cf`: generated-C state-stable `for` loops lower through scoped
  initializer, single PHP-truthiness condition, body, increment, and loop-local
  depth-1 `break`/`continue`; `continue` runs the increment section before the
  next condition check.
- `315a2ca2`: generated-C accepted `while` bodies execute depth-1 `break` and
  `continue` without claiming loop-carried state or multi-level transfer
  semantics.
- `dcd99134`: generated-C top-level `return` evaluates supported operands for
  side effects, cleans discarded values and live owners, and terminates the
  executable path with PHP CLI-compatible success.
- `6b5e480f`: generated-C state-stable `while` loops execute when the condition
  and body preserve compiler-visible ownership state.
- `9a05a58b`: generated-C `strlen()` consumes owned native value-result
  producers through the shared value-to-string boundary.
- Earlier recent primary work covers native array owner output/truthiness,
  stored native-value type/presence checks, branch-local owner cleanup, strict
  value-result comparison, native value-operation diagnostics, by-reference
  foreach slots for selected array lvalue owners, statement-discard cleanup,
  lazy ternaries, logical short-circuiting, request/`$GLOBALS` operations,
  reference-backed active symbol-root lvalues, and direct termination.

## Candidate Work Not Counted

Lane-local candidates:

- Array/range construction, writable string-offset recovery, printf/vprintf
  byte-format output, binary string/runtime helper expansion, request/include
  planning, call-frame contract blockers, and control-flow CFG cleanup
  candidates are active across lanes.
- Object/property/method and call/frame lanes still mostly produce metadata,
  preflight, or blocker contracts rather than primary executable consumers.

## Done / In Progress / Not Done

- [x] Done enough to build on: selected native value ownership, arrays, request
  state, symbol-root handling, diagnostics, output, direct termination, lazy
  expressions, branch cleanup, and bounded generated-C control-flow islands.
- [ ] In progress: owner/reference/native-value control-flow phis,
  cross-feature composition, source-ordered diagnostics, cleanup ownership
  joins, references/COW across arbitrary writable roots, request/symbol
  reconciliation across calls, and backend parity.
- [ ] Not done: generated-native user functions/calls/frames, methods,
  objects/properties, dynamic dispatch, include/require body execution,
  structured unwinding/finally/destructors/shutdown/output buffers, complete
  PHP diagnostic parity, and complete reference/COW identity.

## Active Focus

1. Integrate one small generalized primary slice at a time, with focused gates
   and immediate push.
2. Prefer slices that unlock real behavior, not just blocker vocabulary:
   call/frame execution, object/property/method execution, references/COW
   through control flow, structured cleanup/unwinding, source-ordered
   diagnostics, and backend parity.
3. Keep lane-local work code-writing only and reject exact-shape progress: no
   one-fixture recognizers, no fixed-key/value branches, no docs-only progress,
   and no generated-source substring proof without a shared semantic boundary.
4. Preserve primary hygiene: commit only owned semantic/progress hunks, and keep
   the protected null-slot runtime hunk uncommitted until it has its own
   reviewed semantics batch.

## Major Blockers

- Full generated-native user functions, methods, closures, dynamic calls,
  arguments, returns, frames, variadics/spreads, by-reference handoff, and
  cleanup.
- Real object construction, property/method dispatch, `$this`, static context,
  visibility, magic hooks, resources, `ArrayAccess`, and object-compatible
  diagnostics.
- Full references/COW identity across arbitrary writable roots, function
  arguments/returns, foreach, arrays, objects, request/global storage, and
  control-flow joins.
- Structured control flow beyond the accepted generated-C subset: loop-carried
  owner/reference/native-value state, switch, goto/labels, multi-level
  transfers, finally, destructors, shutdown behavior, output buffers, and SAPI
  interactions.
- Exact diagnostics: severity, ordering, suppression, custom handlers, source
  spans, recovery values, fatal/throw behavior, and cleanup during diagnostics.
- LLVM/assembly parity for newer generated-C/runtime ABI consumers.

## Current Steering

The loop-carried scalar batch is useful but narrow. The next accepted batches
should avoid nearby loop variants and target one of the hard cliffs above. Lane
output should be integrated only when it creates a small, reviewable primary
executable consumer or tightens a centralized blocker with proof.
