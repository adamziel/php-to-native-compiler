# PTN Status

Last refresh: 2026-06-14T04:58Z.
Measured: `ptn-tdei` standard array.

## Operating Goal

Hold the RC line to generic PHP semantics while expanding toward broad
php-src PHPT coverage. Report numbers, not compatibility claims.

## Current Signal

Source unit tests 3/3; Native/compiler Rust suite 721/721; PHPT bounded manifest 479/486; PHPT Zend rows 119/119; PHPT ext/standard rows 274/281; PHPT focused array key/callback set rows 38/75.

## Active Buckets

| Bucket | Count |
| --- | ---: |
| Source unit tests | 3/3 |
| Native/compiler Rust suite | 721/721 |
| PHPT bounded manifest | 479/486 |
| PHPT Zend rows | 119/119 |
| PHPT ext/standard rows | 274/281 |
| PHPT focused array key/callback set rows | 38/75 |
| PHPT focused array callback validation rows | 49/66 |
| PHPT focused array diff/intersect rows | 58/61 |
| PHPT broad diff/intersect comparator rows | 64/76 |
| PHPT broad array set-operation frontier | 64/119 |
| PHPT array fill/pad rows | 11/12 |
| PHPT array set/callback frontier | 86/106 |
| PHPT focused filesystem/path/process rows | 13/46 |
| PHPT tests/basic+func+lang | 78/78 |
| PHPT COW manifest | 54/54 |
| PHPT nested foreach/reference rows | 2/3 |
| PHPT array-internal COW frontier | 17/72 |
| PHPT COW foreach/reference frontier | 31/103 |
| PHPT foreach list destructuring rows | 4/4 |
| PHPT broad reference-call bucket | 10/12 |
| PHPT broad Zend assignment/reference frontier | 23/32 |
| PHPT broad class declaration frontier | 0/78 |
| PHPT broad Zend bug regression frontier | 18/37 |
| PHPT broad resource-limit classifier row | 0/1 |
| PHPT broad magic/object conversion frontier | 20/69 |
| PHPT broad magic-method metadata frontier | 0/69 |
| PHPT broad standard-array frontier | 0/297 |
| PHPT broad request/SAPI input frontier | 1/41 |
| PHPT broad 1k attribute blocker bucket | 0/141 |
| PHPT broad heredoc/nowdoc array frontier | 14/70 |
| PHPT broad standard-array tdei slice | 61/71 |
| Post-merge COW gate | 26/26 |
| PHPT broad 1k baseline | 265/1000 |

## Rules

- Update `PROGRESS.md`, then run `tools/update-progress-mirrors.sh`.
- Keep mirrors compact and numeric.
- Never claim broad PHP compatibility from row-specific patches.
