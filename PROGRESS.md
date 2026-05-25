# PHP Native Compiler Progress

Updated: 2026-05-25 21:07 CEST
Evaluation marker: `20260525T190755Z`

Accounting rule: only generalized, tested, committed, and pushed primary work
counts as integrated capability. Dirty WIP, candidate worktrees, lane-local
claims, review-only work, failed prep proofs, probe-only commits, and
dashboard-only commits are excluded.

## Executive Read

Overall estimated progress: **92%** `[##################--]`

Executable PHP semantics: **92%** `[##################--]`

Primary is clean and aligned with `origin/master` at
`2cd78ade native: share conversion result consumers` before this
`PROGRESS.md` edit.

The latest primary-integrated production/prerequisite baseline is now
`2cd78ade`. The latest integrated executable capability baseline remains
`b13c85c6 native: add unary negation source result ABI`.

This review keeps percentages flat. The new primary commit is real
compiler/backend cleanup: LLVM conversion-result consumers for scalar offset
reads and numeric unary source results now share one value/status/diagnostic
consumer helper with cleanup ownership. It is not a new broad PHP semantic
feature by itself.

The recent semantic arc is still useful: byte-backed strings, string-array
operation slots, non-local owner blockers, numeric-unary source/result routing,
and conversion-result consumer consolidation are all primary-integrated. Full
generalized PHP remains blocked on references/COW identity, arbitrary lvalues,
request/global parity and writeback, includes, variable variables, broad
userland frames, real `ArrayAccess`, object/magic/visibility/destructor
semantics, cleanup/unwind/finally/shutdown ordering, exact diagnostics/error
handlers, and backend parity.

## Primary-Integrated Baseline

- Current primary head before this dashboard edit:
  `2cd78ade native: share conversion result consumers`.
- Latest integrated production/prerequisite cleanup:
  `2cd78ade native: share conversion result consumers`.
- Latest integrated executable capability:
  `b13c85c6 native: add unary negation source result ABI`.
- Recent integrated prerequisite:
  `f770d728 native: block non-local assignment owners`.
- Recent integrated prerequisite:
  `5307990c native: add string-array operation slots`.
- Recent integrated prerequisite:
  `1c369d0f native: add byte-backed PHP string value boundary`.
- Recent integrated prerequisite:
  `a501c4d1 native: block request-backed array keys`.
- Recent integrated reference/slot consumers:
  `24ec4a10`, `146c2d64`, `9f373b25`, `8f6266ce`, `9022eb9e`,
  `cc7efc2d`.
- Recent integrated executable object/reference feature:
  `bfbc62c4 native: route object property reference slots`.

## Grand Roadmap Position

| Workstream | Estimate | Bar | Current read |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | **99%** | `[####################]` | Strong selected-path value, byte-string, string-array, array, diagnostic, reference, symbol, call-frame, object, comparison, conversion, numeric-unary, owner-cell, request-state, offset-read, array-key, type/int, text-membership, comparison, truthiness, and non-local owner-boundary surfaces. |
| Compiler/backend consumers | **99%** | `[####################]` | Generated C and LLVM consume many shared ABIs. `2cd78ade` removes duplicated LLVM conversion-result consumption for current offset-read and numeric-unary source/result paths. Direct assembly and some generated-C-only surfaces still lag. |
| Executable PHP semantics | **92%** | `[##################--]` | Primary has closure/callable/object islands, bounded preg callbacks, object-property reference-slot mutation, offset-read continuation proof, reference-backed array-key conversion, type/int, text-membership, comparison, truthiness consumers, request-key blocker parity, byte-backed strings, string-array slots, non-local owner blockers, and selected unary-negation source/result execution. |
| Strings and byte semantics | **62%** | `[############--------]` | Byte-backed values and shared byte-preserving `explode()` / `str_split()` results are integrated. Full byte-exact interpreter output, binary source bytes, `mb_str_split()` codepoint semantics, request/global keys, and exact diagnostics remain open. |
| Arrays, lvalues, references, COW | **78%** | `[################----]` | Value/reference slot ABI reuse is expanding, unsafe request-backed key materialization is blocked, and non-local owner assignment/unset families share a blocker. Full COW, arbitrary roots, foreach, broader expression reference slots, and alias composition remain open. |
| Symbols, globals, request state | **73%** | `[###############-----]` | Selected function globals, root-symbol surfaces, active symbol-table reference consumers, and request-backed key blockers exist. `$GLOBALS` self-cells, request/global alias parity, request writeback, includes, variable variables, and exact unset/global behavior remain incomplete. |
| Calls, functions, frames | **83%** | `[#################---]` | Selected direct/callable/function-table and descriptor-closure surfaces exist. Named/unpacked/by-reference/userland frame breadth remains incomplete; lane-local by-reference closure capture work is not counted. |
| Objects, properties, methods | **51%** | `[##########----------]` | Object-property reference-slot mutation, diagnostic classifiers, and non-local object/static owner blockers are integrated. Full visibility, magic, dynamic/static/typed properties, destructors, references/COW, and `ArrayAccess` execution remain open. |
| Control flow, cleanup, diagnostics | **51%** | `[##########----------]` | Selected branches, loops, transfers, finalizers, output buffers, diagnostics, and truthiness/conversion consumers exist. Broad unwind/finally/destructor/shutdown and exact source ordering remain open. |
| Broad integrated verification | **91%** | `[##################--]` | Focused primary gates are strong for recent packets. Broad gates remain constrained by lane extraction cost, high swap, stale lane expectations, and backend parity gaps. |

## Active Roadmap Items

| Item | Toward Primary Integration | Toward Full Feature | Status |
| --- | ---: | ---: | --- |
| Conversion-result helper and shortcut retirement | **100%** `[####################]` | **44%** `[#########-----------]` | Integrated at `2cd78ade` for current LLVM scalar offset-read and numeric-unary source-result consumers. This is cleanup/prerequisite work: value/status/diagnostic extraction, failure/null guarding, and source-handle cleanup are shared. Broader conversion families, object/resource coercions, exact diagnostics, error control, and cleanup/unwind remain open. |
| Runtime numeric-unary conversion ABI | **100%** `[####################]` | **48%** `[##########----------]` | Integrated at `b13c85c6`. Runtime exposes `PHPC_NATIVE_NUMERIC_UNARY_OP_NEGATE` and `phpc_native_conversion_source_numeric_unary(...)`; LLVM and generated C route covered unary `-` through `NativeConversionSource` / `NativeConversionResult`. |
| Non-local assignment owner-cell blocker refresh | **100%** `[####################]` | **35%** `[#######-------------]` | Integrated at `f770d728`. LLVM assignment statements/expressions and C/assembly codegen paths route non-local object/static owner families through shared assignment/unset owner blockers while preserving ArrayAccess precedence. Blocker/parity work, not executable property/static writeback. |
| String operation-family slot consumers | **100%** `[####################]` | **45%** `[#########-----------]` | Integrated at `5307990c`. Runtime, LLVM, generated C, and linked execution route `explode()` and `str_split()` through shared byte-preserving value/reference-slot string-array contracts. |
| Byte-preserving PHP string value boundary | **100%** `[####################]` | **40%** `[########------------]` | Integrated at `1c369d0f`. Runtime has `Value::BinaryString(Vec<u8>)`, byte-view/runtime string-family preservation, native pointer-plus-length materialization, and linked byte-output proof. |
| Request-backed array-key/RMW blocker parity | **100%** `[####################]` | **37%** `[#######-------------]` | Integrated at `a501c4d1`. LLVM and generated C share blocker classification for request-backed ordinary array-key consumers across selected read, assignment, unset, reference assignment, for-action assignment/RMW, compound assignment, `??=`, and increment/decrement paths. |
| Reference-slot consumer families | **100%** `[####################]` | **45%** `[#########-----------]` | Type/int, text-membership, comparison, truthiness, array-key, and offset-read value/reference slots are integrated for reviewed selected paths. Full alias/COW composition remains open. |
| Broader lvalue/reference-slot materializer | **30%** `[######--------------]` | **39%** `[########------------]` | Needed so non-variable expression families that can carry references can enter shared array-key and consumer slot ABIs safely. Current array-linked evidence is parked and lane-local. |
| Object/resource source materialization | **25%** `[#####---------------]` | **30%** `[######--------------]` | Still a recurring blocker for generic conversion and offset/source consumers. Needs a general value reconstruction/materialization boundary before generic object/resource consumers are safe. |
| LLVM offset-read/error-status cleanup | **35%** `[#######-------------]` | **32%** `[######--------------]` | `2cd78ade` improves current LLVM conversion-result cleanup/guard sharing for scalar offset reads, but a broader control-flow/error-exit status boundary is still missing. |
| Call/frame reference-capture execution | **35%** `[#######-------------]` | **38%** `[########------------]` | Lane-local `impl-native-call-semantics` reports source-aware by-reference closure capture execution across several call consumers. Not counted until extracted, reviewed, integrated, committed, and pushed on primary. |
| Diagnostic selector/path-array cleanup | **45%** `[#########-----------]` | **42%** `[########------------]` | Lane-local diagnostic work reports shared backend selectors for path-array and symbol/object mutation consumers. Useful as packet source, but cleanup-only unless it retires a hard execution blocker on primary. |
| Broad lane extraction backlog | **34%** `[#######-------------]` | **35%** `[#######-------------]` | Broad dirty lanes continue producing useful evidence, but several are parked after cadence failures or old-head probing. Treat lanes as evidence repositories, not integration units. |

## Done / In Progress / Not Done

Primary-integrated executable or executable-prerequisite capability:

- [x] Descriptor-backed closures, selected captures, selected by-reference parameters, and selected callable-array/object invocation.
- [x] Runtime string-valued declared-class `new` for selected declared classes, with destructor-observable allocation blocked before unsafe native allocation.
- [x] Bounded public declared-object properties, methods, statics, constructors, named `instanceof`, and same-family aggregate equality.
- [x] Bounded `preg_replace_callback()` string-callback execution over supported slash-delimited patterns.
- [x] Object-property assignment/unset mutation for covered reference-backed operands through generated-C/native-link shared slot boundaries.
- [x] Shared offset-read source-result ABI for scalar/resource warning continuations, arrays, byte strings, references, and object-property offset-source composition.
- [x] Shared array-key value/reference-slot ABI for generated-native reference-backed variable operands and active symbol-table variable references.
- [x] Shared reference-slot type-name/type-predicate/int, text-byte/text-membership, comparison, and truthiness consumers for selected runtime, LLVM, and generated-C paths.
- [x] Shared request-backed ordinary array-key/RMW blocker classification for selected LLVM and generated-C consumers.
- [x] Byte-backed PHP string value representation and native pointer-plus-length materialization for arbitrary PHP string bytes.
- [x] Shared byte-preserving `explode()` and `str_split()` string-array operation slots across runtime, LLVM, generated C, and linked execution.
- [x] Shared non-local assignment/unset owner-family blockers for object/static property assignment and unset paths across current backend lowering families.
- [x] Shared numeric-unary source/result ABI for covered unary negation across runtime, LLVM, generated C, and focused linked execution.
- [x] Shared LLVM conversion-result consumer helper for current scalar offset-read and numeric-unary source-result paths.

Primary-integrated non-executable infrastructure:

- [x] Object-offset `ArrayAccess` receiver diagnostic classifier for read, append-read, null-coalesce, `isset`, `empty`, and error-control forms.
- [x] Symbol-table ABI probe is pushed, but remains probe-only until real assignment/readback consumers land.

In progress but lane-local or not yet executable primary support:

- [ ] `impl-native-call-semantics` reports source-aware by-reference closure capture execution through multiple call consumers; not primary-integrated.
- [ ] `impl-native-error-diagnostic-semantics` reports backend selector cleanup for path-array, symbol, condition, and list-blocker consumers; not primary-integrated.
- [ ] `impl-native-type-conversion` contains broad shortcut-retirement evidence but is parked after inspection without a new bounded packet.
- [ ] `impl-array-linked-exec` reports cleanup/blocker ownership improvements but is parked after a cadence miss.
- [ ] `impl-symbol-integrator`, `impl-native-comparison-semantics`, and other broad/parked lanes remain evidence only until exact-scope current-primary prep.
- [ ] Full byte-exact tree-walk interpreter output surfaces remain blocked behind a real byte-output/session/debug formatting representation.
- [ ] Request/global key byte coercion for byte-backed strings remains intentionally blocked.
- [ ] Broader expression-family lvalue/reference-slot materialization is needed beyond variable-backed operands.
- [ ] Direct object `ArrayAccess` method dispatch remains blocked behind diagnostic-only classifier support.
- [ ] Alias-aware LLVM direct-root write-through after `=&` remains blocked for both statement assignment and assignment expressions.
- [ ] Object/resource source materialization for generic conversion sources remains blocked.
- [ ] LLVM offset-read error-status cleanup still needs a generalized control-flow boundary beyond the current helper consolidation.

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

- `2cd78ade`: conversion-result consumer helper consolidation. Integrated
  files: `compiler/src/codegen.rs` and
  `compiler/tests/native_runtime_abi.rs`. It shares current LLVM
  `NativeConversionResult` value/status/diagnostic extraction, guard, and
  source-handle cleanup across scalar offset reads and numeric unary source
  results. Proof included reviewed candidate hash
  `99de98d95f2d93efc4f6d54ed5e386513a8e917ce4787830472013d936da36f1`,
  apply proof, four focused nonzero tests, `cargo check`, `cargo fmt --check`,
  `git diff --check`, push proof, and final clean sync.
- `b13c85c6`: unary negation source/result ABI. Integrated files:
  `runtime/src/lib.rs`, `compiler/src/codegen.rs`, and
  `compiler/tests/native_runtime_abi.rs`.
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

- [x] Primary was clean and synced at `2cd78ade` before this `PROGRESS.md`
  edit.
- [x] Current integrated production/prerequisite head is `2cd78ade`.
- [x] Latest integrated executable capability baseline is `b13c85c6`.
- [x] Covered unary `-` lowering uses a shared runtime numeric-unary
  source/result ABI instead of backend-local primitive negation/folding.
- [x] Current LLVM scalar offset-read and numeric-unary source-result paths now
  share conversion-result extraction, diagnostics, failure/null guard, and
  source-handle cleanup.
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

- [ ] `impl-native-call-semantics` reports source-aware by-reference closure
  capture execution through direct, dynamic, nested, discard, and
  by-reference-return closure consumers. This needs fresh extraction/review.
- [ ] `impl-native-error-diagnostic-semantics` reports shared backend selector
  cleanup for path-array and symbol/object mutation consumers. This is useful
  cleanup evidence, not integrated capability.
- [ ] `impl-array-linked-exec` and `impl-native-type-conversion` are parked;
  use only as evidence for fresh current-primary packets.
- [ ] Broad dirty lanes should not be imported as integration units.

Resource posture:

- `/dev/shm`: live df `40G` total, `24G` used, `17G` available, 58% used.
  Largest observed targets are `phpc-target-native-call-semantics` at `8.9G`,
  `phpc-target-native-object-seed` at `5.6G`, and
  `phpc-target-native-diagnostics` at `3.0G`.
- `/home`: live df `459G` total, `203G` used, `238G` available, 46% used.
  Largest observed lane/work tree is
  `phpc-lane-native-error-diagnostic-semantics` at `14G`; primary is about
  `2.2G`.
- Memory: about `41Gi` available; swap remains high at `23Gi/29Gi`.
- Continue disk-backed target dirs, `umask 0007`, `CARGO_BUILD_JOBS=1`,
  `CARGO_INCREMENTAL=0`, and focused nonzero gates. No owner-checked cleanup is
  needed while `/dev/shm` remains this healthy.

## Next Steering Read

Best next compact packets to consider:

- A distinct call/frame/reference-capture packet from `impl-native-call-semantics`
  only if a fresh current-primary prep proves exact scope, applyability,
  independent review, and linked execution beyond generated-source assertions.
- A diagnostic selector cleanup packet from `impl-native-error-diagnostic-semantics`
  only if it is codegen-scoped, removes real duplicated current-primary
  backend selector construction, and is counted as cleanup unless it unlocks a
  concrete execution boundary.
- A focused lvalue/reference-slot or cleanup-boundary packet from parked
  array/reference lanes only after read-only inventory and a fresh exact-scope
  handoff.

Avoid:

- Re-landing or extending the post-unary conversion-result helper as if it were
  still pending; it is integrated at `2cd78ade`.
- Whole-lane imports from parked type-conversion, array-linked, symbol,
  comparison, or diagnostic worktrees.
- Percentage bumps for helper-only cleanup, blocker-only work, or lane-local
  claims that are not committed and pushed on primary.
