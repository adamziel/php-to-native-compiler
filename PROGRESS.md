# PHP Native Compiler Progress

Updated: 2026-05-23 01:10 CEST
Evaluation marker: `20260522T224455Z` plus manual post-integration refresh
Primary management baseline before this update: `83e08c94 runtime: route reference-backed array lvalue owners`
Primary semantic baseline: `83e08c94 runtime: route reference-backed array lvalue owners`
Prior evaluator marker: `20260522T210338Z`; scheduled marker
`20260522T215411Z` produced a stale report and did not land a dashboard commit.

These percentages are candid engineering estimates toward generalized PHP
semantics in the native compiler. They are not test pass rates. Lane-local work
and unstaged primary diffs do not count until reviewed, gated, committed to
`master`, and pushed.

## Executive Read

Overall estimated progress: **85%** `[#################---]`

Primary is synced at `83e08c94`, which routes reference-backed symbol-table
roots through shared generated-C array-lvalue owner operations. That converts
an existing reference/COW storage boundary into executable array write/read,
unset, presence, and increment/decrement behavior for ordinary array roots
after reference assignment activates symbol storage. The preceding
`38e98ad0` and `795c60b3` batches route generated-native `array_column(...)`
and `array_change_key_case(...)` through generalized native value array-query
boundaries. These commits improve real generated-C consumers, but they still
leave the largest call/frame, object/property, and control-flow composition
risks open.

The latest integrated baseline includes generated-C request/reference/global
symbol progress, direct and mixed symbol-root unsets, request append suffix
handling, `$GLOBALS["GLOBALS"]` self-prefixed request aliases, value mutations
through reference-backed request roots, by-value `foreach` body array-lvalue
unsets, nested request-superglobal reference paths, and dynamic
`$GLOBALS[$expr]` request-root assignment, read, `isset()`, and `empty()`
dispatch. It also includes ordinary static `$GLOBALS[...]` symbol-path
reference targets/sources through a shared value-path reference ABI, dynamic
`$GLOBALS[$expr]` root/path reference source/target dispatch for non-append
paths, keyed request references through reference-backed request roots, and
PHP-fatal direct no-key `$GLOBALS[]` rejection, request append reference slots,
generated-native array-query routing for `array_change_key_case(...)` and
`array_column(...)`, and reference-backed array-lvalue owners for active
ordinary symbol roots. The previously counted direct no-key `$GLOBALS[]` value
append slice from `aad22967` is superseded by `59f83295` and is not counted as
completed capability.

The preserved `runtime/src/lib.rs` null-slot increment/decrement hunk remains
unintegrated and is not counted.

## Current Primary State

- Primary `master` and `origin/master`: synced at
  `83e08c94a4dd6d971c63a1803e8c107c622688dd`.
- Latest semantic commit: `83e08c94 runtime: route reference-backed array lvalue owners`.
- Current product diff at refresh time: this `PROGRESS.md` update plus the
  preserved unstaged `runtime/src/lib.rs` null-slot increment/decrement hunk.
  After this progress commit lands, the runtime hunk should remain the only
  known primary diff. The runtime hunk remains unintegrated and is not counted.
- Resource note from this review: `/dev/shm` has about 8.9G available out of
  22G; `/home` has about 171G available out of 459G. Headroom is serviceable
  but still too thin for broad concurrent test waves.

## Recent Primary-Integrated Progress

- `83e08c94`: reference-backed generated-C array lvalue owners now route
  ordinary active symbol-table roots through shared owner operations for
  write/read/presence/unset/increment-update behavior after reference
  assignment activates symbol storage. Focused gates included runtime
  owner-slot tests, native-link source and executable tests for active symbol
  array-lvalue reference owners, adjacent symbol-reference and array-lvalue
  native-link suites, `cargo check -q -p php_runtime -p phpc`, rustfmt checks,
  and diff checks.
- `38e98ad0`: generated-native `array_column($rows, $column[, $index])` now
  uses `phpc_native_value_array_query_operation_with_operands_and_diagnostic(...)`,
  a reusable operand-list query ABI for native value array consumers needing
  multiple operands. Focused gates included runtime operand-list proof,
  `native_link array_column`, adjacent array-query/native-link gates,
  `cargo check -q -p php_runtime -p phpc`, rustfmt checks, and diff checks.
- `795c60b3`: generated-native `array_change_key_case($array[, $flag])` now
  routes through the shared native value array-query operation, including
  default lower-case behavior, integer upper-case flags, owned result cleanup,
  diagnostics, and linked executable proof.
- `2edd49ab`: request-superglobal append reference targets and append-reference
  sources now route through
  `phpc_native_request_state_superglobal_path_reference_append_operation(...)`
  and
  `phpc_native_request_state_superglobal_path_reference_append_source_operation(...)`.
  The generated-C consumer covers direct root appends, nested/dynamic parent
  append paths, ordinary symbol source references, and ordinary symbol targets.
  Focused gates: `cargo test -p php_runtime native_request_state_append_reference_results_bind_root_and_nested_cells -- --nocapture`,
  `cargo test -p phpc --test native_link request_append_reference -- --nocapture`,
  adjacent `request_keyed_reference` and `request_path_reference` native-link
  gates, and `cargo check -q -p php_runtime -p phpc`.
- `59f83295`: direct no-key `$GLOBALS[]` value append statements,
  assignment-expression appends, and reference append forms now reject with the
  PHP fatal `Cannot append to $GLOBALS`; the runtime root-append ABI reports
  the same diagnostic without mutating the symbol table. Focused gates:
  `cargo test -p php_runtime native_symbol_table_root_append_reports_php_fatal_without_mutating_globals -- --nocapture`,
  `cargo test -p phpc --test native_link globals_direct_root_append -- --nocapture`,
  `cargo test -p phpc --test native_link globals_symbol_path_append -- --nocapture`,
  and `cargo check -q -p php_runtime -p phpc`.
- `aad22967`: previously admitted direct no-key `$GLOBALS[] = ...` value
  appends through a symbol-table root append ABI. This was semantically wrong
  against local PHP and is superseded by `59f83295`; it is not counted as
  completed PHP capability.
- `2f407ea7`: keyed request-superglobal reference source and target operations
  now route through the shared request path-reference helper, including
  reference-backed request roots. Focused gates:
  `cargo test -p php_runtime native_request_state_keyed_reference_results_bind_shared_cells -- --nocapture`,
  `cargo test -p phpc --test native_link reference_backed_request_keyed_reference -- --nocapture`,
  `cargo test -p phpc --test native_link request_keyed_reference -- --nocapture`,
  and `cargo check -q -p php_runtime -p phpc`.
- `068ec41a`: dynamic non-append `$GLOBALS[$expr]` reference assignment now
  evaluates root/path keys once, dispatches request-root matches to
  request-state root/path reference source and target ABIs, and preserves
  ordinary symbol-table value-path reference fallback. Focused gates:
  `cargo test -p phpc --test native_link globals_dynamic_reference -- --nocapture`
  and `cargo test -p phpc --test native_link reference_assignment_paths -- --nocapture`.
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
| Runtime and ABI foundations | 97% | `[###################-]` | Strong value, array, symbol-table, request-state, comparison, truthiness, and reference ABIs. |
| Compiler/backend consumers | 96% | `[###################-]` | Good generated-C coverage for selected request, `$GLOBALS`, symbol, value, array-query, lvalue, and reference consumers; still uneven across calls, objects, control flow, and LLVM/C parity. |
| Executable generalized PHP semantics | 82% | `[################----]` | Improving through linked executable gates and array-query consumers, but still selected islands rather than a complete PHP execution model. |
| Arrays, lvalues, references, COW | 87% | `[#################---]` | Stronger arrays/lvalues, selected reference paths, generated-native array query consumers, and reference-backed array-lvalue owners for active symbol roots; full references/COW and arbitrary writable roots remain large. |
| Symbols, globals, request state | 96% | `[###################-]` | Request paths, `$GLOBALS` static/self aliases, ordinary `$GLOBALS` symbol references, dynamic root assignment/read/probe dispatch, dynamic non-append `$GLOBALS` references, symbol paths, direct/mixed root unsets, selected request references, request append reference slots, and PHP-fatal direct no-key `$GLOBALS[]` rejection are strong; broader request/global reconciliation remains open. |
| Calls, functions, frames | 25% | `[#####---------------]` | Lane candidates exist, but broad executable call/frame semantics are not primary. |
| Objects, properties, methods | 11% | `[##------------------]` | Mostly lane-local/runtime candidate work; primary still lacks general compiled object/property/method execution. |
| Diagnostics and control flow | 29% | `[######--------------]` | Useful focused diagnostics exist; exact ordering and structured cleanup are not generalized. |
| Broad integrated verification | 84% | `[#################---]` | Focused gates are strong and recent array-query/reference-owner adjacent gates help; cross-feature/backend-composition coverage is still thin. |

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
  direct unresolved root reads, ordinary static symbol-path reference
  assignment through shared symbol-table ABIs, dynamic non-append root/path
  reference dispatch through request-state matching plus ordinary symbol-table
  fallback, and PHP-fatal direct no-key `$GLOBALS[]` rejection.
- [x] Direct symbol-root `unset(...)` through the native symbol-table root unset
  ABI for single and all-direct multi-target forms.
- [x] Mixed generated-C `unset(...)` target sequencing across supported direct
  roots, array-offset roots after symbol-table activation, and `$GLOBALS` paths.
- [x] Selected generated-C reference assignment between ordinary symbol paths,
  ordinary static `$GLOBALS` symbol paths, request roots, keyed request slots,
  nested request-superglobal paths, request append reference slots against
  ordinary symbol references, and dynamic non-append `$GLOBALS[$expr]` root/path
  references.
- [x] Keyed request-superglobal reference operations through reference-backed
  request roots, reusing the path-reference helper so source and target keyed
  aliases update the shared root array.
- [x] Generated-C array-query/value-offset consumers, active-root offset
  writeback, reference-backed active symbol-root array-lvalue owners,
  generated-native `array_change_key_case(...)` and `array_column(...)`
  consumers, and by-value `foreach` body array-lvalue unsets.
- [x] Focused executable linked gates for the newest primary semantic slices.

In progress or candidate only:

- [ ] Request/global alias reconciliation, broader dynamic `$GLOBALS[$expr]`
  nested aliases outside non-append reference assignment, remaining
  request-to-request append reference combinations, by-reference foreach
  request slots, and non-request `$GLOBALS["GLOBALS"]` self-reference behavior.
  Direct no-key `$GLOBALS[]` value/reference append is not a PHP capability and
  is now covered by fatal rejection. Estimate: 67% `[#############-------]`.
- [ ] General generated PHP reference assignment over objects, arbitrary
  owner/value/reference slots, frames, append request slots, and COW-aware
  boundaries. Estimate: 63% `[#############-------]`.
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
  reference, COW, nested lvalue, and cleanup candidates that still need primary
  narrowing.
- `impl-array-value-runtime`: callable-target value-frame blockers, byte-aware
  object property lookup, and runtime array/value query expansion. Useful, but
  broad and still lane-local.
- `impl-function-frame-seed`, `impl-native-call-semantics`,
  `impl-native-object-seed`, `impl-native-object-property-runtime`, and
  diagnostics/control-flow lanes continue to produce candidate frame, object,
  property, diagnostic, and cleanup boundaries that are not primary-integrated.

These are useful signals, but they remain lane-local until narrowed and landed
on primary with focused executable proof.

## Current Steering Bias

Keep primary integration on compact structural consumers. After `83e08c94`,
another nearby array-owner helper is valuable only if it unlocks a broader
reference/COW execution boundary rather than repeating active symbol-root
array-lvalue ownership. After `38e98ad0` and `795c60b3`, another nearby array
builtin is valuable only if it unlocks a shared execution boundary used by more
than one PHP construct. After `59f83295`, direct no-key `$GLOBALS[]`
value/reference append remains a fatal path, not a capability to extend.
Ordinary static `$GLOBALS[...]` symbol-path references, dynamic non-append
`$GLOBALS[$expr]` reference dispatch, keyed request references through
reference-backed request roots, request append reference slots, direct no-key
`$GLOBALS[]` fatal rejection, and active symbol-root reference-backed
array-lvalue owners should all be treated as non-repeat guarded.

The next highest-value primary work is either request/global alias
reconciliation, a compact reference/COW owner-slot slice, or a narrow executable
slice in calls/frames, objects/properties, or structured cleanup and diagnostic
ordering. More standalone blocker vocabulary should be deprioritized.
