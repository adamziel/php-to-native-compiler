# PHP Native Compiler Progress

Updated: 2026-05-21 18:25 CEST
Evaluation marker: 20260521T161953Z
Final refresh: 20260521T162508Z

This is a distilled roadmap for a supervisor who needs the current momentum quickly. Percentages are candid engineering estimates, not test-suite completion metrics. Primary-integrated capability means committed on `master`; lane-local and dirty-worktree work is candidate material until selected, gated, committed, and pushed.

## Overall Status

Estimated progress toward a broadly usable generalized PHP native compiler: **22%**

```
Generalized runtime/ABI foundations      [############--------] 60%
Compiler/backend consumers               [###########---------] 55%
Executable generalized PHP semantics     [#####---------------] 24%
Arrays, references, COW, lvalues         [###-----------------] 16%
Objects, properties, methods             [##------------------] 10%
Diagnostics/control-flow composition     [###-----------------] 17%
Broad integrated verification            [###-----------------] 13%
```

## Current Primary State

- Primary semantic HEAD before this progress update: `438ca546 native: route string distance through runtime`.
- Latest primary-integrated semantic commit: `438ca546 native: route string distance through runtime`.
- Product-code state at this refresh: semantic string-distance batch committed; this `PROGRESS.md` update is separate management metadata and not counted as compiler semantic progress.
- Dashboard caveat: `state/dashboard.md` lagged live git during this review, so current repo state outranks dashboard text.

## Grand Roadmap Position

The project has moved from scattered backend rejection paths toward reusable runtime/ABI families and selected generated-C consumers. The main bottleneck is still executable generalized PHP behavior: calls, arrays/lvalues, references/COW, symbols, objects, diagnostics, cleanup, and composition tests need to become correct runtime behavior family by family.

## Primary-Integrated Roadmap

- [x] Establish supervised parallel implementation lanes and primary integration gate.
- [x] Add shared runtime ABI surfaces for symbols, string boundaries, string truthiness, comparison results, diagnostic severity, numeric-string classification, array-key materialization, and selected conversion helpers.
- [x] Route value, lvalue, argument, reference-source, reference-assignment, statement, and unset call-result contexts through shared call-operation boundaries.
- [x] Route unary, binary, comparison, concat, and skipped `echo` operand call blockers through shared call-operation boundaries across LLVM IR and generated-C backends.
- [x] Materialize generated-C comparison operands through runtime byte/native-value boundaries and consume comparison results through report/free/exit sinks.
- [x] Share comparison operation/value-family dispatch, comparison outcomes, string truthiness, and arithmetic-number operand conversion across runtime consumers.
- [x] Route generated-C `strlen()`, string predicates, and string-int builtins over lowerable values through runtime value-to-string ABIs.
- [x] Route generated-C `levenshtein()` and two-argument `similar_text()` over lowerable values through a runtime string-distance ABI.
- [x] Route generated-C array keyed writes, keyed assignments, and indexed echo reads through runtime array-key materialization.
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
| String conversion, truthiness, and byte-buffer results | 43% | 63% | Primary has string-conversion result/free ABI, generated-C `strlen()`, string predicates, `ord()`/`crc32()`, `levenshtein()`, two-argument `similar_text()`, comparison byte materialization, string truthiness, dynamic string lengths, and numeric-string classification. Lanes add broader binary-safe string-result surfaces, but exact diagnostics, object/resource/Stringable parity, non-UTF-8 storage policy, `similar_text()` percent output, and LLVM parity remain limited. |
| Call operation cleanup and ownership | 37% | 56% | Primary routes many call-result contexts through shared blockers. Lanes centralize call cleanup/access/diagnostic families. Actual frames, binding, by-ref args/returns, variadics, callbacks, dynamic dispatch, and return ownership remain mostly non-executable. |
| Comparison/conversion semantics | 40% | 57% | Primary has comparison ABI consumers, runtime comparison operation/value-family sharing, centralized outcomes, arithmetic conversion sharing, report/free/exit sinks, byte materialization, numeric-string reuse, and string truthiness. Broader conversion-source and pair-conversion work is still candidate material. |
| Arrays, lvalues, references, COW | 16% | 53% | Primary now consumes runtime array-key materialization for keyed writes, keyed assignments, and indexed echo reads. Lanes have stronger owner-slot, value-root, RMW, reference-cell, path-preflight, and generated-C value-result candidates. Primary still lacks full executable array lvalues, foreach, nested writes, references/COW, ArrayAccess, and exact warnings. |
| Symbols, globals, request state | 21% | 43% | Primary has symbol-table ABI helpers. Lanes have expression-result consumer classification, root write value-flow contracts, request-state read planners, and immutable snapshot consumers. Mutable request/global/superglobal behavior remains early. |
| Objects, properties, methods | 10% | 36% | Lane-local object/property receiver, class-policy, metadata, and stateful operation blockers are more coherent, but primary still has little broad executable object/property/method behavior. |
| Diagnostics and control-flow cleanup | 17% | 42% | Severity tags and selected blockers are integrated. Lanes are active on request diagnostics, diagnostic-result producers/sinks, structured CFG, branch/termination cleanup, and exit handoff. Exact warning/recovery order and executable cleanup remain broad blockers. |
| Broad composition verification | 13% | 30% | Recent primary batches have focused runtime/source/linked executable gates. Broad differential PHP composition coverage is still thin. |

## Recent Primary-Integrated Work

- `438ca546 native: route string distance through runtime`
  - Adds a runtime string-distance ABI and generated-C consumers for lowerable `levenshtein()` and two-argument `similar_text()` over the shared value-to-string byte boundary, with runtime, source-generation, and linked executable coverage.
- `65f74970 native: materialize array keys in generated C`
  - Adds runtime array-key materialization and generated-C consumers for keyed array inserts, keyed assignments, indexed echo reads, diagnostics, and cleanup across scalar/string/null/bool/integral-float key families.
- `d2411a58 runtime: centralize comparison outcomes`
  - Shares comparison evaluation outcomes before native result or branch conversion.
- `d0e6ab37 native: consume comparison results through exit sink`
  - Routes generated-C comparison lowering over the owned runtime comparison result ABI and shared report/free/exit-code sink.
- `851d540f native: route string-int builtins through conversion`
  - Routes generated-C `ord()` and `crc32()` through native value materialization plus runtime value-to-string integer conversion.
- Earlier integrated batches
  - Route generated-C `strlen()`, string predicates, comparison byte operands, branch predicates, dynamic string lengths, string truthiness, arithmetic conversion, and call-result blockers through shared runtime/compiler boundaries.

## Candidate Work Not Yet Counted

- Lane-local array candidates: generated-C array key/value expression materialization through value-result ABI, direct array-handle path preflight, owner-slot/value-root/RMW/reference-cell contracts, foreach/reference blockers, and array lvalue operation contracts.
- Lane-local string candidates: broader binary-safe string result/debug/regex/hash/serialization/stream/configuration/HTML/entity/`chr()`/`ucwords()` surfaces, plus interpreter/runtime PHP string-byte boundaries.
- Lane-local symbol/request/call candidates: expression-result consumer classification, request-state read access, root write value-flow, slot contract plans, and call operation diagnostic families.
- Lane-local object/control/diagnostic candidates: object/property receiver/class-policy blockers, structured CFG/effect rows, termination cleanup stack scans, request diagnostics, and diagnostic result carriers.

Lane-local work is useful source material, but it is not product capability until selected into a small primary batch, gated, committed, and pushed.

## Candid Assessment

The direction is right. Recent primary commits are turning shared ABIs into generated-C consumers, especially comparison, string conversion, and array keys. That is real progress toward generalized semantics.

The product is still boundary-heavy. Many candidate changes make unsupported PHP fail through better semantic surfaces rather than execute more PHP correctly. The next highest-value work is a small executable consumer of an already-landed ABI surface, with cleanup and linked coverage, not more standalone vocabulary.

## Near-Term Steering

1. Count `438ca546` as the latest integrated semantic baseline once pushed.
2. Prefer executable generated-C/LLVM consumers of existing ABI surfaces over pure blocker or diagnostic vocabulary.
3. Keep string-distance follow-up focused on generalized missing semantics: `similar_text()` percent output, exact warning parity, non-UTF-8 storage, ownership, cleanup, and LLVM parity.
4. Consider the lane-local array key/value value-result consumer if it can land as a narrow primary batch without importing broad array-lvalue churn.
5. Keep `/dev/shm` above the 10-12 GiB free warning band before primary gates; current review saw only 7.8 GiB free.
6. Refresh the supervisor dashboard after primary movement so steering is not based on stale `master` and dirty-file data.
