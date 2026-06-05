# 221205Z Standard Filesystem/HTTP Regression Shard

Owner: developer-92
Lane: 16
Mode: read-only M0 report, no compiler/runtime source edits

## Evidence Inputs

- Gate directory:
  `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377`
- Regression list:
  `regressions-from-latest-published-passes.txt`
- Current status/result files:
  `current-status.normalized.tsv`, `all-results.txt`, `shard-*/results.txt`,
  `shard-*/stdout.log`
- Gate preflight:
  `current-score-gate-preflight.tsv`
- PHP source checkout from gate:
  `/home/claude/php-src-phpt` at
  `f97ff597429a2fe633665a7e02d97c8077f9f90f`
- Baseline pass file:
  `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67/current-passes.normalized.txt`

## Commands Used

```sh
wc -l \
  /home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/regressions-from-latest-published-passes.txt \
  /home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/all-results.txt

python3 - <<'PY'
from pathlib import Path
from collections import Counter
base = Path("/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377")
prefixes = [
 "php-src/ext/standard/tests/file/",
 "php-src/ext/standard/tests/dir/",
 "php-src/ext/standard/tests/directory/",
 "php-src/ext/standard/tests/streams/",
 "php-src/ext/standard/tests/http/",
 "php-src/ext/standard/tests/network/",
]
rows = [line.strip() for line in (base / "regressions-from-latest-published-passes.txt").read_text().splitlines() if line.strip()]
owned = [row for row in rows if any(row.startswith(prefix) for prefix in prefixes)]
print(len(owned))
print(Counter(row.split("/")[4] for row in owned))
PY

rg -n "005_error|bug71542|file_exists_variation1|DirectoryClass_readonly" \
  /home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67/current-passes.normalized.txt \
  /home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/current-passes.normalized.txt \
  /home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/current-status.normalized.tsv
```

## Shard Size

The full blocked candidate gate reports 1,166 latest-public PASS regressions.
This lane owns 200 of them:

| Subtree | Count |
|---|---:|
| `ext/standard/tests/file` | 160 |
| `ext/standard/tests/dir` | 14 |
| `ext/standard/tests/streams` | 10 |
| `ext/standard/tests/http` | 7 |
| `ext/standard/tests/directory` | 6 |
| `ext/standard/tests/network` | 3 |

## Current Evidence Status

Only 2 of the 200 owned regression rows appear in `all-results.txt` and
`current-status.normalized.tsv` as current direct failures:

- `php-src/ext/standard/tests/directory/DirectoryClass_readonly_handle.phpt`
- `php-src/ext/standard/tests/directory/DirectoryClass_readonly_path.phpt`

The other 198 owned rows are present in the baseline pass list but absent from
the current candidate status/results files. Sample absent rows are:

- `php-src/ext/standard/tests/file/005_error.phpt`
- `php-src/ext/standard/tests/file/file_exists_variation1.phpt`
- `php-src/ext/standard/tests/dir/bug71542.phpt`
- `php-src/ext/standard/tests/streams/stream_context_set_option_basic.phpt`
- `php-src/ext/standard/tests/http/http_build_query/http_build_query_variation1.phpt`
- `php-src/ext/standard/tests/network/getprotobynumber_basic.phpt`

Those PHPT files do exist in `/home/claude/php-src-phpt` at the pinned source
checkout. This makes the dominant shard symptom an evidence/result-manifest
gap, not 198 proven runtime regressions.

Related gate health detail: `counts.tsv` reports 18,949 aggregate result rows
while `current-score-gate-preflight.tsv` reports 21,827 PHPT files
(`21,780` sharded plus `47` serialized open_basedir). All six shards exited
with code `1`, and `shard-signal-summary.tsv` reports no signal exits.

## Symptom Buckets

The owned row names cluster as follows. These are baseline-pass regression rows,
not all direct current failures.

| Bucket | Rows | Notes |
|---|---:|---|
| Broad file path/stat/mutation/stream functions | 160 | Includes `fopen`, `fscanf`, `fgetcsv`, `file_get_contents`, `file_put_contents`, `filesize`, `lstat/stat`, `realpath`, symlink/link, `rename`, `copy`, `chmod/chown`, `touch`, `unlink`, and bug/security rows. Most are absent from current result status. |
| Directory functions | 14 | `scandir`, `opendir`, `readdir`, `rewinddir`, `dir`, `chdir`, and `closedir` rows. All are absent from current result status. |
| Directory class metadata/readonly properties | 6 | Two direct current failures are `DirectoryClass_readonly_handle` and `DirectoryClass_readonly_path`; four related rows are absent from current result status. |
| Stream context/metadata helpers | 10 | `stream_context_*`, `stream_get_contents`, `stream_get_meta_data`, `stream_get_wrappers`, and `stream_is_local`. All are absent from current result status. |
| HTTP helpers/request parsing | 7 | Mostly `http_build_query` object/variation rows and `request_parse_body` invalid option rows. All are absent from current result status. |
| Network helpers | 3 | `bug69523`, `getprotobyname_error`, and `getprotobynumber_basic`. All are absent from current result status. |

## Representative Replay Rows

Use these rows before assigning implementation work:

1. `php-src/ext/standard/tests/directory/DirectoryClass_readonly_handle.phpt`
   - Direct current failure.
   - Likely owner: internal object property/readonly metadata, not filesystem I/O.
2. `php-src/ext/standard/tests/directory/DirectoryClass_readonly_path.phpt`
   - Direct current failure.
   - Same likely owner as row 1.
3. `php-src/ext/standard/tests/file/005_error.phpt`
   - Baseline PASS, absent from current status.
   - Replays the file API error-path bucket.
4. `php-src/ext/standard/tests/file/file_exists_variation1.phpt`
   - Baseline PASS, absent from current status.
   - Replays file metadata/path handling.
5. `php-src/ext/standard/tests/dir/bug71542.phpt`
   - Baseline PASS, absent from current status.
   - Replays directory handling.
6. `php-src/ext/standard/tests/streams/stream_context_set_option_basic.phpt`
   - Baseline PASS, absent from current status.
   - Replays stream context behavior.
7. `php-src/ext/standard/tests/http/http_build_query/http_build_query_variation1.phpt`
   - Baseline PASS, absent from current status.
   - Replays HTTP helper behavior without network I/O.
8. `php-src/ext/standard/tests/network/getprotobynumber_basic.phpt`
   - Baseline PASS, absent from current status.
   - Replays one small network helper row.

Suggested replay shape, using a small row file instead of the full suite:

```sh
cat > /tmp/lane16-standard-filesystem-http-replay.tests <<'EOF'
/home/claude/php-src-phpt/ext/standard/tests/directory/DirectoryClass_readonly_handle.phpt
/home/claude/php-src-phpt/ext/standard/tests/directory/DirectoryClass_readonly_path.phpt
/home/claude/php-src-phpt/ext/standard/tests/file/005_error.phpt
/home/claude/php-src-phpt/ext/standard/tests/file/file_exists_variation1.phpt
/home/claude/php-src-phpt/ext/standard/tests/dir/bug71542.phpt
/home/claude/php-src-phpt/ext/standard/tests/streams/stream_context_set_option_basic.phpt
/home/claude/php-src-phpt/ext/standard/tests/http/http_build_query/http_build_query_variation1.phpt
/home/claude/php-src-phpt/ext/standard/tests/network/getprotobynumber_basic.phpt
EOF

PHPC_BIN=<candidate phpc binary> \
TEST_PHP_EXECUTABLE=/home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper \
php /home/claude/php-src-phpt/run-tests.php \
  -q -n \
  -p /home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper \
  -r /tmp/lane16-standard-filesystem-http-replay.tests \
  -W /tmp/lane16-standard-filesystem-http-results.txt \
  -s /tmp/lane16-standard-filesystem-http-run-tests.log \
  --no-color \
  --set-timeout 65 \
  --temp-source /home/claude/php-src-phpt \
  --temp-target /tmp/lane16-standard-filesystem-http-tmp
```

## Recommendation

Do not start broad filesystem, stream, HTTP, or network implementation from the
200-row count alone. First run the focused replay above, or regenerate the
current gate manifest, because 198/200 owned rows are absent from the current
result status despite existing in the pinned source checkout.

The only implementation-looking direct failures in this shard are the two
`DirectoryClass_readonly_*` rows. Treat those as a narrow internal `Directory`
class metadata/readonly-property lane after focused replay confirms they still
fail. The broader file/stream/http/network rows should wait for replay evidence
that distinguishes real semantic failures from the current result-manifest gap.
