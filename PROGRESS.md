# PHP Native Compiler Progress

Updated: 2026-05-22 21:15 CEST
Evaluation marker: `20260522T183315Z`
Primary management HEAD: this PROGRESS.md update
Primary semantic HEAD: `fa37d429 codegen: route keyed request aliases through state ABI`
Current pushed semantic baseline: `fa37d429 codegen: route keyed request aliases through state ABI`

These percentages are candid engineering estimates toward generalized PHP
semantics in the native compiler. They are not test pass rates. Lane-local work
and unstaged primary diffs do not count until reviewed, gated, committed to
`master`, and pushed.

## Executive Read

Overall estimated progress: **82%** `[################----]`

Primary made real integrated progress in the latest review window. Generated-C
array-query builtins now consume native-value operands through one shared
runtime ABI for filtered `array_keys()`, `in_array()`, `array_search()`,
`array_flip()`, `array_count_values()`, `array_sum()`, `array_product()`,
`array_fill_keys()`, and `array_combine()`, with executable proof that
array-valued query results compose through offset reads. Generated-C reference
assignments over ordinary symbol roots and symbol-rooted array paths now use a
shared symbol-table path-reference ABI, covering direct aliases, dynamic nested
keys, source appends, target appends, and path-write composition. Generated-C
request-superglobal roots can now bind to ordinary symbol references through
the request-state root reference replacement ABI, covering multiple request
bags plus direct, nested-key, append-created, and array-valued symbol sources.
LLVM and generated-C truthiness for owned native value operands now share
`phpc_native_value_is_truthy(...)`, covering unary `!` and non-short-circuit
`xor` across native value producers with explicit cleanup of consumed handles.
Generated-C reference assignment can now also acquire request-superglobal roots
as reference sources, promote table-backed request bags to shared root cells,
and bind those cells into direct, nested-key, and append-created ordinary
symbol targets. Generated-C keyed request-superglobal slots now participate in
the same reference-assignment family in both directions with ordinary symbol
paths, using request key materialization, request-state reference results, and
symbol-table path binding. Generated-C request-superglobal root-to-root
reference assignment now acquires the source request root's shared cell and
replaces the target request root with that same cell, covering table-backed and
scalar roots across multiple request bags while preserving the same diagnostic
and cleanup ABI. Generated-C keyed request-superglobal slots can now alias
other keyed request-superglobal slots through the same keyed reference-result
and keyed bind ABI, with source/target key materialization and cleanup shared
across multiple request bags and scalar/dynamic key shapes.

That progress is primary-integrated: it is committed, pushed, focused-gated,
and tied to executable generated-code behavior. It is not lane-local status
work and not fixture-shaped expected-output patching.

The work remains bounded. Primary is stronger for selected request-state,
`$GLOBALS[...]`, symbol-table, native-value comparison, active symbol writeback,
value-result offset-read paths, selected ordinary symbol reference assignment
paths, request-root reference replacement from ordinary symbol paths,
request-root reference source binding into ordinary symbol paths,
request-root/request-keyed aliasing, and
native-value truthiness for selected boolean consumers, but it still does not
have complete PHP global/request/reference/control-flow semantics. Dynamic
`$GLOBALS[$expr]` request-root alias dispatch, direct no-key `$GLOBALS[]`,
request append suffix wrapping, `$GLOBALS["GLOBALS"]` self-reference behavior,
frames, full references/COW, exact diagnostics, object/property semantics,
ordered short-circuit cleanup, and broader LLVM/C parity remain substantial
open systems.

## Grand Roadmap Position

| Roadmap item | Estimate | Visual | Primary-integrated status |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | 96% | `[###################-]` | Strong shared ABI base; avoid standalone vocabulary without immediate compiler consumers. |
| Compiler/backend consumers | 95% | `[###################-]` | Good for selected request/array/string/`$GLOBALS` read/write/unset/append/null-coalesce paths, active root offset-mutation writeback, value-result offset reads, array-query value consumers, static request aliases, direct undefined root reads, generated-C symbol references, request-root and keyed request-slot reference assignment with ordinary symbol paths plus request-to-request aliases, generated-C native-value strict identity, and LLVM/generated-C native-value truthiness for unary `!` / `xor`; uneven across calls, objects, short-circuit control flow, and broader LLVM/C parity. |
| Executable generalized PHP semantics | 76% | `[###############-----]` | Improving through executable path/reference/logical consumers, but many real PHP compositions still block. |
| Arrays, lvalues, references, COW | 79% | `[################----]` | Arrays/lvalues advanced with query/value-result consumers, selected symbol-path references, request-root reference replacement/source/root-alias paths, keyed request-slot references with ordinary symbol paths, and keyed request-to-keyed request aliases; full references/COW and arbitrary writable roots remain large. |
| Symbols, globals, request state | 84% | `[#################---]` | Request paths/null-coalesce, static `$GLOBALS` request aliases, `$GLOBALS` reads/writes/probes/unsets/appends, active-root offset mutation writeback, direct undefined root reads, ordinary symbol-path reference assignment, request-root reference replacement/source/root-alias paths, and keyed request-slot aliases are stronger; dynamic aliases, direct root appends, frames, nested/path request references, full references, and self-reference remain incomplete. |
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
- [x] Generated-C array-query builtins over native value operands through one
  shared runtime ABI, covering filtered `array_keys()`, `in_array()`,
  `array_search()`, `array_flip()`, `array_count_values()`, `array_sum()`,
  `array_product()`, `array_fill_keys()`, and `array_combine()` with
  value-offset composition for array-valued results.
- [x] Generated-C reference assignment over ordinary symbol roots and
  symbol-rooted array paths through a shared path-reference ABI, including
  direct aliases, dynamic nested keys, source append references, target append
  binding, path-write composition, and cleanup of materialized keys/reference
  handles.
- [x] Generated-C request-superglobal root reference assignment from ordinary
  symbol roots and symbol-rooted paths through the existing request-state root
  reference replacement ABI, including multiple request bags, direct sources,
  dynamic nested-key sources, append-created sources, array-valued source
  roots, and later symbol write composition through shared reference cells.
- [x] Generated-C reference assignment from request-superglobal root sources to
  ordinary symbol roots and symbol-rooted paths through a request-state root
  reference acquisition ABI, including table-backed request root promotion,
  direct targets, dynamic nested-key targets, append-created targets, and later
  symbol writes observed through request root snapshots/path reads.
- [x] Generated-C reference assignment between keyed request-superglobal slots
  and ordinary symbol roots/symbol-rooted paths through a request-state
  reference-result ABI, including direct keyed request targets, keyed request
  sources, dynamic/scalar key materialization, symbol nested-path sources, and
  append-created ordinary symbol targets.
- [x] Generated-C request-superglobal root-to-root reference assignment
  through the request-state root reference acquisition and replacement ABIs,
  including table-backed source promotion, scalar source roots, multiple
  request bags, and later ordinary alias writes observed through both roots.
- [x] Generated-C keyed request-superglobal slot-to-slot reference assignment
  through the request-state keyed reference-result and keyed bind ABIs,
  including dynamic/scalar source and target key materialization, multiple
  request bags, and later ordinary alias writes observed through both slots.
- [x] LLVM and generated-C truthiness for owned native value operands through
  `phpc_native_value_is_truthy(...)`, covering unary `!` and
  non-short-circuit `xor` across native value producers while leaving ordered
  short-circuit operators to structured control-flow cleanup.

In progress / candidate integration themes:

- [ ] Request/global alias reconciliation, direct no-key `$GLOBALS[]`, request
  append suffix wrapping, request-root write/append alias behavior, and
  `$GLOBALS["GLOBALS"]` semantics. Estimate: 55%
  `[###########---------]`.
- [ ] Generated PHP reference assignment over `$GLOBALS`, object, arbitrary
  owner/value/reference-slot, frame, nested/path request slots, and COW-aware
  boundaries. Estimate: 55% `[###########---------]`.
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

- `fa37d429 codegen: route keyed request aliases through state ABI`
- `029087f0 codegen: route request root aliases through state ABI`
- `beff266e codegen: route keyed request references through state ABI`
- `ad02c761 codegen: route request root reference sources through state ABI`
- `ee990dad codegen: route native value truthiness through ABI`
- `e9dc9ca9 codegen: route request root references through state ABI`
- `49b8ad8a codegen: route symbol references through path ABI`
- `72c3b2d5 codegen: route array query builtins through value ABI`
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
value-result offset reads over existing native value-result producers. It now
also includes generated-C array-query builtins routed through a shared
native-value query ABI with executable offset-read composition for their
array-valued results, plus ordinary symbol-root reference assignments routed
through a shared path-reference ABI with executable direct, nested, source
append, and target append coverage. Request-superglobal roots now consume that
symbol reference boundary as sources for request-state root reference
replacement, with executable proof across request bags and source path shapes.
LLVM and native C now share a runtime truthiness ABI for owned native value
operands in unary `!` and non-short-circuit `xor`, including cleanup of
consumed native handles after boolean conversion. Generated-C reference
assignment now also acquires request roots as reference sources through the
request-state root reference ABI and binds them into direct, nested, and append
ordinary symbol targets with executable request snapshot/path-read proof.
Generated-C request-root-to-request-root reference assignment now also shares
the same request-state root cell between source and target roots, with
executable proof that writes through ordinary aliases are visible through both
request roots.
Generated-C keyed request slots now also acquire and bind references through
request-state reference results, composing with ordinary symbol direct, nested,
append-created paths, and keyed request-to-keyed request aliases while leaving
nested/path request references, `$GLOBALS`, and full COW/reference ownership
open.

## Lane-Local And Active Candidate Work

Lane-local candidates, not counted:

- `impl-global-symbols`: ordered short-circuit and broader symbol-derived
  value-handle truthiness/control-flow contracts remain active candidate
  material. Unary `!` and non-short-circuit `xor` over native value handles are
  now primary-integrated.
- `impl-array-value-runtime`: recent `array_map(null, ...)` identity/zip,
  value-frame, type-name, and metadata byte-registry work is plausible but
  conflict-heavy; extract only narrow executable consumers.
- `impl-native-integration-batch`: value-result offset-read composition and
  array-query value operations are now primary-integrated; remaining
  string/list transform candidates need fresh transplant notes before primary
  use.
- `impl-native-type-conversion`: stream-resource operation work is large and
  lane-local; extract one narrow generalized consumer if useful.
- `impl-native-comparison-semantics`: object/comparison operand candidates may
  be useful after focused extraction, but object execution remains early.
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
- `impl-binary-string-runtime`, `impl-array-linked-exec`, and
  `impl-array-lowering`: active candidate slices exist, but none should count
  until isolated, reviewed, gated, committed, and pushed on primary.

## Current Steering

The next integration batches should favor small executable slices:

- Keep semantic progress tied to executable primary commits, not lane-local
  status or management-only dashboard refreshes.
- Build directly on the current request/global/reference work: alias
  reconciliation, direct no-key `$GLOBALS[]`, request append suffix wrapping,
  request-root write/append alias behavior, `$GLOBALS["GLOBALS"]`,
  keyed request-slot references, or one narrow reference/writeback consumer.
- Treat native-value truthiness as landed only for unary `!` and
  non-short-circuit `xor`; do not overclaim ordered `&&` / `||`, branch
  cleanup, diagnostic timing, references/COW, or broad control flow.
- Require one source of truth for `$GLOBALS`, request roots, symbol-table roots,
  request-state storage, frames, self-reference, references, and COW before
  importing broader dynamic request-root dispatch.
- Consider non-global/request lane candidates only when a narrow executable
  consumer is isolated with low conflict risk and clear focused gates.
- Defer broad byte-string, call-frame, object, diagnostic-state, stream, and
  control-flow stacks until a single primary-compatible consumer can be
  extracted without importing full-lane churn.
- Do not repeat the array-query ABI or generated-C query builtin family; next
  array work should move toward arbitrary roots, references/COW,
  nested/writeback/RMW gaps, or LLVM/C parity.

Rejected distractions:

- Exact-shape lowering for one fixture or one PHP snippet.
- Standalone blocker/status vocabulary without a near-term executable consumer.
- Large wholesale lane merges.
- Progress percentage bumps from lane-local work alone.
- Documentation churn beyond the required evaluator marker update.

## Live Notes

Primary dirty-state note: primary is synced with `origin/master` at
`b6e61a77`; the only remaining dirty file is the preserved unstaged
`runtime/src/lib.rs` null-slot increment/decrement hunk. Keep staging surgical.

Resource snapshot for this review: `/dev/shm` is usable but still worth
watching at 22G total, 14G used, 8.0G free, 64% used by `df`. `/home` has
190G free by `df`; `du -sh /home` timed out during the evaluator run, while
`du -sh /home/claude/php-to-native-compiler` reported 11G. Use disk targets or
single-threaded focused gates when tmpfs drops below the dispatcher floor.

Evaluator cadence: one candid strategy/progress evaluation every 45 minutes,
feeding advisory steering back to the supervisor. This marker was refreshed
even though overall percentages did not change, so the completed review is
observable.
