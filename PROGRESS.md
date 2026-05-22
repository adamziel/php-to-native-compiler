# PHP Native Compiler Progress

Updated: 2026-05-22 19:45 CEST
Evaluation marker: `20260522T174352Z`
Primary management HEAD: `e34fc783 docs: update progress after value-result offset reads`
Primary semantic HEAD: `87e27b6b codegen: route value-result offset reads through ABI`
Current pushed semantic baseline: `87e27b6b codegen: route value-result offset reads through ABI`

These percentages are candid engineering estimates toward generalized PHP
semantics in the native compiler. They are not test pass rates. Lane-local work
and unstaged primary diffs do not count until reviewed, gated, committed to
`master`, and pushed.

## Executive Read

Overall estimated progress: **80%** `[################----]`

Primary made real integrated progress in the latest review window. Generated-C
strict identity over owned `NativeValueHandle` values now routes through the
shared comparison operand/relation ABI. Active root value-offset mutations now
write cloned mutation results back through the persistent symbol-table path ABI
when the global symbol table is active, which made the adjacent
`native_value_variable_storage` executable gate green. Generated-C offset reads
whose subject is already an owned native value-result producer now route
through the shared value-offset ABI, covering array callback results,
cast-produced arrays, nested offset-read values, and binary string value
results while preserving the direct string-offset byte path.

That progress is primary-integrated: it is committed, pushed, focused-gated,
and tied to executable generated-code behavior. It is not lane-local status
work and not fixture-shaped expected-output patching.

The work remains bounded. Primary is stronger for selected request-state,
`$GLOBALS[...]`, symbol-table, native-value comparison, active symbol writeback,
and value-result offset-read paths, but it still does not have complete PHP
global/request/reference semantics. Dynamic `$GLOBALS[$expr]` request-root
alias dispatch, direct no-key `$GLOBALS[]`, request append suffix wrapping,
`$GLOBALS["GLOBALS"]` self-reference behavior, frames, references, COW, exact
diagnostics, object/property semantics, broad control-flow cleanup, and LLVM/C
parity remain substantial open systems.

## Grand Roadmap Position

| Roadmap item | Estimate | Visual | Primary-integrated status |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | 96% | `[###################-]` | Strong shared ABI base; avoid standalone vocabulary without immediate compiler consumers. |
| Compiler/backend consumers | 92% | `[##################--]` | Good for selected request/array/string/`$GLOBALS` read/write/unset/append/null-coalesce paths, active root offset-mutation writeback, value-result offset reads, static request aliases, direct undefined root reads, and generated-C native-value strict identity; uneven across calls, objects, control flow, and LLVM/C parity. |
| Executable generalized PHP semantics | 73% | `[###############-----]` | Improving through executable path consumers, but many real PHP compositions still block. |
| Arrays, lvalues, references, COW | 73% | `[###############-----]` | Arrays/lvalues advanced; full references/COW and arbitrary writable roots remain large. |
| Symbols, globals, request state | 78% | `[################----]` | Request paths/null-coalesce, static `$GLOBALS` request aliases, `$GLOBALS` reads/writes/probes/unsets/appends, active-root offset mutation writeback, and direct undefined root reads are stronger; dynamic aliases, direct root appends, frames, references, and self-reference remain incomplete. |
| Calls, functions, frames | 25% | `[#####---------------]` | Early; lane candidates exist, but broad executable call/frame semantics are not primary yet. |
| Objects, properties, methods | 11% | `[##------------------]` | Early; runtime candidates exist, but general compiled object/property/method execution remains missing. |
| Diagnostics and control flow | 29% | `[######--------------]` | Useful focused work, but exact diagnostic ordering and structured cleanup are not generalized. |
| Broad integrated verification | 81% | `[################----]` | Focused gates are useful; cross-feature and backend-composition coverage remains thin. |

## Done / In Progress / Not Done

Done on primary:

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
- [x] Compiler-lowered `$GLOBALS[$expr]` and nested `$GLOBALS[...]` writes,
  unsets, and appends through persistent root symbol-table path ABIs in
  generated C, including suffix-key wrapping and assignment-expression values.
- [x] Compiler-lowered direct unresolved root-variable reads through a
  symbol-table diagnostic ABI in generated C, returning PHP null values while
  reporting undefined-variable diagnostics across output, assignment/storage,
  and discarded-read consumers.
- [x] Generated-C strict identity/non-identity over owned `NativeValueHandle`
  values through the shared comparison operand/relation ABI, including stored
  array-read values, `$GLOBALS[...]` path reads, and request-state reads.
- [x] Active root value-offset mutations in generated C write cloned mutation
  results back through the persistent symbol-table path ABI when the global
  symbol table is active, covering native-value storage, direct keyed writes,
  direct appends, and nested/path append writeback.
- [x] Generated-C offset reads over owned native value-result producers through
  the shared value-offset ABI, covering array callback results, cast-produced
  arrays, nested offset-read values, and binary string value results without
  replacing the direct string-offset byte path.

In progress / candidate integration themes:

- [ ] Request/global alias reconciliation, direct no-key `$GLOBALS[]`, request
  append suffix wrapping, request-root write/append alias behavior, and
  `$GLOBALS["GLOBALS"]` semantics. Estimate: 55%
  `[###########---------]`.
- [ ] Generated PHP reference assignment over proven array/request/symbol
  reference boundaries. Estimate: 35% `[#######-------------]`.
- [ ] Narrow call/frame consumers that execute real PHP-visible behavior rather
  than only centralizing blockers. Estimate: 25% `[#####---------------]`.
- [ ] Object/property/method executable semantics beyond lane-local runtime
  candidates. Estimate: 11% `[##------------------]`.
- [ ] Structured control-flow cleanup and source-ordered diagnostics across
  broad branch/loop/call paths. Estimate: 29% `[######--------------]`.

Not done:

- [ ] Dynamic `$GLOBALS[$expr]` request-root alias dispatch as part of one
  coherent alias/storage model.
- [ ] Full references/COW, arbitrary writable roots, owner/value/reference
  slots, by-reference args/returns, and by-reference foreach parity.
- [ ] User function/method/closure frames, dynamic calls, variadics/spreads, and
  cleanup ownership across calls.
- [ ] Real object/property/method semantics, `ArrayAccess`, resource offsets,
  and PHP-compatible diagnostics around those features.
- [ ] LLVM/C assembly parity for the newer generated-C/runtime ABI consumers.

## Recent Primary-Integrated Work

Recent semantic commits on primary:

- `87e27b6b codegen: route value-result offset reads through ABI`
- `a6afb405 codegen: sync active root offset mutations through symbol ABI`
- `0b28771d codegen: route native value strict identity through comparison ABI`
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

Primary-integrated capability now includes selected request-superglobal path
execution through shared request-state ABIs; static `$GLOBALS[request-root]`
alias routing; generated-C `$GLOBALS[...]` read/probe/write/unset/append
lowering through shared symbol-table path ABIs; direct unresolved root-variable
reads through a diagnostic symbol-table ABI; generated-C strict identity over
owned native-value handles through the shared comparison ABI; active root
value-offset mutation writeback into the persistent symbol table; and
value-result offset reads over existing native value-result producers.

## Lane-Local And Active Candidate Work

Lane-local candidates, not counted:

- `impl-array-value-runtime`: recent `array_map(null, ...)` identity/zip,
  value-frame, type-name, and metadata byte-registry work is plausible but
  conflict-heavy; extract only narrow executable consumers.
- `impl-native-integration-batch`: value-result offset-read composition is now
  primary-integrated; remaining string/list transform candidates need fresh
  transplant notes before primary use.
- `impl-global-symbols` and `impl-native-comparison-semantics`: symbol-derived
  value-handle truthiness, type introspection, and comparison work may be useful
  if it stays aligned with primary's request/global alias model.
- `impl-function-frame-seed` and `impl-native-call-semantics`: call/frame
  contracts are advancing, but broad executable user-function semantics are not
  primary yet.
- `impl-native-object-property-runtime` and `impl-native-object-seed`: object
  and property candidates remain lane-local and early relative to general
  compiled object execution.
- `impl-native-diagnostics`, `impl-native-error-diagnostic-semantics`,
  `impl-native-control-flow-seed`, and `impl-native-exit-seed`: useful cleanup,
  boundary, and diagnostic work, but much of it still centralizes blockers
  rather than executing broad PHP semantics.
- `impl-binary-string-runtime`, `impl-native-type-conversion`,
  `impl-array-linked-exec`, and `impl-array-lowering`: active candidate slices
  exist, but none should count until isolated, reviewed, gated, committed, and
  pushed on primary.

## Current Steering

The next integration batches should favor small executable slices:

- Keep semantic progress tied to executable primary commits, not lane-local
  status or management-only dashboard refreshes.
- Build directly on the current request/global/value ownership work: alias
  reconciliation, direct no-key `$GLOBALS[]`, request append suffix wrapping,
  request-root write/append alias behavior, `$GLOBALS["GLOBALS"]`, or one
  narrow reference/writeback consumer.
- Require one source of truth for `$GLOBALS`, request roots, symbol-table roots,
  request-state storage, frames, self-reference, references, and COW before
  importing broader dynamic request-root dispatch.
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

Primary dirty-state note: the bounded evidence and first live check showed only
the preserved `runtime/src/lib.rs` null-slot increment/decrement hunk. Final
verification during this evaluator run saw additional active uncommitted
implementation diffs in `compiler/src/codegen.rs` and `runtime/src/lib.rs`.
Those diffs were left untouched, are not counted as progress, and need separate
classification, focused tests, and a semantic commit or rejection before any
runtime/compiler staging.

Resource snapshot for this review: `/dev/shm` is about 15G free by `df` and
7.5G used by `du`; `/home` is about 193G free by `df`, with `du -sh /home`
reporting about 229G used but exiting nonzero because of permission-denied
overlay paths. Resources are acceptable for focused gates, but broad
overlapping cargo waves should remain controlled.

Evaluator cadence: one candid strategy/progress evaluation every 45 minutes,
feeding advisory steering back to the supervisor. This marker was refreshed
even though overall percentages did not change, so the completed review is
observable.
