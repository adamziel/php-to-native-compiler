# PHP Native Compiler Progress

Updated: 2026-05-26 16:22 CEST
Evaluation marker: `20260526T040843Z`
Strategy evaluator marker: `20260526T040843Z`

Accounting rule: only generalized, tested, committed, and pushed primary work
counts as integrated capability. Dirty WIP, candidate worktrees, lane-local
claims, review-only work, failed prep proofs, probe-only commits, architecture
notes, and dashboard-only commits are excluded.

## Executive Read

Overall supervised-roadmap progress: **95%** `[###################-]`

Selected executable PHP semantics: **95%** `[###################-]`

Primary tracked `HEAD` is aligned with `origin/master` after accounting for the
latest source capability. The worktree still has preserved foreign scratch
state: untracked `examples/class.php`.

Latest primary-integrated source capability baseline:
`7fb9db15 native: share object metadata preflight diagnostics`.

`7fb9db15` adds a shared
`NativeObjectMetadataCallOperation` /
`NativeObjectMetadataCallPreflightFailure` boundary for native
metadata/type-introspection builtin families. Class/interface/trait/enum
existence, property/method existence, and relationship metadata calls now
classify their semantic family once, check call-result argument dependencies
before arity, and project failures through the existing backend-neutral
`NativeCallDiagnostics` path for both LLVM and generated C. Existing metadata
execution remains intentionally narrow; the patch centralizes rejection
selection without importing lane-local object-property WIP or widening
production lowering. Focused proof covers dependency-before-arity and arity
preflight across `class_exists`, `property_exists`, and `is_a`, LLVM/generated-C
diagnostic projection, adjacent direct-call cleanup diagnostics, current
builtin-class metadata rejection, `cargo check -p phpc --lib`, fmt, and diff
checks. The older static metadata IR snapshot gate still fails on stale stdout
helper text (`phpc_native_value_echo_stdout` versus
`phpc_native_value_format_stdout_with_diagnostic`) and was recorded as a
non-gate; the affected metadata fixtures still emit IR successfully. Full
object metadata tables, object-property rejection carriers, receiver/static
receiver execution, visibility/magic/typed-property policy, references/COW,
exact diagnostics, and backend parity remain open.

This follows `a3826e2f`, which adds the runtime ABI helper
`phpc_native_value_dynamic_method_name_matches()` and routes generated-C dynamic
instance method dispatch through it. Dynamic method-name matching now uses a
method-specific scalar-normalization boundary instead of the string-only
dynamic function-call matcher: string and binary-string names compare
case-insensitively, int/true/float operands normalize through PHP echo-string
conversion for lookup and diagnostics, and false/null/empty/non-scalar values
remain centralized misses or diagnostics. Focused proof covers runtime method
lookup normalization, dynamic method failure diagnostics, generated-C helper
selection, fmt, and diff checks. Broader method-frame lookup, inherited/magic
method fallback, visibility policy, executable object/property assignment,
references/COW, exact diagnostics, and backend parity remain open.

This follows `3ac78d8b`, which adds
`NativeObjectCallDispatchArgumentHandles` as the shared generated-C
argument-handle materialization boundary for object call dispatch.
Declared constructor calls, declared instance methods, dynamic declared instance
method branches, object/static receiver calls, named static methods,
callable-array method branches, invokable-object branches, and constructorless
argument arrays now retain call arguments, owned `NativeValue` handles, and
cleanup through one frame-argument carrier instead of repeated tuple/local
aggregation. The extraction intentionally leaves LLVM object/class parity and
broader policy-check ABI work blocked until the missing method-frame,
receiver/static receiver, lifecycle, visibility/magic/typed-property,
references/COW, and exact diagnostic surfaces exist.

This follows `73195f96`, which routes generated-C cast lowering through the shared
`phpc_native_value_cast_result()` / `NativeValueOperationResult` carrier.
Array-to-string casts now produce the PHP-shaped value `"Array"` while carrying
`Warning: Array to string conversion` through the same value-result diagnostic
path used by unary, binary, comparison, and type-name operations. The legacy
diagnostic-pointer cast helper remains available for runtime callers but now
delegates through the shared result boundary, keeping generated C and runtime
callers aligned. Focused proof covers runtime cast results, legacy helper
compatibility, generated C source routing, linked executable warning/output
behavior for `(string)` and `strval()` over array values, and neighboring cast
ABI consumers.

This follows `0bebd2e9`, which adds a reusable runtime/compiler boundary for
produced call results
supplied to fixed by-reference user-function parameters. Generated C now
detects direct declared user-function calls where a by-reference parameter is
fed by a produced call result, packages all arguments as owned
`NativeCallResultHandle` values, and routes them through
`phpc_native_call_frame_reference_parameter_alias_transfer_result_from_results_with_diagnostic()`.
The helper preserves prior argument diagnostics, records caller-visible target
labels and value families, nulls consumed result handles, and reports the
missing executable alias-transfer semantics through the shared call-result
consumer path instead of using a backend-local rejection.

This follows `273c2e6e`, which adds generated-C scoped callable-string signature planning for
known `Class::method` strings. Variable-held, concatenated, ternary, and
short-ternary finite string sets can now resolve declared public static-method
metadata before runtime callable-value invocation, so by-reference argument
planning uses the shared `NativeCallArgumentsHandle` carrier instead of the
older finite function-string ladder. Known by-reference public static method
returns can satisfy reference-assignment consumers through the runtime
callable-value reference path, while known non-reference returns stay on the
central dynamic-call return-ownership blocker.

This follows `b3d90dbc`, which adds shared runtime/compiler reference-cell predicate and membership
boundaries. Reference-backed `isset`, `empty`, and truthiness now route through
`phpc_native_reference_predicate()` instead of cloning the reference value just
to ask null/truthiness questions. `array_key_exists()` and `key_exists()` now
share diagnostic-aware value and reference-cell membership helpers, so LLVM and
generated-C consumers can handle direct values, aliases, symbol/reference slots,
and by-reference foreach cells through one key-coercion and array-membership
surface.

`73a9c58d` tightens runtime call-result ownership by
routing value, reference, and discard consumers through one diagnostic-aware
call-result boundary. Stale
diagnostic slots are cleared before a consumer inspects even a null
`NativeCallResultHandle`, so missing-result and failure-result cleanup no longer
leaks an unrelated prior diagnostic through direct callable and callable-value
invocation wrappers.

`05214fd4` aligns compiler-known declared-method callable
identity facts with PHP receiver policy. Class-method strings and class-string
callable arrays now publish facts only for public static methods. Object
receiver callable arrays can publish facts for public instance and public static
methods, matching PHP's `[$object, "staticMethod"]` behavior. Invokable object
facts reject static `__invoke` metadata while preserving PHP's current
non-public `__invoke` warning-but-callable behavior.

`4dc4d791` keeps compiler-known callable-array identities honest after native
array owner mutations. Variable-backed callable arrays can still publish facts
when stable, but direct element writes, append/unset, compound/update writes,
null-coalesce lvalue writes, by-reference foreach over native array owners, and
native array mutating builtins now clear those identities before later dynamic
calls can consume stale class/method pairs.

This builds on `7aa27530`, which extended the same dynamic-call return-fact
boundary to compiler-known callable arrays. Literal/static callable arrays such
as `["ClassName", "method"]`, keyed numeric forms such as
`[1 => "method", 0 => $object]`, and object receiver arrays such as
`[$object, "method"]` resolve through the shared callable identity and
return-summary intersection path. Callable-array identities survive ordinary
variable assignment, so `$cb = ["Class", "method"]; $cb()` can publish known
native object/interface facts for downstream consumers until a compiler-visible
mutation invalidates them. The resolver stays conservative: by-reference array
elements, unknown array shapes or runtime mutations, arity mismatches,
by-reference returns, and mixed receiver sets where any possible class lacks the
method do not publish facts.

This composes with `7fb9db15` object metadata preflight diagnostics,
`a3826e2f` dynamic method-name normalization,
`73195f96` native value-result cast diagnostics,
`0bebd2e9` by-reference alias-transfer result diagnostics,
`273c2e6e` scoped callable-string signatures, `b3d90dbc`
reference-cell predicate/membership helpers, `73a9c58d` call-result diagnostic
cleanup, `05214fd4` callable method identity policy, `4dc4d791`
callable-array identity invalidation, `7aa27530` callable-array return facts,
`d2b60ba7` known string/invokable dynamic callable return facts, `4ddbfc47`
generated-C
ArrayAccess read/`isset`,
`9aa933a8` generated-C ArrayAccess write/append/unset, `4311df7e`
generated-C ArrayAccess `empty()`/null-coalesce consumers, receiver-free static
`Class::method` string callable lookup, `f9b721a2` shared native value/object
facts for known generated declared ArrayAccess producers, `1a9f0a1c`
ArrayAccess owner writeback, `653a5918` ArrayAccess RMW/`??=`, `369099d7`
callable return producer facts, `6caaa387` callable identity return summaries,
`67efa804` descriptor-closure return summaries, `2ee642ff` ReferenceSlot owner
facts, cleanup/unwind requirement preflight, selected generated-C RMW
array-lvalue owner/writeback, runtime ArrayAccess read/write dispatch ABIs,
generated declared-method callable-table publication, native value-result cast
diagnostics, shared generated-C object-call argument handles, and
method-specific generated-C dynamic method-name matching.

Lane-local momentum is active but not counted. The object metadata preflight,
dynamic method-name normalization, object-call argument-handle,
array-cast value-result, scoped callable-string, and function-frame
by-reference alias-transfer routes have now been primary-integrated. The fresh
property-held ArrayAccess, ordered symbol diagnostic cleanup, interface parent
descriptor, method lookup candidate-miss, and call-preflight source-aware
lanes are parked or pending as blocker maps.
Broader producer fact work, property/nested owner design, unknown runtime
callable consumers, runtime array-shape callable identity facts, builtin return
summaries, and callable receiver fallback remain advisory until routed,
audited, integrated, committed, and pushed.

The project continues moving through reusable runtime/compiler boundaries, but
this is not full PHP parity. Broader object/interface facts for properties,
calls, methods, clones, static properties, and references, property-held and
nested object offsets, ArrayAccess append/increment/reference-returning and
property-held/nested RMW/`??=` shapes, references/COW,
destructors/finally/unwind execution, object/static property storage, exact
diagnostics, magic/autoload/name resolution, and backend parity remain major
gaps.

## Grand Roadmap Position

| Workstream | Estimate | Bar | Current read |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | **99%** | `[####################]` | Strong selected-path value, byte-string, array, reference, symbol, callable table, callable-value dispatch, call-frame/result, native value-result cast diagnostics, produced-result by-reference alias-transfer diagnostics, request-state, diagnostics, lvalue, and ArrayAccess read/write dispatch surfaces, including diagnostic cleanup for null call-result consumers and shared reference-cell predicate/membership helpers. Remaining gaps include executable alias transfer, broader callable lookup parity, namespace fallback, autoload, magic calls, constructors, closure frame handoff, reference-return ArrayAccess, and cleanup/unwind parity. |
| Compiler/backend consumers | **99%** | `[####################]` | Generated C has the freshest executable consumers for direct/dynamic callables, declared methods, selected dynamic method-name normalization, selected native object-metadata preflight diagnostics, selected known scoped `Class::method` callable-string signatures, shared generated-C object-call argument handles, selected known string/invokable/callable-array callable return facts, selected reference-cell predicates and `array_key_exists`/`key_exists` membership, selected native value-result cast diagnostics including array-to-string warnings, selected RMW array-lvalue owner/writeback, and compiler-known generated declared ArrayAccess read/isset/write/append/unset/empty/null-coalesce/RMW/`??=`, including known dynamic generated-declared class-name producers. LLVM and direct assembly still lag recent object offset and lvalue/runtime ABIs. |
| Executable PHP semantics | **95%** | `[###################-]` | Many selected executable islands exist, but major semantics remain open: full assignment/RMW/writeback, references/COW, executable object/ArrayAccess operations, cleanup/unwind/finally/destructors, exact diagnostics, and backend parity. |
| Strings and byte semantics | **62%** | `[############--------]` | Byte-backed values and byte-preserving selected string-array slots are integrated. Binary source bytes, byte-exact interpreter/session/debug output, `mb_str_split()`, request/global byte keys, and exact diagnostics remain open. |
| Arrays, lvalues, references, COW | **85%** | `[#################---]` | Selected reference-source/lvalue extraction, closure capture from reference-backed slots, ReferenceSlot value-owner facts/commit, reference-cell predicates, reference/value array-key membership, reference-binding diagnostics, assignment/RMW-lvalue diagnostics, generated-C RMW array-lvalue owner/writeback, direct-variable and selected reference-slot ArrayAccess RMW/`??=`, and Object/ArrayAccess blocker/runtime dispatch pieces are integrated. Object/static property storage, property-held/nested ArrayAccess RMW, arbitrary alias roots, foreach, broader writeback, and full COW remain incomplete. |
| Symbols, globals, request state | **75%** | `[###############-----]` | Selected globals, root-symbol consumers, active symbol-table consumers, request-key blockers, append-shaped symbol reference-source materialization, direct generated-C request-state frame handoff, and generated-C dynamic user-function handoff proof exist. `$GLOBALS` self-cells, closure request-state handoff, request/global alias parity, request writeback, includes, variable variables, and exact unset/global behavior remain incomplete. |
| Calls, functions, frames | **96%** | `[###################-]` | Runtime callable table/value dispatch, call arguments/frame/result ABI, direct and dynamic generated-C callable consumers, generated declared-method callable registration/wrapper frames, method-specific dynamic method-name normalization, shared generated-C object-call argument handles, receiver-free static `Class::method` strings, scoped callable-string public static method signatures with by-reference argument planning and selected reference-return consumers, generated-callable return-result facts, produced-call-result by-reference alias-transfer diagnostics for direct user-function calls, callable identity return-summary resolution with PHP-compatible external declared-method policy, descriptor-closure return summaries, known string callable, callable-array, and definite `__invoke` object return facts, by-reference argument transport, descriptor closures, closure returns, generated-C request-state frame handoff, and null call-result diagnostic cleanup are integrated. Unknown runtime callable strings/objects/arrays, builtin return summaries, executable by-reference alias transfer, full object/method callable parity, namespace fallback, autoload, magic calls, named/spread breadth, broader return references, constructors, cleanup/unwind execution, and backend parity remain open. |
| Objects, properties, methods | **65%** | `[#############-------]` | Public object-property reference-source extraction, object-property reference-slot mutation, declared-class allocation cleanup-risk metadata, shared native object-metadata preflight diagnostics, Object/ArrayAccess write blockers, runtime ArrayAccess write/read/exists dispatch, generated-C ArrayAccess read/isset/write/append/unset/empty/null-coalesce/RMW/`??=` consumers for compiler-known generated objects and selected reference slots, known dynamic generated-declared class-name producers, generated-callable, descriptor-closure, known string callable, callable-array, definite `__invoke` object return producers, shared generated-C object-call argument handles, PHP-compatible object receiver public static callable facts, and generated declared-method callable-table publication exist for selected paths. Property/magic/unknown-runtime-dynamic-call/clone/static-property producers, property-held/nested ArrayAccess owners, broader visibility parity, magic, dynamic/static/typed properties, destructors, interfaces/traits execution, references/COW, constructors, and backend parity remain open. |
| Control flow, cleanup, diagnostics | **54%** | `[###########---------]` | Selected branches, loops, transfers, finalizers, output buffers, diagnostics, try-body call-boundary preflight, generic operand-list blockers, reference/assignment/RMW blockers, Object/ArrayAccess write blockers, and cleanup/unwind requirement preflight exist. Broad unwind/finally/destructor/shutdown execution, cleanup ownership, executable reference binding, and source-ordered diagnostics remain open. |
| Broad integrated verification | **92%** | `[##################--]` | Focused gates around recent source work are strong. Broad verification is still constrained by lane extraction cost, stale candidate expectations, heavy swap usage, and backend parity gaps. |

## Recent Primary-Integrated Work

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
| ArrayAccess read/isset compiler consumer | **100%** `[####################]` | **100%** `[####################]` | **48%** `[##########----------]` | Integrated at `4ddbfc47` for generated-C direct object offset read and `isset` on compiler-known generated declared `ArrayAccess` objects. |
| ArrayAccess write/append/unset compiler consumer | **100%** `[####################]` | **100%** `[####################]` | **54%** `[###########---------]` | Integrated through `1a9f0a1c` for generated-C direct-variable keyed write, append, assignment-expression result, and unset over compiler-known generated declared `ArrayAccess` object values, with shared owner materialization/writeback replacing path-local lowering. |
| ArrayAccess `empty`/null-coalesce sequencing | **100%** `[####################]` | **100%** `[####################]` | **46%** `[#########-----------]` | Integrated at `4311df7e` for generated-C direct object offset `empty()` and `$aa[$key] ?? rhs` over compiler-known generated declared `ArrayAccess` object values, including known dynamic generated-declared class-name facts. |
| ArrayAccess RMW/null-coalesce assignment sequencing | **100%** `[####################]` | **100%** `[####################]` | **52%** `[##########----------]` | Integrated at `653a5918` for generated-C direct-variable compound assignment and `$aa[$key] ??= rhs` over compiler-known generated declared `ArrayAccess` object values using the shared owner/writeback boundary. Property-held/nested owners, append RMW, increment/decrement, reference-returning `offsetGet`, references/COW, cleanup/unwind, and backend parity remain open. |
| ReferenceSlot value owner facts | **100%** `[####################]` | **100%** `[####################]` | **45%** `[#########-----------]` | Integrated at `2ee642ff` for compiler-visible native reference handles, including by-reference closure capture promotion and `global` import roots feeding existing ArrayAccess write/RMW consumers through the shared owner source/commit boundary. Arbitrary alias roots, request/superglobal path facts, property-held references, closure callback fact transport, references/COW, and backend parity remain open. |
| Callable identity return summaries | **100%** `[####################]` | **100%** `[####################]` | **61%** `[############--------]` | Integrated through `05214fd4` for generated functions, declared instance methods, declared static methods, compiler-owned descriptor closures, known string callables, definite `__invoke` objects, compiler-known callable arrays, compiler-visible callable-array identity invalidation after native array owner mutation, and PHP-compatible declared-method external callable policy. Unknown runtime strings/arrays/objects/builtins, non-descriptor closures, recursive/fixed-point summaries, reference returns, property/magic producers, references/COW, and backend parity remain open. |
| Callable return producer facts | **100%** `[####################]` | **100%** `[####################]` | **61%** `[############--------]` | Integrated through `05214fd4` for generated function/method/static-method, descriptor-closure, known string/invokable, and compiler-known callable-array return summaries feeding existing object/interface fact consumers through the shared callable identity resolver, with stale variable-held callable-array identities cleared after compiler-visible native array mutations and method identity facts filtered by PHP-compatible declared-method receiver policy. Recursive/fixed-point summaries, unknown runtime callables/arrays, property/magic producers, references/COW, and backend parity remain open. |
| Dynamic object/interface fact carrier | **100%** `[####################]` | **100%** `[####################]` | **66%** `[#############-------]` | Integrated through `05214fd4` for generated-C native value/object facts over generated declared objects, known dynamic class-name `new`, copies, gotos, branch joins, generated-callable, descriptor-closure, known string/invokable, and compiler-known callable-array identity return summaries, including invalidation after compiler-visible native array mutations and PHP-compatible object receiver public static callable facts, and compiler-visible reference slots, consumed by ArrayAccess read/isset/write/append/unset/empty/null-coalesce/RMW/`??=`. Producers from properties, clones, static properties, arbitrary symbols, unknown runtime callables/arrays, and non-descriptor closures remain open. |
| Object/method callable receiver parity fallback | **0%** `[--------------------]` | **80%** `[################----]` | **42%** `[########------------]` | Route-audited fallback if ArrayAccess write/unset blocks. Not the active primary route. |
| Cleanup/unwind execution | **25%** `[#####---------------]` | **25%** `[#####---------------]` | **25%** `[#####---------------]` | Requirement/preflight boundary is integrated; actual unwind/finally/destructor execution is still not implemented. |
| Broad dirty lane extraction backlog | **0%** `[--------------------]` | **35%** `[#######-------------]` | **36%** `[#######-------------]` | Dirty call, diagnostic, object, control-flow, symbol, byte/string, and array lanes remain evidence pools until split into fresh current-head candidates. |

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
