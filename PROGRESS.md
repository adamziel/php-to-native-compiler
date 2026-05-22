# PHP Native Compiler Progress

Updated: 2026-05-22 18:55 CEST
Evaluation marker: `20260522T165500Z`
Primary semantic HEAD: `aa94e4bd codegen: route GLOBALS request aliases through state ABI`
Current pushed semantic baseline: `aa94e4bd codegen: route GLOBALS request aliases through state ABI`

These percentages are candid engineering estimates toward generalized PHP
semantics in the native compiler. They are not test pass rates. Lane-local work
and unstaged primary diffs do not count until reviewed, gated, committed to
`master`, and pushed.

## Executive Read

Overall estimated progress: **80%** `[################----]`

This review confirms real integrated movement in the last cycle: primary now
has generated-C `$GLOBALS[...]` write/unset/append paths, request-superglobal
root/nested appends, request-superglobal null-coalescing reads, static
`$GLOBALS["_GET"]`-style request-root aliases, and direct unresolved
root-variable reads routed through shared runtime/compiler ABIs. These are
generalized symbol/path/key-driven slices with focused linked evidence, not
fixture-shaped recognizers.

The work remains bounded. Primary is stronger for selected request-state and
`$GLOBALS[...]` path operations, but it still does not have complete PHP
global/request/reference semantics. Dynamic `$GLOBALS[$expr]` request-root
alias dispatch, direct no-key `$GLOBALS[]`, request append suffix wrapping,
`$GLOBALS["GLOBALS"]` self-reference behavior, frames, references, COW, exact
diagnostics, object/property semantics, and broad control-flow cleanup remain
substantial open systems.

## Roadmap Position

| Roadmap item | Estimate | Visual | Primary-integrated status |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | 96% | `[###################-]` | Strong shared ABI base; avoid standalone vocabulary without immediate compiler consumers. |
| Compiler/backend consumers | 89% | `[##################--]` | Good for selected request/array/string/`$GLOBALS` read/write/unset/append/null-coalesce paths, static request aliases, and direct undefined root reads; uneven across calls, objects, control flow, and LLVM/C parity. |
| Executable generalized PHP semantics | 73% | `[###############-----]` | Improving through executable path consumers, but many real PHP compositions still block. |
| Arrays, lvalues, references, COW | 73% | `[###############-----]` | Arrays/lvalues advanced; full references/COW and arbitrary writable roots remain large. |
| Symbols, globals, request state | 77% | `[###############-----]` | Request paths/null-coalesce, static `$GLOBALS` request aliases, `$GLOBALS` reads/writes/probes/unsets/appends, and direct undefined root reads are stronger; dynamic aliases, direct root appends, frames, and self-reference remain incomplete. |
| Calls, functions, frames | 25% | `[#####---------------]` | Early; lane candidates exist, but broad executable call/frame semantics are not primary yet. |
| Objects, properties, methods | 11% | `[##------------------]` | Early; runtime candidates exist, but general compiled object/property/method execution remains missing. |
| Diagnostics and control flow | 29% | `[######--------------]` | Useful focused work, but exact diagnostic ordering and structured cleanup are not generalized. |
| Broad integrated verification | 81% | `[################----]` | Focused gates are useful; cross-feature and backend-composition coverage remains thin. |

## Done / In Progress / Not Done

- [x] Runtime/value foundations for selected scalar, string, array, comparison,
  diagnostic, symbol-table, request-state, reference-slot, and native-value
  operations.
- [x] Generated-C consumers for selected scalar/string/array/lvalue behavior,
  including tracked array owner mutations and natural sort families.
- [x] Request-state root, keyed, and nested/path reads, writes, unsets,
  `isset()`, `empty()`, assignment-expression values, and appends through
  shared request ABIs.
- [x] Request-superglobal `??` over root, keyed, and nested paths through
  shared request-state presence/value operations in generated C, including lazy
  fallback values and symbol-table read composition.
- [x] Static `$GLOBALS["_GET"]` / request-root aliases for generated-C root and
  nested request read, write, append, unset, `isset()`, `empty()`, and
  assignment-expression consumers through shared request-state ABIs.
- [x] Direct `$GLOBALS` root snapshots and runtime symbol-table nested
  write/read/probe ABIs.
- [x] Compiler-lowered `$GLOBALS[$expr]` and nested `$GLOBALS[...]`
  read/`isset`/`empty` paths through the symbol-table path ABI in generated C.
- [x] Compiler-lowered `$GLOBALS[$expr]` and nested `$GLOBALS[...]` writes
  through a persistent root symbol-table path ABI in generated C.
- [x] Compiler-lowered `$GLOBALS[$expr]` and nested `$GLOBALS[...]` unsets
  through the persistent root symbol-table path ABI in generated C.
- [x] Compiler-lowered `$GLOBALS[$expr]...[]...` appends through the persistent
  root symbol-table path ABI in generated C, including suffix-key wrapping and
  assignment-expression values.
- [x] Compiler-lowered direct unresolved root-variable reads through a
  symbol-table diagnostic ABI in generated C, returning PHP null values while
  reporting undefined-variable diagnostics across output, assignment/storage,
  and discarded-read consumers.
- [ ] Dynamic `$GLOBALS[$expr]` request-root alias dispatch and direct no-key
  `$GLOBALS[]` root append.
- [ ] Request append suffix wrapping after the append hole.
- [ ] Request-root append/write behavior reconciled with `$GLOBALS` aliases and
  request-state storage.
- [ ] Generated PHP reference assignment over proven array/request/symbol
  reference boundaries.
- [ ] Full references/COW, arbitrary writable roots, owner/value/reference
  slots, by-reference args/returns, and by-reference foreach parity.
- [ ] User function/method/closure frames, dynamic calls, variadics/spreads, and
  cleanup ownership across calls.
- [ ] Real object/property/method semantics, `ArrayAccess`, resource offsets,
  and PHP-compatible diagnostics around those features.
- [ ] Structured control-flow cleanup, branch joins, loop/switch transfer, and
  source-ordered warnings/errors at broad scale.

## Recent Primary-Integrated Work

Recent semantic commits on primary:

- `aa94e4bd codegen: route GLOBALS request aliases through state ABI`
- `f41f2342 codegen: route request null coalesce through state ABI`
- `6adf3530 codegen: route undefined root reads through symbol ABI`
- `46d0ba88 codegen: route request path appends through state ABI`
- `3574e350 codegen: route GLOBALS path appends through ABI`
- `af1511d3 codegen: route GLOBALS path unsets through ABI`
- `6ded95bc codegen: route GLOBALS symbol writes through ABI`
- `633c8713 codegen: route GLOBALS symbol paths through ABI`
- `39586978 runtime: add symbol-table nested read probes`
- `8c13b871 codegen: return request assignment values`
- `f88a624d codegen: route request path reads through state ABI`
- `15657b95 codegen: route request path mutations through state ABI`
- `3bda4f51 codegen: route array mutation builtins through lvalue ABI`
- `d7fc807d codegen: materialize direct $GLOBALS snapshots`
- `764cf014 runtime: add symbol-table nested write ABI`
- `ed2d9031 runtime: add array reference path ABI`

Primary-integrated capability now includes strong selected request-superglobal
path execution through shared request-state ABIs, including root/nested
appends, request null-coalescing reads, and static `$GLOBALS[request-root]`
alias routing, generated-C `$GLOBALS[...]` read/probe/write/unset/append
lowering through shared symbol-table path ABIs, and direct unresolved
root-variable reads through a diagnostic symbol-table ABI. The latest alias
slice intentionally leaves dynamic first-key request-root dispatch, exact
diagnostic ordering through arbitrary control flow, request-root
mutation/reference parity, direct no-key `$GLOBALS[]`, self-reference behavior,
frames, references/COW, and LLVM/C assembly parity blocked.

## Lane-Local And Active Candidate Work

Lane-local candidates, not counted:

- `impl-native-integration-batch`: native diagnostic queue/state runtime
  boundary with policy filtering and last-error tracking; useful foundation,
  but no primary compiler consumer yet.
- `impl-global-symbols`: function return-flow and call-dispatch contracts plus
  ongoing request/global symbol-environment work; metadata/blocker-heavy and
  must not split the primary alias model.
- `impl-array-value-runtime`: scalar non-string offset read/`isset`/`empty`
  continuation and byte-string value-frame work; includes pre-existing stale
  expectation failures that need transplant notes before primary use.
- `impl-native-diagnostics`: request-backed native builtin consumer blockers
  survive `@` error control; classification only, not executable coercion.
- Other fresh lanes continue around lvalue diagnostic spans, call cleanup,
  control-flow state reconciliation, exit/termination handoff, references,
  comparison metadata, type conversion, binary strings, and objects/properties.

## Current Steering

The next integration batches should favor small executable slices:

- Keep semantic progress tied to executable primary commits, not lane-local
  status or management-only dashboard refreshes.
- Build directly on the current global/request work: request/global alias
  reconciliation, direct no-key `$GLOBALS[]`, request append suffix wrapping,
  or request-root write/append alias behavior.
- Require one source of truth for `$GLOBALS`, request roots, symbol-table roots,
  request-state storage, self-reference, and aliases before importing broader
  dynamic request-root dispatch.
- Consider non-global/request lane candidates only when a narrow executable
  consumer is isolated with low conflict risk and clear focused gates.
- Defer broad byte-string, call-frame, object, diagnostic-state, and
  control-flow stacks until a single primary-compatible consumer can be
  extracted without importing full-lane churn.

Rejected distractions:

- Exact-shape lowering for one fixture or one PHP snippet.
- Standalone blocker/status vocabulary without a near-term executable consumer.
- Large wholesale lane merges.
- Progress percentage bumps from lane-local work alone.
- Documentation churn beyond the required evaluator marker update.

## Live Notes

Primary currently has one preserved unstaged implementation diff:
`runtime/src/lib.rs` null-slot increment/decrement behavior. It is not counted
as progress and still needs explicit classification, focused tests, and a
separate commit or rejection before any runtime staging.

Resource snapshot: `/dev/shm` has about 6.4G free of 22G and is close to the
dispatcher floor; `/home` has about 197G free. Keep focused gates and avoid
broad simultaneous cargo waves.

The supervisor dashboard tail is behind fresher worker evidence; use the
bounded snapshot and status files for this review's current facts.

Evaluator cadence: one candid strategy/progress evaluation every 45 minutes,
feeding advisory steering back to the supervisor. This marker was refreshed
even though percentages did not change, so the completed review is observable.
