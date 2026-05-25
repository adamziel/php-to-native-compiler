# PHP Native Compiler Progress

Updated: 2026-05-25 21:47 CEST
Evaluation marker: `20260525T194746Z`

Accounting rule: only generalized, tested, committed, and pushed primary work
counts as integrated capability. Dirty WIP, candidate worktrees, lane-local
claims, review-only work, failed prep proofs, probe-only commits, and
dashboard-only commits are excluded.

## Executive Read

Overall estimated progress: **92%** `[##################--]`

Executable PHP semantics: **92%** `[##################--]`

Primary is clean and aligned with `origin/master` at
`2a535631 docs: update progress dashboard` before this `PROGRESS.md` edit.

The latest primary-integrated production/prerequisite baseline remains
`2cd78ade native: share conversion result consumers`. The latest integrated
executable capability baseline remains
`b13c85c6 native: add unary negation source result ABI`.

This review keeps percentages flat. No new primary compiler/runtime source
commit landed after `2cd78ade`; `2a535631` is docs-only. The active
request-key selector fallback is a codegen-only cleanup candidate, not new
executable PHP behavior. The post-helper call split proved useful current
closure coverage, but by-reference closure returns still need a real native
closure value/reference return ABI before they can count.

## Grand Roadmap Position

| Workstream | Estimate | Bar | Current read |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | **99%** | `[####################]` | Strong selected-path value, byte-string, string-array, array, diagnostic, reference, symbol, call-frame, object, comparison, conversion, numeric-unary, owner-cell, request-state, offset-read, array-key, type/int, text-membership, comparison, truthiness, and non-local owner-boundary surfaces. |
| Compiler/backend consumers | **99%** | `[####################]` | LLVM and generated C share many ABIs. `2cd78ade` consolidated current LLVM conversion-result consumers for scalar offset-read and numeric-unary source/result paths. Direct assembly and some generated-C-only surfaces still lag. |
| Executable PHP semantics | **92%** | `[##################--]` | Primary has many selected semantic islands, including closure/callable/object paths, bounded preg callbacks, object-property reference-slot mutation, byte strings, string-array slots, array-key/reference-slot consumers, non-local blockers, and unary negation through the shared conversion ABI. |
| Strings and byte semantics | **62%** | `[############--------]` | Byte-backed values and byte-preserving `explode()` / `str_split()` slots are integrated. Binary source bytes, byte-exact interpreter output/session/debug formatting, `mb_str_split()` codepoint behavior, request/global byte keys, and exact diagnostics remain open. |
| Arrays, lvalues, references, COW | **78%** | `[################----]` | Value/reference slot ABIs continue to expand, but full COW, arbitrary lvalue roots, foreach, broader expression reference slots, alias composition, and writeback remain incomplete. |
| Symbols, globals, request state | **73%** | `[###############-----]` | Selected function globals, root-symbol surfaces, active symbol-table reference consumers, request-backed key blockers, and request-key diagnostics exist. `$GLOBALS` self-cells, request/global alias parity, request writeback, includes, variable variables, and exact unset/global behavior remain incomplete. |
| Calls, functions, frames | **83%** | `[#################---]` | Selected direct/callable/function-table, descriptor-closure, capture, and method-frame surfaces exist. The fresh split shows by-reference closure returns need a shared value/reference return ABI before broader call/reference semantics can move. |
| Objects, properties, methods | **51%** | `[##########----------]` | Object-property reference-slot mutation, diagnostic classifiers, and non-local object/static owner blockers are integrated. Full visibility, magic, dynamic/static/typed properties, destructors, references/COW, and `ArrayAccess` execution remain open. |
| Control flow, cleanup, diagnostics | **51%** | `[##########----------]` | Selected branches, loops, transfers, finalizers, output buffers, diagnostics, and truthiness/conversion consumers exist. Broad unwind/finally/destructor/shutdown and exact source ordering remain open. |
| Broad integrated verification | **91%** | `[##################--]` | Focused primary gates are strong for recent packets. Broad gates remain constrained by lane extraction cost, high swap, stale lane expectations, and backend parity gaps. |

## Active Roadmap Items

| Item | Toward Primary Integration | Toward Full Feature | Status |
| --- | ---: | ---: | --- |
| Conversion-result helper and shortcut retirement | **100%** `[####################]` | **44%** `[#########-----------]` | Integrated at `2cd78ade` for current LLVM scalar offset-read and numeric-unary source-result consumers. Broader conversion families, object/resource coercions, exact diagnostics, error control, and cleanup/unwind remain open. |
| Runtime numeric-unary conversion ABI | **100%** `[####################]` | **48%** `[##########----------]` | Integrated at `b13c85c6`. Covered unary `-` routes through `NativeConversionSource` / `NativeConversionResult` across runtime, LLVM, generated C, and focused linked execution. |
| Non-local assignment owner-cell blocker refresh | **100%** `[####################]` | **35%** `[#######-------------]` | Integrated at `f770d728`. This is blocker/parity work, not executable property/static writeback. |
| String operation-family slot consumers | **100%** `[####################]` | **45%** `[#########-----------]` | Integrated at `5307990c`; `explode()` and `str_split()` use shared byte-preserving value/reference-slot contracts. |
| Byte-preserving PHP string value boundary | **100%** `[####################]` | **40%** `[########------------]` | Integrated at `1c369d0f`; byte-backed PHP values and native pointer-plus-length materialization are available for selected paths. |
| Request-key backend diagnostic selector fallback | **70%** `[##############------]` | **34%** `[#######-------------]` | Candidate `request-key-selector-fallback-prep` is ready for primary review at hash `e71bf25f9426cc058ef6d741fd5801357b1f5ac9a71474df83b35e0b74c8e0fb`, dirty only in `compiler/src/codegen.rs`. Count as cleanup only; two zero-filter gate caveats require review. |
| Call/closure reference return ABI | **15%** `[###-----------------]` | **32%** `[######--------------]` | Current split returned `needs-split`. It proved descriptor-ready closure invocation and capture surfaces, but by-reference closure returns require a reusable closure value/reference return ABI and reference-result consumers. |
| Broader source-call cleanup lanes | **20%** `[####----------------]` | **38%** `[########------------]` | `impl-native-call-semantics` has useful lane-local call cleanup and callable-context evidence, but broad dirty state keeps it evidence only until exact current-primary extraction. |
| Broader lvalue/reference-slot materializer | **30%** `[######--------------]` | **39%** `[########------------]` | Needed so non-variable expression families that can carry references can enter shared array-key and consumer slot ABIs safely. Current broad lane evidence is parked or lane-local. |
| Object/resource source materialization | **25%** `[#####---------------]` | **30%** `[######--------------]` | Still a recurring blocker for generic conversion and offset/source consumers. Needs a general value reconstruction/materialization boundary. |
| Broad lane extraction backlog | **34%** `[#######-------------]` | **35%** `[#######-------------]` | Broad dirty lanes remain useful evidence repositories, not integration units. Several are parked after cadence failures. |

## Done / In Progress / Not Done

Primary-integrated executable or executable-prerequisite capability:

- [x] Descriptor-backed closures, selected captures, selected by-reference parameters, and selected callable-array/object invocation.
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

In progress but lane-local or not yet executable primary support:

- [ ] `request-key-selector-fallback-prep` is ready for review as codegen-only cleanup; do not count as executable semantics unless a separate execution boundary is proved.
- [ ] `post-helper-call-semantics-split-prep` proved current primary closure/capture coverage but returned `needs-split` for by-reference closure returns.
- [ ] A native closure value/reference return ABI is needed before by-reference closure return execution can count.
- [ ] `impl-native-call-semantics` has lane-local caller-scope, dynamic callable, source-call cleanup, and reference/call evidence; not primary-integrated.
- [ ] `impl-native-error-diagnostic-semantics` has lane-local selector/report-sink cleanup evidence; not primary-integrated.
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

- `2cd78ade`: conversion-result consumer helper consolidation. Integrated
  `compiler/src/codegen.rs` and `compiler/tests/native_runtime_abi.rs`.
  Focused gates, `cargo check`, formatting, diff checks, push proof, and clean
  sync were reported by the primary integrator.
- `b13c85c6`: unary negation source/result ABI across runtime, LLVM, generated
  C, and focused linked execution.
- `f770d728`: non-local assignment owner blockers across current backend
  lowering families.
- `5307990c`: string-array operation slots for byte-preserving `explode()` and
  `str_split()`.
- `1c369d0f`: byte-backed PHP string value boundary.

## Current Work Snapshot

Primary-integrated:

- [x] Primary was clean and synced at `2a535631` before this `PROGRESS.md`
  edit.
- [x] Latest production/prerequisite head remains `2cd78ade`.
- [x] Latest executable capability head remains `b13c85c6`.
- [x] Percentages remain flat for this review.

Lane-local:

- [ ] `request-key-selector-fallback-prep` is the active review candidate,
  based on `2a535631`, dirty only in `compiler/src/codegen.rs`, with stable hash
  `e71bf25f9426cc058ef6d741fd5801357b1f5ac9a71474df83b35e0b74c8e0fb`.
- [ ] The request-key selector candidate should be counted as cleanup only and
  its two zero-filter native-runtime ABI caveats should be reviewed explicitly.
- [ ] `post-helper-call-semantics-split-prep` produced stable current-primary
  closure/capture evidence, but its own decision is `needs-split`; do not route
  it as a ready review packet.
- [ ] The next call/frame semantic target is a reusable closure
  value/reference return ABI with linked alias/write-through proof.
- [ ] Broad dirty lanes should remain evidence repositories only.

Resource posture:

- `/dev/shm`: 40G total, 24G used, 17G available, 58% used.
  Largest observed targets: `phpc-target-native-call-semantics` 8.9G,
  `phpc-target-native-object-seed` 5.6G,
  `phpc-target-native-diagnostics` 3.0G.
- `/home`: 459G total, 206G used, 235G available, 47% used.
  Largest observed lane/work tree:
  `phpc-lane-native-error-diagnostic-semantics` 14G; primary is about 2.2G.
- Memory available is about 40Gi, but swap remains high at 23Gi/29Gi used.
- Continue disk-backed `/tmp` target dirs, `umask 0007`,
  `CARGO_BUILD_JOBS=1`, `CARGO_INCREMENTAL=0`, and focused nonzero gates.

## Next Steering Read

Best compact cleanup packet to consider:

- Finish independent review of the request-key backend selector fallback. If it
  is accepted, integrate only as cleanup and keep executable-progress
  percentages flat unless a separate semantic execution boundary is proved.

Best next executable semantic packet to design:

- A closure value/reference return ABI that can prove linked alias/write-through
  across direct calls, dynamic descriptor closure calls, nested call consumers,
  discarded calls, and reference assignment sources.

Avoid:

- Re-counting `2cd78ade` conversion-result helper work as pending.
- Routing the post-helper call split candidate to integration despite its
  `needs-split` decision.
- Whole-lane imports from parked or broad dirty worktrees.
- Percentage bumps for selector-only cleanup, helper-only cleanup, blocker-only
  work, or lane-local claims that are not committed and pushed on primary.
