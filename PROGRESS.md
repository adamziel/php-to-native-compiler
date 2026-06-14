# PTN Progress

Refresh: 2026-06-14T03:22Z.
Measured: `ptn-99q3` class-name fetch bounded run.

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
| Post-merge COW gate | 26 | 26 | 0 |
| PHPT callback manifest | 5 | 5 | 0 |
| PHPT include manifest | 2 | 2 | 0 |
| PHPT formatted string rows | 75 | 25 | 50 |
| PHPT focused array predicate/find/first/last rows | 6 | 6 | 0 |
| PHPT broad 1k baseline | 1000 | 265 | 735 |

## Remaining Exclusions

- Bounded: 486 selected, 456 runnable, 30 excluded, 449 pass, 7
  `array_filter` failures (`ptn-ueir`).
- Broad 1k classify-only: 443 runnable/557 excluded.
- Class declarations: 78 trait/interface/anonymous-class rows classified.
- Heredoc/nowdoc: 70 moved; 21 runnable, 49 metadata blockers.
- `ptn-ndkl` helper frontier is 29/39; 10 residual rows mapped.
- Zend assignment/reference 22/32; array diff/intersect 58/61.
- COW/reference: array-internal 17/72, foreach/reference 31/103,
  reference-call 9/12.

## Verification

`ptn-7xxw` guards huge `array_fill()`; `ptn-v1mu` maps 34 unpacking blockers
and confirms adjacent broad `array_chunk()` is 32/32.

`ptn-99q3` adds object/expression `::class`; `php_uname_error.phpt` passed in
the bounded run, with seven unrelated `array_filter` failures.

`ptn-zzr2` maps 78 class blockers; `ptn-ndkl` helper PHPT is 6/6; `ptn-4fd3`
keeps plain heredoc/nowdoc runnable at 14/70.
