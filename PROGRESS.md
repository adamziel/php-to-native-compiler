# PHP Native Compiler Progress

Updated: 2026-05-24 03:20 CEST
Evaluation marker: `20260523T235117Z`

Latest primary semantic/test baseline:
`de01bfe0 codegen: lower known dynamic user function calls`
Latest integrated semantic baseline: `de01bfe0 codegen: lower known dynamic user function calls`
Latest evaluator report: `20260523T235117Z`

These are candid engineering estimates toward generalized PHP semantics in the
native compiler. They are not test pass rates. Only primary-integrated,
pushed work counts; lane-local candidates, dirty WIP, parked diffs, and
exact-shape fixtures do not.

## Executive Read

Overall estimated progress: **52%** `[##########----------]`

Primary integrated progress now includes generated-C top-level state-stable
`goto`/label dispatch, state-stable `do...while`, normal-flow `try`/`finally`,
top-level `return` transfer through active generated-C `finally` bodies, and a
diagnostic-aware native value stdout formatter consumed by LLVM and generated-C
display paths. Generated-C non-strict comparison conditions and direct
comparison echoes now consume the shared native value comparison result
boundary across scalar, string, null/bool, builtin-result, and array operand
families. The compiler has strong generated-C islands for native values,
arrays, selected references, request/symbol state, diagnostics, cleanup, lazy
expressions, and bounded control flow.
Generated-native `strpos()` and `substr_count()` now share a PHP-shaped
string-search value-result ABI across LLVM IR and generated-C linked execution.
Generated-C now has a compact direct by-value user-function frame subset:
top-level declarations are registered before `main`, argument/default values
are cloned into callee-owned native handles, direct calls receive owned return
handles, fallthrough returns `null`, registered direct frames are visible to
generated-C `function_exists()` / `is_callable()` introspection, finite
known-string dynamic calls can dispatch to one registered by-value frame, and
unsupported frame shapes are rejected.

This is still not close to complete PHP execution. The foundation is strong,
but the remaining gaps are central language semantics rather than edge cases:
full generated-native calls/frames, object/property/method execution, complete
references/COW identity, source-ordered diagnostics, cleanup/unwinding, and
LLVM/assembly parity.

Current primary state: primary semantic head is `de01bfe0`. The earlier unowned
`compiler/src/interpreter.rs` formatting spillover was restored and is not
present. The former protected `runtime/src/lib.rs` null-slot hunk was rejected
after focused runtime proof showed it broke existing-key increment/decrement,
then parked outside the repo and restored. The earlier narrow generated-C
user-function WIP remains rejected; the accepted frame slices add explicit
frame ownership, caller handoff, cleanup/failure exits, registered
function-symbol introspection, known-string dynamic calls to registered frames,
and linked proof for a small direct/dynamic-call subset.

Current resource read: `/dev/shm` is above the dispatch floor after reclaiming
inactive exit/object-property build caches with live-owner checks. Keep broad
waves conservative and reclaim large inactive target dirs only after live-owner
checks.

## Roadmap Snapshot

| Workstream | Estimate | Bar | Current read |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | **79%** | `[################----]` | Strong shared value, array, reference, symbol, request, comparison, truthiness, string-search, diagnostic, termination, and cleanup surfaces, but several are still scaffolding until consumed end-to-end. |
| Compiler/backend consumers | **66%** | `[#############-------]` | Generated-C has broad selected coverage including a direct and finite known-string dynamic user-function frame subset, and LLVM consumes another value-result string ABI. LLVM/assembly parity remains uneven and many consumers still stop at blockers. |
| Executable PHP semantics | **47%** | `[#########-----------]` | Many focused linked programs run, including PHP-shaped string-search results and direct/known-string dynamic by-value function frames, but behavior is still selected islands rather than a complete PHP execution model. |
| Arrays, lvalues, references, COW | **58%** | `[############--------]` | Strong selected array/lvalue/reference paths. Full COW, arbitrary writable roots, and by-reference call/foreach parity remain open. |
| Symbols, globals, request state | **64%** | `[#############-------]` | Strong request and `$GLOBALS` generated-C coverage. Reconciliation across calls/requests still needs work. |
| Calls, functions, frames | **33%** | `[#######-------------]` | Generated-C now lowers a compact direct by-value user-function frame subset with owned argument/default/return handles, registered-function introspection, and finite known-string dynamic calls to one registered frame. Runtime dynamic lookup, methods, closures, by-reference/variadic frames, typed signatures, recursion, and mixed-target dispatch are still missing. |
| Objects, properties, methods | **10%** | `[##------------------]` | Mostly lane-local/runtime candidate work. Primary lacks general compiled object/property/method execution. |
| Control flow, cleanup, diagnostics | **44%** | `[#########-----------]` | Bounded generated-C branches, loops including state-stable `do...while`, returns, transfers, switches, top-level state-stable gotos, normal-flow try/finally, top-level return through finally, and diagnostic-aware stdout formatting exist; owner/reference joins, broad unwinding, handlers, and exact ordering remain open. |
| Broad integrated verification | **40%** | `[########------------]` | Focused gates are strong. Cross-feature composition, end-to-end PHP programs, and backend parity need much broader proof. |

## Done / In Progress / Not Done

- [x] Generated-C value-result diagnostics through shared diagnostic reporting.
- [x] Generated-C native value-result `strlen()` consumption.
- [x] Generated-native `strpos()` and `substr_count()` value results through a shared PHP-shaped string-search ABI.
- [x] Generated-C non-strict comparison conditions and direct echoes route through shared native value comparison results across scalar, string, null/bool, builtin-result, and array families.
- [x] Generated-C top-level `return`, state-stable `while`/`do...while`/`for`, scalar loop-carried slots, multi-level loop transfers, state-stable `switch` dispatch/fallthrough/break, top-level state-stable `goto` labels, normal-flow `try`/`finally`, and top-level return transfer through active `finally` bodies.
- [x] Diagnostic-aware native value stdout formatting consumed by LLVM and generated-C display paths.
- [x] Strong selected generated-C arrays, lvalues, references, request state, `$GLOBALS`, lazy ternaries, logical short-circuiting, branch cleanup, foreach storage, and output/truthiness paths.
- [x] Primary-integrated bounded generated-C direct by-value user-function/call-frame execution, registered-function introspection, and finite known-string dynamic calls to registered frames.
- [ ] Full generated-native user-function/call-frame execution across runtime dynamic calls, mixed-target dispatch, typed signatures, recursion, by-reference/variadic frames, closures, methods, and runtime lookup.
- [ ] Primary-integrated object construction, property access, method dispatch, `$this`, static context, visibility, and magic behavior.
- [ ] Full reference/COW identity across calls, arrays, objects, globals, foreach, and control-flow joins.
- [ ] Full structured cleanup/unwinding/finally/destructors/output-buffer/SAPI behavior.
- [ ] Exact diagnostic severity, ordering, suppression, handlers, spans, recovery, fatal/throw behavior.
- [ ] LLVM/assembly parity for newer generated-C/runtime ABI consumers.

## Recent Primary-Integrated Work

- `de01bfe0`: lowers generated-C dynamic calls when the callee expression has a
  finite known-string set that resolves to the same registered direct
  by-value user-function frame. The call reuses the existing owned
  argument/default/return handoff path and keeps unknown dynamic calls,
  callable builtins, mixed finite callees, dynamic calls inside generated
  function frames, closures, methods, recursion, and runtime lookup blocked.
  Focused proof covers generated-C source, linked executable output,
  case-insensitive callable spelling, nested dynamic calls used as direct-frame
  arguments, default arguments, and the broad native function-call boundary.
- `92728ce9`: wires generated-C `function_exists()` and `is_callable()` over
  known string values to the registered direct user-function table as well as
  the existing builtin function set. Focused proof covers generated-C source
  and linked executable behavior for case-insensitive user-function lookup,
  builtin lookup, missing-user-function false results, and subsequent direct
  user-function execution.
- `cbda996b`: lowers a compact generated-C direct by-value user-function frame
  subset. Top-level function declarations are registered before `main`,
  acyclic direct calls materialize argument/default values into callee-owned
  native handles, returns hand owned value handles back to callers, fallthrough
  returns owned `null`, and callee cleanup/failure exits release frame-local
  state. Focused proof covers generated-C source, linked executable output,
  nested direct calls, defaults, early return, fallthrough return, echo side
  effects inside functions, and blockers for nested functions, typed/by-ref/
  variadic parameters, typed returns, recursion, static locals, dynamic calls,
  and unsupported control-flow cleanup shapes.
- `3cc56bfd`: routes lowerable generated-native `strpos()` and
  `substr_count()` calls through
  `phpc_native_value_string_search_result_with_diagnostic(...)`, returning
  owned PHP values (`int` or `false`) instead of an int-only substring-count
  path or generic function-call rejection. Focused proof covers LLVM IR,
  assembly reachability, generated-C source, linked executable output, binary
  strings, offset/length conversion, missing-needle false results, stdout
  formatting, cleanup, and unsupported arity blockers.
- `089498c1`: generated-C now lowers accepted state-stable `do...while`
  loops through the same scoped loop-carried scalar storage, body/condition
  cleanup checks, and loop transfer targets used by the existing while/for
  family. Focused proof covers body-first execution, int/float/bool
  loop-carried slots, `continue` to the trailing condition, `break`, body-local
  native-value cleanup, and rejections for unsupported state joins or unbounded
  always-true conditions.
- `b53a1e88`: schedules active generated-C `finally` bodies before a top-level
  `return` terminates from inside a supported `try` body. Return operands still
  evaluate before the finally path, linked proof covers finally-observed array
  mutation and terminal cleanup, and exit/goto/throw/break/continue plus
  returns from finally bodies remain blocked until broader unwind scheduling
  exists.
- `1bd6d615`: routes generated-C non-strict comparison conditions and direct
  comparison echoes through `phpc_native_value_compare_result(...)`, replacing
  the older comparison-decision consumer for supported scalar, string,
  null/bool, builtin-result, and non-strict array comparison families while
  keeping strict array comparisons on the existing array branch boundary.
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

Lane-local work includes useful candidates around call-frame/reference
contracts, object/property boundaries, conversion/request diagnostics, native
loop cleanup blockers, array-key/value diagnostics, assembly helper fallback
parity, nested symbol result handles, and reference/COW-adjacent runtime ABI
surfaces. These remain uncounted until primary integration reviews and lands a
generalized semantic slice with focused source and executable proof.

The narrow generated-C user-function WIP was parked because it looked too much
like direct/top-level/single-return execution without real frame ownership,
caller handoff, cleanup/failure exits, or accepted linked proof. It should not
be counted or repeated in that shape.

## Active Focus

1. Integrate one small generalized primary slice at a time, with focused gates
   and immediate push.
2. Prefer hard-cliff work and executable consumers over more vocabulary:
   calls/frames, objects/properties/methods, references/COW through real
   control flow, structured cleanup/unwinding/source-ordered diagnostics, and
   backend parity.
3. Keep lane-local work code-writing only and reject exact-shape progress:
   no one-fixture recognizers, no fixed-key/value branches, no docs-only
   progress, and no generated-source substring proof without a shared semantic
   boundary.
4. Preserve primary hygiene: commit only owned semantic/progress hunks, keep
   parked or active WIP uncounted until it has its own reviewed semantics
   batch.

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
