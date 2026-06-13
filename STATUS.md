# PTN Status

Last refresh: 2026-06-13T17:51Z
Measured: `ptn-7o62` attribute syntax blocker documentation/evidence rebased

## Operating Goal

Hold the RC line to generic PHP semantics while expanding toward broad
php-src PHPT coverage. Report numbers, not compatibility claims.

## Current Signal

Source unit tests 3/3; Native/compiler Rust suite 645/645; Native smoke matrix 6/6; PHPT bounded manifest 435/435; PHPT Zend rows 119/119; PHPT ext/standard rows 230/230.

## Active Buckets

| Bucket | Count |
| --- | ---: |
| Source unit tests | 3/3 |
| Native/compiler Rust suite | 645/645 |
| Native smoke matrix | 6/6 |
| PHPT bounded manifest | 435/435 |
| PHPT Zend rows | 119/119 |
| PHPT ext/standard rows | 230/230 |
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
