# PHP Native Compiler Progress

Updated: 2026-05-25 17:57 CEST
Evaluation marker: `20260525T155753Z`

Accounting rule: only generalized, tested, committed, and pushed primary work
counts as integrated capability. Dirty WIP, candidate worktrees, lane-local
claims, probe-only commits, review-only work, and dashboard-only commits are
excluded.

## Executive Read

Overall estimated progress: **90%** `[##################--]`

Executable PHP semantics: **90%** `[##################--]`

Primary is clean and aligned with `origin/master` at
`1c369d0f native: add byte-backed PHP string value boundary` before this
dashboard edit. This is a real integrated prerequisite: PHP string values now
have a byte-backed representation boundary with native pointer-plus-length
materialization and linked executable byte-output proof.

This does **not** mean full byte-string semantics are done. Binary literal
syntax, invalid-UTF-8 source parsing, byte-exact tree-walk interpreter output,
request/global byte-key coercion, string-array builtin families, and full
request/global parity remain open.

Full generalized PHP remains blocked on references/COW identity, arbitrary
lvalues, request/global parity, includes, variable variables, broad userland
frames, real `ArrayAccess`, object/magic/visibility/destructor semantics,
cleanup/unwind/finally/shutdown ordering, exact diagnostics/error handlers, and
backend parity.

## Primary-Integrated Baseline

- Current primary head before this dashboard edit:
  `1c369d0f native: add byte-backed PHP string value boundary`.
- Current head type: semantic/prerequisite compiler-runtime commit.
- Latest integrated executable/prerequisite semantic baseline:
  `1c369d0f native: add byte-backed PHP string value boundary`.
- Recent integrated prerequisite:
  `a501c4d1 native: block request-backed array keys`.
- Recent integrated prerequisite:
  `24ec4a10 native: route reference truthiness slots`.
- Recent integrated prerequisite:
  `146c2d64 native: route reference comparison slots`.
- Recent integrated prerequisite:
  `9f373b25 native: route reference text membership slots`.
- Recent integrated prerequisite:
  `8f6266ce native: route reference slot type and int consumers`.
- Recent integrated prerequisite:
  `9022eb9e native: add array key reference slot ABI`.
- Recent integrated prerequisite:
  `cc7efc2d native: add offset read source result ABI`.
- Recent integrated executable object/reference feature:
  `bfbc62c4 native: route object property reference slots`.

## Grand Roadmap Position

| Workstream | Estimate | Bar | Current read |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | **99%** | `[####################]` | Strong selected-path value, byte-string, array, diagnostic, reference, symbol, call-frame, object, comparison, conversion, owner-cell, request-state, offset-read, array-key, type/int, text-membership, comparison, and truthiness slot surfaces. |
| Compiler/backend consumers | **99%** | `[####################]` | Generated C and LLVM consume many shared ABIs, request-backed array-key blockers, and byte-backed string value materialization. Direct assembly and some generated-C-only surfaces still lag. |
| Executable PHP semantics | **90%** | `[##################--]` | Primary has closure/callable/object islands, bounded preg callbacks, object-property reference-slot mutation, offset-read continuation proof, reference-backed array-key conversion, type/int, text-membership, comparison, truthiness consumers, request-key blocker parity, and byte-backed string value proof. |
| Strings and byte semantics | **57%** | `[###########---------]` | Byte-backed value representation is integrated. Full byte-exact interpreter output, binary literals/source bytes, string-array operation families, request/global keys, and exact diagnostics remain open. |
| Arrays, lvalues, references, COW | **77%** | `[###############-----]` | Value/reference slot ABI reuse is expanding and unsafe request-backed key materialization is blocked. Full COW, arbitrary roots, foreach, broader expression reference slots, and alias composition remain open. |
| Symbols, globals, request state | **73%** | `[###############-----]` | Selected function globals, root-symbol surfaces, active symbol-table reference consumers, and request-backed key blockers exist. `$GLOBALS` self-cells, request/global alias parity, request writeback, includes, variable variables, and exact unset/global behavior remain incomplete. |
| Calls, functions, frames | **83%** | `[#################---]` | Selected direct/callable/function-table surfaces exist. Named/unpacked/by-reference/userland frame breadth remains incomplete. |
| Objects, properties, methods | **50%** | `[##########----------]` | Object-property reference-slot mutation and diagnostic classifiers are integrated. Full visibility, magic, dynamic/static/typed properties, destructors, references/COW, and `ArrayAccess` execution remain open. |
| Control flow, cleanup, diagnostics | **51%** | `[##########----------]` | Selected branches, loops, transfers, finalizers, output buffers, diagnostics, and truthiness consumers exist. Broad unwind/finally/destructor/shutdown and exact source ordering remain open. |
| Broad integrated verification | **89%** | `[##################--]` | Focused gates are strong for recent packets. Broad gates remain constrained by lane extraction cost, high swap, stale lane expectations, and backend parity gaps. |

## Active Roadmap Items

| Item | Toward Primary Integration | Toward Full Feature | Status |
| --- | ---: | ---: | --- |
| Byte-preserving PHP string value boundary | **100%** `[####################]` | **39%** `[########------------]` | Integrated at `1c369d0f`. Runtime has `Value::BinaryString(Vec<u8>)`, `value_from_php_string_bytes(Vec<u8>)`, byte-view/runtime string-family preservation, native pointer-plus-length materialization, and linked byte-output proof. Full byte-output interpreter surfaces, binary source syntax, request/global key coercion, and string-array execution remain open. |
| Request-backed array-key/RMW blocker parity | **100%** `[####################]` | **37%** `[#######-------------]` | Integrated at `a501c4d1`. LLVM and generated C share blocker classification for request-backed ordinary array-key consumers across selected read, assignment, unset, reference assignment, for-action assignment/RMW, compound assignment, `??=`, and increment/decrement paths. Blocker-only: no request storage/writeback or `$GLOBALS` parity. |
| Truthiness value/reference-slot consumers | **100%** `[####################]` | **45%** `[#########-----------]` | Integrated at `24ec4a10`. Runtime, LLVM, and generated C route covered unary/logical truthiness, scalar/static `empty()`, reference-held operands, and native-value variable `isset`/`empty` proof paths through a shared value/reference-slot diagnostic ABI. |
| Reference-slot comparison consumers | **100%** `[####################]` | **45%** `[#########-----------]` | Integrated at `146c2d64`. Runtime, LLVM, and generated C route covered native value/reference comparison operands through shared diagnostic comparison slot ABIs. |
| Text-membership/reference text-byte conversion | **100%** `[####################]` | **44%** `[#########-----------]` | Integrated at `9f373b25`. Shared runtime value/reference text slot feeds selected `function_exists()` and `extension_loaded()` consumers. |
| Reference-slot type/int consumer ABI | **100%** `[####################]` | **45%** `[#########-----------]` | Integrated at `8f6266ce`. Runtime, LLVM, and generated C route reference-held type names, type predicates, and supported int operands through shared value/reference slots. |
| Array-key value/reference-slot ABI | **100%** `[####################]` | **42%** `[########------------]` | Integrated at `9022eb9e`. Broader expression lvalues, COW identity, object/resource/Stringable keys, request/global execution, and direct `ArrayAccess` remain open. |
| Scalar/resource offset-read source-result prerequisite | **100%** `[####################]` | **44%** `[#########-----------]` | Integrated at `cc7efc2d`. Direct object `ArrayAccess`, object/resource materialization, and LLVM error-status cleanup remain open. |
| Object-property reference-slot mutation | **100%** `[####################]` | **39%** `[########------------]` | Integrated at `bfbc62c4`. Covered assignment/unset operands use shared value/reference slot handling. |
| Bounded `preg_replace_callback()` string callbacks | **100%** `[####################]` | **32%** `[######--------------]` | Integrated at `6aca392d`. Full PCRE, broader captures/modifiers, non-string callables, `limit`/`count`/`flags`, and legacy recognizer cleanup remain open. |
| String operation-family slot consumers | **45%** `[#########-----------]` | **40%** `[########------------]` | The previous blocker, byte-preserving values, is now integrated. Re-run a fresh split from current primary and keep it operation-family level, not one UTF-8-only builtin. |
| Broader lvalue/reference-slot materializer | **30%** `[######--------------]` | **39%** `[########------------]` | Needed so non-variable expression families that can carry references can enter shared array-key and consumer slot ABIs safely. Should stay focused on slot materialization, not foreach or `ArrayAccess` execution. |
| Object/resource source materialization | **25%** `[#####---------------]` | **30%** `[######--------------]` | Explicit blocker left by the offset-read ABI. Needs a general value reconstruction boundary before generic object/resource consumers are safe. |
| LLVM offset-read/error-status cleanup | **25%** `[#####---------------]` | **30%** `[######--------------]` | Offset-read diagnostics exist, but LLVM still needs a generalized control-flow/error-exit status boundary for failed conversion results. |
| Callable-object/dynamic-constructor candidates | **52%** `[##########----------]` | **42%** `[########------------]` | Useful May 24 candidates remain stale relative to current primary and May 25 slot/request/string integrations. Refresh from current primary before review and do not combine them. |
| Diagnostics, request, and cleanup boundaries | **63%** `[#############-------]` | **41%** `[########------------]` | Lane-local request handle, writeback, branch cleanup, stateful-call cleanup, callback source spans, value-key, call-result, owner-cell, and result-boundary work is useful infrastructure. Control-flow reference-assignment is not bounded because a structured-CFG guard is failing. |
| Broad lane extraction backlog | **34%** `[#######-------------]` | **35%** `[#######-------------]` | Broad dirty lanes continue producing useful surfaces, but several are checkpointed, parked, paused, stopped, conflict-heavy, or not reviewable as whole lanes. Treat lanes as packet sources, not integration units. |

## Done / In Progress / Not Done

Primary-integrated executable or executable-prerequisite capability:

- [x] Descriptor-backed closures, selected captures, selected by-reference parameters, and selected callable-array/object invocation.
- [x] Runtime string-valued declared-class `new` for selected declared classes, with destructor-observable allocation blocked before unsafe native allocation.
- [x] Bounded public declared-object properties, methods, statics, constructors, named `instanceof`, and same-family aggregate equality.
- [x] Bounded `preg_replace_callback()` string-callback execution over supported slash-delimited patterns.
- [x] Object-property assignment/unset mutation for covered reference-backed operands through generated-C/native-link shared slot boundaries.
- [x] Shared offset-read source-result ABI for scalar/resource warning continuations, arrays, byte strings, references, and object-property offset-source composition.
- [x] Shared array-key value/reference-slot ABI for generated-native reference-backed variable operands and active symbol-table variable references.
- [x] Shared reference-slot type-name/type-predicate/int consumer ABI for runtime, LLVM, and generated C.
- [x] Shared reference-slot text-byte/text-membership ABI for runtime, LLVM, and generated C selected paths.
- [x] Shared reference-slot comparison ABI for covered runtime, LLVM, and generated-C native value comparison consumers.
- [x] Shared reference-slot truthiness ABI for covered runtime, LLVM, and generated-C native value truthiness consumers.
- [x] Shared request-backed ordinary array-key/RMW blocker classification for selected LLVM and generated-C consumers.
- [x] Byte-backed PHP string value representation and native pointer-plus-length materialization for arbitrary PHP string bytes.

Primary-integrated non-executable infrastructure:

- [x] Object-offset `ArrayAccess` receiver diagnostic classifier for read, append-read, null-coalesce, `isset`, `empty`, and error-control forms.
- [x] Symbol-table ABI probe is pushed, but remains probe-only until real assignment/readback consumers land.

In progress but lane-local or not yet executable primary support:

- [ ] String-array operation-family value/reference slots can now be retried from current primary after `1c369d0f`.
- [ ] Full byte-exact tree-walk interpreter output surfaces remain blocked behind a real byte-output/session/debug formatting representation.
- [ ] Request/global key byte coercion for byte-backed strings remains intentionally blocked.
- [ ] Broader expression-family lvalue/reference-slot materialization is needed beyond variable-backed operands.
- [ ] State cleanup, callback source spans, include/require path cleanup, call-result diagnostics, owner-cell sinks, value-key/null-coalesce, and control-flow result boundaries are active lane-local evidence.
- [ ] Direct object `ArrayAccess` method dispatch remains blocked behind diagnostic-only classifier support.
- [ ] Alias-aware LLVM direct-root write-through after `=&` remains blocked for both statement assignment and assignment expressions.
- [ ] Object/resource source materialization for generic conversion sources remains blocked.
- [ ] LLVM offset-read error-status cleanup needs a generalized control-flow boundary.
- [ ] Callable-object, dynamic-constructor, object-instantiation, and destructor-blocker candidates need current-primary refresh before review.
- [ ] Binary-string, stream, PCRE, callback, pathinfo, filesystem, and broad internal-callback surfaces remain lane-local until extracted into compact semantic packets.

Not done:

- [ ] Binary literal syntax, invalid-UTF-8 PHP source parsing, and byte-exact output/session/debug formatting across the tree-walk interpreter.
- [ ] Executable request storage/writeback, `$GLOBALS` self-cells, request/global alias parity, request foreach, and mutation-during-iteration behavior.
- [ ] Full callable lookup and invocation, including non-string preg callbacks, closures, arrays, invokable objects, magic/visibility, and rebinding rules.
- [ ] Runtime `ArrayAccess` method dispatch for `offsetGet`, `offsetExists`, `offsetSet`, and `offsetUnset`.
- [ ] Full references/COW identity and arbitrary alias roots.
- [ ] Includes, variable variables, and dynamic symbol behavior.
- [ ] Full PCRE behavior beyond the bounded slash-delimited subset.
- [ ] Retirement or reframing of unrelated legacy WordPress-named preg/database recognizers behind generalized PHP semantic boundaries.
- [ ] General object model: non-public methods, overrides, interfaces/traits execution, magic methods, dynamic/static/typed properties, destructors.
- [ ] Complete cleanup/unwind/finally/destructor/output-buffer shutdown behavior.
- [ ] Exact/source-ordered diagnostics, custom handler execution, warning/error continuation, and suppression parity.
- [ ] LLVM/direct assembly parity for recent generated-C semantics.

## Recent Primary-Integrated Work

- `1c369d0f`: byte-backed PHP string value boundary. Integrated files:
  `runtime/src/lib.rs`, `compiler/src/interpreter.rs`,
  `compiler/tests/native_runtime_abi.rs`, and `compiler/tests/native_link.rs`.
  Review/integration proof included exact hash/scope/apply checks, four
  nonzero focused gates, `cargo check`, `cargo fmt --check`,
  `git diff --check`, push proof, and clean post-push state.
- `6f0bf3a9`: progress-dashboard commit only. No executable compiler/runtime
  semantic code changed.
- `ff1b3509`: progress-dashboard commit only. No executable compiler/runtime
  semantic code changed.
- `a501c4d1`: request-backed array-key/RMW blocker parity. Integrated files:
  `compiler/src/codegen.rs`, `compiler/tests/native_runtime_abi.rs`, and
  `compiler/tests/superglobals.rs`.
- `24ec4a10`: reference truthiness slots. Integrated files:
  `runtime/src/lib.rs`, `compiler/src/codegen.rs`,
  `compiler/tests/native_runtime_abi.rs`, and `compiler/tests/native_link.rs`.
- `146c2d64`: reference comparison slots. Integrated files:
  `runtime/src/lib.rs`, `compiler/src/codegen.rs`,
  `compiler/tests/native_runtime_abi.rs`, and `compiler/tests/native_link.rs`.
- `9f373b25`: reference text-membership slots. Integrated files:
  `runtime/src/lib.rs`, `compiler/src/codegen.rs`,
  `compiler/tests/native_runtime_abi.rs`, and `compiler/tests/native_link.rs`.

## Current Work Snapshot

Primary-integrated:

- [x] Primary was clean and synced at `1c369d0f` before this `PROGRESS.md`
  edit.
- [x] Latest counted semantic/prerequisite commit is `1c369d0f`.
- [x] Byte-backed PHP string value representation is integrated with native
  IR/C materialization and linked executable byte-output proof.
- [x] Request-backed ordinary array-key/RMW consumers share blocker
  classification across selected LLVM and generated-C paths.
- [x] Reference-held truthiness, comparison, text-membership, type/int,
  array-key value/reference slots, and offset-read source-result support remain
  integrated for reviewed selected paths.
- [x] No uncommitted primary implementation diffs were present before this
  `PROGRESS.md` edit.

Lane-local:

- [ ] `string-array-slot-split-extract` remains `needs-split`, but its prior
  byte-string representation blocker is now removed on primary.
- [ ] `impl-symbol-integrator` has lane-local generated-C scalar-cast,
  unary-not, and filtered `array_keys()` result-contract work. Treat as packet
  source only.
- [ ] `impl-array-linked-exec` has lane-local reference/lvalue diagnostics and
  resource by-reference foreach recovery work. Still not a whole-lane
  integration candidate.
- [ ] `impl-native-call-semantics` has receiver-object class-set planning, but
  actual instance method execution still hits frame-callback blockers.
- [ ] `impl-native-type-conversion` is removing backend-local folds and routing
  several expression families through shared conversion-result boundaries, but
  broad lane state and stale full-gate expectations make it extraction-only.
- [ ] `impl-native-control-flow-seed` is not bounded because a structured-CFG
  guard is failing. Do not review for primary until that is fixed.
- [ ] `impl-function-frame-seed` is parked after missed cadence.
- [ ] `impl-array-value-runtime` is still mostly read-only/checkpoint evidence
  in its latest visible status, not an integration candidate.

Resource posture:

- `/dev/shm`: live df `40G` total, `24G` used, `17G` available, 58% used.
  Largest observed target dirs include `phpc-target-native-call-semantics`
  at `8.9G`, `phpc-target-native-object-seed` at `5.6G`, and
  `phpc-target-native-diagnostics` at `3.0G`.
- `/home`: live df `459G` total, `202G` used, `239G` available, 46% used.
  Large observed home dirs include
  `/home/claude/phpc-lane-native-error-diagnostic-semantics` at `14G`,
  `/home/claude/playground-all-prs` at `5.7G`, and
  `/home/claude/playground-gap-address` at `4.9G`.
- Memory: `43Gi` total, about `37Gi` available; swap remains high at
  `23Gi/29Gi`.
- Continue using disk-backed target dirs, `umask 0007`, `CARGO_BUILD_JOBS=1`,
  `CARGO_INCREMENTAL=0`, and focused nonzero gates.

Dashboard/watch status:

- `state/dashboard.md` was live-read at 17:43 CEST and is stale after the
  17:55 primary integration; it still reports primary at dashboard-only
  `ff1b3509`.
- The integration status artifact correctly reports
  `primary-integrator-byte-string-value-interpreter-adoption` as
  `integrated-and-pushed`.
- Cadence issues remain worth watching: `impl-function-frame-seed` is parked,
  and `impl-native-control-flow-seed` is not bounded after its failing guard.

## Next Steering Read

Best next compact packets to consider:

- re-run one string-array operation-family value/reference-slot split from
  current primary now that byte-backed PHP string values are integrated;
- a broader lvalue/reference materializer prerequisite, if it stays focused on
  enabling shared slot ABIs rather than adding foreach or `ArrayAccess`
  execution;
- a refreshed callable-object, dynamic-constructor, object-instantiation, or
  destructor-blocker packet, after rebasing and reviewing from current primary;
- a cleanup/control-flow packet only if it resolves the structured-CFG guard,
  replaces duplicated backend routing, and has nonzero focused proof.

Do not count lane-local triage, stopped dirty lanes, broad source-lane work,
stale May 24 candidates, callback/filesystem-family accumulation,
blocker-only metadata, review-only candidates, or docs-only commits as product
capability.
