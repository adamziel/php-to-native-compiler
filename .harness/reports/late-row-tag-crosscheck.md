# Late-Row Tag Cross-Check

Lane: 24, developer-83

Scope: read-only M1 cross-check of eval and variable-variable late-priority PHPT row counts using `/home/claude/php-src-phpt`. This report independently verifies the planning counts and records the exact scan commands. No compiler/runtime source edits were made.

## Result

The planning-compatible scan confirms the existing counts:

| Set | Count |
| --- | ---: |
| `eval` pattern rows | 142 |
| `variable-variable` pattern rows | 86 |
| Unique combined late-priority rows | 226 |
| Overlap rows | 2 |

No correction is needed for the `142`, `86`, and `226` planning figures when using the same scan definition as the manifest lane.

## Pattern Definition

The confirmed count uses lexical PHPT text matching, not PHP parsing and not PHPT section filtering:

```python
eval_re = re.compile(r"(?i)(^|[^A-Za-z0-9_$])eval\s*\(")
vv_re = re.compile(r"\$\$|\$\{\s*\$|(?i:variable[-_ ]variables?)")
```

This definition intentionally includes rows that mention `variable variables` in PHPT titles, expected deprecation text, or diagnostic text. That is why it produces the planning-compatible `86` variable-variable rows.

A syntax-only scan:

```python
vv_syntax_re = re.compile(r"\$\$|\$\{\s*\$")
```

returns `77` rows, not `86`. The `9` broader-only rows are:

```text
php-src/Zend/tests/exception_in_nested_rope.phpt
php-src/Zend/tests/grammar/bug61681.phpt
php-src/Zend/tests/heredoc_nowdoc/flexible-heredoc-complex-test1.phpt
php-src/Zend/tests/heredoc_nowdoc/flexible-heredoc-complex-test2.phpt
php-src/Zend/tests/heredoc_nowdoc/flexible-heredoc-complex-test3.phpt
php-src/Zend/tests/heredoc_nowdoc/flexible-heredoc-complex-test4.phpt
php-src/Zend/tests/heredoc_nowdoc/warning_during_heredoc_scan_ahead.phpt
php-src/Zend/tests/temporary_cleaning/temporary_cleaning_016.phpt
php-src/ext/opcache/tests/bug69159.phpt
```

Examples:

- `Zend/tests/grammar/bug61681.phpt` contains `${substr(...)}` and expected `${expr}` variable-variable deprecation text.
- `Zend/tests/heredoc_nowdoc/flexible-heredoc-complex-test1.phpt` contains heredoc `${...}` forms and expected `${expr}` variable-variable deprecation text.
- `ext/opcache/tests/bug69159.phpt` has a title about passing a variable variable and contains `${"x$i"}`.

The overlap rows are:

```text
php-src/Zend/tests/assert/expect_015.phpt
php-src/ext/simplexml/tests/000.phpt
```

## Directory Distribution

Eval rows by top bucket:

| Bucket | Rows |
| --- | ---: |
| `Zend/tests` | 88 |
| `tests/lang` | 14 |
| `ext/standard` | 8 |
| `ext/opcache` | 5 |
| `ext/spl` | 5 |
| `ext/date` | 4 |
| `ext/mysqli` | 2 |
| `ext/openssl` | 2 |
| `ext/reflection` | 2 |
| `ext/session` | 2 |
| `ext/tokenizer` | 2 |
| `tests/classes` | 2 |
| one-row buckets | 6 |

Variable-variable rows by top bucket:

| Bucket | Rows |
| --- | ---: |
| `Zend/tests` | 52 |
| `ext/standard` | 14 |
| `tests/lang` | 5 |
| `ext/opcache` | 4 |
| `ext/dom` | 2 |
| `ext/mysqli` | 2 |
| one-row buckets | 7 |

Combined late-priority rows by top bucket:

| Bucket | Rows |
| --- | ---: |
| `Zend/tests` | 139 |
| `ext/standard` | 22 |
| `tests/lang` | 19 |
| `ext/opcache` | 9 |
| `ext/spl` | 5 |
| `ext/date` | 4 |
| `ext/mysqli` | 4 |
| `ext/reflection` | 3 |
| `ext/dom` | 2 |
| `ext/openssl` | 2 |
| `ext/session` | 2 |
| `ext/tokenizer` | 2 |
| `tests/classes` | 2 |
| one-row buckets | 7 |

## Exact Command

```sh
python3 - <<'PY'
from pathlib import Path
import re

base = Path('/home/claude/php-src-phpt')
files = sorted(base.rglob('*.phpt'))

eval_re = re.compile(r'(?i)(^|[^A-Za-z0-9_$])eval\s*\(')
vv_re = re.compile(r'\$\$|\$\{\s*\$|(?i:variable[-_ ]variables?)')
vv_syntax_re = re.compile(r'\$\$|\$\{\s*\$')

def scan(pattern):
    rows = []
    for path in files:
        text = path.read_text(errors='ignore')
        if pattern.search(text):
            rows.append('php-src/' + str(path.relative_to(base)))
    return rows

eval_rows = scan(eval_re)
vv_rows = scan(vv_re)
vv_syntax_rows = scan(vv_syntax_re)
combined = sorted(set(eval_rows) | set(vv_rows))
overlap = sorted(set(eval_rows) & set(vv_rows))
broader_only = sorted(set(vv_rows) - set(vv_syntax_rows))

print('eval', len(eval_rows))
print('variable-variable', len(vv_rows))
print('combined', len(combined))
print('overlap', len(overlap))
print('syntax-only variable-variable', len(vv_syntax_rows))
print('broader-only variable-variable', len(broader_only))
print('overlap rows')
print('\n'.join(overlap))
print('broader-only rows')
print('\n'.join(broader_only))
PY
```

Verified output:

```text
eval 142
variable-variable 86
combined 226
overlap 2
syntax-only variable-variable 77
broader-only variable-variable 9
```

## Unsupported Matching Edges

- No PHP parser is used for these tags.
- No PHPT section filtering is applied.
- Lexical matches in comments, expected output, titles, `SKIPIF`, heredoc, and nowdoc sections are counted.
- Dynamic property, dynamic method, and static-property names are not included unless the row also matches the explicit variable-variable pattern.
- These rows remain in the public denominator; the tag is only for late-priority planning, not for score exclusion.
