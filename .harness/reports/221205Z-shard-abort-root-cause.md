# 221205Z Shard Abort Root Cause

Owner: developer-278
Lane: 69
Mode: read-only report; no compiler/runtime source edits; no full PHPT gate.

## Decision

The 221205Z candidate remains blocked. The accepted public score is still
`7873 / 20294 = 38.79%`; the blocked candidate remains
`7197 / 20294 = 35.46%` with `1166` latest-public PASS regressions.

The shard-03 and shard-04 aborts are control-plane evidence failures, not
compiler/runtime feature failures. The archived gate copied `run-tests.php`
into per-shard harness directories, but did not mirror the php-src tree layout
under those harness directories. PHPT `REDIRECTTEST` code that uses `__DIR__`
therefore resolved paths under:

`/tmp/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/run-tests-harnesses/shard-0{3,4}`

instead of the real php-src checkout. Both aborted when redirected PDO driver
tests tried to enumerate `ext/pdo/tests` under that incomplete harness root.

## Primary Evidence

Candidate evidence directory:

`/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377`

Key archived inputs:

| Item | Evidence |
| --- | --- |
| public/source head | `56fe9377fb46be00db5fdd30c966fdba406dc581` in `current-score-gate-preflight.tsv` |
| php-src pin | `f97ff597429a2fe633665a7e02d97c8077f9f90f` in `environment.txt`; confirmed by `/home/claude/php-src-phpt` |
| wrapper | `/home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper` |
| PHPT count | `all_phpt_files=21827`, `sharded_phpt_files=21780`, `serialized_openbasedir_phpt_files=47` |
| regression count | `pass-regression-summary.tsv` reports `1166` |
| row count mismatch | `all-results.txt` has `18949` rows, below `21827` recorded PHPT files |

Shard files:

| Shard | `results.txt` rows | `run-tests.log` | Exit | Terminal evidence |
| --- | ---: | --- | ---: | --- |
| `shard-01` | `3630` | present | `1` | reached `Report saved to:` |
| `shard-02` | `3630` | present | `1` | reached `Report saved to:` |
| `shard-03` | `2114` | missing | `1` | `ERROR: cannot open directory: .../run-tests-harnesses/shard-03/ext/pdo/tests` |
| `shard-04` | `2268` | missing | `1` | `ERROR: cannot open directory: .../run-tests-harnesses/shard-04/ext/pdo/tests` |
| `shard-05` | `3630` | present | `1` | reached `Report saved to:` |
| `shard-06` | `3630` | present | `1` | reached `Report saved to:` |
| `serial-openbasedir` | `47` | present | `1` | reached `Report saved to:` |

`aggregate-warnings.tsv` reports `missing_results	0`, but that only means
every shard produced a `results.txt` file. It did not detect partial shard
execution or missing `run-tests.log`.

## Root Cause

`run_gate.sh` prepares each shard harness like this:

```sh
prepare_run_tests_harness() {
  local harness_dir="$1"
  local aux
  install -d -m 700 "$harness_dir"
  cp "$PHP_SRC/run-tests.php" "$harness_dir/run-tests.php"
  for aux in .github build scripts; do
    if [ -e "$PHP_SRC/$aux" ] && [ ! -e "$harness_dir/$aux" ]; then
      ln -s "$PHP_SRC/$aux" "$harness_dir/$aux"
    fi
  done
}
```

It then runs:

```sh
cd "$PHP_SRC"
php "$shard_harness/run-tests.php" -q -n -p "$WRAPPER" -r "$list" \
  -W "$shard_dir/results.txt" -s "$shard_dir/run-tests.log" \
  --no-color --set-timeout 65 --temp-source "$PHP_SRC" \
  --temp-target "$shard_tmp"
```

The copied `run-tests.php` changes `__DIR__` from the php-src root to the
per-shard harness directory. That matters because `run-tests.php` evaluates
PHPT `REDIRECTTEST` sections directly. The pinned php-src files show the
affected redirect forms:

| File | Redirect target |
| --- | --- |
| `/home/claude/php-src-phpt/ext/pdo_mysql/tests/common.phpt` | `'TESTS' => __DIR__.'/ext/pdo/tests'` |
| `/home/claude/php-src-phpt/ext/pdo_pgsql/tests/common.phpt` | `'TESTS' => __DIR__ . '/ext/pdo/tests'` |
| `/home/claude/php-src-phpt/ext/pdo_odbc/tests/common.phpt` | `'TESTS' => 'ext/pdo/tests'` |

The ODBC redirect uses a relative path and reached its redirected section.
The MySQL and PgSQL redirects use `__DIR__`; under the copied harness, they
look for `run-tests-harnesses/shard-03/ext/pdo/tests` and
`run-tests-harnesses/shard-04/ext/pdo/tests`.

Those paths were not created. The actual pinned php-src checkout has
`/home/claude/php-src-phpt/ext/pdo/tests` with `117` top-level PHPT files.

Direct abort lines:

- `shard-03/stdout.log:2370-2372`: aborts while running
  `ext/pdo_mysql/tests/common.phpt`, ending with
  `ERROR: cannot open directory: .../run-tests-harnesses/shard-03/ext/pdo/tests`.
- `shard-04/stdout.log:2562-2564`: aborts while running
  `ext/pdo_pgsql/tests/common.phpt`, ending with
  `ERROR: cannot open directory: .../run-tests-harnesses/shard-04/ext/pdo/tests`.
- `/home/claude/php-src-phpt/run-tests.php:1023` is the `opendir($dir) or
  error("cannot open directory: $dir")` site.

## PASS-Regression Impact

`regressions-from-latest-published-passes.txt` has `1166` rows.
Only `30` have direct candidate statuses: `27 FAILED` and `3 BORKED`.
The other `1136` are absent from candidate normalized status.

Reconstructing shard assignment from pinned `/home/claude/php-src-phpt` and
the archived round-robin rule in `run_gate.sh` gives:

| Planned shard | Regression rows | Direct candidate status | Absent rows |
| --- | ---: | --- | ---: |
| `01` | `77` | `3 FAILED` | `74` |
| `02` | `75` | `4 FAILED` | `71` |
| `03` | `207` | `7 FAILED`, `1 BORKED` | `199` |
| `04` | `312` | `5 FAILED` | `307` |
| `05` | `301` | `4 FAILED` | `297` |
| `06` | `194` | `4 FAILED`, `2 BORKED` | `188` |

The two aborted shards therefore account for `506` absent regression rows
whose candidate behavior is ambiguous from current evidence. The remaining
`630` absent rows are broader incomplete aggregate evidence and still need a
strict expected-path reconciliation; they are not proven semantic failures.

The evidence directory did not archive `all-tests.txt`, `sharded-tests.txt`,
or `shard-01.tests` through `shard-06.tests`; only
`serialized-openbasedir.tests` was saved. That is another restartability gap:
shard membership has to be reconstructed rather than read from the scored
artifact.

## Deterministic Repair Path

Do not move public score from `7873 / 20294`.

The next deterministic repair is harness-only:

1. Change the full-gate shard harness so `run-tests.php` either runs from the
   real php-src root or the per-shard harness mirrors enough of the php-src
   root layout for `__DIR__`-based `REDIRECTTEST` code. The smallest likely
   patch is to symlink php-src root directories such as `ext`, `Zend`,
   `tests`, and `sapi` into each `run-tests-harnesses/shard-*` directory.
   A broader, cleaner option is to avoid copying `run-tests.php` and invoke
   `$PHP_SRC/run-tests.php` directly while keeping shard-specific temp/result
   paths.
2. Add pre-aggregation checks that fail the gate when any shard lacks
   `run-tests.log`, any shard stdout contains `ERROR: cannot open directory`,
   or normalized status does not cover every expected PHPT path.
3. Archive `all-tests.txt`, `sharded-tests.txt`, every `shard-XX.tests`, and
   all shard-local logs/results/exit files in `evidence-files.sha256`.
4. Run the lane-78 smoke, not a full gate: construct a minimal shard list with
   `ext/pdo_mysql/tests/common.phpt` and `ext/pdo_pgsql/tests/common.phpt`,
   run through the repaired harness layout with the existing wrapper and
   pinned php-src checkout, and prove both no longer abort on
   `run-tests-harnesses/.../ext/pdo/tests`.
5. Only after that smoke and the lane-8 command-selection blocker are accepted
   should a full pinned PHPT gate be restarted.

## Commands Run

No full PHPT gate was run. Commands were artifact reads and low-CPU
reconstruction:

```sh
rg -n "run-tests-harnesses|prepare_run_tests_harness|temp-source|aggregate" \
  "$CAND/run_gate.sh"

nl -ba "$CAND/run_gate.sh" | sed -n '560,590p;690,735p;750,835p;850,872p'

cat "$CAND/shard-exit-codes.tsv"

for d in shard-01 shard-02 shard-03 shard-04 shard-05 shard-06 serial-openbasedir; do
  test -f "$CAND/$d/run-tests.log" && echo "$d present" || echo "$d missing"
done

rg -n "Report saved to:|ERROR: cannot open directory" \
  "$CAND"/shard-*/stdout.log "$CAND"/serial-openbasedir/stdout.log

python - <<'PY'
# Count all-results/current-status/current-passes/regressions, per-shard result
# rows, direct regression statuses, and reconstructed regression shard mapping
# from /home/claude/php-src-phpt plus run_gate.sh round-robin assignment.
PY

nl -ba /home/claude/php-src-phpt/run-tests.php | sed -n '990,1035p'
sed -n '1,120p' /home/claude/php-src-phpt/ext/pdo_mysql/tests/common.phpt
sed -n '1,120p' /home/claude/php-src-phpt/ext/pdo_pgsql/tests/common.phpt
sed -n '1,120p' /home/claude/php-src-phpt/ext/pdo_odbc/tests/common.phpt

sha256sum -c "$CAND/evidence-files.sha256"
```

SQLite inspection used Python's standard `sqlite3` module because the local
`sqlite3` CLI and MCP memory tools were unavailable in this session.
