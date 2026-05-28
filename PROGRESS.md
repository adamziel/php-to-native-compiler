# PHP Native Compiler Progress

Updated: 2026-05-28 04:56 CEST
Primary branch: `master`
Latest source head: `0fa7b666 runtime: autoload enum_exists misses`

## Progress Score

This file is the public progress report for the project. AO workers and the
supervisor must update this file before claiming public progress.

Progress is the pinned php-src PHPT full-suite pass rate:

`passed runnable PHPTs / total runnable PHPTs`

Current score: **1118 / 20294 runnable PHPTs = 5.51%**.

The first full-suite baseline was recorded for Batch 001 stack10 on php-src
`f97ff597429a2fe633665a7e02d97c8077f9f90f`, run
`phpt-full-batch001-20260528T010422Z-php-src-f97ff59-base-3e702be4-stack10`.
Counts: 1118 passed, 19156 failed, 964 skipped, 20 xfailed, 0 borked;
`run-tests.php` exited 1. Evidence lives under
`/home/claude/supervised-php-compiler/state/logs/phpt-full-batch001-20260528T010422Z-php-src-f97ff59-base-3e702be4-stack10`.

## PHPT Harness

| Item | State | Evidence |
| --- | --- | --- |
| php-src pin | Done | `/home/claude/php-src-phpt` at `f97ff597429a2fe633665a7e02d97c8077f9f90f` |
| Static inventory | Done | 21,827 PHPT files; 12,777 static runnable candidates |
| `phpc` PHPT wrapper | Done | `/home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper` |
| Skip/xfail ledger | Started | `/home/claude/supervised-php-compiler/state/php-core-suite-skip-ledger.tsv` |
| First full-suite baseline | Done | 1118 / 20294 runnable PHPTs passed (5.51%); run id `phpt-full-batch001-20260528T010422Z-php-src-f97ff59-base-3e702be4-stack10` |

Focused PHPT history is tracked separately in
`/home/claude/supervised-php-compiler/state/php-core-suite-focused-history.tsv`.
Focused passes prove candidate direction; they do not define project percent.

## Batch 001

Policy: stage 10 accepted generalized source PRs, run focused gates per PR, run
the full PHPT suite once after PR 10, repair regressions, then merge the whole
batch.

Current batch status: **10/10 accepted, not merged; full PHPT baseline recorded; regression/failure repair next**.
Independent reviewer `phpc-7` accepted r81 / PR #1 as Batch 001 PR 10
at 2026-05-28 02:59 CEST after accepted-stack apply, exact-shape audit,
focused Rust/compiler gates, and focused wrapper PHPT `Zend/tests/namespaces/ns_065.phpt`
passed (1/1).

Accepted for staging:

| # | Candidate | Main proof |
| ---: | --- | --- |
| 1 | Magic-method signature diagnostics | Current-head review, focused diagnostics gates |
| 2 | Symbol-table foreach owners | Current-head review, focused native-link gates |
| 3 | Exception/catch/finally propagation | Current-head review, focused exception gates |
| 4 | Generated-C return-reference sources | Current-head review, accepted-stack compatibility, focused reference-return gates |
| 5 | Closing-tag statement terminator | Focused `tests/basic/001.phpt`, invalid-syntax review, accepted-stack compatibility |
| 6 | Object lifecycle live roots | Caller-frame live-root review, accepted-stack compatibility, focused destructor gates |
| 7 | Grouped namespace class imports | Current-head review, accepted-stack compatibility, focused compiler gates, wrapper PHPT proof |
| 8 | By-reference foreach lingering slots | Accepted-stack review, focused `Zend/tests/foreach/foreach_reference.phpt`, slot-preserving array-copy gates |
| 9 | Magic method startup signature fatals | Accepted-stack review, focused `tests/classes/__call_002.phpt`, generalized magic contract gates |
| 10 | Multiple unbracketed namespace declarations | Independent accepted-stack review, focused `Zend/tests/namespaces/ns_065.phpt`, namespace parser/import gates |

Gate status and parked candidates:

| Item | State |
| --- | --- |
| Batch 001 full PHPT gate | Done in AO session `phpc-11`; first baseline recorded at 1118 / 20294 runnable PHPTs (5.51%) |
| Full-suite count guard | Done; `all-results.txt` used `PASSED/FAILED/SKIPPED/XFAILED`, the parser counted those statuses, and the verified row is in `state/php-core-suite-history.tsv` |
| PR #4 by-reference call expressions | Batch002 stack decision says PR #4 supersedes r82/PR #2; use PR #4 as the by-reference candidate because it covers `Zend/tests/bug39944.phpt` plus adjacent return/pass-by-reference PHPTs |
| PR #5 named by-reference arguments | GO-CANDIDATE after independent review on accepted stack10 + PR #4 + PR #5; focused Rust gates passed and wrapper PHPTs `Zend/tests/named_params/references.phpt`, `tests/lang/passByReference_007.phpt`, and `tests/lang/returnByReference.002.phpt` passed 3/3 |
| PR #6 foreach reference-backed `print_r()` | GO-CANDIDATE after independent review on accepted stack10; focused Rust/build gates passed and wrapper PHPT `tests/lang/foreach_with_references_001.phpt` plus foreach anchors passed after a generalized reference-backed array formatting fix |
| PR #7 magic `__call()` by-reference array args | GO-CANDIDATE after refreshed stack-safe independent review and p14 `SAFE-FOR-PROGRESS`; focused Rust/build/fixture gates passed and wrapper PHPTs `tests/classes/__call_003.phpt` plus `tests/classes/__call_001.phpt` passed 2/2; no full-suite run and no percent change |
| PR #7 follow-up: `__call_004` static-syntax fallback to current `__call()` | GO-CANDIDATE after independent review on accepted stack10 plus reviewed Batch002 through refreshed PR #7 and p14 `SAFE-FOR-PROGRESS`; focused Rust/build/fixture gates passed and wrapper PHPTs `tests/classes/__call_004.phpt`, `tests/classes/__call_003.phpt`, and `tests/classes/__call_001.phpt` passed 3/3; no full-suite run and no percent change |
| `Zend/tests/bug39944.phpt` reference invocation | PR #2/r82 is parked/superseded for Batch002; do not stack it with PR #4 because both conflict in `compiler/src/interpreter.rs` and `compiler/tests/functions_and_scopes.rs` |
| Magic visibility warnings | PR #3 is `REBASE-NEEDED` for Batch 002 after r81/stack10 due docs conflict; production/test hunks replay |
| Foreach `$GLOBALS` lane | PASS-NO-PATCH accepted by reviewer; accepted stack10 passes `foreach_unset_globals`, `foreach_reference`, and `foreach_temp_array_expr_with_refs` |
| Foreach object-property by-reference lane | GO-CANDIDATE after independent review; focused PHPT `Zend/tests/foreach/foreach_by_ref_to_property.phpt` plus foreach anchors passed 3/3, with PR #3/#4 stack compatibility checks |
| Anonymous-class dynamic-call blocker | AO scout classified this as NO-GO for Batch 001 PR 10; deferred as a broader parser/interpreter/native feature |
| PHPT focused queue | `tests/classes/__set__get_002.phpt` passes on the 9/10 stack; r85 queue now feeds additional coder lanes |
| Codex thread-store permissions | Fixed current session directory execute bit; smoke passed |
| Disk/data cleanup | Reclaimed Codex SQLite WAL; `/home` currently has 286G free |
| Agent Orchestrator migration | AO is installed, configured, polling this project, and persistent critic/reviewer/progress-reporter/coder roles are active |

## AO Control Plane

AO dashboard: `http://localhost:3000/projects/php-to-native-compiler`.

Required live roles:

| Role | Responsibility |
| --- | --- |
| Critic | Read-only audit for exact-shape lowering, shallow evidence, stale artifacts, and premature completion |
| Reviewer | Independent candidate apply/review/focused-gate proof before Batch 001 acceptance |
| Progress reporter | Keeps this `PROGRESS.md` file and durable supervisor state current after material AO events |
| Coders | Work disjoint focused PHPT lanes from the queue; each lane must produce a patch, PASS-NO-PATCH, or NO-GO artifact |

Current AO snapshot: `phpc-orchestrator` supervising; `phpc-14` critic;
`phpc-7` reviewer; `phpc-8` progress reporter; active coder/support lanes
`phpc-15`, `phpc-16`, `phpc-17`, `phpc-18`, and `phpc-19`. Current
public-progress watch targets are PR #13 /
`passByReference_012` review artifacts, PR #8 / `passByReference_002`
real-stack review artifacts, PR #15 / `returnByReference.003` review
artifacts, and any new full-suite PHPT row. Old idle session `phpc-2` was
killed after being rediscovered outside the active roster.

## Current Rules

- No exact-shape production lowering for individual PHPTs.
- No docs-only or tests-only progress.
- No full PHPT suite for every change.
- No batch merge before 10 accepted PRs, a full PHPT run, and regression repair.
- Legacy roadmap bars are retired; use PHPT pass rate as the only percent.

## Recent Source Anchors

| Commit | Capability | Gate log |
| --- | --- | --- |
| `0fa7b666` | Interpreter `enum_exists()` now uses SPL autoload callback/recheck for enum misses. | `state/logs/phpc-primary-enum-autoload-a5abdbb5-20260528.gates.log` |
| `2ef16e0d` | Request-scope `throw` inside active generated-C `finally` replays `finally` before the current unsupported-throw fatal boundary. | `state/logs/phpc-primary-throw-finally-fd52417e-20260528.gates.log` |
| `9c49c29b` | Generated-C comparison aborts now use cleanup-aware native error exits. | `state/logs/phpc-primary-comparison-abort-cleanup-4ed1624e-20260528.gates.log` |
| `d97a9fcf` | Dynamic runtime-registry missing required includes run active generated-C `finally` before fatal diagnostics. | `state/logs/phpc-primary-dynamic-include-finally-8a0a982f-20260528.gates.log` |

Detailed worker logs, PHPT inventory, batch review reports, and skip policy live
under `/home/claude/supervised-php-compiler/state/`.
