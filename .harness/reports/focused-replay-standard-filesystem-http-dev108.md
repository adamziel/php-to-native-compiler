# Focused Replay: Standard Filesystem/HTTP Rows

Agent: developer-102

Lane: 81, replacement owner for the focused standard filesystem/http replay
report originally named for developer-108.

Scope: read-only M0 replay/classification report for selected
`ext/standard` file, dir, directory, streams, http, and network rows from the
blocked candidate gate
`phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377`.
No compiler/runtime source files were edited, no full PHPT gate was run, and no
eval or variable-variable row was selected. The only tracked project change for
this lane is this report artifact.

Required startup documents were read from the assigned worktree where present.
`DEVELOPMENT.md` is absent from this worktree root, but present at the
repository root `/home/claude/php-to-native-compiler/DEVELOPMENT.md` and was
read there.

## Evidence Roots

Accepted baseline:

- `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67`
- Public/source head: `0b917f67a37d9ca9779d77f87173b628431c2425`
- Public score artifact: `7873/20294`

Blocked candidate:

- `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377`
- Public/source head: `56fe9377fb46be00db5fdd30c966fdba406dc581`
- Public score artifact: `7197/20294`
- PASS-regression summary: `1166` latest-public PASS regressions

Shared PHPT checkout and wrapper:

- `/home/claude/php-src-phpt`
- `/home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper`

Prior shard report:

- `.harness/reports/221205Z-standard-filesystem-http.md`

## Shard Cross-Check

This lane owns 200 latest-public PASS regressions in the selected standard
filesystem/http prefixes. The 221205Z candidate artifact has only two explicit
row-level failures for this lane; the other 198 owned rows are absent from
candidate normalized status and aggregate results.

```text
owned 200
by_prefix {'dir': 14, 'directory': 6, 'file': 160, 'http': 7, 'network': 3, 'streams': 10}
by_candidate_status {'FAILED': 2, 'MISSING': 198}
```

## Representative Rows

Rows were selected from the prior shard report to cover the two concrete
candidate failures and one representative missing row from each larger subtree.

| Row | PHPT title | Bucket |
| --- | --- | --- |
| `php-src/ext/standard/tests/directory/DirectoryClass_readonly_handle.phpt` | `Changing Directory::$handle property` | direct candidate failure: internal `Directory` readonly metadata |
| `php-src/ext/standard/tests/directory/DirectoryClass_readonly_path.phpt` | `Changing Directory::$handle property` | direct candidate failure: internal `Directory` readonly metadata |
| `php-src/ext/standard/tests/file/005_error.phpt` | `Test fileatime(), filemtime(), filectime() & touch() functions : error conditions` | file API error paths |
| `php-src/ext/standard/tests/file/file_exists_variation1.phpt` | `Test file_exists() function : usage variations` | file metadata/path handling |
| `php-src/ext/standard/tests/dir/bug71542.phpt` | `Bug #71542 (disk_total_space does not work with relative paths)` | directory/path helper |
| `php-src/ext/standard/tests/streams/stream_context_set_option_basic.phpt` | `stream_context_set_option() function - basic test for stream_context_set_option()` | stream context mutation |
| `php-src/ext/standard/tests/http/http_build_query/http_build_query_variation1.phpt` | `Test http_build_query() function: usage variations - first arguments as object` | HTTP helper/object encoding |
| `php-src/ext/standard/tests/network/getprotobynumber_basic.phpt` | `getprotobynumber function basic test` | network helper; has SKIPIF in source |

## Historical Binary Rebuild

The historical `/tmp/phpt-full-current-score-*` run roots and `cargo-target`
binaries were absent, so I rebuilt exact release `phpc` binaries from detached
temporary worktrees under `/tmp/phpc-card81-dev102.G8YybJ`.

Commands:

```sh
git worktree add --detach /tmp/phpc-card81-dev102.G8YybJ/accepted-src 0b917f67a37d9ca9779d77f87173b628431c2425
git worktree add --detach /tmp/phpc-card81-dev102.G8YybJ/candidate-src 56fe9377fb46be00db5fdd30c966fdba406dc581

CARGO_TARGET_DIR=/tmp/phpc-card81-dev102.G8YybJ/accepted-target \
CARGO_BUILD_JOBS=1 \
CARGO_INCREMENTAL=0 \
cargo build --release -p phpc

CARGO_TARGET_DIR=/tmp/phpc-card81-dev102.G8YybJ/candidate-target \
CARGO_BUILD_JOBS=1 \
CARGO_INCREMENTAL=0 \
cargo build --release -p phpc
```

Build results:

- Accepted `0b917f67`: passed, release build completed in `4m 25s`.
- Candidate `56fe9377`: passed, release build completed in `4m 34s`.
- Cargo emitted a non-fatal registry cache write warning for `gost94`, then
  continued.

Binary/result hashes:

```text
421d406e12ca29f68e27ed8c694503b1b0ebfa082e912e8784df85a9eeb453ae  /tmp/phpc-card81-dev102.G8YybJ/accepted-target/release/phpc
564c6a14ee64c345e75d68de746ef6c64640d6c6307822d5e9b0a94a4a5bf7a4  /tmp/phpc-card81-dev102.G8YybJ/candidate-target/release/phpc
87203eb38188c7f800b9ef3c1d4c35ff5d877a56e06f015854b2e5407e79f4c1  /tmp/phpc-card81-dev102.G8YybJ/accepted/results.txt
108540da5b8192c80c3bd5e180318b2a6cbaf6a30b0456754f3404d0286ee6f5  /tmp/phpc-card81-dev102.G8YybJ/candidate/results.txt
```

## Focused Replay Commands

Row file:

```text
/home/claude/php-src-phpt/ext/standard/tests/directory/DirectoryClass_readonly_handle.phpt
/home/claude/php-src-phpt/ext/standard/tests/directory/DirectoryClass_readonly_path.phpt
/home/claude/php-src-phpt/ext/standard/tests/file/005_error.phpt
/home/claude/php-src-phpt/ext/standard/tests/file/file_exists_variation1.phpt
/home/claude/php-src-phpt/ext/standard/tests/dir/bug71542.phpt
/home/claude/php-src-phpt/ext/standard/tests/streams/stream_context_set_option_basic.phpt
/home/claude/php-src-phpt/ext/standard/tests/http/http_build_query/http_build_query_variation1.phpt
/home/claude/php-src-phpt/ext/standard/tests/network/getprotobynumber_basic.phpt
```

Accepted replay command:

```sh
PHPC_BIN=/tmp/phpc-card81-dev102.G8YybJ/accepted-target/release/phpc \
TEST_PHP_EXECUTABLE=/home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper \
TEST_PHP_ARGS= \
TMPDIR=/tmp/phpc-card81-dev102.G8YybJ/accepted/tmp \
TEMP=/tmp/phpc-card81-dev102.G8YybJ/accepted/tmp \
TMP=/tmp/phpc-card81-dev102.G8YybJ/accepted/tmp \
PHPC_PHPT_TIMEOUT_SECONDS=55 \
PHPC_PHPT_KILL_AFTER_SECONDS=5 \
PHPT_SYSTEM_PHP=php \
TEST_PHP_SRCDIR=/home/claude/php-src-phpt \
NO_INTERACTION=1 \
php run-tests.php -q -n \
  -p /home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper \
  -r /tmp/phpc-card81-dev102.G8YybJ/standard-filesystem-http.tests \
  -W /tmp/phpc-card81-dev102.G8YybJ/accepted/results.txt \
  -s /tmp/phpc-card81-dev102.G8YybJ/accepted/run-tests.log \
  --no-color \
  --set-timeout 65 \
  --temp-source /home/claude/php-src-phpt \
  --temp-target /tmp/phpc-card81-dev102.G8YybJ/accepted/phpt-tmp
```

Candidate replay command was identical except:

```text
PHPC_BIN=/tmp/phpc-card81-dev102.G8YybJ/candidate-target/release/phpc
TMPDIR=/tmp/phpc-card81-dev102.G8YybJ/candidate/tmp
TEMP=/tmp/phpc-card81-dev102.G8YybJ/candidate/tmp
TMP=/tmp/phpc-card81-dev102.G8YybJ/candidate/tmp
-W /tmp/phpc-card81-dev102.G8YybJ/candidate/results.txt
-s /tmp/phpc-card81-dev102.G8YybJ/candidate/run-tests.log
--temp-target /tmp/phpc-card81-dev102.G8YybJ/candidate/phpt-tmp
```

Both replay commands printed post-report shell noise from the pinned php-src
checkout: `autoconf: command not found` and missing
`/home/claude/php-src-phpt/libtool`. The result files were still written and
the PHPT summaries completed.

## Replay Results

Accepted `0b917f67` replay:

```text
Number of tests :     8                 8
Tests skipped   :     0 (  0.0%) --------
Tests warned    :     0 (  0.0%) (  0.0%)
Tests failed    :     0 (  0.0%) (  0.0%)
Tests passed    :     8 (100.0%) (100.0%)
```

Accepted `results.txt`:

```text
PASSED  /home/claude/php-src-phpt/ext/standard/tests/dir/bug71542.phpt
PASSED  /home/claude/php-src-phpt/ext/standard/tests/directory/DirectoryClass_readonly_handle.phpt
PASSED  /home/claude/php-src-phpt/ext/standard/tests/directory/DirectoryClass_readonly_path.phpt
PASSED  /home/claude/php-src-phpt/ext/standard/tests/file/005_error.phpt
PASSED  /home/claude/php-src-phpt/ext/standard/tests/file/file_exists_variation1.phpt
PASSED  /home/claude/php-src-phpt/ext/standard/tests/http/http_build_query/http_build_query_variation1.phpt
PASSED  /home/claude/php-src-phpt/ext/standard/tests/network/getprotobynumber_basic.phpt
PASSED  /home/claude/php-src-phpt/ext/standard/tests/streams/stream_context_set_option_basic.phpt
```

Candidate `56fe9377` replay:

```text
Number of tests :     8                 8
Tests skipped   :     0 (  0.0%) --------
Tests warned    :     0 (  0.0%) (  0.0%)
Tests failed    :     2 ( 25.0%) ( 25.0%)
Tests passed    :     6 ( 75.0%) ( 75.0%)
```

Candidate `results.txt`:

```text
PASSED  /home/claude/php-src-phpt/ext/standard/tests/dir/bug71542.phpt
FAILED  /home/claude/php-src-phpt/ext/standard/tests/directory/DirectoryClass_readonly_handle.phpt
FAILED  /home/claude/php-src-phpt/ext/standard/tests/directory/DirectoryClass_readonly_path.phpt
PASSED  /home/claude/php-src-phpt/ext/standard/tests/file/005_error.phpt
PASSED  /home/claude/php-src-phpt/ext/standard/tests/file/file_exists_variation1.phpt
PASSED  /home/claude/php-src-phpt/ext/standard/tests/http/http_build_query/http_build_query_variation1.phpt
PASSED  /home/claude/php-src-phpt/ext/standard/tests/network/getprotobynumber_basic.phpt
PASSED  /home/claude/php-src-phpt/ext/standard/tests/streams/stream_context_set_option_basic.phpt
```

## Candidate Failure Evidence

The two candidate failures are not broad filesystem I/O regressions. They are
internal `Directory` object readonly-property diagnostic drift:

- `DirectoryClass_readonly_handle.phpt`
  - Candidate replay result: `FAILED`
  - Candidate log: `/tmp/phpc-card81-dev102.G8YybJ/candidate/run-tests.log`
  - Observed candidate diagnostics add `protected(set)` and
    `from global scope` to both readonly modify and unset diagnostics.

- `DirectoryClass_readonly_path.phpt`
  - Candidate replay result: `FAILED`
  - Candidate log: `/tmp/phpc-card81-dev102.G8YybJ/candidate/run-tests.log`
  - Observed candidate diagnostics add `protected(set)` and
    `from global scope` to both readonly modify and unset diagnostics.

Relevant replay diff lines:

```text
Directory::$handle:
001- Error: Cannot modify readonly property Directory::$handle
001+ Error: Cannot modify protected(set) readonly property Directory::$handle from global scope
003- Error: Cannot unset readonly property Directory::$handle
003+ Error: Cannot unset protected(set) readonly property Directory::$handle from global scope

Directory::$path:
001- Error: Cannot modify readonly property Directory::$path
001+ Error: Cannot modify protected(set) readonly property Directory::$path from global scope
003- Error: Cannot unset readonly property Directory::$path
003+ Error: Cannot unset protected(set) readonly property Directory::$path from global scope
```

## Classification

| Row | Accepted replay | Candidate replay | 221205Z candidate artifact | Classification |
| --- | --- | --- | --- | --- |
| `DirectoryClass_readonly_handle.phpt` | `PASSED` | `FAILED` | explicit `FAILED` | semantic/direct failure: internal readonly-property diagnostic parity |
| `DirectoryClass_readonly_path.phpt` | `PASSED` | `FAILED` | explicit `FAILED` | semantic/direct failure: internal readonly-property diagnostic parity |
| `005_error.phpt` | `PASSED` | `PASSED` | missing from candidate status/results | control-plane absent in 221205Z, semantic pass on focused replay |
| `file_exists_variation1.phpt` | `PASSED` | `PASSED` | missing from candidate status/results | control-plane absent in 221205Z, semantic pass on focused replay |
| `bug71542.phpt` | `PASSED` | `PASSED` | missing from candidate status/results | control-plane absent in 221205Z, semantic pass on focused replay |
| `stream_context_set_option_basic.phpt` | `PASSED` | `PASSED` | missing from candidate status/results | control-plane absent in 221205Z, semantic pass on focused replay |
| `http_build_query_variation1.phpt` | `PASSED` | `PASSED` | missing from candidate status/results | control-plane absent in 221205Z, semantic pass on focused replay |
| `getprotobynumber_basic.phpt` | `PASSED` | `PASSED` | missing from candidate status/results; has SKIPIF source | control-plane absent in 221205Z, semantic pass on focused replay |

## Conclusion

This focused replay does not justify a broad filesystem, stream, HTTP, or
network implementation lane. The six representative rows that were missing
from the 221205Z candidate manifest pass against the rebuilt candidate binary,
so they are control-plane/result-manifest gaps in that gate, not current
semantic evidence.

The only concrete repair candidate in this lane remains the pair of
`DirectoryClass_readonly_*` failures, which belong to an internal `Directory`
readonly-property diagnostic parity lane. Broader standard file/dir/stream/http
and network rows should remain blocked on PHPT harness/result completeness or
larger focused replay evidence before any runtime implementation work is
assigned.
