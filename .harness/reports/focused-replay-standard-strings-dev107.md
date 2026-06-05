# Focused Replay: Standard Strings Rows (developer-107 replacement)

Lane 80 read-only report produced by developer-141 after developer-107 was
retired as stale capacity. This report narrows the completed
`221205Z-standard-strings.md` evidence to eight representative
`ext/standard/tests/strings` PASS regressions and classifies whether the
blocked candidate shows row-level semantic failures or control-plane absence.

No compiler, runtime, source, test-list, or PHPT gate files were edited. No full
PHPT gate was run.

## Inputs

- Completed standard strings evidence:
  `/home/claude/php-to-native-compiler/.harness/reports/221205Z-standard-strings.md`
- Accepted baseline artifact root:
  `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67`
- Blocked candidate artifact root:
  `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377`
- Accepted public/source head: `0b917f67a37d9ca9779d77f87173b628431c2425`
- Candidate public/source head: `56fe9377fb46be00db5fdd30c966fdba406dc581`
- php-src replay checkout: `/home/claude/php-src-phpt`
- php-src pin verified in that checkout:
  `f97ff597429a2fe633665a7e02d97c8077f9f90f`

## Representative Rows

The selected rows cover distinct string-function clusters from the prior shard
report and avoid eval and variable-variable work.

| Row | PHPT title | Cluster |
| --- | --- | --- |
| `php-src/ext/standard/tests/strings/005.phpt` | `highlight_string(), output buffer and error level` | legacy/basic output-buffer string helper |
| `php-src/ext/standard/tests/strings/bin2hex.phpt` | `bin2hex() function` | binary/encoding |
| `php-src/ext/standard/tests/strings/basename_invalid_path.phpt` | `Test basename() function : usage variations with invalid paths` | path helper |
| `php-src/ext/standard/tests/strings/md5.phpt` | `md5() with ASCII output` | hash/encoding |
| `php-src/ext/standard/tests/strings/sprintf_variation3.phpt` | `Test sprintf() function : usage variations - int formats with int values` | printf formatting |
| `php-src/ext/standard/tests/strings/strtr_with_reference.phpt` | `strtr() with references` | translate/reference-sensitive |
| `php-src/ext/standard/tests/strings/html_entity_decode_cp866.phpt` | `Translation of HTML entities for encoding CP866` | HTML/entity/charset |
| `php-src/ext/standard/tests/strings/parse_str_null_bytes.phpt` | `parse_str() rejects null bytes` | parse_str/input validation |

## Artifact Status Join

Command:

```sh
python3 - <<'PY'
from pathlib import Path
ACC=Path('/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67')
CAND=Path('/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377')
rows=[
'php-src/ext/standard/tests/strings/005.phpt',
'php-src/ext/standard/tests/strings/bin2hex.phpt',
'php-src/ext/standard/tests/strings/basename_invalid_path.phpt',
'php-src/ext/standard/tests/strings/md5.phpt',
'php-src/ext/standard/tests/strings/sprintf_variation3.phpt',
'php-src/ext/standard/tests/strings/strtr_with_reference.phpt',
'php-src/ext/standard/tests/strings/html_entity_decode_cp866.phpt',
'php-src/ext/standard/tests/strings/parse_str_null_bytes.phpt',
]
def load_status(path):
    d={}
    for line in path.read_text(errors='replace').splitlines():
        if not line.strip(): continue
        status,p=line.split('\t',1)
        d.setdefault(p,[]).append(status)
    return d
def norm_result_path(raw):
    marker='/php-src/'
    if marker in raw:
        return 'php-src/'+raw.split(marker,1)[1]
    return raw
def load_results(path):
    d={}
    for line in path.read_text(errors='replace').splitlines():
        if not line.strip(): continue
        status,p=line.split('\t',1)
        d.setdefault(norm_result_path(p),[]).append(status)
    return d
acc_status=load_status(ACC/'current-status.normalized.tsv')
cand_status=load_status(CAND/'current-status.normalized.tsv')
acc_results=load_results(ACC/'all-results.txt')
cand_results=load_results(CAND/'all-results.txt')
reg=set((CAND/'regressions-from-latest-published-passes.txt').read_text().splitlines())
print('row\tin_regression_list\taccepted_status\taccepted_all_results\tcandidate_status\tcandidate_all_results')
for r in rows:
    def fmt(vals): return ','.join(vals) if vals else 'MISSING'
    print('\t'.join([r, str(r in reg), fmt(acc_status.get(r,[])), fmt(acc_results.get(r,[])), fmt(cand_status.get(r,[])), fmt(cand_results.get(r,[]))]))
PY
```

Result:

```text
row	in_regression_list	accepted_status	accepted_all_results	candidate_status	candidate_all_results
php-src/ext/standard/tests/strings/005.phpt	True	PASSED	PASSED	MISSING	MISSING
php-src/ext/standard/tests/strings/bin2hex.phpt	True	PASSED	PASSED	MISSING	MISSING
php-src/ext/standard/tests/strings/basename_invalid_path.phpt	True	PASSED	PASSED	MISSING	MISSING
php-src/ext/standard/tests/strings/md5.phpt	True	PASSED	PASSED	MISSING	MISSING
php-src/ext/standard/tests/strings/sprintf_variation3.phpt	True	PASSED	PASSED	MISSING	MISSING
php-src/ext/standard/tests/strings/strtr_with_reference.phpt	True	PASSED	PASSED	MISSING	MISSING
php-src/ext/standard/tests/strings/html_entity_decode_cp866.phpt	True	PASSED	PASSED	MISSING	MISSING
php-src/ext/standard/tests/strings/parse_str_null_bytes.phpt	True	PASSED	PASSED	MISSING	MISSING
```

All eight rows are accepted-baseline PASS rows and candidate regressions only
because the candidate has no normalized status row and no aggregate result row
for them.

## Assigned Shard Count Cross-Check

Command:

```sh
python3 - <<'PY'
from pathlib import Path
CAND=Path('/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377')
rows=[r for r in (CAND/'regressions-from-latest-published-passes.txt').read_text().splitlines() if r.startswith('php-src/ext/standard/tests/strings/')]
status_paths={line.split('\t',1)[1] for line in (CAND/'current-status.normalized.tsv').read_text(errors='replace').splitlines() if '\t' in line}
all_paths=set()
for line in (CAND/'all-results.txt').read_text(errors='replace').splitlines():
    if '\t' not in line: continue
    _,p=line.split('\t',1)
    marker='/php-src/'
    if marker in p:
        p='php-src/'+p.split(marker,1)[1]
    all_paths.add(p)
print('standard_strings_regressions', len(rows))
print('candidate_status_present', sum(r in status_paths for r in rows))
print('candidate_status_missing', sum(r not in status_paths for r in rows))
print('candidate_all_results_present', sum(r in all_paths for r in rows))
print('candidate_all_results_missing', sum(r not in all_paths for r in rows))
PY
```

Result:

```text
standard_strings_regressions 197
candidate_status_present 0
candidate_status_missing 197
candidate_all_results_present 0
candidate_all_results_missing 197
```

This matches the completed lane evidence: all 197 assigned standard-string PASS
regressions are absent from candidate row/status output.

## Focused Replay Preflight

The saved replay cookbook requires historical accepted and candidate release
`phpc` binaries through `PHPC_BIN`. The wrapper and pinned php-src checkout are
present, but both historical binaries are absent.

Command:

```sh
for p in /tmp/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67/cargo-target/release/phpc /tmp/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/cargo-target/release/phpc; do
  if test -x "$p"; then printf 'present\t%s\n' "$p"; else printf 'missing\t%s\n' "$p"; fi
done
```

Result:

```text
missing	/tmp/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67/cargo-target/release/phpc
missing	/tmp/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/cargo-target/release/phpc
```

Focused `run-tests.php` replay was therefore not executed. Running those rows
without the historical binaries would test a broken replay setup rather than
accepted-vs-candidate compiler behavior. This is a replay availability limit,
not evidence of string semantic failure.

## Candidate Harness Symptoms

Gate-level accounting:

```text
counts.tsv: 7197 passed, 8851 failed, 2222 skipped, 669 borked, 2 warned
pass-regression-summary.tsv: baseline_passes=7869, current_passes=7196, pass_regressions=1166
aggregate-warnings.tsv: missing_results=0
```

Shard-level string coverage command result:

```text
shard-01	122	BORKED=1,FAILED=10,PASSED=101,SKIPPED=10
shard-02	123	FAILED=15,PASSED=93,SKIPPED=15
shard-03	0
shard-04	0
shard-05	122	FAILED=7,PASSED=106,SKIPPED=9
shard-06	122	FAILED=7,PASSED=107,SKIPPED=8
```

Shard abort evidence:

```text
/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/shard-04/stdout.log:2564:ERROR: cannot open directory: /tmp/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/run-tests-harnesses/shard-04/ext/pdo/tests
/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/shard-03/stdout.log:2372:ERROR: cannot open directory: /tmp/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/run-tests-harnesses/shard-03/ext/pdo/tests
```

The gate aggregate says every shard produced a result file, but that is weaker
than row completeness. The selected rows and all 197 assigned strings
regressions are absent from candidate result/status output.

## Classification

| Row | Classification | Reason |
| --- | --- | --- |
| `005.phpt` | absent/control-plane | Accepted PASS; candidate status and aggregate result are missing. No candidate diff exists. |
| `bin2hex.phpt` | absent/control-plane | Accepted PASS; candidate status and aggregate result are missing. No evidence that `bin2hex()` itself regressed. |
| `basename_invalid_path.phpt` | absent/control-plane | Accepted PASS; candidate status and aggregate result are missing. No candidate row-level path-helper output exists. |
| `md5.phpt` | absent/control-plane | Accepted PASS; candidate status and aggregate result are missing. No hash semantic failure is shown for this row. |
| `sprintf_variation3.phpt` | absent/control-plane | Accepted PASS; candidate status and aggregate result are missing. No formatting diff exists. |
| `strtr_with_reference.phpt` | absent/control-plane, semantic unknown | Accepted PASS; candidate status and aggregate result are missing. Reference-sensitive semantics remain unclassified without replay output. |
| `html_entity_decode_cp866.phpt` | absent/control-plane | Accepted PASS; candidate status and aggregate result are missing. No charset/entity diff exists. |
| `parse_str_null_bytes.phpt` | absent/control-plane | Accepted PASS; candidate status and aggregate result are missing. No null-byte validation diff exists. |

## Conclusion

For this focused standard-strings sample, the blocked 221205Z candidate does
not demonstrate semantic regressions in string functions. The representative
rows are PASS regressions because candidate row/status output is absent. The
proper bucket is harness/control-plane row absence with semantic status unknown
until a replay can be run with restored or rebuilt accepted and candidate
`PHPC_BIN` binaries.

The most deterministic next action is to fix or reproduce the PHPT harness
directory/list completeness issue, especially the shard 03/04 copied harness
`ext/pdo/tests` aborts and the aggregator's weak `missing_results=0` check.
