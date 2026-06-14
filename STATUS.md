# PTN Status

Last refresh: 2026-06-14T06:40Z.
Measured: `ptn-h47t`.

## Operating Goal

Hold the RC line to generic PHP semantics while expanding toward broad
php-src PHPT coverage. Report numbers, not compatibility claims.

## Current Signal

Source unit tests 3/3; Native Rust 724/724; Bounded PHPT 479/486; Zend PHPT 119/119; ext/standard PHPT 274/281; Array key/callback 38/75.

## Active Buckets

| Bucket | Count |
| --- | ---: |
| Source unit tests | 3/3 |
| Native Rust | 724/724 |
| Bounded PHPT | 479/486 |
| Zend PHPT | 119/119 |
| ext/standard PHPT | 274/281 |
| Array key/callback | 38/75 |
| Array callback validation | 49/66 |
| Array diff/intersect | 58/61 |
| Diff/intersect comparator | 64/76 |
| Array set-operation frontier | 64/119 |
| Array fill/pad | 11/12 |
| Array set/callback frontier | 86/106 |
| Array callback slice | 28/38 |
| Filesystem/path/process | 13/46 |
| tests/basic+func+lang | 78/78 |
| COW manifest | 54/54 |
| Nested foreach/reference | 2/3 |
| Array-internal COW | 17/72 |
| COW foreach/reference | 31/103 |
| Foreach list destructuring | 4/4 |
| Reference-call bucket | 10/12 |
| Zend assignment/reference | 23/32 |
| Recursive dump frontier | 2/4 |
| Class declaration frontier | 1/78 |
| Zend bug regression | 18/37 |
| Class-name scalar | 9/10 |
| Dynamic type blockers | 0/44 |
| Diagnostics/assertion frontier | 0/47 |
| Non-array class metadata | 0/74 |
| Resource-limit row | 0/1 |
| Magic/object conversion | 20/69 |
| Magic-method metadata | 0/69 |
| Standard-array frontier | 0/297 |
| Standard-array execution | 243/296 |
| Array map/filter callback | 21/30 |
| Request/SAPI input | 1/41 |
| Attribute blocker bucket | 0/141 |
| Attribute metadata frontier | 0/204 |
| Heredoc/nowdoc array frontier | 14/70 |
| Standard-array tdei slice | 61/71 |
| Broad array_rand slice | 6/7 |
| Zend op/control frontier | 15/26 |
| Binary key row | 1/1 |
| COW gate | 26/26 |
| PHPT 1k baseline | 265/1000 |

## Rules

- Update `PROGRESS.md`, then run `tools/update-progress-mirrors.sh`.
- Keep mirrors compact and numeric.
- Never claim broad PHP compatibility from row-specific patches.
