# Progress Dashboard Input Audit

Lane: 116, developer-428
Generated: 2026-06-05T10:00Z

Scope: read-only M0/M1 status artifact audit. No compiler, runtime, harness
implementation, dashboard, or product source files were edited. No full PHPT
gate was run, and no public score movement is claimed.

## Result

The authoritative public metric remains:

| Metric | Value | Source |
| --- | ---: | --- |
| Accepted public PHPT score | `7873 / 20294 = 38.79%` | SQLite `goals.id=1`, latest `metric_samples` pair, root `PLAN.md`, completed metric reports |
| Blocked 221205Z candidate | `7197 / 20294 = 35.46%` | SQLite `goals.id=1`, latest `metric_samples` pair, root `PLAN.md`, completed metric reports |
| Candidate PASS regressions | `1166` | root `PLAN.md`, `.harness/reports/221205Z-regression-status-summary-refresh-dev313.md` |

Do not move the public score from `7873 / 20294`. The `221205Z` candidate
remains blocked by `1166` latest-public PASS regressions.

## Current SQLite State

`metric_samples` now contains four matching accepted/blocked sample pairs:

| ids | timestamp | accepted | blocked |
| --- | --- | ---: | ---: |
| `1`, `2` | `2026-06-04T23:41:27+00:00` | `7873 / 20294 = 38.79%` | `7197 / 20294 = 35.46%` |
| `3`, `4` | `2026-06-05T08:06:10+00:00` | `7873 / 20294 = 38.79%` | `7197 / 20294 = 35.46%` |
| `5`, `6` | `2026-06-05T08:14:00+00:00` | `7873 / 20294 = 38.79%` | `7197 / 20294 = 35.46%` |
| `7`, `8` | `2026-06-05T09:32:24+00:00` | `7873 / 20294 = 38.79%` | `7197 / 20294 = 35.46%` |

`goals.id=1` still defines the primary metric as
`accepted_public_phpt_passes / pinned_public_runnable_denominator` and says
only a full pinned zero-regression PHPT gate, or auditor-accepted adjudicated
regressions, may move public score.

Current open bug rows remain relevant to dashboard interpretation:

| bug_report | status | dashboard implication |
| --- | --- | --- |
| `#1 php_runtime::lib::run62_runtime_abi_expectation_cluster` | `open` | Runtime source repair remains unresolved. |
| `#2 harness::test_command_selection::python_unittest_zero_tests_run63` | `fixed` | Marked fixed, but run 215 created a new in-progress lane and manager evidence says live selector proof is contested. |
| `#3`, `#4`, `#5`, `#6` idle-alert/liveness variants | `open` | Control-plane liveness/dedupe cannot be shown as fully closed. |

## Status And Progress Artifacts

### Fresh Enough

- `/home/claude/php-to-native-compiler/STATUS.md`
- `/home/claude/php-to-native-compiler/STATUS.html`
- `/home/claude/php-to-native-compiler/progress.md`
- `/home/claude/php-to-native-compiler/progress.html`

These files were regenerated around `2026-06-05T10:00Z`. They include recent
work lanes `143` through `146` and recent events such as the run 215 follow-up
and lane 116 assignment/deconflict notes. Their lane/event sections are current
enough for this audit.

### Needs Progress-Maintainer Refresh

The generated status/progress files still show only:

```text
blocked_221205_candidate_phpt_passes: 7197.0 / 20294.0 (35.5%)
```

That value is real but non-public. It should not be the only visible metric.
The progress maintainer should change the generated `Metric` field/card to show
both:

- accepted public score: `accepted_public_phpt_passes: 7873 / 20294 = 38.79%`;
- blocked candidate, clearly labelled non-public:
  `blocked_221205_candidate_phpt_passes: 7197 / 20294 = 35.46%`, with
  `1166` PASS regressions.

The root public report is stale:

- `/home/claude/php-to-native-compiler/PROGRESS.md`
  - `Updated:` still says `2026-05-30 01:42 CEST`.
  - `Latest source head:` still says `73358122 fix: repair array_multisort metadata`.
  - `Current score:` still says `3646 / 20294 = 17.97%`.
  - Latest score-history row is still `Batch015 checkpoint9`.

Those fields conflict with the authoritative accepted public metric
`7873 / 20294 = 38.79%`. Either update `PROGRESS.md` through the dedicated
progress-maintainer path or mark it explicitly stale/non-authoritative.

The previous consistency report remains directionally correct but is now stale
in details:

- `/home/claude/php-to-native-compiler/.harness/reports/public-metric-status-consistency-dev230.md`
  - It says there are three samples for each metric and the latest samples are
    from `2026-06-05T08:14:00+00:00`.
  - Current SQLite has four samples for each metric, latest at
    `2026-06-05T09:32:24+00:00`.
  - Its main conclusion still holds: accepted public score is `7873 / 20294`
    and the generated status metric should not show only the blocked
    candidate.

## Files/Fields To Refresh

| File | Field or section | Required state |
| --- | --- | --- |
| `STATUS.md` | `## Metric` | Include accepted public metric and clearly separate blocked candidate. |
| `STATUS.html` | Metric card | Same as `STATUS.md`; blocked metric must not appear as sole progress value. |
| `progress.md` | `## Metric` | Same as `STATUS.md`; current lane/event table can stay. |
| `progress.html` | Metric card | Same as `STATUS.html`; current lane/event table can stay. |
| `PROGRESS.md` | `Updated`, `Latest source head`, `Current score`, score history | Refresh via progress maintainer or mark stale/non-authoritative. |
| `.harness/reports/public-metric-status-consistency-dev230.md` | sample-count details | Supersede or annotate with latest metric sample ids `7` and `8`. |

## Commands And Queries

No recursive `.harness/worktrees` scan was run.

```sql
SELECT * FROM metric_samples ORDER BY id;
SELECT id, substr(text,1,240), substr(measure,1,1000), status, updated_at
FROM goals ORDER BY id;
SELECT id,test_nodeid,status,occurrences,first_failed_commit,fixed_commit,
       updated_at,substr(root_cause,1,220)
FROM bug_reports ORDER BY id;
SELECT id,command,status,commit_sha,started_at,ended_at,summary_json
FROM test_runs ORDER BY id DESC LIMIT 20;
SELECT status,COUNT(*) FROM work_lanes GROUP BY status ORDER BY status;
```

```sh
find /home/claude/php-to-native-compiler -maxdepth 2 \
  \( -name 'STATUS*' -o -name 'PLAN.md' -o -name 'GOAL.MD' \) \
  -not -path '/home/claude/php-to-native-compiler/.harness/worktrees/*' -print

rg -n "accepted_public_phpt_passes|blocked_221205_candidate_phpt_passes|7873|7197|20294|38\\.79|35\\.46|3646|17\\.97|Current score|Metric" \
  /home/claude/php-to-native-compiler/STATUS.md \
  /home/claude/php-to-native-compiler/STATUS.html \
  /home/claude/php-to-native-compiler/progress.md \
  /home/claude/php-to-native-compiler/progress.html \
  /home/claude/php-to-native-compiler/PLAN.md \
  /home/claude/php-to-native-compiler/PROGRESS.md
```

Late DB reads briefly hit `database is locked` while managers were updating
lanes; successful reads before and after the lock were sufficient for this
artifact.
