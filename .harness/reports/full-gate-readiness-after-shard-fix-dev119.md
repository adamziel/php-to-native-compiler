# Full Pinned Gate Readiness After Shard Fix

Owner: developer-129
Lane: 88
Mode: read-only M1 checklist; no compiler/runtime source edits; no full PHPT
gate; public score unchanged.

## Current Decision

The next full pinned public PHPT gate is not ready to start yet.

The accepted public score remains `7873 / 20294 = 38.79%` at
`0b917f67a37d9ca9779d77f87173b628431c2425`. The
`phpt-full-current-score-20260604T221205Z` candidate remains blocked at
`7197 / 20294 = 35.46%` with `1166` latest-public PASS regressions.

As of this report:

- Runtime focused verification has fresh green evidence in harness DB
  `test_runs#75`: `cargo test -p php_runtime --lib -- --test-threads=1`
  passed `419/419` on commit `e04e3df9a49f3a1cce20764279bc83cc81a48ebf`.
- Runtime source repair is still tracked by lane 67 and should be integrated
  before scoring.
- Python zero-test command routing remains owned by lane 8 and is still a gate
  blocker until scheduler-visible proof exists.
- Shard-abort root cause remains owned by lane 69 and is still a gate blocker.
- The root filesystem currently has about `25G` available, below the
  candidate gate preflight minimum of `35 GiB`.

## Exact Source Requirements

A scoreable run must record all of the following before execution:

- `public_head` and `source_head` are the same clean Git commit.
- `git status --porcelain` is empty in the source checkout used to build
  `phpc`.
- The commit includes the accepted runtime fix or equivalent proof from lane 67.
- The commit includes the accepted harness command-routing fix from lane 8, or
  the scheduler/test-loop configuration demonstrably selects the documented
  command outside the source tree.
- The full-gate harness includes the shard-abort fix from lane 69 and any smoke
  proof from lane 78.
- Report-only commits may be present for audit artifacts, but they do not move
  score by themselves.

Do not score from the dirty root checkout. The current root
`/home/claude/php-to-native-compiler` is at `dc768f6b865c` but is dirty and
behind `origin/master` by `550` commits.

## Pinned PHPT Inputs

Use the same pinned inputs unless a manager/integrator deliberately updates the
public denominator:

- php-src checkout seed: `/home/claude/php-src-phpt`
- php-src pin: `f97ff597429a2fe633665a7e02d97c8077f9f90f`
- PHPT wrapper: `/home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper`
- pinned public runnable denominator: `20294`
- accepted regression baseline:
  `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67/current-passes.normalized.txt`

The wrapper must be executable, and `git -C /home/claude/php-src-phpt rev-parse
HEAD` must equal the php-src pin.

## Required Environment

Record these values in `environment.txt` and the preflight artifact:

- `CARGO_BUILD_JOBS=1`
- `CARGO_INCREMENTAL=0`
- `CARGO_TARGET_DIR=$RUN_ROOT/cargo-target`
- `PHPC_BIN=$RUN_ROOT/cargo-target/release/phpc`
- `PHPC_PHPT_TIMEOUT_SECONDS=55`
- `PHPC_PHPT_KILL_AFTER_SECONDS=5`
- `PHPT_SYSTEM_PHP=php`
- `NO_INTERACTION=1`
- `TEST_PHP_EXECUTABLE=/home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper`
- `TEST_PHP_SRCDIR=$RUN_ROOT/php-src`
- `TMPDIR`, `TEMP`, and `TMP` under the run root, not shared worker
  directories

Build command shape:

```sh
cargo build --release -p phpc
```

PHPT shard command shape:

```sh
php "$shard_harness/run-tests.php" -q -n \
  -p "$WRAPPER" \
  -r "$list" \
  -W "$shard_dir/results.txt" \
  -s "$shard_dir/run-tests.log" \
  --no-color \
  --set-timeout 65 \
  --temp-source "$PHP_SRC" \
  --temp-target "$shard_tmp"
```

The serialized open_basedir command must use the same wrapper, source, timeout,
and log/result conventions.

## Required Artifact Outputs

Top-level evidence must include:

- `run_gate.sh`
- `environment.txt`
- `current-score-gate-preflight.tsv`
- `counts.tsv`
- `public-comparable-score.tsv`
- `pass-regression-summary.tsv`
- `current-status.normalized.tsv`
- `current-passes.normalized.txt`
- `baseline-passes.normalized.txt`
- `regressions-from-latest-published-passes.txt`
- `all-results.txt`
- `aggregate-warnings.tsv`
- `invalid-proof-marker-summary.tsv`
- `invalid-proof-markers.txt`
- `evidence-files.sha256`

Each shard directory, including `serial-openbasedir`, must include:

- PHPT assignment list used by that shard
- `results.txt`
- `run-tests.log`
- `stdout.log`
- `stderr.log`
- exit status

`evidence-files.sha256` must cover the shard-local files as well as the
top-level score files. The 221205Z evidence hash covered only top-level files;
that is not sufficient for restart acceptance.

## Regression Comparison Inputs

The gate must recompute normalized PASS regressions exactly as:

```sh
awk -F '\t' '$1=="PASSED" {print $2}' \
  "$EVIDENCE_DIR/current-status.normalized.tsv" |
  sort -u > "$EVIDENCE_DIR/current-passes.normalized.txt"

comm -23 \
  "$EVIDENCE_DIR/baseline-passes.normalized.txt" \
  "$EVIDENCE_DIR/current-passes.normalized.txt" \
  > "$EVIDENCE_DIR/regressions-from-latest-published-passes.txt"
```

Acceptance requires `wc -l regressions-from-latest-published-passes.txt` to be
`0`, unless every row has an auditor-accepted adjudication record.

Candidate-only PASS rows are useful implementation signal but do not offset
latest-public PASS regressions.

## Row Completeness Checks

The preflight or aggregation step must record the expected PHPT path set and
then prove all expected rows have a normalized status. This must be stricter
than the old `missing_results=0` check.

Minimum checks:

- count all PHPT files under the pinned php-src checkout
- record the sharded PHPT count and serialized open_basedir count
- save every shard assignment list
- compare expected paths against `current-status.normalized.tsv`
- fail the gate if any expected path is absent from normalized status
- fail the gate if any shard is missing `run-tests.log`

The 221205Z candidate had `18949` result rows while `environment.txt` recorded
`21827` PHPT files. That class of incomplete evidence must stop the restart.

## Disk And Build Guardrails

The 221205Z preflight required:

- `disk_available_gib >= 35`
- `memory_available_gib >= 30`
- `active_team_workers <= 9`

Current local observation for this report:

```text
/ and /tmp: 459G size, 415G used, 25G available, 95% used
/dev/shm: 22G size, 829M used, 22G available
```

Do not start a full gate while `/` or `/tmp` is below `35 GiB` available. If
builds use `/dev/shm` for focused tests, do not reuse that as the full-gate
release target unless the full artifact size is explicitly budgeted. No cleanup
should be performed by a worker unless a manager or integrator approves exact
paths to remove.

## Stop Conditions

Stop before launching the full gate if any of these are true:

- source checkout is dirty
- source commit is not the intended scored commit
- php-src pin differs from `f97ff597429a2fe633665a7e02d97c8077f9f90f`
- wrapper path is missing or not executable
- scheduler/test-loop still selects `python -m unittest discover -s tests -v`
- `cargo test -p php_runtime --lib -- --test-threads=1` is not green on the
  scoring commit
- disk available on `/` or `/tmp` is below `35 GiB`
- command routing, runtime repair, or shard-abort lanes are still in progress
  without accepted handoff evidence

Stop during or after the gate if any of these occur:

- a shard exits before writing `run-tests.log`
- any expected PHPT row is absent from normalized status
- hash coverage omits shard-local logs/results/assignment lists
- `pass_regressions` is nonzero and not auditor-adjudicated
- invalid proof markers are detected
- aggregate files disagree on source commit, php-src pin, denominator, or
  baseline path

## Readiness Checklist

Mark each item before scheduling:

- [ ] Lane 67 runtime source repair integrated or accepted on the scoring
  commit.
- [ ] Lane 66 post-fix runtime verification green on that same scoring commit.
- [ ] Lane 8 command-routing fix accepted with scheduler-visible proof.
- [ ] Lane 69 shard-abort root cause closed or handed off with a deterministic
  harness fix.
- [ ] Lane 78 shard rerun smoke confirms the missing-directory abort class
  would not recur.
- [ ] Focused replay/report lanes have classified representative absent rows,
  or have exact blocker reports for missing binaries/artifacts.
- [ ] Disk on `/` and `/tmp` is at least `35 GiB` free.
- [ ] Source checkout and php-src checkout are clean and pinned.
- [ ] Evidence hash plan covers top-level and shard-local artifacts.
- [ ] Expected PHPT path reconciliation is part of the gate, not a post-hoc
  manual audit.

Until all boxes are checked, keep the public score unchanged.
