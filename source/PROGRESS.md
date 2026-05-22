# PHP Native Compiler Progress

Updated: 2026-05-22 11:39 CEST
Current primary semantic baseline: `4aef9974 runtime: share symbol and request reference slots`

This is a high-level roadmap view. Percentages are candid engineering estimates, not test-suite pass rates. Lane-local work is not counted as product capability until it is integrated into `master`, gated, committed, and pushed.

## Overall Estimate

Estimated progress toward a broadly usable generalized PHP native compiler: **63%**

| Area | Estimated progress | Current read |
| --- | ---: | --- |
| Runtime and ABI foundations | 92% | Strong shared surfaces now exist for values, arrays, strings, comparisons, diagnostics, symbol tables, request state, and reference slots. |
| Compiler/backend consumers | 97% | Many generated-C and LLVM paths consume shared ABIs, but the hard missing consumers are now in symbols, calls, objects, references, and control flow. |
| Executable generalized PHP semantics | 77% | Selected scalar, string, array, lvalue, symbol, and request runtime behavior works; ordinary broad PHP programs still hit major blockers. |
| Arrays, references, COW, lvalues | 63% | Good selected array/lvalue execution exists; arbitrary writable roots, full references/COW, by-reference foreach, and ArrayAccess/resource offsets remain open. |
| Symbols, globals, request state | 33% | Runtime symbol/request roots can snapshot, populate, and share reference cells; compiler-level `$GLOBALS`, superglobal mutation, request lifetime, and frame propagation are not done. |
| Objects, properties, methods | 11% | Mostly blockers and metadata. Real allocation/property/method behavior remains largely absent. |
| Diagnostics and control-flow cleanup | 26% | Shared diagnostic/status surfaces exist, but exact ordering, recovery, loops/switch/goto/finally/exceptions, and cleanup stacks are not integrated. |
| Broad integrated verification | 65% | Focused gates are useful and recent clean-worktree runtime checks passed; broad differential composition coverage remains thin. |

## Current Primary State

- `master` is pushed and synced with `origin/master` at `4aef9974`.
- The only expected dirty primary diff is the preserved `runtime/src/lib.rs` append/null-slot cleanup hunk. It is not counted here.
- The latest integrated semantic sequence is:
  - `b6e271e6 runtime: snapshot request superglobal storage`
  - `9ca31007 runtime: populate request roots from symbol tables`
  - `4aef9974 runtime: share symbol and request reference slots`
- These commits are generalized runtime/symbol/request foundations. They do not yet mean executable PHP superglobal, `$GLOBALS`, or reference semantics are complete.

## Recent Work

The current wave moved request and symbol state from isolated storage APIs toward shared runtime roots:

- Request state can snapshot backed superglobal arrays and rebuild `$_REQUEST` from `_GET`, `_POST`, and `_COOKIE` policy order.
- Request roots can be populated from native symbol-table snapshots through one runtime ABI.
- Symbol-table slots now store `ArraySlot`s, allowing root symbols to hold shared PHP reference cells instead of only cloned values.
- Request superglobal slots can also share reference cells, so root symbol references and request slots can point at the same runtime cell.
- Clean verification for `4aef9974` passed the full `php_runtime` crate in a separate worktree with only the staged semantic patch applied.

## What Is Still Missing

- Generalized PHP-level symbol lowering: `$GLOBALS`, locals, imports, superglobals, dynamic variables, undefined slots, repeated calls, and request/global separation.
- Real reference/COW semantics: owner-slot/value-slot/reference-slot materialization, mutation barriers, alias visibility, by-reference assignment/args/returns, and cleanup.
- Full array semantics: arbitrary writable roots, string/object/resource offsets, ArrayAccess, by-reference foreach, undefined/scalar recovery, and LLVM parity.
- Function/call semantics: dynamic calls, frame handoff, argument cleanup, by-reference parameters, variadics/spreads, unknown callees, and return ownership.
- Object/property/method semantics: allocation, dynamic names, visibility, magic hooks, stdClass, static properties, property array offsets, references/COW, and diagnostics.
- Control-flow and diagnostics: loops, switch, break/continue, goto/labels, branch joins, cleanup stacks, exact warning/error ordering, and recovery behavior.
- Type conversion/comparison parity: numeric strings, truthiness, object/resource/reference blockers, recursive structures, and backend parity.

## Roadmap Priorities

1. Turn landed runtime/symbol/request reference surfaces into executable compiler/backend consumers.
2. Replace blockers with real generalized behavior one semantic family at a time, starting where the runtime ABI is already present.
3. Keep integrating small primary batches with focused gates and immediate pushes.
4. Reject whole-lane merges, fixture-shaped production lowering, generated-source substring-only progress, and blocker vocabulary that does not unlock execution.
5. Expand broad composition verification after each semantic family gains a real consumer.

## Near-Term Steering

The next best primary slices should be executable consumers rather than more runtime-only vocabulary:

- whole-bag request/superglobal reads or `isset()` / `empty()` probes over the request-state boundary;
- compiler/backend consumers for shared symbol/request reference slots;
- a narrow references/COW owner-slot materialization slice;
- a concrete array/object/call/control-flow blocker replacement with linked executable evidence.

Lane-local work remains useful source material, but only pushed primary commits count as shipped compiler capability.
