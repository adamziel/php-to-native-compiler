# PTN Progress

Refresh: 2026-06-14T06:20Z.
Measured: `ptn-iyhh`.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native Rust | 721 | 721 | 0 |
| Bounded PHPT | 486 | 479 | 7 |
| Zend PHPT | 119 | 119 | 0 |
| ext/standard PHPT | 281 | 274 | 7 |
| Array key/callback | 75 | 38 | 37 |
| Array callback validation | 66 | 49 | 17 |
| Array diff/intersect | 61 | 58 | 3 |
| Diff/intersect comparator | 76 | 64 | 12 |
| Array set-operation frontier | 119 | 64 | 55 |
| Array fill/pad | 12 | 11 | 1 |
| Array set/callback frontier | 106 | 86 | 20 |
| Array callback slice | 38 | 28 | 10 |
| Filesystem/path/process | 46 | 13 | 33 |
| tests/basic+func+lang | 78 | 78 | 0 |
| COW manifest | 54 | 54 | 0 |
| Nested foreach/reference | 3 | 2 | 1 |
| Array-internal COW | 72 | 17 | 55 |
| COW foreach/reference | 103 | 31 | 72 |
| Foreach list destructuring | 4 | 4 | 0 |
| Reference-call bucket | 12 | 10 | 2 |
| Zend assignment/reference | 32 | 23 | 9 |
| Recursive dump frontier | 4 | 2 | 2 |
| Class declaration frontier | 78 | 1 | 77 |
| Zend bug regression | 37 | 18 | 19 |
| Dynamic type blockers | 44 | 0 | 44 |
| Non-array class metadata | 74 | 0 | 74 |
| Resource-limit row | 1 | 0 | 1 |
| Magic/object conversion | 69 | 20 | 49 |
| Magic-method metadata | 69 | 0 | 69 |
| Standard-array frontier | 297 | 0 | 297 |
| Standard-array execution | 296 | 243 | 53 |
| Array map/filter callback | 30 | 21 | 9 |
| Request/SAPI input | 41 | 1 | 40 |
| Attribute blocker bucket | 141 | 0 | 141 |
| Attribute metadata frontier | 204 | 0 | 204 |
| Heredoc/nowdoc array | 70 | 14 | 56 |
| Standard-array tdei | 71 | 61 | 10 |
| Zend op/control | 26 | 15 | 11 |
| Binary key row | 1 | 1 | 0 |
| COW gate | 26 | 26 | 0 |
| PHPT 1k baseline | 1000 | 265 | 735 |

## Remaining

- Broad 428/572; bounded 456/486; arrays: setops 64/119, tdei 61/71, standard 243/296.
- COW/ref 10/12,17/72,31/103; metadata 0/74; attributes 0/204; recursive dump 2/4.
