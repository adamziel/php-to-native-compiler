# Late-Priority Guardrail For Active Replay Lanes

Lane: 98, developer-135

Scope: read-only guardrail check for active replay lanes. No compiler,
runtime, harness, or PHPT execution changes were made.

## Summary

The active replay lanes are mostly focused on non-late rows. The concrete
selected standard-array and standard-string replacement replay sets have zero
overlap with the planning-compatible `eval` / variable-variable late-priority
row set.

The only rows that need explicit guardrails are the five late-priority rows
that also appear in the blocked `221205Z` latest-public PASS regression list:

| Row | Late reason | Guardrail owner scope |
| --- | --- | --- |
| `php-src/ext/reflection/tests/bug64936.phpt` | `eval` | Reflection replay lanes should not select this as a first-wave semantic repair probe. |
| `php-src/ext/spl/tests/autoloading/bug74372.phpt` | `eval` | SPL/autoload replay lanes should keep this deferred unless a manager opens eval-specific work. |
| `php-src/ext/spl/tests/autoloading/spl_autoload_bug48541.phpt` | `eval` | SPL/autoload replay lanes should keep this deferred unless a manager opens eval-specific work. |
| `php-src/ext/standard/tests/general_functions/is_callable_variation1.phpt` | variable-variable lexical match | Standard scalar/general-functions replay should mark this as a lexical caveat, not a first-wave variable-variable implementation target. |
| `php-src/ext/tokenizer/tests/token_get_all_variation19.phpt` | `eval` | Secondary-extension/tokenizer replay should keep this deferred unless replay is explicitly about late-priority boundaries. |

These five rows are `5 / 1166` of the blocked PASS-regression set. They remain
in the public denominator and should not move score by exclusion.

## Evidence Inputs

Candidate gate:

`/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377`

Late-priority definition and prior reports:

- `.harness/reports/phpt-manifest-late-row-tags.md`
- `.harness/reports/late-row-tag-crosscheck.md`
- `.harness/reports/221205Z-late-priority-overlap.md`
- `.harness/reports/standard-array-replay-selector.md`
- `.harness/reports/221205Z-standard-strings-replace-replay.md`
- `.harness/reports/first-repair-lane-proposals.md`

Planning-compatible lexical patterns:

```python
eval_re = re.compile(r"(?i)(^|[^A-Za-z0-9_$])eval\s*\(")
vv_re = re.compile(r"\$\$|\$\{\s*\$|(?i:variable[-_ ]variables?)")
```

Verified counts from the pinned PHPT checkout `/home/claude/php-src-phpt`:

| Set | Count |
| --- | ---: |
| Full-corpus late-priority rows | 226 |
| Full-corpus `eval` rows | 142 |
| Full-corpus variable-variable rows | 86 |
| 221205Z latest-public PASS regressions | 1166 |
| 221205Z late-priority regression overlap | 5 |

## Lane Check

| Lane | Active scope checked | 221205Z regressions in scope | Late regressions in scope | Guardrail result |
| --- | --- | ---: | ---: | --- |
| 70 | Focused standard array replay | 249 | 0 | Clear. The eight selected rows have no late-priority overlap. |
| 71 | Focused standard strings replacement replay | 197 | 0 | Clear. The three selected rows have no late-priority overlap. |
| 72 | Reflection and SPL absent-regression replay sampling | 247 | 3 | Exclude `bug64936.phpt`, `bug74372.phpt`, and `spl_autoload_bug48541.phpt` from first-wave replay samples. |
| 78 | Shard rerun smoke around affected standard/SPL/reflection coverage | 1041 | 4 | Keep smoke path directory-focused; do not use the three eval reflection/SPL rows or `is_callable_variation1.phpt` as semantic probes. |
| 80 | Focused standard strings regression rows | 197 | 0 | Clear for 221205Z regressions. Broad full-corpus string late rows are not part of this blocked regression scope. |
| 81 | Focused standard filesystem/http rows | 200 | 0 | Clear for 221205Z regressions. |
| 82 | Focused SPL regression rows | 137 | 2 | Exclude `bug74372.phpt` and `spl_autoload_bug48541.phpt` from first-wave SPL samples. |
| 83 | Focused reflection regression rows | 110 | 1 | Exclude `bug64936.phpt` from first-wave reflection samples. |
| 84 | Focused Zend/classes/sapi regression rows | 22 | 0 | Clear for 221205Z regressions. Many full-corpus Zend rows are late-tagged, so keep selection constrained to the blocked regression list. |
| 85 | Focused secondary extension regression rows | 103 | 1 | Exclude `token_get_all_variation19.phpt` from first-wave tokenizer/secondary-extension samples. |
| 86 | Focused standard scalar/misc regression rows | 142 | 1 | Treat `is_callable_variation1.phpt` as a lexical late-priority caveat; avoid using it to justify variable-variable work. |

Lane 79 is currently queued/duplicate standard-array replay work in the harness
database, not an active owner lane in this worktree. Its candidate rows are the
same standard-array selector rows covered by lane 70, with zero late-priority
overlap.

## Selected Replay Set Checks

Standard array selector rows from lane 70:

| Row | Regression row | Late-priority row |
| --- | --- | --- |
| `php-src/ext/standard/tests/array/array_chunk2.phpt` | yes | no |
| `php-src/ext/standard/tests/array/array_count_values.phpt` | yes | no |
| `php-src/ext/standard/tests/array/array_diff_single_array.phpt` | yes | no |
| `php-src/ext/standard/tests/array/array_filter_basic.phpt` | yes | no |
| `php-src/ext/standard/tests/array/array_map_basic.phpt` | yes | no |
| `php-src/ext/standard/tests/array/array_merge.phpt` | yes | no |
| `php-src/ext/standard/tests/array/array_walk/array_walk_basic1.phpt` | yes | no |
| `php-src/ext/standard/tests/array/sort/array_multisort_basic1.phpt` | yes | no |

Standard string replacement selector rows from lane 71:

| Row | Regression row | Late-priority row |
| --- | --- | --- |
| `php-src/ext/standard/tests/strings/str_replace_basic.phpt` | yes | no |
| `php-src/ext/standard/tests/strings/str_replace_array_refs.phpt` | yes | no |
| `php-src/ext/standard/tests/strings/bug27675.phpt` | yes | no |

## Commands

```sh
python - <<'PY'
from pathlib import Path
import re

php_src = Path('/home/claude/php-src-phpt')
root = Path('/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377')
regs = [line.strip() for line in (root / 'regressions-from-latest-published-passes.txt').read_text().splitlines() if line.strip()]

eval_re = re.compile(r'(?i)(^|[^A-Za-z0-9_$])eval\s*\(')
vv_re = re.compile(r'\$\$|\$\{\s*\$|(?i:variable[-_ ]variables?)')

late = []
for path in sorted(php_src.rglob('*.phpt')):
    rel = 'php-src/' + str(path.relative_to(php_src))
    text = path.read_text(errors='ignore')
    reasons = []
    if eval_re.search(text):
        reasons.append('eval')
    if vv_re.search(text):
        reasons.append('variable-variable')
    if reasons:
        late.append((rel, reasons))

late_map = dict(late)
print('late_total', len(late))
print('eval', sum('eval' in reasons for _, reasons in late))
print('variable_variable', sum('variable-variable' in reasons for _, reasons in late))
print('regressions', len(regs))
print('late_regression_overlap', sum(row in late_map for row in regs))
for row in regs:
    if row in late_map:
        print(','.join(late_map[row]), row, sep='\t')
PY
```

Additional lane-scope joins were computed from `work_lanes` notes and the
scope prefixes listed in the lane table above. This was an audit-only join; no
PHPT rows were executed.

## Recommendation

Replay workers should keep their first-wave samples on non-late rows unless a
manager explicitly opens a late-priority boundary lane. The active standard
array and standard string replacement replay plans already satisfy this
guardrail. Reflection, SPL, secondary-extension/tokenizer, and scalar/misc
workers should explicitly skip the five listed rows when selecting first-wave
semantic probes.
