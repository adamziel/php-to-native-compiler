# PHP Native Compiler Progress

Updated: 2026-05-26 02:41 CEST
Evaluation marker: `20260526T003949Z`

Accounting rule: only generalized, tested, committed, and pushed primary work
counts as integrated capability. Dirty WIP, candidate worktrees, lane-local
claims, review-only work, failed prep proofs, probe-only commits, architecture
notes, and dashboard-only commits are excluded.

## Executive Read

Overall estimated progress: **95%** `[###################-]`

Executable PHP semantics: **95%** `[###################-]`

Primary was clean and aligned with `origin/master` at
`64f97660 docs: account reference-binding operand-list requirements` before
this `PROGRESS.md` edit. The latest source capability remains `b4b21937`;
`64f97660` is docs/accounting only.

Latest primary-integrated source capability baseline:
`b4b21937 native: add reference-binding operand-list blocker requirements`.

Current evaluator read (`20260526T003949Z`): no primary source capability was
added after `b4b21937`. Request/global frame direct handoff now has formal and
shadow go-for-primary-review evidence, and trait effective-method metadata has
current-primary reconciliation evidence, but both remain lane-local candidate
work until integrated and pushed. Dynamic callable compiler consumption still
conflicts with the request/global frame-environment change and should be
rebased after that semantic boundary lands. Focused current-primary smoke after
accounting is clean: the assigned focused runtime/compiler/direct-call gates
and `cargo check -p php_runtime -p phpc` passed. Estimates remain flat.

`b4b21937` extends the generalized diagnostic operation and operand-list
requirement blocker ABI to reference-binding surfaces. Reference-assignment
target/source blockers and by-reference array item blockers now route through
the shared operation-list requirement vocabulary, with runtime and compiler
tests covering target binding, source binding, and array-item binding
requirements across multiple expression/reference surfaces. This is
diagnostic/blocker infrastructure only: it does not implement executable PHP
reference binding, full alias/COW behavior, writeback, cleanup/unwind ordering,
or source-ordered diagnostics. Overall, executable, and workstream estimates
remain flat.

Non-repeat guard: future reference-binding work must extend or consume the
shared diagnostic operation-list and operand requirement ABI from `a544daa8`
and `b4b21937`, not add source-shape blockers, generated-output recognizers,
or fixture-specific reference-binding branches.

`a544daa8` adds a generalized diagnostic operation and operand-list
requirement blocker boundary in the runtime and compiler. Call-argument and
lvalue operand-list blockers now route through the shared runtime-backed
requirement list vocabulary instead of source-shape call-result scans, with
tests covering multiple operation families, requirement tags, expression
surfaces, ownership paths, and call-expression families. This is diagnostic
blocker infrastructure only: it does not implement the blocked cleanup,
ownership, frame handoff, callable dispatch, reference binding, or executable
operand-evaluation semantics. Overall, executable, and workstream estimates
remain flat.

Non-repeat guard: diagnostic work must layer on the generic operand-list
requirement boundary from `a544daa8` instead of adding source-shape blockers.

`dae1b44c` repairs the runtime callable-value dispatch foundation already
counted at `5abf8525`: inherited callable lookup now preserves the requested
called scope while invoking the declaring descriptor, protected visibility uses
same-hierarchy access, and descriptor-backed closure dispatch shapes
value/reference argument slots from descriptor parameter modes. This is a
narrow runtime semantics repair over the shared callable table and
call-arguments/frame/result ABI, not new compiler dynamic-call consumption, not
constructor execution, not broader callable lookup parity, and not a
percentage-moving semantic expansion. Overall, executable, and workstream
estimates remain flat.

Non-repeat guard: compiler dynamic callable consumers must consume the repaired
runtime lookup/called-scope/visibility/descriptor-argument boundary, not
source-shape callable branches.

`5abf8525` adds runtime callable-value dispatch over the existing callable
table, call-arguments, call-frame, and call-result ABI. The runtime now looks
up and invokes string and binary-string function names, callable arrays,
descriptor-backed closures, inherited table-backed methods, bound object
receivers, and object `__invoke` through shared value/reference/discard result
entrypoints; executable unit coverage exercises lookup, value-result
invocation, caller-scope visibility, bound receivers, and diagnostics. This is
runtime ABI progress, not a
compiler dynamic-call consumer: `Class::method` strings, namespace fallback,
autoload, magic calls, named/spread argument support, by-reference variadic
breadth, constructor `new` execution, request/global frame separation, and full
cleanup/unwind parity remain open. Overall and executable estimates stay flat
at 95%; calls/functions/frames advances one point because the previous
dashboard explicitly named callable-value dispatch beyond direct user-function
lookup/registration as the next missing callable runtime foundation, and this
commit closes that runtime portion with generalized tests across callable
value families. `dae1b44c` repairs the inherited-dispatch called-scope,
protected same-hierarchy visibility, and descriptor-aware closure argument-mode
risks without changing that accounting.

## Grand Roadmap Position

| Workstream | Estimate | Bar | Current read |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | **99%** | `[####################]` | Strong selected-path value, byte-string, string-array, array, diagnostic operation/operand-list blocker, reference-binding operand-list blocker, reference, symbol, call-frame, object, comparison, conversion, numeric-unary, request-state, closure-result, reference-source, callable table/arguments/result/frame, runtime callable-value dispatch, and declared-class allocation cleanup-risk metadata surfaces. Remaining runtime gaps include broader PHP callable lookup parity, namespace fallback, autoload, magic calls, constructors, request/global frame separation, and cleanup/unwind parity. |
| Compiler/backend consumers | **99%** | `[####################]` | Generated C has the freshest executable semantics; LLVM and direct assembly still lag several recent semantic packets. Recent compiler-side work routes direct generated-C user-function calls through the runtime callable ABI, improved try/catch/finally body call preflight plus allocatable class cleanup-risk classification, and call-argument/lvalue/reference-binding diagnostics through the shared operand-list requirement blocker boundary without adding dynamic callable dispatch, exception execution, destructor execution, or reference-binding execution. |
| Executable PHP semantics | **95%** | `[###################-]` | Primary has many selected executable islands, including reference-backed by-value closure capture materialization, closure value/reference returns, and reference-source append/lvalue extraction. |
| Strings and byte semantics | **62%** | `[############--------]` | Byte-backed values and byte-preserving `explode()` / `str_split()` slots are integrated. Binary source bytes, byte-exact interpreter/session/debug output, `mb_str_split()`, request/global byte keys, and exact diagnostics remain open. |
| Arrays, lvalues, references, COW | **82%** | `[################----]` | Selected reference-source/lvalue extraction, closure capture from reference-backed slots, and reference-binding diagnostic operand requirements are integrated. Full executable reference binding, COW, arbitrary alias roots, foreach, alias composition, static/magic/non-public properties, ArrayAccess, and broad writeback remain incomplete. |
| Symbols, globals, request state | **74%** | `[###############-----]` | Selected globals, root-symbol, active symbol-table reference consumers, request-key blockers, and append-shaped symbol reference-source materialization exist. `$GLOBALS` self-cells, request/global alias parity, request writeback, includes, variable variables, and exact unset/global behavior remain incomplete. |
| Calls, functions, frames | **91%** | `[##################--]` | Descriptor closures, selected captures, selected by-reference parameters, callable/function-table surfaces, method-frame surfaces, descriptor closure value/reference returns, reference-backed by-value captures, a runtime callable table/arguments/result/frame ABI, runtime callable-value dispatch across string/binary functions, callable arrays, descriptor closures, inherited methods, bound receivers, and object `__invoke`, direct generated-C user-function consumers across zero/fixed/default/variadic calls and by-reference argument transport, shared symbol-environment constructor blockers, and try/catch/finally body call-boundary preflight routing are integrated. Compiler dynamic callable-value consumption, `Class::method` strings, namespace fallback, autoload, magic calls, named/spread argument breadth, by-reference variadic breadth, non-descriptor closures, constructors, request/global frame separation, and cleanup/unwind parity remain open. |
| Objects, properties, methods | **53%** | `[###########---------]` | Public object-property reference-source extraction, object-property reference-slot mutation, and declared-class allocation cleanup-risk metadata exist for selected paths. Full visibility, magic, dynamic/static/typed properties, destructor execution and ordering, references/COW, and `ArrayAccess` execution remain open. |
| Control flow, cleanup, diagnostics | **51%** | `[##########----------]` | Selected branches, loops, transfers, finalizers, output buffers, diagnostics, truthiness, conversion consumers, try/catch/finally body call-boundary preflight diagnostics, generic diagnostic operand-list requirement blockers, and reference-binding operand-list requirement blockers exist. Broad unwind/finally/destructor/shutdown execution, executable reference binding, cleanup ownership, and exact source ordering remain open. Current broad diagnostic-lane work beyond the integrated operand-list boundary is not counted. |
| Broad integrated verification | **92%** | `[##################--]` | Recent focused gates are strong and nonzero, including runtime callable-value dispatch unit coverage, direct user-function callable ABI consumer execution, try-body call-boundary preflight, allocatable destructor-risk metadata gates, diagnostic operand-list blocker boundary tests, and reference-binding operand-list requirement tests. The full `native_runtime_abi` suite still has known current-primary failures, and broad gates remain constrained by lane extraction cost, stale lane expectations, high swap, and backend parity gaps. |

## Recent Primary-Integrated Work

- `b4b21937`: extended the shared diagnostic operation/operand-list
  requirement ABI with reference-binding operand-list requirements. Runtime
  diagnostics now name reference target, source, and array-item binding
  requirements, and compiler blocker discovery routes reference assignment
  target/source operands plus by-reference array items through that shared
  requirement vocabulary instead of source-shape recognizers. Focused runtime
  and compiler tests cover multiple binding requirement tags and representative
  assignment/source/array item surfaces. This is diagnostic/blocker
  infrastructure only; executable reference binding, full alias/COW behavior,
  writeback, cleanup/unwind ordering, and exact source-ordered diagnostics
  remain unimplemented. Future reference-binding work must extend or consume
  the shared operation-list and operand requirement ABI rather than adding
  source-shape or generated-output recognizers. Estimates remain flat.
- `a544daa8`: added a generic diagnostic operation and operand-list
  requirement blocker boundary across runtime and compiler code. Call-argument
  and lvalue operand-list blockers now consume the shared runtime-backed
  requirement list vocabulary instead of a direct call-result source-shape
  scan. Focused tests cover multiple operation families, requirement tags,
  expression surfaces, ownership paths, and call-expression families. This is
  a diagnostic blocker boundary only; cleanup/ownership execution,
  reference-binding semantics, frame handoff, callable dispatch expansion, and
  exact diagnostic ordering remain unimplemented. Future reference-binding and
  diagnostic work must layer on this generic operand-list requirement boundary
  rather than adding source-shape blockers. Estimates remain flat.
- `dae1b44c`: repaired callable-value runtime dispatch semantics over the
  already integrated callable table and call-arguments/frame/result ABI.
  Inherited callable lookup now invokes the declaring descriptor while carrying
  the requested called scope, protected visibility accepts same-hierarchy
  access, and descriptor-backed closure dispatch consults descriptor parameter
  modes before shaping value/reference argument slots. Compiler dynamic
  callable consumers must consume this repaired runtime boundary rather than
  adding source-shape callable branches. Estimates remain flat because this is
  a narrow repair to the already-counted runtime callable-value dispatch
  foundation, not a new compiler consumer.
- `501a4fb8`: centralized generated-C `phpc_NativeReferenceHandle` typedef
  gating behind `uses_native_reference_handle_type()`. Future native-link
  helpers that can emit `phpc_NativeReferenceHandle` declarations must consume
  that shared predicate rather than extending inline typedef predicates. This
  repairs declaration-before-use ordering for current reference-handle helper
  families without adding request/global frame execution, dynamic callable
  compiler consumption, or exact-shape production lowering.
- `5abf8525`: added runtime callable-value dispatch over the callable table
  and call-arguments/frame/result ABI. Integrated exactly `runtime/src/lib.rs`;
  focused runtime unit tests cover string and binary-string function names,
  callable arrays, descriptor-backed closures, inherited table-backed method
  lookup, bound object receivers, object `__invoke`, caller-scope visibility,
  shared value-result invocation, and diagnostic failures. Compiler
  dynamic-call consumption, `Class::method` strings, namespace fallback,
  autoload, magic calls, named/spread argument support, by-reference variadic
  breadth, constructor `new` execution, request/global frame separation, and
  full cleanup/unwind parity remain unclaimed.
- `f0cc17c1`: routed direct generated-C user-function calls through the
  runtime callable table, lookup, call arguments, call frame, and call result
  ABI. Integrated exactly `compiler/src/codegen.rs` and
  `compiler/tests/native_function_call_boundary.rs`; focused executable tests
  covered zero/fixed/default/variadic calls plus by-reference argument
  transport, focused `native_call` unit tests, `cargo check -p phpc`, fmt, and
  diff checks passed.
- `b400a23d`: added declared-class allocation metadata for
  destructor-observable cleanup risk, consumed by allocatable registration and
  dynamic constructor allocation checks across finite and unknown class-name
  paths. Integrated exactly `compiler/src/codegen.rs` and
  `compiler/tests/native_function_call_boundary.rs`; focused nonzero tests,
  `cargo check -p phpc`, fmt, and diff checks passed.
- `6f7d550d`: routed try/catch/finally body call operations through the shared
  `NativeCallOperation` / `NativeCallBlocker` preflight diagnostic boundary
  before generic try rejection. Integrated exactly `compiler/src/codegen.rs`
  and `compiler/tests/native_function_call_boundary.rs`; focused nonzero
  tests, `cargo check -p phpc`, fmt, and diff checks passed.
- `ea0c7675`: routed dynamic constructor class-name operands that are PHP
  symbol environments through the shared
  `NativeCallBlocker::GlobalFrameSeparation` constructor operation before
  class lookup, destructor-risk planning, argument cleanup, or generated-C
  object materialization. Integrated exactly `compiler/src/codegen.rs` and
  `compiler/tests/native_function_call_boundary.rs`; focused nonzero tests,
  `cargo check -p phpc`, fmt, and diff checks passed.
- `e32f5735`: added a runtime callable table and shared
  call arguments/result/frame ABI for function, method, and constructor
  callable kinds, including value/reference/failure ownership operations,
  called-scope propagation, and public/protected/private caller-scope
  visibility checks. Integrated exactly `runtime/src/lib.rs`; focused nonzero
  php_runtime gates, `cargo check -p php_runtime -p phpc`, fmt, and diff
  checks passed.
- `90e53401`: materialized by-value descriptor closure captures from
  reference-backed locals/frame slots through a shared helper consumed by both
  descriptor capture storage families. Integrated exactly
  `compiler/src/codegen.rs` and
  `compiler/tests/native_function_call_boundary.rs`; focused nonzero gates,
  `cargo check`, fmt, and diff checks passed.
- `7aa162ca`: reference-source append/lvalue extraction for selected symbol,
  native reference local, public object-property, object-property array-path,
  and append paths.
- `ae93da8c`: closure value/reference return result ABI for descriptor closure
  value returns, reference returns, value-consumer reference cloning, and
  reference-assignment binding.
- `22f56b67`: request-key operation selector cleanup for existing request-key,
  path, and bag mutation consumers.
- `2cd78ade`: conversion-result consumer helper consolidation for current LLVM
  scalar offset-read and numeric-unary source-result consumers.
- `b13c85c6`: unary negation source/result ABI across runtime, LLVM, generated
  C, and focused linked execution.
- `5307990c`: string-array operation slots for byte-preserving `explode()` and
  `str_split()`.
- `1c369d0f`: byte-backed PHP string value boundary.

## Active Roadmap Items

Primary-integrated capability and lane-local candidate work are separated here.

| Item | Toward Primary Integration | Toward Full Feature | Status |
| --- | ---: | ---: | --- |
| Allocatable destructor-observable cleanup-risk metadata | **100%** `[####################]` | **24%** `[#####---------------]` | Integrated at `b400a23d`. Declared-class allocation metadata now tracks destructor-observable cleanup risk and is consumed by allocatable registration plus static/dynamic constructor allocation checks. Destructor execution, object lifetime cleanup, shutdown/finally ordering, trait-composed destructor metadata, runtime class lookup, and broad object semantics remain unimplemented. |
| Try/catch/finally body call-boundary preflight | **100%** `[####################]` | **18%** `[####----------------]` | Integrated at `6f7d550d`. Try body, catch body, and finally body traversal now routes contained call families through the shared call-boundary diagnostic contract before generic unsupported-try rejection. Exception execution, catch matching, unwinding, finally ordering, and compiler dynamic callable-value consumption remain unimplemented. |
| Dynamic constructor symbol-environment blockers | **100%** `[####################]` | **22%** `[####----------------]` | Integrated at `ea0c7675`. Dynamic constructor class-name operands from `$GLOBALS` and request superglobals now route to `NativeCallBlocker::GlobalFrameSeparation` before constructor lookup, destructor-risk classification, argument cleanup, or generated-C object materialization. Request/global frame separation execution remains unimplemented. |
| Diagnostic operand-list blocker boundary | **100%** `[####################]` | **14%** `[###-----------------]` | Integrated at `a544daa8`. Runtime and compiler diagnostics now share a generic operation/operand-list requirement boundary for call-argument and lvalue blocker families, with runtime ownership/free coverage and compiler boundary tests across representative shapes. This is not cleanup, ownership, frame handoff, reference-binding, callable-dispatch, or exact diagnostic-order execution; later reference-binding and diagnostic work must consume the generic operand-list requirement boundary rather than adding source-shape blockers. |
| Reference-binding operand-list blocker requirements | **100%** `[####################]` | **15%** `[###-----------------]` | Integrated at `b4b21937`. Reference-assignment target/source blockers and by-reference array item blockers now consume the shared diagnostic operation-list and operand requirement ABI with runtime/compiler coverage across multiple binding requirement tags and representative operand families. This is not executable PHP reference binding, full alias/COW identity, writeback, cleanup/unwind ordering, or source-ordered diagnostics; future reference-binding work must extend or consume this shared ABI rather than adding source-shape or generated-output recognizers. |
| Runtime callable ABI, callable-value dispatch, and direct user-function consumers | **100%** `[####################]` | **58%** `[############--------]` | Runtime callable table, arguments/result/frame ownership, function/method/constructor invocation callbacks, called-scope propagation, and public/protected/private visibility checks are integrated at `e32f5735`. Direct generated-C user-function registration, lookup, argument transport, frame entry, and value-result consumption are integrated at `f0cc17c1` across zero/fixed/default/variadic calls and by-reference argument transport. Runtime callable-value dispatch is integrated at `5abf8525` for string/binary function names, callable arrays, descriptor-backed closures, inherited table-backed methods, bound object receivers, and object `__invoke` over the shared callable table and call-arguments/frame/result ABI, with `dae1b44c` repairing inherited called-scope carriage, protected same-hierarchy visibility, and descriptor-aware closure argument modes. Compiler dynamic callable-value consumers, `Class::method` strings, namespace fallback, autoload, magic calls, named/spread argument breadth, by-reference variadic breadth, constructor `new` execution, return references, request/global frame separation, and cleanup/unwind parity remain unimplemented. |
| Reference-backed by-value closure captures | **100%** `[####################]` | **48%** `[##########----------]` | Integrated at `90e53401`. Covers descriptor closure value captures from ordinary locals, native reference handles, and active symbol-table storage across direct function, static method, receiver method, and closure factory frames. Non-static implicit `$this`, secondary-alias writeback, and broad callable/object semantics remain open. |
| Reference-source append/lvalue extraction | **100%** `[####################]` | **52%** `[##########----------]` | Integrated at `7aa162ca`. Covers selected symbol, native reference local, public object-property, object-property array-path, append, reference assignment, and by-reference call consumers. Static/magic/non-public properties, ArrayAccess, arbitrary alias roots, and full references/COW remain open. |
| Closure value/reference return ABI | **100%** `[####################]` | **50%** `[##########----------]` | Integrated at `ae93da8c`. Runtime closure invocation has a shared value/reference/diagnostic/status result contract for descriptor closures. Broader function/method/static/constructor reference returns remain open. |
| Request/global frame direct handoff (candidate, not counted) | **90%** `[##################--]` | **34%** `[#######-------------]` | Lane-local candidate has formal `go-for-primary-integrator` and shadow `shadow-go-for-primary-review` evidence, applies to current primary, and would replace the root-symbol-only frame flag with `CFrameEnvironmentRequirement { root_symbols, request_state }` for direct user-function frame handoff. It is not primary progress until integrated; dynamic request-state callable handoff, closure environments, `$GLOBALS` parity, request aliasing/writeback, includes, variable variables, and exact unset/global behavior remain open. |
| Trait effective-method metadata (candidate, not counted) | **85%** `[#################---]` | **18%** `[####----------------]` | Lane-local candidate has current-primary reconciliation and a passing focused `trait_semantics` gate. It adds shared trait composition metadata and destructor-risk classification, but remains metadata-only: trait method execution, trait properties/constants, destructor execution, object lifetime cleanup, shutdown/finally ordering, and broad object semantics are still missing. |
| Dynamic callable compiler consumer (candidate, not counted) | **55%** `[###########---------]` | **42%** `[########------------]` | Candidate applies to current primary but conflicts with request/global frame-environment work and still depends on the old `requires_root_symbols` model. Rebase after `CFrameEnvironmentRequirement` lands, then either seed request-state for all supported dynamic callable user-function targets through the shared boundary or keep a centralized unsupported blocker. |
| Assignment/RMW lvalue operand-list requirements (active prep, not counted) | **45%** `[#########-----------]` | **12%** `[##------------------]` | Narrow prep lanes are active for assignment-lvalue and RMW-lvalue diagnostic operand-list requirements over the generic operation-list ABI. They are useful diagnostic prerequisites but not executable semantics; serialize them around ABI tag allocation and do not duplicate broad-lane extraction. |
| Diagnostic result callable/RMW/control scanner contracts | **0%** `[--------------------]` | **46%** `[#########-----------]` | `impl-native-error-diagnostic-semantics` continues producing broad lane-local contracts and scanner work beyond the integrated `a544daa8` operand-list blocker boundary. It remains dirty evidence, not primary progress; one visible status section is chronologically suspect relative to the evaluator clock. |
| Broader lvalue/reference-slot materializer | **42%** `[########------------]` | **45%** `[#########-----------]` | Improved by recent reference-source and closure-capture packets. Non-variable expression families, static/magic/non-public properties, ArrayAccess, arbitrary alias roots, and writeback remain open. |
| Object/resource source materialization | **25%** `[#####---------------]` | **30%** `[######--------------]` | Still a recurring blocker for generic conversion and offset/source consumers. Needs a general value reconstruction/materialization boundary. |
| Broad lane extraction backlog | **35%** `[#######-------------]` | **36%** `[#######-------------]` | Broad dirty lanes remain useful evidence repositories, not integration units. Several are parked, stale, or only current as lane-local artifacts. |

## Done / In Progress / Not Done

Primary-integrated executable or executable-prerequisite capability:

- [x] Shared closure value-capture materialization for reference-backed locals,
  native reference handles, and active symbol-table storage in descriptor
  closure capture families.
- [x] Shared reference-source/lvalue materialization for selected symbol paths,
  native local reference variables, direct/dynamic public object-property
  sources, object-property array paths, append paths, by-reference call
  argument extraction, and supported reference-assignment consumers.
- [x] Descriptor-backed closures, selected captures, selected by-reference
  parameters, and selected callable-array/object invocation.
- [x] Shared closure invocation result ABI for descriptor closure value returns,
  reference returns, value-consumer reference cloning, and reference-assignment
  binding.
- [x] Bounded `preg_replace_callback()` string-callback execution over supported
  slash-delimited patterns.
- [x] Object-property assignment/unset mutation for covered reference-backed
  operands through generated-C/native-link shared slot boundaries.
- [x] Shared offset-read source-result ABI for scalar/resource warning
  continuations, arrays, byte strings, references, and object-property
  offset-source composition.
- [x] Shared array-key value/reference-slot ABI for generated-native
  reference-backed variable operands and active symbol-table variable
  references.
- [x] Shared request-backed ordinary array-key/RMW blocker classification for
  selected LLVM and generated-C consumers.
- [x] Shared request-state operation selector for existing request-key/path/bag
  mutation consumers.
- [x] Byte-backed PHP string value representation and native pointer-plus-length
  materialization for arbitrary PHP string bytes.
- [x] Shared byte-preserving `explode()` and `str_split()` string-array slots.
- [x] Shared non-local assignment/unset owner-family blockers for object/static
  property assignment and unset paths.
- [x] Shared numeric-unary source/result ABI for covered unary negation.
- [x] Shared LLVM conversion-result consumer helper for current scalar
  offset-read and numeric-unary source-result paths.
- [x] Runtime callable table plus shared call arguments/result/frame ABI for
  function, method, and constructor callable kinds, including value/reference/
  failure ownership, called-scope propagation, and caller-scope visibility.
- [x] Runtime callable-value dispatch for string and binary-string function
  names, callable arrays, descriptor-backed closures, inherited table-backed
  method lookup, bound object receivers, and object `__invoke` through the
  callable table and call-arguments/frame/result ABI.
- [x] Direct generated-C user-function calls consume the runtime callable
  table/arguments/frame/result ABI across zero/fixed/default/variadic calls
  and by-reference argument transport.
- [x] Shared dynamic constructor symbol-environment blocker for `$GLOBALS` and
  request-superglobal class-name operands, routed through
  `NativeCallBlocker::GlobalFrameSeparation` before constructor execution
  planning.
- [x] Shared try/catch/finally body call-boundary preflight routing through
  `NativeCallOperation` / `NativeCallBlocker` before unsupported-try
  diagnostics.
- [x] Shared declared-class allocation cleanup-risk metadata for
  destructor-observable allocatable and constructor-allocation blockers.
- [x] Shared diagnostic operation/operand-list requirement blocker boundary for
  call-argument and lvalue diagnostic families.
- [x] Shared reference-binding operand-list requirement blockers for
  reference-assignment target/source operands and by-reference array item
  operands.

In progress but lane-local or not yet executable primary support:

- [ ] Dynamic callable compiler consumers beyond direct user functions must now
  use the integrated runtime ABI: compiler consumption of callable-value
  dispatch, method/static/constructor consumers, generated-C reference/discard
  consumers, `Class::method` strings, namespace fallback, autoload, magic
  calls, and named/spread breadth remain unintegrated.
- [ ] `impl-native-call-semantics` has broad dirty evidence for call ordering,
  callable-value dispatch, source-call ordering, and object/call blockers.
  The try-body call-routing and allocatable destructor-risk packets now count
  only through their pushed primary commits; remaining lane-local work still
  needs fresh current-primary review and integration.
- [ ] `impl-native-error-diagnostic-semantics` has broad dirty diagnostic
  result/scanner contracts beyond the integrated operand-list blocker
  boundary, except for the integrated `b4b21937` reference-binding
  operand-list requirement packet. Useful evidence, but not primary progress
  and not a substitute for executable PHP semantics.
- [ ] Broader closure/call reference returns need reusable consumers beyond
  descriptor closures: user functions, methods, static calls, constructors,
  discarded calls, and non-descriptor closure surfaces.
- [ ] Broad parked lanes remain evidence only until exact current-primary prep,
  review, integration, and push.

Not done:

- [ ] Full executable reference binding, references/COW identity, arbitrary
  alias roots, and alias-preserving write-through.
- [ ] Executable request storage/writeback, `$GLOBALS` self-cells,
  request/global alias parity, request foreach, and mutation-during-iteration
  behavior.
- [ ] Includes, variable variables, and dynamic symbol behavior.
- [ ] Full callable lookup and invocation, including compiler consumption of
  dynamic callable values through the repaired runtime boundary,
  `Class::method` strings, namespace fallback, autoload, named/unpacked/
  by-reference breadth beyond the integrated direct user-function path and
  runtime callable-value dispatch, magic calls, non-public method breadth, and
  rebinding rules.
- [ ] Runtime `ArrayAccess` method dispatch for `offsetGet`, `offsetExists`,
  `offsetSet`, and `offsetUnset`.
- [ ] Binary literal syntax, invalid-UTF-8 PHP source parsing, byte-exact
  interpreter output/session/debug formatting, and generalized `mb_str_split()`.
- [ ] Full PCRE behavior beyond the bounded slash-delimited subset.
- [ ] General object model: non-public methods, overrides, interfaces/traits
  execution, magic methods, dynamic/static/typed properties, destructors.
- [ ] Complete cleanup/unwind/finally/destructor/output-buffer shutdown
  behavior.
- [ ] Exact/source-ordered diagnostics, custom handler execution, warning/error
  continuation, and suppression parity.
- [ ] LLVM/direct assembly parity for recent generated-C semantics.
- [ ] Known current-primary full `native_runtime_abi` baseline failures.

## Current Work Snapshot

Primary-integrated:

- [x] Primary is clean and synced at `64f97660` before this `PROGRESS.md` edit;
  latest source capability remains `b4b21937`.
- [x] Latest primary-integrated source capability head is `b4b21937`.
- [x] Overall and executable estimates remain 95% under current project-local
  accounting.
- [x] Diagnostic operand-list blocker boundary is now a completed non-repeat
  diagnostic prerequisite: later diagnostic work must layer on the generic
  operand-list requirement boundary instead of adding source-shape blockers.
- [x] Reference-binding operand-list blocker requirements are now a completed
  non-repeat diagnostic prerequisite: future reference-binding work must extend
  or consume the shared operation-list and operand requirement ABI rather than
  adding source-shape or generated-output recognizers.
- [x] Native-link reference-handle typedef emission is now a completed
  non-repeat declaration-order repair: future helpers that can emit
  `phpc_NativeReferenceHandle` declarations must use
  `uses_native_reference_handle_type()` rather than adding inline typedef
  predicates.
- [x] Calls/functions/frames advances to 91% because runtime callable-value
  dispatch now covers multiple callable value families over the shared callable
  table and call-arguments/frame/result ABI with executable unit coverage; this
  does not claim compiler dynamic-call consumption.
- [x] Closure capture from reference-backed storage remains a completed
  non-repeat item.
- [x] Runtime callable ABI is now a completed non-repeat prerequisite.
- [x] Runtime callable-value dispatch is now a completed non-repeat runtime
  prerequisite.
- [x] Runtime callable-value dispatch repair is now a completed non-repeat
  runtime prerequisite: compiler dynamic callable consumers must use the
  repaired runtime lookup/called-scope/visibility/descriptor-argument boundary,
  not source-shape callable branches.
- [x] Direct user-function callable ABI consumers are now a completed
  non-repeat item.
- [x] Dynamic constructor symbol-environment blocking is now a completed
  non-repeat item.
- [x] Try/catch/finally body call-boundary preflight is now a completed
  non-repeat item.
- [x] Allocatable destructor-observable cleanup-risk metadata is now a
  completed non-repeat item.

Lane-local:

- [x] Dynamic callable value-context prep correctly returned
  `needs-architecture`; the runtime ABI split, runtime callable-value dispatch,
  and direct generated-C user-function consumer are now integrated.
- [ ] Next callable work should target compiler consumption of dynamic
  callable-value dispatch and method/static/constructor consumer packets over
  the integrated ABI, not another direct user-function or ABI-only inventory.
- [x] Reference-binding operation-list follow-up is integrated at `b4b21937`
  as a diagnostic/blocker extension over `a544daa8`; it is not executable
  reference binding and does not move percentages.
- [ ] Dynamic callable compiler-consumer work has an apply-ready candidate, but
  it conflicts with request/global frame-environment work and must be rebased
  after `CFrameEnvironmentRequirement`; do not preserve root-symbol-only
  request handling or extend legacy generated-C branch ladders.
- [ ] Request/global direct handoff has formal and shadow go-for-primary-review
  evidence, but it is a large frame/environment packet; integrate only after
  strict gates confirm callable ABI handoff, request-state ownership, and no
  source-shape request-key shortcuts.
- [ ] Trait effective-method metadata prep has current-primary reconciliation
  proof and a focused `trait_semantics` gate, but it is metadata-only and still
  leaves trait method execution, destructor execution, and object lifetime
  cleanup open.
- [ ] Assignment/RMW lvalue operand-list diagnostic preps are active and covered;
  serialize them around generic ABI tag allocation rather than duplicating a
  broad-lane extraction.
- [ ] Call-lane loop/call ordering work is fresh but broad and dirty.
- [ ] Diagnostic-lane callable operand, RMW, report dispatch, control-flow
  scanner contracts, and any reference-binding work beyond the integrated
  operand-list requirement packet are broad and dirty; status chronology is not
  fully reliable.
- [ ] Multiple broad lanes are evidence repositories. Do not route them to
  primary without a new narrow extraction.

Resource posture:

- `/dev/shm`: 40G total, 24G used, 17G available, 58% used; `du -sh /dev/shm`
  reported 24G.
- `/home`: 459G total, 234G used, 207G available, 54% used by `df`; bounded
  `du -sh /home` did not complete within 15 seconds.
- Memory available is about 39Gi, but swap remains high at 23Gi/29Gi used.
- Continue disk-backed `/tmp` target dirs, `umask 0007`,
  `CARGO_BUILD_JOBS=1`, `CARGO_INCREMENTAL=0`, and focused nonzero gates.

## Next Steering Read

Best next action:

- Consider request/global frame direct handoff before dynamic callable compiler
  consumption if strict final integration gates hold, because dynamic callable
  work still depends on the old root-symbol-only frame model and current-primary
  focused smoke is clean. If trait metadata lands first, account it as
  metadata/destructor-risk
  classification only. Keep diagnostic reference-binding, assignment-lvalue,
  and RMW-lvalue work layered on the generic operand-list requirement boundary
  from `a544daa8`/`b4b21937`; do not add source-shape blockers for one PHP
  source shape, one operand layout, or one diagnostic fixture. Future
  exception/destructor work should first add reusable execution,
  cleanup-ordering, or metadata boundaries rather than one-shape try,
  constructor, or destructor recognizers.

Avoid:

- Percentage bumps for architecture notes, candidate creation, selector-only
  cleanup, diagnostic/scanner-only cleanup, future-dated status claims, or
  lane-local work.
- Routing broad dirty call/diagnostic lanes directly to primary.
- Repeating runtime-only callable table/arguments/result/frame ABI work,
  runtime callable-value dispatch work,
  direct user-function callable ABI consumer work,
  reference-backed closure capture materialization,
  reference-source append/lvalue extraction, descriptor closure result ABI,
  request-key selector cleanup, conversion helper cleanup, or unary negation ABI
  work under new names.
- Extending generated-C dynamic-call shape helpers instead of consuming the
  integrated runtime callable ABI.
- Adding reference-binding or diagnostic source-shape blockers instead of
  consuming the generic operation-list and operand requirement boundary.
- Extending inline generated-C `phpc_NativeReferenceHandle` typedef predicates;
  native-link helpers that can emit reference-handle declarations must use
  `uses_native_reference_handle_type()`.
