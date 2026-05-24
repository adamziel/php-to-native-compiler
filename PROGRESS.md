# PHP Native Compiler Progress

Updated: 2026-05-24 04:55 CEST
Evaluation marker: `20260524T022032Z`

Latest primary semantic/test baseline:
`2cf2adda codegen: lower llvm string result operations`
Latest integrated semantic baseline: `2cf2adda codegen: lower llvm string result operations`
Latest evaluator report: `20260524T022032Z`

These are candid engineering estimates toward generalized PHP semantics in the
native compiler. They are not test pass rates. Only primary-integrated,
pushed work counts; lane-local candidates, dirty WIP, parked diffs, and
exact-shape fixtures do not.

## Progress Accounting Note

No compiler work was rolled back. The apparent drop from 88% to the current
58% is a correction in the estimating rubric, not a loss of integrated code.
The older high-80s number overstated completion by counting strong foundations,
selected generated-C islands, and focused gates as if they implied broad PHP
execution. Starting with `93f55aee docs: clarify completion progress`, the
estimate was recalibrated to count only primary-integrated, pushed progress
toward generalized end-to-end PHP semantics. Under that stricter rubric,
runtime/ABI foundations remain high, but user-visible executable PHP semantics
are still held back by calls/frames, objects/properties/methods, references/COW,
cleanup/unwinding, diagnostics, and backend parity.

Read the current numbers this way:

- **58% overall**: weighted progress across primary-integrated foundations,
  compiler/backend consumers, executable semantics, and verification.
- **54% executable PHP semantics**: the stricter user-visible estimate for how
  much generalized PHP behavior can actually execute today.
- **88% is retired**: it was an older, non-comparable estimate that counted too
  much lane-local and foundation-only work as completion.

## Executive Read

Overall estimated progress: **58%** `[############--------]`

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
LLVM IR and the LLVM-backed assembly path now also consume the shared native
string-result ABI for lowerable direct `strrev()`, `bin2hex()`, `str_rot13()`,
ASCII case transforms, and shell-escape result operations; generated-C keeps
the already integrated nested string-result execution, while nested LLVM
call-result operands remain blocked.
Generated-C now has a compact direct by-value user-function frame subset:
top-level declarations are registered before `main`, argument/default values
are cloned into callee-owned native handles, direct calls receive owned return
handles, fallthrough returns `null`, registered direct frames are visible to
generated-C `function_exists()` / `is_callable()` introspection, finite
known-string dynamic calls and runtime string-valued dynamic calls can dispatch
to registered by-value frames, recursive and mutually recursive by-value frames
execute behind a generated call-depth guard, and unsupported frame shapes are
rejected. Finite known-string dynamic calls to supported native builtin
families now canonicalize callable spelling and aliases and reuse the existing
generated-C native value/string/type builtin materializers instead of a
one-off callable recognizer. Runtime string-valued dynamic calls now also
dispatch to supported native builtin families after registered by-value frames,
using the same materialized callee/argument handles, dynamic-call name matcher,
and cleanup path. Finite known-string callable sets that mix registered
by-value user frames and supported native builtin families now reuse that same
runtime dispatch table when every possible spelling is supported, covering
user/user, user/builtin, and builtin/builtin target sets while unsupported
builtin, array-callable, method, closure, and broader callable forms stay
blocked. Request
superglobal roots now have a shared root-value operation with explicit
missing-root state after root `unset(...)`, and keyed writes can reseed an
unset root through the existing request-state boundary. Supported generated-C
by-value user-function parameter and return type metadata now executes through
a shared runtime type-coercion helper for scalar, nullable, union, array, and
mixed families, including linked success and mismatch diagnostics.

This is still not close to complete PHP execution. The foundation is strong,
but the remaining gaps are central language semantics rather than edge cases:
full generated-native calls/frames, object/property/method execution, complete
references/COW identity, source-ordered diagnostics, cleanup/unwinding, and
LLVM/assembly parity.

Current primary state: primary semantic head is `2cf2adda`. The LLVM
string-result backend-parity batch landed after IR proof across the whole
one-argument string-result operation family, LLVM-backed assembly reachability,
unsupported arity and nested-call blocker proof, existing generated-C linked
string-result proof, full native function-call boundary, string-case tests,
cargo-check, rustfmt, and diff gates passed.

Current resource read: `/dev/shm` is above the dispatcher floor at about 9.5G
free; `/home` has about 306G free.
Keep broad waves conservative and reclaim large inactive target dirs only after
live-owner checks.

## Roadmap Snapshot

| Workstream | Estimate | Bar | Current read |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | **82%** | `[################----]` | Strong shared value, array, reference, symbol, request, comparison, truthiness, string-search, diagnostic, termination, cleanup, request-root, call-frame type-coercion, and runtime dynamic-call result surfaces, but several are still scaffolding until consumed end-to-end. |
| Compiler/backend consumers | **73%** | `[###############-----]` | Generated-C has broad selected coverage including direct, recursive, finite known-string dynamic, runtime string-valued dynamic, bounded typed by-value user-function frame subsets, finite known-string plus runtime string-valued dynamic calls to supported native builtin families, and supported finite mixed user/builtin target sets. LLVM/assembly now consume the shared string-result ABI for lowerable direct operands, but parity remains uneven and many consumers still stop at blockers. |
| Executable PHP semantics | **54%** | `[###########---------]` | Many focused linked programs run, including PHP-shaped string-search results, request-root unset/reseed behavior, direct/recursive/known-string dynamic/runtime dynamic/typed by-value function frames, finite/runtime dynamic builtin calls, and finite mixed user/builtin dynamic calls, but behavior is still selected islands rather than a complete PHP execution model. |
| Arrays, lvalues, references, COW | **58%** | `[############--------]` | Strong selected array/lvalue/reference paths. Full COW, arbitrary writable roots, and by-reference call/foreach parity remain open. |
| Symbols, globals, request state | **66%** | `[#############-------]` | Strong request and `$GLOBALS` generated-C coverage now includes explicit missing-root state and root reseeding. Reconciliation across calls/requests still needs work. |
| Calls, functions, frames | **48%** | `[##########----------]` | Generated-C now lowers a compact by-value user-function frame subset with owned argument/default/return handles, registered-function introspection, recursive/mutually recursive direct frames behind a depth guard, finite known-string dynamic calls, runtime string-valued dynamic calls to registered frames, finite known-string and runtime string-valued dynamic calls to supported native builtin families, supported finite mixed user/builtin target sets, and bounded scalar/nullable/union/array/mixed parameter and return type enforcement. Full callable lookup, unsupported runtime callable builtin families, callable array/object forms, methods, closures, by-reference/variadic frames, full type-system coverage, and broader mixed callable dispatch are still missing. |
| Objects, properties, methods | **10%** | `[##------------------]` | Mostly lane-local/runtime candidate work. Primary lacks general compiled object/property/method execution. |
| Control flow, cleanup, diagnostics | **45%** | `[#########-----------]` | Bounded generated-C branches, loops including state-stable `do...while`, returns, transfers, switches, top-level state-stable gotos, normal-flow try/finally, top-level return through finally, diagnostic-aware stdout formatting, and corrected diagnostic-report ownership in generated stdout paths exist; owner/reference joins, broad unwinding, handlers, and exact ordering remain open. |
| Broad integrated verification | **47%** | `[#########-----------]` | Focused gates are strong and now include broader request-root, user-function, typed-frame, runtime dynamic-call, dynamic-builtin, finite mixed-callable, string-result backend-parity, and call-boundary filters. Cross-feature composition, end-to-end PHP programs, and backend parity need much broader proof. |

## Candidate Work Not Counted

As of this review, primary semantic progress is counted only through pushed
baseline `2cf2adda`. Active worker lanes continue to produce candidate work
that is not counted in the percentages until it lands in primary with focused
proof.

- Call-semantics cleanup ownership for native source-call failure paths.
- Function-frame result-consumer contracts for cleanup-sensitive values.
- Global-symbol diagnostic-ordering contracts across control-flow surfaces.
- Object/class metadata and method-call blocker refinements.
- Direct type-predicate consumers and additional array/lvalue cleanup routing.

## Done / In Progress / Not Done

- [x] Generated-C value-result diagnostics through shared diagnostic reporting.
- [x] Generated-C native value-result `strlen()` consumption.
- [x] Generated-native `strpos()` and `substr_count()` value results through a shared PHP-shaped string-search ABI.
- [x] LLVM IR/assembly direct string-result builtins consume the shared native string-result ABI for lowerable operands.
- [x] Generated-C non-strict comparison conditions and direct echoes route through shared native value comparison results across scalar, string, null/bool, builtin-result, and array families.
- [x] Generated-C top-level `return`, state-stable `while`/`do...while`/`for`, scalar loop-carried slots, multi-level loop transfers, state-stable `switch` dispatch/fallthrough/break, top-level state-stable `goto` labels, normal-flow `try`/`finally`, and top-level return transfer through active `finally` bodies.
- [x] Diagnostic-aware native value stdout formatting consumed by LLVM and generated-C display paths.
- [x] Strong selected generated-C arrays, lvalues, references, request state, `$GLOBALS`, lazy ternaries, logical short-circuiting, branch cleanup, foreach storage, and output/truthiness paths.
- [x] Primary-integrated bounded generated-C direct/recursive/typed by-value user-function/call-frame execution, registered-function introspection, finite known-string and runtime string-valued dynamic calls to registered frames, and request-superglobal root-value/missing-root state.
- [x] Generated-C finite known-string dynamic calls to supported native builtin families reuse shared builtin materialization and value-result paths.
- [x] Generated-C runtime string-valued dynamic calls dispatch to supported native builtin families through shared dynamic-call lookup and cleanup.
- [x] Generated-C finite mixed dynamic calls dispatch across supported registered user-function and native builtin-family target sets through the shared dynamic-call lookup and cleanup path.
- [ ] Full generated-native user-function/call-frame execution across full callable lookup, unsupported runtime callable builtin families, callable array/object forms, full PHP type metadata, by-reference/variadic frames, closures, and methods.
- [ ] Primary-integrated object construction, property access, method dispatch, `$this`, static context, visibility, and magic behavior.
- [ ] Full reference/COW identity across calls, arrays, objects, globals, foreach, and control-flow joins.
- [ ] Full structured cleanup/unwinding/finally/destructors/output-buffer/SAPI behavior.
- [ ] Exact diagnostic severity, ordering, suppression, handlers, spans, recovery, fatal/throw behavior.
- [ ] LLVM/assembly parity for newer generated-C/runtime ABI consumers.

## Recent Primary-Integrated Work

- `2cf2adda`: LLVM direct string-result builtins now lower lowerable operands
  through `phpc_native_value_string_result_operation_with_diagnostic(...)`,
  matching the shared runtime ABI already used by generated-C for `strrev()`,
  `bin2hex()`, `str_rot13()`, ASCII case transforms, and shell-escape result
  operations. The slice deliberately does not claim nested LLVM call-result
  operands: `strtoupper(strtolower(...))` remains blocked in LLVM at the
  shared call-result boundary, while generated-C keeps its linked nested
  string-result behavior. Focused proof covers IR source, LLVM-backed
  assembly reachability, unsupported arity blockers, nested-call blockers,
  existing generated-C linked string-result execution, full native
  function-call boundary, string-case tests, cargo-check, rustfmt, and diff
  gates.
- `59f3be42`: generated-C finite known-string dynamic calls whose possible
  spellings mix registered by-value user-function frames and supported native
  builtin families now reuse the shared runtime dynamic-call dispatch path.
  The compiler materializes the callee and arguments once, gates entry on all
  finite spellings being supported, then dispatches through
  `phpc_native_value_dynamic_call_name_matches(...)` over registered frames
  and supported builtin-family branches. Unsupported builtin families, array
  callable forms, methods, closures, broader callable lookup, by-reference/
  variadic frames, exact callable diagnostics, references/COW through calls,
  cleanup/unwinding, and LLVM/assembly parity remain blocked. Focused proof
  covers generated-C source, linked executable user/user, user/builtin, and
  builtin/builtin target sets, unsupported builtin blockers, runtime
  array-callable failure, closure/method blockers, adjacent dynamic-builtin/
  dynamic-user/user-function filters, the full native function-call boundary,
  cargo-check, rustfmt, and diff gates.
- `8790f3a4`: generated-C runtime string-valued dynamic calls now continue
  past registered by-value user-function frames into supported native builtin
  families. The compiler still materializes the callee and all arguments once,
  then uses `phpc_native_value_dynamic_call_name_matches(...)` for the shared
  lookup and routes supported string length, string result, string predicate,
  cast, type-name, and type-predicate families through existing builtin
  materializers with shared cleanup. Unsupported runtime builtin families,
  mixed finite targets, callbacks, methods, closures, and exact callable
  diagnostics remain blocked. Focused proof covers generated-C source,
  linked executable success, linked unsupported-builtin failure, adjacent
  dynamic user-function/user-function filters, the full native function-call
  boundary, cargo-check, rustfmt, and diff gates.
- `1209d8cb`: generated-C finite known-string dynamic calls now route
  supported native builtin families through a canonical callable-builtin
  classifier and the existing direct builtin materializers rather than a
  fixture-shaped callable special case. The accepted surface covers string
  length, string result, string predicate, string search, type-name, and
  type-predicate families with case-insensitive spellings and aliases, while
  unsupported builtin families, mixed finite targets, runtime callable
  builtins, callbacks, methods, closures, and exact callable diagnostics stay
  blocked. Focused proof covers generated-C source emission, linked executable
  output, unsupported-family and mixed-target blockers, adjacent dynamic
  user-function filters, user-function filters, cargo-check, rustfmt, and diff
  gates.
- `61b609cd`: generated-C dynamic calls now dispatch runtime string-valued
  callees through the registered by-value user-function frame table instead
  of requiring a finite compile-time string set. The compiler materializes the
  callee and arguments once, checks each registered frame name through
  `phpc_native_value_dynamic_call_name_matches(...)`, and routes unknown
  strings, non-string callables, and arity/default mismatches through
  `phpc_native_value_dynamic_call_failure_with_diagnostic(...)` plus the
  existing generated-C cleanup path. Focused proof covers runtime helper
  matching/failure diagnostics, generated-C source emission over multiple
  registered frames, linked executable success through an indirect `invoke`
  frame, linked unknown/builtin, arity, and non-string failures, adjacent
  known-string dynamic calls, full user-function filters, full runtime tests,
  and the full native function-call boundary.
- `2f599360`: generated-C by-value user-function frames now admit supported
  scalar, nullable, union, array, and mixed parameter/return type metadata
  instead of rejecting all typed signatures. Parameters and return values route
  through `phpc_native_value_coerce_call_type_with_diagnostic(...)`, which
  reuses the runtime's existing weak scalar/array/null type boundary and
  returns owned value handles plus diagnostics. Focused proof covers runtime
  helper coercion, generated-C source metadata emission, linked executable
  success for typed params/defaults/returns/mixed passthrough, linked
  parameter and return mismatch diagnostics, adjacent user-function filters,
  and the full native function-call boundary.
- `528d1d26`: advances two shared generated-C/runtime boundaries. Recursive
  and mutually recursive by-value user-function frames now execute through the
  existing owned argument/default/return handoff with a generated call-depth
  guard, and finite known-string dynamic calls can dispatch from inside
  supported frames when they resolve to one registered frame. Request
  superglobal roots now expose a root-value operation, root `unset(...)`
  records explicit missing-root state, `isset()`/`empty()`/direct root reads
  consume that boundary, and keyed writes can reseed an unset root. The batch
  also removes a generated stdout diagnostic double-free pattern by respecting
  `phpc_native_diagnostic_report(...)` ownership. Focused proof covers runtime
  request-state operations, linked request-root unset/reseed behavior,
  recursive direct/mutual/dynamic-in-frame user calls, adjacent user-function
  filters, and the full native function-call boundary.
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
contracts, object/property boundaries, object-to-string blocker classification,
array/lvalue cleanup routing, `array_slice()` transforms, conversion/request
diagnostics, native loop cleanup blockers, array-key/value diagnostics,
assembly helper fallback parity, symbol-table handles, and reference/COW
adjacent runtime ABI surfaces. These remain uncounted until primary
integration reviews and lands a generalized semantic slice with focused source
and executable proof.

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
