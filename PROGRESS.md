# PTN Progress

Refresh: 2026-06-14T03:30Z.
Measured: `ptn-knrm`.

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
| PHPT broad 1k attribute blocker bucket | 141 | 0 | 141 |
| PHPT broad heredoc/nowdoc array frontier | 70 | 14 | 56 |
| PHPT broad magic metadata blocker bucket | 69 | 0 | 69 |
| Post-merge COW gate | 26 | 26 | 0 |
| PHPT callback manifest | 5 | 5 | 0 |
| PHPT include manifest | 2 | 2 | 0 |
| PHPT formatted string rows | 75 | 25 | 50 |
| PHPT focused array predicate/find/first/last rows | 6 | 6 | 0 |
| PHPT broad 1k baseline | 1000 | 265 | 735 |

## Remaining Exclusions

- Bounded: 486 selected, 456 runnable, 30 excluded, 449 pass; 7
  `array_filter` failures (`ptn-ueir`).
- Broad 1k classify-only: 443/557.
- Class declarations: 78 classified.
- Heredoc/nowdoc: 21 runnable, 49 metadata blockers.
- Magic metadata: 69 unsupported-class-metadata rows.
- `ptn-ndkl`: 29/39; 10 residual rows.
- Zend assignment/reference 22/32; diff/intersect 58/61.
- COW/reference: internal 17/72, foreach/reference 31/103,
  reference-call 9/12.

## Verification

`ptn-7xxw`: huge `array_fill()` guard. `ptn-v1mu`: 34 unpacking blockers,
adjacent `array_chunk()` 32/32.

`ptn-knrm`: 69 magic metadata rows, all classified in
`tools/phpt-broad-magic-metadata-manifest.txt`.

`ptn-99q3`: object/expression `::class`; `php_uname_error.phpt` passed; seven
unrelated `array_filter` failures remain.

`ptn-zzr2`: 78 class blockers. `ptn-ndkl`: helper PHPT 6/6. `ptn-4fd3`:
plain heredoc/nowdoc 14/70.
