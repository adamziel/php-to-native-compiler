# PHP Native Compiler Progress

Updated: 2026-05-23 02:11 CEST

Primary baseline: `106cc646 codegen: lower scoped native if branches`
Evaluator cadence: every 45 minutes; latest consumed evaluator report is
`20260522T233339Z`, next scheduled report is `2026-05-23T02:23:15+02:00`.

These percentages are candid engineering estimates toward generalized PHP
semantics in the native compiler. They are not test pass rates. Lane-local
work, uncommitted diffs, and exact-shape fixtures do not count here.

## Executive Read

Overall estimated progress: **86%** `[#################---]`

The project is moving in the right direction, but most recent progress is still
about making selected generated-native paths execute through shared PHP
semantic boundaries. The current primary baseline now includes three important
generated-C execution slices:

- bounded `if`/`else` lowering for state-stable branches whose conditions route
  through native truthiness or comparison boundaries;
- bounded direct `exit()`/`die()` lowering for no-arg, `null`, `int`, and
  `string` operands with shared cleanup;
- generated-C diagnostic report ownership cleanup so reported diagnostics are
  consumed consistently.

This is real generalized movement, not a single PHP fixture. It is still not
full PHP control flow, calls, objects, references, or diagnostics.

The only expected dirty primary diff is the preserved
`runtime/src/lib.rs` null-slot increment/decrement hunk. It is unintegrated and
not counted.

## Roadmap

| Roadmap item | Estimate | Status |
| --- | ---: | --- |
| Runtime and ABI foundations | **97%** `[###################-]` | Strong value, array, symbol-table, request-state, reference, comparison, truthiness, diagnostic, and exit-result surfaces. |
| Compiler/backend consumers | **97%** `[###################-]` | Good generated-C coverage for selected request, `$GLOBALS`, symbol, array-query, lvalue, reference, exit, diagnostic, and scoped-branch consumers. LLVM/assembly parity is uneven. |
| Executable PHP semantics | **84%** `[#################---]` | Improving through linked executable gates, but still selected islands rather than a complete native PHP execution model. |
| Arrays, lvalues, references, COW | **87%** `[#################---]` | Strong selected paths, including reference-backed active symbol-root array lvalues. Arbitrary writable roots, full COW, and by-reference foreach remain large. |
| Symbols, globals, request state | **96%** `[###################-]` | Strong request/`$GLOBALS` generated-C coverage. Broader request/global reconciliation remains open. |
| Calls, functions, frames | **27%** `[#####---------------]` | Runtime/interpreter call-frame metadata enforcement landed; generated-native call/frame execution remains the major missing piece. |
| Objects, properties, methods | **11%** `[##------------------]` | Mostly lane-local/runtime candidate work; primary still lacks general compiled object/property/method execution. |
| Control flow, cleanup, diagnostics | **34%** `[#######-------------]` | Direct exit, diagnostic ownership, and bounded state-stable `if`/`else` now execute on generated C. Loops, switch, branch joins, exact diagnostic ordering, shutdown/finally/destructors, and exception-like unwinding are not generalized. |
| Broad integrated verification | **86%** `[#################---]` | Focused gates are strong; cross-feature composition and backend parity still need broader proof. |

## Recent Integrated Work

- `106cc646`: generated-native C-link `if`/`else` lowering now emits scoped
  branch bodies when branch conditions use existing native truthiness or value
  comparison boundaries and both branches leave persistent compiler state
  unchanged. It rejects branches needing environment merge/phi reconciliation
  instead of silently losing locals or cleanup ownership. Focused proof covers
  truthiness conditions, comparison conditions, linked executable output, and
  rejection for branch assignments that need persistent environment merging.
- `2545d173`: generated-C diagnostic-report cleanup now uses one owner-consuming
  report helper and guards against report-then-free double ownership.
- `0cfae034`: generated-native direct `exit()`/`die()` now handles no-argument,
  `null`, `int`, and `string` operands on the C-link backend with shared
  cleanup and focused linked executable proof.
- `9ade9293`: runtime/interpreter call-frame execution enforces declared
  by-value parameter/default/return type metadata across ordinary calls,
  dynamic string calls, methods, closures, `call_user_func()`, and reflection.
  This is useful call-frame infrastructure, but it is not generated-native call
  lowering.
- `83e08c94`: generated-C active ordinary symbol roots route array-lvalue
  operations through reference-backed owner slots after references activate the
  native symbol table.

## Done

- Selected native value, string, array, symbol-table, request-state,
  comparison, truthiness, diagnostic, exit, and reference-slot ABIs.
- Generated-C request root/key/path reads, writes, unsets, appends,
  assignment-expression values, `isset()`, `empty()`, and selected `??` paths.
- Static and dynamic selected `$GLOBALS[...]` request/symbol dispatch,
  including non-append reference paths and fatal direct no-key `$GLOBALS[]`
  rejection.
- Selected generated-C array-query, array-lvalue, reference-backed lvalue,
  diagnostic cleanup, direct termination, and state-stable branch consumers.
- Focused linked executable gates for the newest primary semantic slices.

## In Progress

- Compact primary integrations from lane-local candidate work, one semantic
  boundary at a time.
- Reference/COW owner expansion beyond active symbol-root arrays.
- Generated-native call/frame handoff, argument cleanup, return ownership, and
  dynamic call blockers.
- Object/property/method execution, including `$this`, static context,
  visibility/magic hooks, constructor behavior, and ArrayAccess boundaries.
- Structured control flow beyond state-stable `if`/`else`: branch joins,
  loops, switch, goto, break/continue, cleanup stacks, and source-ordered
  diagnostics.

## Not Done

- Full PHP references/COW, arbitrary writable roots, by-reference args/returns,
  and by-reference foreach parity.
- Full generated-native user function, method, closure, dynamic call,
  variadic/spread, frame-local symbol, and cleanup semantics.
- Real object construction, property/method dispatch, magic hooks, resources,
  `ArrayAccess`, and object-compatible diagnostics.
- Exact PHP diagnostics, warning masks, source spans, suppression/custom
  handlers, exception-like unwinding, shutdown callbacks, destructors, finally
  ordering, output buffers, and SAPI behavior.
- LLVM/assembly parity for newer generated-C/runtime ABI consumers.

## Steering Bias

Primary integration should keep landing small executable generalized slices.
Reject exact-shape lowering, nearby builtin-only expansions, docs-only progress,
and helper vocabulary that does not unlock behavior.

Highest-value next work:

- generated-native call/frame handoff;
- object/property/method execution;
- reference/COW owner slots through real control flow;
- structured cleanup and diagnostic ordering;
- broader conversion/comparison behavior that removes shared blockers.
