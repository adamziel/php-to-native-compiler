# PHP Native Compiler Progress

Updated: 2026-05-26 07:43 CEST
Evaluation marker: `20260526T040843Z`
Strategy evaluator marker: `20260526T040843Z`

Accounting rule: only generalized, tested, committed, and pushed primary work
counts as integrated capability. Dirty WIP, candidate worktrees, lane-local
claims, review-only work, failed prep proofs, probe-only commits, architecture
notes, and dashboard-only commits are excluded.

## Executive Read

Overall supervised-roadmap progress: **95%** `[###################-]`

Selected executable PHP semantics: **95%** `[###################-]`

Primary `HEAD` is clean and aligned with `origin/master` after accounting for
the latest source capability.

Latest primary-integrated source capability baseline:
`369099d7 native: add callable return producer facts`.

`369099d7` adds conservative generated-callable return-result facts to the
shared `CNativeValueFacts` carrier. Direct generated functions, instance
methods, and static methods can now produce object/interface facts from known
return summaries, with branch-return intersection and conservative clearing
for fallthrough, unknown, and non-mixed return cases.

This composes with `4ddbfc47` generated-C ArrayAccess read/`isset`,
`9aa933a8` generated-C ArrayAccess write/append/unset, `4311df7e`
generated-C ArrayAccess `empty()`/null-coalesce consumers, receiver-free static
`Class::method` string callable lookup, `f9b721a2` shared native value/object
facts for known generated declared ArrayAccess producers, `1a9f0a1c`
ArrayAccess owner writeback, `653a5918` ArrayAccess RMW/`??=`, cleanup/unwind
requirement preflight, selected generated-C RMW array-lvalue owner/writeback,
runtime ArrayAccess read/write dispatch ABIs, and generated declared-method
callable-table publication.

Lane-local momentum is active but not counted. Broader producer fact work,
property/nested owner design, dynamic callable identity summaries, and callable
receiver fallback remain advisory until routed, audited, integrated, committed,
and pushed.

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
| Runtime and ABI foundations | **99%** | `[####################]` | Strong selected-path value, byte-string, array, reference, symbol, callable table, callable-value dispatch, call-frame/result, request-state, diagnostics, lvalue, and ArrayAccess read/write dispatch surfaces. Remaining gaps include broader callable lookup parity, namespace fallback, autoload, magic calls, constructors, closure frame handoff, reference-return ArrayAccess, and cleanup/unwind parity. |
| Compiler/backend consumers | **99%** | `[####################]` | Generated C has the freshest executable consumers for direct/dynamic callables, declared methods, selected RMW array-lvalue owner/writeback, and compiler-known generated declared ArrayAccess read/isset/write/append/unset/empty/null-coalesce/RMW/`??=`, including known dynamic generated-declared class-name producers. LLVM and direct assembly still lag recent object offset and lvalue/runtime ABIs. |
| Executable PHP semantics | **95%** | `[###################-]` | Many selected executable islands exist, but major semantics remain open: full assignment/RMW/writeback, references/COW, executable object/ArrayAccess operations, cleanup/unwind/finally/destructors, exact diagnostics, and backend parity. |
| Strings and byte semantics | **62%** | `[############--------]` | Byte-backed values and byte-preserving selected string-array slots are integrated. Binary source bytes, byte-exact interpreter/session/debug output, `mb_str_split()`, request/global byte keys, and exact diagnostics remain open. |
| Arrays, lvalues, references, COW | **84%** | `[#################---]` | Selected reference-source/lvalue extraction, closure capture from reference-backed slots, reference-binding diagnostics, assignment/RMW-lvalue diagnostics, generated-C RMW array-lvalue owner/writeback, direct-variable ArrayAccess RMW/`??=`, and Object/ArrayAccess blocker/runtime dispatch pieces are integrated. Object/static property storage, property-held/nested ArrayAccess RMW, arbitrary alias roots, foreach, broader writeback, and full COW remain incomplete. |
| Symbols, globals, request state | **75%** | `[###############-----]` | Selected globals, root-symbol consumers, active symbol-table consumers, request-key blockers, append-shaped symbol reference-source materialization, direct generated-C request-state frame handoff, and generated-C dynamic user-function handoff proof exist. `$GLOBALS` self-cells, closure request-state handoff, request/global alias parity, request writeback, includes, variable variables, and exact unset/global behavior remain incomplete. |
| Calls, functions, frames | **95%** | `[###################-]` | Runtime callable table/value dispatch, call arguments/frame/result ABI, direct and dynamic generated-C callable consumers, generated declared-method callable registration/wrapper frames, receiver-free static `Class::method` strings, generated-callable return-result facts, by-reference argument transport, descriptor closures, closure returns, and generated-C request-state frame handoff are integrated. Dynamic runtime callable return summaries, full object/method callable parity, callable array validation parity, namespace fallback, autoload, magic calls, named/spread breadth, return references, constructors, closure frame handoff, cleanup/unwind execution, and backend parity remain open. |
| Objects, properties, methods | **63%** | `[#############-------]` | Public object-property reference-source extraction, object-property reference-slot mutation, declared-class allocation cleanup-risk metadata, Object/ArrayAccess write blockers, runtime ArrayAccess write/read/exists dispatch, generated-C ArrayAccess read/isset/write/append/unset/empty/null-coalesce/RMW/`??=` consumers for compiler-known generated objects, known dynamic generated-declared class-name producers, generated-callable return producers, and generated declared-method callable-table publication exist for selected paths. Property/magic/dynamic-call/clone/static-property producers, property-held/nested ArrayAccess owners, visibility parity, magic, dynamic/static/typed properties, destructors, interfaces/traits execution, references/COW, constructors, and backend parity remain open. |
| Control flow, cleanup, diagnostics | **54%** | `[###########---------]` | Selected branches, loops, transfers, finalizers, output buffers, diagnostics, try-body call-boundary preflight, generic operand-list blockers, reference/assignment/RMW blockers, Object/ArrayAccess write blockers, and cleanup/unwind requirement preflight exist. Broad unwind/finally/destructor/shutdown execution, cleanup ownership, executable reference binding, and source-ordered diagnostics remain open. |
| Broad integrated verification | **92%** | `[##################--]` | Focused gates around recent source work are strong. Broad verification is still constrained by lane extraction cost, stale candidate expectations, heavy swap usage, and backend parity gaps. |

## Recent Primary-Integrated Work

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
| Callable return producer facts | **100%** `[####################]` | **100%** `[####################]` | **50%** `[##########----------]` | Integrated at `369099d7` for generated function/method/static-method return summaries feeding existing object/interface fact consumers. Recursive/fixed-point summaries, dynamic runtime callables, descriptor closures, property/magic producers, references/COW, and backend parity remain open. |
| Dynamic object/interface fact carrier | **100%** `[####################]` | **100%** `[####################]` | **58%** `[############--------]` | Integrated through `369099d7` for generated-C native value/object facts over generated declared objects, known dynamic class-name `new`, copies, gotos, branch joins, and generated-callable returns, consumed by ArrayAccess read/isset/write/append/unset/empty/null-coalesce/RMW/`??=`. Producers from properties, clones, static properties, symbols, references, dynamic runtime callables, and descriptor closures remain open. |
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
- [ ] Property-held/nested ArrayAccess owner and callable-return producer fact
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
