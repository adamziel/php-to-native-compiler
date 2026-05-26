# PHP Native Compiler Progress

Updated: 2026-05-26 22:31 CEST
Evaluation marker: `20260526T040843Z`
Strategy evaluator marker: `20260526T040843Z`

Accounting rule: progress counts only generalized, tested, committed, and
pushed primary work. Dirty WIP, lane-local claims, candidate artifacts,
review-only work, probe-only commits, docs-only substitutions, and broad tests
without focused proof do not increase capability bars.

Progress bars use 20 slots. One `#` is 5%. Percentages are intentionally
coarse; they do not move for narrow scaffolding unless the integrated behavior
changes the roadmap position.

## Executive Read

Overall integrated-roadmap progress: **70%** `[##############------]`

Selected executable PHP semantics: **70%** `[##############------]`

Latest accounted source capability: `a991cf34` routes `echo` operands and
statement-form `print` through owned `NativeDiagnosticResult` output operands
and the shared stderr-plus-stdout report/free sink. The work is generalized
compiler/runtime semantics, not exact-shape lowering: existing LLVM and
generated-C expression evaluation now produce output-surface diagnostic-result
value operands for real output statements, and the runtime sink preserves
array-to-string conversion diagnostics while owning result cleanup.

Why the headline bars did not jump: the new source adds the next production
semantic operand family onto the diagnostic-result stack, but terminal,
control-flow, cleanup, lvalue, RMW, reference-binding, and call-argument
operands still need their own ownership and ordering contracts.

Current critical path to 100%:

1. Finish expression-owned `NativeCallResultHandle` carriers, invoke-result
   helper ABIs, and exactly-once call argument ownership/cleanup.
2. Continue migrating expression, statement, terminal, cleanup, lvalue,
   reference, and call-argument lowering onto produced
   `NativeDiagnosticResult` operands.
3. Route value/reference/return/deferred-cleanup consumers through the shared
   diagnostic-result and call-result carrier stack.
4. Implement references/COW, arbitrary alias-root writeback, property-held and
   nested ArrayAccess, object/static/dynamic/typed property storage.
5. Implement real exception/Throwable propagation, catch/finally/destructor/
   shutdown cleanup, source-ordered diagnostics, and custom handler behavior.
6. Broaden namespace fallback, autoload, aliases, visibility/magic,
   constructors, named/spread arguments, return references, and backend parity.

## Roadmap Bars

| Workstream | Integrated | Bar | Current read |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | **90%** | `[##################--]` | Strong selected-path value, byte-string, array, reference, symbol, callable, call-frame/result, diagnostic-result, request-state, lvalue, and ArrayAccess surfaces. Recent source adds callable access contexts, allocatable class metadata, diagnostic continuation helpers, and report sinks. Remaining gaps include executable alias transfer, autoload, namespace fallback, magic calls, closure frame handoff, cleanup/unwind parity, and broader lookup parity. |
| Compiler/backend consumers | **75%** | `[###############-----]` | Generated C has the freshest consumers for calls, callable facts, selected object metadata, selected ArrayAccess/lvalue paths, value-result casts, diagnostic-result family consumers, discarded statement-expression diagnostic operands, and echo/print output diagnostic operands. LLVM shares the discarded-expression and output operand paths, while direct assembly still lags newer object-offset/lvalue/runtime ABIs and most semantic result operands remain unmigrated. |
| Executable PHP semantics | **70%** | `[##############------]` | Many executable islands exist, but full assignment/RMW/writeback, references/COW, object/ArrayAccess operations, cleanup/unwind/finally/destructors, exact diagnostics, and backend parity remain open. |
| Strings and byte semantics | **60%** | `[############--------]` | Byte-backed values and selected byte-preserving string-array slots are integrated. Binary source bytes, byte-exact interpreter/session/debug output, `mb_str_split()`, request/global byte keys, and exact diagnostics remain open. |
| Arrays, lvalues, references, COW | **70%** | `[##############------]` | Selected lvalue/reference-source extraction, ReferenceSlot owner facts, reference-cell predicates, membership helpers, RMW array-lvalue owner/writeback, and selected ArrayAccess RMW/`??=` paths are integrated. Object/static property storage, property-held/nested ArrayAccess, arbitrary alias roots, foreach breadth, broader writeback, and full COW remain incomplete. |
| Symbols, globals, request state | **70%** | `[##############------]` | Selected globals, root-symbol consumers, active symbol-table consumers, request-key blockers, append-shaped symbol reference-source materialization, direct generated-C request-state frame handoff, and dynamic user-function handoff proof exist. `$GLOBALS` self-cells, closure request-state handoff, request/global alias parity, request writeback, includes, variable variables, and exact unset/global behavior remain incomplete. |
| Calls, functions, frames | **80%** | `[################----]` | Runtime callable table/value dispatch, call arguments/frame/result ABI, conditional handoff, generated-C direct/dynamic callable consumers, declared-method registration/wrapper frames, callable return facts, by-reference argument transport, descriptor closures, closure returns, request-state frame handoff, and recent access-context lookup ABI are integrated. Unknown runtime callables, builtin return summaries, executable by-reference alias transfer, full object/method parity, namespace fallback, autoload, magic calls, named/spread breadth, broader return references, constructors, cleanup/unwind, and backend parity remain open. |
| Objects, properties, methods | **65%** | `[#############-------]` | Selected object metadata, public property reference-source extraction, object-property reference-slot mutation, ArrayAccess dispatch, generated-C ArrayAccess consumers for compiler-known generated objects, dynamic generated class-name producers, object-call argument handles, declared-method callable-table publication, allocatable class metadata, and access-context preflights exist. Property/magic/unknown-runtime-dynamic-call/clone/static-property producers, property-held/nested ArrayAccess, broader visibility parity, typed properties, destructors, interfaces/traits execution, references/COW, constructors, and backend parity remain open. |
| Control flow, cleanup, diagnostics | **60%** | `[############--------]` | Selected branches, loops, transfers, finalizers, output buffers, diagnostic blockers, owned diagnostic-result list contracts, consumer contracts, backend family consumers, deferred-cleanup blockers, try-body call-boundary preflight, report sinks, continuation helpers, discarded statement-expression operands, and echo/print output operands exist. Broad unwind/finally/destructor/shutdown execution, cleanup ownership, executable reference binding, remaining semantic diagnostic-result producer migration, and source-ordered diagnostics remain open. |
| Broad integrated verification | **70%** | `[##############------]` | Focused gates around recent source work are strong. Broad verification is still constrained by lane extraction cost, stale candidate expectations, heavy formatter/log pressure, and backend parity gaps. |

## Recently Accounted Source Work

| Commit | Capability | Proof shape |
| --- | --- | --- |
| `a991cf34` | Echo operands and statement-form `print` lower into owned `NativeDiagnosticResult` output operands and report/free through the shared echo sink. | Compiler output-operand tests, runtime echo-sink diagnostic test, executable generated-C link/run proof, fmt, diff check. |
| `dcdd330f` | Discarded expression statements lower into owned `NativeDiagnosticResult` operands and report/free through diagnostics-only sinks in LLVM and generated C. | Compiler result-operand tests, `native_runtime_abi` sink tests, executable generated-C link/run proof, fmt, diff check. |
| `5902369c` | Shared callable access-context lookup, allocatable class metadata, diagnostic-result continuation helpers, and stderr/echo report sinks. | Runtime focused tests, compiler ABI declaration tests, `native_runtime_abi` tests, fmt, diff check. |
| `950a17fe` | LLVM/generated-C diagnostic-result family consumers over already-produced operand lists. | Family selector, backend emission, empty-list, missing-runtime-ABI, fmt, diff check. |
| `81c60f38` | Runtime diagnostic-result consumer contracts for value-required and cleanup families. | Result-list ownership, terminal preservation, null/empty list behavior. |
| `099b76fc` | Owned `NativeDiagnosticResult` value/diagnostic/null result contract. | Value, diagnostic, null, list cleanup, adjacent blocker behavior. |
| `08d00fe1` | Conditional call-frame result handoff for short-ternary/null-coalescing families. | Success/diagnostic preservation and cleanup-sensitive blockers. |
| `7fb9db15` | Shared object metadata/type-introspection builtin preflight. | `class_exists`, `property_exists`, `is_a`, direct-call cleanup diagnostics. |
| `a3826e2f` | Generated-C dynamic instance method name normalization through native value helper. | Runtime lookup normalization and generated-C helper selection. |
| `3ac78d8b` | Shared generated-C object-call argument handles. | Constructor, method, static, callable-array, invokable-object argument families. |
| `73195f96` | Native value-result cast diagnostics. | Array-to-string warnings over direct and compound value-result paths. |
| `0bebd2e9` | By-reference alias-transfer result boundary for produced call results. | Direct generated user-function call consumers for echo/print/discard. |
| `b3d90dbc` | Runtime/compiler reference-cell predicate and membership boundaries. | `isset`, `empty`, truthiness, `array_key_exists` over value/reference subjects. |
| `05214fd4` | Compiler-known declared-method callable identities and return summaries. | Public/static/object receiver policy, callable identities, return-summary resolution. |

## Active Roadmap Items

Primary-integrated capability and candidate/lane-local work are separated.

| Item | Primary Integrated | Candidate Readiness | Toward Full Feature | Status |
| --- | ---: | ---: | ---: | --- |
| Diagnostic-result carrier stack | **100%** `[####################]` | **100%** `[####################]` | **55%** `[###########---------]` | Runtime/result contracts, family consumers, continuation helpers, report sinks, discarded statement-expression operands, and echo/print output operands are integrated. Terminal, cleanup, lvalue, reference, RMW, and call-argument operands still need exact ownership and ordering migrations. |
| Callable access context and class metadata | **100%** `[####################]` | **100%** `[####################]` | **40%** `[########------------]` | Shared runtime access-context policy and allocatable-class metadata are integrated for function/method/static/constructor lookup preflights. Generated semantic call lowering, constructor execution, autoload, magic, and full visibility parity remain open. |
| ArrayAccess compiler consumers | **100%** `[####################]` | **100%** `[####################]` | **55%** `[###########---------]` | Generated-C direct-object/direct-variable read, `isset`, `empty`, `??`, write, append, unset, compound assignment, and `??=` are integrated for compiler-known generated declared `ArrayAccess` objects. Property-held/nested owners, append RMW, increment/decrement, reference-returning `offsetGet`, references/COW, cleanup/unwind, and backend parity remain open. |
| ReferenceSlot owner facts | **100%** `[####################]` | **100%** `[####################]` | **45%** `[#########-----------]` | Compiler-visible native reference handles can recover facts, source owners, and commit writeback for selected paths. Arbitrary alias roots, request/superglobal path facts, property-held references, closure callback fact transport, references/COW, and backend parity remain open. |
| Callable identity return summaries | **100%** `[####################]` | **100%** `[####################]` | **60%** `[############--------]` | Generated functions, declared methods/static methods, descriptor closures, known strings, definite `__invoke` objects, and compiler-known callable arrays can publish selected return facts. Unknown runtime callables, builtins, non-descriptor closures, recursive summaries, reference returns, property/magic producers, references/COW, and backend parity remain open. |
| Dynamic object/interface fact carrier | **100%** `[####################]` | **100%** `[####################]` | **65%** `[#############-------]` | Generated declared objects, known dynamic class-name `new`, copies, gotos, branches, generated-callable returns, descriptor closures, known string/invokable/callable-array summaries, and compiler-visible reference slots feed existing object/interface consumers. Properties, clones, static properties, arbitrary symbols, unknown runtime callables/arrays, and non-descriptor closures remain open. |
| Cleanup/unwind execution | **25%** `[#####---------------]` | **25%** `[#####---------------]` | **25%** `[#####---------------]` | Requirement/preflight boundaries are integrated. Actual exception propagation, catch/finally/destructor/shutdown execution, cleanup ordering, and object lifetime cleanup are still not implemented. |
| Broad dirty lane extraction backlog | **0%** `[--------------------]` | **35%** `[#######-------------]` | **35%** `[#######-------------]` | Dirty call, diagnostic, object, control-flow, symbol, byte/string, and array lanes remain evidence pools until split into fresh current-head candidates with focused proof. |

## Done

- Runtime callable table plus call arguments/frame/result ABI.
- Runtime callable-value dispatch for selected function names, callable arrays,
  descriptor closures, inherited methods, bound receivers, and object
  `__invoke`.
- Direct generated-C user-function calls and dynamic generated-C callee
  expressions through shared runtime callable lookup/invocation.
- Generated declared-method callable-table registration and wrapper frames.
- Receiver-free static `Class::method` string callable lookup through the
  runtime callable-value ABI.
- Shared diagnostic operation/operand-list blocker boundary.
- Owned diagnostic-result contracts, family consumers, continuation helpers,
  and report sinks for selected diagnostic-result paths.
- Discarded expression statements in LLVM and generated C lower through owned
  `NativeDiagnosticResult` statement operands and diagnostics-only report
  sinks.
- Echo operands and statement-form `print` in LLVM and generated C lower
  through owned `NativeDiagnosticResult` output operands and the shared echo
  report sink, including array-to-string conversion diagnostics.
- Reference-binding, assignment-lvalue, and RMW-lvalue operand-list blockers.
- Generated-C selected RMW array-lvalue owner/writeback for local native arrays
  and active-symbol/global-import reference-slot owners.
- Cleanup/unwind requirement diagnostics/preflight.
- Runtime ArrayAccess read/exists and write/append/unset dispatch ABIs.
- Generated-C ArrayAccess read/isset/write/append/unset/empty/null-coalesce/
  RMW/`??=` consumers for compiler-known generated declared `ArrayAccess`
  objects.
- Shared generated-C native value/object facts for selected generated declared
  object producers and callable return summaries.
- Shared generated-C ReferenceSlot value owner source/commit and reference-cell
  fact ledger for compiler-visible native reference handles.
- Declared-class allocation cleanup-risk metadata and allocatable-class lookup
  metadata.
- Selected reference-source/lvalue extraction, reference-backed closure capture
  materialization, descriptor closure returns, byte-backed PHP string values,
  and byte-preserving selected string-array slots.

## Not Done

- Dynamic ArrayAccess producers beyond known generated declared-class `new` and
  direct generated-callable return summaries.
- Property-held and nested ArrayAccess writes/unsets, compound/RMW,
  null-coalesce assignment, append/increment forms, and reference-returning
  ArrayAccess semantics.
- Object/static/dynamic/typed property storage and full method/object model
  execution.
- Full reference/COW identity and arbitrary alias-root writeback.
- Actual exception/Throwable propagation, catch matching/binding, `finally`,
  destructors, shutdown cleanup, and object lifetime cleanup.
- Namespace fallback, autoload, class aliases, broader visibility, magic calls,
  constructors, named/spread arguments, and return references.
- Remaining semantic diagnostic-result operand migration for terminal, cleanup,
  lvalue, reference, RMW, and call-argument families; exact PHP diagnostics,
  source ordering, suppression/custom handlers, and backend parity across
  generated C, LLVM, and direct assembly.

## Latest Focused Verification

For `a991cf34`:

- `cargo test -p phpc --lib native_diagnostic_result_ -- --nocapture`
- `cargo test -p phpc --test native_runtime_abi native_diagnostic_result_ -- --nocapture`
- `cargo test -p phpc --test native_link emit_exe_links_and_runs_output_diagnostic_result_operands -- --nocapture`
- `cargo fmt --check -p phpc`
- `git diff --check`
