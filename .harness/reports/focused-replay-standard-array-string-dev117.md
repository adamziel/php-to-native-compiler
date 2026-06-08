# Focused Replay: Standard Array/String Accepted-vs-Candidate Rows

| Field | Value |
| --- | --- |
| Owner | developer-117 |
| Card / worklane | 91 |
| Mode | focused replay and report artifact |
| Source edits | none |
| Full PHPT gate | not run |
| Public score movement | none |

## Summary

Card 91 asked for standard array/string accepted-vs-candidate PHPT sample
replay. The historical `/tmp` gate binaries referenced by earlier reports were
still absent, so this lane rebuilt release `phpc` binaries from the pinned
accepted and candidate commits and replayed the selected rows through the
existing PHPT wrapper.

Result: both rebuilt binaries passed all 16 selected rows.

| Replay | Commit | Rows | Passed | Failed | Skipped | Exit |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| Accepted | `0b917f67a37d9ca9779d77f87173b628431c2425` | 16 | 16 | 0 | 0 | 0 |
| Candidate | `56fe9377fb46be00db5fdd30c966fdba406dc581` | 16 | 16 | 0 | 0 | 0 |

This does not prove the blocked `221205Z` full gate was good. It proves that
these representative standard array/string rows do not reproduce as semantic
failures when the accepted and candidate commits are rebuilt and run directly.
That supports the earlier classification that the `221205Z` standard
array/string PASS regressions were control-plane row-absence symptoms rather
than preserved row-level semantic failures.

## Inputs

| Input | Value |
| --- | --- |
| Accepted commit | `0b917f67a37d9ca9779d77f87173b628431c2425` |
| Candidate commit | `56fe9377fb46be00db5fdd30c966fdba406dc581` |
| PHPT checkout | `/home/claude/php-src-phpt` |
| Wrapper | `/home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper` |
| Accepted build worktree | `/tmp/phpt-card91-accepted` |
| Candidate build worktree | `/tmp/phpt-card91-candidate` |
| Accepted binary | `/tmp/phpt-card91-target-accepted/release/phpc` |
| Candidate binary | `/tmp/phpt-card91-target-candidate/release/phpc` |
| Replay output root | `/tmp/phpt-card91-replay` |

The old durable gate directories cited by prior selector reports were no
longer present under `/home/claude/supervised-php-compiler/state/logs` during
this lane, and no `regressions-from-latest-published-passes.txt` file for the
`221205Z` gate was found there. Historical candidate-row absence therefore
comes from the committed prior artifacts:

- `.harness/reports/standard-array-replay-selector.md`
- `.harness/reports/focused-replay-standard-array-replacement.md`
- `.harness/reports/focused-replay-standard-strings-dev107.md`
- `.harness/reports/221205Z-standard-strings-replace-replay.md`

## Rows

| Row | Accepted replay | Candidate replay |
| --- | --- | --- |
| `php-src/ext/standard/tests/array/array_chunk2.phpt` | `PASSED` | `PASSED` |
| `php-src/ext/standard/tests/array/array_count_values.phpt` | `PASSED` | `PASSED` |
| `php-src/ext/standard/tests/array/array_diff_single_array.phpt` | `PASSED` | `PASSED` |
| `php-src/ext/standard/tests/array/array_filter_basic.phpt` | `PASSED` | `PASSED` |
| `php-src/ext/standard/tests/array/array_map_basic.phpt` | `PASSED` | `PASSED` |
| `php-src/ext/standard/tests/array/array_merge.phpt` | `PASSED` | `PASSED` |
| `php-src/ext/standard/tests/array/array_walk/array_walk_basic1.phpt` | `PASSED` | `PASSED` |
| `php-src/ext/standard/tests/array/sort/array_multisort_basic1.phpt` | `PASSED` | `PASSED` |
| `php-src/ext/standard/tests/strings/005.phpt` | `PASSED` | `PASSED` |
| `php-src/ext/standard/tests/strings/basename_invalid_path.phpt` | `PASSED` | `PASSED` |
| `php-src/ext/standard/tests/strings/bin2hex.phpt` | `PASSED` | `PASSED` |
| `php-src/ext/standard/tests/strings/html_entity_decode_cp866.phpt` | `PASSED` | `PASSED` |
| `php-src/ext/standard/tests/strings/md5.phpt` | `PASSED` | `PASSED` |
| `php-src/ext/standard/tests/strings/parse_str_null_bytes.phpt` | `PASSED` | `PASSED` |
| `php-src/ext/standard/tests/strings/sprintf_variation3.phpt` | `PASSED` | `PASSED` |
| `php-src/ext/standard/tests/strings/strtr_with_reference.phpt` | `PASSED` | `PASSED` |

## Replay Artifacts

| Artifact | Purpose |
| --- | --- |
| `/tmp/phpt-card91-replay/accepted/results.txt` | Accepted per-row PHPT result list |
| `/tmp/phpt-card91-replay/accepted/stdout.log` | Accepted `run-tests.php` summary, 16/16 pass |
| `/tmp/phpt-card91-replay/accepted/run-tests.log` | Accepted full PHPT log |
| `/tmp/phpt-card91-replay/candidate/results.txt` | Candidate per-row PHPT result list |
| `/tmp/phpt-card91-replay/candidate/stdout.log` | Candidate `run-tests.php` summary, 16/16 pass |
| `/tmp/phpt-card91-replay/candidate/run-tests.log` | Candidate full PHPT log |

Both `stderr.log` files contain only the shared `run-tests.php` environment
probe messages:

```text
sh: line 1: autoconf: command not found
sh: line 1: /home/claude/php-src-phpt/libtool: No such file or directory
```

Those messages appeared in both accepted and candidate runs and did not prevent
the 16 selected rows from passing.

## Commands

Build accepted:

```sh
git worktree add --detach /tmp/phpt-card91-accepted 0b917f67a37d9ca9779d77f87173b628431c2425
CARGO_TARGET_DIR=/tmp/phpt-card91-target-accepted CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 cargo build -p phpc --release
```

Build candidate:

```sh
git worktree add --detach /tmp/phpt-card91-candidate 56fe9377fb46be00db5fdd30c966fdba406dc581
CARGO_TARGET_DIR=/tmp/phpt-card91-target-candidate CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 cargo build -p phpc --release
```

Replay shape used for each binary:

```sh
cd /home/claude/php-src-phpt
PHPC_BIN=<rebuilt-binary> \
TEST_PHP_EXECUTABLE=/home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper \
TEST_PHP_ARGS= \
TMPDIR=<replay-output>/tmp \
TEMP=<replay-output>/tmp \
TMP=<replay-output>/tmp \
PHPC_PHPT_TIMEOUT_SECONDS=55 \
PHPC_PHPT_KILL_AFTER_SECONDS=5 \
PHPT_SYSTEM_PHP=php \
TEST_PHP_SRCDIR=/home/claude/php-src-phpt \
NO_INTERACTION=1 \
php run-tests.php -q -n \
  -p /home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper \
  -W <replay-output>/results.txt \
  -s <replay-output>/run-tests.log \
  --no-color \
  --set-timeout 65 \
  --temp-source /home/claude/php-src-phpt \
  --temp-target <replay-output>/phpt-tmp \
  <16 selected PHPT paths>
```

## Next Action

Treat these standard array/string samples as replay-cleared for the rebuilt
accepted/candidate commits. The remaining deterministic action for the blocked
`221205Z` score is harness/control-plane investigation: why the full candidate
gate omitted previously passing rows from row-level result artifacts, and why
that omission was not detected as missing results.
