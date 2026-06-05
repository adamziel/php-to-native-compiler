# 221205Z Standard Array Replay Selector

Lane: 33, developer-83

Scope: read-only M0 selector for standard array regressions in the blocked
`221205Z` public PHPT gate. This report chooses eight low-dependency
representative rows for accepted-vs-candidate replay and explains the expected
signal. No compiler/runtime source edits were made.

## Evidence

- Candidate gate:
  `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377`
- Regression list:
  `regressions-from-latest-published-passes.txt`
- Candidate result/status artifacts:
  `current-status.normalized.tsv`, `all-results.txt`,
  `baseline-passes.normalized.txt`, `current-passes.normalized.txt`
- PHPT source checkout:
  `/home/claude/php-src-phpt`

The blocked gate remains `7197 / 20294` by raw public score, with `1166`
normalized latest-public PASS regressions. The standard array area accounts for
`249` of those regression rows.

## Findings

All `249` standard array regression rows are in the accepted baseline PASS set
and none are in the candidate PASS set. They are also absent from both candidate
status artifacts:

| Candidate artifact view | Rows |
| --- | ---: |
| `ABSENT` from `current-status.normalized.tsv` | 249 |
| `ABSENT` from normalized `all-results.txt` paths | 249 |
| Present as `FAILED`, `BORKED`, `SKIPPED`, or `PASSED` | 0 |

This means the replay set should first answer whether these rows are true
semantic failures or missing-result/control-plane regressions. The candidate
artifacts do not preserve per-row diffs for this standard array subset.

PHPT dependency screening found `246 / 249` rows without `SKIPIF`. The only
`SKIPIF` rows in this subset are the 64-bit rows:

- `php-src/ext/standard/tests/array/end_64bit.phpt`
- `php-src/ext/standard/tests/array/max_basiclong_64bit.phpt`
- `php-src/ext/standard/tests/array/min_basiclong_64bit.phpt`

Directory distribution:

| Area | Rows |
| --- | ---: |
| top-level `ext/standard/tests/array` | 175 |
| `sort/` | 49 |
| `array_walk/` | 14 |
| `range/` | 5 |
| `in_array/` | 4 |
| `gh16649/` | 2 |

Largest filename-derived clusters:

| Cluster | Rows |
| --- | ---: |
| `bug*` / `gh*` historical rows | 30 |
| `array_walk/` | 14 |
| `array_chunk*` | 11 |
| `array_map*` | 7 |
| `extract*` | 6 |
| `sort/array_multisort*` | 6 |
| `array_unshift*` | 5 |
| `range/` | 5 |
| `sort/asort*` | 5 |
| `sort/sort*` | 5 |

## Replay Rows

Use this eight-row replay set first. Each row is a latest-public PASS
regression, has no `SKIPIF`, is absent from both candidate status artifacts, and
uses only core PHP behavior.

| Row | PHPT title | Expected signal |
| --- | --- | --- |
| `php-src/ext/standard/tests/array/array_chunk2.phpt` | `basic array_chunk test` | Small packed-array chunking row with `ValueError` boundary; checks whether simple array builtin execution disappeared or now fails semantically. |
| `php-src/ext/standard/tests/array/array_count_values.phpt` | `array_count_values()` | Exercises scalar bucketing and PHP key coercion without callbacks or external setup. |
| `php-src/ext/standard/tests/array/array_diff_single_array.phpt` | `array_diff() with single array argument` | Compact set-operation row covering single-array `array_diff*` / key-diff behavior plus callback argument plumbing. |
| `php-src/ext/standard/tests/array/array_filter_basic.phpt` | `Test array_filter() function : basic functionality` | Tests callback filtering and default falsey filtering over a small numeric array. |
| `php-src/ext/standard/tests/array/array_map_basic.phpt` | `Test array_map() function : basic functionality` | Representative callback/multi-array traversal row from the seven-row `array_map*` regression cluster. |
| `php-src/ext/standard/tests/array/array_merge.phpt` | `Test array_merge() function` | Covers packed/associative merge semantics, key preservation, key renumbering, and mixed value shapes. |
| `php-src/ext/standard/tests/array/array_walk/array_walk_basic1.phpt` | `Test array_walk() function : basic functionality - regular array` | Representative `array_walk/` traversal callback row from the 14-row subdirectory cluster. |
| `php-src/ext/standard/tests/array/sort/array_multisort_basic1.phpt` | `Test array_multisort() function : basic functionality` | Low-line-count sort row covering parallel array sorting, sort flags, and key preservation effects. |

If these rows replay as candidate semantic failures, implementation lanes should
split by mechanism: array builtin dispatch/coercion, callback invocation,
mutation/key ordering, and sort flags. If they replay as absent-result artifacts,
keep the next lane in M0 runner/result-normalization investigation before
assigning standard array runtime work.

## Commands Run

```sh
python3 - <<'PY'
from pathlib import Path
from collections import Counter

root = Path('/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377')
regs = (root / 'regressions-from-latest-published-passes.txt').read_text().splitlines()
array_rows = [r for r in regs if r.startswith('php-src/ext/standard/tests/array/')]
print(len(regs), len(array_rows))

status = {}
for line in (root / 'current-status.normalized.tsv').read_text().splitlines():
    state, row = line.split('\t', 1)
    status[row] = state
print(Counter(status.get(r, 'ABSENT') for r in array_rows))
PY
```

```sh
python3 - <<'PY'
from pathlib import Path
import re

root = Path('/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377')
base = Path('/home/claude/php-src-phpt')
selected = [
    'php-src/ext/standard/tests/array/array_chunk2.phpt',
    'php-src/ext/standard/tests/array/array_count_values.phpt',
    'php-src/ext/standard/tests/array/array_diff_single_array.phpt',
    'php-src/ext/standard/tests/array/array_filter_basic.phpt',
    'php-src/ext/standard/tests/array/array_map_basic.phpt',
    'php-src/ext/standard/tests/array/array_merge.phpt',
    'php-src/ext/standard/tests/array/array_walk/array_walk_basic1.phpt',
    'php-src/ext/standard/tests/array/sort/array_multisort_basic1.phpt',
]
regs = set((root / 'regressions-from-latest-published-passes.txt').read_text().splitlines())

def section(text, name):
    match = re.search(rf'--{name}--\n(.*?)(?=\n--[A-Z]+--|\Z)', text, re.S)
    return match.group(1).strip() if match else ''

for row in selected:
    text = (base / row.removeprefix('php-src/')).read_text(errors='replace')
    print(row, row in regs, bool(section(text, 'SKIPIF')), section(text, 'TEST'))
PY
```

## Next Action

Replay only the eight selected PASS-regression rows against the accepted and
candidate binaries first. Do not start a broad standard-array implementation
lane from the `249` count alone, because the current evidence points to absent
candidate artifacts rather than preserved per-test semantic diffs.
