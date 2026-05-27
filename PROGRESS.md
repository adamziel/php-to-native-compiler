# PHP Native Compiler Progress

Updated: 2026-05-28 00:59 CEST
Latest source head: `3ff60469 native: run finally before include return`
Evaluation marker: `20260526T040843Z`

Progress counts only generalized, tested, committed primary source work.
Dirty WIP, lane-local claims, docs-only substitutions, tests-only GO,
exact-shape production lowering, and broad gates without focused proof do not
move the bars.

## Snapshot

| Area | Integrated | Current read |
| --- | ---: | --- |
| Overall roadmap | **93%** `[##################--]` | Generated-C semantics are strong across selected calls, includes, classes, refs, properties, traits, magic access, output buffers, destructors, shutdown callbacks, and terminal cleanup. |
| Selected executable PHP semantics | **99%** `[###################-]` | Latest source work runs active generated-C `finally` bodies before compile-time-known missing `require` / `require_once` fatal diagnostics and `255` exits. |
| Runtime and ABI foundations | **90%** `[##################--]` | Remaining gaps are arbitrary alias transfer, full autoload/import fallback, malformed magic signatures, cleanup parity, and broader lookup parity. |
| Compiler/backend consumers | **85%** `[#################---]` | Generated C is the primary executable consumer; LLVM has selected parity; direct assembly lags newer ABIs. |
| Arrays, lvalues, refs, COW | **86%** `[#################---]` | Selected owner/reference paths include direct reference-slot foreach owners; arbitrary alias roots, broader foreach owners, and backend parity remain. |
| Object model/classes/traits | **90%** `[##################--]` | Metadata, traits, typed properties, aliases, autoload-selected lookup, visibility, and magic reads/writes are selected-path integrated. |
| Cleanup/unwind/lifecycle | **36%** `[#######-------------]` | Output-buffer shutdown, fatal destructor finalization, shutdown callbacks, missing-require active-finally replay, and include-unit return-through-finally exist for selected paths. |
| Backend parity | **45%** `[#########-----------]` | Queued LLVM parity targets include references, output buffers, property metadata, include/require blockers, and callable membership. |

## Recent Source Ledger

| Commit | Capability | Proof anchor |
| --- | --- | --- |
| `3ff60469` | Included units that `return` through an active generated-C `finally` now replay the `finally` body before handing the include value back to the caller. | `state/logs/phpc-primary-exception-finally-67bfce5a-20260528.gates.log`, sha256 `3e3c6ea2603b712ef45c9f0111f1ca777f138f07924c811f6ccdb36d5e7616e1`. |
| `520d7b62` | Direct local reference-slot array owners now feed by-value and by-reference generated-C `foreach` iterable snapshots through shared native array-lvalue owner APIs without broad symbol-table owner routing. | `state/logs/phpc-primary-foreach-refslot-9ba0c62a-20260528.gates.log`, sha256 `8a1eaa139f79c134b19bde7fed4ae27f38993862945192d34f56329b880db6fd`. |
| `37dc8a85` | Missing required include paths run active `finally` before fatal diagnostics/exit. | `state/logs/phpc-primary-include-finally-missing-require-f9bcd680-20260528.gates.log`, sha256 `01d45a27884cbaa4b5a3efdffb5a85972f38a67997901f279003c23da95c2d2f`. |
| `bf8d2e0d` | Request cleanup runs native shutdown callbacks, including nested same-pass registration, before destructors/output buffers. | `state/logs/phpc-primary-shutdown-callbacks-858eeea3-20260528.gates.log`, sha256 `2cfd21340dba95bc789ddb02b3e26a629cbd0df355c6d0099e55c6a7b435a9f9`. |
| `3cb69e40` | Trait composition resolves unique unqualified multi-trait adaptations and rejects ambiguous targets. | `state/logs/phpc-primary-trait-precedence-f5d497c2-20260528.gates.log`, sha256 `e7db123153bf23c570c78b85686e6e2087c48d1365502156843faff8dd7f70b2`. |
| `2168719d` | `call_user_func_array()` preserves stored reference-backed array entries through materialized argument bridging. | `state/logs/phpc-primary-callable-array-byref-19ad2463-20260528.gates.log`, sha256 `7a2af3bfa0c53335d49feaf39694ed476980b1327b3feb9aee66d2ef0d37cf97`. |
| `b6b063c8` | Generated functions declared by include units preserve declaring source path for registry lookup inside function bodies. | `state/logs/phpc-primary-source-activation-ef20ccf-20260528.gates.log`, sha256 `e88eab351d11ed0ff666835527a3edd097050532fccf2e505415c4aa256a7e30`. |

Older committed source work is intentionally summarized by the bars. Use
`git log --oneline` and gate logs under
`/home/claude/supervised-php-compiler/state/logs/` for historical proof.

## Not Done

- Full reference/COW identity, arbitrary alias-root writeback, broader
  foreach iterable owners, and complete reference-returning ArrayAccess.
- Dynamic/broader object and static-property shapes, broader magic and
  constructor coverage, return references, spread breadth, and exact metadata
  diagnostics.
- Actual exception/Throwable propagation, catch matching/binding, broad
  `finally` transfer semantics, dynamic include/require fatal cleanup through
  active `finally`, exact destructor/shutdown/output-buffer ordering, and
  object lifetime cleanup.
- Full SPL autoload, class-alias parity, function/const import fallback,
  namespace fallback, malformed magic signature parity, exact diagnostics,
  source ordering, suppression/custom handlers, and generated-C/LLVM/ASM
  backend parity.

## Current Queue Read

- Consumed: shutdown callback cleanup (`bf8d2e0d`), missing-required-include
  active-finally replay (`37dc8a85`), and direct local reference-slot
  foreach owners (`520d7b62`), and include-unit return-through-finally
  (`3ff60469`). Do not requeue older variants.
- Highest current source/semantics watch: comparison-abort cleanup refresh,
  dynamic include/finally scoped terminal-arm work, and broader throw/catch
  `finally` transfer scouting.
- Highest backend watch: LLVM output-buffer unwind is a refreshed GO candidate;
  LLVM property metadata and callable membership have focused IR proof;
  LLVM reference write-through is parked until real LLVM/ASM tooling is
  available.
- Reject or hold: docs-only, tests-only, empty patches, stale consumed
  artifacts, recursive require/source-shape recognizers, and broad tests
  without focused proof.

## Verification Policy

Keep this file compact: update bars only when behavior moves, add at most one
short row per source capability, cite one gate-log path and hash, and keep
detailed command transcripts in supervisor logs instead of embedding them here.
