# PHP Native Compiler Progress

Updated: 2026-05-22 22:49 CEST
Evaluation marker: `20260522T201400Z`
Primary management baseline before this update: `5612398d docs: update progress after request path references`
Primary semantic baseline: `ee46e5e5 codegen: dispatch dynamic GLOBALS request roots`

These percentages are candid engineering estimates toward generalized PHP
semantics in the native compiler. They are not test pass rates. Lane-local work
and unstaged primary diffs do not count until reviewed, gated, committed to
`master`, and pushed.

## Executive Read

Overall estimated progress: **82%** `[################----]`

Primary continues to move in the right direction through small, executable,
shared-boundary slices. The latest integrated baseline includes generated-C
request/reference/global-symbol progress, direct symbol-root unsets through the
native symbol table, request append suffix handling, `$GLOBALS["GLOBALS"]`
self-prefixed request aliases, value mutations through reference-backed request
roots, by-value `foreach` body array-lvalue unsets, and mixed `unset(...)`
target sequencing across direct roots, array offsets, `$GLOBALS` paths, nested
request-superglobal reference paths, and dynamic `$GLOBALS[$expr]` request-root
assignment dispatch.

The preserved `runtime/src/lib.rs` null-slot increment/decrement hunk is still
unintegrated.

## Recent Primary-Integrated Progress

- `ee46e5e5`: dynamic `$GLOBALS[$expr] = ...` root assignments now evaluate the
  key through the request-state PHP key boundary, dispatch any request
  superglobal root name to the request-state root replacement ABI, and preserve
  the ordinary `$GLOBALS` symbol-table path fallback for non-request dynamic
  roots.
- `9e32c56e`: nested request-superglobal reference assignments now acquire and
  bind path reference cells through request-state path reference ABIs, including
  request path targets, request path sources, request path-to-path aliases, and
  runtime reference-backed request-root path materialization.
- `4dc70807`: mixed `unset(...)` operands now sequence through the existing
  direct symbol-root, active symbol-table value-path writeback, and `$GLOBALS`
  symbol-path unset boundaries.
- `8b963fd4`: by-value `foreach` bodies can unset already-materialized array
  roots through the shared array-lvalue unset boundary while keeping body-local
  storage creation blocked.
- `b4945697`: direct symbol-root `unset(...)` uses the native symbol-table root
  unset ABI for single and all-direct multi-target unsets.
- `5a6c2304`: request-superglobal append suffix paths wrap appended values into
  nested arrays before the shared request-state append mutation ABI.
- `e6037b7f`: static `$GLOBALS["GLOBALS"]` self-prefixed request aliases route
  through existing request-state ABIs.
- `6b80fd79`: keyed/path request mutations write through reference-backed
  request roots when the referenced root can be treated as an array-like root.

## Roadmap Position

| Roadmap item | Estimate | Visual | Primary-integrated status |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | 96% | `[###################-]` | Strong base of value, array, symbol-table, request-state, comparison, truthiness, and reference ABIs. |
| Compiler/backend consumers | 95% | `[###################-]` | Good generated-C coverage for selected request, `$GLOBALS`, symbol, value, array, lvalue, and reference consumers; uneven across calls, objects, control flow, and LLVM/C parity. |
| Executable generalized PHP semantics | 78% | `[################----]` | Improving through linked executable gates, but still selected islands rather than a complete PHP execution model. |
| Arrays, lvalues, references, COW | 81% | `[################----]` | Stronger arrays/lvalues and selected reference paths; full references/COW and arbitrary writable roots remain large. |
| Symbols, globals, request state | 91% | `[##################--]` | Request paths, `$GLOBALS` static/self aliases, dynamic root assignment dispatch, symbol paths, direct/mixed root unsets, and selected request references are strong; dynamic reads/probes/nested aliases and several append/reference forms remain open. |
| Calls, functions, frames | 25% | `[#####---------------]` | Lane candidates exist, but broad executable call/frame semantics are not primary. |
| Objects, properties, methods | 11% | `[##------------------]` | Mostly lane-local/runtime candidate work; primary still lacks general compiled object/property/method execution. |
| Diagnostics and control flow | 29% | `[######--------------]` | Useful focused diagnostics exist; exact ordering and structured cleanup are not generalized. |
| Broad integrated verification | 81% | `[################----]` | Focused gates are strong; cross-feature/backend-composition coverage is still thin. |

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
  ordinary symbol-table fallback for non-request dynamic roots.
- [x] `$GLOBALS[...]` symbol-table path reads, probes, writes, unsets, appends,
  and direct unresolved root reads through shared symbol-table ABIs.
- [x] Direct symbol-root `unset(...)` through the native symbol-table root unset
  ABI for single and all-direct multi-target forms.
- [x] Mixed generated-C `unset(...)` target sequencing across supported direct
  roots, array-offset roots after symbol-table activation, and `$GLOBALS` paths.
- [x] Selected generated-C reference assignment between ordinary symbol paths,
  request roots, keyed request slots, and nested request-superglobal paths.
- [x] Generated-C array-query/value-offset consumers, active-root offset
  writeback, and by-value `foreach` body array-lvalue unsets.
- [x] Focused executable linked gates for the newest primary semantic slices.

In progress or candidate only:

- [ ] Request/global alias reconciliation, dynamic `$GLOBALS[$expr]`
  request-root reads/probes/nested aliases, direct no-key `$GLOBALS[]`, keyed
  reference binding through reference-backed request roots, request append
  reference/by-reference behavior, and non-request `$GLOBALS["GLOBALS"]`
  self-reference behavior. Estimate: 60% `[############--------]`.
- [ ] General generated PHP reference assignment over `$GLOBALS`, objects,
  arbitrary owner/value/reference slots, frames, append request slots, and
  COW-aware boundaries. Estimate: 57% `[###########---------]`.
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

- `impl-native-type-conversion`: object/class metadata helpers for
  `property_exists()`, `method_exists()`, `is_a()`, and `is_subclass_of()`.
- `impl-binary-string-runtime`: `preg_match_all()` outputs and several
  stream/context/filesystem surfaces through shared native ABIs.
- `impl-native-diagnostics`: request append-reference source ABI and
  request-to-request append-source reference routing.
- `impl-native-integration-batch`: shared `compact()`/`extract()` blocker
  boundary.
- `impl-array-linked-exec`: by-reference foreach cursor reference escape
  candidates.

These are useful signals, but they remain lane-local until narrowed and landed
on primary with focused executable proof.

## Current Steering Bias

Keep primary integration on compact structural consumers. The highest-value next
areas are dynamic `$GLOBALS[$expr]` request-root dispatch, direct no-key
`$GLOBALS[]`, request/global alias reconciliation, keyed request references
through reference-backed roots, request append reference/by-reference forms, and
narrow real call/frame or object/property slices only when they execute PHP
behavior rather than merely centralizing a blocker. After `ee46e5e5`, the
dynamic `$GLOBALS[$expr]` request-root item means reads/probes/nested paths or
reference/append forms, not root assignment alone.
