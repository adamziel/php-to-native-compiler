# PHP Native Compiler Progress

Updated: 2026-05-22 23:12 CEST
Evaluation marker: `20260522T210338Z`
Primary management baseline before this update: `207d31ca docs: update progress dashboard`
Primary semantic baseline: `0398c9f0 codegen: route GLOBALS symbol references`
Prior evaluator marker: `20260522T201400Z`

These percentages are candid engineering estimates toward generalized PHP
semantics in the native compiler. They are not test pass rates. Lane-local work
and unstaged primary diffs do not count until reviewed, gated, committed to
`master`, and pushed.

## Executive Read

Overall estimated progress: **83%** `[#################---]`

Primary landed and pushed `0398c9f0`, a small generated-C/runtime slice for
ordinary `$GLOBALS[...]` symbol-path reference assignment. The overall estimate
stays at 83% because the slice is narrow, but the symbols/globals/reference
surface is materially less blocked.

The latest integrated baseline includes generated-C request/reference/global
symbol progress, direct and mixed symbol-root unsets, request append suffix
handling, `$GLOBALS["GLOBALS"]` self-prefixed request aliases, value mutations
through reference-backed request roots, by-value `foreach` body array-lvalue
unsets, nested request-superglobal reference paths, and dynamic
`$GLOBALS[$expr]` request-root assignment, read, `isset()`, and `empty()`
dispatch. It now also includes ordinary static `$GLOBALS[...]` symbol-path
reference targets and sources through a shared value-path reference ABI.

The preserved `runtime/src/lib.rs` null-slot increment/decrement hunk remains
unintegrated and is not counted.

## Current Primary State

- Primary `master...origin/master`: synced at pushed semantic head `0398c9f0`
  before this management update.
- Latest semantic commit: `0398c9f0 codegen: route GLOBALS symbol references`.
- Current product diff at final verification before this management update:
  this `PROGRESS.md` update plus the preserved runtime null-slot hunk.
- The preserved null-slot increment/decrement hunk is still present inside the
  `runtime/src/lib.rs` diff and remains unintegrated.
- Resource note from this batch: `/dev/shm` was above the 6G floor but still
  close enough to prior pressure that semantic gates used disk target
  `/home/claude/php-to-native-compiler/target-primary-integrator-disk`.

## Recent Primary-Integrated Progress

- `0398c9f0`: ordinary static `$GLOBALS[...]` symbol-path reference assignment
  now uses a reusable symbol-table value-path reference ABI for root, nested,
  and append reference sources/targets while keeping request aliases and
  dynamic request roots out of this slice.
- `ad16bd67`: dynamic `$GLOBALS[$expr]` root reads, `isset()`, and `empty()`
  probes evaluate the key through the request-state PHP key boundary, dispatch
  request superglobal names to request-state snapshots/root probes, and
  preserve ordinary `$GLOBALS` symbol-table read/presence/empty fallbacks.
- `ee46e5e5`: dynamic `$GLOBALS[$expr] = ...` root assignments dispatch request
  superglobal root names to request-state root replacement while preserving the
  ordinary `$GLOBALS` symbol-table path fallback.
- `9e32c56e`: nested request-superglobal reference assignments acquire and bind
  path reference cells through request-state path reference ABIs, including
  request path targets, request path sources, request path-to-path aliases, and
  runtime reference-backed request-root path materialization.
- `4dc70807`: mixed `unset(...)` operands sequence through direct symbol-root,
  active symbol-table value-path writeback, and `$GLOBALS` symbol-path unset
  boundaries.
- `8b963fd4`: by-value `foreach` bodies can unset already-materialized array
  roots through the shared array-lvalue unset boundary while keeping body-local
  storage creation blocked.
- `b4945697`: direct symbol-root `unset(...)` uses the native symbol-table root
  unset ABI for single and all-direct multi-target forms.
- `5a6c2304`: request-superglobal append suffix paths wrap appended values into
  nested arrays before the shared request-state append mutation ABI.
- `e6037b7f`: static `$GLOBALS["GLOBALS"]` self-prefixed request aliases route
  through existing request-state ABIs.
- `6b80fd79`: keyed/path request mutations write through reference-backed
  request roots when the referenced root can be treated as array-like.

## Grand Roadmap Position

Foundations and selected generated-C request/symbol/array/reference consumers
are now strong. The remaining hard work is less about adding more helper
surfaces and more about making the compiler execute coherent PHP across
references/COW, frames, calls, objects, control flow, cleanup, diagnostics, and
backend parity.

| Roadmap item | Estimate | Visual | Primary-integrated status |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | 96% | `[###################-]` | Strong value, array, symbol-table, request-state, comparison, truthiness, and reference ABIs. |
| Compiler/backend consumers | 95% | `[###################-]` | Good generated-C coverage for selected request, `$GLOBALS`, symbol, value, array, lvalue, and reference consumers; uneven across calls, objects, control flow, and LLVM/C parity. |
| Executable generalized PHP semantics | 79% | `[################----]` | Improving through linked executable gates, but still selected islands rather than a complete PHP execution model. |
| Arrays, lvalues, references, COW | 82% | `[################----]` | Stronger arrays/lvalues and selected reference paths, including ordinary `$GLOBALS` symbol references; full references/COW and arbitrary writable roots remain large. |
| Symbols, globals, request state | 93% | `[###################-]` | Request paths, `$GLOBALS` static/self aliases, ordinary `$GLOBALS` symbol references, dynamic root assignment/read/probe dispatch, symbol paths, direct/mixed root unsets, and selected request references are strong; nested dynamic aliases and request append/reference forms remain open. |
| Calls, functions, frames | 25% | `[#####---------------]` | Lane candidates exist, but broad executable call/frame semantics are not primary. |
| Objects, properties, methods | 11% | `[##------------------]` | Mostly lane-local/runtime candidate work; primary still lacks general compiled object/property/method execution. |
| Diagnostics and control flow | 29% | `[######--------------]` | Useful focused diagnostics exist; exact ordering and structured cleanup are not generalized. |
| Broad integrated verification | 82% | `[################----]` | Focused gates are strong; cross-feature/backend-composition coverage is still thin. |

## Done / In Progress / Not Done

Done on primary:

- [x] Selected runtime/value foundations for scalars, strings, arrays,
  diagnostics, symbol tables, request state, native values, comparisons,
  truthiness, and reference-slot operations.
- [x] Generated-C request root/key/path reads, writes, unsets, appends,
  assignment-expression values, `isset()`, `empty()`, and `??` over selected
  request-superglobal forms.
- [x] Static `$GLOBALS["_GET"]`-style request aliases and static
  `$GLOBALS["GLOBALS"]` self-prefixed request aliases for selected generated-C
  request consumers.
- [x] Dynamic `$GLOBALS[$expr] = ...` root assignments dispatch request
  superglobal names through request-state root replacement while preserving the
  ordinary symbol-table fallback.
- [x] Dynamic `$GLOBALS[$expr]` root reads, `isset()`, and `empty()` dispatch
  request superglobal names through request-state key matching and preserve
  ordinary symbol-table read/presence/empty fallbacks.
- [x] `$GLOBALS[...]` symbol-table path reads, probes, writes, unsets, appends,
  direct unresolved root reads, and ordinary static symbol-path reference
  assignment through shared symbol-table ABIs.
- [x] Direct symbol-root `unset(...)` through the native symbol-table root unset
  ABI for single and all-direct multi-target forms.
- [x] Mixed generated-C `unset(...)` target sequencing across supported direct
  roots, array-offset roots after symbol-table activation, and `$GLOBALS` paths.
- [x] Selected generated-C reference assignment between ordinary symbol paths,
  ordinary static `$GLOBALS` symbol paths, request roots, keyed request slots,
  and nested request-superglobal paths.
- [x] Generated-C array-query/value-offset consumers, active-root offset
  writeback, and by-value `foreach` body array-lvalue unsets.
- [x] Focused executable linked gates for the newest primary semantic slices.

In progress or candidate only:

- [ ] Request/global alias reconciliation, dynamic `$GLOBALS[$expr]`
  request-root nested aliases, dynamic `$GLOBALS[$expr]` reference dispatch,
  direct no-key `$GLOBALS[]`, keyed reference binding through reference-backed
  request roots, request append reference/by-reference behavior, and
  non-request `$GLOBALS["GLOBALS"]` self-reference behavior. Estimate: 63%
  `[#############-------]`.
- [ ] General generated PHP reference assignment over dynamic `$GLOBALS`,
  objects, arbitrary owner/value/reference slots, frames, append request slots,
  and COW-aware boundaries. Estimate: 58% `[############--------]`.
- [ ] Narrow real call/frame execution beyond helper/blocker routing.
  Estimate: 25% `[#####---------------]`.
- [ ] Object/property/method executable semantics beyond lane-local candidates.
  Estimate: 11% `[##------------------]`.
- [ ] Structured control-flow cleanup and source-ordered diagnostics across
  branches, loops, calls, and fatal/exception-like exits. Estimate: 29%
  `[######--------------]`.

Not done:

- [ ] Full PHP references/COW, arbitrary writable roots, by-reference
  args/returns, and by-reference foreach parity.
- [ ] User function/method/closure frames, dynamic calls, variadics/spreads,
  frame-local symbol ownership, and cleanup ownership across calls.
- [ ] Real object construction, property/method dispatch, magic hooks,
  `ArrayAccess`, resources-as-objects, and object-compatible diagnostics.
- [ ] Exact PHP diagnostics, warning masks, source spans, suppression/custom
  handlers, and source-ordered cleanup through broad control flow.
- [ ] LLVM/C assembly parity for newer generated-C/runtime ABI consumers.

## Lane-Local Candidate Work Not Counted

- `impl-native-diagnostics`: dynamic known-request-root `$GLOBALS[$root]`
  operands and runtime bag-name materialization through a shared request-state
  operation path. Needs reconciliation with landed dynamic root dispatch.
- `impl-global-symbols`: direct `$GLOBALS` write-mutation fatal surfaces,
  native value `(array)` casts, and call-dispatch activation boundaries.
- `impl-native-call-semantics`: runtime value comparison ABI consumed by
  native executable C call-result operands.
- `impl-native-object-seed`: native-value string-cast carrier and object
  foreach precheck carrier for object/class metadata paths.
- `impl-symbol-integrator`: broad cleanup/rejection centralization across
  legacy call/unary/switch/foreach/reference producer paths.
- `impl-array-linked-exec` and related lanes: additional array, binary string,
  reference, and cleanup candidates that still need primary narrowing.

These are useful signals, but they remain lane-local until narrowed and landed
on primary with focused executable proof.

## Current Steering Bias

Keep primary integration on compact structural consumers. After `0398c9f0`,
ordinary static `$GLOBALS[...]` symbol-path references should be treated as done
for non-repeat purposes. The highest-value request/global work now is dynamic
nested aliases, dynamic `$GLOBALS[$expr]` reference dispatch, direct no-key
`$GLOBALS[]`, request append/reference forms, keyed references through
reference-backed request roots, and request/global alias reconciliation.

The low-percentage areas are calls, objects, and control flow. A narrow primary
slice there is valuable only if it executes real PHP behavior with linked proof;
more standalone blocker vocabulary should be deprioritized.
