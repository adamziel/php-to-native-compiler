# PHP Native Compiler Progress

Updated: 2026-05-21 17:03 CEST
Evaluation marker: 20260521T143958Z
Final refresh: 20260521T150316Z

This is a high-level roadmap for a supervisor who needs the current momentum quickly. Percentages are candid engineering estimates, not test-suite completion metrics. Primary-integrated capability means committed on `master`; lane-local work is candidate material only until selected, gated, committed, and pushed.

## Overall Status

Estimated progress toward a broadly usable generalized PHP native compiler: **20%**

```
Generalized runtime/ABI foundations      [############--------] 60%
Compiler/backend consumers               [##########----------] 48%
Executable generalized PHP semantics     [####----------------] 19%
Arrays, references, COW, lvalues         [##------------------] 12%
Objects, properties, methods             [##------------------] 10%
Diagnostics/control-flow composition     [###-----------------] 17%
Broad integrated verification            [###-----------------] 13%
```

## Current Primary State

- Latest pushed primary commit observed before this update: `633da5fc runtime: share arithmetic operand conversion`.
- Latest integrated semantic compiler/runtime commit: `633da5fc runtime: share arithmetic operand conversion`.
- Product-code state at progress refresh: clean and synced with `origin/master`; only this `PROGRESS.md` update is dirty for the wrapper to commit.
- Latest integrated read: runtime arithmetic-number operand conversion now feeds `+`, `-`, `*`, and `/`, replacing duplicated production conversion sequencing while preserving left-to-right blocker ordering. This is shared runtime conversion infrastructure, not broad native arithmetic lowering.

## Grand Roadmap Position

The project is moving from scattered backend rejection paths toward shared semantic-family boundaries and backend consumers. The bottleneck remains executable generalized PHP behavior: calls, conversions, arrays, references/COW, symbols, objects, diagnostics, cleanup, and composition tests need to stop only reaching good blockers and start executing correct PHP semantics family by family.

## Primary-Integrated Roadmap

- [x] Establish supervised parallel implementation lanes and primary integration gate.
- [x] Add shared runtime ABI surfaces for symbols, string boundaries, string truthiness, comparison results, diagnostic severity, numeric-string classification, and selected conversion helpers.
- [x] Route value, lvalue, argument, reference-source, reference-assignment, statement, and unset call-result contexts through shared call-operation boundaries.
- [x] Route unary, binary, comparison, and concat value-operand call blockers through the shared call-operation boundary across LLVM IR and generated-C backends.
- [x] Route skipped later `echo` operands through the shared call-operation boundary across LLVM IR and generated-C backends.
- [x] Consume comparison branch-result ABI from generated C instead of direct struct-field reads.
- [x] Materialize generated-C comparison string operands through a runtime byte-boundary with diagnostic results.
- [x] Centralize generated-C comparison operand materialization failures behind a runtime-owned exit-code/report/free ABI.
- [x] Share runtime comparison operation/value-family dispatch across existing runtime and native comparison consumers.
- [x] Share runtime arithmetic-number operand conversion across `+`, `-`, `*`, and `/` consumers.
- [x] Reuse runtime PHP string truthiness semantics in runtime values plus LLVM/generated-C known-string consumers.
- [x] Track generated-C dynamic string byte lengths so mixed-length and embedded-NUL comparison operands can consume the runtime comparison ABI.
- [ ] Replace selected shared blockers with real generalized execution for one semantic family at a time.
- [ ] Generalize value/result ownership across returns, call args, conditions, branch joins, discarded temporaries, stdout, and cleanup.
- [ ] Implement full array lvalue/RMW semantics, writable roots, foreach/by-ref foreach, ArrayAccess/object/resource offsets, and COW/reference behavior.
- [ ] Implement full symbol environment semantics for roots, locals, imports, globals, superglobals, undefined slots, repeated calls, and request/global separation.
- [ ] Implement function/method call frames, argument binding/cleanup, by-ref args/returns, variadics/spreads, dynamic calls, callbacks, constructors, and frame handoff.
- [ ] Implement object/class/property semantics including dynamic names, visibility/magic hooks, stdClass behavior, property offsets, diagnostics, and references/COW.
- [ ] Implement generalized diagnostics, conversion/comparison semantics, control-flow cleanup, loops/switch/break/continue/goto/finally blockers, and broad composition tests.

## Active Roadmap Estimates

| Active item | Primary-integrated estimate | Lane-local candidate maturity | Current read |
| --- | ---: | ---: | --- |
| String conversion, truthiness, and byte-buffer results | 34% | 60% | Primary has string-conversion result/free ABI, comparison byte materialization, runtime-shared string truthiness, generated-C tracked dynamic string lengths, and runtime numeric-string classification. Lanes have broader binary-safe string-result, debug-output, regex, concat/interpolation, and array-transform consumers. Exact diagnostics and full backend parity remain limited. |
| Call operation cleanup and ownership | 37% | 55% | Primary routes many call-result contexts through shared blockers across LLVM IR and generated C, including value operands in unary/binary/comparison/concat families and skipped later `echo` operands. Actual frames, binding, by-ref args/returns, variadics, callbacks, dynamic dispatch, and return ownership remain mostly non-executable. |
| Comparison/conversion semantics | 38% | 56% | Primary has comparison ABI consumers, runtime comparison operation/value-family sharing for loose equality/order and strict identity, shared arithmetic-number operand conversion for `+`, `-`, `*`, and `/`, canonical branch-result predicates/exit-code consumers, centralized comparison operand materialization failure handling, runtime numeric-string reuse, generated-C string-byte comparison operands with tracked dynamic lengths, and shared string truthiness. Lane conversion-source/pair work is promising but still candidate material. |
| Arrays, lvalues, references, COW | 12% | 47% | Lane-local generated-C owner-slot/value-root/RMW/reference-cell work is strong, especially undefined/null/false value-root routing, but primary still lacks full executable array lvalue, foreach, reference/COW, and ArrayAccess behavior. |
| Symbols, globals, request state | 21% | 41% | Primary has symbol-table ABI helpers; lanes have expression-result consumers, frame-slot plans, request-state contracts, and stored call-result symbol boundaries. Full request/global/superglobal behavior and exact diagnostics remain early. |
| Objects, properties, methods | 10% | 35% | Lane-local object/property receiver and operation blockers are more coherent, but primary has little broad executable object/property/method behavior beyond bounded blocker and seed paths. |
| Diagnostics and control-flow cleanup | 17% | 40% | Severity tags and selected blockers are integrated; request diagnostics, diagnostic-result producers/sinks, structured CFG, and termination cleanup lanes are active. Exact warning/recovery order and executable cleanup remain broad blockers. |
| Broad composition verification | 13% | 29% | Focused gates are good and recent primary batches are well-tested locally. Broad differential PHP composition coverage remains limited. |

## Recent Integrated Primary Work

Recent pushed primary commits show useful movement from lane-local artifacts into `master`:

- `633da5fc runtime: share arithmetic operand conversion`
  - Shares runtime arithmetic-number operand conversion across addition, subtraction, multiplication, and division, replacing duplicated production conversion sequencing while preserving shared left-to-right blocker behavior across scalar and unsupported value families.
- `ca029eb4 runtime: share comparison operation families`
  - Shares runtime comparison dispatch through operation families and scalar/array/object/resource value families, replacing duplicated production comparison blocker and strict-identity dispatch for existing runtime and native-value comparison consumers.
- `c0ee80ae codegen: route echo operand calls through shared boundary`
  - Routes later `echo` statement-list operands skipped after an earlier blocked operand through the shared native call-operation boundary in LLVM IR and generated-C lowering.
- `dce9e75a codegen: route value operand calls through shared boundary`
  - Routes unary, generic binary, comparison-left-failure, and static string concat value operands through the shared native call-operation boundary in LLVM IR and generated-C lowering.
- `a79d75de native: centralize comparison materialization failures`
  - Adds a runtime-owned comparison operand materialization failure boundary and routes generated-C comparison string/byte operands through it before the owned branch comparison ABI.
- `7c4017f1 codegen: consume comparison branch predicates`
  - Adds canonical runtime accessors for comparison branch truth and process exit status, then routes generated-C comparison control flow through those predicates instead of backend-local field checks.
- `495526a2 codegen: track native C string lengths`
  - Tracks generated-C string-expression byte lengths through selected expression lowering so embedded-NUL dynamic string operands can use byte-aware runtime materialization.
- `227c3a63 runtime: share PHP string truthiness semantics`
  - Adds a shared runtime `is_php_truthy_string()` helper and routes runtime value truthiness plus LLVM/generated-C known string consumers through it.
- `d11d3c8d`, `5c6b9393`, `b8052d11`, and earlier call-boundary batches
  - Continue routing statement, unset, reference-source, reference-assignment, lvalue, argument, and value call-result contexts into shared call-operation cleanup blockers.

## Lane-Local Candidate Work

Lane-local workers are producing candidate work in these areas:

- additional statement-list and declaration-initializer call-boundary routing;
- generated-C array value-root, undefined-root, unset/read/probe/foreach, and owner-slot routing;
- array RMW operation contracts and assignment-expression lvalue contracts;
- binary-safe string result/debug/regex/concat/interpolation/array-transform consumers;
- expression-result ownership and consumer/effect cleanup contracts;
- runtime call/frame result, stored caller result, and gettype ownership contracts;
- conversion-source and pair-conversion runtime ABI work;
- object/property receiver, stateful expression-operation, class-policy, and property-operation blockers;
- reference/owner-cell descriptor, transfer, borrow, apply, and cleanup scaffolding;
- request/superglobal operation snapshots and diagnostic consumers;
- structured CFG/control-flow effect rows and termination cleanup stack scans.

Lane-local work is useful source material, but it is not product capability until selected into a small primary batch, gated, committed, and pushed.

## Candid Assessment

The direction is right, and primary integration is landing real generalized consumers. The work is still boundary-heavy: many improvements make unsupported PHP fail through better semantic surfaces, while broad successful PHP execution remains narrow.

The next highest-value work is to convert landed ABI/result surfaces into actual backend behavior with cleanup, diagnostics, and composition tests. More result vocabulary is lower priority unless it immediately feeds executable generated-C/LLVM consumers or replaces a blocker with real generalized behavior.

## Near-Term Steering

1. Keep integrating one small generalized primary batch at a time.
2. Prefer executable compiler/generated-C/LLVM consumers of already-landed ABI surfaces over more standalone vocabulary.
3. Consider the next call-boundary slice only if it consumes an already-landed shared operation across multiple source surfaces.
4. Consider a narrow array-linked/value-root slice only if it applies cleanly without importing the full array-lvalue surface.
5. Review large `compiler/src/codegen.rs` diffs skeptically before letting them harden.
6. Keep `/dev/shm` above the 10-12 GiB warning zone before primary gates; reclaim inactive target dirs only after checking live cargo/rustc/linker/phpc users.
