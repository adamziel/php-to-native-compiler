# PHP Native Compiler Progress

Updated: 2026-05-21 15:03 CEST

This file is a high-level roadmap and status digest for the native PHP compiler effort. Percentages are candid engineering estimates, not test-suite completion metrics. They separate work that is integrated in `master` from broader lane-local work that still needs review, integration, and gates.

## Overall Status

Estimated progress toward a broadly usable generalized PHP native compiler: **18%**

```
Generalized runtime/ABI foundations      [##########----------] 52%
Compiler/backend consumers               [######--------------] 32%
Executable generalized PHP semantics     [###-----------------] 16%
Arrays, references, COW, lvalues         [##------------------] 10%
Objects, properties, methods             [##------------------] 10%
Diagnostics/control-flow composition     [###-----------------] 15%
Broad integrated verification            [##------------------] 10%
```

## Roadmap

- [x] Establish supervised parallel implementation lanes and primary integration gate.
- [x] Add shared runtime ABI surfaces for symbols, string boundaries, comparison results, diagnostic severity, and selected conversion helpers.
- [x] Route multiple native call-result contexts through shared call-operation blockers instead of exact-shape backend rejections.
- [x] Consume comparison branch-result ABI from generated C instead of direct struct-field reads.
- [x] Share PHP numeric-string classification between runtime and codegen.
- [x] Expose native string-conversion result/free boundaries for value/reference conversion blockers.
- [ ] Replace selected shared blockers with real generalized execution for one semantic family at a time.
- [ ] Generalize value/result ownership across returns, call args, conditions, branch joins, discarded temporaries, stdout, and cleanup.
- [ ] Implement full array lvalue/RMW semantics, writable roots, foreach/by-ref foreach, ArrayAccess/object/resource offsets, and COW/reference behavior.
- [ ] Implement full symbol environment semantics for roots, locals, imports, globals, superglobals, undefined slots, repeated calls, and request/global separation.
- [ ] Implement function/method call frames, argument binding/cleanup, by-ref args/returns, variadics/spreads, dynamic calls, callbacks, constructors, and frame handoff.
- [ ] Implement object/class/property semantics including dynamic names, visibility/magic hooks, stdClass behavior, property offsets, diagnostics, and references/COW.
- [ ] Implement generalized diagnostics, conversion/comparison semantics, control-flow cleanup, loops/switch/break/continue/goto/finally blockers, and broad composition tests.

## Recent Integrated Primary Work

Recent pushed primary commits show useful movement from lane-local artifacts into `master`:

- `1c7b0495 codegen: reuse runtime numeric string semantics`
  - Exposes the runtime PHP numeric-string classifier and makes LLVM/generated-C known-value `is_numeric()` lowering use the shared runtime semantics.
- `1aab675e runtime: expose native string conversion results`
  - Adds a generalized runtime/ABI string-conversion result/free boundary for value/reference conversion success and blockers, with compiler ABI probes and focused scalar echo coverage.
- `6c1b8eaa native: consume comparison branch accessors`
  - Adds runtime accessors for comparison branch results and makes generated C consume status/value through the ABI boundary.
- `9be386f1 native: route value operands through call boundary`
  - Routes call results inside unsupported value-expression operands through the shared native call-operation boundary.
- Prior recent batches integrated value/lvalue/call argument call-boundary consumers, comparison ABI, diagnostic severity ABI, symbol-table ABI helpers, scalar string boundaries, and runtime symbol table ABI.

## Current Work In Flight

The primary worktree is currently clean and synced with `origin/master` at `1aab675e`. The latest string-conversion slice is counted as integrated because it was tested, committed, and pushed.

Lane-local workers are also producing candidate work in these areas:

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

The biggest limitation is that much of the work is still infrastructure or shared blockers. Routing failures through the right generalized boundary is necessary, but it is not the same as executing broad PHP semantics. The next highest-value work is to convert landed ABI/result surfaces into actual generalized compiler/backend behavior with composition tests.

## Near-Term Steering

1. Keep integrating one small generalized primary batch at a time.
2. Prefer executable compiler/generated-C/LLVM consumers of already-landed ABI surfaces over more standalone vocabulary.
3. Reject fixture-shaped or one-source-shape production paths, even if they make tests pass.
4. Review large `compiler/src/codegen.rs` diffs skeptically before letting them harden.
5. Keep `/dev/shm` above the restart threshold by reclaiming inactive target dirs only after checking active cargo/rustc/linker users.
