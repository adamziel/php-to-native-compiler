# PHP Native Compiler Progress

Updated: 2026-05-24 05:45 CEST
Evaluation marker: `20260524T031006Z`

Latest primary semantic/test baseline:
`9fa9aa92 codegen: lower variadic user function frames`

Latest integrated semantic baseline: `9fa9aa92 codegen: lower variadic user function frames`
Latest evaluator report: `20260524T031006Z`

Current primary git state:

- `master` contains `9fa9aa92 codegen: lower variadic user function frames`
  on top of `1580aeaa docs: update progress after nested llvm call operands`.
  After this progress update is pushed, `origin/master` should match the
  progress update commit.
- No primary semantic WIP remains in the worktree.

These are candid engineering estimates toward generalized PHP semantics in the
native compiler. They are not test pass rates. Only primary-integrated, pushed
work counts; lane-local candidates, dirty WIP, parked diffs, and exact-shape
fixtures do not.

## Progress Accounting Note

No compiler work was rolled back. The old high-80s completion number is retired
because it counted strong foundations, lane-local candidates, and selected
generated-C islands as if they implied broad PHP execution. The stricter rubric
counts only primary-integrated progress toward generalized end-to-end PHP
semantics. Under that rubric, runtime/ABI foundations are strong, but broad
user-visible PHP execution is still held back by calls/frames, objects,
properties, methods, references/COW, cleanup/unwind, diagnostics, and backend
parity.

## Executive Read

Overall estimated progress: **59%** `[############--------]`

Executable PHP semantics: **55%** `[###########---------]`

The primary branch is advancing at a useful pace. Recent integrated work
improved generated-C dynamic calls across registered by-value user frames and
supported native builtin families, added bounded type enforcement and variadic
argument packing for by-value frames, and moved LLVM closer to generated-C for
direct string-result, string-predicate, string-search, string-int, and
`strlen()` operand families through shared native ABIs.

This is still selected island execution, not complete PHP. The hard remaining
work is central language behavior: full callable lookup, closures, methods,
objects/properties, by-reference frames, named/unpacked arguments,
reference/COW identity, source-ordered diagnostics, cleanup/unwind/finally/
destructors, and broad backend parity.

## Grand Roadmap Position

| Workstream | Estimate | Bar | Current read |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | **82%** | `[################----]` | Strong shared value, array, reference, symbol, request, comparison, truthiness, string, diagnostic, cleanup, request-root, call-frame type-coercion, and dynamic-call surfaces. Some remain scaffolding until consumed end to end. |
| Compiler/backend consumers | **75%** | `[###############-----]` | Generated-C has broad selected coverage. LLVM now consumes shared direct string-result, string-predicate, string-search, string-int, and selected `strlen()` nested operand ABIs. Direct assembly and many nested/backend consumers still stop at blockers. |
| Executable PHP semantics | **55%** | `[###########---------]` | Many focused linked programs run, but behavior is still selected islands rather than a complete PHP execution model. |
| Arrays, lvalues, references, COW | **58%** | `[############--------]` | Strong selected array/lvalue/reference paths. Full COW, arbitrary writable roots, by-reference calls, foreach parity, and object/reference joins remain open. |
| Symbols, globals, request state | **66%** | `[#############-------]` | Request roots and selected `$GLOBALS` paths are strong. Reconciliation across calls, requests, includes, and aliases remains incomplete. |
| Calls, functions, frames | **50%** | `[##########----------]` | Bounded generated-C by-value fixed/default/variadic frames, typed params/returns, recursion guards, registered introspection, dynamic user calls, dynamic builtin calls, and finite mixed user/builtin sets are integrated. Full callable lookup, closures, methods, by-reference frames, named/unpacked arguments, and broader type behavior remain missing. |
| Objects, properties, methods | **10%** | `[##------------------]` | Mostly lane-local/runtime candidate work. Primary lacks general compiled object construction, property access, method dispatch, `$this`, visibility, static context, and magic behavior. |
| Control flow, cleanup, diagnostics | **45%** | `[#########-----------]` | Bounded generated-C branches, loops, transfers, switch/goto, normal-flow `try`/`finally`, return-through-finally, diagnostic-aware stdout formatting, and selected cleanup paths exist. Broad unwind, handlers, destructors, output buffers, and exact ordering remain open. |
| Broad integrated verification | **49%** | `[##########----------]` | Focused gates are strong. Cross-feature composition, end-to-end PHP programs, backend parity, and the unfiltered `native_runtime_abi` debt need broader proof. |

## Recent Primary-Integrated Work

- `9fa9aa92`: generated-C by-value variadic user-function frames now pack
  surplus positional arguments through the shared native array/value ABI across
  direct, finite known-string dynamic, and runtime string-valued dispatch, with
  typed variadic elements routed through the existing call-frame type-coercion
  diagnostic path.
- `2633fe55`: LLVM direct string/native-value consumers now admit lowerable
  nested direct call-result operands across string-result, string-predicate,
  string-search, string-int, and selected `strlen()` paths. The compiler keeps
  dynamic calls, methods, constructors, closures, unknown calls, unsupported
  builtin families, and direct assembly on shared blockers.
- `ac875386`: LLVM direct string-predicate builtins now lower lowerable
  operands through `phpc_native_value_string_predicate_with_diagnostic(...)`
  for `str_starts_with()`, `str_ends_with()`, and `str_contains()`. Direct
  assembly remains explicitly blocked.
- `2cf2adda`: LLVM direct string-result builtins now lower lowerable operands
  through `phpc_native_value_string_result_operation_with_diagnostic(...)` for
  `strrev()`, `bin2hex()`, `str_rot13()`, ASCII case transforms, and
  shell-escape result operations. Nested LLVM call-result operands remained
  blocked in that accepted slice.
- `59f3be42`: generated-C finite known-string dynamic calls now dispatch
  across supported registered user-function and native builtin-family target
  sets through the shared dynamic-call lookup and cleanup path.
- `8790f3a4`: generated-C runtime string-valued dynamic calls now dispatch to
  supported native builtin families after registered by-value frames.
- `1209d8cb`: generated-C finite known-string dynamic calls to supported
  native builtin families reuse shared builtin materialization and value-result
  paths.
- `61b609cd`: generated-C runtime string-valued dynamic calls dispatch through
  the registered by-value user-function frame table.
- `2f599360`: generated-C by-value user-function frames admit supported
  scalar, nullable, union, array, and mixed parameter/return type metadata
  through the shared runtime type-coercion helper.

## Candidate Work Not Counted

Primary semantic progress is counted only through pushed baseline `9fa9aa92`.
Current and lane-local candidates are not counted until primary integration
lands them with focused proof.

- Lane-local branch-merge cleanup readiness and grouped cleanup/terminal
  contract work.
- Lane-local array builtin candidates such as `array_unique()` and diagnostic
  dispatch for `array_keys()` / value-search families.
- Lane-local call/frame result-spread, method lookup vector, ternary result,
  and activation-binding guards.
- Lane-local reference-slot ABI candidates for regex and string operation
  families.
- Lane-local symbol/blocker transport and cleanup-contract work across LLVM
  and generated-C consumers.

## Done / In Progress / Not Done

Done:

- [x] Shared native value, array, string, comparison, truthiness, diagnostic,
  request-state, and selected cleanup/runtime ABI foundations.
- [x] Generated-C selected arrays, lvalues, references, request roots,
  `$GLOBALS`, lazy expressions, branches, loops, switch/goto, selected
  `try`/`finally`, and stdout diagnostics.
- [x] Generated-C bounded by-value direct, recursive, typed, variadic, dynamic
  user, dynamic builtin, and finite mixed user/builtin calls.
- [x] Generated-native `strpos()` and `substr_count()` through a shared
  PHP-shaped string-search ABI.
- [x] LLVM direct string-result and string-predicate builtin families through
  shared native ABIs for lowerable operands.
- [x] LLVM lowerable nested direct call-result operands for direct string/native
  value consumers across string-result, string-predicate, string-search,
  string-int, and selected `strlen()` paths.

In progress / candidates:

- [ ] Lane-local cleanup/readiness contracts that may support broader
  control-flow and unwind semantics.
- [ ] Lane-local array, string, diagnostic, call-frame, reference-slot, and
  symbol-cleanup candidates awaiting primary selection.
- [ ] Broader verification and composition gates beyond focused filters.

Not done:

- [ ] Full callable lookup across strings, arrays, objects, closures, methods,
  static methods, callbacks, and unsupported builtin families.
- [ ] General object construction, properties, methods, `$this`, visibility,
  static context, magic methods, and object lifecycle behavior.
- [ ] Full references/COW identity across calls, arrays, objects, globals,
  foreach, and control-flow joins.
- [ ] By-reference, named/unpacked argument, by-reference return, closure
  capture, and method-frame semantics.
- [ ] Full structured cleanup/unwind/finally/destructor/output-buffer/SAPI
  behavior.
- [ ] Exact diagnostic severity, ordering, suppression, handlers, spans,
  recovery, fatal behavior, and throw behavior.
- [ ] Direct assembly and LLVM parity for newer generated-C/runtime ABI
  consumers, plus broad end-to-end PHP program proof.

## Steering Read

Best next primary integrations should come from a hard semantic cliff rather
than another narrow adjacent builtin/backend slice: callable lookup, methods
and object/property execution, references/COW through calls or real control
flow, by-reference/named/unpacked frames, cleanup/unwind/finally, or
source-ordered diagnostics.

Resource note from this review: `/dev/shm` has about 7.7G free, above the 6G
floor but not enough for extra broad waves without owner-checked reclamation.
`/home` has about 304G free.
