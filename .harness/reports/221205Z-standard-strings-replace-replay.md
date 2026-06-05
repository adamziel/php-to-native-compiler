# 221205Z Standard Strings Replacement Replay Selector

Lane: 25, developer-83

Scope: read-only M0 selector for standard string replacement/case-insensitive rows in the blocked `221205Z` public PHPT gate. This report answers whether `str_replace()` / `str_ireplace()`-related PASS regressions exist and chooses at most three replay rows. No compiler/runtime source edits were made.

## Evidence

- Candidate gate:
  `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377`
- Regression list:
  `regressions-from-latest-published-passes.txt`
- Candidate result/status artifacts:
  `current-status.normalized.tsv`, `all-results.txt`, `current-passes.normalized.txt`,
  `baseline-passes.normalized.txt`, `shard-*/results.txt`, `shard-*/stdout.log`,
  `shard-*/run-tests.log`
- PHPT source checkout:
  `/home/claude/php-src-phpt`

The overall blocked gate remains `7197 / 20294` by raw public score, with `1166` normalized latest-public PASS regressions. The standard strings directory accounts for `197` regression rows.

## Findings

All `197` standard strings regression rows are absent from both `current-status.normalized.tsv` and normalized `all-results.txt`:

| Candidate artifact status | Rows |
| --- | ---: |
| `ABSENT` | 197 |
| `FAILED` | 0 |
| `BORKED` | 0 |
| `PASSED` | 0 |

That means the selected replacement rows are replay selectors, not direct semantic failures preserved by the candidate status artifacts.

Function-call token scan over the `197` regressed PHPT files:

| Function token | Regression rows |
| --- | ---: |
| `str_replace(` | 4 |
| `str_ireplace(` | 2 |
| `substr_replace(` | 3 |
| `strtr(` | 7 |
| `strncasecmp(` | 3 |
| `strnatcasecmp(` | 1 |
| `stripos(` | 6 |
| `strripos(` | 5 |
| `stristr(` | 2 |

Strict `str_replace()` / `str_ireplace()` PASS-regression candidates:

| Row | Function signal | Notes |
| --- | --- | --- |
| `php-src/ext/standard/tests/strings/str_replace_basic.phpt` | `str_replace()` | Basic operations, `$count` output by-reference variable, and resource argument `TypeError` boundary. |
| `php-src/ext/standard/tests/strings/str_replace_array_refs.phpt` | `str_replace()` | Replacement array values with reference-backed array entries. |
| `php-src/ext/standard/tests/strings/bug71188.phpt` | `str_replace()` | Verifies `str_replace()` does not convert integer search-array entries in the original array to strings. |
| `php-src/ext/standard/tests/strings/bug27675.phpt` | `str_ireplace()` | Small case-insensitive replacement shrink case. |
| `php-src/ext/standard/tests/strings/bug33076.phpt` | `str_ireplace()` | Case-insensitive replacement result length/counting regression row. |

`php-src/ext/standard/tests/strings/substr_replace.phpt` also contains `str_replace()` calls, but those calls format test labels by stripping newlines from `var_export()` output. Its main semantic target is `substr_replace()`, so it is not counted as a strict `str_replace()` replay row here.

## Adjacent Signals

Some replacement-related rows are not latest-public PASS regressions and should not drive this replay selector:

| Row | Candidate status | Why not selected |
| --- | --- | --- |
| `php-src/ext/standard/tests/strings/str_ireplace.phpt` | `PASSED` | Baseline PASS and current PASS; it is not in `regressions-from-latest-published-passes.txt`. |
| `php-src/ext/standard/tests/strings/str_replace_variation1.phpt` | `PASSED` | Baseline PASS and current PASS; not a regression. |
| `php-src/ext/standard/tests/strings/str_replace_variation2.phpt` | `PASSED` | Baseline PASS and current PASS; not a regression. |
| `php-src/ext/standard/tests/strings/str_replace_variation3.phpt` | `FAILED` | Candidate failed, but baseline was not a latest-public PASS, so it is not part of the 1166 regression list. |
| `php-src/ext/standard/tests/strings/str_replace_array_refs2.phpt` | `FAILED` | Candidate failed, but baseline was not a latest-public PASS. Its preserved diff shows `property property` instead of `a property`, which may still be useful after the PASS-regression replay. |

The adjacent failures are useful context but should not be mixed into the first replay batch for the blocked-score regression count.

## Replay Rows

Use this three-row replay set first:

| Row | Expected signal |
| --- | --- |
| `php-src/ext/standard/tests/strings/str_replace_basic.phpt` | Confirms whether the absent PASS regression reproduces in a low-dependency `str_replace()` row that covers basic replacement, `$count` output, and resource `TypeError` handling. |
| `php-src/ext/standard/tests/strings/str_replace_array_refs.phpt` | Probes the reference-backed replacement-array path without pulling in the non-regression `str_replace_array_refs2.phpt` failure. |
| `php-src/ext/standard/tests/strings/bug27675.phpt` | Gives the smallest `str_ireplace()` PASS-regression replay and checks case-insensitive shrinking behavior. |

If those replay as normal semantic failures rather than absent-result artifacts, expand with `bug71188.phpt` for integer search-array preservation and `bug33076.phpt` for `str_ireplace()` length/count behavior. If they replay as absent from candidate status/result normalization, keep the lane in M0 control-plane investigation before assigning implementation work.

## Commands Run

```sh
python3 - <<'PY'
from pathlib import Path
from collections import Counter
root = Path('/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377')
regs = (root / 'regressions-from-latest-published-passes.txt').read_text().splitlines()
strings = [r for r in regs if r.startswith('php-src/ext/standard/tests/strings/')]
print(len(strings))
PY
```

```sh
python3 - <<'PY'
from pathlib import Path
import re
root = Path('/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377')
base = Path('/home/claude/php-src-phpt')
regs = (root / 'regressions-from-latest-published-passes.txt').read_text().splitlines()
strings = [r for r in regs if r.startswith('php-src/ext/standard/tests/strings/')]
for func in ['str_replace','str_ireplace','substr_replace','strtr','strncasecmp','strnatcasecmp','stripos','strripos','stristr']:
    pat = re.compile(r'(?<![A-Za-z0-9_])' + re.escape(func) + r'\s*\(', re.I)
    rows = []
    for row in strings:
        text = (base / row.removeprefix('php-src/')).read_text(errors='replace')
        if pat.search(text):
            rows.append(row)
    print(func, len(rows))
    print('\n'.join(rows))
PY
```

```sh
rg -n 'str_ireplace|str_replace|substr_replace' \
  /home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377 \
  -g '*.txt' -g '*.log' -g '*.tsv'
```

## Next Action

Replay only the three selected PASS-regression rows first. Do not start from `str_ireplace.phpt` or the two preserved direct failures, because those are not rows in the normalized PASS-regression list that is blocking the candidate score.
