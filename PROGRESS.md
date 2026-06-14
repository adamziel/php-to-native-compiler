# PTN Progress

Refresh: 2026-06-14T05:42Z.
Measured: `ptn-ingc`.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native Rust | 721 | 721 | 0 |
| PHPT bounded manifest | 486 | 479 | 7 |
| PHPT Zend rows | 119 | 119 | 0 |
| PHPT ext/standard rows | 281 | 274 | 7 |
| PHPT focused array key/callback set rows | 75 | 38 | 37 |
| PHPT focused array callback validation rows | 66 | 49 | 17 |
| PHPT focused array diff/intersect rows | 61 | 58 | 3 |
| PHPT broad diff/intersect comparator rows | 76 | 64 | 12 |
| PHPT broad array set-operation frontier | 119 | 64 | 55 |
| PHPT array fill/pad rows | 12 | 11 | 1 |
| PHPT array set/callback frontier | 106 | 86 | 20 |
| PHPT broad array callback slice | 38 | 28 | 10 |
| PHPT filesystem/path/process rows | 46 | 13 | 33 |
| PHPT tests/basic+func+lang | 78 | 78 | 0 |
| PHPT COW manifest | 54 | 54 | 0 |
| PHPT nested foreach/reference rows | 3 | 2 | 1 |
| PHPT array-internal COW frontier | 72 | 17 | 55 |
| PHPT COW foreach/reference frontier | 103 | 31 | 72 |
| PHPT foreach list destructuring rows | 4 | 4 | 0 |
| PHPT broad reference-call bucket | 12 | 10 | 2 |
| PHPT broad Zend assignment/reference frontier | 32 | 23 | 9 |
| PHPT broad class declaration frontier | 78 | 1 | 77 |
| PHPT broad Zend bug regression frontier | 37 | 18 | 19 |
| Function/dynamic type blockers | 44 | 0 | 44 |
| Resource-limit classifier row | 1 | 0 | 1 |
| Magic/object conversion frontier | 69 | 20 | 49 |
| Magic-method metadata frontier | 69 | 0 | 69 |
| PHPT broad standard-array frontier | 297 | 0 | 297 |
| Standard-array execution | 296 | 243 | 53 |
| Array map/filter callback slice | 30 | 21 | 9 |
| Request/SAPI input frontier | 41 | 1 | 40 |
| PHPT attribute blocker bucket | 141 | 0 | 141 |
| PHPT broad heredoc/nowdoc array frontier | 70 | 14 | 56 |
| PHPT broad standard-array tdei slice | 71 | 61 | 10 |
| COW gate | 26 | 26 | 0 |
| PHPT 1k baseline | 1000 | 265 | 735 |

## Remaining

- Broad 428/572; bounded 456/486.
- Arrays 64/119 setops, 61/71 tdei, 49/66 callbacks, 28/38 registry, 21/30 map/filter, 243/296 standard.
- COW/ref 10/12, 17/72, 31/103; classes 1/78.
