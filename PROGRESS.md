# PHP Native Compiler Progress

Updated: 2026-05-24 01:07 CEST
Evaluation marker: `20260523T225737Z`

Latest primary semantic/test baseline:
`cc919d46 runtime: report native formatter stdout diagnostics`
Latest integrated semantic baseline: `cc919d46 runtime: report native formatter stdout diagnostics`
Latest evaluator report: `20260523T225737Z`

These are candid engineering estimates toward generalized PHP semantics in the
native compiler. They are not test pass rates. Only primary-integrated,
pushed work counts; lane-local candidates, dirty WIP, parked diffs, and
exact-shape fixtures do not.

## Executive Read

Overall estimated progress: **88%** `[##################--]`

Primary integrated progress now includes generated-C top-level state-stable
`goto`/label dispatch, normal-flow `try`/`finally`, and a diagnostic-aware
native value stdout formatter consumed by LLVM and generated-C display paths.
The compiler has strong generated-C islands for native values, arrays, selected
references, request/symbol state, diagnostics, cleanup, lazy expressions, and
bounded control flow.

This is still not full PHP semantics. The largest gaps remain generated-native
calls/frames, object/property/method execution, complete references/COW
identity, source-ordered diagnostics, cleanup/unwinding, and LLVM/assembly
parity.

Current primary state: primary is synced with `origin/master` at `cc919d46`.
The protected `runtime/src/lib.rs` null-slot hunk remains dirty, unstaged, and
uncounted. The narrow generated-C user-function WIP was parked and is not
counted.

## Roadmap Snapshot

| Workstream | Estimate | Bar | Current read |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | **97%** | `[###################-]` | Strong shared value, array, reference, symbol, request, comparison, truthiness, diagnostic, termination, and cleanup surfaces. |
| Compiler/backend consumers | **98%** | `[####################]` | Generated-C has broad selected coverage. LLVM/assembly parity remains uneven. |
| Executable PHP semantics | **88%** | `[##################--]` | Many focused linked programs run, but behavior is still selected islands rather than a complete PHP execution model. |
| Arrays, lvalues, references, COW | **88%** | `[##################--]` | Strong selected array/lvalue/reference paths. Full COW, arbitrary writable roots, and by-reference call/foreach parity remain open. |
| Symbols, globals, request state | **96%** | `[###################-]` | Strong request and `$GLOBALS` generated-C coverage. Reconciliation across calls/requests still needs work. |
| Calls, functions, frames | **27%** | `[#####---------------]` | Runtime/interpreter metadata and lane contracts exist, but real generated-native frame execution is still missing. |
| Objects, properties, methods | **11%** | `[##------------------]` | Mostly lane-local/runtime candidate work. Primary lacks general compiled object/property/method execution. |
| Control flow, cleanup, diagnostics | **52%** | `[##########----------]` | Bounded generated-C branches, loops, returns, transfers, switches, top-level state-stable gotos, normal-flow try/finally, and diagnostic-aware stdout formatting exist; owner/reference joins, unwinding, handlers, and exact ordering remain open. |
| Broad integrated verification | **88%** | `[##################--]` | Focused gates are strong. Cross-feature composition and backend parity need broader proof. |

## Done / In Progress / Not Done

- [x] Generated-C value-result diagnostics through shared diagnostic reporting.
- [x] Generated-C native value-result `strlen()` consumption.
- [x] Generated-C top-level `return`, state-stable `while`/`for`, scalar loop-carried slots, multi-level loop transfers, state-stable `switch` dispatch/fallthrough/break, top-level state-stable `goto` labels, and normal-flow `try`/`finally`.
- [x] Diagnostic-aware native value stdout formatting consumed by LLVM and generated-C display paths.
- [x] Strong selected generated-C arrays, lvalues, references, request state, `$GLOBALS`, lazy ternaries, logical short-circuiting, branch cleanup, foreach storage, and output/truthiness paths.
- [ ] Primary-integrated generated-native user-function/call-frame execution.
- [ ] Primary-integrated object construction, property access, method dispatch, `$this`, static context, visibility, and magic behavior.
- [ ] Full reference/COW identity across calls, arrays, objects, globals, foreach, and control-flow joins.
- [ ] Full structured cleanup/unwinding/finally/destructors/output-buffer/SAPI behavior.
- [ ] Exact diagnostic severity, ordering, suppression, handlers, spans, recovery, fatal/throw behavior.
- [ ] LLVM/assembly parity for newer generated-C/runtime ABI consumers.

## Recent Primary-Integrated Work

- `cc919d46`: routes native value stdout display through
  `phpc_native_value_format_stdout_with_diagnostic(...)` for the accepted echo
  formatter tag, with LLVM and generated-C consumers across scalar, binary
  string, array-owner, native-value, reference-read, and symbol-table display
  families. Runtime-only formatter vocabulary for serialize/var_dump/print_r/
  var_export was trimmed instead of counted.
- `68eea573`: tightens the generated-C `try`/`finally` boundary so
  `break`/`continue` transfers reject with the same unwinding blocker as
  `return`, `exit`, `goto`, and `throw` unless real finally scheduling exists.
- `0988519d`: generated-C normal-flow `try`/`finally` executes accepted try
  bodies, skips catch bodies when no throw path exists, runs finally bodies,
  and rejects transfer paths that would require real unwinding/finally routing.
- `0d049a06`: generated-C top-level `goto`/label lowering emits C labels for
  accepted statement-list targets, validates every transfer against the
  target's persistent compiler state snapshot, and rejects state-changing
  target joins plus nested gotos instead of claiming broader goto semantics.
- `97ba85b6`: generated-C `switch` dispatches accepted state-stable case
  conditions through shared native value comparison/truthiness, preserves
  source-order fallthrough/default/break labels, and rejects state-changing
  case bodies plus switch-local `continue`.
- `fe73cc3e`: generated-C `while` and bounded `for` loops can carry
  int/float/bool scalar variables through mutable C slots while preserving
  blockers for unsupported owner/reference/native-value state.
- `171fd0f1`: generated-C nested loop `break N`/`continue N` route through
  explicit loop target labels for accepted `while`/`for` loops, with focused
  source and linked executable proof.
- `d6d5d1cf`: generated-C state-stable `for` loops lower scoped initializer,
  single PHP-truthiness condition, body, increment, and loop-local depth-1
  `break`/`continue`.
- `315a2ca2`: generated-C accepted `while` bodies execute depth-1 `break` and
  `continue` without claiming broader loop-carried state semantics.
- `dcd99134`: generated-C top-level `return` evaluates supported operands for
  side effects, cleans discarded values and live owners, and terminates the
  executable path with PHP CLI-compatible success.
- Earlier recent work also covers state-stable `while`, `strlen()` value
  results, diagnostic reporting, native array owner output/truthiness, stored
  native-value type/presence checks, branch-local owner cleanup, strict
  value-result comparison, selected by-reference foreach slots, statement
  discard cleanup, request/`$GLOBALS`, reference-backed active symbol-root
  lvalues, and direct termination.

## Lane-Local Candidate Work

Lane-local work includes useful candidates around generated-C comparison
consumers, call-frame/reference contracts, object/property boundaries,
conversion/request diagnostics, native loop cleanup blockers, and
reference/COW-adjacent runtime ABI surfaces. These remain uncounted until
primary integration reviews and lands a generalized semantic slice with
focused source and executable proof.

The narrow generated-C user-function WIP was parked because it looked too much
like direct/top-level/single-return execution without real frame ownership,
caller handoff, cleanup/failure exits, or accepted linked proof. It should not
be counted or repeated in that shape.

## Active Focus

1. Integrate one small generalized primary slice at a time, with focused gates
   and immediate push.
2. Prefer hard-cliff work and executable consumers over more vocabulary:
   generated-C comparison execution, calls/frames, objects/properties/methods,
   references/COW through real control flow, structured cleanup/unwinding/
   source-ordered diagnostics, and backend parity.
3. Keep lane-local work code-writing only and reject exact-shape progress:
   no one-fixture recognizers, no fixed-key/value branches, no docs-only
   progress, and no generated-source substring proof without a shared semantic
   boundary.
4. Preserve primary hygiene: commit only owned semantic/progress hunks, keep
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
  owner/reference/native-value loop-carried state, switch state joins,
  broader goto/label joins, cleanup joins, finally during
  break/continue/return/exit/goto/throw transfers, destructors, shutdown
  behavior, output buffers, and SAPI interactions.
- Exact diagnostics: severity, ordering, suppression, custom handlers, source
  spans, recovery values, fatal/throw behavior, and cleanup during diagnostics.
- LLVM/assembly parity for newer generated-C/runtime ABI consumers.

## Current Steering

The direction is right only while primary integration keeps converting shared
semantic boundaries into executable behavior. The next accepted batch should
target one of the hard cliffs above. If a lane cannot make generalized
progress, it should tighten a centralized blocker with proof rather than
introducing exact-shape lowering.
