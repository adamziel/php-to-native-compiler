# PHP Native Compiler Progress

Updated: 2026-05-31 15:20 CEST

Primary/public branch: `origin/master`
Publication base before this update:
`54829387 docs: simplify PHPT progress report`
Published source head for current score:
`54f3c2c3 fix: preserve strictness provenance in magic typed properties`

This file is the public PHPT-suite progress report. AO workers and the
supervisor must update this file before claiming public progress.

Pinned PHPT metric:

`passed runnable PHPTs / total pinned runnable PHPTs`

The pinned php-src suite uses the stable public denominator `20294`. Raw
runner denominators that exclude BORKED rows are not public progress.

Current public score: **5173 / 20294 pinned runnable PHPTs = 25.49%**.

## Current Public Gate

Published gate: Batch023 repair01.

- Gate run:
  `phpt-full-batch023-repair01-sharded-serialized-openbasedir-20260531T1308Z-php-src-f97ff59-public-54829387-source-54f3c2c3`
- Source: `54f3c2c3 fix: preserve strictness provenance in magic typed properties`
- Score: **5173 / 20294 pinned runnable PHPTs = 25.49%**
- Regression result: zero latest-published PASS regressions against the
  Batch022 repair02 PASS baseline.
- Gate notes: the full `open_basedir_*` family was serialized; the known
  sockets expected-output marker was adjudicated as failed-row output, not a
  harness marker failure. The previously blocking
  `strnatcmp_leftalign.phpt` and `typed_properties_009.phpt` rows now pass.

No focused PHPT run, source checkpoint, status note, PR, or candidate gate
changes the public score until it is parsed, regression-checked against the
latest published PASS set, and recorded here.

## Blocked / Unpublished Candidates

- Batch023 checkpoint10 was superseded by Batch023 repair01. Its candidate
  score is no longer current public progress.

## Score History

| Gate | Passed / pinned runnable | Percent | Publication note |
| --- | ---: | ---: | --- |
| Batch001 baseline | 1118 / 20294 | 5.51% | Initial pinned full-suite baseline |
| Batch002 | 1193 / 20294 | 5.88% | 0 PASS regressions |
| Batch003 | 1311 / 20294 | 6.46% | 0 PASS regressions |
| Batch004 checkpoint8 repair | 1369 / 20294 | 6.75% | 0 PASS regressions |
| Batch004 checkpoint10 | 1413 / 20294 | 6.96% | 0 PASS regressions |
| Batch005 checkpoint10 | 1618 / 20294 | 7.97% | 0 semantic regressions |
| Batch006 checkpoint10 | 1836 / 20294 | 9.05% | 0 PASS regressions |
| Batch007 checkpoint10 repair | 2047 / 20294 | 10.09% | 0 PASS regressions |
| Batch008 checkpoint5 | 2180 / 20294 | 10.74% | 0 PASS regressions |
| Batch008 checkpoint10 repair | 2286 / 20294 | 11.26% | 0 PASS regressions |
| Batch009 burst1 | 2388 / 20294 | 11.77% | 0 PASS regressions |
| Batch010 checkpoint10 repair | 2563 / 20294 | 12.63% | 0 semantic regressions |
| Batch011 burst1 | 2741 / 20294 | 13.51% | 0 PASS regressions |
| Batch012 dynamic-call repair | 2945 / 20294 | 14.51% | 0 PASS regressions |
| Batch013 checkpoint10 | 3170 / 20294 | 15.62% | 0 PASS regressions |
| Batch014 regression repair | 3378 / 20294 | 16.65% | 0 PASS regressions |
| Batch015 checkpoint9 | 3646 / 20294 | 17.97% | 0 semantic regressions; `bug75679.phpt` long-root guard |
| Batch016 selected integration | 3868 / 20294 | 19.06% | 0 semantic regressions; 6 platform-SKIPIF rows adjudicated |
| Batch016 regression7 repair | 4048 / 20294 | 19.95% | 0 PASS regressions |
| Batch017 checkpoint10 | 4132 / 20294 | 20.36% | 0 PASS regressions; invalid-marker hits adjudicated as failed-row output |
| Batch018 repair01 | 4178 / 20294 | 20.59% | 0 PASS regressions; invalid-marker hits adjudicated |
| Batch019 repair02 | 4321 / 20294 | 21.29% | 0 semantic regressions; `bug75679.phpt` and `open_basedir_filemtime.phpt` adjudicated |
| Batch020 repair01 | 4425 / 20294 | 21.80% | 0 PASS regressions; sockets marker adjudicated |
| Batch021 regression repair | 4685 / 20294 | 23.09% | 0 PASS regressions; sockets marker adjudicated |
| Batch022 repair02 | 4949 / 20294 | 24.39% | 0 PASS regressions; sockets marker adjudicated |
| Batch023 repair01 | 5173 / 20294 | 25.49% | 0 PASS regressions; current public score |

## Operating Rules / Gates

- Public progress is only the pinned php-src PHPT full-suite pass rate.
- The total pinned runnable PHPT denominator stays `20294` until the pin or
  inventory policy is intentionally changed and documented here.
- A candidate can publish only after a full-suite gate is parsed, every
  latest-published PASS loss is reviewed, and semantic regressions are
  repaired.
- Focused PHPT proof must use lowercase `run-tests.php -p` with the
  `phpc-phpt-wrapper`; uppercase `-P` proof does not count for publication.
- Focused tests and source checkpoints are evidence for the next gate, not a
  public percentage change.
- Harness, platform, or expected-output adjudications must name the affected
  rows and evidence. Silent score substitution is not allowed.
- Blocked candidates may be listed here only as unpublished candidates, without
  replacing the current public score.

## Evidence Pointers

- php-src pin: `/home/claude/php-src-phpt` at
  `f97ff597429a2fe633665a7e02d97c8077f9f90f`
- PHPT wrapper:
  `/home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper`
- Current Batch023 gate evidence:
  `/home/claude/supervised-php-compiler/state/logs/phpt-full-batch023-repair01-sharded-serialized-openbasedir-20260531T1308Z-php-src-f97ff59-public-54829387-source-54f3c2c3`
- Previous Batch022 baseline evidence:
  `/home/claude/supervised-php-compiler/state/logs/phpt-full-batch022-repair02-sharded-serialized-openbasedir-20260531T0839Z-php-src-f97ff59-public-5530d1da-source-69c5111f`
- Previous Batch021 baseline evidence:
  `/home/claude/supervised-php-compiler/state/logs/phpt-full-batch021-regression-repair-sharded-serialized-openbasedir-20260531T0838Z-php-src-f97ff59-public-049ff7b5-source-7e9c4fd8`
- Previous Batch020 baseline evidence:
  `/home/claude/supervised-php-compiler/state/logs/phpt-full-batch020-repair01-sharded-serialized-openbasedir-20260531T0415Z-php-src-f97ff59-public-5e8f521a-source-4e7a7a41`
- Skip / xfail ledger:
  `/home/claude/supervised-php-compiler/state/php-core-suite-skip-ledger.tsv`
- Detailed chronological implementation proof remains in `docs/PROGRESS.md`.
