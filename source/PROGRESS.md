# PHP Native Compiler Progress

Updated: 2026-05-25 20:52 CEST
Evaluation marker: `20260525T185240Z`

Accounting rule: only generalized, tested, committed, and pushed primary work
counts as integrated capability. Dirty WIP, candidate worktrees, lane-local
claims, review-only work, failed prep proofs, probe-only commits, and
dashboard-only commits are excluded.

## Executive Read

Overall estimated progress: **92%** `[##################--]`

Executable PHP semantics: **92%** `[##################--]`

Primary was clean and aligned with `origin/master` at
`77cca4ec docs: update progress dashboard` before this `PROGRESS.md` edit.
The latest counted semantic/prerequisite baseline remains `b13c85c6`.

No new semantic primary commit landed after the unary `-` source/result ABI.
This review keeps capability percentages flat while recording current
lane-local progress: a post-unary conversion-result helper candidate is ready
for primary review, but it is helper consolidation and is not counted as
integrated capability.

The latest counted semantic window converted the unary `-` conversion gap from
a failed codegen-only prep into an integrated runtime/compiler boundary.
Runtime now exposes a shared numeric-unary source/result ABI for unary
negation, and LLVM plus generated C route covered unary `-` operands through
`NativeConversionSource` and `NativeConversionResult` instead of backend-local
primitive negation/folding.

This is real primary progress, but it is still a selected conversion-family
boundary. Full generalized PHP remains blocked on references/COW identity,
arbitrary lvalues, request/global parity and writeback, includes, variable
variables, broad userland frames, real `ArrayAccess`, object/magic/visibility/
destructor semantics, cleanup/unwind/finally/shutdown ordering, exact
diagnostics/error handlers, and backend parity.

## Primary-Integrated Baseline

- Current primary head before this dashboard edit:
  `77cca4ec docs: update progress dashboard`.
- Latest integrated executable/prerequisite semantic baseline:
  `b13c85c6 native: add unary negation source result ABI`.
- Recent integrated prerequisite:
  `f770d728 native: block non-local assignment owners`.
- Recent integrated prerequisite:
  `5307990c native: add string-array operation slots`.
- Recent integrated prerequisite:
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
| Runtime and ABI foundations | **99%** | `[####################]` | Strong selected-path value, byte-string, string-array, array, diagnostic, reference, symbol, call-frame, object, comparison, conversion, numeric-unary, owner-cell, request-state, offset-read, array-key, type/int, text-membership, comparison, truthiness, and non-local owner-boundary surfaces. |
| Compiler/backend consumers | **99%** | `[####################]` | Generated C and LLVM consume many shared ABIs, including unary negation source/result routing, request-backed array-key blockers, byte-backed string values, string-array operation slots, and non-local owner-family blockers. Direct assembly and some generated-C-only surfaces still lag. |
| Executable PHP semantics | **92%** | `[##################--]` | Primary has closure/callable/object islands, bounded preg callbacks, object-property reference-slot mutation, offset-read continuation proof, reference-backed array-key conversion, type/int, text-membership, comparison, truthiness consumers, request-key blocker parity, byte-backed strings, string-array slots, non-local owner blockers, and selected unary-negation source/result execution. |
| Strings and byte semantics | **62%** | `[############--------]` | Byte-backed values and shared byte-preserving string-array results are integrated. Full byte-exact interpreter output, binary source bytes, `mb_str_split()` codepoint semantics, request/global keys, and exact diagnostics remain open. |
| Arrays, lvalues, references, COW | **78%** | `[################----]` | Value/reference slot ABI reuse is expanding, unsafe request-backed key materialization is blocked, and non-local owner assignment/unset families now share a blocker. Full COW, arbitrary roots, foreach, broader expression reference slots, and alias composition remain open. |
| Symbols, globals, request state | **73%** | `[###############-----]` | Selected function globals, root-symbol surfaces, active symbol-table reference consumers, and request-backed key blockers exist. `$GLOBALS` self-cells, request/global alias parity, request writeback, includes, variable variables, and exact unset/global behavior remain incomplete. |
| Calls, functions, frames | **83%** | `[#################---]` | Selected direct/callable/function-table surfaces exist. Named/unpacked/by-reference/userland frame breadth remains incomplete. |
| Objects, properties, methods | **51%** | `[##########----------]` | Object-property reference-slot mutation, diagnostic classifiers, and non-local object/static owner blockers are integrated. Full visibility, magic, dynamic/static/typed properties, destructors, references/COW, and `ArrayAccess` execution remain open. |
| Control flow, cleanup, diagnostics | **51%** | `[##########----------]` | Selected branches, loops, transfers, finalizers, output buffers, diagnostics, and truthiness/conversion consumers exist. Broad unwind/finally/destructor/shutdown and exact source ordering remain open. |
| Broad integrated verification | **91%** | `[##################--]` | Focused primary gates are strong for recent packets. Broad gates remain constrained by lane extraction cost, high swap, stale lane expectations, and backend parity gaps. |

## Active Roadmap Items

| Item | Toward Primary Integration | Toward Full Feature | Status |
| --- | ---: | ---: | --- |
| Runtime numeric-unary conversion ABI | **100%** `[####################]` | **48%** `[##########----------]` | Integrated at `b13c85c6`. Runtime exposes `PHPC_NATIVE_NUMERIC_UNARY_OP_NEGATE` and `phpc_native_conversion_source_numeric_unary(...)`; LLVM and generated C route covered unary `-` through `NativeConversionSource` / `NativeConversionResult`. Broader numeric builtins, object/resource coercions, exact diagnostics, custom handlers, error control, and cleanup/unwind remain open. |
| Non-local assignment owner-cell blocker refresh | **100%** `[####################]` | **35%** `[#######-------------]` | Integrated at `f770d728`. LLVM assignment statements/expressions and C/assembly codegen paths route non-local object/static owner families through shared assignment/unset owner blockers while preserving ArrayAccess precedence. Blocker/parity work, not executable property/static writeback. |
| String operation-family slot consumers | **100%** `[####################]` | **45%** `[#########-----------]` | Integrated at `5307990c`. Runtime, LLVM, generated C, and linked execution route `explode()` and `str_split()` through shared byte-preserving value/reference-slot string-array contracts. `mb_str_split()`, binary source parsing, request/global execution, and broad multibyte semantics remain open. |
| Byte-preserving PHP string value boundary | **100%** `[####################]` | **40%** `[########------------]` | Integrated at `1c369d0f`. Runtime has `Value::BinaryString(Vec<u8>)`, byte-view/runtime string-family preservation, native pointer-plus-length materialization, and linked byte-output proof. Full byte-output interpreter surfaces, binary source syntax, and request/global key coercion remain open. |
| Request-backed array-key/RMW blocker parity | **100%** `[####################]` | **37%** `[#######-------------]` | Integrated at `a501c4d1`. LLVM and generated C share blocker classification for request-backed ordinary array-key consumers across selected read, assignment, unset, reference assignment, for-action assignment/RMW, compound assignment, `??=`, and increment/decrement paths. Blocker-only: no request storage/writeback or `$GLOBALS` parity. |
| Reference-slot consumer families | **100%** `[####################]` | **45%** `[#########-----------]` | Type/int, text-membership, comparison, truthiness, array-key, and offset-read value/reference slots are integrated for reviewed selected paths. Full alias/COW composition remains open. |
| Conversion and call-boundary shortcut retirement | **50%** `[##########----------]` | **42%** `[########------------]` | Unary negation now has primary source/result routing. Lane-local type-conversion work reports more shortcut retirement for arithmetic, concat, comparison, `empty()`, type introspection, nested unary, and logical families, but that lane is broad and currently not a primary packet. |
| Broader lvalue/reference-slot materializer | **30%** `[######--------------]` | **39%** `[########------------]` | Needed so non-variable expression families that can carry references can enter shared array-key and consumer slot ABIs safely. Should stay focused on slot materialization, not foreach or `ArrayAccess` execution. |
| Object/resource source materialization | **25%** `[#####---------------]` | **30%** `[######--------------]` | Still a recurring blocker for generic conversion and offset/source consumers. Needs a general value reconstruction/materialization boundary before generic object/resource consumers are safe. |
| LLVM offset-read/error-status cleanup | **25%** `[#####---------------]` | **30%** `[######--------------]` | Offset-read diagnostics exist, but LLVM still needs a generalized control-flow/error-exit status boundary for failed conversion results. |
| Symbol/object metadata candidates | **40%** `[########------------]` | **35%** `[#######-------------]` | Lane-local symbol work reports core type/member/relationship metadata ABIs. Interesting, but broad and dirty; must be packetized and audited for hard-coded-table risk before primary review. |
| Diagnostics, request, and cleanup boundaries | **66%** `[#############-------]` | **42%** `[########------------]` | Lane-local diagnostic selectors, path-RMW, request, symbol, call, conversion, and cleanup-boundary work is producing useful packet sources. Several are blocker-only and should not be counted until exact-scope primary integration. |
| Broad lane extraction backlog | **34%** `[#######-------------]` | **35%** `[#######-------------]` | Broad dirty lanes continue producing useful evidence, but many are checkpointed, parked, paused, stopped, conflict-heavy, or not reviewable as whole lanes. Treat lanes as packet sources, not integration units. |

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
- [x] Shared byte-preserving `explode()` and `str_split()` string-array operation slots across runtime, LLVM, generated C, and linked execution.
- [x] Shared non-local assignment/unset owner-family blockers for object/static property assignment and unset paths across current backend lowering families.
- [x] Shared numeric-unary source/result ABI for covered unary negation across runtime, LLVM, generated C, and focused linked execution.

Primary-integrated non-executable infrastructure:

- [x] Object-offset `ArrayAccess` receiver diagnostic classifier for read, append-read, null-coalesce, `isset`, `empty`, and error-control forms.
- [x] Symbol-table ABI probe is pushed, but remains probe-only until real assignment/readback consumers land.

In progress but lane-local or not yet executable primary support:

- [ ] `impl-native-type-conversion` is removing exact-shape arithmetic, comparison, concat, `empty()`, `is_numeric()`, `isset()`, nested unary, and logical shortcuts in a broad lane; current latest pass was hard-stopped without a new bounded packet.
- [ ] `impl-native-call-semantics` reports lane-local call-result discard/failure cleanup, frame array-path reference, callable descriptor, and object `__invoke` planning improvements.
- [ ] `impl-native-error-diagnostic-semantics` reports lane-local shared backend selectors for diagnostic symbol, request, path, array, binary, unary, and call consumers.
- [ ] `impl-array-linked-exec` reports lane-local cleanup/blocker ownership improvements for object-property array updates, stateful call operands, try/throw exits, resource roots, and computed roots.
- [ ] `impl-symbol-integrator` reports lane-local core type/member/relationship metadata ABIs, but the lane is broad, dirty, and parked after hard-stop.
- [ ] Several broad lanes are parked after cadence failures or conflict-heavy exploration. Reclaim only through fresh, exact-scope current-primary prep.
- [ ] Full byte-exact tree-walk interpreter output surfaces remain blocked behind a real byte-output/session/debug formatting representation.
- [ ] Request/global key byte coercion for byte-backed strings remains intentionally blocked.
- [ ] Broader expression-family lvalue/reference-slot materialization is needed beyond variable-backed operands.
- [ ] Direct object `ArrayAccess` method dispatch remains blocked behind diagnostic-only classifier support.
- [ ] Alias-aware LLVM direct-root write-through after `=&` remains blocked for both statement assignment and assignment expressions.
- [ ] Object/resource source materialization for generic conversion sources remains blocked.
- [ ] LLVM offset-read error-status cleanup needs a generalized control-flow boundary.
- [ ] Callable-object, dynamic-constructor, object-instantiation, and destructor-blocker candidates need current-primary refresh before review.

Not done:

- [ ] `mb_str_split()` generalized multibyte/codepoint splitting.
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

- `b13c85c6`: unary negation source/result ABI. Integrated files:
  `runtime/src/lib.rs`, `compiler/src/codegen.rs`, and
  `compiler/tests/native_runtime_abi.rs`. Review/integration proof included
  stable candidate hash
  `c9e21d86ef7c6997e9721d5726d63b2ef3cd098ce4d9841bb72e119d5cb6006a`,
  current-primary apply proof, three focused nonzero gates, `cargo check`,
  `cargo fmt --check`, `git diff --check`, push proof, and clean post-push
  sync except for the evaluator-owned `PROGRESS.md` update.
- `a8268e0e`: progress-dashboard commit only. No executable compiler/runtime
  semantic code changed.
- `f770d728`: non-local assignment owner blockers. Integrated files:
  `compiler/src/codegen.rs` and `compiler/tests/native_array_boundary.rs`.
- `5307990c`: string-array operation slots. Integrated files:
  `runtime/src/lib.rs`, `compiler/src/codegen.rs`,
  `compiler/tests/native_runtime_abi.rs`, and `compiler/tests/native_link.rs`.
- `1c369d0f`: byte-backed PHP string value boundary. Integrated files:
  `runtime/src/lib.rs`, `compiler/src/interpreter.rs`,
  `compiler/tests/native_runtime_abi.rs`, and `compiler/tests/native_link.rs`.
- `a501c4d1`: request-backed array-key/RMW blocker parity. Integrated files:
  `compiler/src/codegen.rs`, `compiler/tests/native_runtime_abi.rs`, and
  `compiler/tests/superglobals.rs`.

## Current Work Snapshot

Primary-integrated:

- [x] Primary was clean and synced at `b13c85c6` before this `PROGRESS.md`
  edit.
- [x] Latest counted semantic/prerequisite commit is `b13c85c6`.
- [x] Covered unary `-` lowering now uses a shared runtime numeric-unary
  source/result ABI instead of backend-local primitive negation/folding.
- [x] Non-local assignment and unset object/static owner families share
  assignment/unset owner-boundary classification before backend lowering.
- [x] Byte-backed PHP string value representation remains integrated with
  native IR/C materialization and linked executable byte-output proof.
- [x] `explode()` and `str_split()` have shared byte-preserving string-array
  operation slots across runtime, LLVM, generated C, and linked execution.
- [x] Request-backed ordinary array-key/RMW consumers share blocker
  classification across selected LLVM and generated-C paths.
- [x] Reference-held truthiness, comparison, text-membership, type/int,
  array-key value/reference slots, and offset-read source-result support remain
  integrated for reviewed selected paths.

Lane-local:

- [ ] `impl-native-type-conversion` contains useful but very broad conversion
  shortcut-retirement work and was most recently hard-stopped without selecting
  a new bounded packet.
- [ ] `impl-native-call-semantics` reports call-result discard/failure and
  frame array-path reference improvements, still lane-local.
- [ ] `impl-native-error-diagnostic-semantics` reports backend selector cleanup
  for diagnostic consumers, still lane-local.
- [ ] `impl-array-linked-exec` reports cleanup/blocker ordering improvements,
  mostly blocker-boundary work, still lane-local.
- [ ] `impl-symbol-integrator` reports core metadata ABI work but is broad,
  dirty, and parked; use only as evidence.
- [ ] Forced-parked or stale broad lanes remain evidence only, not importable
  work units.

Resource posture:

- `/dev/shm`: live df `40G` total, `24G` used, `17G` available, 58% used.
  Largest observed target dirs are `phpc-target-native-call-semantics` at
  `8.9G`, `phpc-target-native-object-seed` at `5.6G`, and
  `phpc-target-native-diagnostics` at `3.0G`.
- `/home`: live df `459G` total, `194G` used, `247G` available, 45% used.
  Largest observed lane/work tree is
  `phpc-lane-native-error-diagnostic-semantics` at `14G`; primary is about
  `2.2G`.
- Memory: `43Gi` total, about `40Gi` available; swap remains high at
  `23Gi/29Gi`.
- Continue disk-backed target dirs, `umask 0007`, `CARGO_BUILD_JOBS=1`,
  `CARGO_INCREMENTAL=0`, and focused nonzero gates. Consider owner-checked
  cleanup only if `/dev/shm` approaches the dispatch floor.

## Next Steering Read

Best next compact packets to consider:

- conversion-result consumer/free/report helper consolidation following the
  integrated unary-negation source/result ABI;
- a narrow call-result discard/failure cleanup packet, if it can be extracted
  from `impl-native-call-semantics` without pre-existing lane collateral;
- diagnostic consumer selector cleanup only when it removes duplicated backend
  ABI selection without claiming new executable PHP behavior;
- broader lvalue/reference materializer prerequisite, if it stays focused on
  enabling shared slot ABIs rather than adding foreach or `ArrayAccess`
  execution;
- object/resource source materialization for generic conversion consumers;
- symbol/core metadata ABI only after it is trimmed from broad lane collateral
  and audited for hard-coded-table risk;
- refreshed callable-object, dynamic-constructor, object-instantiation, or
  destructor-blocker packets after rebasing and reviewing from current primary.

Do not count lane-local triage, stopped dirty lanes, parked broad lanes, stale
candidates, blocker-only metadata, review-only candidates, failed prep tests,
or docs-only commits as product capability.
