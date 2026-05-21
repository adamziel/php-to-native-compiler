# PHP Native Compiler Progress

Updated: 2026-05-21 20:01 CEST
Evaluation marker: 20260521T180100Z
Final refresh: 20260521T180100Z

This is a distilled roadmap for a supervisor who needs the current momentum quickly. Percentages are candid engineering estimates, not test-suite completion metrics. Primary-integrated capability means committed on `master`; lane-local and dirty-worktree work is candidate material until selected, gated, committed, and pushed.

## Overall Status

Estimated progress toward a broadly usable generalized PHP native compiler: **24%**

```
Generalized runtime/ABI foundations      [##############------] 68%
Compiler/backend consumers               [############--------] 60%
Executable generalized PHP semantics     [#####---------------] 27%
Arrays, references, COW, lvalues         [####----------------] 18%
Objects, properties, methods             [##------------------] 10%
Diagnostics/control-flow composition     [###-----------------] 17%
Broad integrated verification            [###-----------------] 15%
```

## Current Primary State

- Primary semantic HEAD before this progress update: `a93f3bb5 runtime: centralize value string semantics`.
- Latest primary-integrated semantic commit: `a93f3bb5 runtime: centralize value string semantics`.
- Product-code state at this refresh: generalized full-value string-form runtime boundary committed and pushed; this `PROGRESS.md` update is separate management metadata and not counted as compiler semantic progress.
- Resource caveat: `/dev/shm` is healthy at about 16G free after active-process-aware cleanup and active lane builds.

## Grand Roadmap Position

The project has moved from scattered backend rejection paths toward reusable runtime/ABI families and selected generated-C consumers. That is the right foundation, but the product is still much stronger at centralized boundaries/blockers than at executing arbitrary PHP with correct values, diagnostics, references, cleanup, objects, and request/global state.

## Primary-Integrated Roadmap

- [x] Establish supervised parallel implementation lanes and primary integration gate.
- [x] Add shared runtime ABI surfaces for symbols, string boundaries, string truthiness, comparison results, diagnostic severity, numeric-string classification, array-key materialization, and selected conversion helpers.
- [x] Route value, lvalue, argument, reference-source, reference-assignment, statement, and unset call-result contexts through shared call-operation boundaries.
- [x] Route unary, binary, comparison, concat, and skipped `echo` operand call blockers through shared call-operation boundaries across LLVM IR and generated-C backends.
- [x] Materialize generated-C comparison operands through runtime byte/native-value boundaries and consume comparison results through report/free/exit sinks.
- [x] Route generated-C comparison operands through a reusable comparison operand ABI that owns value materialization diagnostics and branch/result cleanup.
- [x] Share comparison operation/value-family dispatch, comparison outcomes, string truthiness, and arithmetic-number operand conversion across runtime consumers.
- [x] Share PHP numeric-string classification across runtime parsing, runtime `is_numeric()`, and compiler `is_numeric()` folding.
- [x] Route generated-C `strlen()`, string predicates, string-int builtins, and string-distance builtins over lowerable values through shared runtime conversion/string ABIs.
- [x] Route generated-C array keyed writes, keyed assignments, and indexed echo reads through runtime array-key materialization.
- [x] Route generated-C array key/value operation consumers through a reusable runtime value-operation result ABI with linked coverage.
- [x] Route generated-C filesystem path/state builtins through a shared operation-tagged runtime boundary and centralized blocker/result surface.
- [x] Add a generalized request-state/superglobal value snapshot ABI with bag/key coercion status, operation results, cleanup, and pointer-width ABI probes.
- [x] Add branch/free runtime ABI helpers for array comparison results so array comparison diagnostics, branch results, and handle cleanup share the same comparison outcome contract.
- [x] Centralize full-value PHP string-form semantics for runtime echo/scalar/native conversion consumers.
- [ ] Replace selected shared blockers with real generalized execution for one semantic family at a time.
- [ ] Generalize value/result ownership across returns, call args, conditions, branch joins, discarded temporaries, stdout, and cleanup.
- [ ] Implement full array lvalue/RMW semantics, writable roots, foreach/by-ref foreach, ArrayAccess/object/resource offsets, and COW/reference behavior.
- [ ] Implement full symbol environment semantics for roots, locals, imports, globals, superglobals, undefined slots, repeated calls, and request/global separation.
- [ ] Implement function/method call frames, argument binding/cleanup, by-ref args/returns, variadics/spreads, dynamic calls, callbacks, constructors, and frame handoff.
- [ ] Implement object/class/property semantics including dynamic names, visibility/magic hooks, stdClass behavior, property offsets, diagnostics, and references/COW.
- [ ] Implement generalized diagnostics, conversion/comparison semantics, control-flow cleanup, loops/switch/break/continue/goto/finally blockers, and broad composition tests.

## Active Roadmap Estimates

| Active item | Primary-integrated estimate | Candidate maturity | Current read |
| --- | ---: | ---: | --- |
| String conversion, truthiness, and byte-buffer results | 47% | 69% | Primary has string-conversion result/free ABI, generated-C `strlen()`, string predicates, `ord()`/`crc32()`, `strcasecmp()`, `substr_count()`, `levenshtein()`, two-argument `similar_text()`, comparison byte materialization, string truthiness, dynamic string lengths, numeric-string classification, shared runtime value-to-int conversion for selected generated-C operands, and a centralized full-value string-form analyzer feeding echo/scalar/native string consumers. Lane-local work now includes more executable string-result surfaces such as byte-preserving `wordwrap()`, but exact diagnostics, object/resource/Stringable parity, non-UTF-8 policy, LLVM parity, and full warning/recovery parity remain limited. |
| Call operation cleanup and ownership | 37% | 56% | Primary routes many call-result contexts through shared blockers. Lanes centralize call cleanup/access/diagnostic families. Actual frames, binding, by-ref args/returns, variadics, callbacks, dynamic dispatch, and return ownership remain mostly non-executable. |
| Comparison/conversion semantics | 44% | 61% | Primary has comparison ABI consumers, centralized outcomes, arithmetic conversion sharing, report/free/exit sinks, byte materialization, comparison operand materialization with owned diagnostics, numeric-string classification, string truthiness, native value-to-int conversion for selected generated-C consumers, and array comparison branch/free runtime ABI helpers. Leading-numeric recovery, warning ordering, dynamic native `is_numeric()` lowering, generated-C array comparison consumers, and broader conversion-source/pair work are still candidate or blocked material. |
| Arrays, lvalues, references, COW | 18% | 57% | Primary consumes runtime array-key materialization for keyed writes, keyed assignments, indexed echo reads, a runtime value-operation result ABI for generated-C array key/value consumers, and now shared array comparison result/branch/free runtime helpers. Lanes still have stronger RMW/update-result, `??=`, direct-owner, value-root, reference-cell, path-preflight, foreach/result, and generated-C lvalue candidates. Primary still lacks full executable array lvalues, foreach, nested writes, references/COW, ArrayAccess, generated-C array comparison consumers, and exact warnings. |
| Symbols, globals, request state | 24% | 46% | Primary has symbol-table ABI helpers plus a request-state/superglobal value snapshot ABI covering request bags, key coercion status, value/array/presence result shape, cleanup, and pointer-width ABI probes. Lanes have expression-result consumer classification, root write value-flow contracts, slot plans, and immutable snapshot consumers. Mutable request/global/superglobal behavior, symbol-table integration, writes/unset, repeated calls, references/COW, and exact diagnostics remain early. |
| Objects, properties, methods | 10% | 38% | Lane-local object/property receiver, class-policy, declaration-body blocker, metadata, and stateful operation routes are more coherent, but primary still has little broad executable object/property/method behavior. |
| Diagnostics and control-flow cleanup | 17% | 50% | Severity tags and selected blockers are integrated. Lanes now carry a broader diagnostic-result/family contract with ordered entries, report/free sinks, and producers for conversion, arrays, offsets, undefined variables, invalid callables, and non-object property reads. Exact warning/recovery order and executable cleanup remain broad blockers. |
| Filesystem/path builtins and request state | 15% | 33% | Primary has a shared operation-tagged boundary for generated-C filesystem path/state calls across multiple builtin families and now has a request-state/superglobal snapshot ABI. Filesystem work still deliberately reports blockers/placeholder values instead of real filesystem semantics; request-state work is snapshot/read ABI infrastructure, not full mutable superglobal semantics. Stat metadata, cache mutation, stream wrappers, warning-plus-false recovery, writes/unset, request/global separation, and exact diagnostics remain open. |
| Broad composition verification | 15% | 31% | Recent primary batches have focused runtime/source/linked executable gates, including full native-link checks for the array value-operation consumer and comparison operand ABI. Broad differential PHP composition coverage remains thin relative to the number of runtime/compiler contracts being introduced. |

## Recent Primary-Integrated Work

- `a93f3bb5 runtime: centralize value string semantics`
  - Adds `PhpValueStringSemantics` / `analyze_php_value_string_semantics(&Value)` as a shared runtime string-form boundary for null, booleans, ints, floats, strings, arrays, object/closure display form, and resource display form. Runtime echo, scalar string-byte materialization, checked string conversion, array string-comparison consumers, and native value-to-string conversion results now share that analyzer. Object `__toString`, warning-plus-value recovery, binary/non-UTF-8 string storage, generated-native dynamic consumers, and exact diagnostics remain open.
- `150e3733 native: add array compare branch/free ABI`
  - Adds array comparison branch/free runtime helpers that reuse the shared comparison outcome contract for result conversion, blocked diagnostics, branch status/value reporting, and owned handle cleanup. This improves array/comparison ABI consistency, but generated-C/LLVM array comparison consumers, full PHP array comparison semantics, references/COW, ArrayAccess, and exact diagnostics remain open.
- `b05ed08b native: add request-state value snapshot ABI`
  - Moves request state beyond a null-only handle by adding request bag storage, scalar-key coercion status, a reusable superglobal operation result, cleanup/free APIs, and ABI probe coverage for both pointer widths. This is a real generalized runtime/ABI surface for request/superglobal snapshots, but mutable symbol-state integration, writes/unset, repeated-call request separation, references/COW, generated-C consumers, and exact diagnostics remain open.
- `a56f1953 native: route comparisons through operand ABI`
  - Adds a reusable comparison operand ABI that owns value handles plus materialization diagnostics, then routes generated-C comparison lowering through operand comparison/result/branch helpers with cleanup. This reduces backend-local comparison materialization handling, but broader comparison/conversion parity, exact diagnostics, and LLVM parity remain open.
- `92b8eb4d native: materialize array value ops via runtime`
  - Adds a runtime value-operation result ABI and routes generated-C array key/value consumers through that shared result path, with focused runtime and native-link coverage. This is closer to executable generalized array/value behavior than pure blocker vocabulary, but full lvalues, foreach, references/COW, ArrayAccess, and exact diagnostics remain open.
- `13a5d783 native: route filesystem path ops through shared boundary`
  - Adds a reusable runtime filesystem path-operation ABI and routes generated-native C filesystem path/state builtins through that operation-tagged boundary, with focused runtime/native-link coverage. This is generalized blocker/backend infrastructure, not complete executable filesystem behavior.
- `91481d33 native: convert string integer args through runtime`
  - Adds a shared runtime value-to-int conversion ABI for generated-native string offset, string length, and string distance cost operands, then routes generated-C `substr_count()` offset/length and `levenshtein()` cost arguments through that diagnostic boundary.
- `cafb8dc5 native: extend string-int runtime boundary`
  - Extends the shared string-int runtime ABI and generated-C consumers from `ord()`/`crc32()` to lowerable `strcasecmp()` and two-/three-/four-argument `substr_count()`.
- `5d75ac52 runtime: classify numeric strings centrally`
  - Adds shared `PhpNumericStringClassification` / `classify_php_numeric_string()` and routes runtime parsing, runtime `is_numeric()`, and compiler `is_numeric()` folding through that boundary.
- `438ca546 native: route string distance through runtime`
  - Adds a runtime string-distance ABI and generated-C consumers for lowerable `levenshtein()` and two-argument `similar_text()` over the shared value-to-string byte boundary.
- `65f74970 native: materialize array keys in generated C`
  - Adds runtime array-key materialization and generated-C consumers for keyed array inserts, keyed assignments, indexed echo reads, diagnostics, and cleanup across scalar/string/null/bool/integral-float key families.
- `d2411a58 runtime: centralize comparison outcomes`
  - Shares comparison evaluation outcomes before native result or branch conversion.

## Candidate Work Not Yet Counted

- Lane-local array candidates: broader RMW/update-result contracts, null-coalescing assignment through the shared RMW boundary, direct owner policy, foreach/result finishers, reference-cell/lvalue contracts, and array path preflight.
- Lane-local string/filesystem candidates: byte-preserving string-result execution such as `wordwrap()`, broader binary-safe string surfaces, printable/UU/byte-wrap helpers, stream/resource/path metadata boundaries, and interpreter/runtime PHP string-byte boundaries.
- Lane-local symbol/request/call candidates: expression-result consumer classification, request-state generated-C consumers, mutable request/global/superglobal symbol-state, root write value-flow, slot contract plans, and call operation diagnostic families.
- Lane-local object/control/diagnostic candidates: object/property receiver/class-policy blockers, unsupported declaration-body entrypoints, structured CFG/effect rows, termination cleanup stack scans, request diagnostics, and diagnostic result carriers.

Lane-local work is useful source material, but it is not product capability until selected into a small primary batch, gated, committed, and pushed.

## Candid Assessment

The direction is right. Recent primary commits are turning shared ABIs into generated-C consumers and reusable runtime surfaces, especially comparison operand/result handling, string conversion, array keys/value operations, numeric strings, filesystem path-operation blockers, and request-state snapshots.

The product is still boundary-heavy. Many changes make unsupported PHP fail through better semantic surfaces rather than execute more PHP correctly. The next highest-value work is a small executable consumer of an already-landed ABI surface, with cleanup and linked coverage, not more standalone vocabulary.

## Near-Term Steering

1. Count `a93f3bb5` as the latest integrated semantic baseline.
2. Keep preferring executable generated-C/LLVM consumers of existing ABI surfaces over pure blocker or diagnostic vocabulary.
3. Next high-value candidates are generated-C consumers of the value string semantics boundary, generated-C consumers of the request-state snapshot ABI, generated-C consumers of the array comparison branch/free ABI, narrow string-result execution, array RMW/lvalue behavior, or broader differential composition gates around the value-operation path.
4. Keep filesystem work honest: current primary work centralizes blockers; it does not yet implement real stream/stat/cache/current-directory semantics.
5. Keep `/dev/shm` above the 10-12G free warning band before primary gates; use disk-backed targets for broad checks when lane builds are active.
6. Add broader differential composition checks around the families already touched by primary: comparisons, numeric strings, string conversions/results, array keys/RMW, call-result cleanup, and diagnostics.
