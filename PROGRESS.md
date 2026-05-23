# PHP Native Compiler Progress

Updated: 2026-05-23 02:23 CEST
Evaluation marker: `20260523T002315Z`

Primary baseline: `f54e293e docs: distill progress roadmap`
Latest semantic baseline: `106cc646 codegen: lower scoped native if branches`
Latest evaluator report: `20260523T002315Z`

These percentages are candid engineering estimates toward generalized PHP
semantics in the native compiler. They are not test pass rates. Primary
integrated work counts; lane-local candidates, uncommitted diffs, and
exact-shape fixtures do not.

## Executive Read

Overall estimated progress: **86%** `[#################---]`

Momentum is positive. Primary now includes several generated-C executable
semantic slices: selected request/global/array/reference paths from earlier
work, direct `exit()`/`die()` cleanup, diagnostic report ownership cleanup, and
bounded state-stable `if`/`else` lowering. The latest branch slice is useful
because it executes real branch bodies only when persistent compiler state does
not need merge/phi reconciliation, and it rejects unsupported branch state
instead of faking it.

This is still not a complete native PHP execution model. Calls/functions,
objects/properties/methods, full references/COW, broad structured control flow,
exact diagnostics, and LLVM/assembly parity remain the main gaps.

Current primary cleanliness: HEAD is synced with `origin/master` at
`f54e293e`, with concurrent uncommitted integration WIP in compiler/runtime/doc
files. Those diffs are not integrated and are not counted until they are
reviewed, gated, committed, and pushed.

## Grand Roadmap

| Roadmap item | Estimate | Visual | Current read |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | **97%** | `[###################-]` | Strong value, array, symbol-table, request-state, reference, comparison, truthiness, diagnostic, and exit-result surfaces. |
| Compiler/backend consumers | **97%** | `[###################-]` | Good generated-C coverage for selected request, `$GLOBALS`, symbols, arrays, lvalues, references, exit, diagnostics, and state-stable branches. LLVM/assembly parity is uneven. |
| Executable PHP semantics | **84%** | `[#################---]` | Improving through linked executable gates, but still selected islands rather than a complete execution model. |
| Arrays, lvalues, references, COW | **87%** | `[#################---]` | Strong selected paths, including reference-backed active symbol-root array lvalues. Arbitrary writable roots, full COW, and by-reference foreach remain large. |
| Symbols, globals, request state | **96%** | `[###################-]` | Strong request/`$GLOBALS` generated-C coverage. Broader request/global reconciliation remains open. |
| Calls, functions, frames | **27%** | `[#####---------------]` | Runtime/interpreter call-frame metadata enforcement landed; generated-native call/frame execution is still the major missing piece. |
| Objects, properties, methods | **11%** | `[##------------------]` | Mostly lane-local/runtime candidate work. Primary still lacks general compiled object/property/method execution. |
| Control flow, cleanup, diagnostics | **34%** | `[#######-------------]` | Direct exit, diagnostic ownership, and bounded state-stable `if`/`else` execute on generated C. Loops, switch, branch joins, exact ordering, shutdown/finally/destructors, and unwinding are not generalized. |
| Broad integrated verification | **86%** | `[#################---]` | Focused gates are strong. Cross-feature composition and backend parity need broader proof. |

## Primary-Integrated Progress

- [x] `106cc646`: generated-native C-link `if`/`else` lowering for branch
  conditions using native truthiness or comparison boundaries, guarded by a
  persistent-state stability check.
- [x] `2545d173`: generated-C diagnostic-report cleanup now uses one
  owner-consuming report helper and avoids report-then-free ownership errors.
- [x] `0cfae034`: generated-native direct `exit()`/`die()` handles no-arg,
  `null`, `int`, and `string` operands with shared cleanup and focused linked
  proof.
- [x] `9ade9293`: runtime/interpreter call-frame execution enforces declared
  by-value parameter/default/return type metadata. This is useful call-frame
  groundwork, but not generated-native call lowering.
- [x] Earlier baseline: selected generated-C request roots, `$GLOBALS`,
  symbol-table operations, array queries/lvalues, and reference-backed active
  symbol-root lvalue owners.

## Lane-Local Candidate Work

Not counted until primary integrates it:

- [ ] Generated named-call and callback-call blockers routed through shared
  conversion-result cleanup paths.
- [ ] Class method bodies consuming shared object-operation blockers instead
  of vanishing behind metadata-only class registration.
- [ ] Builtin interface/object metadata carriers for `ArrayAccess`,
  `Countable`, `Iterator`, `IteratorAggregate`, and `Stringable`.
- [ ] Diagnostic merge/severity-mask ABIs and branch diagnostic metadata.
- [ ] Request population policy, byte-preserving request-name work, and
  runtime/environment comparison-call blockers.
- [ ] Output-argument writeback frames for selected byte replacement builtins.
- [ ] Nested transient reference-source and array/lvalue diagnostic boundary
  refinements.

## Done

- [x] Selected native value, string, array, symbol-table, request-state,
  comparison, truthiness, diagnostic, exit, and reference-slot ABIs.
- [x] Generated-C request root/key/path reads, writes, unsets, appends,
  assignment-expression values, `isset()`, `empty()`, and selected `??` paths.
- [x] Static and dynamic selected `$GLOBALS[...]` request/symbol dispatch,
  including non-append reference paths and fatal direct no-key `$GLOBALS[]`
  rejection.
- [x] Selected generated-C array-query, array-lvalue, reference-backed lvalue,
  diagnostic cleanup, direct termination, and state-stable branch consumers.
- [x] Focused linked executable gates for the newest primary semantic slices.

## In Progress

- [ ] Compact primary integrations from lane-local candidate work, one
  semantic boundary at a time.
- [ ] Generated-native call/frame handoff, argument cleanup, return ownership,
  and dynamic call blockers. Estimate: **27%** `[#####---------------]`
- [ ] Reference/COW owner expansion beyond active symbol-root arrays and
  through real control flow. Estimate: **87%** `[#################---]`
- [ ] Object/property/method execution, including `$this`, static context,
  visibility/magic hooks, constructor behavior, and ArrayAccess boundaries.
  Estimate: **11%** `[##------------------]`
- [ ] Structured control flow beyond state-stable `if`/`else`: branch joins,
  loops, switch, goto, break/continue, cleanup stacks, and source-ordered
  diagnostics. Estimate: **34%** `[#######-------------]`
- [ ] Broader conversion/comparison behavior that removes shared blockers
  rather than adding one-off builtin slices.

## Not Done

- [ ] Full PHP references/COW, arbitrary writable roots, by-reference
  args/returns, and by-reference foreach parity.
- [ ] Full generated-native user function, method, closure, dynamic call,
  variadic/spread, frame-local symbol, and cleanup semantics.
- [ ] Real object construction, property/method dispatch, magic hooks,
  resources, `ArrayAccess`, and object-compatible diagnostics.
- [ ] Exact PHP diagnostics, warning masks, source spans, suppression/custom
  handlers, exception-like unwinding, shutdown callbacks, destructors, finally
  ordering, output buffers, and SAPI behavior.
- [ ] LLVM/assembly parity for newer generated-C/runtime ABI consumers.

## Steering Bias

Keep primary landing small executable generalized slices. Good next work would
remove one of the major shared blockers: generated-native call/frame execution,
object/property/method execution, reference/COW through real control flow,
structured cleanup and diagnostic ordering, or broad conversion/comparison
semantics. Continue rejecting nearby builtin-only expansions, interpreter-only
metadata progress, docs-only progress, and exact-shape lowering.
