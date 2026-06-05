# Zero-Regression Full-Gate Restart Checklist

Owner: developer-129
Lane: 93
Mode: control-plane report only; no compiler/runtime source edits; no public
score movement.

## Decision

Do not start another full pinned public PHPT score gate until the prerequisites
below are closed with recorded evidence. The latest accepted public score stays
`7873 / 20294 = 38.79%` at source `0b917f67`. The later
`phpt-full-current-score-20260604T221205Z` candidate stays blocked at
`7197 / 20294 = 35.46%` because it has `1166` latest-public PASS regressions.

The restart objective is not "get a higher raw pass count." The objective is a
complete, reproducible full gate with zero latest-public PASS regressions, or
with every remaining regression explicitly adjudicated and accepted by the
auditor.

## Evidence Inputs

- `PLAN.md`: public score can move only from a full pinned PHPT gate with zero
  latest-public PASS regressions or auditor-accepted adjudication.
- `origin/work/developer-83:.harness/reports/accepted-score-accounting-audit.md`:
  raw public counts are `7873` accepted and `7197` blocked candidate; normalized
  regression accounting uses `7869` accepted rows and `7196` candidate rows.
- `origin/work/developer-94:.harness/reports/221205Z-pass-regression-manifest.md`:
  `1166` regressions split into `1136` ABSENT, `27` FAILED, and `3` BORKED.
- `origin/work/developer-97:.harness/reports/221205Z-evidence-integrity.md`:
  shard 03/04 aborted with missing `run-tests-harnesses/.../ext/pdo/tests`
  directories, both lack `run-tests.log`, and aggregate evidence has `18949`
  result rows versus `21827` PHPT files.
- Root `.harness/reports/integration-backlog-triage-20260605T0026Z.md`:
  completed M0 report branches are report-only candidates, but most artifacts
  were not yet materialized into root `.harness/reports`.
- Root `.harness/reports/221205Z-standard-filesystem-http.md`: representative
  filesystem/http rows are mostly absent from candidate status, so replay is
  required before semantic repair.
- Harness DB test runs: run 62 failed `tools/run-tests.sh` at stale
  `e147c033` with `php_runtime --lib`; run 63 incorrectly executed
  `python -m unittest discover -s tests -v` and ran zero tests.

## Restart Prerequisites

### 1. Runtime failed-test blocker is closed

Required evidence:

- The scoring source commit includes the accepted runtime repair, or an
  equivalent focused fix.
- `cargo test -p php_runtime --lib -- --test-threads=1` passes from that exact
  scoring source commit with a unique `CARGO_TARGET_DIR`,
  `CARGO_BUILD_JOBS=1`, and `CARGO_INCREMENTAL=0`.
- The completion report names changed files, the verified commit SHA, and the
  final pass/fail count.

Current reason this is not yet restart-ready:

- Run 62 failed at `e147c033` with 16 `php_runtime --lib` failures.
- Lane 40 reports commit `2f8aec28` passed `419/419`, but the integrator still
  needs to accept it or verify an equivalent fix on the eventual scoring commit.

### 2. Harness command routing is deterministic

Required evidence:

- The scheduler/test-loop selector for this repository resolves to
  `tools/run-tests.sh` or another documented Rust/PHP product gate, never
  `python -m unittest discover -s tests -v`.
- Harness self-tests cover command selection and duplicate failed-lane churn.
- A dry-run or scheduler-visible proof records the selected command after the
  deployed fix, not only in a worker branch.

Current reason this is not yet restart-ready:

- Lane 37 reported a selector patch, but run 63 later still used Python
  unittest and produced `NO TESTS RAN`.
- Lane 67/lane 76 must close the recurrence before a full PHPT gate is useful.

### 3. 221205Z shard abort class is repaired or ruled out

Required evidence:

- Full-gate harness setup no longer creates shard-local `run-tests.php`
  harnesses that can abort on missing shard directories such as
  `run-tests-harnesses/shard-03/ext/pdo/tests`.
- Every shard writes `results.txt`, `run-tests.log`, `stdout.log`, and
  `stderr.log`.
- The gate records the exact PHPT path list assigned to each shard.
- The aggregator compares expected PHPT paths against normalized current
  status paths and blocks the gate as incomplete if any expected row is absent.
- Evidence hashing covers per-shard logs/results and assignment lists, not only
  top-level summary files.

Current reason this is not yet restart-ready:

- The blocked 221205Z artifact has `18949` result rows against `21827` PHPT
  files, yet `aggregate-warnings.tsv` reports `missing_results=0`. That check
  is too weak for score movement.

### 4. Completed M0 reports are integrated or manifest-listed

Required evidence:

- Root `.harness/reports` contains the report artifacts, or one manifest lists
  branch, commit, artifact path, and integration status for each report-only
  branch.
- At minimum, include accepted score accounting, PASS-regression manifest,
  evidence integrity, late-row tags/crosscheck, late-overlap, source-diff risk,
  focused replay cookbook, standard array/string/filesystem/SPL/reflection
  shard reports, and first repair lane proposals.
- Source-changing branches remain excluded unless separately reviewed by the
  integrator with focused proof.

Current reason this is not yet restart-ready:

- Root `.harness/reports` currently has only two materialized reports. Lane 90
  is now in progress to integrate or manifest the completed M0 report backlog.

### 5. Focused replay samples classify absent rows

Required evidence:

- Accepted-vs-candidate replay samples are run or explicitly blocked with exact
  missing binary/artifact reasons.
- Rows cover at least standard arrays, standard strings, SPL, reflection, and
  one filesystem/http sample set.
- Each row is classified as control-plane absent, semantic failure,
  environment/SKIPIF blocker, or still passing.
- Replay commands name `PHPC_BIN`, wrapper path, php-src pin, row list, output
  directory, and all relevant environment variables.

Current reason this is not yet restart-ready:

- The dominant 221205Z symptom is `1136` absent rows. Treating those as semantic
  failures before replay would create broad, low-confidence implementation work.

### 6. Late-priority rows stay out of near-term repair selection

Required evidence:

- `eval` and variable-variable rows remain tagged as late-priority planning
  inputs and stay in denominator accounting.
- Any replay/repair backlog excludes those rows unless the issue is gate
  infrastructure rather than language implementation.

Current reason this matters:

- Only `5` of the `1166` 221205Z regressions overlap late-priority tags, so
  pursuing `eval` or variable-variable support cannot repair the current
  blocker.

## Full-Gate Acceptance Criteria

A restarted full pinned public PHPT gate may move score only if all of these
hold:

- The source commit, php-src pin, wrapper path, command shape, denominator, and
  artifact directory are recorded.
- The scoring commit is not a stale dirty checkout and includes the verified
  runtime and harness/control-plane repairs.
- `public-comparable-score.tsv`, `counts.tsv`,
  `current-passes.normalized.txt`, `current-status.normalized.tsv`,
  `all-results.txt`, `pass-regression-summary.tsv`, and
  `regressions-from-latest-published-passes.txt` are present.
- Expected PHPT rows are reconciled against normalized current-status rows.
- Per-shard results, logs, stdout/stderr, assignment lists, and top-level score
  files are hash-covered.
- Latest-public PASS regressions are `0`, or every nonzero row has an
  auditor-accepted adjudication record.
- Any candidate-only PASS improvements are reported as useful signal, not as an
  offset against latest-public PASS regressions.

## Stop Conditions

Stop the gate and keep public score unchanged if any of these occur:

- `php_runtime --lib` fails on the scoring commit.
- The scheduler selects Python unittest or any zero-test command for this
  Rust/PHP repository.
- A shard exits before writing its requested `run-tests.log`.
- Any expected PHPT path is absent from normalized current status.
- The evidence hash manifest omits shard-local logs/results needed for audit.
- Latest-public PASS regressions are nonzero and not auditor-adjudicated.

## Next Deterministic Actions

1. Close lane 66/lane 75 runtime repair with proof on the scoring commit.
2. Close lane 67/lane 76 command-routing repair with scheduler-visible proof.
3. Close lane 77/lane 78 shard-abort root cause and smoke proof.
4. Finish lane 90 report integration/manifesting.
5. Run or explicitly block lanes 91 and 92 focused replay samples.
6. Only then schedule a new full pinned PHPT gate.
