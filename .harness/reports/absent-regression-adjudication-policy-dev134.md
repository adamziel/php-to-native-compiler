# Absent-Regression Adjudication Policy

Owner: developer-218
Lane: 97
Artifact requested: `.harness/reports/absent-regression-adjudication-policy-dev134.md`

Scope: read-only auditor-facing policy for absent rows after shard aborts in
the blocked `221205Z` public PHPT gate. No compiler/runtime source edits were
made, no focused PHPT replay was run, and no full PHPT gate was run.

## Decision

Absent candidate rows are not semantic PHP failures by themselves. They are
evidence gaps until a row-level replay or a complete gate proves otherwise.

For the blocked `221205Z` gate, the accepted public score must remain
`7873 / 20294 = 38.79%` at `0b917f67a37d9ca9779d77f87173b628431c2425`.
The `221205Z` candidate remains blocked at `7197 / 20294 = 35.46%` with
`1166` latest-public PASS regressions. Those regressions split as:

| Bucket | Rows | Policy result |
| --- | ---: | --- |
| Candidate row absent from `current-status.normalized.tsv` and `all-results.txt` | 1136 | Control-plane/evidence gap unless replay produces a real row status. |
| Candidate `FAILED` | 27 | Direct non-PASS evidence; replay or repair/adjudicate by narrow semantic/environment bucket. |
| Candidate `BORKED` | 3 | Direct SKIPIF/environment evidence; replay or repair/adjudicate by narrow constant/environment bucket. |

Focused report work, artifact audits, and row-level smoke replays cannot move
the public score. Public score movement requires a complete pinned gate with
zero latest-public PASS regressions, or an explicit auditor-accepted
adjudication record for every remaining regression row.

## Evidence Inputs

Accepted baseline:

`/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67`

Blocked candidate:

`/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377`

Supporting reports in this worktree:

- `.harness/reports/221205Z-pass-regression-manifest.md`
- `.harness/reports/221205Z-evidence-integrity.md`
- `.harness/reports/221205Z-status-symptom-crosscheck.md`
- `.harness/reports/accepted-score-accounting-audit.md`
- `.harness/reports/focused-replay-cookbook.md`
- `.harness/reports/full-gate-readiness-after-shard-fix-dev119.md`
- `.harness/reports/221205Z-unsupported-boundary-overlap.md`

Independent text check in this lane confirmed:

```text
regressions: 1166
candidate current-status buckets: ABSENT=1136, FAILED=27, BORKED=3
candidate all-results buckets:    ABSENT=1136, FAILED=27, BORKED=3
```

Shard 03 and shard 04 stdout end with:

```text
ERROR: cannot open directory: .../run-tests-harnesses/shard-03/ext/pdo/tests
ERROR: cannot open directory: .../run-tests-harnesses/shard-04/ext/pdo/tests
```

Those two shards also lack `run-tests.log`, even though every shard produced a
`results.txt`, which is why `aggregate-warnings.tsv` reporting
`missing_results=0` is not sufficient completeness proof.

## Terms

Use these labels consistently in auditor reports:

| Label | Meaning |
| --- | --- |
| `direct-non-pass` | The candidate aggregate has a row in `current-status.normalized.tsv` or `all-results.txt` with `FAILED`, `BORKED`, or another non-PASS status. |
| `absent-regression` | The row passed in the accepted normalized baseline and is missing from the candidate normalized PASS set, candidate normalized status, and candidate aggregate results. |
| `aborted-shard-absent` | An `absent-regression` whose assigned shard has direct abort/truncation evidence before the row could be observed. |
| `unexplained-absent` | An `absent-regression` without enough durable shard/list evidence to prove whether the row was skipped by harness truncation, omitted by aggregation/listing, or semantically failed without preservation. |
| `late-priority-adjacent` | A row containing clear `eval` or variable-variable evidence. These rows should not drive first-wave implementation, but still need explicit adjudication before score movement. |

Do not use `unsupported` as a standalone row status. If a row was a
latest-public PASS, unsupported-boundary prose is planning context, not a
waiver.

## What Can Be Adjudicated As Control-Plane

An auditor may classify a row as a control-plane/evidence issue when all of
these are true:

1. The row is present in the accepted normalized PASS baseline.
2. The row is missing from the candidate normalized PASS set.
3. The row is missing from candidate `current-status.normalized.tsv` and
   candidate `all-results.txt`.
4. No shard stdout or `run-tests.log` contains a row-level failure, bork, skip,
   or pass for that PHPT path.
5. There is durable evidence that the candidate gate could not observe the row
   cleanly, such as a shard abort before completion, a missing shard
   `run-tests.log`, a missing saved assignment list, a row-completeness
   mismatch, or an aggregation/listing defect.

For `221205Z`, the safest control-plane classification is for rows mapped to
the aborted shard 03 or shard 04 windows identified by the status/symptom
cross-check. The prior report counted `506` ambiguous rows in this aborted
shard bucket: shard 03 has `199`, and shard 04 has `307`.

This classification means:

- The row should not be used to open a compiler/runtime semantic repair lane.
- The row should remain in the blocked regression set.
- The next action is harness repair, evidence completion, or focused replay.
- The accepted public score remains unchanged.

Control-plane adjudication may explain why a candidate was blocked, but it
does not prove that the candidate would have passed the row.

## What Needs Replay

Replay is required before assigning semantic root cause when any of these are
true:

1. The row is absent, but shard assignment or completion evidence is incomplete.
2. The row is absent from a shard that appears to have produced some results,
   but the artifact set does not prove expected row coverage.
3. The row overlaps broad unsupported-adjacent areas such as reflection, SPL,
   filesystem/streams, session/SAPI, URI/POSIX, or extension metadata, but has
   no candidate row-level status.
4. The row has direct `FAILED`/`BORKED` status but the preserved artifacts lack
   enough diff/log detail to distinguish compiler behavior from SKIPIF,
   wrapper, environment, setup, or run-tests behavior.
5. The row is being proposed for a repair lane and the only current evidence is
   membership in the latest-public PASS regression list.

Replay should be focused, not a full gate, unless a manager/integrator asks for
the full pinned gate. Use the documented focused replay cookbook shape:

- Rebuild or restore exact accepted and candidate `phpc` binaries if historical
  `/tmp` run roots are gone.
- Use the pinned php-src checkout
  `f97ff597429a2fe633665a7e02d97c8077f9f90f`.
- Use the existing PHPT wrapper
  `/home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper`.
- Run a small row file with representative accepted-vs-candidate rows.
- Preserve `results.txt`, `run-tests.log`, `stdout.log`, `stderr.log`, the row
  file, exit status, and the exact `PHPC_BIN` commits.

Replay output can convert an absent row into one of three useful outcomes:

| Replay result | Next action |
| --- | --- |
| Candidate now emits `PASSED` | Treat the old row as control-plane evidence loss; include it in gate-completeness repair, not a semantic lane. |
| Candidate emits `FAILED`/`BORKED` with stable details | Move to a narrow semantic/environment repair or adjudication lane. |
| Candidate remains unobserved or aborts | Keep it in control-plane/harness repair until the row can be observed. |

## What Can Never Move Public Score

These must not be used as public-score movement proof:

1. A report-only artifact, including this policy.
2. A focused replay or smoke test, even if every sampled row passes.
3. Candidate-only PASS rows. They are useful progress signal but do not offset
   latest-public PASS regressions.
4. Raw pass-count improvement when normalized latest-public PASS regressions
   remain nonzero.
5. `aggregate-warnings.tsv` showing `missing_results=0` without expected
   PHPT path reconciliation and shard-local log coverage.
6. A blanket statement that rows are unsupported, dynamic, broad SPL, broad
   reflection, or late-priority. Every latest-public PASS regression still
   needs replay, repair, or explicit auditor adjudication.
7. A candidate gate whose evidence omits shard assignment lists, shard-local
   hashes, `run-tests.log`, or expected-path coverage checks.
8. Any candidate gate with unadjudicated `FAILED`, `BORKED`, `ABSENT`, or
   duplicate/conflicting status rows in the latest-public PASS regression set.

For `eval` and variable-variable rows: they may remain late-priority and should
not drive near-term implementation, but they are still score blockers unless an
auditor explicitly accepts a deferral/adjudication record for those exact rows.

## Auditor Workflow

For each regression row, auditors should apply this order:

1. Confirm the row is in the accepted normalized PASS baseline.
2. Look up candidate normalized status and candidate aggregate status.
3. If direct non-PASS exists, classify by concrete status and preserve the
   row-level evidence.
4. If absent, check shard assignment, shard stdout, `run-tests.log`, result
   count, and assignment-list preservation.
5. Mark `aborted-shard-absent` only when truncation/abort evidence explains
   why the row has no candidate status.
6. Mark `unexplained-absent` when the row is absent but the current artifacts
   do not prove the control-plane mechanism.
7. Require focused replay before semantic repair proposals for absent rows.
8. Escalate public score only after a complete pinned gate has zero
   latest-public PASS regressions, or after every remaining row has an
   explicit auditor-accepted adjudication record.

Use this minimal report row shape:

```text
row:
accepted_status:
candidate_status:
candidate_status_source:
absence_class:
control_plane_evidence:
replay_required:
semantic_lane_allowed:
score_effect:
```

`score_effect` should be one of:

- `none-report-only`
- `none-focused-replay-only`
- `none-blocked-regression`
- `eligible-only-after-complete-pinned-gate`
- `eligible-only-after-explicit-auditor-adjudication`

## Applied To 221205Z

Current recommended treatment:

| Set | Rows | Treatment |
| --- | ---: | --- |
| Absent rows on aborted shard 03/04 mapping | 506 | Control-plane/evidence loss; replay after shard harness fix before semantic repair. |
| Other absent rows | 630 | Replay/evidence-integrity target; current artifacts cannot assign semantic root cause. |
| Direct `FAILED` rows | 27 | Narrow replay/repair/adjudication by concrete failure cluster. |
| Direct `BORKED` rows | 3 | SKIPIF/environment constant exposure lane or explicit adjudication. |
| Clear late-priority `eval` / variable-variable rows | 5 | Defer from first-wave implementation; still score blockers unless explicitly adjudicated. |

The next deterministic public-score path is not to waive the absent rows. It is
to fix gate completeness, preserve shard-local evidence and assignment lists,
repair the shard 03/04 copied-harness abort class, replay representative absent
rows, and only then schedule another complete pinned gate when preflight
guardrails are satisfied.

## Commands Used

No full PHPT gate was run. Commands were low-CPU report reads and archived
artifact text checks:

```sh
sed -n '1,240p' .harness/reports/221205Z-pass-regression-manifest.md
sed -n '1,240p' .harness/reports/221205Z-evidence-integrity.md
sed -n '1,240p' .harness/reports/221205Z-status-symptom-crosscheck.md
sed -n '1,240p' .harness/reports/accepted-score-accounting-audit.md
sed -n '1,240p' .harness/reports/focused-replay-cookbook.md
rg -n "absent|ABSENT|control-plane|replay|public score|shard|missing" .harness/reports/*.md
```

Python's standard `sqlite3` module was used for harness DB inspection and
status/event updates because the `sqlite3` CLI and named SQLite MCP tools were
not available in this session.
