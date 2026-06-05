# 221205Z Late-Priority Regression Overlap

Lane: 30, developer-83

Scope: read-only M0/M3 boundary audit. This report intersects the `1166` latest-public PASS regression rows from the blocked `221205Z` gate with the planning-compatible `eval` and variable-variable PHPT content patterns. No compiler/runtime source edits were made.

## Evidence

- Candidate gate:
  `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377`
- Regression list:
  `regressions-from-latest-published-passes.txt`
- Candidate result/status artifacts:
  `current-status.normalized.tsv`, `all-results.txt`, `baseline-passes.normalized.txt`,
  `current-passes.normalized.txt`, `pass-regression-summary.tsv`
- PHPT source checkout:
  `/home/claude/php-src-phpt`
- Full-corpus late-row cross-check:
  `.harness/reports/late-row-tag-crosscheck.md`

The blocked gate remains `7197 / 20294` by raw public score, with `1166` normalized latest-public PASS regressions.

## Pattern Definition

This lane uses the same planning-compatible lexical scan recorded in the late-row cross-check:

```python
eval_re = re.compile(r"(?i)(^|[^A-Za-z0-9_$])eval\s*\(")
vv_re = re.compile(r"\$\$|\$\{\s*\$|(?i:variable[-_ ]variables?)")
```

This is not a PHP parser and does not filter PHPT sections. It is intentionally a planning tag, not an implementation proof.

## Counts

| Set | Count |
| --- | ---: |
| Total latest-public PASS regressions | 1166 |
| Regressions matching `eval` pattern | 4 |
| Regressions matching variable-variable pattern | 1 |
| Regressions matching both patterns | 0 |
| Unique late-priority overlap rows | 5 |
| Non-late regression rows | 1161 |

All five overlap rows have the same candidate artifact status:

| Candidate artifact view | Count |
| --- | ---: |
| Present in accepted baseline PASS set | 5 |
| Absent from current PASS set | 5 |
| Absent from `current-status.normalized.tsv` | 5 |
| Absent from normalized `all-results.txt` | 5 |

The overlap is therefore a very small fraction of the blocked regression set: `5 / 1166`, or about `0.43%`. Removing or deferring late-priority work does not explain the `221205Z` regression cliff. It only identifies five rows that should not be first repair-lane targets while `eval` and variable-variable support remain late-priority.

## Overlap Rows

| Row | Reason | PHPT title | Matching source line |
| --- | --- | --- | --- |
| `php-src/ext/reflection/tests/bug64936.phpt` | `eval` | `ReflectionMethod::getDocComment() uses left over doc comment from previous scanner run` | `eval('class A { }');` |
| `php-src/ext/spl/tests/autoloading/bug74372.phpt` | `eval` | `Bug #74372: autoloading file with syntax error uses next autoloader, may hide parse error` | `eval("ha ha ha");` |
| `php-src/ext/spl/tests/autoloading/spl_autoload_bug48541.phpt` | `eval` | `SPL: spl_autoload_register() Bug #48541: registering multiple closures fails with memleaks` | `eval('class ' . $class . '{function __construct(){echo "foo\n";}}');` |
| `php-src/ext/standard/tests/general_functions/is_callable_variation1.phpt` | `variable-variable` | `Test is_callable() function : usage variations - undefined functions` | string literal containing `==%%%***$$$@@@!!` |
| `php-src/ext/tokenizer/tests/token_get_all_variation19.phpt` | `eval` | `Reconstructing a script using token_get_all()` | `eval($script);` |

The `is_callable_variation1.phpt` row is a lexical caveat: it matches the broad variable-variable regex because a string literal contains `$$$`, not because the row visibly exercises executable variable-variable syntax. Keep it tagged for consistency with the planning-compatible scan, but do not treat it as proof that variable-variable implementation would repair that row.

## Bucket Distribution

| Bucket | Rows |
| --- | ---: |
| `ext/spl` | 2 |
| `ext/reflection` | 1 |
| `ext/standard` | 1 |
| `ext/tokenizer` | 1 |

No `Zend/tests`, `tests/lang`, `tests/classes`, or `sapi` rows from the 1166 PASS-regression list overlap with the late-priority tags.

## Interpretation

The 226 full-corpus late-priority rows remain useful for planning, but only five of the current 1166 latest-public PASS regressions fall into that late-priority set. The practical split is:

- Defer the four clear `eval` rows and the one lexical variable-variable-pattern row from first-wave repair lanes.
- Continue focusing M0 repair planning on the remaining 1161 non-late regressions.
- Do not remove any of the five from public denominator accounting. They remain regression rows; the tag only affects priority.

## Exact Command

```sh
python3 - <<'PY'
from pathlib import Path
import re

root = Path('/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377')
php_src = Path('/home/claude/php-src-phpt')
regs = (root / 'regressions-from-latest-published-passes.txt').read_text().splitlines()

eval_re = re.compile(r'(?i)(^|[^A-Za-z0-9_$])eval\s*\(')
vv_re = re.compile(r'\$\$|\$\{\s*\$|(?i:variable[-_ ]variables?)')

rows = []
for row in regs:
    text = (php_src / row.removeprefix('php-src/')).read_text(errors='ignore')
    reasons = []
    if eval_re.search(text):
        reasons.append('eval')
    if vv_re.search(text):
        reasons.append('variable-variable')
    if reasons:
        rows.append((row, reasons))

print('regressions', len(regs))
print('late overlap rows', len(rows))
print('eval rows', sum(1 for _, reasons in rows if 'eval' in reasons))
print('variable-variable rows', sum(1 for _, reasons in rows if 'variable-variable' in reasons))
print('both rows', sum(1 for _, reasons in rows if len(reasons) == 2))
for row, reasons in rows:
    print(','.join(reasons), row, sep='\t')
PY
```

Verified output:

```text
regressions 1166
late overlap rows 5
eval rows 4
variable-variable rows 1
both rows 0
eval	php-src/ext/reflection/tests/bug64936.phpt
eval	php-src/ext/spl/tests/autoloading/bug74372.phpt
eval	php-src/ext/spl/tests/autoloading/spl_autoload_bug48541.phpt
variable-variable	php-src/ext/standard/tests/general_functions/is_callable_variation1.phpt
eval	php-src/ext/tokenizer/tests/token_get_all_variation19.phpt
```

## Next Action

For first repair-lane proposals, exclude these five rows from high-priority non-late clusters unless the replay itself is specifically about late-priority boundaries. The current blocked score is dominated by the 1161 non-late regressions and by candidate artifact absence/status issues, not by `eval` or variable-variable work.
