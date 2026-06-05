# Focused Replay: Standard Filesystem/HTTP Rows

Agent: developer-409

Lane: 81, replacement owner for the focused standard filesystem/http replay
report originally named for developer-108.

Scope: read-only M0 replay/classification report for selected
`ext/standard` file, dir, directory, streams, http, and network rows from the
blocked candidate gate
`phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377`.
No compiler/runtime source files were edited, no full PHPT gate was run, and no
eval or variable-variable row was selected.

`DEVELOPMENT.md` was requested by the harness prompt but is absent under both
this worktree and `/home/claude/php-to-native-compiler`.

## Evidence Roots

Accepted baseline:

- `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67`
- Public/source head: `0b917f67a37d9ca9779d77f87173b628431c2425`
- Public score artifact: `7873/20294`

Blocked candidate:

- `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377`
- Public/source head: `56fe9377fb46be00db5fdd30c966fdba406dc581`
- Public score artifact: `7197/20294`
- PASS-regression summary: `1166` latest-public PASS regressions.

Shared PHPT checkout and wrapper:

- `/home/claude/php-src-phpt`
- `/home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper`

Prior shard report:

- `.harness/reports/221205Z-standard-filesystem-http.md`

## Shard Cross-Check

This lane owns 200 latest-public PASS regressions in the selected standard
filesystem/http prefixes. The candidate has only two row-level failures for
this lane; the other 198 owned rows are absent from candidate normalized status
and aggregate results.

```text
owned 200
by_prefix {'dir': 14, 'directory': 6, 'file': 160, 'http': 7, 'network': 3, 'streams': 10}
by_candidate_status {'FAILED': 2, 'MISSING': 198}
```

Command:

```sh
python3 - <<'PY'
from pathlib import Path
from collections import Counter
CAND=Path('/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377')
prefixes=[
('file','php-src/ext/standard/tests/file/'),
('dir','php-src/ext/standard/tests/dir/'),
('directory','php-src/ext/standard/tests/directory/'),
('streams','php-src/ext/standard/tests/streams/'),
('http','php-src/ext/standard/tests/http/'),
('network','php-src/ext/standard/tests/network/'),
]
rows=[r for r in (CAND/'regressions-from-latest-published-passes.txt').read_text().splitlines() if any(r.startswith(p) for _,p in prefixes)]
status={}
for line in (CAND/'current-status.normalized.tsv').read_text(errors='replace').splitlines():
    if '\t' in line:
        s,p=line.split('\t',1); status[p]=s
by_prefix=Counter()
by_status=Counter()
for r in rows:
    label=next(label for label,p in prefixes if r.startswith(p))
    by_prefix[label]+=1
    by_status[status.get(r,'MISSING')]+=1
print('owned', len(rows))
print('by_prefix', dict(sorted(by_prefix.items())))
print('by_candidate_status', dict(sorted(by_status.items())))
PY
```

## Representative Rows

Rows were selected from the prior shard report to cover the two concrete
candidate failures and one representative missing row from each larger
subtree. These rows avoid eval and variable-variable syntax.

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

## Artifact Status Join

All eight selected rows are in the candidate regression list and were accepted
baseline PASS rows. The candidate has explicit `FAILED` rows only for the two
`DirectoryClass_readonly_*` cases. The other six selected rows are missing
from both `current-status.normalized.tsv` and `all-results.txt`.

```text
row	title	in_reg	baseline_pass	candidate_pass	accepted_status	accepted_results	candidate_status	candidate_results	skipif
php-src/ext/standard/tests/directory/DirectoryClass_readonly_handle.phpt	Changing Directory::$handle property	True	True	False	PASSED	PASSED	FAILED	FAILED	False
php-src/ext/standard/tests/directory/DirectoryClass_readonly_path.phpt	Changing Directory::$handle property	True	True	False	PASSED	PASSED	FAILED	FAILED	False
php-src/ext/standard/tests/file/005_error.phpt	Test fileatime(), filemtime(), filectime() & touch() functions : error conditions	True	True	False	PASSED	PASSED	MISSING	MISSING	False
php-src/ext/standard/tests/file/file_exists_variation1.phpt	Test file_exists() function : usage variations	True	True	False	PASSED	PASSED	MISSING	MISSING	False
php-src/ext/standard/tests/dir/bug71542.phpt	Bug #71542 (disk_total_space does not work with relative paths)	True	True	False	PASSED	PASSED	MISSING	MISSING	False
php-src/ext/standard/tests/streams/stream_context_set_option_basic.phpt	stream_context_set_option() function - basic test for stream_context_set_option()	True	True	False	PASSED	PASSED	MISSING	MISSING	False
php-src/ext/standard/tests/http/http_build_query/http_build_query_variation1.phpt	Test http_build_query() function: usage variations - first arguments as object	True	True	False	PASSED	PASSED	MISSING	MISSING	False
php-src/ext/standard/tests/network/getprotobynumber_basic.phpt	getprotobynumber function basic test	True	True	False	PASSED	PASSED	MISSING	MISSING	True
```

Command:

```sh
python3 - <<'PY'
from pathlib import Path
import re
ACC=Path('/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67')
CAND=Path('/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377')
PHP=Path('/home/claude/php-src-phpt')
rows=[
'php-src/ext/standard/tests/directory/DirectoryClass_readonly_handle.phpt',
'php-src/ext/standard/tests/directory/DirectoryClass_readonly_path.phpt',
'php-src/ext/standard/tests/file/005_error.phpt',
'php-src/ext/standard/tests/file/file_exists_variation1.phpt',
'php-src/ext/standard/tests/dir/bug71542.phpt',
'php-src/ext/standard/tests/streams/stream_context_set_option_basic.phpt',
'php-src/ext/standard/tests/http/http_build_query/http_build_query_variation1.phpt',
'php-src/ext/standard/tests/network/getprotobynumber_basic.phpt',
]
def load_status(p):
    d={}
    for line in p.read_text(errors='replace').splitlines():
        if '\t' not in line: continue
        s,path=line.split('\t',1)
        d.setdefault(path,[]).append(s)
    return d
def norm(p):
    marker='/php-src/'
    if marker in p:
        return 'php-src/'+p.split(marker,1)[1]
    return p
def load_results(p):
    d={}
    for line in p.read_text(errors='replace').splitlines():
        if '\t' not in line: continue
        s,path=line.split('\t',1)
        d.setdefault(norm(path),[]).append(s)
    return d
def title(row):
    text=(PHP/row.removeprefix('php-src/')).read_text(errors='replace')
    m=re.search(r'--TEST--\n(.*?)(?=\n--[A-Z]+--|\Z)', text, re.S)
    return ' '.join(m.group(1).strip().split()) if m else ''
def has_skipif(row):
    return '--SKIPIF--' in (PHP/row.removeprefix('php-src/')).read_text(errors='replace')
acc_s=load_status(ACC/'current-status.normalized.tsv')
cand_s=load_status(CAND/'current-status.normalized.tsv')
acc_r=load_results(ACC/'all-results.txt')
cand_r=load_results(CAND/'all-results.txt')
reg=set((CAND/'regressions-from-latest-published-passes.txt').read_text().splitlines())
base_pass=set((CAND/'baseline-passes.normalized.txt').read_text().splitlines())
cand_pass=set((CAND/'current-passes.normalized.txt').read_text().splitlines())
print('row\ttitle\tin_reg\tbaseline_pass\tcandidate_pass\taccepted_status\taccepted_results\tcandidate_status\tcandidate_results\tskipif')
for r in rows:
    fmt=lambda v: ','.join(v) if v else 'MISSING'
    print('\t'.join([r,title(r),str(r in reg),str(r in base_pass),str(r in cand_pass),fmt(acc_s.get(r,[])),fmt(acc_r.get(r,[])),fmt(cand_s.get(r,[])),fmt(cand_r.get(r,[])),str(has_skipif(r))]))
PY
```

## Replay Preflight

Focused `run-tests.php` execution replay was not run because the historical
accepted and candidate `PHPC_BIN` binaries referenced by the cookbook are no
longer present. The wrapper, php-src checkout, and both source commits are
present, so replay can be run later after the binaries are restored or rebuilt
under an explicitly assigned replay lane.

Preflight:

```text
missing /tmp/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67/cargo-target/release/phpc
missing /tmp/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/cargo-target/release/phpc
present /home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper
present /home/claude/php-src-phpt/run-tests.php
accepted-present 0b917f67a37d9ca9779d77f87173b628431c2425
candidate-present 56fe9377fb46be00db5fdd30c966fdba406dc581
```

Running these rows with the wrong or current `PHPC_BIN` would not measure the
accepted-vs-candidate behavior under review. Rebuilding historical release
binaries was not done in this report-only lane.

Prepared row list for a later restored-binary focused replay:

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

## Candidate Failure Evidence

The two direct candidate failures are not broad filesystem I/O regressions.
They are internal `Directory` object readonly-property diagnostic drift:

- `DirectoryClass_readonly_handle.phpt`
  - Candidate status: `current-status.normalized.tsv:5775`
  - Candidate result: `all-results.txt:18180`, `shard-06/results.txt:2908`
  - Candidate stdout: `shard-06/stdout.log:3267`, `shard-06/stdout.log:5937`
  - Candidate diff: `shard-06/run-tests.log` around line `52534`
  - Observed candidate message adds `protected(set)` and `from global scope`
    to both readonly modify and unset diagnostics, while the expected output
    uses the shorter `Cannot modify readonly property Directory::$handle` and
    `Cannot unset readonly property Directory::$handle` forms.

- `DirectoryClass_readonly_path.phpt`
  - Candidate status: `current-status.normalized.tsv:5776`
  - Candidate result: `all-results.txt:2908`, `shard-01/results.txt:2908`
  - Candidate stdout: `shard-01/stdout.log:3244`, `shard-01/stdout.log:5874`
  - Candidate diff: `shard-01/run-tests.log` around line `56454`
  - Observed candidate message adds `protected(set)` and `from global scope`
    to both readonly modify and unset diagnostics, while the expected output
    uses the shorter `Cannot modify readonly property Directory::$path` and
    `Cannot unset readonly property Directory::$path` forms.

## Classification

| Row | Accepted evidence | Candidate evidence | Classification |
| --- | --- | --- | --- |
| `DirectoryClass_readonly_handle.phpt` | `PASSED` | `FAILED` with row-level diff | semantic/direct failure: internal readonly-property diagnostic parity |
| `DirectoryClass_readonly_path.phpt` | `PASSED` | `FAILED` with row-level diff | semantic/direct failure: internal readonly-property diagnostic parity |
| `005_error.phpt` | `PASSED` | missing from candidate status/results | absent/control-plane; semantic unknown |
| `file_exists_variation1.phpt` | `PASSED` | missing from candidate status/results | absent/control-plane; semantic unknown |
| `bug71542.phpt` | `PASSED` | missing from candidate status/results | absent/control-plane; semantic unknown |
| `stream_context_set_option_basic.phpt` | `PASSED` | missing from candidate status/results | absent/control-plane; semantic unknown |
| `http_build_query_variation1.phpt` | `PASSED` | missing from candidate status/results | absent/control-plane; semantic unknown |
| `getprotobynumber_basic.phpt` | `PASSED` | missing from candidate status/results | absent/control-plane; semantic unknown; has SKIPIF source |

## Conclusion

This focused lane does not justify a broad filesystem, stream, HTTP, or network
implementation lane. The representative sample confirms the prior shard report:
198 of 200 owned standard filesystem/http PASS regressions are absent
candidate rows, not proven semantic regressions.

The only concrete repair candidate in this lane is the pair of
`DirectoryClass_readonly_*` failures, which belong to an internal object
readonly-property diagnostic parity lane. The larger file/dir/stream/http/
network rows should stay blocked on PHPT harness completeness and focused
restored-binary replay before any runtime implementation work is assigned.
