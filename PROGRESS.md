# PTN Progress

Refresh: 2026-06-14T05:49Z.
Measured: `ptn-oe6i`.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native Rust | 721 | 721 | 0 |
| Bounded PHPT | 486 | 479 | 7 |
| Zend PHPT | 119 | 119 | 0 |
| ext/standard PHPT | 281 | 274 | 7 |
| Array key/callback set rows | 75 | 38 | 37 |
| Array callback validation rows | 66 | 49 | 17 |
| Array diff/intersect rows | 61 | 58 | 3 |
| Diff/intersect comparator rows | 76 | 64 | 12 |
| Array set-operation frontier | 119 | 64 | 55 |
| Array fill/pad rows | 12 | 11 | 1 |
| Array set/callback frontier | 106 | 86 | 20 |
| Array callback slice | 38 | 28 | 10 |
| Filesystem/path/process rows | 46 | 13 | 33 |
| tests/basic+func+lang | 78 | 78 | 0 |
| COW manifest | 54 | 54 | 0 |
| Nested foreach/reference rows | 3 | 2 | 1 |
| Array-internal COW frontier | 72 | 17 | 55 |
| COW foreach/reference frontier | 103 | 31 | 72 |
| Foreach list destructuring rows | 4 | 4 | 0 |
| Reference-call bucket | 12 | 10 | 2 |
| Zend assignment/reference frontier | 32 | 23 | 9 |
| Class declaration frontier | 78 | 1 | 77 |
| Zend bug regression frontier | 37 | 18 | 19 |
| Dynamic type blockers | 44 | 0 | 44 |
| Non-array class metadata | 74 | 0 | 74 |
| Resource-limit classifier row | 1 | 0 | 1 |
| Magic/object conversion frontier | 69 | 20 | 49 |
| Magic-method metadata frontier | 69 | 0 | 69 |
| Standard-array frontier | 297 | 0 | 297 |
| Standard-array execution | 296 | 243 | 53 |
| Array map/filter callback slice | 30 | 21 | 9 |
| Request/SAPI input frontier | 41 | 1 | 40 |
| Attribute blocker bucket | 141 | 0 | 141 |
| Heredoc/nowdoc array frontier | 70 | 14 | 56 |
| Standard-array tdei slice | 71 | 61 | 10 |
| COW gate | 26 | 26 | 0 |
| PHPT 1k baseline | 1000 | 265 | 735 |

## Remaining

- Broad 428/572; bounded 456/486.
- Arrays 64/119 setops, 61/71 tdei, 49/66 callbacks, 28/38 registry, 21/30 map/filter, 243/296 standard.
- COW/ref 10/12, 17/72, 31/103; classes 1/78; metadata 0/74; dynamic types 0/44.
