# PTN Status

Last refresh: 2026-06-13T16:43Z
Measured: `ptn-iwit` PHPT classify-only blocker maps on `origin/master`

## Operating Goal

Hold the RC line to generic PHP semantics while expanding toward broad
php-src PHPT coverage. Report numbers, not compatibility claims.

## Current Signal

Source unit tests 3/3; Native/compiler Rust suite 645/645; Native smoke matrix 6/6; PHPT bounded manifest 400/400; PHPT Zend rows 94/94; PHPT ext/standard rows 223/223.

## Active Buckets

| Bucket | Count |
| --- | ---: |
| Source unit tests | 3/3 |
| Native/compiler Rust suite | 645/645 |
| Native smoke matrix | 6/6 |
| PHPT bounded manifest | 400/400 |
| PHPT Zend rows | 94/94 |
| PHPT ext/standard rows | 223/223 |
| PHPT focused array key/callback set rows | 38/75 |
| PHPT focused stream rows | 2/2 |
| PHPT focused cwd rows | 2/2 |
| PHPT tests/basic+func+lang | 78/78 |
| PHPT other rows | 5/5 |
| PHPT COW manifest | 29/29 |
| Post-merge COW gate | 26/26 |
| PHPT callback manifest | 5/5 |
| PHPT include manifest | 2/2 |
| PHPT formatted string rows | 25/75 |

## Rules

- Update `PROGRESS.md`, then run `tools/update-progress-mirrors.sh`.
- Keep mirrors compact and numeric.
- Never claim broad PHP compatibility from row-specific patches.
