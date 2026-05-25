# PHP Native Compiler Progress

Updated: 2026-05-25 23:08 CEST
Evaluation marker: `20260525T210830Z`

Accounting rule: only generalized, tested, committed, and pushed primary work
counts as integrated capability. Dirty WIP, candidate worktrees, lane-local
claims, review-only work, failed prep proofs, probe-only commits, and
dashboard-only commits are excluded.

## Executive Read

Overall estimated progress: **93%** `[###################-]`

Executable PHP semantics: **93%** `[###################-]`

Primary was clean and aligned with `origin/master` at
`7c8fccba docs: update progress dashboard` before this `PROGRESS.md` edit.

Latest primary-integrated executable capability baseline:
`ae93da8c native: add closure reference return result ABI`.

This review keeps overall and executable estimates unchanged at 93%. No new
source-semantic commit landed after the previous progress update. The useful
new work is steering/current-candidate movement: post-closure triage picked
reference-source append/lvalue extraction as the next packet, and a fresh
candidate prep started from current primary with no diff or gates yet.

## Grand Roadmap Position

| Workstream | Estimate | Bar | Current read |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | **99%** | `[####################]` | Strong selected-path value, byte-string, string-array, array, diagnostic, reference, symbol, call-frame, object, comparison, conversion, numeric-unary, owner-cell, request-state, offset-read, array-key, type/int, text-membership, truthiness, and closure result surfaces. |
| Compiler/backend consumers | **99%** | `[####################]` | LLVM and generated C share many ABIs. Generated-C has the freshest executable semantics; direct assembly and some LLVM parity still lag. |
| Executable PHP semantics | **93%** | `[###################-]` | Primary has many selected semantic islands, including closure value/reference return consumers, bounded preg callbacks, object-property reference-slot mutation, byte strings, string-array slots, array-key/reference-slot consumers, non-local blockers, request-key blockers, and unary negation through shared conversion ABI. |
| Strings and byte semantics | **62%** | `[############--------]` | Byte-backed values and byte-preserving `explode()` / `str_split()` slots are integrated. Binary source bytes, byte-exact interpreter output/session/debug formatting, `mb_str_split()` codepoint behavior, request/global byte keys, and exact diagnostics remain open. |
| Arrays, lvalues, references, COW | **79%** | `[################----]` | Value/reference slot ABIs continue to expand, and closure reference returns can bind through selected symbol-reference consumers. Full COW, arbitrary lvalue roots, foreach, broader expression reference slots, alias composition, and writeback remain incomplete. |
| Symbols, globals, request state | **73%** | `[###############-----]` | Selected function globals, root-symbol surfaces, active symbol-table reference consumers, request-backed key blockers, and request-key diagnostics exist. `$GLOBALS` self-cells, request/global alias parity, request writeback, includes, variable variables, and exact unset/global behavior remain incomplete. |
| Calls, functions, frames | **86%** | `[#################---]` | Selected direct/callable/function-table, descriptor-closure, capture, method-frame, and descriptor closure value/reference return surfaces exist. General user-function/method/static/constructor reference returns, non-descriptor closures, named/unpacked breadth, and exact callable semantics remain open. |
| Objects, properties, methods | **51%** | `[##########----------]` | Object-property reference-slot mutation, diagnostic classifiers, declared-object blockers, and non-local object/static owner blockers are integrated. Full visibility, magic, dynamic/static/typed properties, destructors, references/COW, and `ArrayAccess` execution remain open. |
| Control flow, cleanup, diagnostics | **51%** | `[##########----------]` | Selected branches, loops, transfers, finalizers, output buffers, diagnostics, truthiness, and conversion consumers exist. Broad unwind/finally/destructor/shutdown and exact source ordering remain open. |
| Broad integrated verification | **91%** | `[##################--]` | Focused primary gates are strong for recent packets. The full `native_runtime_abi` suite still has known current-primary failures, and broad gates remain constrained by lane extraction cost, stale lane expectations, high swap, and backend parity gaps. |

## Active Roadmap Items

Primary-integrated items are separated from lane-local candidate work below.

| Item | Toward Primary Integration | Toward Full Feature | Status |
| --- | ---: | ---: | --- |
| Closure value/reference return ABI | **100%** `[####################]` | **50%** `[##########----------]` | Integrated at `ae93da8c`. Runtime closure invocation has a shared value/reference/diagnostic/status result contract; generated-C descriptor closures route value and reference returns through result consumers. Broader call/reference semantics remain open. |
| Request-key operation selector cleanup | **100%** `[####################]` | **34%** `[#######-------------]` | Integrated at `22f56b67`. Cleanup/prerequisite only; full request/global behavior remains open. |
| Conversion-result helper and shortcut retirement | **100%** `[####################]` | **44%** `[#########-----------]` | Integrated at `2cd78ade` for current LLVM scalar offset-read and numeric-unary source-result consumers. Broader conversion families, object/resource coercions, exact diagnostics, error control, and cleanup/unwind remain open. |
| Runtime numeric-unary conversion ABI | **100%** `[####################]` | **48%** `[##########----------]` | Integrated at `b13c85c6`. Covered unary `-` routes through `NativeConversionSource` / `NativeConversionResult` across runtime, LLVM, generated C, and focused linked execution. |
| String operation-family slot consumers | **100%** `[####################]` | **45%** `[#########-----------]` | Integrated at `5307990c`; `explode()` and `str_split()` use shared byte-preserving value/reference-slot contracts. |
| Byte-preserving PHP string value boundary | **100%** `[####################]` | **40%** `[########------------]` | Integrated at `1c369d0f`; byte-backed PHP values and native pointer-plus-length materialization are available for selected paths. |
| Reference-source append/lvalue extraction | **5%** `[#-------------------]` | **40%** `[########------------]` | Fresh post-closure candidate prep started from `7c8fccba`; no dirty files or gates yet. Lane evidence is promising but not counted. |
| Dynamic callable class-context dispatch | **0%** `[--------------------]` | **42%** `[########------------]` | Ranked fallback after reference-source extraction. Useful call-lane evidence exists, but no current-primary candidate packet is open for it. |
| Native source-call cleanup and reference-source lanes | **30%** `[######--------------]` | **43%** `[#########-----------]` | `impl-native-call-semantics` has useful lane-local dynamic callable dispatch, source-call preflight, source-call target ownership, append-reference, and result-consumer evidence; not primary-integrated. |
| Diagnostic result sink/write-operation contracts | **32%** `[######--------------]` | **43%** `[#########-----------]` | `impl-native-error-diagnostic-semantics` has lane-local diagnostic operation contracts. Its huge dirty scope means no direct integration. |
| Broader lvalue/reference-slot materializer | **32%** `[######--------------]` | **41%** `[########------------]` | Needed so non-variable expression families that can carry references can enter shared array-key, call, and consumer slot ABIs safely. Current broad lane evidence remains parked or lane-local. |
| Object/resource source materialization | **25%** `[#####---------------]` | **30%** `[######--------------]` | Still a recurring blocker for generic conversion and offset/source consumers. Needs a general value reconstruction/materialization boundary. |
| Broad lane extraction backlog | **34%** `[#######-------------]` | **35%** `[#######-------------]` | Broad dirty lanes remain useful evidence repositories, not integration units. Several are parked, stale, or only current as lane-local artifacts. |

## Done / In Progress / Not Done

Primary-integrated executable or executable-prerequisite capability:

- [x] Descriptor-backed closures, selected captures, selected by-reference parameters, and selected callable-array/object invocation.
- [x] Shared closure invocation result ABI for descriptor closure value returns, reference returns, value-consumer reference cloning, and reference-assignment binding.
- [x] Bounded `preg_replace_callback()` string-callback execution over supported slash-delimited patterns.
- [x] Object-property assignment/unset mutation for covered reference-backed operands through generated-C/native-link shared slot boundaries.
- [x] Shared offset-read source-result ABI for scalar/resource warning continuations, arrays, byte strings, references, and object-property offset-source composition.
- [x] Shared array-key value/reference-slot ABI for generated-native reference-backed variable operands and active symbol-table variable references.
- [x] Shared reference-slot type-name/type-predicate/int, text-byte/text-membership, comparison, and truthiness consumers for selected runtime, LLVM, and generated-C paths.
- [x] Shared request-backed ordinary array-key/RMW blocker classification for selected LLVM and generated-C consumers.
- [x] Shared request-state operation selector for existing request-key/path/bag mutation consumers.
- [x] Byte-backed PHP string value representation and native pointer-plus-length materialization for arbitrary PHP string bytes.
- [x] Shared byte-preserving `explode()` and `str_split()` string-array operation slots across runtime, LLVM, generated C, and linked execution.
- [x] Shared non-local assignment/unset owner-family blockers for object/static property assignment and unset paths across current backend lowering families.
- [x] Shared numeric-unary source/result ABI for covered unary negation across runtime, LLVM, generated C, and focused linked execution.
- [x] Shared LLVM conversion-result consumer helper for current scalar offset-read and numeric-unary source-result paths.

In progress but lane-local or not yet executable primary support:

- [x] Post-closure next-candidate triage ran from current primary `7c8fccba` and ranked reference-source append/lvalue extraction first.
- [ ] `reference-source-append-lvalue-prep` has a fresh candidate worktree from `7c8fccba`; no product diff or gates yet.
- [ ] `impl-native-call-semantics` has lane-local callable dispatch, caller-scope, dynamic callable preflight, source-call cleanup, append-reference, and reference/call evidence; not primary-integrated.
- [ ] `impl-native-error-diagnostic-semantics` has lane-local diagnostic operation contracts; not primary-integrated.
- [ ] Broader closure/call reference returns need reusable consumers beyond descriptor closures: user functions, methods, static calls, constructors, discarded calls, and non-descriptor closure surfaces.
- [ ] Broad parked lanes remain evidence only until exact current-primary prep, review, and integration.

Not done:

- [ ] Full references/COW identity, arbitrary alias roots, and alias-preserving write-through.
- [ ] Executable request storage/writeback, `$GLOBALS` self-cells, request/global alias parity, request foreach, and mutation-during-iteration behavior.
- [ ] Includes, variable variables, and dynamic symbol behavior.
- [ ] Full callable lookup and invocation, including named/unpacked/by-reference breadth, closures, arrays, invokable objects, magic/visibility, and rebinding rules.
- [ ] Runtime `ArrayAccess` method dispatch for `offsetGet`, `offsetExists`, `offsetSet`, and `offsetUnset`.
- [ ] Binary literal syntax, invalid-UTF-8 PHP source parsing, byte-exact interpreter output/session/debug formatting, and `mb_str_split()` generalized multibyte/codepoint splitting.
- [ ] Full PCRE behavior beyond the bounded slash-delimited subset.
- [ ] General object model: non-public methods, overrides, interfaces/traits execution, magic methods, dynamic/static/typed properties, destructors.
- [ ] Complete cleanup/unwind/finally/destructor/output-buffer shutdown behavior.
- [ ] Exact/source-ordered diagnostics, custom handler execution, warning/error continuation, and suppression parity.
- [ ] LLVM/direct assembly parity for recent generated-C semantics.
- [ ] Known current-primary full `native_runtime_abi` baseline failures.

## Recent Primary-Integrated Work

- `ae93da8c`: closure value/reference return result ABI. Integrated exactly
  `runtime/src/lib.rs`, `compiler/src/codegen.rs`,
  `compiler/tests/native_function_call_boundary.rs`, and
  `compiler/tests/native_runtime_abi.rs`; 10 required gates passed, including
  6 nonzero focused tests.
- `22f56b67`: request-key operation selector cleanup. Integrated exactly
  `compiler/src/codegen.rs`; focused gates and zero-test caveat handling were
  recorded by the primary integrator.
- `2cd78ade`: conversion-result consumer helper consolidation for current LLVM
  scalar offset-read and numeric-unary source-result consumers.
- `b13c85c6`: unary negation source/result ABI across runtime, LLVM, generated
  C, and focused linked execution.
- `f770d728`: non-local assignment owner blockers across current backend
  lowering families.
- `5307990c`: string-array operation slots for byte-preserving `explode()` and
  `str_split()`.
- `1c369d0f`: byte-backed PHP string value boundary.

## Current Work Snapshot

Primary-integrated:

- [x] Primary was clean and synced at `7c8fccba` before this `PROGRESS.md` edit.
- [x] Latest executable capability head remains `ae93da8c`.
- [x] Overall and executable percentages remain at 93% for this review.
- [x] The closure ABI packet is a non-repeat guard, not active work.

Lane-local:

- [ ] `reference-source-append-lvalue-prep` is open against `7c8fccba`; no
  product diff or gates yet.
- [ ] `impl-native-call-semantics` reported fresh dynamic callable operand
  preflight evidence at 2026-05-25 23:06 CEST. It remains a broad dirty lane
  and should be mined only through fresh current-primary packets.
- [ ] `impl-native-error-diagnostic-semantics` remains lane-local and too broad
  for direct import.
- [ ] Multiple broad lanes are parked or noncompliant-cadence evidence
  repositories. Do not route them to primary without a new narrow extraction.

Resource posture:

- `/dev/shm`: 40G total, 24G used, 17G available, 58% used.
  Largest observed targets: `phpc-target-native-call-semantics` 8.9G,
  `phpc-target-native-object-seed` 5.6G,
  `phpc-target-native-diagnostics` 3.0G.
- `/home`: 459G total, 193G used, 248G available, 44% used.
  Largest observed lane/work tree:
  `phpc-lane-native-error-diagnostic-semantics` 14G.
- Memory available is about 41Gi, but swap remains high at 23Gi/29Gi used.
- Continue disk-backed `/tmp` target dirs, `umask 0007`,
  `CARGO_BUILD_JOBS=1`, `CARGO_INCREMENTAL=0`, and focused nonzero gates.

## Next Steering Read

Best next action:

- Let `reference-source-append-lvalue-prep` continue as the next candidate, but
  require exact current-primary scope, stable hash, clean apply proof,
  independent review, and focused nonzero gates before any integration.

Fallback executable semantic packet:

- If append/lvalue extraction returns `needs-split` or proves too broad,
  consider dynamic callable class-context invocation / callable-value dispatch
  before more cleanup-only diagnostic selector/report-sink work.

Good cleanup discipline to keep:

- Treat `ae93da8c`, `22f56b67`, `2cd78ade`, and `b13c85c6` as completed
  non-repeat work.
- Keep zero-test filters out of proof accounting.
- Require current-primary base, exact dirty-file scope, stable hash, clean apply
  proof, independent review, and focused nonzero gates before any integration.

Avoid:

- Percentage bumps for candidate creation, selector-only cleanup, helper-only
  cleanup, blocker-only work, or lane-local claims that are not committed and
  pushed on primary.
- Routing broad dirty call/diagnostic lanes directly to primary.
- Repeating descriptor closure result ABI work under a new name.
- Letting cleanup-only diagnostic operation vocabulary displace executable PHP
  semantic packets while a reference/call packet is available.
