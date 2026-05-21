# PHP Native Compiler Progress

Updated: 2026-05-21 19:23 CEST
Evaluation marker: 20260521T172300Z
Final refresh: 20260521T172300Z

This is a distilled roadmap for a supervisor who needs the current momentum quickly. Percentages are candid engineering estimates, not test-suite completion metrics. Primary-integrated capability means committed on `master`; lane-local and dirty-worktree work is candidate material until selected, gated, committed, and pushed.

## Overall Status

Estimated progress toward a broadly usable generalized PHP native compiler: **23%**

```
Generalized runtime/ABI foundations      [#############-------] 63%
Compiler/backend consumers               [############--------] 59%
Executable generalized PHP semantics     [#####---------------] 26%
Arrays, references, COW, lvalues         [###-----------------] 17%
Objects, properties, methods             [##------------------] 10%
Diagnostics/control-flow composition     [###-----------------] 17%
Broad integrated verification            [###-----------------] 14%
```

## Current Primary State

- Primary semantic HEAD before this progress update: `92b8eb4d native: materialize array value ops via runtime`.
- Latest primary-integrated semantic commit: `92b8eb4d native: materialize array value ops via runtime`.
- Product-code state at this refresh: semantic array value-operation runtime/generated-C consumer batch committed and pushed; this `PROGRESS.md` update is separate management metadata and not counted as compiler semantic progress.
- Resource caveat: `/dev/shm` recovered to about 14G free after the integration gate used a disk-backed primary target for its broader checks.

## Grand Roadmap Position

The project has moved from scattered backend rejection paths toward reusable runtime/ABI families and selected generated-C consumers. That is the right foundation, but the product is still much stronger at centralized boundaries/blockers than at executing arbitrary PHP with correct values, diagnostics, references, cleanup, objects, and request/global state.

## Primary-Integrated Roadmap

- [x] Establish supervised parallel implementation lanes and primary integration gate.
- [x] Add shared runtime ABI surfaces for symbols, string boundaries, string truthiness, comparison results, diagnostic severity, numeric-string classification, array-key materialization, and selected conversion helpers.
- [x] Route value, lvalue, argument, reference-source, reference-assignment, statement, and unset call-result contexts through shared call-operation boundaries.
- [x] Route unary, binary, comparison, concat, and skipped `echo` operand call blockers through shared call-operation boundaries across LLVM IR and generated-C backends.
- [x] Materialize generated-C comparison operands through runtime byte/native-value boundaries and consume comparison results through report/free/exit sinks.
- [x] Share comparison operation/value-family dispatch, comparison outcomes, string truthiness, and arithmetic-number operand conversion across runtime consumers.
- [x] Share PHP numeric-string classification across runtime parsing, runtime `is_numeric()`, and compiler `is_numeric()` folding.
- [x] Route generated-C `strlen()`, string predicates, string-int builtins, and string-distance builtins over lowerable values through shared runtime conversion/string ABIs.
- [x] Route generated-C array keyed writes, keyed assignments, and indexed echo reads through runtime array-key materialization.
- [x] Route generated-C array key/value operation consumers through a reusable runtime value-operation result ABI with linked coverage.
- [x] Route generated-C filesystem path/state builtins through a shared operation-tagged runtime boundary and centralized blocker/result surface.
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
| String conversion, truthiness, and byte-buffer results | 45% | 68% | Primary has string-conversion result/free ABI, generated-C `strlen()`, string predicates, `ord()`/`crc32()`, `strcasecmp()`, `substr_count()`, `levenshtein()`, two-argument `similar_text()`, comparison byte materialization, string truthiness, dynamic string lengths, numeric-string classification, and shared runtime value-to-int conversion for selected generated-C operands. Lane-local work now includes more executable string-result surfaces such as byte-preserving `wordwrap()`, but exact diagnostics, object/resource/Stringable parity, non-UTF-8 policy, LLVM parity, and full warning/recovery parity remain limited. |
| Call operation cleanup and ownership | 37% | 56% | Primary routes many call-result contexts through shared blockers. Lanes centralize call cleanup/access/diagnostic families. Actual frames, binding, by-ref args/returns, variadics, callbacks, dynamic dispatch, and return ownership remain mostly non-executable. |
| Comparison/conversion semantics | 42% | 60% | Primary has comparison ABI consumers, centralized outcomes, arithmetic conversion sharing, report/free/exit sinks, byte materialization, numeric-string classification, string truthiness, and native value-to-int conversion for selected generated-C consumers. Leading-numeric recovery, warning ordering, dynamic native `is_numeric()` lowering, and broader conversion-source/pair work are still candidate or blocked material. |
| Arrays, lvalues, references, COW | 17% | 57% | Primary consumes runtime array-key materialization for keyed writes, keyed assignments, indexed echo reads, and now a runtime value-operation result ABI for generated-C array key/value consumers. Lanes still have stronger RMW/update-result, `??=`, direct-owner, value-root, reference-cell, path-preflight, foreach/result, and generated-C lvalue candidates. Primary still lacks full executable array lvalues, foreach, nested writes, references/COW, ArrayAccess, and exact warnings. |
| Symbols, globals, request state | 21% | 44% | Primary has symbol-table ABI helpers. Lanes have expression-result consumer classification, root write value-flow contracts, request-state read access, slot plans, and immutable snapshot consumers. Mutable request/global/superglobal behavior remains early. |
| Objects, properties, methods | 10% | 38% | Lane-local object/property receiver, class-policy, declaration-body blocker, metadata, and stateful operation routes are more coherent, but primary still has little broad executable object/property/method behavior. |
| Diagnostics and control-flow cleanup | 17% | 50% | Severity tags and selected blockers are integrated. Lanes now carry a broader diagnostic-result/family contract with ordered entries, report/free sinks, and producers for conversion, arrays, offsets, undefined variables, invalid callables, and non-object property reads. Exact warning/recovery order and executable cleanup remain broad blockers. |
| Filesystem/path builtins and request state | 12% | 30% | Primary has a shared operation-tagged boundary for generated-C filesystem path/state calls across multiple builtin families. It deliberately reports blockers/placeholder values instead of real filesystem semantics; stat metadata, cache mutation, stream wrappers, warning-plus-false recovery, request state, and exact diagnostics remain open. |
| Broad composition verification | 14% | 31% | Recent primary batches have focused runtime/source/linked executable gates, including full native-link checks for the array value-operation consumer. Broad differential PHP composition coverage remains thin relative to the number of runtime/compiler contracts being introduced. |

## Recent Primary-Integrated Work

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
- Lane-local symbol/request/call candidates: expression-result consumer classification, request-state read access, root write value-flow, slot contract plans, and call operation diagnostic families.
- Lane-local object/control/diagnostic candidates: object/property receiver/class-policy blockers, unsupported declaration-body entrypoints, structured CFG/effect rows, termination cleanup stack scans, request diagnostics, and diagnostic result carriers.

Lane-local work is useful source material, but it is not product capability until selected into a small primary batch, gated, committed, and pushed.

## Candid Assessment

The direction is right. Recent primary commits are turning shared ABIs into generated-C consumers, especially comparison, string conversion, array keys/value operations, numeric strings, and filesystem path-operation blockers.

The product is still boundary-heavy. Many changes make unsupported PHP fail through better semantic surfaces rather than execute more PHP correctly. The next highest-value work is a small executable consumer of an already-landed ABI surface, with cleanup and linked coverage, not more standalone vocabulary.

## Near-Term Steering

1. Count `92b8eb4d` as the latest integrated semantic baseline.
2. Keep preferring executable generated-C/LLVM consumers of existing ABI surfaces over pure blocker or diagnostic vocabulary.
3. Next high-value candidates are narrow string-result execution, array RMW/lvalue behavior, or broader differential composition gates around the value-operation path.
4. Keep filesystem work honest: current primary work centralizes blockers; it does not yet implement real stream/stat/cache/current-directory semantics.
5. Keep `/dev/shm` above the 10-12G free warning band before primary gates; use disk-backed targets for broad checks when lane builds are active.
6. Add broader differential composition checks around the families already touched by primary: comparisons, numeric strings, string conversions/results, array keys/RMW, call-result cleanup, and diagnostics.
