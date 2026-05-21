# PHP Native Compiler Progress

Updated: 2026-05-21 16:20 CEST
Evaluation marker: 20260521T135026Z

This is a high-level roadmap for a supervisor who needs the current momentum quickly. Percentages are candid engineering estimates, not test-suite completion metrics. Primary-integrated capability means committed on `master`; lane-local work is candidate material only until selected, gated, committed, and pushed.

## Overall Status

Estimated progress toward a broadly usable generalized PHP native compiler: **20%**

```
Generalized runtime/ABI foundations      [############--------] 58%
Compiler/backend consumers               [#########-----------] 44%
Executable generalized PHP semantics     [####----------------] 19%
Arrays, references, COW, lvalues         [##------------------] 12%
Objects, properties, methods             [##------------------] 10%
Diagnostics/control-flow composition     [###-----------------] 17%
Broad integrated verification            [##------------------] 12%
```

## Grand Roadmap Position

The project is moving from isolated backend rejection paths toward shared semantic-family boundaries and compiler/backend consumers. The bottleneck is still executable generalized PHP behavior: calls, conversions, arrays, references/COW, symbols, objects, diagnostics, cleanup, and composition tests need to stop merely reaching good blockers and start executing correct PHP semantics family by family.

## Primary-Integrated Roadmap

- [x] Establish supervised parallel implementation lanes and primary integration gate.
- [x] Add shared runtime ABI surfaces for symbols, string boundaries, string truthiness, comparison results, diagnostic severity, numeric-string classification, and selected conversion helpers.
- [x] Route value, lvalue, argument, reference-source, reference-assignment, statement, and unset call-result contexts through shared call-operation boundaries.
- [x] Consume comparison branch-result ABI from generated C instead of direct struct-field reads.
- [x] Materialize generated-C comparison string operands through a runtime byte-boundary with diagnostic results.
- [x] Expose native string-conversion result/free boundaries for value/reference conversion blockers.
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
| String conversion, truthiness, and byte-buffer results | 34% | 58% | Primary has string-conversion result/free ABI, comparison byte materialization, runtime-shared string truthiness, generated-C tracked dynamic string lengths, and runtime numeric-string classification. Lanes have broader string-result/debug/config/callable/regex blockers. Production execution and exact diagnostics remain limited. |
| Call operation cleanup and ownership | 33% | 52% | Primary routes many call-result contexts through shared blockers. Actual frames, binding, by-ref args/returns, variadics, callbacks, dynamic dispatch, and return ownership remain mostly non-executable. |
| Comparison/conversion semantics | 36% | 55% | Primary has comparison ABI consumers, canonical branch-result predicates/exit-code consumers, runtime numeric-string reuse, generated-C string-byte comparison operands with tracked dynamic lengths, and shared string truthiness for known conditions/logical expressions. Lane conversion-source/pair work is promising but not integrated. |
| Arrays, lvalues, references, COW | 12% | 44% | Lane-local generated-C owner-slot/RMW/reference-cell work is strong, but primary still lacks full executable array lvalue, foreach, reference/COW, and ArrayAccess behavior. |
| Symbols, globals, request state | 21% | 39% | Primary has symbol-table ABI helpers; lanes have frame-slot/request operation contracts. Full request/global/superglobal behavior and exact diagnostics remain early. |
| Objects, properties, methods | 10% | 33% | Mostly lane-local blocker/carrier work. Primary has little broad executable object/property/method behavior. |
| Diagnostics and control-flow cleanup | 17% | 38% | Severity tags and selected blockers are integrated; request diagnostics and cleanup lanes are active. Exact warning/recovery order and structured cleanup are not broad. |
| Broad composition verification | 12% | 27% | Focused gates are good and primary is clean; broad PHP composition and differential coverage remain limited. |

## Recent Integrated Primary Work

Recent pushed primary commits show useful movement from lane-local artifacts into `master`:

- `7c4017f1 codegen: consume comparison branch predicates`
  - Adds canonical runtime accessors for comparison branch truth and process exit status, then routes generated-C comparison control flow through those predicates instead of backend-local status/value field checks. This keeps loose, strict, ordering, scalar/null/string, dynamic byte-string, and branch-condition comparison consumers on one result contract.
- `495526a2 codegen: track native C string lengths`
  - Tracks generated-C string-expression byte lengths through ternary lowering so mixed-length and embedded-NUL dynamic string operands materialize through `phpc_native_value_from_string_bytes_with_diagnostic(..., len, ...)` and the owned runtime comparison branch ABI instead of C `strlen()`/`strcmp()` shortcuts.
- `227c3a63 runtime: share PHP string truthiness semantics`
  - Adds a shared runtime `is_php_truthy_string()` helper and routes runtime value truthiness plus LLVM/generated-C known string condition, ternary, short-ternary, logical, and logical-not consumers through it. This removes duplicate compiler-local string truthiness parsing without adding dynamic exact-shape lowering.
- `5c6b9393 codegen: route unset calls through statement boundary`
  - Consolidates unset lvalue call-result preflight behind the shared statement call-operation boundary and removes duplicated LLVM/generated-C unset checks.
- `b8052d11 codegen: route reference assignment calls through call boundary`
  - Routes reference-assignment target and source operands through the shared call-operation boundary before generic reference-assignment blockers.
- `d11d3c8d codegen: route statement operands through call boundary`
  - Routes call results in statement operands through the shared call-operation cleanup boundary before broader statement blockers.
- `d535a734 native: materialize comparison strings from bytes`
  - Adds a runtime byte-buffer-to-native-value boundary with diagnostics and routes generated-C comparison string operands through it.
- `869bc280 native: route reference-source call operands`
  - Routes reference-source lvalue operands containing direct, dynamic, method, and constructor call results through the shared native call-operation boundary.
- Earlier recent batches integrated native string-conversion results, runtime numeric-string classifier consumption, comparison branch accessors, value/lvalue/call-argument call-boundary consumers, diagnostic severity ABI, symbol-table ABI helpers, scalar string boundaries, and runtime symbol table ABI.

## Current Primary State

During this progress refresh, primary git was clean and synced with `origin/master` at `7c4017f1 codegen: consume comparison branch predicates`. The latest semantic compiler/runtime commit is `7c4017f1`.

## Lane-Local Candidate Work

Lane-local workers are producing candidate work in these areas:

- binary-safe string/value operation boundaries and many generated-C string-result blocker consumers;
- array lvalue/read-modify-write/value-result contracts and generated-C owner-slot routing;
- linked symbol/value-flow and expression-result consumer cleanup contracts;
- runtime call/frame result and cleanup contracts, including stored caller call-result boundaries;
- conversion-source and pair-conversion runtime ABI work;
- object/property operation carriers and assign-target blocker families;
- reference/owner-cell resolved operation and transfer result scaffolding;
- request/superglobal operation snapshots and diagnostic consumers;
- comparison null predicates and byte-length operand metadata;
- control-flow, termination, branch merge, and cleanup/unwind contracts.

Lane-local work is useful source material, but it is not product capability until selected into a small primary batch, gated, committed, and pushed.

## Candid Assessment

The direction is right. Primary integration is landing real generalized consumers instead of simply accumulating lane-local patches. That said, the product is still boundary-heavy: many improvements make unsupported PHP fail through better semantic surfaces, while broad successful PHP execution remains narrow.

The next highest-value work is to convert landed ABI/result surfaces into actual backend behavior with cleanup, diagnostics, and composition tests. More result vocabulary is lower priority unless it immediately feeds executable generated-C/LLVM consumers or replaces a blocker with real generalized behavior.

## Near-Term Steering

1. Keep integrating one small generalized primary batch at a time.
2. Prefer executable compiler/generated-C/LLVM consumers of already-landed ABI surfaces over more standalone vocabulary.
3. Consider the array linked-executable/value-slot lane only as a narrow gated slice, not a wholesale merge.
4. Reject fixture-shaped or one-source-shape production paths, even when they make tests pass.
5. Review large `compiler/src/codegen.rs` diffs skeptically before letting them harden.
6. Keep `/dev/shm` above the 10-12 GiB warning zone before primary gates; reclaim inactive target dirs only after checking live cargo/rustc/linker/phpc users.
