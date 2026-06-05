# Zero-Regression Gate Preflight Command Checklist

Owner: developer-433
Lane: work_lanes#146
Generated: 2026-06-05T10:02Z

Scope: read-only M0 gate preflight. No compiler, runtime, harness, dashboard,
or php-src source files were edited. No full PHPT gate was run. No public score
movement is claimed.

## Decision

Do not start a full pinned zero-regression PHPT gate yet.

The authoritative score state remains:

| Metric | Value | Evidence |
| --- | ---: | --- |
| Accepted public PHPT score | `7873 / 20294 = 38.79%` | `metric_samples.id=7`, accepted `public-comparable-score.tsv`, `PLAN.md` |
| Blocked 221205Z candidate | `7197 / 20294 = 35.46%` | `metric_samples.id=8`, candidate `public-comparable-score.tsv` |
| Latest-public PASS regressions | `1166` | candidate `pass-regression-summary.tsv`, `regressions-from-latest-published-passes.txt` |

The smallest deterministic path is not another full gate. It is:

1. Close the post-fix command-selection recurrence from `test_runs#215`.
2. Integrate or explicitly accept the runtime merge prerequisites for the
   `php_runtime --lib` expectation cluster.
3. Land the shard harness directory-layout fix proven by the shard smoke.
4. Restore or rebuild durable accepted/candidate `PHPC_BIN` binaries for
   focused replay.
5. Clear resource and cleanliness stop conditions.
6. Only then run a scoreable full gate with expected-path reconciliation and
   shard-local artifact hashing.

`eval` and variable-variable rows remain late-priority and are not part of this
preflight path.

## Blocking State

| Blocker | Current Evidence | Required Before Gate |
| --- | --- | --- |
| Public score regression | Candidate has `1166` PASS regressions: `1136 ABSENT`, `27 FAILED`, `3 BORKED`. | A new candidate must have zero latest-public PASS regressions, or every remaining row must be auditor-adjudicated. |
| Command selection recurrence | `.harness` focused proof passed in `test_runs#204/#205`, but later `test_runs#215` still ran `python -m unittest discover -s tests -v` and zero tests. Lane 143 owns this recurrence. | A scheduler-visible run after lane 143 must select `tools/run-tests.sh` or another documented nonzero project command. |
| Runtime merge blocker | Bug report #1 remains open. In this worktree, `e04e3df9` and `7a17b7e` are not ancestors of `HEAD`; do not assume the runtime pair is integrated here. | Integrator must merge or explicitly supersede the canonical `work/developer-120` then `work/developer-124` sequence and recheck `php_runtime --lib`. |
| Shard abort/control plane | Lane 78 smoke proves copied `run-tests.php` aborts on `ext/pdo/tests`, while linking `ext` from php-src avoids the abort. | Full-gate harness must either invoke `$PHP_SRC/run-tests.php` directly or link required php-src roots into each shard harness. |
| Missing historical replay binaries | Historical accepted and candidate `/tmp/.../cargo-target/release/phpc` paths are absent. | Rebuild release binaries from accepted `0b917f67...` and candidate `56fe9377...`, or produce a durable current scorer binary manifest. |
| Disk floor | `df -h / /tmp /dev/shm` observed `/` and `/tmp` at `21G` free. Prior readiness floor is `35 GiB`. | Do not run full PHPT gate until `/` and `/tmp` have at least `35 GiB` free. |
| Dirty/stale root | Root checkout `/home/claude/php-to-native-compiler` is dirty and divergent; this lane worktree is clean at `8381ad999b89`. | Score from a clean, current, intended source checkout; do not score from the dirty root. |
| Evidence completeness | 221205Z archived `current-status.normalized.tsv` has `18940` rows while `all-results.txt` has `18949`; shard-local assignment files were not fully archived. | Gate must fail if any expected PHPT row is absent from normalized status, and hash top-level plus shard-local artifacts. |

## Required Evidence Paths

Accepted baseline:

- `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67`
- Source/public head: `0b917f67a37d9ca9779d77f87173b628431c2425`
- Public score artifact: `public-comparable-score.tsv` reports `7873 / 20294 = 38.79%`
- Baseline PASS set: `current-passes.normalized.txt`

Blocked candidate:

- `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377`
- Source/public head: `56fe9377fb46be00db5fdd30c966fdba406dc581`
- Public score artifact: `public-comparable-score.tsv` reports `7197 / 20294 = 35.46%`
- Regression list: `regressions-from-latest-published-passes.txt`
- Regression summary: `pass-regression-summary.tsv`
- Candidate status files: `current-status.normalized.tsv`, `all-results.txt`

Shared inputs:

- php-src checkout: `/home/claude/php-src-phpt`
- php-src pin: `f97ff597429a2fe633665a7e02d97c8077f9f90f`
- Wrapper: `/home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper`
- Denominator: `20294`

Read before launch:

- `.harness/reports/full-gate-readiness-after-shard-fix-dev119.md`
- `.harness/reports/phpt-binary-wrapper-availability-recheck-dev305.md`
- `.harness/reports/run62-runtime-candidate-merge-prereqs-dev308.md`
- `.harness/reports/221205Z-regression-status-summary-refresh-dev313.md`
- `/home/claude/php-to-native-compiler/.harness/reports/221205Z-direct-failed-borked-triage.md`
- `/home/claude/php-to-native-compiler/.harness/reports/absent-row-rerun-prioritizer-dev118.md`
- `/home/claude/php-to-native-compiler/.harness/reports/221205Z-shard-rerun-smoke-dev116.md`

## Minimal Command Sequence

### Phase 0: Read-Only Admission Snapshot

Run these first. They are cheap and should not launch a gate.

```sh
git status --short --branch
git rev-parse HEAD
git -C /home/claude/php-src-phpt rev-parse HEAD
test -x /home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper
test -x /home/claude/php-src-phpt/run-tests.php
df -h / /tmp /dev/shm
uptime
```

Expected current results:

- php-src must be `f97ff597429a2fe633665a7e02d97c8077f9f90f`.
- wrapper and `run-tests.php` must be executable.
- `/` and `/tmp` must be at least `35 GiB` free; current `21G` is not enough.

### Phase 1: Control-Plane Selector Closure

Do not proceed while the latest test-loop observation is still
`python -m unittest discover -s tests -v`.

```sh
python3 -m unittest discover -s /home/claude/php-to-native-compiler/.harness/tests -v

python3 - <<'PY'
from pathlib import Path
from llm_harness.testing_loop import discover_test_command
root = Path('/home/claude/php-to-native-compiler')
print(discover_test_command(root))
PY
```

Acceptance:

- focused harness tests pass with nonzero test count;
- selector dry run prints `['tools/run-tests.sh']`;
- a later scheduler/test-loop record supersedes `test_runs#215` with a nonzero
  project command, or manager/auditor explicitly accepts lane 143 closure.

### Phase 2: Runtime Merge Closure

The report-only preflight observed that the two canonical runtime commits are
not ancestors of this lane worktree. The integration target must prove its own
state.

```sh
git merge-base --is-ancestor e04e3df9a49f3a1cce20764279bc83cc81a48ebf HEAD
git merge-base --is-ancestor 7a17b7eee5edb4ec2f2a12aa01d8ffddf2793d90 HEAD

umask 0002
export CARGO_TARGET_DIR=/dev/shm/phpc-target-runtime-preflight
export CARGO_BUILD_JOBS=1
export CARGO_INCREMENTAL=0
export RUST_TEST_THREADS=1
cargo test -p php_runtime --lib -- --test-threads=1
```

Acceptance:

- both merge-base checks return success, or the integrator records an explicit
  superseding decision;
- `php_runtime --lib` is green on the exact scoring commit.

### Phase 3: Shard Harness Redirect Smoke

Before any full gate, prove the redirected PDO abort cannot recur.

Minimum accepted harness fix:

- invoke `$PHP_SRC/run-tests.php` directly with shard-specific output/temp
  paths; or
- if copying `run-tests.php`, link at least `ext`, `Zend`, `tests`, and `sapi`
  from `$PHP_SRC` into every shard harness before running tests.

Smoke criteria:

- `ext/pdo_mysql/tests/common.phpt` and
  `ext/pdo_pgsql/tests/common.phpt` must not emit
  `ERROR: cannot open directory: .../ext/pdo/tests`;
- each run must write `run-tests.log` and `results.txt`;
- the smoke may use a tiny row list and must not be reported as score movement.

### Phase 4: Focused Replay Binary Restoration

Historical gate run roots under `/tmp/phpt-full-current-score-*` are gone.
Focused accepted-vs-candidate replay needs durable release binaries first.

```sh
# Accepted baseline source.
git worktree add /tmp/phpc-accepted-0b917f67 0b917f67a37d9ca9779d77f87173b628431c2425

# Blocked candidate source.
git worktree add /tmp/phpc-candidate-56fe9377 56fe9377fb46be00db5fdd30c966fdba406dc581

# Build each with unique target dirs, then record binary sha256 and one phpc run smoke.
```

Persist the rebuilt binaries outside disposable `/tmp/phpt-full-*` run roots,
with a manifest recording source commit, command, `CARGO_TARGET_DIR`, binary
path, sha256, and smoke result. Focused replay lanes can then use the replay
cookbook row lists with `PHPC_BIN` pointing at those durable binaries.

### Phase 5: Expected-Path Reconciliation

The next full gate must make completeness a hard preflight and aggregation
contract.

```sh
find "$PHP_SRC" -path "$PHP_SRC/.git" -prune -o -type f -name '*.phpt' -print |
  sort > "$RUN_ROOT/all-tests.txt"

find "$PHP_SRC/tests/security" -type f -name 'open_basedir_*.phpt' -print |
  sort > "$RUN_ROOT/serialized-openbasedir.tests"

awk 'NR==FNR{skip[$0]=1; next} !($0 in skip)' \
  "$RUN_ROOT/serialized-openbasedir.tests" \
  "$RUN_ROOT/all-tests.txt" > "$RUN_ROOT/sharded-tests.txt"
```

During aggregation:

```sh
awk -F '\t' '{ print $2 }' "$RUN_ROOT/current-status.normalized.tsv" |
  sort -u > "$RUN_ROOT/status-paths.txt"

comm -23 "$RUN_ROOT/expected-paths.normalized.txt" "$RUN_ROOT/status-paths.txt" \
  > "$RUN_ROOT/missing-normalized-status.txt"

test "$(wc -l < "$RUN_ROOT/missing-normalized-status.txt")" -eq 0
```

Acceptance:

- every expected PHPT path has a normalized status;
- every shard has `shard-XX.tests`, `results.txt`, `run-tests.log`,
  `stdout.log`, `stderr.log`, and exit status;
- `evidence-files.sha256` covers top-level and shard-local files.

### Phase 6: Full Gate Launch

Launch only after phases 0-5 pass and the source checkout is clean.

Required environment shape:

```sh
export CARGO_BUILD_JOBS=1
export CARGO_INCREMENTAL=0
export CARGO_TARGET_DIR="$RUN_ROOT/cargo-target"
export PHPC_BIN="$RUN_ROOT/cargo-target/release/phpc"
export PHPC_PHPT_TIMEOUT_SECONDS=55
export PHPC_PHPT_KILL_AFTER_SECONDS=5
export PHPT_SYSTEM_PHP=php
export NO_INTERACTION=1
export TEST_PHP_EXECUTABLE=/home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper
export TEST_PHP_SRCDIR="$RUN_ROOT/php-src"
export TMPDIR="$RUN_ROOT/tmp"
export TEMP="$TMPDIR"
export TMP="$TMPDIR"
```

Stop immediately if:

- `git status --porcelain` is nonempty in the scoring source checkout;
- php-src pin differs from `f97ff597429a2fe633665a7e02d97c8077f9f90f`;
- wrapper or `run-tests.php` is not executable;
- free disk on `/` or `/tmp` is below `35 GiB`;
- scheduler still selects the zero-test Python command;
- runtime merge proof is missing;
- shard redirect smoke has not passed;
- expected-path reconciliation or shard artifact hashing is not implemented.

## Current Direct Regression Priorities

Once control-plane and binary blockers are closed, use the existing triage order
instead of opening broad product lanes:

1. Direct readonly/internal property diagnostics and property-hook/interface
   metadata: highest-yield explicit `FAILED` bucket.
2. SKIPIF constant exposure for the `3 BORKED` rows:
   `INTL_ICU_VERSION`, `ZEND_THREAD_SAFE`, and `PCRE_JIT_SUPPORT`.
3. Object lifecycle/destructor/iterator lifetime rows.
4. Assertion/throwable formatting and serialization rows.
5. One-row inheritance diagnostic and opcache SCCP rows.

For the `1136 ABSENT` rows, follow the absent-row rerun order:

1. redirect smoke proof;
2. shard-04 absent rows (`307`);
3. shard-03 absent rows (`199`);
4. shard-05 (`297`);
5. shard-06 (`188`);
6. shard-01 (`74`);
7. shard-02 (`71`).

Treat absent rows as incomplete evidence until a rerun gives each row a
candidate status.

## Commands Run For This Report

MCP memory writes/queries were attempted first, but several returned
`database is locked`. A Python sqlite read-only immutable fallback was used for
lane/message/metric/bug/test-run snapshots.

Read-only checks:

```sh
sed -n '1,260p' AGENTS.md
sed -n '1,260p' docs/PROGRESS.md
sed -n '1,260p' docs/ARCHITECTURE.md
sed -n '1,260p' docs/SUPPORT.md
sed -n '1,260p' README.md
sed -n '1,240p' docs/LOOP_MEMORY.md
sed -n '1,220p' docs/OPERATIONS.md
sed -n '1,260p' /home/claude/php-to-native-compiler/PLAN.md
sed -n '1,260p' .harness/reports/full-gate-readiness-after-shard-fix-dev119.md
sed -n '1,240p' .harness/reports/phpt-binary-wrapper-availability-recheck-dev305.md
sed -n '1,220p' .harness/reports/run62-runtime-candidate-merge-prereqs-dev308.md
sed -n '1,220p' .harness/reports/221205Z-regression-status-summary-refresh-dev313.md
sed -n '1,220p' /home/claude/php-to-native-compiler/.harness/reports/221205Z-direct-failed-borked-triage.md
sed -n '1,220p' /home/claude/php-to-native-compiler/.harness/reports/absent-row-rerun-prioritizer-dev118.md
sed -n '1,220p' /home/claude/php-to-native-compiler/.harness/reports/221205Z-shard-rerun-smoke-dev116.md
df -h / /tmp /dev/shm
uptime
nproc
git status --short --branch
git -C /home/claude/php-src-phpt rev-parse HEAD
test -x /home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper
test -x /home/claude/php-src-phpt/run-tests.php
wc -l "$CAND/regressions-from-latest-published-passes.txt" "$CAND/current-status.normalized.tsv" "$CAND/all-results.txt"
```

No cargo build, no focused replay, and no full PHPT gate were run.
