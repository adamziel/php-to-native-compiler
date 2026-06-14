# PTN Status

Last refresh: 2026-06-14T02:22Z.
Measured: `ptn-4fd3` heredoc classifier frontier.

## Operating Goal

Hold the RC line to generic PHP semantics while expanding toward broad
php-src PHPT coverage. Report numbers, not compatibility claims.

## Current Signal

Source unit tests 3/3; Native/compiler Rust suite 715/715; Native smoke matrix 6/6; PHPT bounded manifest 485/485; PHPT Zend rows 119/119; PHPT ext/standard rows 280/280.

## Active Buckets

| Bucket | Count |
| --- | ---: |
| Source unit tests | 3/3 |
| Native/compiler Rust suite | 715/715 |
| Native smoke matrix | 6/6 |
| PHPT bounded manifest | 485/485 |
| PHPT Zend rows | 119/119 |
| PHPT ext/standard rows | 280/280 |
| PHPT focused array key/callback set rows | 38/75 |
| PHPT focused array callback validation rows | 46/65 |
| PHPT focused array diff/intersect rows | 58/61 |
| PHPT focused filesystem/path/process rows | 13/46 |
| PHPT tests/basic+func+lang | 78/78 |
| PHPT other rows | 8/8 |
| PHPT COW manifest | 54/54 |
| PHPT nested foreach/reference rows | 2/3 |
| PHPT array-internal COW frontier | 17/72 |
| PHPT COW foreach/reference frontier | 31/103 |
| PHPT foreach list destructuring rows | 4/4 |
| PHPT broad reference-call bucket | 9/12 |
| PHPT broad Zend assignment/reference frontier | 22/32 |
| PHPT broad 1k attribute blocker bucket | 0/141 |
| PHPT broad heredoc/nowdoc array frontier | 14/70 |
| Post-merge COW gate | 26/26 |
| PHPT callback manifest | 5/5 |
| PHPT include manifest | 2/2 |
| PHPT formatted string rows | 25/75 |
| PHPT focused array predicate/find rows | 4/4 |
| PHPT broad 1k baseline | 265/1000 |

## Rules

- Update `PROGRESS.md`, then run `tools/update-progress-mirrors.sh`.
- Keep mirrors compact and numeric.
- Never claim broad PHP compatibility from row-specific patches.
