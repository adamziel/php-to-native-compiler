# 221205Z Evidence Integrity Audit

Audit worker: developer-97
Lane: 27, replacement for developer-84
Scope: read-only artifact and SQLite inspection; no compiler/runtime source edits; no full PHPT gate

## Decision

The `phpt-full-current-score-20260604T221205Z` candidate must not move the
public score. The accepted public score remains `7873/20294`. The blocked
candidate score is `7197/20294`, and the candidate has `1166` normalized
PASS regressions against the latest published PASS baseline.

The candidate is also not evidence-clean enough to promote: shard-03 and
shard-04 stopped with `ERROR: cannot open directory` under their copied
`run-tests.php` harnesses, both lack per-shard `run-tests.log`, and the
aggregate contains `18949` result rows versus `21827` PHPT files recorded in
`environment.txt`. This is compatible with keeping the candidate blocked; it
is not compatible with publishing a higher public score.

## Reproducibility Inputs

Artifact directory:

`/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377`

Wrapper path:

`/home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper`

Archived gate wrapper:

`/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/run_gate.sh`

Recorded inputs from `run_gate.sh`, `environment.txt`, and preflight:

| Input | Value |
| --- | --- |
| run id | `phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377` |
| run root | `/tmp/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377` |
| source repo seed | `/home/claude/supervised-php-compiler/state/worktrees/batch024-current-supervisor-20260601T093225` |
| public head | `56fe9377fb46be00db5fdd30c966fdba406dc581` |
| source head | `56fe9377fb46be00db5fdd30c966fdba406dc581` |
| php-src checkout seed | `/home/claude/php-src-phpt` |
| php-src pin | `f97ff597429a2fe633665a7e02d97c8077f9f90f` |
| baseline passes | `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67/current-passes.normalized.txt` |
| PHPC_BIN | `/tmp/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/cargo-target/release/phpc` |
| denominator source | `PINNED_RUNNABLE=20294` in `run_gate.sh` and `pinned_public_runnable=20294` in `environment.txt` |
| shard count | `6` plus serialized `tests/security/open_basedir_*.phpt` |

Environment variables recorded by the gate:

| Variable | Value |
| --- | --- |
| `CARGO_BUILD_JOBS` | `1` |
| `CARGO_INCREMENTAL` | `0` |
| `CARGO_TARGET_DIR` | `/tmp/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/cargo-target` |
| `PHPC_BIN` | `/tmp/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/cargo-target/release/phpc` |
| `PHPC_PHPT_TIMEOUT_SECONDS` | `55` |
| `PHPC_PHPT_KILL_AFTER_SECONDS` | `5` |
| `NO_INTERACTION` | `1` |
| `PHPT_SYSTEM_PHP` | `php` |

Command shape from `run_gate.sh`:

```sh
git clone --no-local "$SOURCE_REPO_SEED" "$SOURCE_REPO"
git -C "$SOURCE_REPO" checkout -f "$SOURCE_HEAD"
(cd "$SOURCE_REPO" && cargo build --release -p phpc)
git clone --no-checkout "$PHP_SRC_CHECKOUT" "$PHP_SRC"
git -C "$PHP_SRC" checkout -f "$PHP_SRC_PIN"
php "$shard_harness/run-tests.php" -q -n -p "$WRAPPER" -r "$list" \
  -W "$shard_dir/results.txt" -s "$shard_dir/run-tests.log" \
  --no-color --set-timeout 65 --temp-source "$PHP_SRC" \
  --temp-target "$shard_tmp"
php "$serial_harness/run-tests.php" -q -n -p "$WRAPPER" \
  -r "$RUN_ROOT/serialized-openbasedir.tests" \
  -W "$serial_dir/results.txt" -s "$serial_dir/run-tests.log" \
  --no-color --set-timeout 65 --temp-source "$PHP_SRC" \
  --temp-target "$serial_tmp"
```

## Evidence Integrity Checks

Expected assignment files all exist and are non-empty:

| File | Size |
| --- | ---: |
| `run_gate.sh` | `32534` |
| `environment.txt` | `2883` |
| `evidence-files.sha256` | `8327` |
| `current-score-gate-preflight.tsv` | `946` |
| `public-comparable-score.tsv` | `55` |
| `pass-regression-summary.tsv` | `239` |
| `counts.tsv` | `123` |

Hash verification:

- `sha256sum -c evidence-files.sha256` returned `OK` for every listed entry.
- The manifest has `37` entries. It covers top-level evidence files only.
- Top-level files not covered by the manifest are `evidence-files.sha256`
  itself and `signal-sentry.stop`.
- The manifest does not recursively cover `shard-01` through `shard-06` or
  `serial-openbasedir` subdirectory logs/results. This follows the archived
  `sha256sum "$EVIDENCE_DIR"/*` command shape and is a coverage limitation.

Empty top-level files:

- `invalid-proof-markers.txt` is empty, matching
  `invalid-proof-marker-summary.tsv` value `invalid_marker_hits=0`.
- `php-src-status.txt` and `source-status.txt` are empty, matching clean
  checkout status.
- `runner.stdout.log`, `runner.stderr.log`, `signal-sentry.stdout.log`, and
  `signal-sentry.stderr.log` are empty.
- `signal-sentry.stop` is empty and not covered by the hash manifest.

Staleness check:

- `run_gate.sh` and `current-score-gate-preflight.tsv` were created at
  `2026-06-04T22:12:05Z`.
- `environment.txt` was written at `2026-06-04T22:18:38Z`.
- aggregate results and score files were written around
  `2026-06-04T22:29:33Z` to `2026-06-04T22:29:34Z`.
- The timestamp order is consistent with one archived gate run.

Incomplete or weak evidence concerns:

- All six shard exit codes are `1`; serialized open_basedir exit is also `1`.
- `shard-03/run-tests.log` and `shard-04/run-tests.log` are missing even
  though the command requested `-s "$shard_dir/run-tests.log"`.
- `shard-03/stdout.log` ends with:
  `ERROR: cannot open directory: .../run-tests-harnesses/shard-03/ext/pdo/tests`.
- `shard-04/stdout.log` ends with:
  `ERROR: cannot open directory: .../run-tests-harnesses/shard-04/ext/pdo/tests`.
- `counts.tsv` accounts for `18949` result rows, matching `all-results.txt`.
  `environment.txt` records `all_phpt_files=21827`, so `2878` PHPT files are
  not represented in the aggregate result rows.
- `aggregate-warnings.tsv` says `missing_results=0`, but that only proves
  expected result files existed; it does not prove all listed PHPT rows ran to
  completion.

Score file agreement:

- `counts.tsv`: `passed=7197`, `runnable=16058`, raw percent `44.82`.
- `public-comparable-score.tsv`: `passed=7197`,
  `pinned_runnable=20294`, public percent `35.46`.
- `pass-regression-summary.tsv`: baseline normalized passes `7869`, current
  normalized unique passes `7196`, PASS regressions `1166`.
- The raw passed count and public-comparable passed count agree at `7197`.
  The regression comparison uses unique normalized pass rows and reports
  `7196`; that is a metric-definition difference from raw counts, not a hash
  mismatch. The blocking regression count is `1166`.

## SQLite State

Relevant harness database findings from
`/home/claude/php-to-native-compiler/.harness/harness.sqlite3`:

- `metric_samples`: accepted public PHPT passes are `7873/20294` at `38.79%`.
- `metric_samples`: blocked 221205Z candidate passes are `7197/20294` at
  `35.46%`.
- `work_lanes.id=27`: lane is assigned to `work/developer-97` after
  developer-84 was requeued.
- `agents.developer-84`: `current_status=crashed`, `ended_at=2026-06-05T00:10:12+00:00`,
  notes say no lane 27 evidence report was produced.
- `agents.developer-97`: current worker for lane 27.
- No `test_runs` rows were found for commands containing `221205Z` or
  `phpt-full-current-score`.

## Low-CPU Commands Used

No full PHPT gate was run. Commands used were low-CPU artifact inspection and
SQLite metadata reads/writes:

```sh
rg --files ...
sed -n ...
tail -n ...
stat -c ...
find ... -maxdepth ...
sha256sum -c evidence-files.sha256
python3 - <<'PY'  # sqlite3 module queries/agent status update
```

The local `sqlite3` CLI was unavailable, so Python's standard `sqlite3` module
was used only for database inspection and status/event updates.

## Next Deterministic Action

Before the public score can move, a worker must repair or rerun the pinned
gate evidence so the shard harnesses do not abort and the artifact set has
complete, hash-covered evidence for the scored rows. After that, the `1166`
normalized PASS regressions must be classified and adjudicated. Only a
regression-free, reproducible candidate should replace the accepted public
score.
