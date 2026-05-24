# PHP Native Compiler Progress

Updated: 2026-05-24 06:25 CEST
Evaluation marker: `20260524T040111Z`

Latest primary semantic/test baseline:
`f7050310 runtime: expose request key result accessors`

Latest integrated semantic baseline: `f7050310 runtime: expose request key result accessors`
Latest evaluator report: `20260524T040111Z`

Current primary git state at review:

- `f7050310` is the latest counted semantic commit in this progress update.
- Request-state key results now expose buffer/status through runtime ABI
  accessors consumed by the LLVM ABI probe and generated-C dynamic request-key
  paths.
- This is a small ABI/layout encapsulation slice, not a broad new user-visible
  PHP feature; the overall estimate deliberately remains unchanged.

These are candid engineering estimates toward generalized PHP semantics in the
native compiler. They are not test pass rates. Only primary-integrated, pushed
work counts; lane-local candidates, dirty WIP, parked diffs, and exact-shape
fixtures do not.

## Executive Read

Overall estimated progress: **60%** `[############--------]`

Executable PHP semantics: **57%** `[###########---------]`

The primary branch has made useful integrated progress since the last evaluator
marker: bounded generated-C variadic by-value frames landed, bounded
function-local `try`/`finally` landed inside supported by-value frames, LLVM
consumed more shared direct string/native-value operand contracts, and
generated-C frames now have a first alias-visible by-reference parameter path.
The latest request-key result accessor slice removes generated backend
dependence on the concrete key-result return layout across request keyed/path
consumers. This broadens real executable calls/frames and tightens request ABI
encapsulation without pretending the selected generated-C subset equals full
PHP.

The main remaining work is still central language semantics: full callable
lookup, closures, methods, objects/properties, `$this`, typed/default/variadic
by-reference binding, named/unpacked arguments, by-reference returns,
reference/COW identity, source-ordered diagnostics, cleanup/unwind/finally/
destructors, and backend parity.

## Grand Roadmap Position

| Workstream | Estimate | Bar | Current read |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | **83%** | `[#################---]` | Strong shared value, array, reference, symbol, request, comparison, truthiness, string, diagnostic, cleanup, request-root, call-frame type-coercion, dynamic-call, and reference-clone surfaces. Some remain scaffolding until consumed end to end. |
| Compiler/backend consumers | **76%** | `[###############-----]` | Generated-C has broad selected coverage, including untyped by-reference frame parameters. LLVM now consumes shared direct string-result, string-predicate, string-search, string-int, and selected `strlen()` nested operand ABIs. Direct assembly and many nested/backend consumers still stop at blockers. |
| Executable PHP semantics | **57%** | `[###########---------]` | Many focused linked programs run, including function-local bounded `try`/`finally` and alias-visible by-reference writes in selected generated-C frames, but behavior is still selected islands rather than a complete PHP execution model. |
| Arrays, lvalues, references, COW | **60%** | `[############--------]` | Strong selected array/lvalue/reference paths now include generated-C by-reference call binding for direct variables and nested symbol-table paths. Full COW, arbitrary writable roots, foreach parity, object/reference joins, and broader frame/reference composition remain open. |
| Symbols, globals, request state | **67%** | `[#############-------]` | Request roots and selected `$GLOBALS` paths are strong. Generated-C by-reference calls now reuse symbol-table reference paths for ordinary variables and nested array slots. Reconciliation across calls, requests, includes, aliases, and broader reference frames remains incomplete. |
| Calls, functions, frames | **54%** | `[###########---------]` | Bounded generated-C by-value fixed/default/variadic frames, typed params/returns, recursion guards, registered introspection, dynamic user calls, dynamic builtin calls, finite mixed user/builtin sets, function-local bounded `try`/`finally`, and untyped by-reference direct/compiler-known single-target frame calls are integrated. |
| Objects, properties, methods | **10%** | `[##------------------]` | Mostly lane-local/runtime candidate work. Primary lacks general compiled object construction, property access, method dispatch, `$this`, visibility, static context, and magic behavior. |
| Control flow, cleanup, diagnostics | **46%** | `[#########-----------]` | Bounded generated-C branches, loops, transfers, switch/goto, normal-flow `try`/`finally`, return-through-finally inside supported by-value frames, diagnostic-aware stdout formatting, and selected cleanup paths exist. Broad unwind, handlers, destructors, output buffers, and exact ordering remain open. |
| Broad integrated verification | **51%** | `[##########----------]` | Focused gates are strong, including function-frame `try`/`finally`, by-reference frame source/linked execution, and dynamic by-reference blocker proof. Cross-feature composition, end-to-end PHP programs, backend parity, and the unfiltered `native_runtime_abi` debt need broader proof. |

## Recent Primary-Integrated Work

- `f7050310`: request-state key results now expose their owned byte buffer and
  status through `phpc_native_request_state_key_result_buffer(...)` and
  `phpc_native_request_state_key_result_status(...)`. The LLVM runtime ABI
  probe no longer extracts the concrete `%phpc.NativeRequestStateKeyResult`
  fields, and generated-C request keyed/path/reference/global-dispatch
  consumers materialize key buffer/status locals through the same accessors
  before calling the existing request-state operation ABIs. Focused proof
  covers scalar/value key coercion, generated-C source paths across keyed
  storage, nested paths, request-root dispatch, and linked request/global
  executables.
- `241e1222`: generated-C user-function frames now accept untyped
  by-reference parameters for direct calls and compiler-known single-target
  dynamic calls. The compiler passes `phpc_NativeReferenceHandle` frame
  arguments, clones reference handles through the shared
  `phpc_native_reference_clone(...)` ABI, binds writable direct-variable and
  nested array-slot arguments through existing symbol-table reference paths,
  and keeps runtime string-valued by-reference dispatch on a shared dynamic-call
  failure path. Linked proof covers direct variable writeback, known dynamic
  call writeback, nested array slot mutation, multi-reference swap behavior,
  non-lvalue rejection, unsupported by-reference declarations, and runtime
  dynamic by-reference blockers.
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
  frames plus untyped by-reference direct and compiler-known single-target
  frame calls.
- Supported dynamic dispatch to registered by-value user frames and selected
  native builtin families.
- Shared runtime reference-handle cloning and generated-C by-reference argument
  binding for ordinary symbol-table variables and nested array-slot paths.
- Shared request-state key-result buffer/status accessors consumed by LLVM ABI
  proof and generated-C request keyed/path/reference/global dispatch paths.
- Function-local bounded no-throw `try`/`finally` inside supported by-value
  frames.
- LLVM consumption of selected shared string/native-value runtime contracts.
- Selected arrays, lvalues, references, request roots, `$GLOBALS`, lazy
  expressions, branches, loops, switch/goto, and stdout diagnostics.

Candidate work not counted:

- Lane-local foreach root rebinding, reference-slot operation families,
  branch-decision diagnostic cleanup, call-frame carrier cleanup,
  object/interface metadata contracts, and many array/string/diagnostic builtin
  candidates.
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
  user, dynamic builtin, finite mixed user/builtin calls, untyped
  by-reference direct/compiler-known single-target frame calls, and bounded
  function-local `try`/`finally`.
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
- [ ] Typed/default/variadic by-reference arguments, runtime string-valued
  by-reference dispatch, named/unpacked arguments, by-reference returns,
  closure capture, and method-frame semantics.
- [ ] Full structured cleanup/unwind/finally/destructor/output-buffer/SAPI
  behavior.
- [ ] Exact diagnostic severity, ordering, suppression, handlers, spans,
  recovery, fatal behavior, and throw behavior.
- [ ] Direct assembly and LLVM parity for newer generated-C/runtime ABI
  consumers, plus broad end-to-end PHP program proof.

## Steering Read

The request-key accessor slice was accepted because it removes a concrete
backend return-layout dependency from already executable request-key paths; it
does not change the hard steering picture. The next primary direction should
attack a different cliff: callable array/object forms, closures/methods/object
execution, references/COW through real control-flow joins, structured
unwind/cleanup/finally, or source-ordered diagnostics.

Resource note from this review: `/dev/shm` has recovered to about 16G free
and `/home` remains healthy. Primary gates for this batch used disk-backed
`/tmp/phpc-primary-target`; keep checking resource ownership before broad
dispatch.
