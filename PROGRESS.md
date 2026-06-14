# PTN Progress

Refresh: 2026-06-14T04:13Z.
Measured: `ptn-h8f7` array object/metadata frontier after `ptn-odac`.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 718 | 718 | 0 |
| PHPT bounded manifest | 486 | 479 | 7 |
| PHPT Zend rows | 119 | 119 | 0 |
| PHPT ext/standard rows | 281 | 274 | 7 |
| PHPT focused array key/callback set rows | 75 | 38 | 37 |
| PHPT focused array callback validation rows | 65 | 46 | 19 |
| PHPT focused array diff/intersect rows | 61 | 58 | 3 |
| PHPT broad diff/intersect comparator rows | 76 | 64 | 12 |
| PHPT array fill/pad rows | 12 | 11 | 1 |
| PHPT array set/callback frontier | 106 | 86 | 20 |
| PHPT focused filesystem/path/process rows | 46 | 13 | 33 |
| PHPT tests/basic+func+lang | 78 | 78 | 0 |
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
| PHPT broad magic-method metadata frontier | 69 | 0 | 69 |
| PHPT broad standard-array frontier | 297 | 0 | 297 |
| PHPT broad request/SAPI input frontier | 41 | 1 | 40 |
| PHPT broad 1k attribute blocker bucket | 141 | 0 | 141 |
| PHPT broad heredoc/nowdoc array frontier | 70 | 14 | 56 |
| Post-merge COW gate | 26 | 26 | 0 |
| PHPT broad 1k baseline | 1000 | 265 | 735 |

## Remaining Exclusions

- Bounded: 486 selected, 456 runnable, 30 excluded; broad 1k 443/557.
- Array frontier: set/callback 86/106; request/SAPI raw 1 pass, 3 fail,
  37 skips; `ptn-h8f7` maps 70 object/metadata blockers.
- COW/reference: internal 17/72, foreach 31/103, reference-call 9/12.

## Verification

`ptn-yvgh`: huge-count preflight. `ptn-ri9o`: request/SAPI excludes 41.
`ptn-odac`: `array_chunk*` 32/32, leading-dot 5/5, diff/intersect 58/61.
`ptn-h8f7`: object/metadata manifest selects 70; classify-only excludes 70.
