# 221205Z Shard Rerun Smoke

Owner: developer-408
Lane: 78
Mode: read-only M0/M1 smoke report; no compiler/runtime source edits; no full
PHPT gate.

## Decision

The smallest deterministic smoke supports the lane-69 root-cause fix path:
the old copied `run-tests.php` harness layout reproduces the missing-directory
abort, while the same layout with `ext` linked from the pinned php-src checkout
lets both affected `REDIRECTTEST` rows enumerate redirected PDO tests and write
normal `run-tests.log` / `results.txt` artifacts.

This does not move the public PHPT score. The accepted public score remains
`7873 / 20294 = 38.79%`; the blocked `221205Z` candidate remains
`7197 / 20294 = 35.46%` with `1166` latest-public PASS regressions.

## Inputs

- Candidate evidence root:
  `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377`
- Gate script inspected:
  `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/run_gate.sh`
- Pinned php-src checkout: `/home/claude/php-src-phpt`
- php-src pin confirmed: `f97ff597429a2fe633665a7e02d97c8077f9f90f`
- Historical wrapper: `/home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper`
- Smoke root: `/tmp/phpt-shard-rerun-smoke-dev408`
- Test executable for this smoke: `/run/current-system/sw/bin/php`

Historical accepted/candidate `phpc` release binaries under `/tmp/phpt-full-*`
are no longer present, and this lane explicitly preferred no rebuild. The
smoke therefore used system PHP as `-p` to exercise `run-tests.php`
control-plane redirect behavior only. The counted smoke omits `-n` because
`php -n` disables `pdo_mysql` / `pdo_pgsql` in this local PHP and skips before
`REDIRECTTEST`; the scored gate's wrapper path is a different executable.

## Selected Paths

The two rows are the exact terminal rows observed in the blocked candidate
shards:

- `/home/claude/php-src-phpt/ext/pdo_mysql/tests/common.phpt`
- `/home/claude/php-src-phpt/ext/pdo_pgsql/tests/common.phpt`

Both contain `REDIRECTTEST` sections with `TESTS` set from `__DIR__`:

- `__DIR__.'/ext/pdo/tests'`
- `__DIR__ . '/ext/pdo/tests'`

When `run-tests.php` is copied to a shard harness, `__DIR__` points at that
harness, not at php-src. Linking `ext -> /home/claude/php-src-phpt/ext` into
the harness is enough for `opendir("$harness/ext/pdo/tests")` to succeed.

## Smoke Shape

Three harnesses were created under `/tmp/phpt-shard-rerun-smoke-dev408`:

| Harness | Layout | Purpose |
| --- | --- | --- |
| `old-harness` | copied `run-tests.php` plus `.github`, `build`, `scripts` links | Reproduce the archived shard-03/04 abort shape. |
| `fixed-mysql-harness` | old layout plus `ext` symlink | Prove `pdo_mysql` redirect can enumerate tests. |
| `fixed-pgsql-harness` | old layout plus `ext` symlink | Prove `pdo_pgsql` redirect can enumerate tests. |

Environment used for each counted run:

```sh
NO_INTERACTION=1
TEST_PHP_SRCDIR=/home/claude/php-src-phpt
TMPDIR=/tmp/phpt-shard-rerun-smoke-dev408/tmp-<case>
TEMP=$TMPDIR
TMP=$TMPDIR
```

Command shape:

```sh
cd /home/claude/php-src-phpt

timeout 25s php /tmp/phpt-shard-rerun-smoke-dev408/old-harness/run-tests.php \
  -q -p /run/current-system/sw/bin/php \
  -r /tmp/phpt-shard-rerun-smoke-dev408/mysql-common.tests \
  -W /tmp/phpt-shard-rerun-smoke-dev408/out-old/results.txt \
  -s /tmp/phpt-shard-rerun-smoke-dev408/out-old/run-tests.log \
  --no-color --set-timeout 15 \
  --temp-source /home/claude/php-src-phpt \
  --temp-target /tmp/phpt-shard-rerun-smoke-dev408/tmp-old/phpt-tmp

timeout 90s php /tmp/phpt-shard-rerun-smoke-dev408/fixed-mysql-harness/run-tests.php \
  -q -p /run/current-system/sw/bin/php \
  -r /tmp/phpt-shard-rerun-smoke-dev408/mysql-common.tests \
  -W /tmp/phpt-shard-rerun-smoke-dev408/out-mysql/results.txt \
  -s /tmp/phpt-shard-rerun-smoke-dev408/out-mysql/run-tests.log \
  --no-color --set-timeout 15 \
  --temp-source /home/claude/php-src-phpt \
  --temp-target /tmp/phpt-shard-rerun-smoke-dev408/tmp-mysql/phpt-tmp

timeout 90s php /tmp/phpt-shard-rerun-smoke-dev408/fixed-pgsql-harness/run-tests.php \
  -q -p /run/current-system/sw/bin/php \
  -r /tmp/phpt-shard-rerun-smoke-dev408/pgsql-common.tests \
  -W /tmp/phpt-shard-rerun-smoke-dev408/out-pgsql/results.txt \
  -s /tmp/phpt-shard-rerun-smoke-dev408/out-pgsql/run-tests.log \
  --no-color --set-timeout 15 \
  --temp-source /home/claude/php-src-phpt \
  --temp-target /tmp/phpt-shard-rerun-smoke-dev408/tmp-pgsql/phpt-tmp
```

## Results

| Case | Exit | Missing-directory hits | Redirect hits | `run-tests.log` | `results.txt` rows | Status counts |
| --- | ---: | ---: | ---: | --- | ---: | --- |
| Old copied harness, MySQL row | `1` | `1` | `0` | absent | `0` | none |
| Fixed harness, MySQL row | `1` | `0` | `1` | present | `127` | `6 PASSED`, `120 SKIPPED`, `1 FAILED` |
| Fixed harness, PgSQL row | `0` | `0` | `1` | present | `127` | `6 PASSED`, `120 SKIPPED`, `1 FAILED` |

Old-layout terminal line:

```text
ERROR: cannot open directory: /tmp/phpt-shard-rerun-smoke-dev408/old-harness/ext/pdo/tests
```

Fixed-layout terminal evidence:

```text
REDIRECT /tmp/phpt-shard-rerun-smoke-dev408/fixed-mysql-harness/ext/pdo/tests (MySQL [ext/pdo_mysql/tests/common.phpt]) done
Report saved to: /tmp/phpt-shard-rerun-smoke-dev408/out-mysql/run-tests.log

REDIRECT /tmp/phpt-shard-rerun-smoke-dev408/fixed-pgsql-harness/ext/pdo/tests (Postgres [ext/pdo_pgsql/tests/common.phpt]) done
Report saved to: /tmp/phpt-shard-rerun-smoke-dev408/out-pgsql/run-tests.log
```

The fixed runs still report normal PHPT outcomes from local system PHP and
missing database services. Those row outcomes are not compiler/runtime signal
for this lane; the control-plane proof is that `run-tests.php` no longer aborts
while opening the redirected directory and reaches normal report generation.

## Conclusion

For the shard-03/04 abort class observed in `221205Z`, a repaired shard harness
that exposes php-src root directories to copied `run-tests.php` would have
avoided the missing `run-tests-harnesses/shard-0{3,4}/ext/pdo/tests` abort.

The lowest-risk full-gate patch remains either:

1. invoke `$PHP_SRC/run-tests.php` directly instead of copying it, while keeping
   shard-specific result/temp paths; or
2. if copying is retained, link at least `ext`, `Zend`, `tests`, and `sapi`
   from `$PHP_SRC` into every shard harness before running PHPT rows.

The next scoreable PHPT gate should also archive `all-tests.txt`,
`sharded-tests.txt`, every `shard-XX.tests`, each shard's `run-tests.log`,
`stdout.log`, `stderr.log`, `results.txt`, and exit status, and should fail
aggregation if any expected PHPT path is absent from normalized status.
