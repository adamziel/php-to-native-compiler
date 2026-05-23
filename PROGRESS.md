# PHP Native Compiler Progress

Updated: 2026-05-23 05:23 CEST
Evaluation marker: `20260523T025130Z`

Primary HEAD: `dcd99134 codegen: terminate top-level returns`
Latest integrated semantic baseline: `dcd99134 codegen: terminate top-level returns`
Latest evaluator report: `20260523T025130Z`

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
as native arrays and tracked byte buffers. Stored native values can now feed
shared type predicates and `gettype` / `get_debug_type` result ABIs instead of
falling back to return-value ownership rejection, and `isset()` / `empty()`
now classify stored native values through shared null predicates and PHP
truthiness. Native array owners now also feed shared PHP truthiness for
`empty()`, `if` conditions, logical operands, and unary-not checks, and array
owners now route echo/print output through the shared native-value stdout ABI.
Native value-operation diagnostics now report through the shared diagnostic
consumer, and `strlen()` can consume owned native value-result producers through
the shared value-to-string conversion boundary. Generated-C `while` statements
now execute when the condition and body both stay within existing scoped
truthiness/value-result and cleanup-stability boundaries. Generated-C
main-script `return` statements now terminate the executable path after
evaluating and cleaning return operands and live native owners.

This is still selected compiled PHP execution, not complete PHP semantics.
Calls/functions/frames, object/property/method execution, full references/COW,
arbitrary by-reference foreach, structured cleanup/unwinding, exact
diagnostics, and LLVM/assembly parity remain the major completion blockers.

Current primary cleanliness: semantic work is current through the top-level
`return` terminal-transfer batch. The protected `runtime/src/lib.rs`
null-slot hunk is still dirty and uncounted.

## Grand Roadmap

| Roadmap item | Estimate | Visual | Current read |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | **97%** | `[###################-]` | Strong value, array, symbol-table, request-state, reference, comparison, truthiness, diagnostic, exit, and cleanup surfaces. |
| Compiler/backend consumers | **98%** | `[####################]` | Good generated-C coverage for selected request, globals, arrays, lvalues, references, foreach by-value storage, strict value-result comparison, exit, diagnostics, state-stable branches/while loops, branch owner joins, lazy ternaries, logical short-circuiting, and statement cleanup. LLVM/assembly parity is uneven. |
| Executable PHP semantics | **87%** | `[#################---]` | Improving through linked executable gates, but still selected islands rather than a complete execution model. |
| Arrays, lvalues, references, COW | **88%** | `[##################--]` | Strong selected paths, including reference-backed active symbol-root array lvalues, by-value foreach cursor storage, and selected by-reference foreach slot execution. Arbitrary writable roots, full COW, by-reference arguments, and broader by-reference foreach remain open. |
| Symbols, globals, request state | **96%** | `[###################-]` | Strong request/`$GLOBALS` generated-C coverage. Broader request/global reconciliation remains open. |
| Calls, functions, frames | **27%** | `[#####---------------]` | Runtime/interpreter call-frame metadata enforcement exists, but generated-native call/frame execution is still the major missing piece. |
| Objects, properties, methods | **11%** | `[##------------------]` | Mostly lane-local/runtime candidate work. Primary still lacks general compiled object/property/method execution. |
| Control flow, cleanup, diagnostics | **41%** | `[########------------]` | Direct exit, diagnostic ownership/reporting, top-level return termination, bounded `if`/`else`, selected branch joins, logical short-circuiting, state-stable `while`, branch-local value/non-value cleanup, and statement cleanup execute on generated C. Loop-carried joins, function returns/frames, break/continue, switch, cleanup stacks, exact ordering, and unwinding are not generalized. |
| Broad integrated verification | **87%** | `[#################---]` | Focused gates are strong. Cross-feature composition and backend parity need broader proof. |

## Primary-Integrated Progress

- [x] `dcd99134`: generated-C main-script `return` statements now evaluate
  optional operands for side effects, release discarded operand values and live
  native owners through existing cleanup paths, and terminate the linked
  executable path with PHP CLI-compatible success status. User-function
  return, include/require return values, and frame handoff remain out of scope.
- [x] `6b5e480f`: generated-C `while` statements now emit scoped C loops when
  the condition and body leave compiler-visible ownership state unchanged,
  re-evaluate the condition inside the loop through shared truthiness/value
  boundaries, release condition/body temporaries per iteration, and reject loop
  state joins that need broader cleanup phis.
- [x] `9a05a58b`: generated-C `strlen()` now consumes owned native
  value-result producers through the shared native value materialization and
  value-to-string conversion boundary, with linked proof across string-result,
  cast, type-name, and binary value producers.
- [x] `4efa12ba`: generated-C native value-operation diagnostics for bitwise,
  cast, string-result, type-name, and operation-result families now route
  through the shared diagnostic report consumer and clear reported handles
  before later cleanup.
- [x] `2f7c236a`: generated-C echo/print over native array owners now
  materializes the owner as a PHP-shaped native value and routes output through
  the shared native-value stdout ABI.
- [x] `15f20e03`: generated-C native array owners now materialize through the
  shared native-value truthiness ABI for `empty()`, `if` conditions, logical
  operands, and unary-not checks.
- [x] `8f82a50f`: generated-C `isset()` and `empty()` over stored native-value
  owners now use shared null type predicates and native PHP truthiness instead
  of rejecting on return-value ownership, covering string, arithmetic, and cast
  value owners.
- [x] `2035a863`: generated-C stored native values and native references can
  feed shared `is_*` type predicates plus `gettype` / `get_debug_type`
  type-name result ABIs, covering values produced by arithmetic, string,
  casts, comparisons, and array casts.
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
- [ ] Lane-local generated if-statement value truthiness from
  `impl-array-value-runtime`, useful only if narrowed into a clean primary
  slice without bypassing branch-state merge and cleanup blockers.
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
  diagnostic cleanup, direct termination, top-level return termination,
  state-stable branch, branch-owner, state-stable `while`, branch-local
  cleanup, foreach by-value storage, selected by-reference foreach slots,
  strict value-result comparison, value-operation diagnostic reporting,
  value-result string conversion, lazy expression, logical short-circuit, and
  statement-discard cleanup consumers.
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
- [ ] Structured control flow beyond selected `if`/`else`, state-stable
  `while`, top-level `return`, and expression cleanup: broader cleanup-owner
  joins, loop-carried state, function-return/frame handoff, switch, goto,
  break/continue, cleanup stacks, and source-ordered diagnostics. Estimate:
  **41%**
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
ordering, or backend parity. Treat `4efa12ba`, `9a05a58b`, `6b5e480f`, and
`dcd99134` as non-repeat; future diagnostic/string-conversion/control-flow
work should target source ordering, suppression/custom handlers,
object/resource/string hooks, loop-carried cleanup/state, real function
return/frame handoff, terminal transfers beyond top-level return,
cleanup/unwind ordering, or backend parity. Continue rejecting docs-only
progress, nearby builtin-only expansions, interpreter-only metadata patches,
and exact-shape lowering.
