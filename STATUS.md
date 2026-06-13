# PTN Status

Last refresh: 2026-06-13T13:18Z
Measured: `ptn-4qsw` integration on current `origin/master` `14d1d49`;

## Operating Goal

Hold the RC line to generic PHP semantics while expanding toward broad
php-src PHPT coverage. Report numbers, not compatibility claims.

## Current Signal

Source unit tests 3/3; Native/compiler Rust suite 625/625; Native smoke matrix 6/6; PHPT bounded manifest 279/279; PHPT Zend rows 87/87; PHPT ext/standard rows 137/137.

## Active Buckets

| Bucket | Count |
| --- | ---: |
| Source unit tests | 3/3 |
| Native/compiler Rust suite | 625/625 |
| Native smoke matrix | 6/6 |
| PHPT bounded manifest | 279/279 |
| PHPT Zend rows | 87/87 |
| PHPT ext/standard rows | 137/137 |
| PHPT focused stream rows | 2/2 |
| PHPT tests/basic+func+lang | 50/50 |
| PHPT other rows | 5/5 |
| PHPT COW manifest | 29/29 |
| Post-merge COW gate | 26/26 |
| PHPT callback manifest | 3/5 |
| PHPT include manifest | 2/2 |

## Rules

- Update `PROGRESS.md`, then run `tools/update-progress-mirrors.sh`.
- Keep mirrors compact and numeric.
- Never claim broad PHP compatibility from row-specific patches.
