# No-SKIPIF Selector Sanity For Array/String Replay

Agent: developer-312

Lane: 129

Scope: read-only replay-prep report. No compiler, runtime, product source,
php-src, test-list, or full PHPT gate files were edited. The accepted public
score remains `7873/20294`; the blocked `221205Z` candidate remains blocked.

`DEVELOPMENT.md` was requested by the harness prompt but is not present in this
worktree.

## Inputs

- Standard array selector:
  `.harness/reports/standard-array-replay-selector.md`
- Standard array focused replacement:
  `.harness/reports/focused-replay-standard-array-replacement.md`
- Standard strings selector:
  `.harness/reports/focused-replay-standard-strings-dev107.md`
- Standard strings replacement selector:
  `.harness/reports/221205Z-standard-strings-replace-replay.md`
- Late-priority definitions:
  `.harness/reports/late-row-tag-crosscheck.md` and
  `.harness/reports/phpt-manifest-late-row-tags.md`
- Candidate gate:
  `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377`
- Accepted baseline gate:
  `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67`
- Pinned PHPT checkout:
  `/home/claude/php-src-phpt`

The late-priority scan uses the established planning-compatible lexical
patterns:

```python
eval_re = re.compile(r"(?i)(^|[^A-Za-z0-9_$])eval\s*\(")
vv_re = re.compile(r"\$\$|\$\{\s*\$|(?i:variable[-_ ]variables?)")
```

These tags are priority hints only; late rows remain in the public denominator.

## Summary

| Selector set | Rows checked | Exact PHPT paths exist | No-SKIPIF rows | Late-priority overlaps | PASS regressions | Candidate status |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| Standard array selector | 8 | 8 | 8 | 0 | 8 | all `ABSENT` |
| Standard strings selector | 8 | 8 | 7 | 0 | 8 | all `ABSENT` |
| Standard strings replacement selector | 3 | 3 | 3 | 0 | 3 | all `ABSENT` |
| Deduplicated total | 19 | 19 | 18 | 0 | 19 | all `ABSENT` |

Finding: the standard array selector and standard strings replacement selector
are clean no-SKIPIF replay selectors. The broader standard strings selector has
one no-SKIPIF violation:

`php-src/ext/standard/tests/strings/basename_invalid_path.phpt`

That file has a Windows-only `--SKIPIF--` guard:

```php
<?php if((substr(PHP_OS, 0, 3) == "WIN")) die('skip not for Windows"'); ?>
```

No checked row overlaps the `eval` or variable-variable late-priority scan.
Every checked row is in `regressions-from-latest-published-passes.txt`, was an
accepted-baseline `PASSED` row, is absent from candidate normalized status, and
is absent from candidate `all-results.txt`. This report therefore does not
classify semantic compiler/runtime failures; it only verifies replay-selector
sanity.

## Row Check

| Set | Row | SKIPIF | Late tag | Accepted | Candidate | PHPT title |
| --- | --- | --- | --- | --- | --- | --- |
| array | `php-src/ext/standard/tests/array/array_chunk2.phpt` | no | - | `PASSED` | `ABSENT` | `basic array_chunk test` |
| array | `php-src/ext/standard/tests/array/array_count_values.phpt` | no | - | `PASSED` | `ABSENT` | `array_count_values()` |
| array | `php-src/ext/standard/tests/array/array_diff_single_array.phpt` | no | - | `PASSED` | `ABSENT` | `array_diff() with single array argument` |
| array | `php-src/ext/standard/tests/array/array_filter_basic.phpt` | no | - | `PASSED` | `ABSENT` | `Test array_filter() function : basic functionality` |
| array | `php-src/ext/standard/tests/array/array_map_basic.phpt` | no | - | `PASSED` | `ABSENT` | `Test array_map() function : basic functionality` |
| array | `php-src/ext/standard/tests/array/array_merge.phpt` | no | - | `PASSED` | `ABSENT` | `Test array_merge() function` |
| array | `php-src/ext/standard/tests/array/array_walk/array_walk_basic1.phpt` | no | - | `PASSED` | `ABSENT` | `Test array_walk() function : basic functionality - regular array` |
| array | `php-src/ext/standard/tests/array/sort/array_multisort_basic1.phpt` | no | - | `PASSED` | `ABSENT` | `Test array_multisort() function : basic functionality` |
| strings | `php-src/ext/standard/tests/strings/005.phpt` | no | - | `PASSED` | `ABSENT` | `highlight_string(), output buffer and error level` |
| strings | `php-src/ext/standard/tests/strings/bin2hex.phpt` | no | - | `PASSED` | `ABSENT` | `bin2hex() function` |
| strings | `php-src/ext/standard/tests/strings/basename_invalid_path.phpt` | yes | - | `PASSED` | `ABSENT` | `Test basename() function : usage variations with invalid paths` |
| strings | `php-src/ext/standard/tests/strings/md5.phpt` | no | - | `PASSED` | `ABSENT` | `md5() with ASCII output` |
| strings | `php-src/ext/standard/tests/strings/sprintf_variation3.phpt` | no | - | `PASSED` | `ABSENT` | `Test sprintf() function : usage variations - int formats with int values` |
| strings | `php-src/ext/standard/tests/strings/strtr_with_reference.phpt` | no | - | `PASSED` | `ABSENT` | `strtr() with references` |
| strings | `php-src/ext/standard/tests/strings/html_entity_decode_cp866.phpt` | no | - | `PASSED` | `ABSENT` | `Translation of HTML entities for encoding CP866` |
| strings | `php-src/ext/standard/tests/strings/parse_str_null_bytes.phpt` | no | - | `PASSED` | `ABSENT` | `parse_str() rejects null bytes` |
| string replace | `php-src/ext/standard/tests/strings/str_replace_basic.phpt` | no | - | `PASSED` | `ABSENT` | `Test str_replace() function basic function` |
| string replace | `php-src/ext/standard/tests/strings/str_replace_array_refs.phpt` | no | - | `PASSED` | `ABSENT` | `Test str_replace() function and array refs` |
| string replace | `php-src/ext/standard/tests/strings/bug27675.phpt` | no | - | `PASSED` | `ABSENT` | `Bug #27675 (str_ireplace segfaults when shrinking string)` |

## Replacement Recommendation

If the next replay lane requires a strictly no-SKIPIF standard-strings path
helper row, replace:

`php-src/ext/standard/tests/strings/basename_invalid_path.phpt`

with:

`php-src/ext/standard/tests/strings/basename_basic.phpt`

The replacement candidate is in the same standard-strings PASS-regression
shard, has no `--SKIPIF--`, exists in `/home/claude/php-src-phpt`, has accepted
status `PASSED`, and is also candidate `ABSENT`. Its PHPT title is:

`Test basename() function : basic functionality`

Do not use `php-src/ext/standard/tests/strings/highlight_file.phpt` as a
replacement for this PASS-regression replay: it has no `--SKIPIF--`, but it is
not in `regressions-from-latest-published-passes.txt` and already has candidate
status `FAILED`.

## Commands

Read required session docs:

```sh
sed -n '1,240p' AGENTS.md
sed -n '1,260p' docs/PROGRESS.md
sed -n '1,260p' docs/ARCHITECTURE.md
sed -n '1,260p' docs/SUPPORT.md
sed -n '1,260p' README.md
sed -n '1,260p' DEVELOPMENT.md
sed -n '1,260p' docs/LOOP_MEMORY.md
```

`DEVELOPMENT.md` failed with `No such file or directory`; the other required
files were present.

Inspect lane assignment and report artifacts without recursively scanning
`.harness/worktrees`:

```sh
python3 - <<'PY'
import sqlite3, json
path='/home/claude/php-to-native-compiler/.harness/harness.sqlite3'
con=sqlite3.connect(path)
con.row_factory=sqlite3.Row
cur=con.cursor()
for row in cur.execute("SELECT * FROM work_lanes WHERE id=129"):
    print(json.dumps(dict(row), sort_keys=True))
PY

find /home/claude/php-to-native-compiler/.harness \
  -path /home/claude/php-to-native-compiler/.harness/worktrees -prune \
  -o -type f \( -iname '*skipif*' -o -iname '*array*' -o -iname '*string*' \
  -o -iname '*late*' -o -iname '*manifest*' -o -iname '*replay*' \) -print
```

Generate the selector sanity table:

```sh
python3 - <<'PY'
from pathlib import Path
import re
from collections import Counter

php_src = Path('/home/claude/php-src-phpt')
cand = Path('/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377')
acc = Path('/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67')

row_groups = {
    'standard-array selector': [
        'php-src/ext/standard/tests/array/array_chunk2.phpt',
        'php-src/ext/standard/tests/array/array_count_values.phpt',
        'php-src/ext/standard/tests/array/array_diff_single_array.phpt',
        'php-src/ext/standard/tests/array/array_filter_basic.phpt',
        'php-src/ext/standard/tests/array/array_map_basic.phpt',
        'php-src/ext/standard/tests/array/array_merge.phpt',
        'php-src/ext/standard/tests/array/array_walk/array_walk_basic1.phpt',
        'php-src/ext/standard/tests/array/sort/array_multisort_basic1.phpt',
    ],
    'standard-strings selector': [
        'php-src/ext/standard/tests/strings/005.phpt',
        'php-src/ext/standard/tests/strings/bin2hex.phpt',
        'php-src/ext/standard/tests/strings/basename_invalid_path.phpt',
        'php-src/ext/standard/tests/strings/md5.phpt',
        'php-src/ext/standard/tests/strings/sprintf_variation3.phpt',
        'php-src/ext/standard/tests/strings/strtr_with_reference.phpt',
        'php-src/ext/standard/tests/strings/html_entity_decode_cp866.phpt',
        'php-src/ext/standard/tests/strings/parse_str_null_bytes.phpt',
    ],
    'standard-strings replacement selector': [
        'php-src/ext/standard/tests/strings/str_replace_basic.phpt',
        'php-src/ext/standard/tests/strings/str_replace_array_refs.phpt',
        'php-src/ext/standard/tests/strings/bug27675.phpt',
    ],
}

eval_re = re.compile(r'(?i)(^|[^A-Za-z0-9_$])eval\s*\(')
vv_re = re.compile(r'\$\$|\$\{\s*\$|(?i:variable[-_ ]variables?)')

def load_status(path):
    out = {}
    for line in path.read_text(errors='replace').splitlines():
        if '\t' in line:
            status, row = line.split('\t', 1)
            out[row] = status
    return out

acc_status = load_status(acc / 'current-status.normalized.tsv')
cand_status = load_status(cand / 'current-status.normalized.tsv')
regressions = set((cand / 'regressions-from-latest-published-passes.txt').read_text().splitlines())
base_pass = set((cand / 'baseline-passes.normalized.txt').read_text().splitlines())
current_pass = set((cand / 'current-passes.normalized.txt').read_text().splitlines())

all_results_paths = set()
for line in (cand / 'all-results.txt').read_text(errors='replace').splitlines():
    if '\t' not in line:
        continue
    _, raw = line.split('\t', 1)
    marker = '/php-src/'
    if marker in raw:
        raw = 'php-src/' + raw.split(marker, 1)[1]
    all_results_paths.add(raw)

seen = []
print('group\trow\texists\thas_skipif\tlate_reason\ttitle\tregression\tbaseline_pass\tcurrent_pass\taccepted_status\tcandidate_status\tcandidate_all_results')
for group, rows in row_groups.items():
    for row in rows:
        path = php_src / row.removeprefix('php-src/')
        text = path.read_text(errors='replace') if path.exists() else ''
        title_match = re.search(r'--TEST--\n(.*?)(?=\n--[A-Z]+--|\Z)', text, re.S)
        title = ' '.join(title_match.group(1).strip().split()) if title_match else ''
        reasons = []
        if eval_re.search(text):
            reasons.append('eval')
        if vv_re.search(text):
            reasons.append('variable-variable')
        has_skipif = bool(re.search(r'--SKIPIF--', text))
        print('\t'.join([
            group,
            row,
            str(path.exists()),
            str(has_skipif),
            ','.join(reasons) if reasons else '-',
            title,
            str(row in regressions),
            str(row in base_pass),
            str(row in current_pass),
            acc_status.get(row, 'MISSING'),
            cand_status.get(row, 'ABSENT'),
            'present' if row in all_results_paths else 'ABSENT',
        ]))
        seen.append((row, has_skipif, tuple(reasons), row in regressions, cand_status.get(row, 'ABSENT'), row in all_results_paths))

print('summary')
print('unique_rows', len(dict.fromkeys(row for rows in row_groups.values() for row in rows)))
print('skipif_rows', [row for row, skipif, *_ in seen if skipif])
print('late_rows', [(row, reasons) for row, _, reasons, *_ in seen if reasons])
print('non_regression_rows', [row for row, _, _, regression, *_ in seen if not regression])
print('candidate_status_counts', Counter(item[4] for item in seen))
print('candidate_all_results_present', sum(1 for item in seen if item[5]))
PY
```

Check path-helper replacement candidates:

```sh
python3 - <<'PY'
from pathlib import Path
import re

php_src=Path('/home/claude/php-src-phpt')
cand=Path('/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377')
acc=Path('/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67')
rows=[
    'php-src/ext/standard/tests/strings/basename_basic.phpt',
    'php-src/ext/standard/tests/strings/highlight_file.phpt',
]

def load_status(path):
    out = {}
    for line in path.read_text(errors='replace').splitlines():
        if '\t' in line:
            status, row = line.split('\t', 1)
            out[row] = status
    return out

reg=set((cand/'regressions-from-latest-published-passes.txt').read_text().splitlines())
acc_status=load_status(acc/'current-status.normalized.tsv')
cand_status=load_status(cand/'current-status.normalized.tsv')
for row in rows:
    text=(php_src/row.removeprefix('php-src/')).read_text(errors='replace')
    title=re.search(r'--TEST--\n(.*?)(?=\n--[A-Z]+--|\Z)', text, re.S)
    print(row, 'regression', row in reg, 'accepted', acc_status.get(row),
          'candidate', cand_status.get(row,'ABSENT'), 'skipif',
          bool(re.search(r'--SKIPIF--', text)),
          'title', ' '.join(title.group(1).strip().split()) if title else '')
PY
```

## Decision

For no-SKIPIF replay prep, keep all eight standard array rows and all three
standard string replacement rows. For the broader standard strings
representative set, either document that `basename_invalid_path.phpt` is a
Windows-gated SKIPIF row or replace it with `basename_basic.phpt` before
labeling the list no-SKIPIF.

No blocker remains for this lane. Focused replay still requires a manager- or
integrator-provided accepted/candidate `PHPC_BIN`; the historical `/tmp`
binaries recorded in earlier reports were already unavailable.
