# PHP Native Compiler Progress

Updated: 2026-05-25 14:08 CEST
Evaluation marker: `20260525T120836Z`

Accounting rule: only generalized, tested, committed, and pushed primary work
counts as integrated capability. Dirty WIP, lane-local candidates, broad
worktree claims, probe-only commits, and dashboard-only commits are excluded.

## Executive Read

Overall estimated progress: **88%** `[##################--]`

Executable PHP semantics: **88%** `[##################--]`

Primary is clean and aligned with `origin/master` at
`9f373b25 native: route reference text membership slots` before this dashboard
edit. The latest counted semantic/prerequisite commit is now `9f373b25`.

The previously rejected text-membership/reference text-byte packet was repaired
and integrated. The landed slice routes reference-held and value-held
function/extension text-membership consumers through one shared slot boundary
while preserving the broader native known-function source and native-C
user-function membership.

This is useful primary progress, but it remains a selected-path compiler/runtime
boundary. Full generalized PHP remains blocked on references/COW identity,
arbitrary lvalues, request/global parity, includes, variable variables, broad
userland frames, real `ArrayAccess`, object semantics, cleanup/unwind/destructor
shutdown ordering, exact diagnostics/error handlers, and backend parity.

## Current Primary State

- Current primary head before this dashboard edit:
  `9f373b25 native: route reference text membership slots`.
- Latest integrated executable/prerequisite semantic baseline:
  `9f373b25 native: route reference text membership slots`.
- Prior integrated prerequisite:
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
| Runtime and ABI foundations | **99%** | `[####################]` | Strong selected-path value, array, string, diagnostic, reference, symbol, call-frame, object, comparison, conversion, owner-cell, request-state, offset-read, array-key, type/int, and text-membership slot surfaces. |
| Compiler/backend consumers | **99%** | `[####################]` | Generated C and LLVM consume many shared ABIs. Recent text-membership routing covers runtime/LLVM/generated C selected paths; broad backend parity remains incomplete. |
| Executable PHP semantics | **88%** | `[##################--]` | Primary has closure/callable/object islands, bounded preg callbacks, object-property reference-slot mutation, offset-read continuation proof, reference-backed array-key conversion, reference-backed type/int consumers, and reference-backed text-membership consumers. |
| Arrays, lvalues, references, COW | **74%** | `[###############-----]` | Value/reference slot ABI reuse is expanding. Full COW, arbitrary roots, foreach, property references, broad expression reference slots, and alias composition remain open. |
| Symbols, globals, request state | **72%** | `[##############------]` | Selected function globals, root-symbol surfaces, and active symbol-table reference consumers exist. `$GLOBALS` self-cells, request/global alias parity, includes, variable variables, and exact unset/global behavior remain incomplete. |
| Calls, functions, frames | **83%** | `[#################---]` | `function_exists()` now uses the repaired shared text-membership path in selected generated-native routes, including native-C user functions. Named/unpacked/by-reference/userland frame breadth remains incomplete. |
| Objects, properties, methods | **50%** | `[##########----------]` | Object-property reference-slot mutation and diagnostic classifiers are integrated. Full visibility, magic, dynamic/static/typed properties, destructors, references/COW, and `ArrayAccess` execution remain open. |
| Control flow, cleanup, diagnostics | **50%** | `[##########----------]` | Selected branches, loops, transfers, finalizers, output buffers, and diagnostics exist. Broad unwind/finally/destructor/shutdown and exact source ordering remain open. |
| Broad integrated verification | **86%** | `[#################---]` | Focused gates are strong and recent packets include nonzero runtime, IR, generated-C, linked executable, rustfmt, check, diff, and hash/apply proof. Broad gates remain constrained by lane extraction cost, high swap, stale lane expectations, and backend parity gaps. |

## Active Roadmap Items

| Item | Toward Primary Integration | Toward Full Feature | Status |
| --- | ---: | ---: | --- |
| Text-membership/reference text-byte conversion | **100%** `[####################]` | **44%** `[#########-----------]` | Integrated at `9f373b25`. Shared runtime value/reference text slot feeds `function_exists()` and `extension_loaded()` membership consumers; repaired source preserves native known functions and native-C user functions. Dynamic runtime environment discovery and LLVM user-function parity remain open. |
| Reference-slot type/int consumer ABI | **100%** `[####################]` | **45%** `[#########-----------]` | Integrated at `8f6266ce`. Runtime, LLVM, and generated C route reference-held type names, type predicates, and supported int operands through shared value/reference slots. Alias-aware write-through remains intentionally blocked. |
| Array-key value/reference-slot ABI | **100%** `[####################]` | **42%** `[########------------]` | Integrated at `9022eb9e`. Shared checked slot ABI for value/reference-backed array-key operands is in primary. Broader expression lvalues, COW identity, object/resource/Stringable keys, and direct `ArrayAccess` remain open. |
| Scalar/resource offset-read source-result prerequisite | **100%** `[####################]` | **44%** `[#########-----------]` | Integrated at `cc7efc2d`. Direct object `ArrayAccess`, object/resource materialization, and LLVM error-status cleanup remain open. |
| Object-property reference-slot mutation | **100%** `[####################]` | **39%** `[########------------]` | Integrated at `bfbc62c4`. Covered assignment/unset operands use shared value/reference slot handling. Full object/property/reference semantics remain open. |
| Bounded `preg_replace_callback()` string callbacks | **100%** `[####################]` | **32%** `[######--------------]` | Integrated at `6aca392d`. Full PCRE, broader captures/modifiers, non-string callables, `limit`/`count`/`flags`, and legacy recognizer cleanup remain open. |
| Object-offset `ArrayAccess` diagnostic classifier | **100%** `[####################]` | **12%** `[##------------------]` | Integrated at `deaf52ca`. Diagnostic routing only; no `offsetGet`, `offsetExists`, `offsetSet`, or `offsetUnset` execution. |
| Reference-slot comparison consumers | **35%** `[#######-------------]` | **37%** `[#######-------------]` | Lane-local work suggests a shared comparison slot ABI, but strict-identity/static arithmetic expectation churn and broader comparison parity must stay out of any compact packet. |
| String-array/string-position/string-result consumers | **40%** `[########------------]` | **38%** `[########------------]` | Useful lane-local work exists, including string-array/result expansion. It must be split from unrelated builtin breadth and proven as one reusable semantic boundary. |
| Broader lvalue/reference-slot materializer | **25%** `[#####---------------]` | **38%** `[########------------]` | Needed so non-variable expression families that can carry references can enter shared array-key and consumer slot ABIs safely. |
| Object/resource source materialization for shared conversion sources | **25%** `[#####---------------]` | **30%** `[######--------------]` | Explicit blocker left by the offset-read ABI. Needs a general value reconstruction boundary before generic object/resource consumers are safe. |
| LLVM offset-read/error-status cleanup | **25%** `[#####---------------]` | **30%** `[######--------------]` | Offset-read diagnostics exist, but LLVM still needs a generalized control-flow/error-exit status boundary for failed conversion results. |
| Static-property comparison operand ABI | **35%** `[#######-------------]` | **37%** `[#######-------------]` | Prior extraction says `needs-split`: source lane is too broad and entangled. Split metadata/operand prerequisites first. |
| Callable/dynamic constructor candidates | **52%** `[##########----------]` | **42%** `[########------------]` | May 24 lane-local candidates look useful but are stale relative to current primary `9f373b25`; refresh before any primary review. |
| Diagnostics, request, and cleanup boundaries | **60%** `[############--------]` | **40%** `[########------------]` | Lane-local request handle, writeback, branch cleanup, try/catch/finally preflight, stateful-call cleanup, and result-boundary work is useful infrastructure. Exact Zend ordering and real handler/exceptions execution remain open. |
| Broad lane extraction backlog | **34%** `[#######-------------]` | **34%** `[#######-------------]` | Broad dirty lanes continue producing useful surfaces, but many are blocker/preflight-only or adjacent builtin/callback expansions. Treat lanes as packet sources, not integration units. |

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
- [x] Shared reference-slot text-byte/text-membership ABI for runtime, LLVM, and generated C selected paths, including repaired native-C user-function membership for `function_exists()`.

Primary-integrated non-executable infrastructure:

- [x] Object-offset `ArrayAccess` receiver diagnostic classifier for read, append-read, null-coalesce, `isset`, `empty`, and error-control forms.
- [x] Symbol-table ABI probe is pushed, but remains probe-only until real assignment/readback consumers land.

In progress but lane-local or not yet executable primary support:

- [ ] Reference-slot comparison, scanner, and diagnostic consumers remain lane-local.
- [ ] String-array/string-position/string-result consumer families must be split from unrelated builtin breadth and proven separately.
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
- [ ] Binary-string scanner, error-handler dispatch, stream, class-alias, and broad internal-callback surfaces remain lane-local.
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

- `9f373b25`: reference text-membership slots. Integrated files:
  `runtime/src/lib.rs`, `compiler/src/codegen.rs`,
  `compiler/tests/native_runtime_abi.rs`, and `compiler/tests/native_link.rs`.
  Focused runtime, generated IR, generated-C source, linked executable,
  user-function introspection, `cargo check`, rustfmt, diff hygiene,
  apply/hash, and zero-match gates passed.
- `419ee981`: progress-dashboard commit only. No executable compiler/runtime
  semantic code changed.
- `caea4991`: progress-dashboard commit only. No executable compiler/runtime
  semantic code changed.
- `df8141eb`: progress-dashboard commit only. No executable compiler/runtime
  semantic code changed.
- `8f6266ce`: reference-slot type/int consumers. Integrated files:
  `runtime/src/lib.rs`, `compiler/src/codegen.rs`,
  `compiler/tests/native_runtime_abi.rs`, and `compiler/tests/native_link.rs`.
  Focused runtime, generated IR, generated-C source, linked executable,
  `cargo check`, rustfmt, diff hygiene, apply/hash, and zero-match gates passed.
- `9022eb9e`: array-key value/reference-slot ABI. Integrated files:
  `runtime/src/lib.rs`, `compiler/src/codegen.rs`, and
  `compiler/tests/native_link.rs`. Focused runtime, generated-C source, linked
  executable, `cargo check`, rustfmt, and diff hygiene gates passed.
- `cc7efc2d`: scalar/resource offset-read source-result ABI. Integrated files:
  `runtime/src/lib.rs`, `compiler/src/codegen.rs`, and
  `compiler/tests/native_runtime_abi.rs`. Focused runtime, generated-source,
  executable continuation, object-property composition, ArrayAccess rejection,
  `cargo check`, diff hygiene, and rustfmt gates passed.
- `bfbc62c4`: generated-C/native-link object-property assignment and unset
  mutation route subject, property, and replacement operands through shared
  value-or-reference slot handling with runtime dereference boundaries.
- `deaf52ca`: compiler classifies unsupported object-offset `ArrayAccess`
  receiver diagnostics through a shared operation result across covered
  read/probe/error-control families. This is not executable `ArrayAccess`.
- `6aca392d`: interpreter `preg_replace_callback()` executes supported string
  callbacks over a bounded slash-delimited pattern subset.

## Current Work Snapshot

Primary-integrated:

- [x] Primary was clean and synced at `9f373b25` before this dashboard edit.
- [x] Latest counted semantic/prerequisite commit is `9f373b25`.
- [x] Repaired reference-held text-membership consumers are integrated through
  the shared value/reference-slot ABI for runtime, LLVM, and generated C.
- [x] Native-C `function_exists()` membership includes registered user
  functions rather than only a narrowed native table.
- [x] Reference-held type-name/type-predicate/int consumers, array-key
  value/reference slots, and offset-read source-result support remain
  integrated for reviewed selected paths.

Lane-local:

- [ ] `reference-text-membership-slot-review.status.md` remains historically
  useful as the rejected first review; its blocker was repaired before
  integration.
- [ ] `impl-native-type-conversion` is broad and active. It has useful callback,
  comparison, and conversion surfaces, but they need compact extraction and
  independent review before any primary count.
- [ ] `impl-binary-string-runtime` remains broad; useful text/string surfaces
  must be extracted as compact packets, not imported directly.
- [ ] Reference-slot comparison consumers and string-array/string-position/
  string-result families are plausible next packet sources if split cleanly.
- [ ] Stale callable-object and dynamic-constructor candidates need
  current-primary refresh before review.

Resource posture:

- `/dev/shm`: `40G` total, `24G` used, `17G` available; `du` reports `24G`.
- `/home`: `459G` total, `218G` used, `223G` available; `du` reports `128G`.
- Snapshot memory had about `39Gi` available.
- Snapshot swap remained high at `23Gi/29Gi`; use disk-backed target dirs,
  `CARGO_BUILD_JOBS=1`, and focused nonzero gates.

## Next Steering Read

The next useful move is not more text-membership repair; that packet landed.
The next high-value primary candidate should be another compact reusable
boundary, not a broad builtin/callback batch. Good options to consider are:

- reference-slot comparison consumers, if extracted without strict-identity,
  static arithmetic expectation churn, or broad comparison parity;
- a narrow lvalue/reference-slot materializer prerequisite, if it proves
  multiple expression families share one slot boundary without adding a single
  fixture-shaped lowering;
- string-array/string-position/string-result work only if it is split as one
  operation-family boundary with runtime, IR, generated-C, and linked proof.

Do not count lane-local triage, broad source-lane work, stale May 24 candidates,
callback-family accumulation, blocker-only metadata, or docs-only commits as
product capability.
