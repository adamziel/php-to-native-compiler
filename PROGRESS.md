# PHP Native Compiler Progress

Updated: 2026-05-26 18:44 CEST
Evaluation marker: `20260526T040843Z`
Strategy evaluator marker: `20260526T040843Z`

Accounting rule: only generalized, tested, committed, and pushed primary work
counts as integrated capability. Dirty WIP, candidate worktrees, lane-local
claims, review-only work, failed prep proofs, probe-only commits, architecture
notes, and dashboard-only commits are excluded.

## Executive Read

Overall integrated-roadmap progress: **70%** `[##############------]`

Selected executable PHP semantics: **70%** `[##############------]`

Calibration: bars count only primary-integrated, tested, committed, and pushed
work. Each bar has 20 slots, so each `#` is 5%. Dirty lanes and candidate
artifacts do not count until integrated.

Primary `HEAD` is synced with `origin/master` at `0f9132ab` (`docs: account
diagnostic result consumer contracts`). The primary worktree is clean except
for preserved untracked `examples/class.php`.

Latest source capability: `81c60f38` adds semantic-family-driven diagnostic-
result list consumer contracts for generated LLVM/native C. Supported
expression/statement/control-flow/lvalue/reference/call-argument families now
select the shared value-required operation-list ABI; deferred cleanup selects
the cleanup-list ABI; declaration/termination families are explicit
missing-runtime-ABI blockers.

Current blockers to 100%:

- expression-owned `NativeCallResultHandle` carriers, invoke-result helper
  ABIs, and exactly-once argument ownership/cleanup;
- real diagnostic-result consumer wiring for emitted expression, statement,
  terminal, cleanup, lvalue, reference, and call-argument operands;
- references/COW, arbitrary alias-root writeback, property-held/nested
  ArrayAccess, and broader object/static/dynamic/typed property storage;
- actual exception/Throwable propagation, catch/finally/destructor/shutdown
  cleanup, and source-ordered diagnostics;
- namespace fallback, autoload, aliases, visibility/magic, constructors,
  named/spread breadth, return references, and backend parity.

Next direction: finish the call-result and diagnostic-result carrier stack
first, then migrate conditional/deferred-cleanup/value/reference/return
consumers onto it, then broaden objects/properties/ArrayAccess/references/
unwind with focused proof.

## Grand Roadmap Position

| Workstream | Estimate | Bar | Current read |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | **90%** | `[##################--]` | Strong selected-path value, byte-string, array, reference, symbol, callable table, callable-value dispatch, call-frame/result, conditional call-result handoff, owned diagnostic-result list contracts, diagnostic-result consumer contracts, native value-result cast diagnostics, produced-result by-reference alias-transfer diagnostics, request-state, diagnostics, lvalue, and ArrayAccess read/write dispatch surfaces. Remaining gaps include executable alias transfer, broader callable lookup parity, namespace fallback, autoload, magic calls, constructors, compiler-side conditional handoff consumers, closure frame handoff, reference-return ArrayAccess, and cleanup/unwind parity. |
| Compiler/backend consumers | **75%** | `[###############-----]` | Generated C has the freshest executable consumers for direct/dynamic callables, declared methods, selected dynamic method-name normalization, selected object-metadata preflight diagnostics, selected scoped `Class::method` signatures, shared object-call argument handles, selected callable return facts, selected reference-cell predicates, selected value-result cast diagnostics, selected RMW array-lvalue owner/writeback, and selected compiler-known ArrayAccess consumers. LLVM/direct assembly still lag recent object-offset/lvalue/runtime ABIs, and real diagnostic-result consumer wiring is still pending. |
| Executable PHP semantics | **70%** | `[##############------]` | Many selected executable islands exist, but major semantics remain open: full assignment/RMW/writeback, references/COW, executable object/ArrayAccess operations, cleanup/unwind/finally/destructors, exact diagnostics, and backend parity. |
| Strings and byte semantics | **60%** | `[############--------]` | Byte-backed values and byte-preserving selected string-array slots are integrated. Binary source bytes, byte-exact interpreter/session/debug output, `mb_str_split()`, request/global byte keys, and exact diagnostics remain open. |
| Arrays, lvalues, references, COW | **70%** | `[##############------]` | Selected reference-source/lvalue extraction, ReferenceSlot owner facts, reference-cell predicates, membership helpers, reference/assignment/RMW diagnostics, selected RMW array-lvalue owner/writeback, and selected ArrayAccess RMW/`??=` paths are integrated. Object/static property storage, property-held/nested ArrayAccess RMW, arbitrary alias roots, foreach, broader writeback, and full COW remain incomplete. |
| Symbols, globals, request state | **70%** | `[##############------]` | Selected globals, root-symbol consumers, active symbol-table consumers, request-key blockers, append-shaped symbol reference-source materialization, direct generated-C request-state frame handoff, and generated-C dynamic user-function handoff proof exist. `$GLOBALS` self-cells, closure request-state handoff, request/global alias parity, request writeback, includes, variable variables, and exact unset/global behavior remain incomplete. |
| Calls, functions, frames | **80%** | `[################----]` | Runtime callable table/value dispatch, call arguments/frame/result ABI, conditional call-result handoff, direct/dynamic generated-C callable consumers, generated declared-method registration/wrapper frames, selected dynamic method-name normalization, shared object-call argument handles, selected callable identity/return facts, by-reference argument transport, descriptor closures, closure returns, request-state frame handoff, and null call-result diagnostic cleanup are integrated. Unknown runtime callables, builtin return summaries, executable by-reference alias transfer, compiler-side conditional result consumers, full object/method parity, namespace fallback, autoload, magic calls, named/spread breadth, broader return references, constructors, cleanup/unwind execution, and backend parity remain open. |
| Objects, properties, methods | **65%** | `[#############-------]` | Public object-property reference-source extraction, object-property reference-slot mutation, declared-class allocation cleanup-risk metadata, shared native object-metadata preflight diagnostics, Object/ArrayAccess write blockers, runtime ArrayAccess write/read/exists dispatch, generated-C ArrayAccess read/isset/write/append/unset/empty/null-coalesce/RMW/`??=` consumers for compiler-known generated objects and selected reference slots, known dynamic generated-declared class-name producers, generated-callable, descriptor-closure, known string callable, callable-array, definite `__invoke` object return producers, shared generated-C object-call argument handles, PHP-compatible object receiver public static callable facts, and generated declared-method callable-table publication exist for selected paths. Property/magic/unknown-runtime-dynamic-call/clone/static-property producers, property-held/nested ArrayAccess owners, broader visibility parity, magic, dynamic/static/typed properties, destructors, interfaces/traits execution, references/COW, constructors, and backend parity remain open. |
| Control flow, cleanup, diagnostics | **50%** | `[##########----------]` | Selected branches, loops, transfers, finalizers, output buffers, diagnostics, owned diagnostic-result list contracts, diagnostic-result consumer contracts, deferred-cleanup diagnostic-result blockers, try-body call-boundary preflight, generic operand-list blockers, reference/assignment/RMW blockers, Object/ArrayAccess write blockers, and cleanup/unwind requirement preflight exist. Broad unwind/finally/destructor/shutdown execution, cleanup ownership, executable reference binding, cleanup-result compiler migration, and source-ordered diagnostics remain open. |
| Broad integrated verification | **75%** | `[###############-----]` | Focused gates around recent source work are strong. Broad verification is still constrained by lane extraction cost, stale candidate expectations, heavy swap/log pressure, and backend parity gaps. |

## Recent Primary-Integrated Work

- `885bde45`: deferred-cleanup diagnostic results now share
  `NativeDiagnosticResultListBlocker` with the existing value-required
  operation result-list consumer. The new
  `phpc_native_diagnostic_result_deferred_cleanup_blocker_list_and_free(...)`
  ABI consumes ordered cleanup results, releases owned values, preserves
  terminal diagnostics, cleans up remaining entries, handles null
  item/list/empty-list shapes, and appends a centralized cleanup-ordering
  blocker for finally/destructor/shutdown cleanup semantics. Focused proof
  covers value-only cleanup results, terminal diagnostics, null result items,
  null result-list pointers, empty cleanup lists, the existing operation
  result-list path, adjacent diagnostic operation blockers, `cargo check` for
  `php_runtime` and `phpc --lib`, fmt, and diff checks. Compiler/backend
  cleanup-result consumers, real unwinding, diagnostic aggregation,
  exception/fatal ownership, return/control-transfer joins, references/COW,
  exact cleanup ordering, and source-aware reporting remain open.
- `099b76fc`: runtime diagnostics now have an owned
  `NativeDiagnosticResult` result/value contract and
  `phpc_native_diagnostic_result_value_required_operation_blocker_list_and_free(...)`
  as a shared value-required result-list consumer. The ABI carries owned values
  plus ordered diagnostics, exposes diagnostic inspection/freeing helpers, and
  consumes result lists across ordinary values, terminal diagnostics, null
  items, null pointers, empty lists, and cleanup of remaining unconsumed
  results. The older handle-based operand-requirement blocker ABI remains for
  current compiler consumers. Focused proof covers the listed result shapes,
  adjacent diagnostic operation blocker behavior, `cargo check` for
  `php_runtime` and `phpc --lib`, fmt, and diff checks. Compiler/backend
  migration to owned diagnostic results, deferred-cleanup/finally/destructor/
  shutdown result families, source/error-handler freezing, references/COW,
  exact cleanup ordering, and full unwinding remain open.
- `08d00fe1`: runtime call-frame conditional result handoff now shares
  `NativeCallFrameConditionalHandoffFeature` and
  `phpc_native_call_frame_result_conditional_handoff_contract_result(...)`
  across short-ternary condition and null-coalescing left-operand result
  handoff. The helper keeps ordinary owned results, propagates existing
  diagnostics, and centralizes cleanup-sensitive blockers for resource,
  object, reference, array reference-slot, array shared-COW, and closure
  capture families before branch discard/handoff choices. Focused proof covers
  both feature tags, success/diagnostic preservation, cleanup blocker
  selection, adjacent call-result consumer behavior, `cargo check` for
  `php_runtime` and `phpc --lib`, fmt, and diff checks. Compiler-side
  conditional consumers, exact branch discard ordering, references/COW,
  destructor/resource cleanup, structured unwind/finally execution, exact
  diagnostics, and backend integration remain open.
- `7fb9db15`: LLVM and generated-C metadata/type-introspection builtin
  preflight now share `NativeObjectMetadataCallOperation` and
  `NativeObjectMetadataCallPreflightFailure`. The boundary covers
  class/interface/trait/enum existence, property/method existence, and
  relationship metadata families, checks call-result argument dependencies
  before arity, and projects both failure classes through `NativeCallDiagnostics`
  instead of duplicating family-specific preflight in each backend. Focused
  proof covers the new boundary over `class_exists`, `property_exists`, and
  `is_a`, adjacent direct-call cleanup diagnostics, builtin-class metadata
  rejection, `cargo check -p phpc --lib`, fmt, and diff checks. The static
  metadata IR snapshot gate remains a non-gate because it fails on stale
  stdout-helper snapshot text unrelated to this metadata preflight; the same
  metadata fixtures still emit IR successfully. Full native metadata tables,
  object-property rejection carriers, receiver/static receiver execution,
  visibility/magic/typed properties, references/COW, exact diagnostics, and
  backend parity remain open.
- `a3826e2f`: generated-C dynamic instance method dispatch now uses
  `phpc_native_value_dynamic_method_name_matches()` instead of the string-only
  dynamic function-call matcher. Runtime method-name matching and dynamic
  method failure diagnostics share one scalar-normalization boundary for
  string, binary-string, int, true, and float operands, while false, null,
  empty, and non-scalar values remain centralized misses or diagnostics.
  Focused proof covers runtime lookup normalization, diagnostic normalization,
  generated-C helper selection, fmt, and diff checks. The neighboring broader
  dynamic-method native-link fixture remains blocked before its assertions by
  the known non-local object/static property assignment owner-cell contract.
  General method tables, inherited/magic method lookup, visibility/context
  policy, executable method frames, object/property assignment, references/COW,
  exact diagnostics, and backend parity remain open.
- `3ac78d8b`: generated-C object call dispatch now shares
  `NativeObjectCallDispatchArgumentHandles` and
  `CNativeObjectCallDispatchFrameArguments` across declared constructor,
  declared instance method, dynamic declared instance method, static method,
  callable-array method, invokable-object, and constructorless argument-array
  consumers. The carrier keeps call arguments, owned `NativeValue` argument
  handles, and cleanup together instead of duplicating tuple/local aggregation
  in each call family. Focused proof covers method/static/constructor/default
  and variadic argument families, existing constructor dispatch unsupported
  boundaries, public instance method `$this` binding and handle preservation,
  fmt, and diff checks. LLVM object/class lowering, broader method-frame
  execution, receiver/static receiver parity, visibility/magic/typed-property
  policy, references/COW, constructor/destructor lifecycle, autoload/class
  lookup parity, exact diagnostics, and backend parity remain open.
- `73195f96`: generated-C cast lowering now routes through
  `phpc_native_value_cast_result()` and the shared
  `NativeValueOperationResult` carrier instead of the narrower
  diagnostic-pointer ABI. Runtime array-to-string casts return `"Array"` with
  `Warning: Array to string conversion`, and the legacy
  `phpc_native_value_cast_operation_with_diagnostic()` helper delegates through
  the same result boundary. Focused proof covers runtime cast result carriers,
  legacy helper compatibility, generated-C source routing, linked executable
  output/warnings for `(string)` and `strval()` over array values from direct
  assignment, null-coalescing assignment, and compound array-union
  value-result paths, adjacent compare/cast/type-name ABI consumers, fmt, and
  diff checks. Diagnostic-capable runtime dynamic callable builtins, object
  `__toString()`, Closure/resource cast parity, exact cast diagnostics,
  arbitrary callback/frame execution, references/COW, cleanup/unwind, and
  backend parity remain open.
- `0bebd2e9`: runtime/compiler by-reference alias-transfer result boundary for
  produced call results supplied to fixed by-reference user-function
  parameters. Runtime exposes
  `phpc_native_call_frame_reference_parameter_alias_transfer_result_from_results_with_diagnostic()`
  plus a public value-taking call-result consumer; generated C detects direct
  declared user-function calls whose by-reference parameters receive produced
  call results and routes echo, print, and discard consumers through the shared
  diagnostic-aware call-result path. Focused proof covers target/value-family
  preservation, prior argument diagnostic precedence, generated-C consumers
  over echo/print/discard direct-call families, adjacent non-produced
  by-reference and by-value calls avoiding the boundary, fmt, diff checks, and
  `cargo check -p php_runtime -p phpc`. Executable caller-visible rebinding,
  typed/default/variadic/named/unpacked by-reference parameters,
  by-reference return alias handoff, references/COW, cleanup/unwind/destructor
  ordering, exact PHP diagnostics, and backend parity remain open.
- `273c2e6e`: generated-C scoped callable-string signature planning for known
  `Class::method` strings. Variable-held, concatenated, ternary, and
  short-ternary finite string sets can resolve declared public static method
  metadata before runtime callable-value invocation, so by-reference argument
  planning uses the shared call-arguments carrier and selected by-reference
  public static method returns can satisfy reference-assignment consumers
  through the runtime callable-value reference path. Focused proof covers
  variable-held, concatenated, branch-selected, and reference-return scoped
  callable strings, the linked executable output
  `variable:variable|concat:concat|branch:branch|reference`, neighboring
  dynamic string callable values, runtime by-reference dynamic user-function
  frames, callable-array invocation, and ArrayAccess native-link regressions.
  Runtime-only unknown callable strings, branch-selected incompatible
  signatures, constructors, non-public/non-static scoped strings, typed
  by-reference returns, autoload, full callable diagnostics, references/COW,
  cleanup/unwind, LLVM consumers, and backend parity remain open.
- `b3d90dbc`: runtime and compiler reference-cell predicate/membership
  boundaries. Runtime exposes `phpc_native_reference_predicate()` for
  `isset`, `empty`, and truthiness over reference cells, plus
  `phpc_native_value_array_key_exists_value_with_diagnostic()` and
  `phpc_native_reference_array_key_exists_value_with_diagnostic()` for shared
  array-key membership over value and reference-cell subjects. LLVM and
  generated C consume those helpers for direct aliases, reference slots,
  symbol-table reference slots, by-reference foreach cells, and direct native
  values; the interpreter recognizes `key_exists` through the same builtin
  family as `array_key_exists`. Focused proof covers runtime predicate and
  membership helpers, IR reference truthiness without value-clone detours,
  generated-C reference and value membership routing, neighboring
  isset/empty/reference-binding/RMW diagnostics, array-key interpreter parity,
  and native-link reference-slot owner regressions. Arbitrary alias roots,
  property-held/nested reference owners, ArrayAccess reference-return owners,
  cleanup/unwind/destructor ordering, exact source-ordered diagnostics, full
  references/COW, and backend parity remain open.
- `73a9c58d`: runtime call-result diagnostic consumers now clear stale
  diagnostic slots through one shared value/reference/discard consumer boundary
  before inspecting the incoming result handle, including null handles. Direct
  callable and callable-value invocation wrappers route value/reference/discard
  consumption through the shared helpers after preserving diagnostics produced
  by result-producing invocation paths. Focused proof covers null value,
  reference, and discard consumers plus neighboring call-arguments/result
  ownership. Exact PHP fatal timing/text, destructor-observable cleanup,
  request/global frame separation, full references/COW, autoload, broader
  object/property/static-property semantics, unsupported control-flow parity,
  and backend parity remain open.
- `05214fd4`: compiler-known declared-method callable identities now share an
  external callable policy for class-method strings, class-string callable
  arrays, object receiver callable arrays, and invokable object identities.
  Class-string surfaces publish facts only for public static methods; object
  receiver callable arrays publish facts for public instance and public static
  methods; ordinary non-public external method callables do not publish facts;
  static `__invoke` metadata is rejected while local PHP's warning-but-callable
  non-public `__invoke` behavior is preserved. Focused proof covers accepted
  public static strings/arrays, accepted object public instance/static arrays,
  rejected class-string instance methods, rejected non-public ordinary methods,
  accepted public/non-public `__invoke`, static `__invoke` refusal, callable
  identity regressions, callable-array identity invalidation, and generated
  method return-summary resolution. Dynamic method names, caller-scope-sensitive
  non-public ordinary methods, exact runtime diagnostics, unknown runtime
  callables, cleanup/unwind, LLVM consumers, and backend parity remain open.
- `4dc4d791`: callable-array variable identities are now invalidated across
  shared native-array owner mutation boundaries. Direct element writes, append,
  unset, null-coalesce lvalue writes, compound/update lvalue writes,
  by-reference foreach over native array owners, and native array mutating
  builtins clear stale callable-array facts before later dynamic calls can
  publish return facts from outdated class/method pairs. Focused proof covers
  pre-mutation callable-array facts followed by post-mutation refusal for
  element assignment, append, and unset, plus neighboring callable-array and
  native-link callable-array regressions. Unknown runtime arrays, runtime-only
  mutations, list/destructuring replacement, broader references/COW, cleanup/
  unwind, LLVM consumers, and backend parity remain open.
- `7aa27530`: callable-array return facts now consume compiler-known callable
  array identities through the shared generated-C callable summary boundary.
  Literal class/static arrays, object receiver arrays, explicit numeric key
  order, inherited object methods, and variable-assigned callable arrays can
  publish native return facts for downstream consumers such as ArrayAccess
  reads. Focused proof covers static callable arrays, object and inherited
  object method arrays, keyed element order, arity mismatch refusal,
  conservative mixed-receiver refusal, variable assignment/reassignment, a
  generated-C ArrayAccess source route, a linked executable producing
  `C|C|C`, and neighboring dynamic-callable/descriptor-closure regressions.
  Runtime arrays with unknown shape, builtin-produced arrays, interface-only
  receivers, union/object facts without complete method coverage,
  by-reference array elements/returns, cleanup/unwind, LLVM consumers, and
  backend parity remain open.
- `d2b60ba7`: dynamic callable return facts now consume more callable
  identities through the shared generated-C callable summary boundary. Known
  string callable values resolve to generated functions and declared static
  method strings; definite generated object facts resolve callable objects
  through declared `__invoke`, including inherited methods; supported builtin
  names are represented as identities but still publish unknown return
  summaries. Focused proof covers generated-function strings, branched known
  string sets, declared static-method strings, inherited invokable objects,
  descriptor-closure copies, generated-C ArrayAccess fact routing, and a linked
  executable using all four callable producer surfaces. Callable arrays,
  unknown runtime strings/objects, builtin return facts, reference returns,
  cleanup/unwind, LLVM consumers, and backend parity remain open.
- `67efa804`: descriptor-backed closures now participate in the shared
  generated-C callable identity and return-summary boundary. Closure callback
  bodies record descriptor summaries with arity, variadic shape, result kind,
  and native value facts; known descriptor closure identities propagate through
  native value handles, variable storage, copied variables, and branch joins;
  `Expr::DynamicCall` consumes those identities through the existing
  callable-summary intersection path. Focused proof covers descriptor-closure
  arity/result filtering, mixed identity intersections, dynamic calls through
  variables and copies, by-reference return refusal, existing generated
  function/method callable summaries, and executable descriptor-closure
  invocation gates. Dynamic runtime strings/arrays/objects/builtins,
  non-descriptor closures, generalized reference returns, captures/COW,
  cleanup/unwind, exact diagnostics, LLVM consumers, and backend parity remain
  open.
- `6caaa387`: generated-C callable identity and return-summary boundary for
  generated functions, declared instance methods, and declared static methods.
  Existing generated-callable return facts now flow through a shared resolver
  that checks arity, rejects reference returns for native value facts, and
  preserves conservative intersection behavior for method candidate sets.
  Focused proof covers direct function, instance method, and static method
  identities plus the existing ArrayAccess callable-producer native-link
  consumers. Dynamic runtime callables, descriptor closures, property/magic
  producers, recursive/fixed-point summaries, references/COW, cleanup/unwind,
  exact diagnostics, LLVM consumers, and backend parity remain open.
- `2ee642ff`: generated-C ReferenceSlot value owner source/commit and
  reference-cell fact ledger for compiler-visible native reference handles.
  Reference-backed variables can now recover object/interface facts from a
  shared slot, ArrayAccess mutation owners can read through
  `phpc_native_reference_value_clone()`, and owner commits write back through
  `phpc_native_reference_set_value()` while updating facts. Executable proof
  covers by-reference closure capture promotion and function-scope `global`
  import roots using the same ArrayAccess write/RMW owner boundary. Arbitrary
  alias roots, property-held/nested owners, request/superglobal fact recovery,
  closure callback fact transport, reference-returning `offsetGet`,
  cleanup/unwind, exact diagnostics, LLVM consumers, and backend parity remain
  open.
- `369099d7`: conservative generated-callable return-result facts through the
  shared `CNativeValueFacts` carrier. Generated functions and declared methods
  now summarize known return facts, intersect branch returns, and clear facts
  for fallthrough, unknown, or non-mixed return cases. Direct user-function,
  direct instance-method, direct static-method, and object-static candidate
  calls can feed existing ArrayAccess fact consumers without adding an
  ArrayAccess-specific call recognizer. Executable proof covers function,
  instance method, and static method producers consumed by ArrayAccess read,
  `isset`, `empty`, and null-coalesce, plus a default-fallthrough negative
  case. Recursive/fixed-point summaries, dynamic runtime callables, descriptor
  closures, property-held/nested owners, references/COW, cleanup/unwind, exact
  diagnostics, LLVM consumers, and backend parity remain open.
- `653a5918`: generated-C direct-variable ArrayAccess compound assignment and
  null-coalesce assignment for compiler-known generated declared `ArrayAccess`
  object values. Compound assignment routes `offsetGet`, native binary result
  computation, `offsetSet`, and owner commit through the shared owner boundary.
  `??=` routes `offsetExists`, conditional `offsetGet`, native null checks,
  lazy RHS materialization, `offsetSet` when missing/null, and final owner
  commit. Executable proof covers numeric and string RMW, branch-joined
  direct-variable subjects, missing offsets, present null, present integer
  zero, string zero, false, truthy values, and RHS laziness. Negative proof
  keeps property-held, nested, append-RMW, increment/decrement, and unknown
  dynamic class-name owner shapes blocked. Reference-returning `offsetGet`,
  property-held/nested owners, references/COW, cleanup/unwind, exact
  diagnostics, LLVM consumers, and backend parity remain open.
- `1a9f0a1c`: shared generated-C direct-variable ArrayAccess owner
  materialization and writeback commit boundary for keyed write, append, unset,
  and assignment-expression writeback over compiler-known generated declared
  `ArrayAccess` object values. The focused proof preserves existing
  write/unset and broader ArrayAccess native-link behavior while making the
  subject/offset/replacement ownership contract reusable for upcoming RMW and
  `??=` routing. Property-held/nested owners, ArrayAccess RMW, `??=`,
  reference-returning `offsetGet`, references/COW, cleanup/unwind, exact
  diagnostics, LLVM consumers, and backend parity remain open.
- `4311df7e`: generated-C compiler consumption of ArrayAccess `empty()` and
  null-coalesce over the shared runtime ArrayAccess read/exists ABI for
  compiler-known generated declared `ArrayAccess` objects, including the
  shared native value/object facts from `f9b721a2`. `empty($aa[$key])` uses
  `offsetExists`, conditional `offsetGet`, and native PHP truthiness;
  `$aa[$key] ?? rhs` uses `offsetExists`, conditional `offsetGet`, native null
  checks, and lazy RHS materialization. The executable proof covers missing
  offsets, present truthy values, integer zero, string zero, null reads, and
  fallback side effects. Property-held/nested owners, ArrayAccess RMW,
  `??=`, reference-returning `offsetGet`, references/COW, cleanup/unwind,
  exact diagnostics, LLVM consumers, and backend parity remain open.
- `f9b721a2`: shared generated-C native value/object fact carrier for
  ArrayAccess-capable generated declared objects. Read, `isset`, write, append,
  and unset now consume the same definite native object/interface fact query
  instead of a split ArrayAccess-only variable set. Known dynamic class-name
  `new` producers receive ArrayAccess facts only when all known candidate
  classes resolve to generated declared classes whose interface metadata
  includes `ArrayAccess`; copy propagation and branch joins preserve only
  proven facts. Property reads, method/call returns, clone/static-property
  values, reference-backed object slots, property-held/nested ArrayAccess,
  RMW, `empty`/`??`, exact diagnostics, cleanup/unwind, LLVM consumers, and
  backend parity remain open.
- `9aa933a8`: generated-C compiler consumption of the runtime ArrayAccess
  write/append/unset ABI for direct variable object-offset keyed assignment,
  append assignment, assignment-expression result, and unset when the compiler
  can prove the subject is a generated declared `ArrayAccess` object through
  the existing subject fact boundary. The executable proof covers two
  `ArrayAccess` classes, direct subjects, copied subjects, branch-joined
  subjects, string/integer/boolean/false/null-like offset behavior, expression
  replacement values, and assignment-expression result semantics. Dynamic
  producers, property-held/nested owners, compound/RMW, increment/decrement,
  null-coalesce assignment, `empty`, reference-returning `offsetGet`,
  references/COW, cleanup/unwind, exact diagnostics, LLVM consumers, and
  backend parity remain open.
- `4ddbfc47`: generated-C compiler consumption of the runtime ArrayAccess
  read/exists ABI for direct object offset read and `isset` when the compiler
  can prove a generated declared object implements built-in `ArrayAccess`.
  Dynamic producers, `empty`, null coalescing, compound/RMW,
  reference-returning `offsetGet`, references/COW, cleanup/unwind, exact
  diagnostics, LLVM consumers, and backend parity remain open.
- `3fb74a20`: receiver-free static `Class::method` string callable resolution
  through the shared runtime callable-value ABI and method descriptor table,
  plus generated-C frame-validation repair. Object receiver binding,
  non-static receiver-free method invocation, late static binding, namespace
  fallback, autoload, magic calls, exact diagnostics, cleanup/unwind
  execution, references/COW, and backend parity remain open.
- `ccb16eb0`: generalized cleanup/unwind requirement diagnostics/preflight.
  Actual unwinding, `Throwable` propagation, catch matching/binding, finally
  execution, destructor execution, object lifetime cleanup, exact diagnostic
  ordering, and backend parity remain open.
- `e17998ef`: generated-C native array-lvalue owner materialization/writeback
  for selected RMW families over local native arrays and active-symbol/global
  import reference-slot owners. Object/static property storage, ArrayAccess
  RMW dispatch, broad alias roots, references/COW, cleanup/unwind, exact
  diagnostics, LLVM consumers, and backend parity remain open.
- `0f4a8603` and `682f3aef`: runtime-only ArrayAccess read/exists and
  write/append/unset dispatch ABIs through object/interface metadata,
  callable-table lookup, bound receiver invocation, and native call-result
  plumbing. Compiler consumers are partial.
- `d2adc130`: generated declared methods are registered in the runtime
  callable table and wrapper frames bridge `NativeCallFrame` receiver/value/
  reference arguments into declared method frames.

## Active Roadmap Items

Primary-integrated capability and lane-local candidate work are separated
explicitly.

| Item | Primary Integrated | Candidate Readiness | Toward Full Feature | Status |
| --- | ---: | ---: | ---: | --- |
| ArrayAccess read/isset compiler consumer | **100%** `[####################]` | **100%** `[####################]` | **50%** `[##########----------]` | Integrated at `4ddbfc47` for generated-C direct object offset read and `isset` on compiler-known generated declared `ArrayAccess` objects. |
| ArrayAccess write/append/unset compiler consumer | **100%** `[####################]` | **100%** `[####################]` | **55%** `[###########---------]` | Integrated through `1a9f0a1c` for generated-C direct-variable keyed write, append, assignment-expression result, and unset over compiler-known generated declared `ArrayAccess` object values, with shared owner materialization/writeback replacing path-local lowering. |
| ArrayAccess `empty`/null-coalesce sequencing | **100%** `[####################]` | **100%** `[####################]` | **45%** `[#########-----------]` | Integrated at `4311df7e` for generated-C direct object offset `empty()` and `$aa[$key] ?? rhs` over compiler-known generated declared `ArrayAccess` object values, including known dynamic generated-declared class-name facts. |
| ArrayAccess RMW/null-coalesce assignment sequencing | **100%** `[####################]` | **100%** `[####################]` | **50%** `[##########----------]` | Integrated at `653a5918` for generated-C direct-variable compound assignment and `$aa[$key] ??= rhs` over compiler-known generated declared `ArrayAccess` object values using the shared owner/writeback boundary. Property-held/nested owners, append RMW, increment/decrement, reference-returning `offsetGet`, references/COW, cleanup/unwind, and backend parity remain open. |
| ReferenceSlot value owner facts | **100%** `[####################]` | **100%** `[####################]` | **45%** `[#########-----------]` | Integrated at `2ee642ff` for compiler-visible native reference handles, including by-reference closure capture promotion and `global` import roots feeding existing ArrayAccess write/RMW consumers through the shared owner source/commit boundary. Arbitrary alias roots, request/superglobal path facts, property-held references, closure callback fact transport, references/COW, and backend parity remain open. |
| Callable identity return summaries | **100%** `[####################]` | **100%** `[####################]` | **60%** `[############--------]` | Integrated through `05214fd4` for generated functions, declared instance methods, declared static methods, compiler-owned descriptor closures, known string callables, definite `__invoke` objects, compiler-known callable arrays, compiler-visible callable-array identity invalidation after native array owner mutation, and PHP-compatible declared-method external callable policy. Unknown runtime strings/arrays/objects/builtins, non-descriptor closures, recursive/fixed-point summaries, reference returns, property/magic producers, references/COW, and backend parity remain open. |
| Callable return producer facts | **100%** `[####################]` | **100%** `[####################]` | **60%** `[############--------]` | Integrated through `05214fd4` for generated function/method/static-method, descriptor-closure, known string/invokable, and compiler-known callable-array return summaries feeding existing object/interface fact consumers through the shared callable identity resolver, with stale variable-held callable-array identities cleared after compiler-visible native array mutations and method identity facts filtered by PHP-compatible declared-method receiver policy. Recursive/fixed-point summaries, unknown runtime callables/arrays, property/magic producers, references/COW, and backend parity remain open. |
| Dynamic object/interface fact carrier | **100%** `[####################]` | **100%** `[####################]` | **65%** `[#############-------]` | Integrated through `05214fd4` for generated-C native value/object facts over generated declared objects, known dynamic class-name `new`, copies, gotos, branch joins, generated-callable, descriptor-closure, known string/invokable, and compiler-known callable-array identity return summaries, including invalidation after compiler-visible native array mutations and PHP-compatible object receiver public static callable facts, and compiler-visible reference slots, consumed by ArrayAccess read/isset/write/append/unset/empty/null-coalesce/RMW/`??=`. Producers from properties, clones, static properties, arbitrary symbols, unknown runtime callables/arrays, and non-descriptor closures remain open. |
| Object/method callable receiver parity fallback | **0%** `[--------------------]` | **80%** `[################----]` | **40%** `[########------------]` | Route-audited fallback if ArrayAccess write/unset blocks. Not the active primary route. |
| Cleanup/unwind execution | **25%** `[#####---------------]` | **25%** `[#####---------------]` | **25%** `[#####---------------]` | Requirement/preflight boundary is integrated; actual unwind/finally/destructor execution is still not implemented. |
| Broad dirty lane extraction backlog | **0%** `[--------------------]` | **35%** `[#######-------------]` | **35%** `[#######-------------]` | Dirty call, diagnostic, object, control-flow, symbol, byte/string, and array lanes remain evidence pools until split into fresh current-head candidates. |

## Done / In Progress / Not Done

Primary-integrated capability:

- [x] Runtime callable table plus call arguments/frame/result ABI.
- [x] Runtime callable-value dispatch for selected string/binary function
  names, callable arrays, descriptor closures, inherited methods, bound
  receivers, and object `__invoke`.
- [x] Direct generated-C user-function calls and dynamic generated-C callee
  expressions through shared runtime callable lookup/invocation.
- [x] Generated declared-method callable-table registration and wrapper
  frames.
- [x] Receiver-free static `Class::method` string callable lookup through the
  runtime callable-value ABI.
- [x] Shared diagnostic operation/operand-list blocker boundary.
- [x] Reference-binding, assignment-lvalue, and RMW-lvalue operand-list
  requirement blockers.
- [x] Generated-C selected RMW array-lvalue owner/writeback for local native
  arrays and active-symbol/global-import reference-slot owners.
- [x] Cleanup/unwind requirement diagnostics/preflight.
- [x] Runtime ArrayAccess read/exists and write/append/unset dispatch ABIs.
- [x] Generated-C ArrayAccess direct offset read and `isset` compiler consumer
  for compiler-known generated declared `ArrayAccess` objects.
- [x] Generated-C ArrayAccess direct-variable keyed write, append,
  assignment-expression result, and unset compiler consumer for compiler-known
  generated declared `ArrayAccess` objects.
- [x] Shared generated-C direct-variable ArrayAccess owner materialization and
  writeback commit boundary for keyed write, append, unset, and
  assignment-expression writeback.
- [x] Shared generated-C native value/object facts for generated declared
  `ArrayAccess` objects, including known dynamic class-name `new`, copies, and
  branch joins consumed by read/isset/write/append/unset/empty/null-coalesce.
- [x] Generated-callable return-result facts for direct functions, instance
  methods, and static methods feeding shared object/interface fact consumers.
- [x] Shared callable identity and return-summary resolver for generated
  functions, declared instance methods, declared static methods, and
  compiler-owned descriptor closures.
- [x] Shared generated-C ReferenceSlot value owner source/commit and
  reference-cell fact ledger for compiler-visible native reference handles.
- [x] Generated-C ArrayAccess `empty()` and null-coalesce compiler consumers
  for compiler-known generated declared `ArrayAccess` objects.
- [x] Generated-C direct-variable ArrayAccess compound assignment and
  null-coalesce assignment compiler consumers for compiler-known generated
  declared `ArrayAccess` objects.
- [x] Declared-class allocation cleanup-risk metadata.
- [x] Selected reference-source/lvalue extraction, reference-backed closure
  capture materialization, descriptor closure returns, byte-backed PHP string
  values, and byte-preserving selected string-array slots.

Lane-local or currently routed, not counted:

- [ ] Broader object/interface fact carriers for producers beyond generated
  declared-object `new`, copies, branch joins, and direct generated-callable
  returns.
- [ ] Property-held/nested ArrayAccess owner, runtime dynamic callable,
  non-descriptor closure, and property/magic callable-return producer fact
  expansion.
- [ ] Object/method callable receiver parity fallback.
- [ ] Broad lane extraction into fresh current-head, owned-scope candidates.
- [ ] Pages repair blocked by gh-pages generated-output deletions.

Still not done:

- [ ] Dynamic ArrayAccess producers beyond known generated declared-class
  `new` and direct generated-callable return summaries: dynamic runtime
  callables, descriptor closures, properties, clone/static property values, and
  other object sources.
- [ ] Property-held and nested ArrayAccess writes/unsets, compound/RMW,
  null-coalesce assignment, append/increment forms, and reference-returning
  ArrayAccess semantics.
- [ ] Object/static/dynamic/typed property storage and full method/object
  model execution.
- [ ] Full reference/COW identity and arbitrary alias-root writeback.
- [ ] Actual exception/Throwable propagation, catch matching/binding,
  `finally`, destructors, shutdown cleanup, and object lifetime cleanup.
- [ ] Namespace fallback, autoload, class aliases, broader visibility, magic
  calls, constructors, named/spread breadth, and return references.
- [ ] Exact PHP diagnostics, source ordering, suppression/custom handlers, and
  backend parity across generated C, LLVM, and direct assembly.
