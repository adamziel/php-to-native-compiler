# PHP Native Compiler Progress

Updated: 2026-05-22 00:21 CEST
Evaluation marker: 20260521T221248Z-plus-5ccefbcf

This is a high-level supervisor dashboard. Percentages are candid engineering estimates, not test-suite pass rates. Primary-integrated capability means committed on `master`; lane-local work is candidate material until selected, gated, committed, and pushed.

## Overall Status

Estimated progress toward a broadly usable generalized PHP native compiler: **31%**

```
Generalized runtime/ABI foundations      [################----] 79%
Compiler/backend consumers               [###############-----] 73%
Executable generalized PHP semantics     [########------------] 39%
Arrays, references, COW, lvalues         [#####---------------] 23%
Objects, properties, methods             [##------------------] 10%
Diagnostics/control-flow composition     [####----------------] 20%
Broad integrated verification            [######--------------] 28%
```

## Current Primary State

- Product HEAD after this progress update: `cd300c58 docs: update progress after string raw buffer writes`, synced with `origin/master`.
- Latest committed semantic baseline: `5ccefbcf runtime: share PHP string raw buffer writes`.
- Live worktree status before this progress update: clean and synced after the string raw-buffer write commit.
- Resource note: `/dev/shm` was tight at about 6.5G free, then the supervisor reclaimed inactive target directories after active cargo/rustc/linker checks and restored it to about 21G free. Broad primary gates should still remain isolated, single-job, and resource-aware because lane caches will rebuild quickly.

## Grand Roadmap Position

The compiler is steadily replacing backend-local decisions and local byte/value handling with reusable runtime/ABI contracts and selected executable consumers. Recent primary progress is strongest in comparison routing, string-handle comparison operands, nested comparison decision rematerialization, numeric-string classifier sharing, value-result materialization, scalar value-cast ABI consumers, array-handle value operands, selected filesystem/cache ABI routing, call-boundary diagnostics, shared runtime string-byte source materialization, and shared raw-buffer writes for PHP string sources.

The product is still far from full generalized PHP. The largest missing regions remain references/COW, executable lvalues, user calls and frames, object/class/property semantics, mutable globals/superglobals, include/require, exceptions/finally, exact diagnostics, and cleanup across real control flow.

## Done / In Progress / Not Done

- [x] Supervised parallel lanes and primary integration gate established.
- [x] Shared runtime ABI surfaces for strings, raw byte-buffer writes, comparisons, numeric-string classification, array keys/value operations, request-state snapshots, selected conversion helpers, selected filesystem/cache operation blockers, selected diagnostic carriers, branch-decision status, string-handle comparison operands, and runtime string-byte source materialization.
- [x] Generated-C consumers for selected string builtins, array key/value operations, array-handle value operands, array append diagnostics, string-handle comparison operands/results/decisions, nested comparison decision operands, strict array/object identity in selected array-search builtins, array-handle comparisons, comparison status guards, cast/type-name echo value results, scalar value-cast operations, selected realpath-cache filesystem operation blockers, and centralized call diagnostics.
- [x] Reusable array-entry snapshot ABI for future foreach/lvalue/reference consumers.
- [ ] In progress: replace shared blockers with executable semantics one family at a time.
- [ ] In progress: generalize value/result/source ownership across returns, call args, conditions, branch joins, stdout, discarded temporaries, comparisons, casts, string-byte materialization, and cleanup.
- [ ] In progress: array lvalue/RMW/reference/COW work. Primary has foundations, append diagnostics, comparison consumers, and array-handle value operands; lane-local candidates are much stronger than integrated capability.
- [ ] Not done: full symbol environment semantics for locals/imports/globals/superglobals, undefined slots, repeated calls, writes/unset, references, and request/global separation.
- [ ] Not done: function/method frames, argument binding, by-ref args/returns, variadics/spreads, callbacks, constructors, and dynamic dispatch.
- [ ] Not done: object/class/property semantics including visibility, magic hooks, stdClass, dynamic names, references/COW, and exact diagnostics.
- [ ] Not done: full control-flow cleanup, loops/switch/break/continue/goto/finally, exceptions, and broad differential composition coverage.

## Active Roadmap Estimates

| Active item | Primary-integrated | Lane-local candidate maturity | Current read |
| --- | ---: | ---: | --- |
| String conversion, truthiness, byte buffers | 53% | 80% | Primary has shared value string-form semantics, numeric-string classification, selected generated-C string builtin consumers, cast/type-name echo value-result consumers, scalar value-cast generated-C consumers, comparison byte materialization, a shared runtime string-byte source/materialization boundary consumed by native strings, diagnostics, array keys, array-column keys, and PHP string values, and shared raw-buffer writes now used by scalar echo. |
| Call operation cleanup and ownership | 42% | 65% | Primary routes many call-result contexts, termination-construct argument expressions, direct special-form argument/arity failures, and shared backend call diagnostics through common call-boundary contracts. Lanes add call operation-source, binding, sequence, preflight, and result-blocker contracts, but real frames, binding, by-ref args/returns, dynamic calls, and return execution remain mostly non-executable. |
| Comparison and conversion semantics | 61% | 74% | Primary has reusable comparison operation validation, branch/free/decision/status ABIs, generated-C status guards, string-handle comparison operands, shared numeric-string pair classification, materialized value comparison entry points, comparison decision rematerialization for nested generated-C operands, array-handle comparison consumers, compare/cast/type-name value-result consumers, scalar value-cast ABI consumers, array-handle value operands for casts/type names, and operand-side/value-family blockers. Lane-local work adds relation-result operand and opaque-handle ABI candidates. Loose array/object/resource/reference execution, warning order, arbitrary expression materialization, and broader LLVM/generated-C parity remain open. |
| Arrays, lvalues, references, COW | 23% | 75% | Primary has array-key materialization, array value-operation result ABI, array-entry snapshots, array-handle comparisons, diagnostic array append consumers, and a generated-C/runtime boundary for materializing native array handles as owned value operands. Lanes have stronger owner-root, foreach, assignment-statement/value-result, RMW, `??=`, reference-operation, shared path-builder, and generated-C lvalue candidates; full executable lvalues/references/COW are not integrated. |
| Symbols, globals, request state | 24% | 56% | Primary has symbol ABI helpers, request/superglobal snapshot ABI, and `defined()` interpolation routed through expression-result boundaries. Lanes add expression-result consumer consolidation, slot transition contracts, scalar linked symbol-table execution, and request-state blocker/presence consolidation. Mutable globals/superglobals and repeated-call state remain early. |
| Objects, properties, methods | 10% | 46% | Lane-local object/class/property blockers and operation plans continue improving. Primary still has little broad executable object/property/method behavior. |
| Diagnostics and control-flow cleanup | 20% | 61% | Primary has selected severity/blocker surfaces, diagnostic array append behavior, centralized call diagnostic subjects, call-boundary cleanup routing, and comparison status/exit handling. Lanes have richer diagnostic-result carriers, sinks, termination handoffs, and CFG/control-flow rows; most control-flow rows are still non-emitting. |
| Filesystem/path builtins and request state | 18% | 43% | Primary centralizes filesystem/path/request blockers and snapshots, and routes `realpath_cache_get()`/`realpath_cache_size()` through the shared filesystem path/cache operation ABI in generated-C/runtime paths while preserving legacy LLVM blockers for unrelated path predicates. Real stream/stat/cache/current-directory/request mutation behavior is not implemented. |
| Broad composition verification | 28% | 40% | Focused runtime/native-link gates cover comparison decision/status paths, nested comparison decision operand composition, value-result consumers, scalar value-cast consumers, array-handle value operands/comparisons, array append diagnostics, realpath-cache filesystem blockers, call diagnostics, string-byte source materialization, scalar echo raw-buffer writes, and selected full `native_link` batches. Broad differential PHP composition coverage remains thin, and broad `phpc --tests` still has known pre-existing gaps. |

## Recent Primary-Integrated Work

- `5ccefbcf runtime: share PHP string raw buffer writes`
  - Extends the `PhpStringByteSource` / `PhpStringByteView` runtime boundary with raw-buffer write helpers and routes `phpc_native_scalar_echo_write(...)` through that shared path instead of copying bytes locally. Gates included focused string-byte source materialization, native scalar echo, native string-handle, native array-key materialization, `cargo check -q -p phpc`, formatting, cached diff checks, and push verification. This removes another local byte-copy path, but it still does not implement broad generated-C/LLVM string materialization, string offset semantics, binary-string storage, references/COW, or warning/recovery parity.
- `b214ccc5 runtime: share PHP string byte source materialization`
  - Adds a shared `PhpStringByteView` / `PhpStringByteSource` runtime boundary consumed by native strings, diagnostics, array keys, array-column keys, and PHP string values. Existing ABI consumers for string clone bytes, diagnostic clone/stderr bytes, array-key materialization, and array-key string cloning now share that source boundary instead of cloning bytes locally. Gates included focused shared string-byte source coverage, native string-handle tests, native array-key materialization tests, `cargo check -q -p phpc`, `cargo fmt -p php_runtime --check`, and `git diff --check`. This improves runtime/ABI consistency; it does not yet provide arbitrary generated-C/LLVM string materialization, binary-string/non-UTF-8 value storage, text-only admission, object/resource/Stringable conversion, references/COW, or exact warning/recovery ordering.
- `41f88885 native: route realpath cache through filesystem ABI`
  - Adds `realpath_cache_get()` and `realpath_cache_size()` to the shared filesystem path/cache operation enum, routes generated-C/runtime cache introspection through the shared blocker/diagnostic path, and keeps older LLVM per-builtin filesystem rejections intact except for the new realpath-cache pair. Gates included runtime filesystem blocker coverage, generated-C source and linked executable checks, the focused LLVM boundary test, the realpath builtin suite, neighboring filesystem builtin suites, package check, rustfmt, and diff checks. This is still blocker/routing infrastructure; it does not implement real filesystem stat/cache/current-directory semantics, stream wrappers, warning-plus-false recovery, path byte policy, or open_basedir/include-path behavior.
- `61c21cf6 codegen: rematerialize comparison decisions as operands`
  - Adds a shared comparison-branch-decision-to-operand ABI and generated-C carrier so nested comparison results can feed later comparison, condition, echo, truthiness, and type-name consumers through runtime comparison semantics instead of collapsing to a backend-local bool string. Runtime and native-link gates include source inspection, linked executable composition, broader comparison coverage, package check, rustfmt, and diff checks. This improves executable comparison composition, but loose array/object/resource/reference comparison execution, exact warning/recovery order, arbitrary expression materialization, and LLVM parity remain open.
- `d02b1e76 codegen: materialize native arrays as value operands`
  - Adds `phpc_native_value_from_array(...)` and routes generated-C native array handles through the shared value operand boundary, so arrays can participate in casts, `boolval()`, `gettype()`, and nested generated-C value flows without falling back to an assembly-array rejection. Runtime and full native-link gates passed. This is a real executable consumer, but it still does not implement array lvalues, references/COW, foreach/by-ref behavior, ArrayAccess/object/resource offsets, spread semantics, or arbitrary native value propagation.
- `d3287ff6 codegen: consume scalar value cast ABI`
  - Routes scalar cast operations through the shared native value-cast ABI across runtime, generated-C lowering, native link coverage, ABI probes, scalar-cast tests, and cast boundary fixtures. It does not complete arbitrary expression materialization, object/resource/Stringable conversion hooks, reference/COW behavior, full warning/recovery ordering, or broad LLVM parity.
- `325c9c7d codegen: centralize native call diagnostics`
  - Adds shared `NativeCallDiagnostics` and `NativeCallDiagnosticSubject` handling for existing native call diagnostics across direct, dynamic, method, constructor, closure, function-frame handoff, return-value ownership, and explicit call-operation blockers. This removes duplicated backend-local diagnostic construction; it does not add callable lookup, real frames, argument binding, by-ref/variadic ownership, closure/object dispatch, or return execution.

## Current Lane-Local Candidate Work Not Yet Counted

- Array/reference candidates: shared generated-C owner-root builder, foreach iterable setup, assignment-statement update tags, operation-tagged array lvalue value/reference update boundaries, keyed RMW operation boundaries, null-coalescing assignment paths, owner-cell descriptors, reference-cell transfers, shared writable path builders, and generated-C lvalue consumers.
- String/conversion candidates: formatter-tag byte ABI, byte-source/view/result boundaries, raw-buffer sinks, tokenizer/parser/string-result execution, interpreter byte-output sinks, scalar-cast builtin routing, string offset warning blockers, and broader binary-safe string surfaces.
- Symbol/request/call candidates: expression-result sequence consumers, root slot transition contracts, scalar linked symbol-table execution, callable operation/preflight/source contracts, request/global/superglobal operation boundaries, user-function/termination blockers, and call cleanup/diagnostic families.
- Object/control/diagnostic candidates: object-property operation plans, class-policy/object receiver blockers, diagnostic result carriers and sinks, termination handoffs, and non-emitting CFG/control-flow readiness rows.

Lane-local work is useful source material, but it is not product capability until selected into a small primary batch, gated, committed, and pushed.

## Candid Assessment

Recent primary work is directionally sound because it turns shared ABI surfaces into generated-C or LLVM-visible consumers or consolidates runtime materialization paths used by multiple ABI consumers. The strongest pattern remains small and concrete: remove backend-local handling, route through a shared runtime semantic contract, and prove executable or ABI behavior.

The project remains boundary-heavy. More result/blocker vocabulary without immediate executable consumers will not move the product much. The next valuable batches should each answer: what PHP behavior now runs that did not run before, what shared runtime materialization path now has real consumers, or what backend-local bypass has been removed?

## Near-Term Steering

1. Treat `5ccefbcf` as the latest integrated semantic baseline.
2. Prefer small executable generated-C/LLVM consumers of existing ABI surfaces over more standalone vocabulary.
3. Strong next candidates: isolate one narrow array-lvalue consumer from `impl-array-linked-exec`; consume the lane-local comparison relation-result operand ABI in primary; add LLVM/generated-C parity for comparison or array append diagnostic boundaries; or route another existing string/conversion materialization boundary through a real backend path.
4. Avoid whole-lane merges. Several lanes contain broad, conflict-prone, or non-executable contract work.
5. Be skeptical of scalar/local linked-symbol helper paths unless they connect to generalized frame/symbol environment semantics.
6. Keep resource checks explicit before broad gates; `/dev/shm` is just above the dispatcher floor at this review, and isolated target directories remain preferred to avoid target-dir races.
