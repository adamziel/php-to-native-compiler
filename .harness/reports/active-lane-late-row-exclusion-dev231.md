# Active Lane Late-Row Exclusion Audit

Lane: 109
Current worker: developer-293
Original artifact name: `.harness/reports/active-lane-late-row-exclusion-dev231.md`

Scope: read-only M0/M1 guardrail audit. This report checks the current
`work_lanes.status = in_progress` set for accidental near-term focus on the
planning-late `eval` and variable-variable PHPT rows. No compiler, runtime,
harness, or PHPT source edits were made. No PHPT gate or focused replay was
run.

## Result

Current active replay/repair lane notes do not mention any of the five
late-priority `221205Z` latest-public PASS regression rows exactly. I found no
active lane assignment that asks a worker to implement or replay `eval` or
variable-variable support as first-wave repair work.

The only active lane keyword hits are guardrail or report-only lanes:

| Lane | Title | Why it is not a late-priority implementation focus |
| ---: | --- | --- |
| 109 | Active lane late-row exclusion audit | This lane; read-only guardrail report. |
| 123 | Self-selected source branch quarantine map | Read-only quarantine map that explicitly prevents integrating eval/variable-variable or unsliced source work. |
| 128 | Late-row manifest command reproducibility smoke | Read-only command-shape/count smoke for the late-row manifest; no implementation or full gate. |
| 129 | No-SKIPIF selector sanity for array/string replay | Read-only replay-prep check for SKIPIF and late-priority overlap in standard array/string selected rows. |

No current `in_progress` lane title, branch, worktree, or notes contain any of
these exact late-priority regression paths:

```text
php-src/ext/reflection/tests/bug64936.phpt
php-src/ext/spl/tests/autoloading/bug74372.phpt
php-src/ext/spl/tests/autoloading/spl_autoload_bug48541.phpt
php-src/ext/standard/tests/general_functions/is_callable_variation1.phpt
php-src/ext/tokenizer/tests/token_get_all_variation19.phpt
```

## Late-Row Baseline

The late-priority rule is the existing planning-compatible lexical tag from
the checked-in reports:

- `.harness/reports/phpt-manifest-late-row-tags.md`
- `.harness/reports/late-row-tag-crosscheck.md`
- `.harness/reports/221205Z-late-priority-overlap.md`
- `.harness/reports/late-priority-guardrail-active-replays-dev135.md`

Against the pinned 221205Z regression list, the current overlap is still five
rows:

| Reason | PHPT row |
| --- | --- |
| `eval` | `php-src/ext/reflection/tests/bug64936.phpt` |
| `eval` | `php-src/ext/spl/tests/autoloading/bug74372.phpt` |
| `eval` | `php-src/ext/spl/tests/autoloading/spl_autoload_bug48541.phpt` |
| `variable-variable` lexical tag | `php-src/ext/standard/tests/general_functions/is_callable_variation1.phpt` |
| `eval` | `php-src/ext/tokenizer/tests/token_get_all_variation19.phpt` |

The `is_callable_variation1.phpt` row remains a lexical caveat from a broad
variable-variable pattern; existing reports note that it is not visible proof
of executable variable-variable syntax.

## Active Replay/Repair Check

I treated replay, repair, regression, PHPT, shard, gate, FAILED/BORKED, and
ABSENT lanes as the current active repair/replay surface. The relevant active
lanes at audit time were:

| Lane | Branch | Current focus | Late-row guardrail |
| ---: | --- | --- | --- |
| 8 | `work/developer-277` | Harness test-loop command selection for Rust/PHP project | Control-plane only; no late PHPT row focus. |
| 68 | `work/developer-276` | 221205Z direct FAILED/BORKED regression triage | No exact late row mention in active notes. |
| 69 | `work/developer-278` | 221205Z shard abort root-cause investigation | Control-plane/shard evidence; no exact late row mention. |
| 78 | `work/developer-282` | 221205Z shard rerun smoke without full gate | Directory/smoke evidence; keep prior guardrail excluding the four relevant late rows from semantic probes. |
| 81 | `work/developer-283` | Focused replay: standard filesystem/http rows | Prior guardrail reports zero 221205Z late overlap for this lane scope. |
| 82 | `work/developer-284` | Focused replay: SPL rows | Do not select `bug74372.phpt` or `spl_autoload_bug48541.phpt` as first-wave probes. Current active notes do not name them. |
| 83 | `work/developer-285` | Focused replay: reflection rows | Do not select `bug64936.phpt` as a first-wave probe. Current active notes do not name it. |
| 85 | `work/developer-286` | Focused replay: secondary extension rows | Do not select `token_get_all_variation19.phpt` unless the lane is explicitly late-priority. Current active notes do not name it. |
| 86 | `work/developer-287` | Focused replay: standard scalar/misc rows | Treat `is_callable_variation1.phpt` as a lexical caveat, not a variable-variable implementation target. Current active notes do not name it. |
| 87 | `work/developer-288` | Absent-row rerun prioritizer | Prioritize non-late rows first; current active notes do not name late rows. |
| 113 | `work/developer-297` | Focused replay cookbook cross-check | Report-only checklist lane for replay evidence quality. |
| 117 | `work/developer-301` | First repair-lane evidence readiness review | Report-only readiness review; existing proposal docs explicitly avoid eval-dependent rows. |
| 119 | `work/developer-302` | Lane8/lane100 proof evaluator | Control-plane proof review, not PHPT semantic work. |
| 122 | `work/developer-305` | PHPT binary/wrapper availability recheck | Artifact availability only. |
| 123 | `work/developer-306` | Self-selected source branch quarantine map | Explicitly quarantines eval/variable-variable source work. |
| 128 | `work/developer-311` | Late-row manifest command reproducibility smoke | Late-row metadata/count smoke only; no implementation or replay. |
| 129 | `work/developer-312` | No-SKIPIF selector sanity for array/string replay | Selector sanity check; array/string selected rows have no known late overlap. |
| 130 | `work/developer-313` | 221205Z regression status summary refresh | Status-count report only. |

## Existing Report Artifact Hits

The five late rows are present only in reports that either define the
late-priority set, discuss unsupported-boundary overlap, or already flag
specific rows for deferral:

| Report | Late-row role |
| --- | --- |
| `.harness/reports/221205Z-late-priority-overlap.md` | Defines the five-row 221205Z late overlap. |
| `.harness/reports/221205Z-unsupported-boundary-overlap.md` | Deferral accounting for unsupported/late boundaries. |
| `.harness/reports/late-priority-guardrail-active-replays-dev135.md` | Prior active-lane guardrail; says to exclude the five rows from first-wave semantic probes. |
| `.harness/reports/221205Z-secondary-ext.md` | Mentions tokenizer late row as part of shard analysis. |
| `.harness/reports/221205Z-standard-scalar-misc.md` | Mentions the lexical `is_callable_variation1.phpt` caveat. |
| `.harness/reports/regression-repair-backlog-template.md` | Names late rows as items to defer or separate from first repair lanes. |

I did not find an existing checked-in report artifact that turns any of these
five rows into a first implementation target.

## Commands Run

All commands were run from
`/home/claude/php-to-native-compiler/.harness/worktrees/developer-293`.

Confirm the assigned lane, active lanes, and active late keyword/path mentions:

```sh
python - <<'PY'
import sqlite3, re, json

conn = sqlite3.connect('/home/claude/php-to-native-compiler/.harness/harness.sqlite3')
conn.row_factory = sqlite3.Row

late_rows = [
    'php-src/ext/reflection/tests/bug64936.phpt',
    'php-src/ext/spl/tests/autoloading/bug74372.phpt',
    'php-src/ext/spl/tests/autoloading/spl_autoload_bug48541.phpt',
    'php-src/ext/standard/tests/general_functions/is_callable_variation1.phpt',
    'php-src/ext/tokenizer/tests/token_get_all_variation19.phpt',
]

for lane in conn.execute(
    "select id,title,status,branch,worktree,notes from work_lanes "
    "where status='in_progress' order by id"
):
    text = '\n'.join(str(lane[k] or '') for k in lane.keys())
    hits = [row for row in late_rows if row in text or row.removeprefix('php-src/') in text]
    if hits:
        print(json.dumps({'id': lane['id'], 'title': lane['title'], 'hits': hits}))
PY
```

Verified output:

```text
(none)
```

Recompute the late overlap from the pinned artifacts:

```sh
python - <<'PY'
from pathlib import Path
import re

root = Path('/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377')
php_src = Path('/home/claude/php-src-phpt')
regs = [line.strip() for line in (root / 'regressions-from-latest-published-passes.txt').read_text().splitlines() if line.strip()]

eval_re = re.compile(r'(?i)(^|[^A-Za-z0-9_$])eval\s*\(')
vv_re = re.compile(r'\$\$|\$\{\s*\$|(?i:variable[-_ ]variables?)')

late = []
for row in regs:
    text = (php_src / row.removeprefix('php-src/')).read_text(errors='ignore')
    reasons = []
    if eval_re.search(text):
        reasons.append('eval')
    if vv_re.search(text):
        reasons.append('variable-variable')
    if reasons:
        late.append((row, ','.join(reasons)))

print('regression_count', len(regs))
print('late_overlap_count', len(late))
for row, reason in late:
    print(reason, row, sep='\t')
PY
```

Verified output:

```text
regression_count 1166
late_overlap_count 5
eval	php-src/ext/reflection/tests/bug64936.phpt
eval	php-src/ext/spl/tests/autoloading/bug74372.phpt
eval	php-src/ext/spl/tests/autoloading/spl_autoload_bug48541.phpt
variable-variable	php-src/ext/standard/tests/general_functions/is_callable_variation1.phpt
eval	php-src/ext/tokenizer/tests/token_get_all_variation19.phpt
```

List active lane keyword mentions:

```sh
python - <<'PY'
import sqlite3, re, json
conn = sqlite3.connect('/home/claude/php-to-native-compiler/.harness/harness.sqlite3')
conn.row_factory = sqlite3.Row
keyword_re = re.compile(r'(?i)\b(eval|variable[-_ ]?variable|variable variables|late[- ]priority|late row|late-row)\b|\$\$|\$\{\s*\$')

for lane in conn.execute("select id,title,branch,notes from work_lanes where status='in_progress' order by id"):
    text = '\n'.join(str(lane[k] or '') for k in lane.keys())
    matches = sorted(set(m.group(0) for m in keyword_re.finditer(text)))
    if matches:
        print(json.dumps({'id': lane['id'], 'title': lane['title'], 'branch': lane['branch'], 'matches': matches}, sort_keys=True))
PY
```

Verified keyword hits:

```text
lane 109: eval, late-priority, late-row, variable-variable
lane 123: eval, variable-variable
lane 128: Late-row, eval, late-row, variable-variable
lane 129: late-priority
```

## Boundaries

- This audit uses current SQLite `work_lanes` rows, lane notes, and checked-in
  report artifacts. It does not recursively inspect other worktrees.
- The late-row tag is lexical and inherited from the existing reports. It is
  not a PHP parser and does not filter PHPT sections.
- This audit did not execute PHPTs and does not move public score.
- Late-priority rows remain in the public denominator; the guardrail affects
  scheduling priority only.

## Recommendation

Keep current replay/repair work on non-late rows. Reflection, SPL,
secondary-extension/tokenizer, and scalar/misc replay workers should continue
to avoid the five listed rows as first-wave semantic probes. Lane 128 may
validate late-row manifest command shape because that is a control-plane
guardrail, not implementation work.
