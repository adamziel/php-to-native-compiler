# Focused SPL Replay Replacement Report

Lane 82 replacement artifact for stale `developer-109`, produced by
`developer-150`.

This is diagnostic/control-plane work only. It does not move the accepted
public PHPT score, does not edit compiler/runtime source, and does not run a
full PHPT gate.

## Inputs

- Integrated SPL evidence:
  `.harness/reports/221205Z-spl.md`
- Focused replay cookbook:
  `.harness/reports/focused-replay-cookbook.md`
- Accepted baseline evidence:
  `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67`
- Blocked candidate evidence:
  `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377`
- Shared wrapper:
  `/home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper`
- Shared php-src checkout:
  `/home/claude/php-src-phpt` at
  `f97ff597429a2fe633665a7e02d97c8077f9f90f`

Public metric context remains unchanged:

| Source | Passed | Pinned runnable | Percent |
| --- | ---: | ---: | ---: |
| Accepted public score | 7873 | 20294 | 38.79 |
| Blocked candidate score | 7197 | 20294 | 35.46 |

The candidate regression summary still reports 1166 latest-public PASS
regressions. The SPL report counted 137 `php-src/ext/spl/` rows in that
regression set.

## Selected Rows

I selected six representative SPL rows across the requested surfaces:
ArrayObject, autoloading, SplFileObject, SplObjectStorage, iterator utilities,
and SplFixedArray.

| Row | SPL surface | Accepted status | Candidate status | Classification |
| --- | --- | --- | --- | --- |
| `php-src/ext/spl/tests/ArrayObject/array_006.phpt` | ArrayObject / ArrayIterator | `PASSED` | `MISSING` | Absent/control-plane |
| `php-src/ext/spl/tests/autoloading/bug61697.phpt` | SPL autoloading | `PASSED` | `MISSING` | Absent/control-plane |
| `php-src/ext/spl/tests/SplFileObject/bug68479.phpt` | SplFileObject | `PASSED` | `MISSING` | Absent/control-plane |
| `php-src/ext/spl/tests/SplObjectStorage/bug69227.phpt` | SplObjectStorage | `PASSED` | `MISSING` | Absent/control-plane |
| `php-src/ext/spl/tests/DirectoryIterator_getBasename_basic_test.phpt` | Iterator utility | `PASSED` | `MISSING` | Absent/control-plane |
| `php-src/ext/spl/tests/SplFixedArray__construct_param_array.phpt` | SplFixedArray | `PASSED` | `MISSING` | Absent/control-plane |

Selected row count: 6.

Classification summary for selected rows:

| Bucket | Rows |
| --- | ---: |
| Accepted PASS, candidate absent/control-plane | 6 |
| Candidate semantic failure proven by row output | 0 |
| Candidate still passing | 0 |
| Replay blocked by missing historical binaries | 6 |

## Evidence Commands

Status join over saved artifacts:

```sh
ACC=/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67
CAND=/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377

cat > /tmp/lane82-spl-selected.rows <<'EOF'
php-src/ext/spl/tests/ArrayObject/array_006.phpt
php-src/ext/spl/tests/autoloading/bug61697.phpt
php-src/ext/spl/tests/SplFileObject/bug68479.phpt
php-src/ext/spl/tests/SplObjectStorage/bug69227.phpt
php-src/ext/spl/tests/DirectoryIterator_getBasename_basic_test.phpt
php-src/ext/spl/tests/SplFixedArray__construct_param_array.phpt
EOF

while IFS= read -r row; do
  printf '%s\n' "$row"
  awk -F '\t' -v p="$row" '$2==p { print "  accepted: " $1; found=1; exit }
    END { if (!found) print "  accepted: MISSING" }' "$ACC/current-status.normalized.tsv"
  awk -F '\t' -v p="$row" '$2==p { print "  candidate: " $1; found=1; exit }
    END { if (!found) print "  candidate: MISSING" }' "$CAND/current-status.normalized.tsv"
done < /tmp/lane82-spl-selected.rows
```

Observed result: all six selected rows are present as `PASSED` in accepted
`current-status.normalized.tsv` and absent from candidate
`current-status.normalized.tsv`.

Saved artifact availability:

```sh
test -x /home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper
test -x /home/claude/php-src-phpt/run-tests.php
test -x /tmp/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67/cargo-target/release/phpc
test -x /tmp/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/cargo-target/release/phpc
```

Observed result:

- Wrapper exists and is executable.
- `/home/claude/php-src-phpt/run-tests.php` exists and is executable.
- The historical accepted `PHPC_BIN` is missing.
- The historical candidate `PHPC_BIN` is missing.

Shard truncation checks:

```sh
CAND=/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377

test -e "$CAND/shard-03/run-tests.log" || echo shard-03-run-tests-log-absent
test -e "$CAND/shard-04/run-tests.log" || echo shard-04-run-tests-log-absent
rg -m1 'ERROR: cannot open directory' "$CAND/shard-03/stdout.log"
rg -m1 'ERROR: cannot open directory' "$CAND/shard-04/stdout.log"
awk -F '\t' '$2 ~ /^php-src\/ext\/spl\// { c++ } END { print c+0 }' "$CAND/shard-03/results.txt"
awk -F '\t' '$2 ~ /^php-src\/ext\/spl\// { c++ } END { print c+0 }' "$CAND/shard-04/results.txt"
```

Observed result:

- Candidate `shard-03/run-tests.log` is absent.
- Candidate `shard-04/run-tests.log` is absent.
- Candidate shard 03 stdout contains:
  `ERROR: cannot open directory: /tmp/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/run-tests-harnesses/shard-03/ext/pdo/tests`
- Candidate shard 04 stdout contains:
  `ERROR: cannot open directory: /tmp/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/run-tests-harnesses/shard-04/ext/pdo/tests`
- Candidate shard 03 contains 0 SPL result rows.
- Candidate shard 04 contains 0 SPL result rows.

Accepted shard placement for the selected rows:

| Row | Accepted shard evidence |
| --- | --- |
| `php-src/ext/spl/tests/ArrayObject/array_006.phpt` | `shard-03/results.txt` |
| `php-src/ext/spl/tests/autoloading/bug61697.phpt` | `shard-04/results.txt` |
| `php-src/ext/spl/tests/SplFileObject/bug68479.phpt` | `shard-09/results.txt` |
| `php-src/ext/spl/tests/SplObjectStorage/bug69227.phpt` | `shard-09/results.txt` |
| `php-src/ext/spl/tests/DirectoryIterator_getBasename_basic_test.phpt` | `shard-03/results.txt` |
| `php-src/ext/spl/tests/SplFixedArray__construct_param_array.phpt` | `shard-10/results.txt` |

The candidate shard-result search found none of the six selected rows.

## Focused Replay Status

I did not run an accepted-vs-candidate PHPT replay because the saved historical
release binaries required by the cookbook are no longer present:

- Missing accepted binary:
  `/tmp/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67/cargo-target/release/phpc`
- Missing candidate binary:
  `/tmp/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/cargo-target/release/phpc`

Rebuilding those binaries would exceed this read-only report lane and could
overlap integration work. With the saved evidence currently available, the
strongest defensible classification is artifact/status based: selected SPL rows
were accepted `PASSED`, candidate `MISSING`, and candidate shards 03/04 aborted
before SPL coverage.

## Gate Impact

This lane does not move public score. It adds focused evidence that the SPL
slice of the blocked 221205Z regression list is control-plane coverage loss,
not proven row-level SPL semantic regression.

Next gate impact:

- Treat the 6 selected rows as absent/control-plane until focused replay or a
  repaired full candidate gate emits row-level results.
- The broader 137 SPL latest-public PASS regressions remain best classified as
  shard-truncation/control-plane absent rows, matching `.harness/reports/221205Z-spl.md`.
- A public score update still requires a zero-regression pinned PHPT gate or
  auditor-accepted adjudication policy for absent rows. This report is not
  sufficient to move the accepted score.

