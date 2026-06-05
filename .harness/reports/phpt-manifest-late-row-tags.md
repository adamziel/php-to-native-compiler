# PHPT Manifest And Late-Row Tags

Owner: developer-95
Lane: 5
Mode: read-only/control-plane report; no compiler or runtime source edits

## Scope

This artifact pins the PHPT gate/manifest command shape and verifies
late-priority `eval(` and variable-variable row counts. It does not claim new
PHP support, does not remove rows from the public denominator, and did not
start any `eval` or variable-variable implementation.

## Pinned Full-Gate Shape

The blocked candidate gate inspected here is:

`phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377`

Durable artifact directory:

`/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377`

The exact replay source is the saved gate script:

```sh
/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/run_gate.sh
```

Important pinned inputs from that script and `environment.txt`:

| Input | Value |
|---|---|
| php-src checkout seed | `/home/claude/php-src-phpt` |
| php-src pin | `f97ff597429a2fe633665a7e02d97c8077f9f90f` |
| source repo seed | `/home/claude/supervised-php-compiler/state/worktrees/batch024-current-supervisor-20260601T093225` |
| public/source head | `56fe9377fb46be00db5fdd30c966fdba406dc581` |
| built phpc binary | `/tmp/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/cargo-target/release/phpc` |
| wrapper | `/home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper` |
| baseline PASS set | `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67/current-passes.normalized.txt` |
| denominator source | `PINNED_RUNNABLE=20294` in `run_gate.sh`, persisted as `pinned_public_runnable=20294` in `environment.txt` and `public-comparable-score.tsv` |

The gate command shape is lowercase `-p`, with the wrapper selected as the PHP
binary and `PHPC_BIN` pointing at the built release compiler:

```sh
export PHPC_BIN=/tmp/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/cargo-target/release/phpc
export PHPC_PHPT_TIMEOUT_SECONDS=55
export PHPC_PHPT_KILL_AFTER_SECONDS=5
export PHPT_SYSTEM_PHP=php
export NO_INTERACTION=1
export TEST_PHP_SRCDIR=/tmp/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/php-src

cd /tmp/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/php-src
php <per-shard-harness>/run-tests.php \
  -q -n \
  -p /home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper \
  -r <shard-list> \
  -W <shard-results.txt> \
  -s <shard-run-tests.log> \
  --no-color \
  --set-timeout 65 \
  --temp-source /tmp/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/php-src \
  --temp-target <shard-temp>
```

`tests/security/open_basedir_*.phpt` rows are run through the same shape with a
serialized list:

```sh
php <serial-harness>/run-tests.php \
  -q -n \
  -p /home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper \
  -r /tmp/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/serialized-openbasedir.tests \
  -W /home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/serial-openbasedir/results.txt \
  -s /home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/serial-openbasedir/run-tests.log \
  --no-color \
  --set-timeout 65 \
  --temp-source /tmp/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/php-src \
  --temp-target /tmp/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/phpt-tmp/serial-openbasedir
```

The 221205Z durable score artifacts are:

| Artifact | Meaning |
|---|---|
| `all-results.txt` | aggregated raw `run-tests.php` status rows |
| `current-status.normalized.tsv` | normalized status/path rows derived from `all-results.txt` |
| `current-passes.normalized.txt` | current PASS set derived from normalized status |
| `baseline-passes.normalized.txt` | sorted copy of latest accepted public PASS baseline |
| `regressions-from-latest-published-passes.txt` | `comm -23 baseline current` |
| `counts.tsv` | raw aggregate status count summary |
| `public-comparable-score.tsv` | public score over fixed `20294` denominator |
| `current-score-gate-preflight.tsv` | pinned source/baseline/preflight resource inputs |
| `environment.txt` | source, wrapper, binary, denominator, and local tool versions |

For 221205Z, `public-comparable-score.tsv` records `7197 / 20294 = 35.46%`.
The harness database goal and auditor events keep the accepted public score at
`7873 / 20294` and treat 221205Z as blocked by `1166` latest-public PASS
regressions.

## Reproducible Manifest Command

Use the normalized status plus the pinned checkout to tag rows. Late rows stay
`denominator=included`; tags only change priority.

```sh
python3 - <<'PY' > /tmp/phpt-denominator-late-tags.tsv
from pathlib import Path
import re

php_src = Path("/home/claude/php-src-phpt")
status_tsv = Path("/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/current-status.normalized.tsv")
eval_re = re.compile(r"(?i)(^|[^A-Za-z0-9_$])eval\s*\(")
vv_re = re.compile(r"\$\$|\$\{\s*\$|(?i:variable[-_ ]variables?)")

print("path\tstatus\tdenominator\tpriority\tlate_reason")
for line in sorted(status_tsv.read_text().splitlines()):
    if not line.strip():
        continue
    status, path = line.split("\t", 1)
    if status in {"SKIPPED", "BORKED"}:
        continue
    local = path.removeprefix("php-src/")
    text = (php_src / local).read_text(errors="ignore")
    reasons = []
    if eval_re.search(text):
        reasons.append("eval")
    if vv_re.search(text):
        reasons.append("variable-variable")
    print(
        path,
        status,
        "included",
        "late" if reasons else "normal",
        ",".join(reasons) if reasons else "-",
        sep="\t",
    )
PY
```

This command is intentionally a manifest/tagging command. It is not a gate run
and does not execute PHPTs.

## Late-Row Scan Commands

Run these against the pinned checkout to reproduce the planning-compatible
counts:

```sh
rg -l '(?i)(^|[^A-Za-z0-9_$])eval\s*\(' \
  /home/claude/php-src-phpt \
  --glob '*.phpt' |
  sed 's#^/home/claude/php-src-phpt/##' |
  LC_ALL=C sort > /tmp/phpt-eval-pattern.rows

rg -l '\$\$|\$\{\s*\$|(?i:variable[-_ ]variables?)' \
  /home/claude/php-src-phpt \
  --glob '*.phpt' |
  sed 's#^/home/claude/php-src-phpt/##' |
  LC_ALL=C sort > /tmp/phpt-variable-variable-pattern.rows

cat /tmp/phpt-eval-pattern.rows /tmp/phpt-variable-variable-pattern.rows |
  LC_ALL=C sort -u > /tmp/phpt-late-priority-combined.rows

wc -l \
  /tmp/phpt-eval-pattern.rows \
  /tmp/phpt-variable-variable-pattern.rows \
  /tmp/phpt-late-priority-combined.rows
```

Verified counts from that scan:

| Set | Count |
|---|---:|
| eval-pattern rows | 142 |
| variable-variable-pattern rows | 86 |
| unique combined late-priority rows | 226 |
| overlap | 2 |

Overlap rows:

```text
Zend/tests/assert/expect_015.phpt
ext/simplexml/tests/000.phpt
```

The planning inputs `142`, `86`, and `226` are reproducible with the pattern
above; no correction is needed for that scan definition.

## Denominator Handling

Late-priority rows are tagged and deprioritized. They are not removed from the
fixed public denominator, and subtracting `226` from `20294` would produce a
different metric than the accepted project metric.

The durable 221205Z result files show the distinction between public
denominator accounting and a particular runner's result status:

- all 226 late rows exist in the accepted status file;
- 215 of those were non-SKIP/non-BORKED in the accepted status file;
- 11 were SKIP/BORKED in the accepted status file;
- in the blocked candidate status file, 211 were present, 201 were
  non-SKIP/non-BORKED, and 15 were absent from current status output.

Those status differences do not authorize removing late rows from the public
denominator. The fixed denominator remains `20294`; late tags only help lane
selection avoid eval and variable-variable work until the end.

## Ambiguous Pattern Edges

The variable-variable count is pattern-sensitive:

```sh
rg -l '\$\$|\$\{\s*\$' /home/claude/php-src-phpt --glob '*.phpt' | wc -l
```

That syntax-only scan returns `77`, not `86`. The planning-compatible `86`
includes rows that mention PHP's `${expr}` variable-variable deprecation or
have `variable variables` wording in test text. Representative rows included
by the broader pattern but not by the narrow syntax-only pattern:

```text
Zend/tests/exception_in_nested_rope.phpt
Zend/tests/grammar/bug61681.phpt
Zend/tests/heredoc_nowdoc/flexible-heredoc-complex-test1.phpt
Zend/tests/heredoc_nowdoc/flexible-heredoc-complex-test2.phpt
Zend/tests/heredoc_nowdoc/flexible-heredoc-complex-test3.phpt
Zend/tests/heredoc_nowdoc/flexible-heredoc-complex-test4.phpt
Zend/tests/heredoc_nowdoc/warning_during_heredoc_scan_ahead.phpt
Zend/tests/temporary_cleaning/temporary_cleaning_016.phpt
ext/opcache/tests/bug69159.phpt
```

Examples:

- `Zend/tests/grammar/bug61681.phpt` contains `${substr(...)}` and expected
  deprecation text for `${expr}` variable variables.
- `Zend/tests/heredoc_nowdoc/flexible-heredoc-complex-test*.phpt` contain
  `${<<<DOC...}` forms and expected `${expr}` variable-variable deprecations.
- `ext/opcache/tests/bug69159.phpt` has a test title about passing a variable
  variable and contains `${"x$i"}`.

The eval scan is also intentionally lexical. It counts `eval(` in PHPT text
and does not parse PHPT sections or PHP comments. Rows with `eval(` only in
expectations, skip sections, or explanatory text would be tagged late by this
control-plane scan even if the executable `--FILE--` body is not an eval
implementation target.

Unsupported matching edges:

- no PHP parser is used for these tags;
- no PHPT section filtering is applied;
- heredoc/nowdoc and string interpolation forms are lexical matches only;
- `variable variables` wording in PHPT titles/expectations is included by the
  planning-compatible scan;
- dynamic property or dynamic method names are not separately classified unless
  they also match the explicit variable-variable pattern.

## Implementation Status

No `eval` implementation was started.

No variable-variable implementation was started.

No compiler, runtime, parser, fixture, or support-document source files were
edited for this lane.
