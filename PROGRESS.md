# PHP Native Compiler Progress

Updated: 2026-05-26 06:37 CEST
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
`f9b721a2 native: add dynamic ArrayAccess producer facts`.

`f9b721a2` replaces the generated-C split ArrayAccess variable fact set with
reusable native value/object facts carrying declared-class and implemented-
interface metadata. ArrayAccess read, `isset`, write, append, and unset now
consume the same definite native object/interface fact query, including known
dynamic generated-declared class-name `new` producers, copy propagation, and
branch joins.

This composes with `4ddbfc47` generated-C ArrayAccess read/`isset`,
`9aa933a8` generated-C ArrayAccess write/append/unset, receiver-free static
`Class::method` string callable lookup, cleanup/unwind requirement preflight,
selected generated-C RMW array-lvalue owner/writeback, runtime ArrayAccess
read/write dispatch ABIs, and generated declared-method callable-table
publication.

Lane-local momentum is active but not counted. ArrayAccess `empty`/
null-coalesce sequencing, broader producer fact work, property/nested owner
design, ArrayAccess RMW/`??=` scouting, and callable receiver fallback remain
advisory until routed, audited, integrated, committed, and pushed.

The project continues moving through reusable runtime/compiler boundaries, but
this is not full PHP parity. Broader object/interface facts for properties,
calls, methods, clones, static properties, and references, property-held and
nested object offsets, ArrayAccess RMW, `empty`/`??`, references/COW,
destructors/finally/unwind execution, object/static property storage, exact
diagnostics, magic/autoload/name resolution, and backend parity remain major
gaps.

## Grand Roadmap Position

| Workstream | Estimate | Bar | Current read |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | **99%** | `[####################]` | Strong selected-path value, byte-string, array, reference, symbol, callable table, callable-value dispatch, call-frame/result, request-state, diagnostics, lvalue, and ArrayAccess read/write dispatch surfaces. Remaining gaps include broader callable lookup parity, namespace fallback, autoload, magic calls, constructors, closure frame handoff, reference-return ArrayAccess, and cleanup/unwind parity. |
| Compiler/backend consumers | **99%** | `[####################]` | Generated C has the freshest executable consumers for direct/dynamic callables, declared methods, selected RMW array-lvalue owner/writeback, and compiler-known generated declared ArrayAccess read/isset/write/append/unset, including known dynamic generated-declared class-name producers. LLVM and direct assembly still lag recent object offset and lvalue/runtime ABIs. |
| Executable PHP semantics | **95%** | `[###################-]` | Many selected executable islands exist, but major semantics remain open: full assignment/RMW/writeback, references/COW, executable object/ArrayAccess operations, cleanup/unwind/finally/destructors, exact diagnostics, and backend parity. |
| Strings and byte semantics | **62%** | `[############--------]` | Byte-backed values and byte-preserving selected string-array slots are integrated. Binary source bytes, byte-exact interpreter/session/debug output, `mb_str_split()`, request/global byte keys, and exact diagnostics remain open. |
| Arrays, lvalues, references, COW | **83%** | `[#################---]` | Selected reference-source/lvalue extraction, closure capture from reference-backed slots, reference-binding diagnostics, assignment/RMW-lvalue diagnostics, generated-C RMW array-lvalue owner/writeback, and Object/ArrayAccess blocker/runtime dispatch pieces are integrated. Object/static property storage, ArrayAccess RMW dispatch, arbitrary alias roots, foreach, broader writeback, and full COW remain incomplete. |
| Symbols, globals, request state | **75%** | `[###############-----]` | Selected globals, root-symbol consumers, active symbol-table consumers, request-key blockers, append-shaped symbol reference-source materialization, direct generated-C request-state frame handoff, and generated-C dynamic user-function handoff proof exist. `$GLOBALS` self-cells, closure request-state handoff, request/global alias parity, request writeback, includes, variable variables, and exact unset/global behavior remain incomplete. |
| Calls, functions, frames | **94%** | `[###################-]` | Runtime callable table/value dispatch, call arguments/frame/result ABI, direct and dynamic generated-C callable consumers, generated declared-method callable registration/wrapper frames, receiver-free static `Class::method` strings, by-reference argument transport, descriptor closures, closure returns, and generated-C request-state frame handoff are integrated. Full object/method callable parity, callable array validation parity, namespace fallback, autoload, magic calls, named/spread breadth, return references, constructors, closure frame handoff, cleanup/unwind execution, and backend parity remain open. |
| Objects, properties, methods | **60%** | `[############--------]` | Public object-property reference-source extraction, object-property reference-slot mutation, declared-class allocation cleanup-risk metadata, Object/ArrayAccess write blockers, runtime ArrayAccess write/read/exists dispatch, generated-C ArrayAccess read/isset/write/append/unset consumers for compiler-known generated objects and known dynamic generated-declared class-name producers, and generated declared-method callable-table publication exist for selected paths. Property/call/method/clone/static-property producers, property-held/nested ArrayAccess, `empty`/null-coalesce/RMW lowering, visibility parity, magic, dynamic/static/typed properties, destructors, interfaces/traits execution, references/COW, constructors, and backend parity remain open. |
| Control flow, cleanup, diagnostics | **54%** | `[###########---------]` | Selected branches, loops, transfers, finalizers, output buffers, diagnostics, try-body call-boundary preflight, generic operand-list blockers, reference/assignment/RMW blockers, Object/ArrayAccess write blockers, and cleanup/unwind requirement preflight exist. Broad unwind/finally/destructor/shutdown execution, cleanup ownership, executable reference binding, and source-ordered diagnostics remain open. |
| Broad integrated verification | **92%** | `[##################--]` | Focused gates around recent source work are strong. Broad verification is still constrained by lane extraction cost, stale candidate expectations, heavy swap usage, and backend parity gaps. |

## Recent Primary-Integrated Work

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
| ArrayAccess write/append/unset compiler consumer | **100%** `[####################]` | **100%** `[####################]` | **52%** `[##########----------]` | Integrated at `9aa933a8` for generated-C direct-variable keyed write, append, assignment-expression result, and unset over compiler-known generated declared `ArrayAccess` object values, including copied and branch-joined subject facts. |
| ArrayAccess `empty`/null-coalesce sequencing | **0%** `[--------------------]` | **75%** `[###############-----]` | **38%** `[########------------]` | Lane-local candidate has focused gates and lazy sequencing claims. It overlaps the write/unset route and should wait unless explicitly reprioritized. |
| Dynamic object/interface fact carrier | **100%** `[####################]` | **100%** `[####################]` | **52%** `[##########----------]` | Integrated at `f9b721a2` for generated-C native value/object facts over generated declared objects, known dynamic class-name `new`, copies, gotos, and branch joins, consumed by ArrayAccess read/isset/write/append/unset. Producers from properties, calls, methods, clones, static properties, symbols, and references remain open. |
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
- [x] Shared generated-C native value/object facts for generated declared
  `ArrayAccess` objects, including known dynamic class-name `new`, copies, and
  branch joins consumed by read/isset/write/append/unset.
- [x] Declared-class allocation cleanup-risk metadata.
- [x] Selected reference-source/lvalue extraction, reference-backed closure
  capture materialization, descriptor closure returns, byte-backed PHP string
  values, and byte-preserving selected string-array slots.

Lane-local or currently routed, not counted:

- [ ] ArrayAccess `empty`/null-coalesce lazy probe/read sequencing candidate.
- [ ] Broader object/interface fact carriers for producers beyond generated
  declared-object `new`, copies, and branch joins.
- [ ] Object/method callable receiver parity fallback.
- [ ] Broad lane extraction into fresh current-head, owned-scope candidates.
- [ ] Pages repair blocked by gh-pages generated-output deletions.

Still not done:

- [ ] Dynamic ArrayAccess producers beyond known generated declared-class
  `new`: calls, method results, properties, clone/static property values, and
  other object sources.
- [ ] Property-held and nested ArrayAccess writes/unsets, compound/RMW,
  null-coalesce assignment, and reference-returning ArrayAccess semantics.
- [ ] Object/static/dynamic/typed property storage and full method/object
  model execution.
- [ ] Full reference/COW identity and arbitrary alias-root writeback.
- [ ] Actual exception/Throwable propagation, catch matching/binding,
  `finally`, destructors, shutdown cleanup, and object lifetime cleanup.
- [ ] Namespace fallback, autoload, class aliases, broader visibility, magic
  calls, constructors, named/spread breadth, and return references.
- [ ] Exact PHP diagnostics, source ordering, suppression/custom handlers, and
  backend parity across generated C, LLVM, and direct assembly.
