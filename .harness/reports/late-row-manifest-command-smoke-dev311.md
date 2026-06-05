# Late-Row Manifest Command Smoke

Lane: 128, developer-311

Scope: read-only reproducibility smoke for the documented eval and
variable-variable late-row manifest commands. No compiler/runtime source edits
were made, no PHPTs were executed, and no public score movement is claimed.

## Inputs

| Input | Result |
| --- | ---: |
| `/home/claude/php-src-phpt` `.phpt` files | 21827 |
| `current-status.normalized.tsv` rows | 18940 |

Status artifact used by the manifest command:

```text
/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/current-status.normalized.tsv
```

Raw status distribution in that file:

| Status | Rows |
| --- | ---: |
| `BORKED` | 669 |
| `FAILED` | 8845 |
| `PASSED` | 7196 |
| `SKIPPED` | 2220 |
| `WARNED` | 2 |
| `XFAILED` | 8 |

The broader score artifacts still record `7197 / 20294` in
`public-comparable-score.tsv`; this lane only validates the documented manifest
command that reads `current-status.normalized.tsv`.

## Late-Row List Smoke

I used the documented regexes and pipeline shape, writing to an isolated
temporary directory to avoid cross-worker `/tmp` collisions:

```sh
rg -l '(?i)(^|[^A-Za-z0-9_$])eval\s*\(' /home/claude/php-src-phpt --glob '*.phpt' |
  sed 's#^/home/claude/php-src-phpt/##' |
  LC_ALL=C sort > /tmp/dev311-late-row-smoke/phpt-eval-pattern.rows

rg -l '\$\$|\$\{\s*\$|(?i:variable[-_ ]variables?)' /home/claude/php-src-phpt --glob '*.phpt' |
  sed 's#^/home/claude/php-src-phpt/##' |
  LC_ALL=C sort > /tmp/dev311-late-row-smoke/phpt-variable-variable-pattern.rows

cat /tmp/dev311-late-row-smoke/phpt-eval-pattern.rows \
  /tmp/dev311-late-row-smoke/phpt-variable-variable-pattern.rows |
  LC_ALL=C sort -u > /tmp/dev311-late-row-smoke/phpt-late-priority-combined.rows

comm -12 /tmp/dev311-late-row-smoke/phpt-eval-pattern.rows \
  /tmp/dev311-late-row-smoke/phpt-variable-variable-pattern.rows \
  > /tmp/dev311-late-row-smoke/phpt-late-priority-overlap.rows
```

Counts match the documented planning-compatible figures:

| Set | Rows |
| --- | ---: |
| eval-pattern rows | 142 |
| variable-variable-pattern rows | 86 |
| unique combined late-priority rows | 226 |
| overlap rows | 2 |

Overlap rows:

```text
Zend/tests/assert/expect_015.phpt
ext/simplexml/tests/000.phpt
```

## Manifest Command Smoke

I ran the documented Python manifest/tagging command with the same status file,
pinned PHPT checkout, regexes, output columns, and `SKIPPED`/`BORKED` filter,
writing the output to:

```text
/tmp/dev311-late-row-smoke/phpt-denominator-late-tags.tsv
```

Shape and aggregate checks:

| Check | Result |
| --- | ---: |
| output lines including header | 16052 |
| data rows | 16051 |
| malformed data rows | 0 |
| `denominator=included` rows | 16051 |
| `priority=normal` rows | 15850 |
| `priority=late` rows | 201 |

Late reason distribution in the manifest output:

| `late_reason` | Rows |
| --- | ---: |
| `eval` | 124 |
| `variable-variable` | 76 |
| `eval,variable-variable` | 1 |

The manifest output has fewer late rows than the full checkout scan because it
only tags rows present in `current-status.normalized.tsv` and skips
`SKIPPED`/`BORKED` before emission. The full `226` combined late rows split
against that status artifact as:

| Status bucket | Late rows |
| --- | ---: |
| `FAILED` | 150 |
| `PASSED` | 51 |
| `SKIPPED` | 8 |
| `BORKED` | 2 |
| absent from `current-status.normalized.tsv` | 15 |

## Decision

The documented late-row list commands are reproducible at command-shape and
count level: `142` eval rows, `86` variable-variable rows, `226` combined rows,
and `2` overlap rows. The manifest/tagging command also runs cleanly and emits
well-formed rows with denominator inclusion preserved. This does not implement
eval or variable variables, does not execute PHPTs, and does not change the
accepted public score.
