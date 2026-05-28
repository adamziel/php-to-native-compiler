# PHP Native Compiler Progress

Updated: 2026-05-28 03:04 CEST
Primary source head: `5197008b docs: record batch 001 full gate readiness`
Latest source head: `0fa7b666 runtime: autoload enum_exists misses`

## Progress Score

This file is the public progress report for the project. AO workers and the
supervisor must update this file before claiming public progress.

Progress is the pinned php-src PHPT full-suite pass rate:

`passed runnable PHPTs / total runnable PHPTs`

Current score: **unmeasured; 0% claimed**.

No baseline full-suite run has been recorded yet. The harness is bootstrapped,
but the project percent does not move until a full-suite result is written to
`/home/claude/supervised-php-compiler/state/php-core-suite-history.tsv`.

## PHPT Harness

| Item | State | Evidence |
| --- | --- | --- |
| php-src pin | Done | `/home/claude/php-src-phpt` at `f97ff597429a2fe633665a7e02d97c8077f9f90f` |
| Static inventory | Done | 21,827 PHPT files; 12,777 static runnable candidates |
| `phpc` PHPT wrapper | Done | `/home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper` |
| Skip/xfail ledger | Started | `/home/claude/supervised-php-compiler/state/php-core-suite-skip-ledger.tsv` |
| First full-suite baseline | Not run | Required before any percent claim |

Focused PHPT history is tracked separately in
`/home/claude/supervised-php-compiler/state/php-core-suite-focused-history.tsv`.
Focused passes prove candidate direction; they do not define project percent.

## Batch 001

Policy: stage 10 accepted generalized source PRs, run focused gates per PR, run
the full PHPT suite once after PR 10, repair regressions, then merge the whole
batch.

Current batch status: **10/10 accepted, not merged; full PHPT gate running**.
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
| Batch 001 full PHPT gate | Running in AO session `phpc-11` using the r86 clean `/tmp` gate plan; this is the single authorized full-suite run before merge |
| `Zend/tests/bug39944.phpt` reference invocation | PR #2/r82 has focused PASS and patch/report artifacts but is parked for post-Batch 001 / Batch 002 unless r81 is invalidated |
| Anonymous-class dynamic-call blocker | AO scout classified this as NO-GO for Batch 001 PR 10; deferred as a broader parser/interpreter/native feature |
| PHPT focused queue | `tests/classes/__set__get_002.phpt` passes on the 9/10 stack; r85 queue now feeds additional coder lanes |
| Codex thread-store permissions | Fixed current session directory execute bit; smoke passed |
| Disk/data cleanup | Reclaimed Codex SQLite WAL; `/home` free space is back above 290G |
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

Current AO snapshot: `phpc-orchestrator` supervising; `phpc-11` full-suite gate;
`phpc-4` critic audit;
`phpc-7` independent reviewer has accepted r81 as PR 10 and handed off;
`phpc-8` progress reporter; `phpc-9` magic-signature coder; `phpc-10`
by-reference coder. PR #2/r82 remains parked unless Batch 001 PR 10 selection
is reopened.

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
