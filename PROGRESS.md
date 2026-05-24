# PHP Native Compiler Progress

Updated: 2026-05-24 06:06 CEST
Evaluation marker: `20260524T040111Z`

Latest primary semantic/test baseline:
`44fd7cea codegen: lower function-frame finally blocks`

Latest integrated semantic baseline: `44fd7cea codegen: lower function-frame finally blocks`
Latest evaluator report: `20260524T040111Z`

Current primary git state at review:

- `master` and `origin/master` are synced at
  `d4de901b docs: update progress after function-frame finally`.
- Latest counted semantic commit remains
  `44fd7cea codegen: lower function-frame finally blocks`.
- The primary worktree has uncommitted candidate WIP in
  `compiler/src/codegen.rs`,
  `compiler/src/interpreter.rs`,
  `compiler/tests/native_function_call_boundary.rs`,
  `compiler/tests/native_link.rs`, and `runtime/src/lib.rs`.
  The codegen/tests/runtime part appears to target generated-C by-reference
  user-function frames. `compiler/src/interpreter.rs` appeared dirty during
  final verification after the initial freshness check and is treated as
  active implementation WIP. None of this is counted until committed, pushed,
  and proven.

These are candid engineering estimates toward generalized PHP semantics in the
native compiler. They are not test pass rates. Only primary-integrated, pushed
work counts; lane-local candidates, dirty WIP, parked diffs, and exact-shape
fixtures do not.

## Executive Read

Overall estimated progress: **59%** `[############--------]`

Executable PHP semantics: **56%** `[###########---------]`

The primary branch has made useful integrated progress since the last evaluator
marker: bounded generated-C variadic by-value frames landed, bounded
function-local `try`/`finally` landed inside supported by-value frames, and
LLVM consumed more shared direct string/native-value operand contracts. This
keeps broadening executable islands without pretending those islands equal full
PHP.

The current dirty by-reference frame candidate is in a valuable area, but it is
not integrated progress yet. The main remaining work is still central language
semantics: callable lookup, closures, methods, objects/properties, `$this`,
by-reference and named/unpacked argument binding, reference/COW identity,
source-ordered diagnostics, cleanup/unwind/finally/destructors, and backend
parity.

## Grand Roadmap Position

| Workstream | Estimate | Bar | Current read |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | **82%** | `[################----]` | Strong shared value, array, reference, symbol, request, comparison, truthiness, string, diagnostic, cleanup, request-root, call-frame type-coercion, and dynamic-call surfaces. Some remain scaffolding until consumed end to end. |
| Compiler/backend consumers | **75%** | `[###############-----]` | Generated-C has broad selected coverage. LLVM now consumes shared direct string-result, string-predicate, string-search, string-int, and selected `strlen()` nested operand ABIs. Direct assembly and many nested/backend consumers still stop at blockers. |
| Executable PHP semantics | **56%** | `[###########---------]` | Many focused linked programs run, including function-local bounded `try`/`finally` in by-value frames, but behavior is still selected islands rather than a complete PHP execution model. |
| Arrays, lvalues, references, COW | **58%** | `[############--------]` | Strong selected array/lvalue/reference paths. Full COW, arbitrary writable roots, by-reference calls, foreach parity, and object/reference joins remain open. |
| Symbols, globals, request state | **66%** | `[#############-------]` | Request roots and selected `$GLOBALS` paths are strong. Reconciliation across calls, requests, includes, aliases, and by-reference frames remains incomplete. |
| Calls, functions, frames | **51%** | `[##########----------]` | Bounded generated-C by-value fixed/default/variadic frames, typed params/returns, recursion guards, registered introspection, dynamic user calls, dynamic builtin calls, finite mixed user/builtin sets, and function-local bounded `try`/`finally` are integrated. Current by-reference frame work is candidate WIP, not counted. |
| Objects, properties, methods | **10%** | `[##------------------]` | Mostly lane-local/runtime candidate work. Primary lacks general compiled object construction, property access, method dispatch, `$this`, visibility, static context, and magic behavior. |
| Control flow, cleanup, diagnostics | **46%** | `[#########-----------]` | Bounded generated-C branches, loops, transfers, switch/goto, normal-flow `try`/`finally`, return-through-finally inside supported by-value frames, diagnostic-aware stdout formatting, and selected cleanup paths exist. Broad unwind, handlers, destructors, output buffers, and exact ordering remain open. |
| Broad integrated verification | **50%** | `[##########----------]` | Focused gates are strong, including function-frame `try`/`finally` source and linked execution. Cross-feature composition, end-to-end PHP programs, backend parity, and the unfiltered `native_runtime_abi` debt need broader proof. |

## Recent Primary-Integrated Work

- `44fd7cea`: generated-C by-value user-function frames now admit bounded
  no-throw `try`/`finally` bodies through the existing active-finalizer
  scheduler. Direct linked proof covers normal flow, return-through-finally,
  and nested finalizers inside reusable frame entries. `exit`, `break`,
  `continue`, `goto`, `throw`, and returns from active `finally` bodies remain
  blocked until real unwind and transfer-target semantics exist.
- `9fa9aa92`: generated-C by-value variadic user-function frames now pack
  surplus positional arguments through the shared native array/value ABI across
  direct, finite known-string dynamic, and runtime string-valued dispatch, with
  typed variadic elements routed through the existing call-frame type-coercion
  diagnostic path.
- `2633fe55`: LLVM direct string/native-value consumers now admit lowerable
  nested direct call-result operands across string-result, string-predicate,
  string-search, string-int, and selected `strlen()` paths. Dynamic calls,
  methods, constructors, closures, unknown calls, unsupported builtin families,
  and direct assembly remain on shared blockers.
- `ac875386`: LLVM direct string-predicate builtins now lower lowerable
  operands through `phpc_native_value_string_predicate_with_diagnostic(...)`
  for `str_starts_with()`, `str_ends_with()`, and `str_contains()`.
- `2cf2adda`: LLVM direct string-result builtins now lower lowerable operands
  through `phpc_native_value_string_result_operation_with_diagnostic(...)` for
  `strrev()`, `bin2hex()`, `str_rot13()`, ASCII case transforms, and
  shell-escape result operations.
- `59f3be42`, `8790f3a4`, `1209d8cb`, and `61b609cd`: generated-C dynamic
  call support expanded across registered by-value user-function frames and
  supported native builtin families.

## Primary-Integrated Vs Candidate Work

Primary-integrated capability:

- Bounded generated-C by-value fixed/default/typed/variadic user-function
  frames.
- Supported dynamic dispatch to registered by-value user frames and selected
  native builtin families.
- Function-local bounded no-throw `try`/`finally` inside supported by-value
  frames.
- LLVM consumption of selected shared string/native-value runtime contracts.
- Selected arrays, lvalues, references, request roots, `$GLOBALS`, lazy
  expressions, branches, loops, switch/goto, and stdout diagnostics.

Candidate work not counted:

- Current dirty primary by-reference user-function frame WIP, including
  reference-handle frame parameters, symbol-table reference paths, and
  by-reference tests.
- Lane-local foreach root rebinding, reference-slot operation families,
  request key-result accessors, branch-decision diagnostic cleanup, call-frame
  carrier cleanup, object/interface metadata contracts, and many array/string/
  diagnostic builtin candidates.
- Broad lane diffs that are conflict-heavy or metadata/preflight oriented
  unless a small selected contract lands in primary with executable proof.

## Done / In Progress / Not Done

Done:

- [x] Shared native value, array, string, comparison, truthiness, diagnostic,
  request-state, and selected cleanup/runtime ABI foundations.
- [x] Generated-C selected arrays, lvalues, references, request roots,
  `$GLOBALS`, lazy expressions, branches, loops, switch/goto, selected
  `try`/`finally`, and stdout diagnostics.
- [x] Generated-C bounded by-value direct, recursive, typed, variadic, dynamic
  user, dynamic builtin, finite mixed user/builtin calls, and bounded
  function-local `try`/`finally`.
- [x] Generated-native `strpos()` and `substr_count()` through a shared
  PHP-shaped string-search ABI.
- [x] LLVM direct string-result and string-predicate builtin families through
  shared native ABIs for lowerable operands.
- [x] LLVM lowerable nested direct call-result operands for direct string/native
  value consumers across string-result, string-predicate, string-search,
  string-int, and selected `strlen()` paths.

In progress / candidates:

- [ ] Primary dirty by-reference generated-C user-function frame candidate,
  pending integration proof and commit.
- [ ] Lane-local cleanup/readiness contracts that may support broader
  control-flow and unwind semantics.
- [ ] Lane-local array, string, diagnostic, call-frame, reference-slot, object
  metadata, and symbol-cleanup candidates awaiting primary selection.
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

The by-reference frame WIP is a legitimate hard semantic target, but it should
earn integration by proving alias-visible behavior rather than only opening
another narrow frame shape. Good proof would include direct variable and nested
path references, rejection of non-lvalue argument sources, unsupported dynamic
target handling, cleanup on failure, and linked execution showing writeback.

After that, the next primary direction should probably leave the call-frame
adjacency and attack a different cliff: callable array/object forms,
closures/methods/object execution, references/COW through real control-flow
joins, structured unwind/cleanup/finally, or source-ordered diagnostics.

Resource note from this review: `/dev/shm` is under severe pressure at about
527M free, 98% used. `/home` remains healthy at about 284G free. Broad new
dispatch or large gates should wait for owner-aware `/dev/shm` cleanup or for
active jobs to release space.
