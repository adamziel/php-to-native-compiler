# Focused Replay: Zend/classes/sapi Rows (developer-222 replacement)

Lane 84 read-only report for the blocked candidate gate
`phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377`.
This narrows the completed Zend/classes/sapi shard evidence to eight
representative latest-public PASS regressions and classifies whether the
candidate evidence shows semantic failure, control-plane absence, or replay
unavailability.

No compiler/runtime source files were edited. No full PHPT gate was run.
`DEVELOPMENT.md` was requested by the lane instructions but is absent under
`/home/claude/php-to-native-compiler`.

## Inputs

- Completed shard report:
  `/home/claude/php-to-native-compiler/.harness/reports/221205Z-zend-classes-sapi.md`
- Accepted baseline artifact root:
  `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67`
- Blocked candidate artifact root:
  `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377`
- Accepted public/source head: `0b917f67a37d9ca9779d77f87173b628431c2425`
- Candidate public/source head: `56fe9377fb46be00db5fdd30c966fdba406dc581`
- php-src checkout: `/home/claude/php-src-phpt`
- php-src pin verified in that checkout:
  `f97ff597429a2fe633665a7e02d97c8077f9f90f`
- Wrapper: `/home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper`

Gate score context:

```text
accepted public-comparable-score.tsv: passed=7873 pinned_runnable=20294 public_percent=38.79
candidate public-comparable-score.tsv: passed=7197 pinned_runnable=20294 public_percent=35.46
candidate pass-regression-summary.tsv: baseline_passes=7869 current_passes=7196 pass_regressions=1166
candidate counts.tsv: passed=7197 failed=8851 skipped=2222 borked=669 warned=2 runnable=16058 percent=44.82
```

## Shard Cross-Check

The completed shard owns 22 latest-public PASS regressions:

```text
owned_rows 22
by_prefix {'sapi/cli': 3, 'tests/classes': 4, 'Zend/tests': 15}
candidate_status_counts {'FAILED': 19, 'MISSING': 3}
```

Command:

```sh
python - <<'PY'
from pathlib import Path
CAND=Path('/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377')
rows=[r for r in (CAND/'regressions-from-latest-published-passes.txt').read_text().splitlines() if r.startswith(('php-src/Zend/tests/','php-src/tests/classes/','php-src/sapi/cli/'))]
status={}
for line in (CAND/'current-status.normalized.tsv').read_text(errors='replace').splitlines():
    if '\t' in line:
        s,p=line.split('\t',1); status[p]=s
from collections import Counter
by_prefix=Counter()
counts=Counter()
for r in rows:
    if r.startswith('php-src/Zend/tests/'):
        by_prefix['Zend/tests']+=1
    elif r.startswith('php-src/tests/classes/'):
        by_prefix['tests/classes']+=1
    elif r.startswith('php-src/sapi/cli/'):
        by_prefix['sapi/cli']+=1
    counts[status.get(r,'MISSING')]+=1
print('owned_rows', len(rows))
print('by_prefix', dict(by_prefix))
print('candidate_status_counts', dict(sorted(counts.items())))
PY
```

## Representative Rows

These eight rows cover the three assigned areas and avoid eval and variable
variables.

| Row | PHPT title | Bucket |
| --- | --- | --- |
| `php-src/sapi/cli/tests/002.phpt` | `running code with -r` | CLI wrapper/result coverage |
| `php-src/sapi/cli/tests/021.phpt` | `CLI shell shebang` | CLI wrapper/result coverage |
| `php-src/sapi/cli/tests/bug70006.phpt` | `Bug #70006 (cli - function with default arg = STDOUT crash output)` | CLI wrapper/result coverage |
| `php-src/tests/classes/ctor_dtor.phpt` | `ZE2 The new constructor/destructor is called` | destructor timing |
| `php-src/tests/classes/factory_and_singleton_002.phpt` | `ZE2 factory and singleton, test 2` | protected destructor lifecycle |
| `php-src/Zend/tests/assert/expect_011.phpt` | `test overloaded __toString on custom exception` | assertion/throwable stringification |
| `php-src/Zend/tests/property_hooks/gh19548_002.phpt` | `GH-19548: Segfault when using inherited properties and opcache (multiple properties)` | property hook/interface compatibility |
| `php-src/Zend/tests/type_declarations/variance/internal_parent/unresolvable_inheritance_check_param.phpt` | `Test unresolvable inheritance check due to unavailable parameter type when the parent has a tentative return type.` | internal inheritance diagnostic |

Source scan result:

```text
php-src/sapi/cli/tests/002.phpt	no eval/$$ marker
php-src/sapi/cli/tests/021.phpt	no eval/$$ marker
php-src/sapi/cli/tests/bug70006.phpt	no eval/$$ marker
php-src/tests/classes/ctor_dtor.phpt	no eval/$$ marker
php-src/tests/classes/factory_and_singleton_002.phpt	no eval/$$ marker
php-src/Zend/tests/assert/expect_011.phpt	no eval/$$ marker
php-src/Zend/tests/property_hooks/gh19548_002.phpt	no eval/$$ marker
php-src/Zend/tests/type_declarations/variance/internal_parent/unresolvable_inheritance_check_param.phpt	no eval/$$ marker
```

## Artifact Status Join

Command:

```sh
python - <<'PY'
from pathlib import Path
ACC=Path('/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67')
CAND=Path('/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377')
PHP_SRC=Path('/home/claude/php-src-phpt')
rows=[
'php-src/sapi/cli/tests/002.phpt',
'php-src/sapi/cli/tests/021.phpt',
'php-src/sapi/cli/tests/bug70006.phpt',
'php-src/tests/classes/ctor_dtor.phpt',
'php-src/tests/classes/factory_and_singleton_002.phpt',
'php-src/Zend/tests/assert/expect_011.phpt',
'php-src/Zend/tests/property_hooks/gh19548_002.phpt',
'php-src/Zend/tests/type_declarations/variance/internal_parent/unresolvable_inheritance_check_param.phpt',
]
def load_status(path):
    out={}
    for line in path.read_text(errors='replace').splitlines():
        if '\t' not in line: continue
        status,p=line.split('\t',1)
        out.setdefault(p,[]).append(status)
    return out
def norm_path(p):
    marker='/php-src/'
    if marker in p:
        return 'php-src/'+p.split(marker,1)[1]
    return p
def load_results(path):
    out={}
    for line in path.read_text(errors='replace').splitlines():
        if '\t' not in line: continue
        status,p=line.split('\t',1)
        out.setdefault(norm_path(p),[]).append(status)
    return out
def title(row):
    p=PHP_SRC / row.removeprefix('php-src/')
    lines=p.read_text(errors='replace').splitlines()
    for i,line in enumerate(lines):
        if line.strip()=='--TEST--':
            return lines[i+1].strip() if i+1 < len(lines) else ''
    return ''
acc_status=load_status(ACC/'current-status.normalized.tsv')
cand_status=load_status(CAND/'current-status.normalized.tsv')
acc_results=load_results(ACC/'all-results.txt')
cand_results=load_results(CAND/'all-results.txt')
reg=set((CAND/'regressions-from-latest-published-passes.txt').read_text().splitlines())
print('row\ttitle\tin_regression_list\taccepted_status\taccepted_all_results\tcandidate_status\tcandidate_all_results')
for r in rows:
    fmt=lambda vals: ','.join(vals) if vals else 'MISSING'
    print('\t'.join([r,title(r),str(r in reg),fmt(acc_status.get(r,[])),fmt(acc_results.get(r,[])),fmt(cand_status.get(r,[])),fmt(cand_results.get(r,[]))]))
print('\ncounts')
from collections import Counter
for label,d in [('accepted_status',acc_status),('accepted_all_results',acc_results),('candidate_status',cand_status),('candidate_all_results',cand_results)]:
    c=Counter()
    for r in rows:
        vals=d.get(r)
        if vals: c.update(vals)
        else: c['MISSING']+=1
    print(label, dict(sorted(c.items())))
PY
```

Result:

```text
row	title	in_regression_list	accepted_status	accepted_all_results	candidate_status	candidate_all_results
php-src/sapi/cli/tests/002.phpt	running code with -r	True	PASSED	PASSED	MISSING	MISSING
php-src/sapi/cli/tests/021.phpt	CLI shell shebang	True	PASSED	PASSED	MISSING	MISSING
php-src/sapi/cli/tests/bug70006.phpt	Bug #70006 (cli - function with default arg = STDOUT crash output)	True	PASSED	PASSED	MISSING	MISSING
php-src/tests/classes/ctor_dtor.phpt	ZE2 The new constructor/destructor is called	True	PASSED	PASSED	FAILED	FAILED
php-src/tests/classes/factory_and_singleton_002.phpt	ZE2 factory and singleton, test 2	True	PASSED	PASSED	FAILED	FAILED
php-src/Zend/tests/assert/expect_011.phpt	test overloaded __toString on custom exception	True	PASSED	PASSED	FAILED	FAILED
php-src/Zend/tests/property_hooks/gh19548_002.phpt	GH-19548: Segfault when using inherited properties and opcache (multiple properties)	True	PASSED	PASSED	FAILED	FAILED
php-src/Zend/tests/type_declarations/variance/internal_parent/unresolvable_inheritance_check_param.phpt	Test unresolvable inheritance check due to unavailable parameter type when the parent has a tentative return type.	True	PASSED	PASSED	FAILED	FAILED

counts
accepted_status {'PASSED': 8}
accepted_all_results {'PASSED': 8}
candidate_status {'FAILED': 5, 'MISSING': 3}
candidate_all_results {'FAILED': 5, 'MISSING': 3}
```

Focused sample PASS/FAIL/SKIP/BORK/MISSING counts:

| Evidence source | PASS | FAIL | SKIP | BORK | MISSING |
| --- | ---: | ---: | ---: | ---: | ---: |
| accepted `current-status.normalized.tsv` | 8 | 0 | 0 | 0 | 0 |
| accepted `all-results.txt` | 8 | 0 | 0 | 0 | 0 |
| candidate `current-status.normalized.tsv` | 0 | 5 | 0 | 0 | 3 |
| candidate `all-results.txt` | 0 | 5 | 0 | 0 | 3 |

## Focused Replay Availability

The documented focused replay shape requires historical accepted and candidate
release `phpc` binaries through `PHPC_BIN`. The wrapper and pinned php-src
checkout are present, and both repository commits are present, but the required
historical binaries are absent from the cookbook paths.

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

The exact focused replay command shape from the cookbook was therefore not
executed. Running `run-tests.php` without those binaries would only measure a
broken replay setup, not accepted-vs-candidate compiler behavior. Rebuilding
historical release binaries was not done in this M0 read-only report lane.

The row-list paths prepared for replay would be:

```text
/home/claude/php-src-phpt/sapi/cli/tests/002.phpt
/home/claude/php-src-phpt/sapi/cli/tests/021.phpt
/home/claude/php-src-phpt/sapi/cli/tests/bug70006.phpt
/home/claude/php-src-phpt/tests/classes/ctor_dtor.phpt
/home/claude/php-src-phpt/tests/classes/factory_and_singleton_002.phpt
/home/claude/php-src-phpt/Zend/tests/assert/expect_011.phpt
/home/claude/php-src-phpt/Zend/tests/property_hooks/gh19548_002.phpt
/home/claude/php-src-phpt/Zend/tests/type_declarations/variance/internal_parent/unresolvable_inheritance_check_param.phpt
```

## Candidate Failure Evidence

The five explicit candidate `FAILED` rows have row-level diffs in candidate
shard logs:

- `php-src/tests/classes/ctor_dtor.phpt`: candidate emits
  `early::__destruct` and `late::__destruct` before the expected positions.
  Evidence:
  `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/shard-02/run-tests.log`
  around lines 5019-5035.
- `php-src/tests/classes/factory_and_singleton_002.phpt`: candidate fatals on
  `Call to protected test::__destruct() from global scope` during normal
  construction/destruction flow instead of producing the expected singleton
  output plus shutdown warning. Evidence:
  `shard-02/run-tests.log` around lines 5042-5079.
- `php-src/Zend/tests/assert/expect_011.phpt`: candidate reports
  `undefined property MyExpectations::$string` instead of the expected
  `AssertionError` message. Evidence:
  `shard-01/run-tests.log` around lines 17217-17227.
- `php-src/Zend/tests/property_hooks/gh19548_002.phpt`: candidate fatals that
  `C1` must implement `I1::$a::get` and `I1::$b::get`; expected output is
  `Multiple property test passed - no segmentation fault`. Evidence:
  `shard-02/run-tests.log` around lines 35924-35929.
- `php-src/Zend/tests/type_declarations/variance/internal_parent/unresolvable_inheritance_check_param.phpt`:
  candidate emits the direct declaration compatibility fatal instead of the
  expected unavailable-class compatibility-check fatal. Evidence:
  `shard-01/run-tests.log` around lines 25883-25888.

The three `sapi/cli` rows appear in the accepted pass baseline and in the
candidate regression list, but are absent from both candidate normalized status
and aggregate results. The candidate does record nearby
`php-src/sapi/cli/tests/002-unix.phpt` as `PASSED`, which reinforces that these
three selected CLI rows are row coverage/control-plane symptoms, not proven CLI
semantic failures.

## Classification

| Row | Classification | Reason |
| --- | --- | --- |
| `php-src/sapi/cli/tests/002.phpt` | control-plane absent; replay unavailable | Accepted PASS; candidate status/result are missing; no candidate row diff exists; focused replay cannot run without historical `PHPC_BIN`. |
| `php-src/sapi/cli/tests/021.phpt` | control-plane absent; replay unavailable | Accepted PASS; candidate status/result are missing; no candidate row diff exists; focused replay cannot run without historical `PHPC_BIN`. |
| `php-src/sapi/cli/tests/bug70006.phpt` | control-plane absent; replay unavailable | Accepted PASS; candidate status/result are missing; no candidate row diff exists; focused replay cannot run without historical `PHPC_BIN`. |
| `php-src/tests/classes/ctor_dtor.phpt` | semantic failure; replay unavailable | Accepted PASS; candidate explicit FAIL; shard diff shows destructor output emitted too early. |
| `php-src/tests/classes/factory_and_singleton_002.phpt` | semantic failure; replay unavailable | Accepted PASS; candidate explicit FAIL; shard diff shows protected destructor call fatal during execution. |
| `php-src/Zend/tests/assert/expect_011.phpt` | semantic failure; replay unavailable | Accepted PASS; candidate explicit FAIL; shard diff shows undefined property error instead of expected `AssertionError` formatting. |
| `php-src/Zend/tests/property_hooks/gh19548_002.phpt` | semantic failure; replay unavailable | Accepted PASS; candidate explicit FAIL; shard diff shows inherited properties do not satisfy hooked interface properties. |
| `php-src/Zend/tests/type_declarations/variance/internal_parent/unresolvable_inheritance_check_param.phpt` | semantic failure; replay unavailable | Accepted PASS; candidate explicit FAIL; shard diff shows wrong inheritance compatibility diagnostic path. |

## Conclusion

For this focused Zend/classes/sapi sample, the blocked 221205Z candidate has
five row-level semantic failures and three CLI control-plane absent rows. The
sample counts are accepted `PASS=8` and candidate `FAIL=5, MISSING=3`.

The deterministic next action is split:

- Treat the five explicit `FAILED` rows as repair candidates in destructor
  lifecycle, assertion/throwable formatting, property hook/interface
  compatibility, and internal inheritance diagnostic areas.
- Treat the three `sapi/cli` rows as harness/result-coverage work until a
  restored/rebuilt accepted and candidate `PHPC_BIN` focused replay proves a
  semantic CLI failure.
