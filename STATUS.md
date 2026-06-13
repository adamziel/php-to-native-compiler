# PTN Status

Last refresh: 2026-06-13T18:49Z
Measured: `ptn-4tfb` broad 1k blocker map rebased after `ptn-qsmv.9`.

## Operating Goal

Hold the RC line to generic PHP semantics while expanding toward broad
php-src PHPT coverage. Report numbers, not compatibility claims.

## Current Signal

Source unit tests 3/3; Native/compiler Rust suite 651/651; Native smoke matrix 6/6; PHPT bounded manifest 485/485; PHPT Zend rows 119/119; PHPT ext/standard rows 280/280.

## Active Buckets

| Bucket | Count |
| --- | ---: |
| Source unit tests | 3/3 |
| Native/compiler Rust suite | 651/651 |
| Native smoke matrix | 6/6 |
| PHPT bounded manifest | 485/485 |
| PHPT Zend rows | 119/119 |
| PHPT ext/standard rows | 280/280 |
| PHPT focused array key/callback set rows | 38/75 |
| PHPT focused stream rows | 2/2 |
| PHPT focused cwd rows | 2/2 |
| PHPT focused filesystem/path/process rows | 13/46 |
| PHPT tests/basic+func+lang | 78/78 |
| PHPT other rows | 8/8 |
| PHPT COW manifest | 29/29 |
| Post-merge COW gate | 26/26 |
| PHPT callback manifest | 5/5 |
| PHPT include manifest | 2/2 |
| PHPT formatted string rows | 25/75 |

## Rules

- Update `PROGRESS.md`, then run `tools/update-progress-mirrors.sh`.
- Keep mirrors compact and numeric.
- Never claim broad PHP compatibility from row-specific patches.
