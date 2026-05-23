# PHP Native Compiler Progress

Updated: 2026-05-23 05:57 CEST
Evaluation marker: `20260523T034337Z`

Primary HEAD: `fe73cc3e codegen: lower loop-carried scalar state`
Latest integrated semantic baseline: `fe73cc3e codegen: lower loop-carried scalar state`
Latest evaluator report: `20260523T034337Z`

These are candid engineering estimates toward generalized PHP semantics in the
native compiler. They are not test pass rates. Only primary-integrated,
pushed work counts; lane-local candidates, dirty WIP, and exact-shape fixtures
do not.

## Executive Read

Overall estimated progress: **88%** `[##################--]`

The compiler now has strong generated-C islands for native values, arrays,
symbol/request state, selected references, diagnostics, cleanup, branch
ownership, foreach storage, lazy expressions, and bounded generated-C control
flow. Recent primary integration has been useful because it lands small
generalized semantic slices and pushes them immediately, rather than
accumulating lane-local artifacts.

This is still not full PHP semantics. The largest remaining gaps are
generated-native calls/frames, object/property/method execution, complete
references/COW identity, source-ordered diagnostics, cleanup/unwinding, and
LLVM/assembly parity.

Current primary cleanliness: semantic work is current through the loop-carried
scalar-state and multi-level loop-transfer follow-up. The protected
`runtime/src/lib.rs` null-slot hunk remains dirty, unstaged, and uncounted.

## Roadmap

| Workstream | Estimate | Current read |
| --- | ---: | --- |
| Runtime and ABI foundations | **97%** | Strong shared value, array, reference, symbol, request, comparison, truthiness, diagnostic, termination, and cleanup surfaces. |
| Compiler/backend consumers | **98%** | Generated-C has broad selected coverage. LLVM/assembly parity remains uneven. |
| Executable PHP semantics | **88%** | Many focused linked programs run, but behavior is still selected islands rather than a complete PHP execution model. |
| Arrays, lvalues, references, COW | **88%** | Strong selected array/lvalue/reference paths. Full COW, arbitrary writable roots, and by-reference argument/foreach parity remain open. |
| Symbols, globals, request state | **96%** | Strong request and `$GLOBALS` generated-C coverage. Reconciliation across calls/requests still needs work. |
| Calls, functions, frames | **27%** | Runtime/interpreter metadata exists, but generated-native call/frame execution is the biggest missing block. |
| Objects, properties, methods | **11%** | Mostly lane-local/runtime candidate work. Primary lacks general compiled object/property/method execution. |
| Control flow, cleanup, diagnostics | **46%** | Generated-C now has bounded branches, lazy expressions, top-level return, state-stable loops, scalar loop-carried state, and loop-local/multi-level loop transfers; owner/reference phis, switch/goto, unwinding, and exact ordering are still missing. |
| Broad integrated verification | **88%** | Focused gates are strong. Cross-feature composition and backend parity need broader proof. |

## Recent Integrated Work

- `fe73cc3e`: generated-C `while` and bounded `for` loops can carry
  int/float/bool scalar variables through mutable C slots while preserving
  blockers for unsupported owner/reference/native-value state.
- Follow-up after `fe73cc3e`: generated-C nested loop `break N`/`continue N`
  now route through explicit loop target labels for accepted `while`/`for`
  loops, with focused source and linked executable proof.
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
- `6b5e480f`: generated-C state-stable `while` loops execute when the
  condition and body preserve compiler-visible ownership state.
- `9a05a58b`: generated-C `strlen()` consumes owned native value-result
  producers through the shared value-to-string boundary.
- `4efa12ba`: generated-C native value-operation diagnostics report through
  the shared diagnostic consumer.
- Earlier recent primary work also covers native array owner output/truthiness,
  stored native-value type/presence checks, branch-local owner cleanup,
  strict value-result comparison, by-reference foreach slots for selected
  array lvalue owners, statement-discard cleanup, lazy ternaries, logical
  short-circuiting, request/`$GLOBALS` operations, reference-backed active
  symbol-root lvalues, and direct termination.

## Active Focus

1. Integrate one small generalized primary slice at a time, with focused
   gates and immediate push.
2. Prefer slices that unlock real behavior, not just blocker vocabulary:
   call/frame execution, object/property/method execution, references/COW
   through control flow, structured cleanup/unwinding, source-ordered
   diagnostics, and backend parity.
3. Keep lane-local work code-writing only and reject exact-shape progress:
   no one-fixture recognizers, no fixed-key/value branches, no docs-only
   progress, and no generated-source substring proof without a shared semantic
   boundary.
4. Keep the 45-minute candid evaluator cadence active and feed its findings
   back into the supervisor gates.
5. Preserve primary hygiene: commit only owned semantic/progress hunks, keep
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
- Structured control flow beyond the accepted generated-C subset:
  owner/reference/native-value loop-carried state, switch, goto/labels,
  cleanup joins, finally/destructors, shutdown behavior, output buffers, and
  SAPI interactions.
- Exact diagnostics: severity, ordering, suppression, custom handlers, source
  spans, recovery values, fatal/throw behavior, and cleanup during diagnostics.
- LLVM/assembly parity for newer generated-C/runtime ABI consumers.

## Current Steering

The direction is right only while primary integration keeps converting shared
semantic boundaries into executable behavior. The next accepted batches should
avoid nearby `while`/`for` variants and target one of the hard cliffs above.
If a lane cannot make generalized progress, it should add or tighten a
centralized blocker with proof instead of introducing exact-shape lowering.
