# PHP Native Compiler Progress

Updated: 2026-05-21 15:19 CEST
Evaluation marker: 20260521T125958Z

This file is a high-level roadmap and status digest for the native PHP compiler effort. Percentages are candid engineering estimates, not test-suite completion metrics. Primary-integrated capability means it is committed on `master`; lane-local candidate work is useful source material but is not counted as product capability until selected, gated, committed, and pushed.

## Overall Status

Estimated progress toward a broadly usable generalized PHP native compiler: **20%**

```
Generalized runtime/ABI foundations      [###########---------] 56%
Compiler/backend consumers               [#######-------------] 36%
Executable generalized PHP semantics     [###-----------------] 17%
Arrays, references, COW, lvalues         [##------------------] 11%
Objects, properties, methods             [##------------------] 10%
Diagnostics/control-flow composition     [###-----------------] 16%
Broad integrated verification            [##------------------] 11%
```

## Grand Roadmap Position

The leading edge is runtime/ABI and compiler-boundary convergence. The bottleneck remains executable PHP semantics: turning shared result/blocker surfaces into real native behavior across calls, conversions, arrays, references, objects, diagnostics, cleanup, and composition tests.

## Primary-Integrated Roadmap

- [x] Establish supervised parallel implementation lanes and primary integration gate.
- [x] Add shared runtime ABI surfaces for symbols, string boundaries, comparison results, diagnostic severity, and selected conversion helpers.
- [x] Route multiple native call-result contexts through shared call-operation blockers instead of exact-shape backend rejections.
- [x] Consume comparison branch-result ABI from generated C instead of direct struct-field reads.
- [x] Share PHP numeric-string classification between runtime and codegen.
- [x] Expose native string-conversion result/free boundaries for value/reference conversion blockers.
- [x] Route reference-source lvalue call operands through the shared call-operation boundary.
- [x] Materialize generated-C comparison string operands through a runtime byte-boundary with diagnostic results.
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
| String conversion and byte-buffer results | 30% | 55% | Result/free ABI is integrated; production backend consumption and exact PHP diagnostics remain blocked. |
| Call operation cleanup and ownership | 26% | 45% | More reference-source and lvalue operand call contexts route through shared blockers; actual frames, binding, by-ref args, variadics, callbacks, and returns are not native-executable yet. |
| Comparison/conversion semantics | 32% | 52% | Comparison ABI, known numeric-string classification, and generated-C string-byte operand materialization are integrated; full PHP comparison order, recovery, and broad dynamic operands remain incomplete. |
| Arrays, lvalues, references, COW | 11% | 40% | Strong lane-local generated-C/result-boundary work exists; current uncommitted primary/test diffs are not counted yet. |
| Symbols, globals, request state | 20% | 35% | Symbol-table ABI helpers are integrated; full request/global/superglobal behavior is still early. |
| Objects, properties, methods | 10% | 30% | Mostly lane-local carriers/blockers; little primary executable behavior. |
| Diagnostics and control-flow cleanup | 16% | 35% | Severity tags and selected blockers are integrated; exact warning/recovery order and structured cleanup are not broad yet. |
| Broad composition verification | 11% | 25% | Focused gates are good; broad PHP composition coverage remains limited. |

## Recent Integrated Primary Work

Recent pushed primary commits show useful movement from lane-local artifacts into `master`:

- `d535a734 native: materialize comparison strings from bytes`
  - Adds a runtime byte-buffer-to-native-value boundary with diagnostics and routes generated-C comparison string operands through it, including dynamic string operand coverage.
- `869bc280 native: route reference-source call operands`
  - Routes reference-source lvalue operands containing direct, dynamic, method, and constructor call results through the shared native call-operation boundary in LLVM and generated-C paths.
- `1aab675e runtime: expose native string conversion results`
  - Adds `NativeStringConversionResult`, runtime value/reference string-conversion entrypoints, result cleanup, and ABI probe coverage. This is a generalized runtime/ABI surface; broad production consumption is still pending.
- `1c7b0495 codegen: reuse runtime numeric string semantics`
  - Exposes the runtime PHP numeric-string classifier and makes LLVM/generated-C known-value `is_numeric()` lowering use the shared runtime semantics.
- `6c1b8eaa native: consume comparison branch accessors`
  - Adds runtime accessors for comparison branch results and makes generated C consume status/value through the ABI boundary.
- `9be386f1 native: route value operands through call boundary`
  - Routes call results inside unsupported value-expression operands through the shared native call-operation boundary.
- Prior recent batches integrated value/lvalue/call argument call-boundary consumers, comparison ABI, diagnostic severity ABI, symbol-table ABI helpers, scalar string boundaries, and runtime symbol table ABI.

## Current Primary State

Primary git is synced with `origin/master`. The latest semantic compiler/runtime commit is `d535a734 native: materialize comparison strings from bytes`; the primary worktree was clean immediately after that progress update.

## Lane-Local Candidate Work

Lane-local workers are producing candidate work in these areas:

- binary-safe string/value operation boundaries;
- array lvalue/read-modify-write/value-result contracts;
- linked generated-C symbol/value-flow contracts;
- runtime call/frame result and cleanup contracts;
- object/property operation carriers;
- reference/owner-cell scaffolding;
- diagnostic condition/result consumers;
- comparison/conversion runtime consumers.

Lane-local work is useful source material, but it is not product capability until selected into a small primary batch, gated, committed, and pushed.

## Candid Assessment

The direction is right: the project is moving away from exact-shape lowering and toward reusable semantic-family boundaries. The primary integration loop is now landing real slices instead of only accumulating lane-local patches.

The biggest limitation is still that much of the work is infrastructure or shared blockers. Routing failures through the right generalized boundary is necessary, but it is not the same as executing broad PHP semantics. The next highest-value work is to convert landed ABI/result surfaces into actual generalized compiler/backend behavior with composition tests.

## Near-Term Steering

1. Keep integrating one small generalized primary batch at a time.
2. Prefer executable compiler/generated-C/LLVM consumers of already-landed ABI surfaces over more standalone vocabulary.
3. Reject fixture-shaped or one-source-shape production paths, even if they make tests pass.
4. Review large `compiler/src/codegen.rs` diffs skeptically before letting them harden.
5. Treat `/dev/shm` below roughly 10-12 GiB free as a warning zone for primary gates; reclaim inactive target dirs only after checking active cargo/rustc/linker/phpc users.
