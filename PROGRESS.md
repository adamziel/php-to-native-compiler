# PHP Native Compiler Progress

Updated: 2026-05-25 19:17 CEST
Evaluation marker: `20260525T171704Z`

Accounting rule: only generalized, tested, committed, and pushed primary work
counts as integrated capability. Dirty WIP, candidate worktrees, lane-local
claims, review-only work, probe-only commits, and dashboard-only commits are
excluded.

## Executive Read

Overall estimated progress: **91%** `[##################--]`

Executable PHP semantics: **91%** `[##################--]`

Primary is clean and aligned with `origin/master` at
`b4789b5c docs: update progress dashboard`. This is a docs-only evaluator
head. The latest counted semantic/prerequisite baseline remains
`5307990c native: add string-array operation slots`.

No new primary semantic commit landed in this review window. Current primary
capability stays flat: byte-backed PHP string values and byte-preserving
`explode()` / `str_split()` string-array operation slots are integrated, but
the new non-local assignment owner-cell packet is still only review-approved
lane-local candidate work until a dedicated integrator applies, tests,
commits, and pushes it.

Full generalized PHP remains blocked on references/COW identity, arbitrary
lvalues, request/global parity and writeback, includes, variable variables,
broad userland frames, real `ArrayAccess`, object/magic/visibility/destructor
semantics, cleanup/unwind/finally/shutdown ordering, exact diagnostics/error
handlers, and backend parity.

## Primary-Integrated Baseline

- Current primary head before this dashboard edit:
  `b4789b5c docs: update progress dashboard`.
- Latest integrated executable/prerequisite semantic baseline:
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
| Runtime and ABI foundations | **99%** | `[####################]` | Strong selected-path value, byte-string, string-array, array, diagnostic, reference, symbol, call-frame, object, comparison, conversion, owner-cell, request-state, offset-read, array-key, type/int, text-membership, comparison, and truthiness slot surfaces. |
| Compiler/backend consumers | **99%** | `[####################]` | Generated C and LLVM consume many shared ABIs, request-backed array-key blockers, byte-backed string values, and string-array operation slots. Direct assembly and some generated-C-only surfaces still lag. |
| Executable PHP semantics | **91%** | `[##################--]` | Primary has closure/callable/object islands, bounded preg callbacks, object-property reference-slot mutation, offset-read continuation proof, reference-backed array-key conversion, type/int, text-membership, comparison, truthiness consumers, request-key blocker parity, byte-backed string values, and byte-preserving `explode()` / `str_split()` slot proof. |
| Strings and byte semantics | **62%** | `[############--------]` | Byte-backed values and shared byte-preserving string-array results are integrated. Full byte-exact interpreter output, binary source bytes, `mb_str_split()` codepoint semantics, request/global keys, and exact diagnostics remain open. |
| Arrays, lvalues, references, COW | **77%** | `[###############-----]` | Value/reference slot ABI reuse is expanding and unsafe request-backed key materialization is blocked. Full COW, arbitrary roots, foreach, broader expression reference slots, and alias composition remain open. |
| Symbols, globals, request state | **73%** | `[###############-----]` | Selected function globals, root-symbol surfaces, active symbol-table reference consumers, and request-backed key blockers exist. `$GLOBALS` self-cells, request/global alias parity, request writeback, includes, variable variables, and exact unset/global behavior remain incomplete. |
| Calls, functions, frames | **83%** | `[#################---]` | Selected direct/callable/function-table surfaces exist. Named/unpacked/by-reference/userland frame breadth remains incomplete. |
| Objects, properties, methods | **50%** | `[##########----------]` | Object-property reference-slot mutation and diagnostic classifiers are integrated. Full visibility, magic, dynamic/static/typed properties, destructors, references/COW, and `ArrayAccess` execution remain open. |
| Control flow, cleanup, diagnostics | **51%** | `[##########----------]` | Selected branches, loops, transfers, finalizers, output buffers, diagnostics, and truthiness consumers exist. Broad unwind/finally/destructor/shutdown and exact source ordering remain open. |
| Broad integrated verification | **90%** | `[##################--]` | Focused primary gates are strong for recent packets. Broad gates remain constrained by lane extraction cost, high swap, stale lane expectations, and backend parity gaps. |

## Active Roadmap Items

| Item | Toward Primary Integration | Toward Full Feature | Status |
| --- | ---: | ---: | --- |
| String operation-family slot consumers | **100%** `[####################]` | **45%** `[#########-----------]` | Integrated at `5307990c`. Runtime, LLVM, generated C, and linked execution route `explode()` and `str_split()` through shared byte-preserving value/reference-slot string-array contracts. `mb_str_split()`, binary source parsing, request/global execution, and broad multibyte semantics remain open. |
| Byte-preserving PHP string value boundary | **100%** `[####################]` | **40%** `[########------------]` | Integrated at `1c369d0f`. Runtime has `Value::BinaryString(Vec<u8>)`, `value_from_php_string_bytes(Vec<u8>)`, byte-view/runtime string-family preservation, native pointer-plus-length materialization, and linked byte-output proof. Full byte-output interpreter surfaces, binary source syntax, and request/global key coercion remain open. |
| Request-backed array-key/RMW blocker parity | **100%** `[####################]` | **37%** `[#######-------------]` | Integrated at `a501c4d1`. LLVM and generated C share blocker classification for request-backed ordinary array-key consumers across selected read, assignment, unset, reference assignment, for-action assignment/RMW, compound assignment, `??=`, and increment/decrement paths. Blocker-only: no request storage/writeback or `$GLOBALS` parity. |
| Reference-slot consumer families | **100%** `[####################]` | **45%** `[#########-----------]` | Type/int, text-membership, comparison, truthiness, array-key, and offset-read value/reference slots are integrated for reviewed selected paths. Full alias/COW composition remains open. |
| Non-local assignment owner-cell blocker refresh | **90%** `[##################--]` | **33%** `[#######-------------]` | Prep and independent review report `go-for-primary-integrator`. Exact scope is `compiler/src/codegen.rs` plus `compiler/tests/native_array_boundary.rs`, hash `fa382d902123bde2af0759bb4bcdfddf599d84d8cb457a41dd49691fd277f003`, with nonzero owner-family gates and apply proof against `b4789b5c`. Not primary-integrated yet. |
| Broader lvalue/reference-slot materializer | **30%** `[######--------------]` | **39%** `[########------------]` | Needed so non-variable expression families that can carry references can enter shared array-key and consumer slot ABIs safely. Should stay focused on slot materialization, not foreach or `ArrayAccess` execution. |
| Object/resource source materialization | **25%** `[#####---------------]` | **30%** `[######--------------]` | Explicit blocker left by the offset-read ABI. Needs a general value reconstruction boundary before generic object/resource consumers are safe. |
| LLVM offset-read/error-status cleanup | **25%** `[#####---------------]` | **30%** `[######--------------]` | Offset-read diagnostics exist, but LLVM still needs a generalized control-flow/error-exit status boundary for failed conversion results. |
| Callable-object/dynamic-constructor candidates | **52%** `[##########----------]` | **42%** `[########------------]` | Useful May 24 candidates remain stale relative to current primary and May 25 slot/request/string integrations. Refresh from current primary before review and do not combine them. |
| Diagnostics, request, and cleanup boundaries | **64%** `[#############-------]` | **41%** `[########------------]` | Lane-local control-flow, symbol, call, type-conversion, array, diagnostic, and reference-cell work continues producing useful boundaries, but parked/broad lanes are evidence only until extracted into exact current-primary packets. |
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
- [x] Shared byte-preserving `explode()` and `str_split()` string-array operation slots across runtime, LLVM, generated C, and linked execution.

Primary-integrated non-executable infrastructure:

- [x] Object-offset `ArrayAccess` receiver diagnostic classifier for read, append-read, null-coalesce, `isset`, `empty`, and error-control forms.
- [x] Symbol-table ABI probe is pushed, but remains probe-only until real assignment/readback consumers land.

In progress but lane-local or not yet executable primary support:

- [ ] Non-local assignment owner-cell blocker refresh is independently reviewed with `go-for-primary-integrator`, but is not committed or pushed on primary.
- [ ] `impl-native-type-conversion` is removing exact-shape logical/ternary shortcuts and replacing them with generalized conversion-result paths.
- [ ] `impl-native-call-semantics` is centralizing runtime PHP-truthiness condition materialization for call-result values.
- [ ] `impl-symbol-integrator` and `impl-array-linked-exec` continue producing focused boundaries for casts, class-name operands, static-local/default preflight, cleanup, lvalue key cleanup, and switch/try protected exits. These are lane-local until extracted.
- [ ] Several broad lanes are parked after cadence failures or conflict-heavy exploration. They must be reclaimed only through fresh, exact-scope current-primary prep.
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

- `b4789b5c`: progress-dashboard commit only. No executable compiler/runtime
  semantic code changed.
- `1a43fe12`: progress-dashboard commit only. No executable compiler/runtime
  semantic code changed.
- `5307990c`: string-array operation slots. Integrated files:
  `runtime/src/lib.rs`, `compiler/src/codegen.rs`,
  `compiler/tests/native_runtime_abi.rs`, and `compiler/tests/native_link.rs`.
  Review/integration proof included exact hash/scope/apply checks, four
  nonzero focused gates, `cargo check`, `cargo fmt --check`,
  `git diff --check`, push proof, and clean post-push state.
- `1c369d0f`: byte-backed PHP string value boundary. Integrated files:
  `runtime/src/lib.rs`, `compiler/src/interpreter.rs`,
  `compiler/tests/native_runtime_abi.rs`, and `compiler/tests/native_link.rs`.
- `a501c4d1`: request-backed array-key/RMW blocker parity. Integrated files:
  `compiler/src/codegen.rs`, `compiler/tests/native_runtime_abi.rs`, and
  `compiler/tests/superglobals.rs`.
- `24ec4a10`: reference truthiness slots. Integrated files:
  `runtime/src/lib.rs`, `compiler/src/codegen.rs`,
  `compiler/tests/native_runtime_abi.rs`, and `compiler/tests/native_link.rs`.

## Current Work Snapshot

Primary-integrated:

- [x] Primary was clean and synced at `b4789b5c` before this `PROGRESS.md`
  edit.
- [x] Latest counted semantic/prerequisite commit remains `5307990c`.
- [x] Byte-backed PHP string value representation remains integrated with
  native IR/C materialization and linked executable byte-output proof.
- [x] `explode()` and `str_split()` have shared byte-preserving string-array
  operation slots across runtime, LLVM, generated C, and linked execution.
- [x] Request-backed ordinary array-key/RMW consumers share blocker
  classification across selected LLVM and generated-C paths.
- [x] Reference-held truthiness, comparison, text-membership, type/int,
  array-key value/reference slots, and offset-read source-result support remain
  integrated for reviewed selected paths.
- [x] No uncommitted primary implementation diffs were present before this
  `PROGRESS.md` edit.

Lane-local:

- [ ] `non-local-assignment-owner-cell-blocker-refresh-review.status.md`
  reports `Decision: go-for-primary-integrator` at 19:17 CEST. It is the best
  immediate primary candidate, but remains uncounted until integrated.
- [ ] `impl-native-type-conversion` and `impl-native-call-semantics` continue
  producing focused lane-local conversion/call-result proofs with nonzero
  gates.
- [ ] `impl-symbol-integrator`, `impl-array-linked-exec`,
  `impl-native-error-diagnostic-semantics`, and adjacent lanes remain packet
  sources only unless a fresh exact-scope review artifact is produced.
- [ ] Parked broad lanes must be treated as preserved evidence, not reviewable
  integration units.

Resource posture:

- `/dev/shm`: live df `40G` total, `24G` used, `17G` available, 58% used.
  Largest observed target dirs include `phpc-target-native-call-semantics`
  at `8.9G`, `phpc-target-native-object-seed` at `5.6G`, and
  `phpc-target-native-diagnostics` at `3.0G`.
- `/home`: live df `459G` total, `212G` used, `228G` available, 49% used.
- Memory: `43Gi` total, about `41Gi` available; swap remains high at
  `23Gi/29Gi`.
- Continue using disk-backed target dirs, `umask 0007`, `CARGO_BUILD_JOBS=1`,
  `CARGO_INCREMENTAL=0`, and focused nonzero gates.

## Next Steering Read

Best next compact packets to consider:

- the non-local assignment owner-cell blocker refresh, if the dedicated
  integrator rechecks live primary and the reviewed diff still applies;
- a broader lvalue/reference materializer prerequisite, if it stays focused on
  enabling shared slot ABIs rather than adding foreach or `ArrayAccess`
  execution;
- a refreshed callable-object, dynamic-constructor, object-instantiation, or
  destructor-blocker packet, after rebasing and reviewing from current primary;
- a control-flow cleanup packet only after parked switch/control-flow work is
  re-extracted from current primary with exact scope and broader gates;
- a distinct string/byte packet only if it targets a real remaining boundary:
  `mb_str_split()` codepoint semantics, byte-exact source/output, or
  request/global byte-key parity.

Do not count lane-local triage, stopped dirty lanes, parked broad lanes, stale
May 24 candidates, callback/filesystem-family accumulation, blocker-only
metadata, review-only candidates, or docs-only commits as product capability.
