# PHP Native Compiler Progress

Updated: 2026-05-28 00:34 CEST
Evaluation marker: `20260526T040843Z`
Strategy evaluator marker: `20260526T040843Z`

Accounting rule: progress counts only generalized, tested, committed primary
source work. Dirty WIP, lane-local claims, candidate artifacts, docs-only
substitutions, tests-only GO, exact-shape production lowering, and broad gates
without focused proof do not increase capability bars.

Progress bars use 20 slots. One `#` is roughly 5%; percentages are coarse and
do not move for scaffolding unless committed behavior changes the roadmap.

## Executive Read

Overall integrated-roadmap progress: **93%** `[##################--]`

Selected executable PHP semantics: **99%** `[###################-]`

Latest accounted source capability: `bf8d2e0d` runs generated-C
`register_shutdown_function()` callbacks during request cleanup, including
nested callbacks registered during shutdown, and orders them before retained
native value/reference cleanup, request destructor finalizers, and output-buffer
unwind.

Current blockers remain concentrated in broader array COW/reference edges,
remaining foreach/reference owner breadth, dynamic include/require cleanup
through `finally`, exception/throw/catch semantics, exact cleanup ordering,
non-C backend parity, and exact PHP diagnostic/source-order behavior.

## Recent Primary Source Ledger

| Commit | Compact capability | Focused proof anchor |
| --- | --- | --- |
| `bf8d2e0d` | Generated-C request cleanup now runs native shutdown callbacks registered through `register_shutdown_function()`, preserves nested callback registration during the same shutdown pass, prevents repeated runs, and orders callbacks before destructors/output buffers. | Gate log `state/logs/phpc-primary-shutdown-callbacks-858eeea3-20260528.gates.log`, sha256 `2cfd21340dba95bc789ddb02b3e26a629cbd0df355c6d0099e55c6a7b435a9f9`. |
| `3cb69e40` | Trait composition resolves unqualified multi-trait alias/visibility adaptations when a method has a unique declaring trait and rejects ambiguous adaptation targets. | Gate log `state/logs/phpc-primary-trait-precedence-f5d497c2-20260528.gates.log`, sha256 `e7db123153bf23c570c78b85686e6e2087c48d1365502156843faff8dd7f70b2`. |
| `2168719d` | Generated-C `call_user_func_array()` preserves stored reference-backed array entries through the shared materialized argument bridge while value-backed by-reference copies still diagnose. | Gate log `state/logs/phpc-primary-callable-array-byref-19ad2463-20260528.gates.log`, sha256 `7a2af3bfa0c53335d49feaf39694ed476980b1327b3feb9aee66d2ef0d37cf97`. |
| `b6b063c8` | Generated user functions declared in include units carry their declaring source path into generated-C function bodies, preserving include-registry source-dir lookup inside those functions. | Gate log `state/logs/phpc-primary-source-activation-ef20ccf-20260528.gates.log`, sha256 `e88eab351d11ed0ff666835527a3edd097050532fccf2e505415c4aa256a7e30`. |
| `217701f5` | Dynamic object-property reads normalize runtime property-name values and route valid `__get()` methods through shared magic-aware, caller-context-aware runtime property helpers. | Gate log `state/logs/phpc-primary-magic-get-3799361f-20260528.gates.log`, sha256 `0954af9b8e94da4c5797a0c2adab8b98ba8dcc7d7abc028a117e154d84a5dea0`. |
| `9c2aa623` | Generated-C `class_alias()` accepts runtime/frame value operands and autoload-backed alias materialization through shared class-alias value helpers. | Gate log `state/logs/phpc-primary-class-alias-f2c3a096-20260528.gates.log`, sha256 `107c31e622fa6eee73ed9f30f78202352126a0cc607cae570cfe5ce43c280fe8`. |
| `df87dbb2` | Declared instance properties accept class-like, object, nullable, union, and intersection type declarations through shared allocation/property metadata and method-frame write diagnostics. | Gate log `state/logs/phpc-primary-typed-instance-69da7335-20260528.gates.log`, sha256 `dc3e4a8fe44aa04c78030c379d6203b8f72ff00419f313689b73a1e92f9486d6`. |
| `b245b45f` | Direct by-reference `foreach` preserves the non-empty loop-variable alias to the last iterated array slot and keeps empty-loop prior-value behavior. | Gate log `state/logs/phpc-primary-foreach-alias-8acb08f6-20260527.gates.log`, sha256 `ef9d80d906f5ce0f688ba54667a595b6abc014aaedfb0c9fa6309686a443f3d5`. |
| `8231006f` | Method-frame object-property reads/writes/inspection carry caller class context into the runtime property ABI, preserving private/protected visibility for declared properties. | Gate log `state/logs/phpc-primary-visibility-context-5c0ba5af-20260527.gates.log`, sha256 `35eac2413cf3379d7db9063fbfa986df007e803cc007f4e740b08d4b7796ba71`. |
| `e95e5101` | Dynamic-name object-property writes route through the shared magic-aware mutation boundary for valid `__set()` methods. | Gate log `state/logs/phpc-primary-magic-set-31da596f-20260527.gates.log`, sha256 `f03a9a7dc46f73134b3eaff7c568d8206b088079c6de3a353bcb69c5253449e0`. |

Older committed source work is intentionally summarized by the roadmap bars
below. Use `git log --oneline` plus the referenced supervisor gate logs under
`/home/claude/supervised-php-compiler/state/logs/` for exact historical proof.

## Roadmap Bars

| Workstream | Integrated | Bar | Current read |
| --- | ---: | --- | --- |
| Runtime and ABI foundations | **90%** | `[##################--]` | Strong selected-path value, byte-string, array, reference, symbol, callable, call-frame/result, diagnostic-result, terminal-kind, request-state, lvalue, ArrayAccess, class metadata, static-property storage, typed instance-property, magic access, class alias, autoload-policy, destructor finalizer, output-buffer, and shutdown-callback surfaces. Remaining gaps include arbitrary alias transfer, full autoload, namespace fallback, malformed magic signatures, broader closure/call handoff, cleanup/unwind parity, and broader lookup parity. |
| Compiler/backend consumers | **85%** | `[#################---]` | Generated C has the freshest executable consumers for calls, callable facts, class/object metadata, namespace/import policy, selected static/object properties, typed properties, include units, reference owners, foreach aliasing, magic property reads/writes, class aliases, cleanup reports, output buffers, destructors, and shutdown callbacks. LLVM has selected parity for class metadata and diagnostic/output bridges; direct assembly still lags most newer ABIs. |
| Executable PHP semantics | **85%** | `[#################---]` | Many executable islands exist across generated-C calls, methods/statics, constructors, includes, typed/static/object properties, ArrayAccess, references, class metadata, traits, magic properties, output buffers, destructors, and shutdown callbacks. Broad assignment/RMW/writeback, arbitrary references/COW, dynamic/static property breadth, exact exception/finally/destructor/shutdown ordering, exact diagnostics, broader imports/fallbacks, and backend parity remain open. |
| Strings and byte semantics | **60%** | `[############--------]` | Byte-backed values and selected byte-preserving string-array slots are integrated. Binary source bytes, byte-exact interpreter/session/debug output, `mb_str_split()`, request/global byte keys, and exact diagnostics remain open. |
| Arrays, lvalues, references, COW | **85%** | `[#################---]` | Selected lvalue/reference-source extraction, ReferenceSlot owner facts, object-property owners, request-superglobal/reference call bridges, direct array assignment COW, by-reference foreach aliasing, and selected ArrayAccess/RMW/writeback paths are integrated. Arbitrary alias roots, broader foreach/reference owners, full reference-returning ArrayAccess breadth, closure callback fact transport, and backend parity remain open. |
| Object model, classes, traits | **90%** | `[##################--]` | Generated-C user-class metadata, trait composition metadata, trait alias/static/visibility handling, unqualified adaptation resolution, instance/static typed property metadata, class aliases, autoload-selected constructor lookup, visibility context, and selected magic property access are integrated. Remaining gaps include full trait execution parity, interfaces, broader constructor/magic coverage, full autoload, dynamic receiver breadth, and exact metadata diagnostics. |
| Cleanup, unwind, lifecycle | **35%** | `[#######-------------]` | Cleanup result carriers, cleanup frames, terminal cleanup transfer, output-buffer unwind, fatal destructor finalization, and generated-C shutdown callbacks are integrated for selected paths. Actual exceptions/Throwable, catch/finally propagation, dynamic include fatal cleanup, exact destructor/shutdown/output-buffer ordering, object lifetime cleanup, and backend cleanup parity remain open. |

## Active Roadmap Items

| Item | Primary Integrated | Toward full feature | Status |
| --- | ---: | ---: | --- |
| Diagnostic-result carrier stack | **100%** `[####################]` | **60%** `[############--------]` | Runtime/result contracts, family consumers, cleanup report bridges, cleanup-frame producers, and terminal cleanup transfer are integrated; semantic production from throw/exit/default-return/control-flow/lvalue/reference families remains. |
| Callable access and class metadata | **100%** `[####################]` | **65%** `[#############-------]` | Shared lookup/invoke, source-call carriers, selected method/static/default/variadic frames, class metadata, aliases, autoload-policy, magic read/write, and typed properties are integrated for selected paths; broader runtime callables, full fallback, spread, and visibility parity remain. |
| ArrayAccess and ReferenceSlot owners | **100%** `[####################]` | **65%** `[#############-------]` | Direct/generated-object ArrayAccess, selected nested/property-held owners, object/static/request reference bridges, direct array COW, and by-reference foreach aliasing are integrated; arbitrary aliases, reference-return breadth, and broader owner families remain. |
| Cleanup/unwind execution | **35%** `[#######-------------]` | **35%** `[#######-------------]` | Output-buffer shutdown, fatal destructor finalization, and shutdown callbacks now execute on selected generated-C cleanup paths; exception/finally semantics, dynamic include fatal cleanup, exact lifecycle ordering, and non-C backend parity remain. |
| Backend parity | **45%** `[#########-----------]` | **45%** `[#########-----------]` | LLVM/ASM have selected class metadata, diagnostic-result, output operand, and cleanup bridge support. Current queued parity targets include LLVM reference write-through, LLVM output-buffer unwind, LLVM property metadata, LLVM include/require blockers, and LLVM callable reference membership. |

## Not Done

- Full reference/COW identity and arbitrary alias-root writeback.
- Dynamic ArrayAccess producers beyond known generated declared-class objects and selected generated-callable summaries.
- Broader foreach iterable owners, especially reference-slot/local array combinations beyond the integrated direct alias case.
- Dynamic and broader object/static-property shapes, broader static-property reference/`??=`/unset/isset/empty lowering, and full method/object execution outside selected generated-C islands.
- Actual exception/Throwable propagation, catch matching/binding, `finally` transfer semantics, dynamic include/require fatal cleanup through active `finally`, exact destructor/shutdown/output-buffer ordering, and object lifetime cleanup.
- Full SPL autoload, broader class-alias parity, function/const import discovery/fallback, namespace/function/const fallback, malformed magic signature parity, broader magic-call coverage, constructor allocation/execution breadth, spread arguments, unsupported named-call families, and return references.
- Remaining semantic diagnostic-result operand migration for throw/exit/default-return terminals, cleanup production from real control flow, lvalue/reference/RMW/call-argument families, exact PHP diagnostics, source ordering, suppression/custom handlers, and backend parity across generated C, LLVM, and direct assembly.

## Current Queue Read

- `shutdown-r52` was integrated as `bf8d2e0d`; do not requeue older shutdown callback patches.
- `exception-cleanup-r34` remains a focused comparison-abort cleanup candidate, but it must refresh around the shutdown integration because the old queue matrix reported conflicts with shutdown tests.
- LLVM parity candidates that exact-applied at `858eeea3`: output-buffer unwind, property metadata, reference write-through, include/require blockers, and callable reference membership. Recheck after `bf8d2e0d` before integration.
- Include/finally scout selected a narrow future lane: compile-time known missing `require` / `require_once` under active `finally`, emitted through `emit_missing_include_result` without recursive source-shape recognizers or branch-local finalizer insertion.
- Reject or hold: docs-only, tests-only, empty patches, stale source-activation/callable-array/trait artifacts already consumed, `exception-cleanup-r41` broad recursive require recognizers, and exact-shape production lowering.

## Verification Policy

For future commits, keep this file compact: update bars only when behavior moves,
add one short source ledger row per source capability, include one gate-log path
and hash, and keep detailed command transcripts in supervisor logs instead of
embedding them here.
