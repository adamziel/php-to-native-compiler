# PHP Native Compiler Progress

Updated: 2026-05-25 22:08 CEST
Evaluation marker: `20260525T200843Z`

Accounting rule: only generalized, tested, committed, and pushed primary work
counts as integrated capability. Dirty WIP, candidate worktrees, lane-local
claims, review-only work, failed prep proofs, probe-only commits, and
dashboard-only commits are excluded.

## Executive Read

Overall estimated progress: **92%** `[##################--]`

Executable PHP semantics: **92%** `[##################--]`

Primary was clean and aligned with `origin/master` at
`22f56b67 native: share request key operation selector` before this
`PROGRESS.md` edit.

Latest primary-integrated production/prerequisite baseline:
`22f56b67 native: share request key operation selector`.

Latest primary-integrated executable capability baseline:
`b13c85c6 native: add unary negation source result ABI`.

This review keeps overall and executable percentages flat. `22f56b67` is useful
primary-integrated cleanup: it centralizes existing request-state operation ABI
consumer selection across LLVM/generated-C paths. It does not broaden executable
PHP request/global semantics.

## Grand Roadmap Position

| Workstream | Estimate | Bar | Current read |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | **99%** | `[####################]` | Strong selected-path value, byte-string, string-array, array, diagnostic, reference, symbol, call-frame, object, comparison, conversion, numeric-unary, owner-cell, request-state, offset-read, array-key, type/int, text-membership, truthiness, and non-local owner-boundary surfaces. |
| Compiler/backend consumers | **99%** | `[####################]` | LLVM and generated C share many ABIs. `22f56b67` adds a shared request-state operation selector; direct assembly and some generated-C-only surfaces still lag. |
| Executable PHP semantics | **92%** | `[##################--]` | Primary has many selected semantic islands, including closure/callable/object paths, bounded preg callbacks, object-property reference-slot mutation, byte strings, string-array slots, array-key/reference-slot consumers, non-local blockers, and unary negation through the shared conversion ABI. |
| Strings and byte semantics | **62%** | `[############--------]` | Byte-backed values and byte-preserving `explode()` / `str_split()` slots are integrated. Binary source bytes, byte-exact interpreter output/session/debug formatting, `mb_str_split()` codepoint behavior, request/global byte keys, and exact diagnostics remain open. |
| Arrays, lvalues, references, COW | **78%** | `[################----]` | Value/reference slot ABIs continue to expand, but full COW, arbitrary lvalue roots, foreach, broader expression reference slots, alias composition, and writeback remain incomplete. |
| Symbols, globals, request state | **73%** | `[###############-----]` | Selected function globals, root-symbol surfaces, active symbol-table reference consumers, request-backed key blockers, and request-key diagnostics exist. `$GLOBALS` self-cells, request/global alias parity, request writeback, includes, variable variables, and exact unset/global behavior remain incomplete. |
| Calls, functions, frames | **83%** | `[#################---]` | Selected direct/callable/function-table, descriptor-closure, capture, and method-frame surfaces exist. The active next packet is closure value/reference return ABI prep, still lane-local. |
| Objects, properties, methods | **51%** | `[##########----------]` | Object-property reference-slot mutation, diagnostic classifiers, and non-local object/static owner blockers are integrated. Full visibility, magic, dynamic/static/typed properties, destructors, references/COW, and `ArrayAccess` execution remain open. |
| Control flow, cleanup, diagnostics | **51%** | `[##########----------]` | Selected branches, loops, transfers, finalizers, output buffers, diagnostics, and truthiness/conversion consumers exist. Broad unwind/finally/destructor/shutdown and exact source ordering remain open. |
| Broad integrated verification | **91%** | `[##################--]` | Focused primary gates are strong for recent packets. Broad gates remain constrained by lane extraction cost, high swap, stale lane expectations, and backend parity gaps. |

## Active Roadmap Items

| Item | Toward Primary Integration | Toward Full Feature | Status |
| --- | ---: | ---: | --- |
| Request-key operation selector cleanup | **100%** `[####################]` | **34%** `[#######-------------]` | Integrated at `22f56b67`. Counts as cleanup/prerequisite only; full request/global behavior remains open. |
| Conversion-result helper and shortcut retirement | **100%** `[####################]` | **44%** `[#########-----------]` | Integrated at `2cd78ade` for current LLVM scalar offset-read and numeric-unary source-result consumers. Broader conversion families, object/resource coercions, exact diagnostics, error control, and cleanup/unwind remain open. |
| Runtime numeric-unary conversion ABI | **100%** `[####################]` | **48%** `[##########----------]` | Integrated at `b13c85c6`. Covered unary `-` routes through `NativeConversionSource` / `NativeConversionResult` across runtime, LLVM, generated C, and focused linked execution. |
| String operation-family slot consumers | **100%** `[####################]` | **45%** `[#########-----------]` | Integrated at `5307990c`; `explode()` and `str_split()` use shared byte-preserving value/reference-slot contracts. |
| Byte-preserving PHP string value boundary | **100%** `[####################]` | **40%** `[########------------]` | Integrated at `1c369d0f`; byte-backed PHP values and native pointer-plus-length materialization are available for selected paths. |
| Closure value/reference return ABI | **25%** `[#####---------------]` | **33%** `[#######-------------]` | Active candidate prep is visible in `main:30`, but not reviewed or integrated. It must prove one reusable closure result contract for value/reference returns. |
| Native source-call cleanup and reference-source lanes | **20%** `[####----------------]` | **39%** `[########------------]` | `impl-native-call-semantics` has useful lane-local source-call cleanup, dynamic callable, append-reference, and result-consumer evidence; not primary-integrated. |
| Diagnostic result sink/binary-operation contracts | **25%** `[#####---------------]` | **41%** `[########------------]` | `impl-native-error-diagnostic-semantics` has useful lane-local runtime diagnostic consolidation; adjacent property-family gate failures require fresh extraction/review. |
| Broader lvalue/reference-slot materializer | **30%** `[######--------------]` | **39%** `[########------------]` | Needed so non-variable expression families that can carry references can enter shared array-key and consumer slot ABIs safely. Current broad lane evidence is parked or lane-local. |
| Object/resource source materialization | **25%** `[#####---------------]` | **30%** `[######--------------]` | Still a recurring blocker for generic conversion and offset/source consumers. Needs a general value reconstruction/materialization boundary. |
| Broad lane extraction backlog | **34%** `[#######-------------]` | **35%** `[#######-------------]` | Broad dirty lanes remain useful evidence repositories, not integration units. Several are parked or only current as lane-local artifacts. |

## Done / In Progress / Not Done

Primary-integrated executable or executable-prerequisite capability:

- [x] Descriptor-backed closures, selected captures, selected by-reference parameters, and selected callable-array/object invocation.
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

- [ ] `main:30` is actively prepping closure value/reference return ABI work from current primary; it is not reviewed or integrated.
- [ ] Closure by-reference returns need a reusable result ABI and reference-result consumers before broader call/reference semantics can count.
- [ ] `impl-native-call-semantics` has lane-local caller-scope, dynamic callable, source-call cleanup, append-reference, and reference/call evidence; not primary-integrated.
- [ ] `impl-native-error-diagnostic-semantics` has lane-local report-sink and binary-operation contract evidence; not primary-integrated.
- [ ] Broad parked lanes remain evidence only until exact current-primary prep, review, and integration.

Not done:

- [ ] Full references/COW identity, arbitrary alias roots, and alias-preserving write-through.
- [ ] Executable request storage/writeback, `$GLOBALS` self-cells, request/global alias parity, request foreach, and mutation-during-iteration behavior.
- [ ] Includes, variable variables, and dynamic symbol behavior.
- [ ] Full callable lookup and invocation, including named/unpacked/by-reference breadth, closures, arrays, invokable objects, magic/visibility, and rebinding rules.
- [ ] Runtime `ArrayAccess` method dispatch for `offsetGet`, `offsetExists`, `offsetSet`, and `offsetUnset`.
- [ ] Binary literal syntax, invalid-UTF-8 PHP source parsing, byte-exact output/session/debug formatting, and `mb_str_split()` generalized multibyte/codepoint splitting.
- [ ] Full PCRE behavior beyond the bounded slash-delimited subset.
- [ ] General object model: non-public methods, overrides, interfaces/traits execution, magic methods, dynamic/static/typed properties, destructors.
- [ ] Complete cleanup/unwind/finally/destructor/output-buffer shutdown behavior.
- [ ] Exact/source-ordered diagnostics, custom handler execution, warning/error continuation, and suppression parity.
- [ ] LLVM/direct assembly parity for recent generated-C semantics.

## Recent Primary-Integrated Work

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

- [x] Primary was clean and synced at `22f56b67` before this `PROGRESS.md` edit.
- [x] Latest production/prerequisite head is `22f56b67`.
- [x] Latest executable capability head remains `b13c85c6`.
- [x] Overall and executable percentages remain flat for this review.

Lane-local:

- [ ] Closure value/reference return ABI prep is active in a candidate worktree
  and is the best next executable direction if it stays scoped.
- [ ] Active call-lane source-call cleanup, dynamic callable, and append-reference
  evidence should be mined only through fresh current-primary packets.
- [ ] Active diagnostic-lane report-sink and binary-operation contracts are useful
  evidence, but adjacent failures mean no direct integration.
- [ ] The supervisor dashboard is stale relative to 22:00+ CEST status files;
  use worker statuses and primary git state for current steering until refreshed.

Resource posture:

- `/dev/shm`: 40G total, 24G used, 17G available, 58% used.
  Largest observed targets: `phpc-target-native-call-semantics` 8.9G,
  `phpc-target-native-object-seed` 5.6G,
  `phpc-target-native-diagnostics` 3.0G.
- `/home`: 459G total, 201G used, 239G available, 46% used.
  Largest observed lane/work tree:
  `phpc-lane-native-error-diagnostic-semantics` 14G; primary is about 2.2G.
- Memory available is about 41Gi, but swap remains high at 23Gi/29Gi used.
- Continue disk-backed `/tmp` target dirs, `umask 0007`,
  `CARGO_BUILD_JOBS=1`, `CARGO_INCREMENTAL=0`, and focused nonzero gates.

## Next Steering Read

Best next executable semantic packet to consider:

- Closure value/reference return ABI with a reusable runtime result contract,
  explicit diagnostic/cleanup ownership, and compiler consumers for value return,
  reference return, nested call consumers, discarded calls, and reference
  assignment sources.

Good cleanup discipline to keep:

- Treat `22f56b67` and `2cd78ade` as completed prerequisite cleanup, not pending
  semantic work.
- Keep zero-test filters out of proof accounting.
- Require current-primary base, exact dirty-file scope, stable hash, clean apply
  proof, independent review, and focused nonzero gates before any integration.

Avoid:

- Percentage bumps for selector-only cleanup, helper-only cleanup, blocker-only
  work, or lane-local claims that are not committed and pushed on primary.
- Routing broad dirty call/diagnostic lanes directly to primary.
- Letting closure-return prep expand into parser/AST churn, fixture sprawl,
  whole-call-lane imports, or exact-shape generated-C recognition.
