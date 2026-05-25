# PHP Native Compiler Progress

Updated: 2026-05-25 13:49 CEST
Evaluation marker: `20260525T114934Z`

Accounting rule: only generalized, tested, committed, and pushed primary work
counts as integrated capability. Dirty WIP, lane-local candidates, broad
worktree claims, probe-only commits, and dashboard-only commits are excluded.

## Executive Read

Overall estimated progress: **87%** `[#################---]`

Executable PHP semantics: **87%** `[#################---]`

Primary is clean and aligned with `origin/master` at
`caea4991 docs: update progress dashboard` before this dashboard edit. The
latest counted semantic/prerequisite commit remains
`8f6266ce native: route reference slot type and int consumers`.

No new semantic compiler/runtime capability landed in this review window. The
important current movement is lane-local: the reference text-membership slot
candidate completed extraction and focused gates, then failed independent review
with `needs-edits` because its `function_exists()` production path narrowed the
existing known/user-function semantics. It is useful candidate work, but it is
not primary capability yet.

The project continues to advance when packets land through reusable
runtime/compiler ABI boundaries. Full generalized PHP remains blocked on
references/COW identity, arbitrary lvalues, request/global parity, includes,
variable variables, broad userland frames, real `ArrayAccess`, object semantics,
cleanup/unwind/destructor/shutdown ordering, exact diagnostics/error handlers,
and backend parity.

## Current Primary State

- Current primary head before this dashboard edit:
  `caea4991 docs: update progress dashboard`.
- Latest integrated executable/prerequisite semantic baseline:
  `8f6266ce native: route reference slot type and int consumers`.
- Prior integrated prerequisite:
  `9022eb9e native: add array key reference slot ABI`.
- Prior integrated prerequisite:
  `cc7efc2d native: add offset read source result ABI`.
- Prior integrated executable object/reference feature:
  `bfbc62c4 native: route object property reference slots`.
- Latest integrated non-executable classifier:
  `deaf52ca codegen: classify object ArrayAccess receivers`.

## Grand Roadmap Position

| Workstream | Estimate | Bar | Current read |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | **99%** | `[####################]` | Strong selected-path value, array, string, diagnostic, reference, symbol, call-frame, object, comparison, conversion, owner-cell, request-state, offset-read, array-key, and type/int slot surfaces. |
| Compiler/backend consumers | **99%** | `[####################]` | Generated C and LLVM consume many shared ABIs. Recent reference-slot type/int routing covers both, but broad backend parity remains incomplete. |
| Executable PHP semantics | **87%** | `[#################---]` | Primary has closure/callable/object islands, bounded preg callbacks, object-property reference-slot mutation, offset-read continuation proof, reference-backed array-key conversion, and reference-backed type/int consumers. Broad semantics remain incomplete. |
| Arrays, lvalues, references, COW | **73%** | `[###############-----]` | Array-key and type/int reference-slot consumers are integrated. Full COW, arbitrary roots, foreach, property references, broad expression reference slots, and alias composition remain open. |
| Symbols, globals, request state | **72%** | `[##############------]` | Selected function globals, root-symbol surfaces, and active symbol-table reference consumers exist. `$GLOBALS` self-cells, request/global alias parity, includes, variable variables, and exact unset/global behavior remain incomplete. |
| Calls, functions, frames | **82%** | `[################----]` | Bounded closures, callable arrays/objects, public method frames, and constructors exist in selected paths. Named/unpacked/by-reference/userland frame breadth remains incomplete. |
| Objects, properties, methods | **50%** | `[##########----------]` | Object-property reference-slot mutation and diagnostic classifiers are integrated. Full visibility, magic, dynamic/static/typed properties, destructors, references/COW, and `ArrayAccess` execution remain open. |
| Control flow, cleanup, diagnostics | **50%** | `[##########----------]` | Selected branches, loops, transfers, finalizers, output buffers, and diagnostics exist. Broad unwind/finally/destructor/shutdown and exact source ordering remain open. |
| Broad integrated verification | **85%** | `[#################---]` | Focused gates are strong. Broad gates remain constrained by lane extraction cost, high swap, stale lane expectations, and backend parity gaps. |

## Active Roadmap Items

| Item | Toward Primary Integration | Toward Full Feature | Status |
| --- | ---: | ---: | --- |
| Reference-slot type/int consumer ABI | **100%** `[####################]` | **45%** `[#########-----------]` | Integrated at `8f6266ce`. Runtime, LLVM, and generated C route reference-held type names, type predicates, and supported int operands through shared value/reference slots. Alias-aware write-through remains intentionally blocked. |
| Array-key value/reference-slot ABI | **100%** `[####################]` | **42%** `[########------------]` | Integrated at `9022eb9e`. Shared checked slot ABI for value/reference-backed array-key operands is in primary. Broader expression lvalues, COW identity, object/resource/Stringable keys, and direct `ArrayAccess` remain open. |
| Scalar/resource offset-read source-result prerequisite | **100%** `[####################]` | **44%** `[#########-----------]` | Integrated at `cc7efc2d`. Direct object `ArrayAccess`, object/resource materialization, and LLVM error-status cleanup remain open. |
| Object-property reference-slot mutation | **100%** `[####################]` | **39%** `[########------------]` | Integrated at `bfbc62c4`. Covered assignment/unset operands use shared value/reference slot handling. Full object/property/reference semantics remain open. |
| Bounded `preg_replace_callback()` string callbacks | **100%** `[####################]` | **32%** `[######--------------]` | Integrated at `6aca392d`. Full PCRE, broader captures/modifiers, non-string callables, `limit`/`count`/`flags`, and legacy recognizer cleanup remain open. |
| Object-offset `ArrayAccess` diagnostic classifier | **100%** `[####################]` | **12%** `[##------------------]` | Integrated at `deaf52ca`. Diagnostic routing only; no `offsetGet`, `offsetExists`, `offsetSet`, or `offsetUnset` execution. |
| Text-membership/reference text-byte conversion | **55%** `[###########---------]` | **36%** `[#######-------------]` | Candidate extraction completed with stable hash, apply proof, and nonzero focused gates, but independent review returned `needs-edits` because `function_exists()` narrowed the existing known/user-function semantic source. Not countable. |
| Reference-slot comparison consumers | **35%** `[#######-------------]` | **37%** `[#######-------------]` | Lane-local work suggests a shared comparison slot ABI, but strict-identity/static arithmetic expectations and broader comparison parity must stay out of any compact packet. |
| String-array/string-position/string-result consumers | **40%** `[########------------]` | **38%** `[########------------]` | Useful lane-local work exists, including string-array/result expansion. It must be split from unrelated builtin breadth and proven as one reusable semantic boundary. |
| Broader lvalue/reference-slot materializer | **25%** `[#####---------------]` | **38%** `[########------------]` | Needed so non-variable expression families that can carry references can enter shared array-key and consumer slot ABIs safely. |
| Object/resource source materialization for shared conversion sources | **25%** `[#####---------------]` | **30%** `[######--------------]` | Explicit blocker left by the offset-read ABI. Needs a general value reconstruction boundary before generic object/resource consumers are safe. |
| LLVM offset-read/error-status cleanup | **25%** `[#####---------------]` | **30%** `[######--------------]` | Offset-read diagnostics exist, but LLVM still needs a generalized control-flow/error-exit status boundary for failed conversion results. |
| Static-property comparison operand ABI | **35%** `[#######-------------]` | **37%** `[#######-------------]` | Prior extraction says `needs-split`: source lane is too broad and entangled. Split metadata/operand prerequisites first. |
| Callable/dynamic constructor candidates | **52%** `[##########----------]` | **42%** `[########------------]` | May 24 lane-local candidates look useful but are stale relative to current primary `caea4991`; refresh before any primary review. |
| Diagnostics, request, and cleanup boundaries | **60%** `[############--------]` | **40%** `[########------------]` | Lane-local request handle, writeback, branch cleanup, try/catch/finally preflight, stateful-call cleanup, and result-boundary work is useful infrastructure. Exact Zend ordering and real handler/exceptions execution remain open. |
| Broad lane extraction backlog | **33%** `[#######-------------]` | **33%** `[#######-------------]` | Broad dirty lanes continue producing useful surfaces, but many are blocker/preflight-only and not directly integrable. Treat lanes as packet sources, not integration units. |

## Done / In Progress / Not Done

Primary-integrated executable or executable-prerequisite capability:

- [x] Descriptor-backed by-value closure invocation.
- [x] Direct by-value closure captures and non-static arrow implicit captures.
- [x] Untyped by-reference descriptor closure parameters.
- [x] Supported root/reference and promoted frame-local by-reference captures.
- [x] Typed/default/variadic by-value descriptor closure parameters.
- [x] Static anonymous descriptor closures and static arrow closures.
- [x] Non-static closure `$this` binding inside active object frames.
- [x] Callable-array invocation for supported public static/object method frames.
- [x] Callable-object invocation through supported public `__invoke` frames.
- [x] Runtime string-valued declared-class `new` for selected declared classes.
- [x] Destructor-observable declared-class allocation is blocked before unsafe generated-C native allocation.
- [x] Bounded public declared-object properties, methods, statics, constructors, named `instanceof`, and same-family aggregate equality.
- [x] Bounded `preg_replace_callback()` string-callback execution over supported slash-delimited patterns.
- [x] Object-property assignment/unset mutation for covered reference-backed operands through generated-C/native-link shared slot boundaries.
- [x] Shared offset-read source-result ABI for scalar/resource warning continuations, arrays, byte strings, references, and object-property offset-source composition.
- [x] Shared array-key value/reference-slot ABI for generated-native reference-backed variable operands and active symbol-table variable references.
- [x] Shared reference-slot type-name/type-predicate/int consumer ABI for runtime, LLVM, and generated C.

Primary-integrated non-executable infrastructure:

- [x] Object-offset `ArrayAccess` receiver diagnostic classifier for read, append-read, null-coalesce, `isset`, `empty`, and error-control forms.
- [x] Symbol-table ABI probe is pushed, but remains probe-only until real assignment/readback consumers land.

In progress but lane-local or not yet executable primary support:

- [ ] Text-membership/reference text-byte conversion has a reviewed candidate, but it needs edits before another primary review.
- [ ] String-array/string-position/string-result consumer families must be split from unrelated builtin breadth and proven separately.
- [ ] Reference-slot comparison, scanner, and diagnostic consumers remain lane-local.
- [ ] Direct object `ArrayAccess` method dispatch remains blocked behind diagnostic-only classifier support.
- [ ] Broader expression-family lvalue/reference-slot materialization is needed beyond variable-backed operands.
- [ ] Alias-aware LLVM direct-root write-through after `=&` remains blocked for both statement assignment and assignment expressions.
- [ ] Object/resource source materialization for generic conversion sources remains blocked.
- [ ] LLVM offset-read error-status cleanup needs a generalized control-flow boundary.
- [ ] Static-property comparison operands need a smaller prerequisite split before primary review.
- [ ] Callable-object and dynamic-constructor candidates need current-primary refresh before review.
- [ ] Function-frame signature-table blockers, method-table lookup blockers, and object string/Countable preflight work remain lane-local.
- [ ] Symbol/control-flow try/catch/finally preflight and rejecting-statement result boundaries remain lane-local infrastructure.
- [ ] Request-backed throw/clone/instanceof blockers and diagnostic writeback/selection boundaries remain lane-local.
- [ ] Binary-string scanner, error-handler dispatch, stream, and class-alias surfaces remain lane-local.
- [ ] Control-flow loop/switch/goto cleanup-state advances remain lane-local.

Not done:

- [ ] Full callable lookup and invocation, including non-string preg callbacks, closures, arrays, invokable objects, magic/visibility, and rebinding rules.
- [ ] Runtime `ArrayAccess` method dispatch for `offsetGet`, `offsetExists`, `offsetSet`, and `offsetUnset`.
- [ ] Full references/COW identity and arbitrary alias roots.
- [ ] Request and `$GLOBALS` parity, includes, variable variables, and dynamic symbol behavior.
- [ ] Full PCRE behavior beyond the bounded slash-delimited subset.
- [ ] Retirement or reframing of unrelated legacy WordPress-named preg/database recognizers behind generalized PHP semantic boundaries.
- [ ] General object model: non-public methods, overrides, interfaces/traits execution, magic methods, dynamic/static/typed properties, destructors.
- [ ] Complete cleanup/unwind/finally/destructor/output-buffer shutdown behavior.
- [ ] Exact/source-ordered diagnostics, custom handler execution, warning/error continuation, and suppression parity.
- [ ] LLVM/direct assembly parity for recent generated-C semantics.

## Recent Primary-Integrated Work

- `caea4991`: progress-dashboard commit only. No executable compiler/runtime semantic code changed.
- `df8141eb`: progress-dashboard commit only. No executable compiler/runtime semantic code changed.
- `8f6266ce`: reference-slot type/int consumers. Integrated files: `runtime/src/lib.rs`, `compiler/src/codegen.rs`, `compiler/tests/native_runtime_abi.rs`, and `compiler/tests/native_link.rs`. Focused runtime, generated IR, generated-C source, linked executable, `cargo check`, rustfmt, diff hygiene, apply/hash, and zero-match gates passed.
- `bfcf77bf`: progress-dashboard commit only. No executable compiler/runtime semantic code changed.
- `9022eb9e`: array-key value/reference-slot ABI. Integrated files: `runtime/src/lib.rs`, `compiler/src/codegen.rs`, and `compiler/tests/native_link.rs`. Focused runtime, generated-C source, linked executable, `cargo check`, rustfmt, and diff hygiene gates passed.
- `cc7efc2d`: scalar/resource offset-read source-result ABI. Integrated files: `runtime/src/lib.rs`, `compiler/src/codegen.rs`, and `compiler/tests/native_runtime_abi.rs`. Focused runtime, generated-source, executable continuation, object-property composition, ArrayAccess rejection, `cargo check`, diff hygiene, and rustfmt gates passed.
- `bfbc62c4`: generated-C/native-link object-property assignment and unset mutation route subject, property, and replacement operands through shared value-or-reference slot handling with runtime dereference boundaries.
- `deaf52ca`: compiler classifies unsupported object-offset `ArrayAccess` receiver diagnostics through a shared operation result across covered read/probe/error-control families. This is not executable `ArrayAccess`.
- `6aca392d`: interpreter `preg_replace_callback()` executes supported string callbacks over a bounded slash-delimited pattern subset.

## Current Work Snapshot

Primary-integrated:

- [x] Primary was clean and synced at `caea4991` before this dashboard edit.
- [x] Latest counted semantic/prerequisite commit remains `8f6266ce`.
- [x] Reference-held type-name/type-predicate/int consumers are integrated through the shared value/reference-slot ABI for runtime, LLVM, and generated C.
- [x] Array-key value/reference-slot and offset-read source-result support remain integrated for reviewed selected paths.

Lane-local:

- [ ] `reference-text-membership-slot-extract.status.md` completed and produced candidate hash `e226cf0ad7671427118ba6c9e8dcf7debfedabe321447f43bf0978ea0e1b395b` with focused gates.
- [ ] `reference-text-membership-slot-review.status.md` returned `needs-edits`: `function_exists()` must preserve the existing broader native/user-function availability semantics.
- [ ] `impl-binary-string-runtime` remains broad; useful text/string surfaces must be extracted as compact packets, not imported directly.
- [ ] Stale callable-object and dynamic-constructor candidates need current-primary refresh before review.
- [ ] `primary-integrator` should remain idle until a repaired packet has independent `go-for-primary-integrator` review.

Resource posture:

- `/dev/shm`: `40G` total, `24G` used, `17G` available.
- `/home`: `459G` total, `205G` used, `235G` available.
- Memory has about `39Gi` available.
- Swap remains high at `23Gi/29Gi`; use disk-backed target dirs, `CARGO_BUILD_JOBS=1`, and focused nonzero gates.

## Next Steering Read

The next useful move is to repair the text-membership/reference text-byte
candidate rather than sending it to primary integration. A repaired candidate
should prove:

- `function_exists()` uses the same effective semantic source as the existing compiler path;
- user-defined functions are still represented where the current path represents them;
- native candidate coverage has not narrowed relative to the existing known-function source;
- the value/reference text-byte slot ABI still has runtime, IR, generated-C source, and linked executable proof;
- current-primary apply proof, rustfmt, `cargo check`, `git diff --check`, stable diff hash, and zero-test audit all pass.

Do not count lane-local triage, pending repairs, stale May 24 candidates, broad
source-lane work, or docs-only commits as product capability.
