# PHP Native Compiler Progress

Updated: 2026-05-23 04:36 CEST
Evaluation marker: `20260523T020211Z`

Primary HEAD: `10be1209 codegen: cleanup branch-local non-value owners`
Latest integrated semantic baseline: `10be1209 codegen: cleanup branch-local non-value owners`
Latest evaluator report: `20260523T020211Z`

These percentages are candid engineering estimates toward generalized PHP
semantics in the native compiler. They are not test pass rates. Primary
integrated work counts; lane-local candidates, dirty primary WIP, untracked
targets, and exact-shape fixtures do not.

## Executive Read

Overall estimated progress: **87%** `[#################---]`

Primary has strong momentum in generated executable C ownership, cleanup, and
selected reference execution slices. Recent integrated work covers selected
branch owner joins, by-value foreach cursor preservation, branch-local
native-value cleanup, statement boundary cleanup for discarded native-value
expression results, by-reference foreach slot execution for selected array
lvalue owners, strict identity/non-identity routing through the shared native
value-result comparison ABI, and branch-local cleanup for non-value owners such
as native arrays and tracked byte buffers.

This is still selected compiled PHP execution, not complete PHP semantics.
Calls/functions/frames, object/property/method execution, full references/COW,
arbitrary by-reference foreach, structured cleanup/unwinding, exact
diagnostics, and LLVM/assembly parity remain the major completion blockers.

Current primary cleanliness: semantic work is current through the branch-local
non-value-owner cleanup batch. The protected `runtime/src/lib.rs` null-slot
hunk is still dirty and uncounted.

## Grand Roadmap

| Roadmap item | Estimate | Visual | Current read |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | **97%** | `[###################-]` | Strong value, array, symbol-table, request-state, reference, comparison, truthiness, diagnostic, exit, and cleanup surfaces. |
| Compiler/backend consumers | **98%** | `[####################]` | Good generated-C coverage for selected request, globals, arrays, lvalues, references, foreach by-value storage, strict value-result comparison, exit, diagnostics, state-stable branches, branch owner joins, lazy ternaries, logical short-circuiting, and statement cleanup. LLVM/assembly parity is uneven. |
| Executable PHP semantics | **86%** | `[#################---]` | Improving through linked executable gates, but still selected islands rather than a complete execution model. |
| Arrays, lvalues, references, COW | **88%** | `[##################--]` | Strong selected paths, including reference-backed active symbol-root array lvalues, by-value foreach cursor storage, and selected by-reference foreach slot execution. Arbitrary writable roots, full COW, by-reference arguments, and broader by-reference foreach remain open. |
| Symbols, globals, request state | **96%** | `[###################-]` | Strong request/`$GLOBALS` generated-C coverage. Broader request/global reconciliation remains open. |
| Calls, functions, frames | **27%** | `[#####---------------]` | Runtime/interpreter call-frame metadata enforcement exists, but generated-native call/frame execution is still the major missing piece. |
| Objects, properties, methods | **11%** | `[##------------------]` | Mostly lane-local/runtime candidate work. Primary still lacks general compiled object/property/method execution. |
| Control flow, cleanup, diagnostics | **38%** | `[########------------]` | Direct exit, diagnostic ownership, bounded `if`/`else`, selected branch joins, logical short-circuiting, branch-local value/non-value cleanup, and statement cleanup execute on generated C. Loops, switch, cleanup stacks, exact ordering, and unwinding are not generalized. |
| Broad integrated verification | **87%** | `[#################---]` | Focused gates are strong. Cross-feature composition and backend parity need broader proof. |

## Primary-Integrated Progress

- [x] `10be1209`: generated-C `if`/`else` branch lowering releases
  branch-local native array owners and tracked byte-buffer owners before
  rejoining when they do not survive the branch, while rejecting byte-buffer
  values that would remain live after the join.
- [x] `7d4864f6`: generated-C strict `===` and `!==` value-result operands
  route through the shared native value comparison result ABI, with runtime
  strict comparison opcodes and focused linked proof.
- [x] `1914035b`: generated-C by-reference `foreach` over tracked array
  lvalue owners reacquires each live slot as a native reference, exposes loop
  values through reference handles, writes assignments back through
  `phpc_native_reference_set_value()`, and rejects lingering post-loop value
  references unless explicitly unset.
- [x] `27387bd2`: generated-C discarded expression statements now release owned
  native-value results at the statement boundary, with source and linked
  executable proof.
- [x] `a29f292f`: generated-C `if`/`else` branch lowering releases discarded
  branch-local native values on join paths when accepted state remains
  reconcilable.
- [x] `8c504cf8`: generated-C by-value foreach preserves pre-existing key/value
  cursor variables through owned native-value storage, including empty-loop
  preservation.
- [x] `e87fb85b`: generated-C `if`/`else` can join selected branch-created or
  branch-carried owned native-value handles into one post-branch owner.
- [x] `481bc961`: generated-C dynamic logical `&&`/`||` use real short-circuit
  RHS branches for selected native truthiness operands.
- [x] `76ea0597` and `a1ab542a`: generated-C lazy short ternaries and ternaries
  lower through selected owned native-value result transfer.
- [x] Earlier generated-C request roots, `$GLOBALS`, symbol-table operations,
  array queries/lvalues, reference-backed active symbol-root lvalue owners,
  direct `exit()`/`die()`, diagnostic cleanup, and bounded state-stable branch
  consumers remain integrated.

## Lane-Local Or Dirty Candidate Work

Not counted until primary integrates it cleanly:

- [ ] Lane candidates around expression-position by-reference array mutator
  writeback, known string callback reducers, clone-source conversion results,
  non-local unset owner classifiers, diagnostic/error-reporting masks, PCRE
  byte-pattern handling, JSON/serialization surfaces, and object/call/reference
  metadata.
- [ ] Generated named-call/callback-call blockers routed through shared
  conversion/result cleanup paths.
- [ ] Object/property/method execution candidates, including ArrayAccess,
  Countable, Iterator, IteratorAggregate, Stringable, object clone, static
  property, and object-vars metadata.

## Done

- [x] Selected native value, string, array, symbol-table, request-state,
  comparison, truthiness, diagnostic, exit, and reference-slot ABIs.
- [x] Generated-C request root/key/path reads, writes, unsets, appends,
  assignment-expression values, `isset()`, `empty()`, and selected `??` paths.
- [x] Static and dynamic selected `$GLOBALS[...]` request/symbol dispatch,
  including non-append reference paths and fatal direct no-key `$GLOBALS[]`
  rejection.
- [x] Selected generated-C array query, array lvalue, reference-backed lvalue,
  diagnostic cleanup, direct termination, state-stable branch, branch-owner,
  branch-local cleanup, foreach by-value storage, selected by-reference
  foreach slots, strict value-result comparison, lazy expression, logical
  short-circuit, and statement-discard cleanup consumers.
- [x] Focused source and linked executable gates for the newest primary
  semantic slices.

## In Progress

- [ ] Broader by-reference foreach parity beyond selected array lvalue owners:
  temporary iterable owners, arbitrary loop-body mutation, lingering post-loop
  references, symbol/request owners, ArrayAccess/object/resource iteration,
  exact cleanup/error edges, and LLVM/assembly parity.
- [ ] Generated-native call/frame handoff, argument cleanup, return ownership,
  and dynamic call blockers. Estimate: **27%** `[#####---------------]`
- [ ] Reference/COW owner expansion beyond active symbol-root arrays and through
  real control flow. Estimate: **87%** `[#################---]`
- [ ] Object/property/method execution, including `$this`, static context,
  visibility/magic hooks, constructor behavior, and ArrayAccess boundaries.
  Estimate: **11%** `[##------------------]`
- [ ] Structured control flow beyond selected `if`/`else` and expression
  cleanup: broader cleanup-owner joins, loops, switch, goto, break/continue,
  cleanup stacks, and source-ordered diagnostics. Estimate: **38%**
  `[########------------]`
- [ ] Broader conversion/comparison/callback behavior that removes shared
  blockers rather than adding one-off builtin slices.

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

Keep landing small executable generalized slices. The by-reference foreach and
strict value-result comparison slices landed because they targeted real shared
blockers with clean staging and focused proof. The best next work removes major
shared blockers:
generated-native call/frame execution, object/property/method execution,
reference/COW through real control flow, broader cleanup and diagnostic
ordering, or backend parity. Continue rejecting docs-only progress, nearby
builtin-only expansions, interpreter-only metadata patches, and exact-shape
lowering.
