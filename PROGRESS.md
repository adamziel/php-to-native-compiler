# PTN Progress

Refresh: 2026-06-14T03:42Z.
Measured: `ptn-51ey` standard-array frontier map after `ptn-3a8d`.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 718 | 718 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 486 | 479 | 7 |
| PHPT Zend rows | 119 | 119 | 0 |
| PHPT ext/standard rows | 281 | 274 | 7 |
| PHPT focused array key/callback set rows | 75 | 38 | 37 |
| PHPT focused array callback validation rows | 65 | 46 | 19 |
| PHPT focused array diff/intersect rows | 61 | 58 | 3 |
| PHPT focused filesystem/path/process rows | 46 | 13 | 33 |
| PHPT tests/basic+func+lang | 78 | 78 | 0 |
| PHPT other rows | 8 | 8 | 0 |
| PHPT COW manifest | 54 | 54 | 0 |
| PHPT nested foreach/reference rows | 3 | 2 | 1 |
| PHPT array-internal COW frontier | 72 | 17 | 55 |
| PHPT COW foreach/reference frontier | 103 | 31 | 72 |
| PHPT foreach list destructuring rows | 4 | 4 | 0 |
| PHPT broad reference-call bucket | 12 | 9 | 3 |
| PHPT broad Zend assignment/reference frontier | 32 | 22 | 10 |
| PHPT broad class declaration frontier | 78 | 0 | 78 |
| PHPT broad resource-limit classifier row | 1 | 0 | 1 |
| PHPT broad magic/object conversion frontier | 69 | 20 | 49 |
| PHPT broad standard-array frontier | 297 | 0 | 297 |
| PHPT broad 1k attribute blocker bucket | 141 | 0 | 141 |
| PHPT broad heredoc/nowdoc array frontier | 70 | 14 | 56 |
| Post-merge COW gate | 26 | 26 | 0 |
| PHPT formatted string rows | 75 | 25 | 50 |
| PHPT focused array predicate/find/first/last rows | 6 | 6 | 0 |
| PHPT broad 1k baseline | 1000 | 265 | 735 |

## Remaining Exclusions

- Bounded: 486 selected, 456 runnable, 30 excluded; 449 pass, 7 fail.
- Broad 1k classify-only: 443/557; class declarations 78; attributes 141.
- Standard-array frontier: 297 runnable rows; largest families are set/diff/
  intersect 76, covered `array_chunk()` 32, key helpers 21.
- Heredoc/nowdoc: 21 runnable, 49 metadata blockers; magic metadata 69.
- Magic/object raw frontier: 69 rows, 20 pass, 49 fail without classification.
- COW/reference: internal 17/72, foreach 31/103, reference-call 9/12.

## Verification

`ptn-51ey`: broad 1k selected 1000 rows, kept 430 runnable, and excluded 570;
focused standard-array classify-only selected 297 rows, all runnable.

`ptn-3a8d`: 69-row magic/object conversion frontier; classified run keeps all
excluded. `ptn-knrm`: 69 magic metadata rows stay classified.
