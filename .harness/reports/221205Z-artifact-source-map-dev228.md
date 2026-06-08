# 221205Z Artifact And Source Path Map

Agent: developer-379
Lane/card: 107, requeued from the original dev228 artifact name
Generated: 2026-06-08T13:34:09Z
Mode: read-only M0/M1 artifact lane; no compiler/runtime/product source edits;
no PHPT gate; no public score movement.

## Decision

The blocked `221205Z` candidate still must not move the public PHPT score.
The scheduler-visible accepted public score remains `7873 / 20294 = 38.79%`.
The blocked candidate remains `7197 / 20294 = 35.46%` with `1166`
latest-public PASS regressions.

This lane maps the recorded artifact/source paths and their current local
availability. It does not revalidate raw `221205Z` row artifacts because the
historical evidence directories and `/tmp` run roots referenced by the gate are
not present in this environment. The worker status file and integrated
`.harness/reports` artifacts remain available and are the current durable
evidence for this report.

## Current Availability Map

| Item | Path | Current state | Use |
| --- | --- | --- | --- |
| Candidate evidence root | `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377` | missing | Recorded scored artifact root; must be restored for raw row/log revalidation. |
| Accepted baseline evidence root | `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67` | missing | Recorded baseline PASS artifact root; must be restored for set-diff revalidation. |
| Candidate worker status | `/home/claude/supervised-php-compiler/state/workers/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377.status.md` | present | Durable summary of run inputs, final state, shard exits, counts, public score, and PASS-regression summary. |
| Candidate worker lock | `/home/claude/supervised-php-compiler/state/workers/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377.lock` | present | Residual lock artifact only; not score evidence by itself. |
| Candidate run root | `/tmp/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377` | missing | Historical scratch root; contained candidate build, shard temp files, and local harness copies. |
| Accepted run root | `/tmp/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67` | missing | Historical scratch root for accepted baseline replay/binary. |
| Candidate `phpc` binary | `/tmp/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/cargo-target/release/phpc` | missing | Must be rebuilt or restored before focused accepted-vs-candidate replay. |
| Accepted `phpc` binary | `/tmp/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67/cargo-target/release/phpc` | missing | Must be rebuilt or restored before focused accepted-vs-candidate replay. |
| PHPT wrapper | `/home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper` | present and executable | Wrapper recorded by the gate and usable for future focused replay. |
| Pinned php-src checkout | `/home/claude/php-src-phpt` | present at `f97ff597429a2fe633665a7e02d97c8077f9f90f` | Durable PHPT source for row lists, `run-tests.php`, and redirect source inspection. |
| Source repo seed | `/home/claude/supervised-php-compiler/state/worktrees/batch024-current-supervisor-20260601T093225` | present | Recorded seed for the candidate source checkout. |

## Recorded Gate Inputs From Worker Status

The current worker status file records:

| Field | Value |
| --- | --- |
| state | `FINAL / BLOCKED-PASS-REGRESSIONS` |
| updated UTC | `2026-06-04T22:29:34Z` |
| run id | `phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377` |
| public/source head | `56fe9377fb46be00db5fdd30c966fdba406dc581` |
| php-src pin | `f97ff597429a2fe633665a7e02d97c8077f9f90f` |
| shard count | `6 plus serialized open_basedir list` |
| wrapper | `/home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper` |
| `PHPC_BIN` | `/tmp/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/cargo-target/release/phpc` |
| cargo target dir | `/tmp/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/cargo-target` |
| harness root | `/tmp/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/run-tests-harnesses` |
| baseline passes | `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67/current-passes.normalized.txt` |
| pinned denominator | `20294` |

The same status file records all six shards and the serialized open_basedir
slice exiting `1`, aggregate counts `7197` passed / `16058` runnable, public
comparable score `7197 / 20294 = 35.46%`, and PASS-regression summary
`baseline_passes=7869`, `current_passes=7196`, `pass_regressions=1166`.

## Integrated Report Evidence Map

These integrated reports are present in this worktree and provide the durable
evidence while the raw gate directories are absent:

| Report | Evidence carried forward |
| --- | --- |
| `.harness/reports/221205Z-pass-regression-manifest.md` | Exact `1166` PASS-regression count, split into `1136` absent, `27` failed, and `3` borked rows; names major extension/directory clusters. |
| `.harness/reports/221205Z-evidence-integrity.md` | Recorded gate command shape, artifact hash coverage limits, shard 03/04 missing `run-tests.log`, and incomplete evidence concerns. |
| `.harness/reports/221205Z-shard-abort-root-cause.md` | Root cause for shard 03/04 aborts: copied `run-tests.php` changed `__DIR__` under incomplete `run-tests-harnesses/shard-*` roots, breaking PDO `REDIRECTTEST` paths. |
| `.harness/reports/221205Z-shard-rerun-smoke-dev116.md` | Follow-up smoke evidence for the shard rerun path; already integrated as lane 78. |
| `.harness/reports/221205Z-regression-status-summary-refresh-dev313.md` | Refresh of `ABSENT=1136`, `FAILED=27`, `BORKED=3`; refines aborted-shard absent rows as `506` and other absent rows as `630`. |
| `.harness/reports/first-repair-lane-evidence-readiness-dev236.md` | Concludes product/compiler-runtime repair is not ready from absent clusters; control-plane gate completeness and shard layout are the evidence-ready repair class. |
| `.harness/reports/phpt-binary-wrapper-availability-dev133.md` and `.harness/reports/phpt-binary-wrapper-availability-recheck-dev305.md` | Record that historical `/tmp` accepted/candidate binaries are missing and that replay needs restored or rebuilt durable binaries. |
| `.harness/reports/full-gate-restart-checklist.md` and `.harness/reports/zero-regression-gate-preflight-checklist-dev433.md` | Record restart prerequisites: expected-row reconciliation, shard-local artifact hashing, shard logs, assignment lists, and zero/adjudicated latest-public PASS regressions. |

## Source Path Map For Next Work

| Source or owner path | Current status | Why it matters |
| --- | --- | --- |
| Archived `run_gate.sh` under the missing candidate evidence root | unavailable locally | Earlier reports quote the failing `prepare_run_tests_harness` body, but the raw script cannot be reread until the evidence root is restored. |
| `/home/claude/supervised-php-compiler/state/worktrees/batch024-current-supervisor-20260601T093225` | present | Recorded seed for source checkout at candidate head. A future control-plane owner should identify the durable generator for the archived full-gate script from this source or supervisor tooling before patching. |
| `/home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper` | present/executable | Wrapper used by gate and needed for focused replay. |
| `/home/claude/php-src-phpt/run-tests.php` | present | Pinned php-src runner used by the gate; lane69 identifies `run-tests.php` `__DIR__` behavior as central to the copied-harness abort. |
| `/home/claude/php-src-phpt/ext/pdo_mysql/tests/common.phpt` | present via pinned php-src | Redirect source for shard-03 abort class. |
| `/home/claude/php-src-phpt/ext/pdo_pgsql/tests/common.phpt` | present via pinned php-src | Redirect source for shard-04 abort class. |
| `/home/claude/php-src-phpt/ext/pdo_odbc/tests/common.phpt` | present via pinned php-src | Comparison redirect path noted by lane69. |
| `.harness/reports/*221205Z*.md` | present integrated reports | Durable scheduler/git evidence to use when raw logs are absent. |

## Scheduler And Metric State

SQLite `metric_samples` currently records:

| Metric | Value | Target | Percent | Latest sample |
| --- | ---: | ---: | ---: | --- |
| `accepted_public_phpt_passes` | `7873` | `20294` | `38.79` | `2026-06-05T09:32:24+00:00` |
| `blocked_221205_candidate_phpt_passes` | `7197` | `20294` | `35.46` | `2026-06-05T09:32:24+00:00` |

Relevant lane state observed during this report:

| Lane | Status/stage | Artifact implication |
| ---: | --- | --- |
| `69` | `integrated` / `done` | Shard abort root-cause report is integrated. |
| `78` | `integrated` / `done` | Shard rerun smoke report is integrated. |
| `107` | `assigned` / `development` | This missing artifact/source map is the current lane. |
| `117` | `completed` / `planned` | Evidence readiness report exists; status has not been advanced to done in this DB view. |
| `130` | `integrated` / `done` | Regression status summary refresh is integrated. |

## Gaps This Map Leaves Explicit

- Raw candidate and accepted gate directories are absent, so this lane did not
  rerun `sha256sum -c evidence-files.sha256`, recount raw result rows, or read
  shard stdout/log files directly.
- The historical accepted and candidate `phpc` binaries under `/tmp` are
  absent. Focused replay lanes need restored binaries or deterministic rebuilds
  for `0b917f67a37d9ca9779d77f87173b628431c2425` and
  `56fe9377fb46be00db5fdd30c966fdba406dc581`.
- The durable source path that generated the archived full-gate `run_gate.sh`
  still needs an owner. Earlier reports identify the broken script body and a
  likely fix, but implementation should not patch product compiler/runtime code
  for this control-plane failure.
- Public score remains unchanged until a full pinned gate has complete
  expected-row evidence and zero or auditor-adjudicated latest-public PASS
  regressions.

## Commands And Queries Used

Project context and reports:

```sh
sed -n '1,220p' AGENTS.md
sed -n '1,240p' docs/PROGRESS.md
sed -n '1,260p' docs/ARCHITECTURE.md
sed -n '1,260p' docs/SUPPORT.md
sed -n '1,260p' README.md
sed -n '1,260p' /home/claude/php-to-native-compiler/DEVELOPMENT.md
sed -n '1,220p' docs/LOOP_MEMORY.md
sed -n '1,190p' /home/claude/php-to-native-compiler/PLAN.md
sed -n '1,240p' .harness/reports/221205Z-evidence-integrity.md
sed -n '1,260p' .harness/reports/221205Z-shard-abort-root-cause.md
sed -n '1,260p' .harness/reports/first-repair-lane-evidence-readiness-dev236.md
sed -n '1,220p' .harness/reports/report-artifact-branch-map-dev131.md
sed -n '1,260p' .harness/reports/full-gate-restart-checklist.md
sed -n '1,220p' .harness/reports/221205Z-pass-regression-manifest.md
```

Local path checks:

```sh
find /home/claude/supervised-php-compiler/state/logs -maxdepth 1 -type d -name '*221205*' -print
find /home/claude/supervised-php-compiler/state/logs -maxdepth 1 -type d -name '*135138*' -print
find /home/claude/supervised-php-compiler/state/workers -maxdepth 1 -type f -name '*221205*' -print
test -d /home/claude/php-src-phpt && git -C /home/claude/php-src-phpt rev-parse HEAD
test -f /home/claude/php-src-phpt/run-tests.php
test -d /home/claude/php-src-phpt/ext/pdo/tests
test -f /home/claude/php-src-phpt/ext/pdo_mysql/tests/common.phpt
test -f /home/claude/php-src-phpt/ext/pdo_pgsql/tests/common.phpt
test -f /home/claude/php-src-phpt/ext/pdo_odbc/tests/common.phpt
test -x /home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper
test -d /home/claude/supervised-php-compiler/state/worktrees/batch024-current-supervisor-20260601T093225
test -d /tmp/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377
test -d /tmp/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67
test -x /tmp/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/cargo-target/release/phpc
test -x /tmp/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67/cargo-target/release/phpc
```

Harness MCP queries:

```sql
SELECT id, name, role, current_status, cwd, worktree, branch, notes, ended_at
FROM agents WHERE name = 'developer-379';

SELECT id, title, description, goal, acceptance_criteria, status, stage,
       card_type, priority, owner_agent_id, branch_name, worktree_path,
       dependencies, notes, review_required, integration_required,
       created_at, assigned_at, last_activity_at
FROM worklanes
WHERE id = 107;

SELECT id, ts, type, agent_name, message, payload_json
FROM events
WHERE message LIKE '%221205Z%' OR payload_json LIKE '%221205Z%'
ORDER BY id DESC LIMIT 80;

SELECT id, ts, type, agent_name, message, payload_json
FROM events
WHERE message LIKE '%source path%' OR payload_json LIKE '%source path%'
ORDER BY id DESC LIMIT 80;

SELECT * FROM metric_samples ORDER BY id DESC LIMIT 20;

SELECT id, title, status, stage, branch_name, worktree_path, notes,
       assigned_at, last_activity_at
FROM worklanes
WHERE id IN (68,69,78,107,117,130)
ORDER BY id;
```

## Next Deterministic Action

Assign a control-plane owner to locate the durable generator/source for the
archived full-gate `run_gate.sh`, then implement the shard-harness layout and
expected-row completeness fixes described by the integrated lane69/lane78
reports. In parallel, restore or rebuild the accepted/candidate `phpc` binaries
before product replay lanes treat absent rows as semantic failures.
