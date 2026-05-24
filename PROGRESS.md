# PHP Native Compiler Progress

Updated: 2026-05-24 08:21 CEST
Evaluation marker: `20260524T045209Z`

Latest primary semantic/test baseline:
`84d33e6f codegen: thread globals through frame callers`

Latest integrated semantic baseline: `84d33e6f codegen: thread globals through frame callers`
Latest evaluator report: `20260524T045209Z`

Current primary git state at review:

- Current primary head before this progress update was
  `84d33e6f codegen: thread globals through frame callers`.
- `f0b22da2` is now the previous counted semantic commit.
- Generated-C user-function frames now propagate the caller root symbol table
  through direct wrapper frames that can reach an ordinary global-import
  callee, so wrappers do not create isolated function-local symbol tables for
  callee `global $name` imports.
- This is a compact call/frame plus request/global alias execution slice, not
  full global environment or callable parity: request superglobal imports,
  `$GLOBALS` self-import, dynamic-wrapper reachability, includes, variable
  variables, callable arrays, methods/closures/objects, and exact unset/global
  alias behavior remain blocked.

These are candid engineering estimates toward generalized PHP semantics in the
native compiler. They are not test pass rates. Only primary-integrated, pushed
work counts; lane-local candidates, dirty WIP, parked diffs, and exact-shape
fixtures do not.

## Progress Accounting Note

The older **88%** figure is retired and is not comparable with the current
numbers. It counted strong foundations, lane-local candidates, and selected
generated-C execution islands too much like broad PHP completion. The current
percentages use the stricter rubric above: pushed primary work toward
generalized, end-to-end PHP semantics. The move from 88% to the current
65% overall / 62% executable estimate was a measurement correction plus later
primary semantic integration, not a code rollback.

## Executive Read

Overall estimated progress: **65%** `[#############-------]`

Executable PHP semantics: **62%** `[############--------]`

The primary branch has made useful integrated progress since the last evaluator
marker: bounded generated-C variadic by-value frames landed, bounded
function-local `try`/`finally` landed inside supported by-value frames, LLVM
consumed more shared direct string/native-value operand contracts, generated-C
frames gained alias-visible by-reference parameters, and runtime string-valued
dynamic calls now bind those by-reference frame parameters for supported
ordinary symbol-table lvalue arguments. Direct-variable compound assignments
now use the shared native binary value-result ABI across local variables,
reference-backed variables, and active ordinary symbol-table variables.
Loop `break`/`continue` through supported active `finally` scopes now execute
the finalizers they leave, while inner-loop transfers that stay inside a try
body avoid premature finalizer execution.
Function-scope ordinary `global` imports inside generated-C frames now borrow
the caller-owned root symbol table, bind imported locals through shared native
reference handles, and preserve ordinary frame locals separately.
Runtime string-valued and finite mixed dynamic calls can now dispatch to those
global-import frames through the shared dynamic-call table while preserving the
same root symbol environment.
Direct wrapper frames that can reach global-import callees now receive and
forward the same caller root symbol table through the frame call graph.
Request-key result accessors remove generated backend dependence on the
concrete key-result return layout across request keyed/path consumers.
Generated-C also has a first shared runtime consumer for syntax-only callable
array forms, and native value addition now carries PHP array union through
generated-C array-offset and direct-variable compound assignments. This
broadens real executable calls/frames and value/lvalue behavior without
pretending the selected generated-C subset equals full PHP.

The main remaining work is still central language semantics: full callable
lookup, closures, methods, objects/properties, `$this`, typed/default/variadic
by-reference binding, named/unpacked arguments, by-reference returns,
reference/COW identity, source-ordered diagnostics, cleanup/unwind/finally/
destructors, and backend parity.

## Grand Roadmap Position

| Workstream | Estimate | Bar | Current read |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | **83%** | `[#################---]` | Strong shared value, array, reference, symbol, request, comparison, truthiness, string, diagnostic, cleanup, request-root, call-frame type-coercion, dynamic-call, and reference-clone surfaces. Some remain scaffolding until consumed end to end. |
| Compiler/backend consumers | **81%** | `[################----]` | Generated-C has broad selected coverage, including direct-variable compound assignment, function-scope ordinary `global` imports reached by direct, transitive wrapper, and runtime dynamic dispatch, `break`/`continue` through active `finally` scopes, and untyped by-reference frame parameters reached by runtime string-valued dispatch. LLVM now consumes shared direct string-result, string-predicate, string-search, string-int, and selected `strlen()` nested operand ABIs. Direct assembly and many nested/backend consumers still stop at blockers. |
| Executable PHP semantics | **62%** | `[############--------]` | Many focused linked programs run, including direct-variable compound assignment through local, reference-backed, and active symbol-table variables plus function-local bounded `try`/`finally`, loop transfer through finalizers, direct/transitive/runtime dynamic function-scope global imports, and alias-visible by-reference frame writes, but behavior is still selected islands rather than a complete PHP execution model. |
| Arrays, lvalues, references, COW | **63%** | `[#############-------]` | Strong selected array/lvalue/reference paths now include generated-C direct-variable compound assignment, function-scope global imports of scalar variables and array paths, by-reference call binding for direct variables and nested symbol-table paths, and PHP array-union value addition through generated-C array-offset and direct-variable `+=`. Full COW, arbitrary writable roots, foreach parity, object/reference joins, and broader frame/reference composition remain open. |
| Symbols, globals, request state | **72%** | `[##############------]` | Request roots and selected `$GLOBALS` paths are strong. Generated-C function-scope ordinary `global` imports now borrow the caller root symbol table and bind imported locals through shared references across direct, transitive wrapper, and runtime dynamic frame calls. Generated-C by-reference calls and direct-variable compound assignments also reuse symbol-table paths for ordinary variables and nested array slots. Reconciliation across requests, includes, variable variables, aliases, and broader reference frames remains incomplete. |
| Calls, functions, frames | **57%** | `[###########---------]` | Bounded generated-C by-value fixed/default/variadic frames, typed params/returns, recursion guards, registered introspection, syntax-only callable-array checks, dynamic user calls including global-import frames, dynamic builtin calls, finite mixed user/builtin sets, transitive root-symbol wrapper frames, function-local bounded `try`/`finally`, function-scope ordinary `global` imports, and untyped by-reference direct/compiler-known/runtime string-valued frame calls are integrated. |
| Objects, properties, methods | **10%** | `[##------------------]` | Mostly lane-local/runtime candidate work. Primary lacks general compiled object construction, property access, method dispatch, `$this`, visibility, static context, and magic behavior. |
| Control flow, cleanup, diagnostics | **48%** | `[##########----------]` | Bounded generated-C branches, loops, transfers, switch/goto, normal-flow `try`/`finally`, return-through-finally inside supported by-value frames, `break`/`continue` through active finalizers, diagnostic-aware stdout formatting, and selected cleanup paths exist. Broad unwind, handlers, destructors, output buffers, and exact ordering remain open. |
| Broad integrated verification | **54%** | `[###########---------]` | Focused gates are strong, including function-frame `try`/`finally`, loop-transfer-through-finally source/linked execution, direct/transitive/runtime dynamic function-scope global import source/linked execution, by-reference frame source/linked execution, and dynamic by-reference blocker proof. Cross-feature composition, end-to-end PHP programs, backend parity, and the unfiltered `native_runtime_abi` debt need broader proof. |

## Recent Primary-Integrated Work

- `84d33e6f`: generated-C user-function frame metadata now tracks whether a
  frame requires the caller root symbol table, not only whether its own body
  declares `global`. The requirement propagates through the direct registered
  user-function call graph, so wrapper frames that call global-import callees
  receive and forward `phpc_root_symbols` instead of materializing isolated
  function-local symbol tables. Linked proof covers nested direct wrappers and
  global array-path writes observed by the top-level caller. Runtime dynamic
  wrapper reachability, request superglobal imports, `$GLOBALS` self-import,
  includes, variable variables, exact `unset` alias behavior, LLVM, and direct
  assembly remain blocked.
- `f0b22da2`: generated-C runtime string-valued dynamic calls now dispatch to
  registered user-function frames that declare ordinary function-scope
  `global $name` imports. The dynamic lookup table no longer excludes those
  frames; when any candidate may need globals, the caller root symbol table is
  materialized at the dynamic dispatch boundary and threaded into matched
  global-import frame calls after branch-local argument materialization. Linked
  proof covers runtime string-valued global reads/writes, repeated
  caller-visible mutation, imported global array paths, finite user/user
  dispatch across global-import and ordinary frames, and finite mixed
  global-import/builtin dispatch. Request superglobal imports, `$GLOBALS`
  self-import, callable-array/object invocation, closures, methods, includes,
  variable variables, exact `unset` alias behavior, LLVM, and direct assembly
  remain blocked.
- `482e7c76`: generated-C user-function frames now lower ordinary
  function-scope `global $name` imports by passing a borrowed caller-owned root
  symbol table into frames that declare globals, binding imported locals to
  `phpc_NativeReferenceHandle` slots through
  `phpc_native_symbol_table_reference_for_path(...)`, and preserving ordinary
  frame locals outside the imported set. The root symbol table now tracks
  owned-versus-borrowed cleanup so callees do not free caller-owned globals.
  Linked proof covers direct global reads/writes, repeated calls observing
  caller-visible mutation, multiple imported globals, imported global array
  paths through symbol-table array lvalue owners, and unsupported request
  superglobal / `$GLOBALS` imports. Request/global self-import parity,
  `unset` alias behavior, includes, variable variables, LLVM, and direct
  assembly remain blocked.
- `e9ba63d0`: generated-C loop and switch transfer targets now carry the
  active `finally` depth from target creation. Supported `break`/`continue`
  statements run exactly the active finalizers whose scopes they exit before
  jumping to the selected loop/switch target, preserving inner-to-outer
  ordering. Linked proof covers `continue` and `break` through one active
  finalizer, nested finalizers before an exiting transfer, and an inner-loop
  break inside a try body that must not run the outer `finally` early. `exit`,
  `goto`, `throw`, returns from `finally`, exception catch dispatch,
  destructors/output buffers, reference/COW cleanup joins, LLVM, and direct
  assembly remain blocked.
- `a7ffdcd2`: generated-C direct-variable compound assignments now execute
  through a direct-variable RMW boundary. The compiler materializes the
  current variable value, computes `+=`, `-=`, `*=`, `.=` and the other
  compound binary families through `phpc_native_value_binary_result(...)`, and
  writes the computed value back according to owner semantics: cloned storage
  for ordinary locals, `phpc_native_reference_set_value(...)` for
  reference-backed variables, and symbol-path writes for active ordinary
  symbol-table variables. Linked proof covers statement and expression forms,
  scalar arithmetic, concatenation, array union on direct variables,
  by-reference frame parameters, and post-call active symbol-table writeback.
  Undefined-variable parity, request/global roots, object properties,
  ArrayAccess, increment/decrement, `??=`, full COW, LLVM, and direct assembly
  remain blocked.
- `c50866c5`: generated-C runtime string-valued dynamic calls now dispatch to
  registered user-function frames with untyped by-reference parameters. The
  dispatch table matches the callable name first, then materializes the
  matched branch's by-reference or by-value arguments, binding supported
  direct-variable and nested ordinary symbol-table lvalues through the shared
  `phpc_native_symbol_table_reference_for_path(...)` /
  `phpc_NativeReferenceHandle` path. The same branch-local materialization is
  used for runtime dynamic builtin candidates, preserving the shared dynamic
  call failure path for unknown callables, arity/default/variadic subset
  misses, and unsupported by-reference carriers. This does not add closure,
  method, object, callable-array invocation, request/global lvalue, typed/
  default/variadic by-reference, by-reference return, LLVM, or direct assembly
  parity.
- `fb27be7d`: runtime `Value::php_add()` and
  `phpc_native_value_binary_result(...)` now implement PHP array union for
  array-plus-array values. Union preserves left-hand keys, appends missing
  right-hand keys, and preserves right-side reference slots when inserted.
  Generated-C array-offset `+=` already computes through the shared native
  binary value-result ABI, so linked proof now executes direct and nested
  array-owner compound union without source-shape lowering. Array-plus-scalar,
  full direct-variable compound assignment, ArrayAccess/object/resource offset
  behavior, broad COW/reference identity, and exact diagnostics remain blocked.
- `993e96d2`: generated-C `is_callable(..., true)` now consumes shared runtime
  helpers for callable-array syntax over direct native array handles and owned
  native values. Runtime proof covers string, closure, object-receiver array,
  class-string array, normalized numeric-string keys, invalid target/method
  families, extra elements, scalars, and null handles. Source and linked proof
  covers direct array operands, native value operands produced by existing
  array-query value results, and invalid callable-array shapes. Syntax-only
  callable arrays do not imply callable lookup, method dispatch, object
  invocation, or actual callable-array invocation.
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
  frames plus untyped by-reference direct, compiler-known single-target, and
  runtime string-valued frame calls for supported ordinary lvalue arguments.
- Supported dynamic dispatch to registered user frames, including
  global-import frames, and selected native builtin families.
- Transitive direct wrapper frames that can reach ordinary global-import
  callees receive and forward the caller root symbol table.
- Direct-variable compound assignment through shared native binary value
  results for ordinary local variables, reference-backed variables, and active
  ordinary symbol-table variables.
- Function-scope ordinary `global` imports inside generated-C frames through
  borrowed caller root symbol tables and shared reference handles for imported
  scalar variables and array paths.
- Syntax-only callable-array checks in generated-C through shared runtime
  array/value callable-syntax helpers.
- Shared runtime reference-handle cloning and generated-C by-reference argument
  binding for ordinary symbol-table variables and nested array-slot paths.
- Shared request-state key-result buffer/status accessors consumed by LLVM ABI
  proof and generated-C request keyed/path/reference/global dispatch paths.
- PHP array-union value addition consumed by generated-C array-offset compound
  assignment through the shared native binary value-result ABI.
- Function-local bounded no-throw `try`/`finally` inside supported by-value
  frames.
- Loop `break`/`continue` through supported active `finally` scopes with
  finalizer-depth tracking for transfers that exit a try body.
- LLVM consumption of selected shared string/native-value runtime contracts.
- Selected arrays, lvalues, references, request roots, `$GLOBALS`, lazy
  expressions, branches, loops, switch/goto, and stdout diagnostics.

Candidate work not counted:

- Lane-local value-slot owner families beyond the integrated direct-variable
  compound RMW path, broad reference-slot operation families,
  object/magic/autoload prechecks, and cleanup resume capability models.
- Lane-local foreach root rebinding, branch-decision diagnostic cleanup,
  call-frame carrier cleanup, object/interface metadata contracts, and many
  array/string/diagnostic builtin candidates.
- Broad lane diffs that are conflict-heavy or metadata/preflight oriented
  unless a small selected contract lands in primary with executable proof.

## Done / In Progress / Not Done

Done:

- [x] Shared native value, array, string, comparison, truthiness, diagnostic,
  request-state, and selected cleanup/runtime ABI foundations.
- [x] Generated-C selected arrays, lvalues, references, request roots,
  `$GLOBALS`, lazy expressions, branches, loops, switch/goto, selected
  `try`/`finally`, loop transfer through active finalizers,
  direct-variable compound assignment, function-scope ordinary `global`
  imports, and stdout diagnostics.
- [x] Generated-C bounded by-value direct, recursive, typed, variadic, dynamic
  user, dynamic builtin, finite mixed user/builtin calls, runtime dynamic
  global-import frame calls, transitive global-import wrapper frames, untyped
  by-reference direct/compiler-known/runtime string-valued frame calls, and
  bounded function-local `try`/`finally`.
- [x] Generated-C syntax-only callable-array checks through shared runtime
  array/value syntax helpers.
- [x] Runtime/native value array union for `array + array`, consumed by
  generated-C array-offset `+=` through shared native value-result operations.
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

- [ ] Full callable lookup and invocation across strings, arrays, objects,
  closures, methods, static methods, callbacks, and unsupported builtin
  families.
- [ ] General object construction, properties, methods, `$this`, visibility,
  static context, magic methods, and object lifecycle behavior.
- [ ] Full references/COW identity across calls, arrays, objects, globals,
  foreach, and control-flow joins.
- [ ] Direct-variable increment/decrement, `??=`, object/property/ArrayAccess
  compound mutation, undefined-variable parity, and request/global root
  compound mutation.
- [ ] Typed/default/variadic by-reference arguments, request/global lvalue
  carriers for runtime by-reference dispatch, named/unpacked arguments,
  by-reference returns, closure capture, and method-frame semantics.
- [ ] Full structured cleanup/unwind/finally/destructor/output-buffer/SAPI
  behavior.
- [ ] Exact diagnostic severity, ordering, suppression, handlers, spans,
  recovery, fatal behavior, and throw behavior.
- [ ] Direct assembly and LLVM parity for newer generated-C/runtime ABI
  consumers, plus broad end-to-end PHP program proof.

## Steering Read

The transitive global-import wrapper slice was accepted because it fixes root
symbol environment propagation through the registered user-function call graph,
not because it recognizes one wrapper shape. The next primary direction should
attack a different cliff: actual callable-array/object invocation,
closures/methods/object execution, references/COW through calls or real
control-flow joins, broader structured unwind/cleanup, source-ordered
diagnostics, object/property execution, request/global alias behavior beyond
ordinary imports, or backend parity for an already integrated semantic family.

Resource note from this review: `/dev/shm` has recovered above the dispatcher
floor at about 5.2G used and 17G free. The largest visible target dirs are now
ordinary lane-local build targets under 600M. `/home` remains healthy at about
181G used on a 459G filesystem. Primary gates for the latest batches used
disk-backed `/tmp/phpc-primary-target-runtime-dynamic-byref` and
`/tmp/phpc-primary-target-direct-variable-rmw`; this cleanup batch used
`/tmp/phpc-primary-target-try-finally-loop-transfer`, and the latest dynamic
global-import batch used `/tmp/phpc-primary-target-dynamic-global-import`;
the latest transitive wrapper batch used
`/tmp/phpc-primary-target-transitive-global-import`. Keep checking resource
ownership before broad dispatch.
