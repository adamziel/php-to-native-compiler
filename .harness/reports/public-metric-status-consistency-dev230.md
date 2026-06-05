# Public Metric And Status Consistency Audit

Lane: 108, developer-292
Generated: 2026-06-05T08:59Z

Scope: read-only M0/M1 artifact lane. No compiler/runtime/product source files
were edited. No dashboard files were edited. No full PHPT gate was run, and no
public score movement is claimed.

## Authoritative State

The authoritative public metric remains:

| Metric | Value | Source |
| --- | ---: | --- |
| Accepted public PHPT score | `7873 / 20294 = 38.79%` | `goals.id=1`; latest `metric_samples.accepted_public_phpt_passes`; root `PLAN.md`; recent auditor events |
| Blocked 221205Z candidate | `7197 / 20294 = 35.46%` | `goals.id=1`; latest `metric_samples.blocked_221205_candidate_phpt_passes`; root `PLAN.md`; recent auditor events |
| Candidate PASS regressions | `1166` | `goals.id=1`; root `PLAN.md`; recent auditor events and existing 221205Z reports |

Public score must not move from `7873 / 20294` until a full pinned PHPT gate has
zero latest-public PASS regressions or an auditor accepts adjudicated
regressions.

## Consistency Checks

### SQLite Goal And Metric Samples

`goals.id=1` and `metric_samples` agree.

- `goals.measure` states the primary metric is
  `accepted_public_phpt_passes / pinned_public_runnable_denominator`.
- `goals.measure` records accepted `7873 / 20294 = 38.79%` at `0b917f67`.
- `goals.measure` records blocked `221205Z` at `7197 / 20294` with `1166`
  PASS regressions and no score movement.
- `metric_samples` has three samples for `accepted_public_phpt_passes`; all are
  `7873.0 / 20294.0 = 38.79`.
- `metric_samples` has three samples for
  `blocked_221205_candidate_phpt_passes`; all are
  `7197.0 / 20294.0 = 35.46`.
- The latest samples for both metrics are from `2026-06-05T08:14:00+00:00`.

No metric sample conflict was found.

### PLAN.md

The authoritative root checkout file
`/home/claude/php-to-native-compiler/PLAN.md` agrees with SQLite:

- accepted public score: `7873 / 20294 = 38.79%`
- accepted source: `0b917f67a37d9ca9779d77f87173b628431c2425`
- blocked candidate: `7197 / 20294 = 35.46%`
- blocked candidate regressions: `1166`
- explicit instruction that the blocked candidate must not move public score

The lane worktree itself does not contain `PLAN.md`. That is expected from the
current worktree contents, but it means consumers must read the root
`/home/claude/php-to-native-compiler/PLAN.md` or SQLite goal row for current
planning state.

### Recent Events

Recent metric/status events agree with SQLite and `PLAN.md`.

Representative latest events:

- `events.id=93743` records metric unchanged:
  `accepted_public_phpt_passes=7873/20294 (38.79%)` and blocked candidate
  `7197/20294 (35.46%)` with `1166` regressions.
- `events.id=93705`, `93686`, and `93685` repeat the same accepted and blocked
  metric values while deconflicting control-plane lanes.
- Earlier auditor events around `2026-06-05T08:24Z` to `08:25Z` also keep the
  same public metric and explicitly reject PHPT score movement from scheduler
  churn or blocked candidate evidence.

No recent event was found claiming accepted public score movement above
`7873 / 20294`.

## Mismatches Found

### 1. PROGRESS.md Is Stale

Both the lane worktree and root checkout `PROGRESS.md` still report:

```text
Current score: **3646 / 20294 pinned runnable PHPTs = 17.97%**.
```

This conflicts with the authoritative SQLite/PLAN state, where the accepted
public score is `7873 / 20294 = 38.79%`.

Root `PLAN.md` already warns that the local root checkout is dirty and stale,
and says not to treat its local progress files as current product truth. The
stale `PROGRESS.md` should therefore be treated as a known dashboard/progress
input mismatch, not as the current metric.

### 2. Generated STATUS/Progress Dashboard Shows Only The Blocked Metric

The generated root status files:

- `/home/claude/php-to-native-compiler/STATUS.md`
- `/home/claude/php-to-native-compiler/progress.md`
- `/home/claude/php-to-native-compiler/STATUS.html`
- `/home/claude/php-to-native-compiler/progress.html`

currently show the single metric:

```text
blocked_221205_candidate_phpt_passes: 7197.0 / 20294.0 (35.5%)
```

That value is real, but it is a blocked candidate metric, not the accepted
public score. Showing it as the only dashboard metric can mislead readers into
treating `7197 / 20294` as the current project score. The dashboard should
surface accepted `7873 / 20294` and blocked `7197 / 20294` as separate labels,
or make the blocked metric explicitly non-public.

### 3. Worktree PLAN.md Is Absent

`goals.plan_path` points to `PLAN.md`, but `PLAN.md` is absent from this lane
worktree. The current planning truth was found at the repository root
`/home/claude/php-to-native-compiler/PLAN.md` and in SQLite.

This is not a score conflict, but it is an artifact-discovery inconsistency for
workers who only inspect their dedicated worktree.

## Non-Mismatches

- Existing reports such as
  `.harness/reports/blocked-221205Z-progress-refresh.md`,
  `.harness/reports/accepted-score-accounting-audit.md`, and
  `.harness/reports/221205Z-status-symptom-crosscheck.md` align with the
  authoritative metric split.
- The `7873` versus `7869` and `7197` versus `7196` count differences are
  already explained by raw aggregate PASS accounting versus normalized pass-set
  regression accounting in `accepted-score-accounting-audit.md`.
- The blocked `221205Z` candidate remains evidence for repair planning only; it
  does not move public score.

## Recommended Follow-Up

Do not edit score dashboards from this lane. The progress/dashboard maintainer
should refresh status inputs so:

1. accepted public score is shown as `7873 / 20294 = 38.79%`;
2. blocked candidate is shown separately as `7197 / 20294 = 35.46%` with
   `1166` PASS regressions;
3. stale `PROGRESS.md` content at `3646 / 20294 = 17.97%` is either updated or
   clearly marked stale/non-authoritative;
4. dashboard generators do not select `blocked_221205_candidate_phpt_passes` as
   the only displayed metric.

## Commands And Queries Run

No recursive `.harness/worktrees` scan was run.

```sh
sed -n '1,220p' AGENTS.md
sed -n '1,260p' docs/PROGRESS.md
sed -n '1,260p' docs/ARCHITECTURE.md
sed -n '1,260p' docs/SUPPORT.md
sed -n '1,240p' README.md
sed -n '1,260p' docs/LOOP_MEMORY.md
rg --files -g 'DEVELOPMENT.md' -g 'AGENTS.md' -g 'GOAL.MD' -g 'docs/NEXT_TASKS.md' -g 'docs/OPERATIONS.md'
sed -n '1,240p' GOAL.MD
sed -n '1,260p' docs/NEXT_TASKS.md
sed -n '1,260p' docs/OPERATIONS.md
git status --short --branch
```

```sh
python3 - <<'PY'
import sqlite3
con=sqlite3.connect('/home/claude/php-to-native-compiler/.harness/harness.sqlite3')
con.row_factory=sqlite3.Row
cur=con.cursor()
for row in cur.execute('select id,text,measure,status,auditor_summary,updated_at from goals order by id desc limit 3'):
    print(dict(row))
for row in cur.execute('select * from metric_samples order by rowid desc limit 20'):
    print(dict(row))
for row in cur.execute("""
with latest as (
  select metric_name, max(id) as id from metric_samples group by metric_name
)
select m.* from metric_samples m join latest l on m.id=l.id order by m.metric_name
"""):
    print(dict(row))
for row in cur.execute("""
select metric_name, count(*) as rows, count(distinct value) as distinct_values,
       group_concat(distinct value) as values_seen,
       group_concat(distinct target) as targets_seen,
       group_concat(distinct percent_ready) as percents_seen,
       min(ts) as first_ts, max(ts) as last_ts
from metric_samples group by metric_name order by metric_name
"""):
    print(dict(row))
con.close()
PY
```

```sh
find . -maxdepth 3 \( -path './.harness/worktrees' -o -path './target' -o -path './.git' \) -prune -o \( -iname '*status*' -o -iname '*progress*' -o -name 'PLAN.md' -o -name 'GOAL.MD' \) -print | sort
find /home/claude/php-to-native-compiler -maxdepth 1 -type f \( -name 'PLAN.md' -o -iname 'STATUS*' -o -iname '*PROGRESS*' -o -name 'GOAL.MD' \) -print | sort
sed -n '1,260p' /home/claude/php-to-native-compiler/PLAN.md
sed -n '1,80p' /home/claude/php-to-native-compiler/PROGRESS.md
sed -n '1,140p' /home/claude/php-to-native-compiler/progress.md
rg -n "accepted_public_phpt_passes|blocked_221205_candidate_phpt_passes|7873|20294|7197|3646|17\.97|38\.79|35\.46|public score|Current score" /home/claude/php-to-native-compiler/PLAN.md /home/claude/php-to-native-compiler/PROGRESS.md /home/claude/php-to-native-compiler/progress.md /home/claude/php-to-native-compiler/STATUS.md /home/claude/php-to-native-compiler/STATUS.html /home/claude/php-to-native-compiler/progress.html
rg -n "accepted_public_phpt_passes|blocked_221205_candidate_phpt_passes|7873|20294|7197|38\.79|35\.46|public score|public metric|blocked candidate" GOAL.MD docs .harness -g '!worktrees/**' -g '!harness.sqlite3' -g '!*.sqlite3'
sed -n '1,260p' .harness/reports/blocked-221205Z-progress-refresh.md
sed -n '1,180p' .harness/reports/accepted-score-accounting-audit.md
sed -n '1,220p' .harness/reports/221205Z-status-symptom-crosscheck.md
```

Pre-assignment reserve health check, recorded separately as `test_runs.id=135`:

```sh
umask 0002; env CARGO_TARGET_DIR=/tmp/phpc-target-dev292-capacity CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 cargo check -q -p phpc
```
