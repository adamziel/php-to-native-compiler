# Blocked 221205Z Progress Refresh

Lane: 4, developer-96

Generated: 2026-06-05T00:10Z

Scope: control-plane and artifact refresh only. No compiler/runtime source
edits were made. No PHPT gate was run.

## Current Authoritative Metrics

| Metric | Value | Source |
| --- | ---: | --- |
| Accepted public score | `7873 / 20294 = 38.79%` | Accepted gate `public-comparable-score.tsv`; SQLite `metric_samples.id=1`; SQLite `goals.id=1` |
| Accepted source commit | `0b917f67a37d9ca9779d77f87173b628431c2425` | Accepted gate `current-score-gate-preflight.tsv`; root `PLAN.md`; SQLite `goals.id=1` |
| Blocked candidate score | `7197 / 20294 = 35.46%` | Candidate gate `public-comparable-score.tsv`; SQLite `metric_samples.id=2`; SQLite `goals.id=1` |
| Blocked candidate commit | `56fe9377fb46be00db5fdd30c966fdba406dc581` | Candidate gate `current-score-gate-preflight.tsv` |
| PASS regressions | `1166` | Candidate gate `pass-regression-summary.tsv` and `regressions-from-latest-published-passes.txt` |

Accounting caveat: the public score uses raw aggregate PASS counts from
`public-comparable-score.tsv` and `counts.tsv`. Normalized pass-set regression
accounting uses `7869` accepted normalized passes and `7196` candidate
normalized passes. The blocked candidate's regression list is the accepted
normalized pass set minus the candidate normalized pass set.

## Why 221205Z Cannot Move Public Score

The candidate gate status is `FINAL / BLOCKED-PASS-REGRESSIONS`. It finished
with public-comparable score `7197 / 20294`, but it lost `1166` rows that had
been PASS in the accepted public normalized baseline. Current scoring policy in
SQLite `goals.id=1` requires zero latest-public PASS regressions, or explicit
auditor adjudication of those regressions, before public score can move.

The candidate therefore remains control-plane evidence only. The accepted
public score stays `7873 / 20294` at `0b917f67` until the `1166` regression
rows are classified and either repaired, proven to be harness/environment
artifacts, or auditor-adjudicated.

## Primary Gate Artifacts Used

Accepted public gate:

- Directory:
  `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67`
- Score:
  `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67/public-comparable-score.tsv`
  reports `7873 / 20294 = 38.79%`.
- Aggregate counts:
  `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67/counts.tsv`
  reports raw `passed=7873`.
- Normalized pass baseline:
  `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67/current-passes.normalized.txt`
  has `7869` rows.
- Preflight:
  `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67/current-score-gate-preflight.tsv`
  records public/source head `0b917f67a37d9ca9779d77f87173b628431c2425`
  and php-src pin `f97ff597429a2fe633665a7e02d97c8077f9f90f`.

Blocked candidate gate:

- Directory:
  `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377`
- Worker status:
  `/home/claude/supervised-php-compiler/state/workers/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377.status.md`
  reports `FINAL / BLOCKED-PASS-REGRESSIONS`, `pass_regressions=1166`,
  `publication blocked`, and `PROGRESS.md not edited`.
- Score:
  `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/public-comparable-score.tsv`
  reports `7197 / 20294 = 35.46%`.
- Aggregate counts:
  `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/counts.tsv`
  reports raw `passed=7197`, `failed=8851`, `skipped=2222`,
  `borked=669`, `runnable=16058`.
- Regression summary:
  `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/pass-regression-summary.tsv`
  reports baseline normalized passes `7869`, candidate normalized passes
  `7196`, and `pass_regressions=1166`.
- Regression rows:
  `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/regressions-from-latest-published-passes.txt`
  has `1166` rows.
- Preflight:
  `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/current-score-gate-preflight.tsv`
  records created UTC `2026-06-04T22:12:05Z`, public/source head
  `56fe9377fb46be00db5fdd30c966fdba406dc581`, php-src pin
  `f97ff597429a2fe633665a7e02d97c8077f9f90f`, and baseline path to the
  accepted `135138Z` run.
- Environment:
  `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/environment.txt`
  records wrapper `/home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper`,
  php-src checkout `/home/claude/php-src-phpt`, and `PHPC_BIN` under the
  run root cargo target.
- Evidence hash manifest:
  `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/evidence-files.sha256`
  includes hashes for the score, count, normalized pass, regression, status,
  environment, and runner artifacts.
- Invalid marker summary:
  `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/invalid-proof-marker-summary.tsv`
  reports `invalid_marker_hits=0`.

## Relevant SQLite Rows

Database:
`/home/claude/php-to-native-compiler/.harness/harness.sqlite3`

- `goals.id=1`: primary metric policy says only full pinned gates with zero
  latest-public PASS regressions, or auditor-accepted adjudicated regressions,
  may move public score. It records accepted `7873 / 20294` at `0b917f67`
  and blocked `221205Z` at `7197 / 20294` with `1166` regressions.
- `metric_samples.id=1`: `accepted_public_phpt_passes`, value `7873`,
  target `20294`, percent `38.79`.
- `metric_samples.id=2`: `blocked_221205_candidate_phpt_passes`, value
  `7197`, target `20294`, percent `35.46`.
- `work_lanes.id=2`: PASS-regression manifest lane, status `in_progress`,
  branch `work/developer-94`, worktree
  `/home/claude/php-to-native-compiler/.harness/worktrees/developer-94`.
- `work_lanes.id=4`: this progress/control-plane refresh lane, status
  `in_progress`, branch `work/developer-96`, worktree
  `/home/claude/php-to-native-compiler/.harness/worktrees/developer-96`.
- `work_lanes.id=5`: PHPT manifest and late-row tag lane, status
  `in_progress`, branch `work/developer-95`, worktree
  `/home/claude/php-to-native-compiler/.harness/worktrees/developer-95`.
- `agents.id=90` (`developer-79`): status `crashed`, ended
  `2026-06-05T00:02:29+00:00`; notes say the tmux window was missing,
  message `41` was undeliverable, and lane 2 moved to `developer-94`.
- `agents.id=92` (`developer-81`): status `crashed`, ended
  `2026-06-05T00:05:37+00:00`; notes say the tmux window was missing,
  message `43` was undelivered, and no lane 5 artifact existed.
- `agents.id=93` (`developer-82`): status `crashed`, ended
  `2026-06-05T00:08:20+00:00`; notes say live pane `%107` did not process
  redelivered lane 4 prompt `68`.
- `agents.id=107` (`developer-94`): current lane 2 replacement, status
  `in_progress`; notes say it is building the `221205Z` PASS-regression
  manifest from saved artifacts.
- `agents.id=108` (`developer-95`): current lane 5 replacement, status
  `running: lane 5 PHPT manifest and late-row tag report; docs read,
  deterministic scans starting`.
- `agents.id=109` (`developer-96`): current lane 4 replacement.
- `messages.id=41`: original lane 2 assignment to `developer-79`, status
  `undeliverable`.
- `messages.id=43`: original lane 5 assignment to `developer-81`, status
  `undeliverable`.
- `messages.id=44`: original lane 4 assignment to `developer-82`, status
  `superseded_by_auditor_redelivery`.
- `messages.id=68`: corrective lane 4 redelivery to `developer-82`, status
  `delivered_unprocessed`.
- `messages.id=65`, `67`, `69`, `72`: auditor-2 findings/escalations for
  developer-79, developer-81, and developer-82 stale or unprocessed work.
- `spawn_requests.id=1`: replacement request for developer-79 lane 2,
  started as `developer-94`.
- `spawn_requests.id=2`: replacement request for developer-81 lane 5,
  started as `developer-95`.
- `spawn_requests.id=3`: replacement request for developer-82 lane 4,
  started as `developer-96`.
- `events.id=1902`: auditor-2 recorded developer-82 idle alert resolved by
  replacement `developer-96` on lane 4, with public metric unchanged.

## Lane Ownership State

Lane 2, PASS-regression manifest:

- Current owner in `work_lanes.id=2`: `developer-94`.
- Current status: `in_progress`.
- Required artifact:
  `.harness/reports/221205Z-pass-regression-manifest.md`.
- Current concern: repeated auditor idle prompts for `developer-94` appear in
  `events.id=1887` through `events.id=2032`, while `work_lanes.id=2` still
  lists `developer-94` as owner. This lane remains open until the manifest
  artifact classifies all `1166` PASS regressions.

Lane 5, PHPT manifest and late-row tags:

- Current owner in `work_lanes.id=5`: `developer-95`.
- Current status: `in_progress`.
- Required artifact:
  `.harness/reports/phpt-manifest-late-row-tags.md`.
- Current task: verify or correct planning counts for late rows
  (`142` eval-pattern, `86` variable-variable-pattern, `226` unique), keep
  those rows in denominator accounting, and avoid implementing eval or
  variable-variable support.

Lane 4, progress/control-plane refresh:

- Current owner in `work_lanes.id=4`: `developer-96`.
- Required artifact:
  `.harness/reports/blocked-221205Z-progress-refresh.md`.
- Public score must remain unchanged.

## Stale Or Undelivered Assignment Concerns

- `developer-79`: lane 2 was assigned through `messages.id=41`, but auditor-2
  found the tmux window missing, the worktree clean, no report artifact, and
  no test runs. The assignment was marked `undeliverable`, the agent was
  marked `crashed`, and lane 2 was moved to `developer-94`.
- `developer-81`: lane 5 was assigned through `messages.id=43`, but auditor-2
  found the tmux window missing, the worktree clean, and no late-row manifest
  artifact. The assignment was marked `undeliverable`, the agent was marked
  `crashed`, and lane 5 was moved to `developer-95`.
- `developer-82`: lane 4 was assigned through `messages.id=44`, but the
  worker had completed only a previous capacity prompt. Auditor-2 redelivered
  lane 4 through `messages.id=68`; that prompt became `delivered_unprocessed`.
  Auditor-2 then marked `developer-82` crashed and moved lane 4 to
  `developer-96`.

## Next Deterministic Action

Do not move public score from `7873 / 20294`.

Before score movement is possible, lane 2 must produce the full
`221205Z` PASS-regression manifest for all `1166` rows, and the team must use
that manifest plus focused accepted-vs-candidate replays to classify each
regression bucket. Only after the candidate has zero latest-public PASS
regressions, or the remaining regressions have been explicitly adjudicated by
an auditor, should another pinned full PHPT score gate be considered for public
score update.

Late eval and variable-variable rows should stay tagged by lane 5 and remain
in denominator accounting. They should not be implemented as part of this
blocked-gate recovery path.

## Commands Run

No full PHPT gate was run. Low-CPU artifact and SQLite inspection only:

```sh
sed -n '1,80p' /home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/public-comparable-score.tsv
sed -n '1,80p' /home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/pass-regression-summary.tsv
sed -n '1,80p' /home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/counts.tsv
sed -n '1,80p' /home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/current-score-gate-preflight.tsv
wc -l /home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/regressions-from-latest-published-passes.txt
sed -n '1,80p' /home/claude/supervised-php-compiler/state/workers/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377.status.md
python3 -c 'import sqlite3; ...'  # queried goals, metric_samples, work_lanes, agents, messages, spawn_requests, events
```
