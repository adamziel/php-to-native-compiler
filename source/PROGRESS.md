# PHP Native Compiler Progress

Updated: 2026-05-26 02:00 CEST
Evaluation marker: `20260525T235833Z`

Accounting rule: only generalized, tested, committed, and pushed primary work
counts as integrated capability. Dirty WIP, candidate worktrees, lane-local
claims, review-only work, proof-only tests, architecture notes, and
dashboard-only commits are excluded.

## Executive Read

Overall estimated progress: **95%** `[###################-]`

Executable PHP semantics: **95%** `[###################-]`

Primary observed clean and synced with `origin/master` at:
`984229e9 docs: account reference handle typedef guard`.

Latest primary-integrated source capability baseline:
`501a4fb8 native: centralize reference handle C typedef gating`.

This review keeps percentages flat. `501a4fb8` is useful integrated primary
work, but it is a generated-C/native-link ABI declaration-order repair, not a
new executable PHP semantic surface. The next percentage-moving call/callable
work likely depends on integrating the reviewed callable runtime semantics
repair and then consuming that repaired runtime boundary from the compiler.

## Grand Roadmap Position

| Workstream | Estimate | Bar | Current read |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | **99%** | `[####################]` | Strong selected-path runtime surfaces exist for values, strings, arrays, diagnostics, references, symbols, call frames, callable tables/results, closure results, request-state selectors, and callable-value dispatch. Remaining gaps include full callable lookup parity, autoload, magic calls, constructors, request/global frame execution, cleanup/unwind, and broader object/reference behavior. |
| Compiler/backend consumers | **99%** | `[####################]` | Generated C has the freshest executable semantics. LLVM and assembly lag recent semantic packets. Direct user-function calls now consume the callable ABI, but dynamic callable compiler consumers, request/global frame handoff, exceptions, destructors, and backend parity remain open. |
| Executable PHP semantics | **95%** | `[###################-]` | Many executable islands are integrated, especially around direct calls, selected closures, references/lvalues, byte strings, selected arrays, and diagnostics. The remaining gap is broad composition across real PHP programs rather than isolated paths. |
| Strings and byte semantics | **62%** | `[############--------]` | Byte-backed values and byte-preserving `explode()` / `str_split()` slots are integrated. Binary source bytes, byte-exact interpreter/session/debug output, `mb_str_split()`, request/global byte keys, and exact diagnostics remain open. |
| Arrays, lvalues, references, COW | **82%** | `[################----]` | Selected reference-source/lvalue extraction and reference-backed closure captures are integrated. Full COW, arbitrary alias roots, foreach, alias composition, static/magic/non-public properties, ArrayAccess, and broad writeback remain incomplete. |
| Symbols, globals, request state | **74%** | `[###############-----]` | Selected globals, root-symbol, active symbol-table reference consumers, request-key blockers, and append-shaped symbol reference-source materialization exist. `$GLOBALS` self-cells, request/global alias parity, request writeback, includes, variable variables, and exact unset/global behavior remain incomplete. |
| Calls, functions, frames | **91%** | `[##################--]` | Runtime callable table, callable-value dispatch, direct generated-C user-function ABI consumption, descriptor closures, selected by-reference parameters, closure return ABI, symbol-environment constructor blockers, and try-body call preflight are integrated. Compiler dynamic callable consumption, `Class::method` strings, namespace fallback, autoload, magic calls, named/spread arguments, constructor execution, request/global frame separation, and cleanup/unwind parity remain open. |
| Objects, properties, methods | **53%** | `[###########---------]` | Public object-property reference-source extraction, object-property reference-slot mutation, and declared-class allocation cleanup-risk metadata exist for selected paths. Full visibility, magic, dynamic/static/typed properties, destructor execution and ordering, references/COW, and ArrayAccess execution remain open. |
| Control flow, cleanup, diagnostics | **51%** | `[##########----------]` | Selected branches, loops, transfers, finalizers, output buffers, diagnostics, truthiness, conversion consumers, and try/catch/finally body call-boundary preflight diagnostics exist. Broad unwind/finally/destructor/shutdown execution and exact source ordering remain open. |
| Broad integrated verification | **92%** | `[##################--]` | Recent focused gates are strong and nonzero. Full broad gates remain constrained by extraction cost, stale lane expectations, backend parity gaps, and high swap. |

## Recent Primary-Integrated Work

- `501a4fb8`: centralized generated-C `phpc_NativeReferenceHandle` typedef
  gating behind `uses_native_reference_handle_type()`, covering current
  reference-handle helper families including callable ABI helpers. This repairs
  declaration-before-use ordering for native-link paths; it does not implement
  request/global frame execution, compiler dynamic callable consumption, or new
  PHP runtime semantics.
- `5abf8525`: added runtime callable-value dispatch over the callable table and
  call-arguments/frame/result ABI for string and binary-string function names,
  callable arrays, descriptor-backed closures, inherited methods, bound object
  receivers, and object `__invoke`. A shadow audit found narrow runtime repair
  needs around called scope, protected visibility, and descriptor closure
  argument modes.
- `f0cc17c1`: routed direct generated-C user-function calls through the runtime
  callable table, lookup, call arguments, call frame, and call result ABI across
  zero/fixed/default/variadic calls and by-reference argument transport.
- `b400a23d`: added declared-class allocation metadata for destructor-observable
  cleanup risk, consumed by allocatable registration and dynamic constructor
  allocation checks.
- `6f7d550d`: routed try/catch/finally body call operations through the shared
  call-boundary diagnostic preflight before generic unsupported-try rejection.
- `ea0c7675`: routed dynamic constructor class-name operands from PHP symbol
  environments to the shared global-frame-separation blocker before constructor
  planning.
- `e32f5735`: added the runtime callable table and shared
  call arguments/result/frame ABI for function, method, and constructor
  callable kinds.

## Primary-Integrated Capability

| Capability | Primary Integration | Toward Full Feature | Status |
| --- | ---: | ---: | --- |
| Native reference-handle typedef gating | **100%** `[####################]` | **35%** `[#######-------------]` | Integrated at `501a4fb8`. Fixes generated-C declaration ordering for current reference-handle helper families. Broader request/global and references/COW execution remain open. |
| Runtime callable ABI, callable-value dispatch, and direct user-function consumers | **100%** `[####################]` | **58%** `[############--------]` | Integrated across `e32f5735`, `f0cc17c1`, and `5abf8525`. Known runtime repair is reviewed but not integrated. Compiler dynamic callable consumers remain unimplemented. |
| Allocatable destructor-observable cleanup-risk metadata | **100%** `[####################]` | **24%** `[#####---------------]` | Integrated at `b400a23d`. Destructor execution, lifetime cleanup, shutdown/finally ordering, trait-composed destructor metadata, runtime class lookup, and broad object semantics remain unimplemented. |
| Try/catch/finally body call-boundary preflight | **100%** `[####################]` | **18%** `[####----------------]` | Integrated at `6f7d550d`. Exception execution, catch matching, unwinding, finally ordering, and callable-value compiler dispatch remain open. |
| Dynamic constructor symbol-environment blockers | **100%** `[####################]` | **22%** `[####----------------]` | Integrated at `ea0c7675`. Request/global frame separation execution remains unimplemented. |
| Reference-backed by-value closure captures | **100%** `[####################]` | **48%** `[##########----------]` | Integrated at `90e53401`. Non-static implicit `$this`, secondary-alias writeback, and broad callable/object semantics remain open. |
| Reference-source append/lvalue extraction | **100%** `[####################]` | **52%** `[##########----------]` | Integrated at `7aa162ca`. Static/magic/non-public properties, ArrayAccess, arbitrary alias roots, and full references/COW remain open. |
| Closure value/reference return ABI | **100%** `[####################]` | **50%** `[##########----------]` | Integrated at `ae93da8c`. Broader function/method/static/constructor reference returns remain open. |

## Lane-Local Candidate Work

These items are current evidence, not counted primary capability.

| Candidate / lane | Toward Primary Integration | Counted Capability | Current read |
| --- | ---: | ---: | --- |
| Callable-value dispatch runtime semantics repair | **90%** `[##################--]` | **0%** `[--------------------]` | Formal and shadow reviews are `go-for-primary-integrator` / `shadow-go-for-primary-review`. Candidate repairs called-scope propagation, symmetric protected visibility, and descriptor-aware closure argument shaping in `runtime/src/lib.rs`. Needs fresh integration and push. |
| Dynamic callable compiler consumer | **15%** `[###-----------------]` | **0%** `[--------------------]` | Prep correctly stopped with `needs-runtime-repair`; no patch exists. After the runtime repair lands, compiler dynamic calls should consume the shared callable-value dispatch ABI instead of extending the legacy generated-C branch ladder. |
| Diagnostic operation-list blocker | **80%** `[################----]` | **0%** `[--------------------]` | Prep and shadow are ready for review/integration, but this is diagnostic/blocker infrastructure rather than executable PHP semantics. Recheck after callable runtime changes because both touch `runtime/src/lib.rs`. |
| Request/global direct frame handoff | **25%** `[#####---------------]` | **0%** `[--------------------]` | The reference-handle prerequisite has landed. Current useful state is a map; the candidate with no patch should be restarted from current primary as a small direct-call ABI handoff packet. |
| Symbol-env global-frame contract unit proof | **70%** `[##############------]` | **0%** `[--------------------]` | Shadow review is positive, but this is proof-only unit coverage for diagnostic operation routing, not request/global execution. |
| Trait effective method metadata | **15%** `[###-----------------]` | **0%** `[--------------------]` | Architecture/test-scout evidence only. Needs a real generalized metadata implementation before apply review. |

## Done / In Progress / Not Done

Primary-integrated executable or executable-prerequisite capability:

- [x] Runtime callable table and call arguments/result/frame ABI.
- [x] Runtime callable-value dispatch over the callable table.
- [x] Direct generated-C user-function calls through the callable ABI.
- [x] Centralized native reference-handle typedef gating for current helper
  families.
- [x] Shared closure value-capture materialization for reference-backed locals,
  native reference handles, and active symbol-table storage.
- [x] Shared reference-source/lvalue materialization for selected symbol paths,
  native local reference variables, public object-property sources, append
  paths, by-reference call argument extraction, and supported
  reference-assignment consumers.
- [x] Shared closure invocation result ABI for descriptor closure value returns,
  reference returns, value-consumer reference cloning, and reference-assignment
  binding.
- [x] Byte-backed PHP string value representation and byte-preserving
  `explode()` / `str_split()` slots.
- [x] Shared request-backed array-key/RMW blockers and request-state operation
  selector cleanup for existing consumers.
- [x] Shared try/catch/finally body call-boundary preflight diagnostics.
- [x] Shared declared-class allocation cleanup-risk metadata.

In progress but lane-local or not yet executable primary support:

- [ ] Callable-value dispatch runtime semantics repair is reviewed but not yet
  integrated.
- [ ] Dynamic callable compiler consumption is blocked until the runtime repair
  lands.
- [ ] Diagnostic operation-list blocker ABI is ready evidence but not counted
  semantic execution.
- [ ] Request/global direct frame handoff needs fresh prep now that the
  reference-handle typedef prerequisite has landed.
- [ ] Broader closure/call reference returns need consumers beyond descriptor
  closures: user functions, methods, static calls, constructors, discarded
  calls, and non-descriptor closure surfaces.

Not done:

- [ ] Full references/COW identity, arbitrary alias roots, and alias-preserving
  write-through.
- [ ] Executable request storage/writeback, `$GLOBALS` self-cells,
  request/global alias parity, request foreach, and mutation-during-iteration
  behavior.
- [ ] Includes, variable variables, and dynamic symbol behavior.
- [ ] Full callable lookup and invocation: compiler dynamic callable consumers,
  `Class::method` strings, namespace fallback, autoload, magic calls,
  named/spread arguments, by-reference variadic breadth, return references,
  constructor execution, rebinding rules, and cleanup/unwind parity.
- [ ] Runtime `ArrayAccess` method dispatch for `offsetGet`, `offsetExists`,
  `offsetSet`, and `offsetUnset`.
- [ ] Binary literal syntax, invalid-UTF-8 PHP source parsing, byte-exact
  diagnostics, and byte-exact request/global keys.
- [ ] Full object model: visibility, magic methods/properties, dynamic/static/
  typed properties, trait-composed method metadata, destructor execution, and
  object lifetime ordering.
- [ ] Exception execution, catch matching, finally/unwind ordering, shutdown
  functions, output-buffer ordering, and exact diagnostic timing/suppression.
- [ ] LLVM/direct assembly parity with the freshest generated-C semantics.
