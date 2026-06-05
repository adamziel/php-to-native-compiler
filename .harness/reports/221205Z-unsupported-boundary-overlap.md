# 221205Z Unsupported-Boundary Overlap Audit

Lane: 34, developer-83

Scope: read-only M0/M1 audit comparing the `1166` latest-public PASS
regressions from the blocked `221205Z` gate against documented unsupported
boundaries in `docs/SUPPORT.md`, `README.md`, and the current `PLAN.md`. No
compiler/runtime source edits were made.

## Evidence

- Candidate gate:
  `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377`
- Regression list:
  `regressions-from-latest-published-passes.txt`
- Candidate status artifact:
  `current-status.normalized.tsv`
- PHPT source checkout:
  `/home/claude/php-src-phpt`
- Support/priority docs:
  `docs/SUPPORT.md`, `README.md`, `/home/claude/php-to-native-compiler/PLAN.md`
- Supporting reports:
  `.harness/reports/221205Z-late-priority-overlap.md`,
  `.harness/reports/221205Z-source-diff-risk.md`,
  `.harness/reports/221205Z-secondary-ext.md`

The current accepted public score remains `7873 / 20294` at accepted commit
`0b917f67`. The `221205Z` candidate is blocked at `7197 / 20294` with `1166`
PASS regressions, so it cannot move public score.

## Method

This audit uses two levels:

1. Strict deferral: rows overlapping the explicit `PLAN.md` late-priority
   boundaries for `eval` and variable variables.
2. Unsupported-adjacent context: rows whose directory, direct status, or PHPT
   content touches documented bounded/unsupported areas such as broad
   reflection metadata, SPL autoload/object behavior, session/SAPI state,
   filesystem/streams, extension constants, and native-only unsupported
   lowering.

The second level is planning context, not an automatic waiver. These are all
latest-public PASS regressions. Unless an auditor explicitly adjudicates a row,
non-late rows should have stayed PASS for the public metric.

## Top-Level Result

| Bucket | Rows | Candidate status | Interpretation |
| --- | ---: | --- | --- |
| Explicit late-priority deferral (`eval` / variable-variable lexical tag) | 5 | 5 `ABSENT` | May remain deferred from first-wave repair work. Still counted as regressions unless auditor adjudicates them. |
| Non-late rows | 1161 | 1131 `ABSENT`, 27 `FAILED`, 3 `BORKED` | Should have stayed PASS under the current public metric. |
| Broad unsupported-adjacent rows | 481 | 481 `ABSENT` | Need replay/control-plane classification; not direct proof that rows should remain unsupported. |
| Direct unsupported-adjacent failures | 10 | 7 `FAILED`, 3 `BORKED` | Narrow M0-direct repair/adjudication candidates. |
| Native compile-only unsupported boundaries | 0 | n/a | Not relevant to this PHPT regression gate because the public compatibility surface is `phpc run`. |

Strict answer: only the five late-priority rows are reasonable "remain
unsupported/deferred" candidates under current planning rules. The other `1161`
non-late PASS regressions should have stayed PASS or need explicit auditor
adjudication.

## Explicit Late-Priority Rows

| Row | Reason | Note |
| --- | --- | --- |
| `php-src/ext/reflection/tests/bug64936.phpt` | `eval` | Clear executable `eval()` in the PHPT file. |
| `php-src/ext/spl/tests/autoloading/bug74372.phpt` | `eval` | Clear executable `eval()` in an SPL autoloading row. |
| `php-src/ext/spl/tests/autoloading/spl_autoload_bug48541.phpt` | `eval` | Clear executable `eval()` in an SPL autoloading row. |
| `php-src/ext/standard/tests/general_functions/is_callable_variation1.phpt` | variable-variable lexical match | Caveat: the match is a `$$$` string literal, not visible executable variable-variable syntax. |
| `php-src/ext/tokenizer/tests/token_get_all_variation19.phpt` | `eval` | Clear executable `eval($script)`. |

These rows should not drive first-wave implementation lanes while `eval` and
variable variables remain late-priority. They also should not be silently
removed from denominator accounting without an explicit auditor decision.

## Unsupported-Adjacent Rows

These rows overlap broad support-matrix boundaries, but they are not automatic
deferrals because they were accepted baseline PASS rows.

| Context | Rows | Status | Support boundary |
| --- | ---: | --- | --- |
| Broad reflection metadata | 110 | 110 `ABSENT` | `docs/SUPPORT.md` names bounded reflection metadata and broad reflection parity as unsupported. |
| SPL object/iterator breadth | 130 | 130 `ABSENT` | SPL object behavior is broader than the bounded object/iterator/autoload slices. |
| SPL autoload lifecycle | 7 | 7 `ABSENT` | Autoload lifecycle beyond bounded string-callback/default-probe slices is unsupported. |
| Session/SAPI state breadth | 7 | 7 `ABSENT` | Exact SAPI/session persistence/header/cache behavior remains bounded. |
| Filesystem/stream breadth | 170 | 170 `ABSENT` | Filesystem/stream support is bounded to local paths/resources and exact PHP parity remains limited. |
| URI extension surface | 41 | 41 `ABSENT` | No broad URI extension support claim should be inferred from docs. |
| POSIX extension surface | 16 | 16 `ABSENT` | POSIX support is a bounded helper slice, not full extension parity. |

Unique rows in these broad unsupported-adjacent groups: `481`.

Recommended classification for these `481`: `M0-replay`, not "ignore as
unsupported". If replay proves the candidate semantically fails inside a
documented unsupported edge, the manager/auditor can decide whether to defer it.
Until then, their dominant symptom is still candidate artifact absence.

## Direct Unsupported-Adjacent Failures

These rows have direct candidate statuses and overlap support-matrix boundaries
such as readonly/internal property diagnostics or extension constant catalogs:

| Row | Status | Boundary |
| --- | --- | --- |
| `php-src/ext/standard/tests/directory/DirectoryClass_readonly_handle.phpt` | `FAILED` | Internal readonly property diagnostics. |
| `php-src/ext/standard/tests/directory/DirectoryClass_readonly_path.phpt` | `FAILED` | Internal readonly property diagnostics. |
| `php-src/ext/date/tests/DatePeriod_modify_readonly_property.phpt` | `FAILED` | Internal readonly property diagnostics. |
| `php-src/ext/date/tests/DatePeriod_properties2.phpt` | `FAILED` | Internal readonly property diagnostics. |
| `php-src/ext/bcmath/tests/number/properties_unset.phpt` | `FAILED` | Internal readonly property unset diagnostics. |
| `php-src/ext/bcmath/tests/number/properties_write_error.phpt` | `FAILED` | Internal readonly property write diagnostics. |
| `php-src/ext/xmlreader/tests/014.phpt` | `FAILED` | XMLReader virtual/readonly property diagnostics. |
| `php-src/ext/intl/tests/rangeformatter/rangeformatter_icu63_compatibility.phpt` | `BORKED` | `SKIPIF` references missing `INTL_ICU_VERSION`; full extension constant catalogs are not broadly documented. |
| `php-src/ext/openssl/tests/openssl_libctx_without_zts_argon.phpt` | `BORKED` | `SKIPIF` references missing `ZEND_THREAD_SAFE`; constant exposure boundary. |
| `php-src/ext/pcre/tests/grep2.phpt` | `BORKED` | `SKIPIF` references missing `PCRE_JIT_SUPPORT`; constant exposure boundary. |

Recommended classification for these `10`: `M0-direct`. They should either be
fixed narrowly or explicitly adjudicated; they should not be mixed with the
`1136` absent-row control-plane problem.

## Lexical Caveats

Additional unsupported-looking tokens appear in PHPT text, but they are not
enough to defer rows automatically:

| Pattern | Rows | Caveat |
| --- | ---: | --- |
| heredoc/nowdoc marker `<<<` | 67 | Often appears in data strings or expected output; needs row-level review. |
| short echo tag `<?=` | 1 | `tokenizer/bug76437.phpt` tokenizes `<?=$a?>` as a string input; it is not necessarily parser execution support. |
| `match (` | 1 | `str_increment_polyfill.phpt` has a real PHP `match`; because it was accepted PASS, classify by replay/adjudication rather than automatic deferral. |
| backtick characters | 2 | The hits are string contents/diagnostic formatting, not shell execution syntax. |
| namespace declarations | 2 | Namespace support is bounded, but these rows were accepted PASS and need replay if they fail. |

## Rows That Should Have Stayed PASS

Under current planning rules:

- `1161` non-late rows should have stayed PASS or should receive explicit
  auditor adjudication.
- `481` broad unsupported-adjacent rows need replay before anyone assigns
  semantic responsibility or deferral.
- `673` non-late, non-adjacent rows have no obvious support-boundary excuse
  from this audit and should be treated as ordinary regressions.
- Direct `FAILED`/`BORKED` rows should be split into narrow repair lanes or
  adjudication lanes.

Do not use `docs/SUPPORT.md` as a blanket waiver for latest-public PASS
regressions. The support matrix names bounded support and unsupported edges;
the public metric still rejects candidate score movement with unadjudicated
PASS regressions.

## Commands Run

```sh
rg -n -i 'unsupported|not implemented|not supported|remain unsupported|eval|variable-variable|variable variable|variable variables' docs/SUPPORT.md
sed -n '620,760p' README.md
sed -n '920,1018p' README.md
```

```sh
python3 - <<'PY'
from pathlib import Path
from collections import Counter

root = Path('/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377')
rows = [line.strip() for line in (root / 'regressions-from-latest-published-passes.txt').read_text().splitlines() if line.strip()]
status = {}
for line in (root / 'current-status.normalized.tsv').read_text().splitlines():
    state, row = line.split('\t', 1)
    status[row] = state
print(len(rows), Counter(status.get(row, 'ABSENT') for row in rows))
PY
```

```sh
python3 - <<'PY'
from pathlib import Path
import re

root = Path('/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377')
base = Path('/home/claude/php-src-phpt')
rows = [line.strip() for line in (root / 'regressions-from-latest-published-passes.txt').read_text().splitlines() if line.strip()]
eval_re = re.compile(r'(?i)(^|[^A-Za-z0-9_$])eval\s*\(')
vv_re = re.compile(r'\$\$|\$\{\s*\$|(?i:variable[-_ ]variables?)')
late = []
for row in rows:
    text = (base / row.removeprefix('php-src/')).read_text(errors='ignore')
    if eval_re.search(text) or vv_re.search(text):
        late.append(row)
print(len(late))
print('\n'.join(late))
PY
```

## Next Action

Keep first-wave implementation work away from the five late-priority rows. For
everything else, start with replay/control-plane classification for absent rows
and narrow repair/adjudication for direct failures. Do not treat broad
unsupported-boundary prose as sufficient to accept a public PASS regression.
