# PHP Native Compiler Progress

Updated: 2026-05-28 01:29 CEST
Latest source head: `0fa7b666 runtime: autoload enum_exists misses`
Evaluation marker: `20260526T040843Z`

Progress counts only generalized, tested, committed primary source work.
Dirty WIP, lane-local claims, docs-only substitutions, tests-only GO,
exact-shape production lowering, and broad gates without focused proof do not
move the bars.

## Snapshot

| Area | Integrated | Current read |
| --- | ---: | --- |
| Overall roadmap | **93%** `[##################--]` | Generated C covers many selected calls, includes, classes, refs, traits, magic access, buffers, destructors, shutdown callbacks, and cleanup transfers. |
| Executable PHP semantics | **99%** `[###################-]` | Latest source work routes interpreter `enum_exists()` misses through SPL autoload callbacks. |
| Runtime/ABI foundations | **91%** `[##################--]` | Remaining pressure is arbitrary alias transfer, broader autoload/import fallback, malformed magic signatures, cleanup parity, and broader lookup parity. |
| Compiler/backend consumers | **85%** `[#################---]` | Generated C is primary; LLVM has selected parity; direct assembly lags newer ABIs. |
| Arrays/refs/COW | **86%** `[#################---]` | Direct reference-slot `foreach` owners are integrated; arbitrary alias roots, broader owners, and backend parity remain. |
| Classes/traits/objects | **91%** `[##################--]` | Selected metadata, traits, typed props, aliases, enum autoload lookup, visibility, and magic reads/writes are integrated. |
| Cleanup/lifecycle | **41%** `[########------------]` | Selected output-buffer, destructor, shutdown, include/require, comparison-abort, and bounded throw cleanup exist; real exceptions remain. |
| Backend parity | **45%** `[#########-----------]` | Active LLVM watch: output buffers, property metadata, callable membership; reference write-through waits on LLVM/ASM tooling. |

## Recent Source Ledger

| Commit | Capability | Proof anchor |
| --- | --- | --- |
| `0fa7b666` | Interpreter `enum_exists()` now uses the existing SPL autoload callback/recheck path for enum misses while keeping generated-native enum metadata blocked. | `state/logs/phpc-primary-enum-autoload-a5abdbb5-20260528.gates.log`, sha256 `3554aa11ec5e148a73f3e8fd73d5ce6ef5f113e926d8e65f99ce72698b91707f`. |
| `2ef16e0d` | Request-scope `throw` inside active generated-C `finally` replays `finally` before the current unsupported-throw fatal boundary. | `state/logs/phpc-primary-throw-finally-fd52417e-20260528.gates.log`, sha256 `766c5ff4c79567a7874fa34f171ab049a0ae8549abf9aedffd50b354c1ca074d`. |
| `9c49c29b` | Generic generated-C comparison aborts now use cleanup-aware native error exits instead of direct abort-code returns. | `state/logs/phpc-primary-comparison-abort-cleanup-4ed1624e-20260528.gates.log`, sha256 `fe7f708e74134428048c31a12b8e642070bea30228b576a28062c8ea6ee3db33`. |
| `d97a9fcf` | Dynamic runtime-registry missing required include paths run active generated-C `finally` before fatal diagnostics/exit. | `state/logs/phpc-primary-dynamic-include-finally-8a0a982f-20260528.gates.log`, sha256 `f2b7fc3cc6527e3b4b5b91488e5bd409fb568a07f71eba01416ca8e2b2861cd0`. |

Older committed source work is intentionally summarized by the bars. Use
`git log --oneline` and gate logs under
`/home/claude/supervised-php-compiler/state/logs/` for historical proof.

## Remaining Gaps

- Reference/COW identity: arbitrary alias-root writeback, broader `foreach`
  owners, reference-returning ArrayAccess, and backend parity.
- Object model: broader object/static-property shapes, magic/constructor
  breadth, return references, spread breadth, metadata diagnostics, and full
  SPL/autoload/class-alias/import fallback.
- Cleanup/control flow: real Throwable propagation, catch binding, broad
  `finally` transfer semantics, dynamic include/require fatal cleanup, exact
  destructor/shutdown/output-buffer ordering, and object lifetime cleanup.
- Diagnostics/backend parity: malformed magic signatures, source ordering,
  suppression/custom handlers, and generated-C/LLVM/ASM parity.

## Current Queue Read

- Do not requeue consumed shutdown, missing-required-include finally,
  dynamic-registry require/finally, reference-slot `foreach`, or
  include-return-finally variants. Do not requeue interpreter `enum_exists()`
  SPL-autoload lookup.
- Source watch: next current-head queue or scout source candidate that survives
  non-repeat guards.
- Backend watch: LLVM output-buffer, property metadata, and callable
  membership; LLVM references stay parked until LLVM/ASM tooling is available.
- Reject: docs-only, tests-only, empty patches, stale artifacts, recursive
  source recognizers, and broad tests without focused proof.

## Verification Policy

Keep this file compact: update bars only when behavior moves, add at most one
short row per source capability, cite one gate-log path and hash, and keep
detailed command transcripts in supervisor logs instead of embedding them here.
