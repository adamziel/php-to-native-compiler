# PHP Native Compiler Progress

Updated: 2026-05-26 00:36 CEST
Evaluation marker: `20260525T223657Z`

Accounting rule: only generalized, tested, committed, and pushed primary work
counts as integrated capability. Dirty WIP, candidate worktrees, lane-local
claims, review-only work, failed prep proofs, probe-only commits, architecture
notes, and dashboard-only commits are excluded.

## Executive Read

Overall estimated progress: **95%** `[###################-]`

Executable PHP semantics: **95%** `[###################-]`

Primary was clean and aligned with `origin/master` at
`e32f5735 native: add runtime callable ABI` before this `PROGRESS.md` edit.

Latest primary-integrated executable capability baseline:
`e32f5735 native: add runtime callable ABI`.

`e32f5735` landed the runtime-only callable table, call
arguments/result/frame ABI, and caller-scope visibility boundary. This is a
generalized prerequisite for dynamic callable-value dispatch and compiler
consumers, not complete callable execution. Overall and executable estimates
stay flat while the next call work moves from ABI foundation to compiler use.

## Grand Roadmap Position

| Workstream | Estimate | Bar | Current read |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | **99%** | `[####################]` | Strong selected-path value, byte-string, string-array, array, diagnostic, reference, symbol, call-frame, object, comparison, conversion, numeric-unary, request-state, closure-result, reference-source, and callable table/arguments/result/frame surfaces. The next missing callable foundation is value dispatch and compiler consumption through the integrated ABI. |
| Compiler/backend consumers | **99%** | `[####################]` | Generated C has the freshest executable semantics; LLVM and direct assembly still lag several recent semantic packets. No new compiler consumer landed in this review window. |
| Executable PHP semantics | **95%** | `[###################-]` | Primary has many selected executable islands, including reference-backed by-value closure capture materialization, closure value/reference returns, and reference-source append/lvalue extraction. |
| Strings and byte semantics | **62%** | `[############--------]` | Byte-backed values and byte-preserving `explode()` / `str_split()` slots are integrated. Binary source bytes, byte-exact interpreter/session/debug output, `mb_str_split()`, request/global byte keys, and exact diagnostics remain open. |
| Arrays, lvalues, references, COW | **82%** | `[################----]` | Selected reference-source/lvalue extraction and closure capture from reference-backed slots are integrated. Full COW, arbitrary alias roots, foreach, alias composition, static/magic/non-public properties, ArrayAccess, and broad writeback remain incomplete. |
| Symbols, globals, request state | **74%** | `[###############-----]` | Selected globals, root-symbol, active symbol-table reference consumers, request-key blockers, and append-shaped symbol reference-source materialization exist. `$GLOBALS` self-cells, request/global alias parity, request writeback, includes, variable variables, and exact unset/global behavior remain incomplete. |
| Calls, functions, frames | **89%** | `[##################--]` | Descriptor closures, selected captures, selected by-reference parameters, callable/function-table surfaces, method-frame surfaces, descriptor closure value/reference returns, reference-backed by-value captures, and a runtime callable table/arguments/result/frame ABI are integrated. General callable-value dispatch, compiler table registration/consumers, non-descriptor closures, methods, static calls, constructors, and argument binding breadth remain open. |
| Objects, properties, methods | **53%** | `[###########---------]` | Public object-property reference-source extraction and object-property reference-slot mutation exist for selected paths. Full visibility, magic, dynamic/static/typed properties, destructors, references/COW, and `ArrayAccess` execution remain open. |
| Control flow, cleanup, diagnostics | **51%** | `[##########----------]` | Selected branches, loops, transfers, finalizers, output buffers, diagnostics, truthiness, and conversion consumers exist. Broad unwind/finally/destructor/shutdown and exact source ordering remain open. Current broad diagnostic-lane work is not counted. |
| Broad integrated verification | **92%** | `[##################--]` | Recent focused gates are strong and nonzero. The full `native_runtime_abi` suite still has known current-primary failures, and broad gates remain constrained by lane extraction cost, stale lane expectations, high swap, and backend parity gaps. |

## Recent Primary-Integrated Work

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
| Dynamic callable runtime ABI / callable-value dispatch | **100%** `[####################]` | **47%** `[#########-----------]` | Runtime-only callable table, arguments/result/frame ownership, function/method/constructor invocation callbacks, called-scope propagation, and public/protected/private visibility checks are integrated at `e32f5735`. Callable-value dispatch, compiler table registration, and generated-C consumers remain lane-local/future work. |
| Reference-backed by-value closure captures | **100%** `[####################]` | **48%** `[##########----------]` | Integrated at `90e53401`. Covers descriptor closure value captures from ordinary locals, native reference handles, and active symbol-table storage across direct function, static method, receiver method, and closure factory frames. Non-static implicit `$this`, secondary-alias writeback, and broad callable/object semantics remain open. |
| Reference-source append/lvalue extraction | **100%** `[####################]` | **52%** `[##########----------]` | Integrated at `7aa162ca`. Covers selected symbol, native reference local, public object-property, object-property array-path, append, reference assignment, and by-reference call consumers. Static/magic/non-public properties, ArrayAccess, arbitrary alias roots, and full references/COW remain open. |
| Closure value/reference return ABI | **100%** `[####################]` | **50%** `[##########----------]` | Integrated at `ae93da8c`. Runtime closure invocation has a shared value/reference/diagnostic/status result contract for descriptor closures. Broader function/method/static/constructor reference returns remain open. |
| Diagnostic result callable/RMW/control scanner contracts | **0%** `[--------------------]` | **46%** `[#########-----------]` | `impl-native-error-diagnostic-semantics` continues producing broad lane-local contracts and scanner work. It remains dirty evidence, not primary progress; one visible status section is chronologically suspect relative to the evaluator clock. |
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

In progress but lane-local or not yet executable primary support:

- [ ] Dynamic callable compiler consumers must now use the integrated
  `e32f5735` runtime ABI: table registration, callable-value dispatch, and
  generated-C result/value/reference/discard consumers remain unintegrated.
- [ ] Three current-primary candidate packets are ready for review but not
  counted: dynamic constructor symbol-environment blockers,
  allocatable destructor-risk metadata, and try-body call routing. Each still
  needs independent review, current-primary apply proof, and focused gates.
- [ ] `impl-native-call-semantics` has broad dirty evidence for call ordering,
  dynamic constructor/class-name blockers, callable-value dispatch,
  destructor-risk metadata, source-call ordering, and object/call blockers.
  None counts until a fresh current-primary packet is reviewed and integrated.
- [ ] `impl-native-error-diagnostic-semantics` has broad dirty diagnostic
  result/scanner contracts. Useful evidence, but not primary progress and not a
  substitute for executable PHP semantics.
- [ ] Broader closure/call reference returns need reusable consumers beyond
  descriptor closures: user functions, methods, static calls, constructors,
  discarded calls, and non-descriptor closure surfaces.
- [ ] Broad parked lanes remain evidence only until exact current-primary prep,
  review, integration, and push.

Not done:

- [ ] Full references/COW identity, arbitrary alias roots, and alias-preserving
  write-through.
- [ ] Executable request storage/writeback, `$GLOBALS` self-cells,
  request/global alias parity, request foreach, and mutation-during-iteration
  behavior.
- [ ] Includes, variable variables, and dynamic symbol behavior.
- [ ] Full callable lookup and invocation, including named/unpacked/
  by-reference breadth, closures, arrays, invokable objects, magic/visibility,
  non-public methods, and rebinding rules.
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

- [x] Primary is clean and synced at `e32f5735` before this `PROGRESS.md` edit.
- [x] Latest executable capability head is `e32f5735`.
- [x] Overall and executable estimates remain 95% under current project-local
  accounting.
- [x] Closure capture from reference-backed storage remains a completed
  non-repeat item.
- [x] Runtime callable ABI is now a completed non-repeat prerequisite.

Lane-local:

- [x] Dynamic callable value-context prep correctly returned
  `needs-architecture`; the runtime-only ABI split from that blocker is now
  integrated.
- [ ] Next callable work should target callable-value dispatch and first
  compiler consumer packets over `e32f5735`, not another ABI-only inventory.
- [ ] Dynamic constructor symbol-environment, allocatable destructor-risk, and
  try-body call-routing candidates are ready-for-review artifacts only.
- [ ] Call-lane loop/call ordering work is fresh but broad and dirty.
- [ ] Diagnostic-lane callable operand, reference-binding, RMW, report dispatch,
  and control-flow scanner contracts are broad and dirty; status chronology is
  not fully reliable.
- [ ] Multiple broad lanes are evidence repositories. Do not route them to
  primary without a new narrow extraction.

Resource posture:

- `/dev/shm`: 40G total, 24G used, 17G available, 58% used.
  Largest observed targets: `phpc-target-native-call-semantics` 8.9G,
  `phpc-target-native-object-seed` 5.6G, and
  `phpc-target-native-diagnostics` 3.0G.
- `/home`: 459G total, 194G used, 247G available, 44% used. Largest observed
  lane/work tree: `phpc-lane-native-error-diagnostic-semantics` 14G.
- Memory available is about 39Gi, but swap remains high at 23Gi/29Gi used.
- Continue disk-backed `/tmp` target dirs, `umask 0007`,
  `CARGO_BUILD_JOBS=1`, `CARGO_INCREMENTAL=0`, and focused nonzero gates.

## Next Steering Read

Best next action:

- Review the ready current-primary candidate packets in priority order only
  after proving clean apply over `e32f5735` and any docs-only progress commit.
  Favor call/compiler consumer work that uses the integrated runtime callable
  ABI over cleanup-only diagnostic/scanner work.

Avoid:

- Percentage bumps for architecture notes, candidate creation, selector-only
  cleanup, diagnostic/scanner-only cleanup, future-dated status claims, or
  lane-local work.
- Routing broad dirty call/diagnostic lanes directly to primary.
- Repeating runtime-only callable table/arguments/result/frame ABI work,
  reference-backed closure capture materialization,
  reference-source append/lvalue extraction, descriptor closure result ABI,
  request-key selector cleanup, conversion helper cleanup, or unary negation ABI
  work under new names.
- Extending generated-C dynamic-call shape helpers instead of consuming the
  integrated runtime callable ABI.
