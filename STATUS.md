# PTN Status

Last refresh: 2026-06-14T06:56Z.
Measured: `ptn-12on`.

## Operating Goal

Hold the RC line to generic PHP semantics while expanding toward broad
php-src PHPT coverage. Report numbers, not compatibility claims.

## Current Signal

Source units 3/3; Native Rust 728/728; Bounded PHPT 479/486; Zend PHPT 119/119; ext/standard PHPT 274/281; Array key/cb 38/75.

## Active Buckets

| Bucket | Count |
| --- | ---: |
| Source units | 3/3 |
| Native Rust | 728/728 |
| Bounded PHPT | 479/486 |
| Zend PHPT | 119/119 |
| ext/standard PHPT | 274/281 |
| Array key/cb | 38/75 |
| Array cb validation | 49/66 |
| Array diff | 58/61 |
| Diff comparator | 64/76 |
| Array setops | 64/119 |
| Array fill/pad | 11/12 |
| Array set/cb | 86/106 |
| Array cb slice | 28/38 |
| Filesystem/process | 13/46 |
| basic+func+lang | 78/78 |
| COW manifest | 54/54 |
| Nested foreach | 2/3 |
| Array-internal COW | 17/72 |
| COW foreach | 31/103 |
| Foreach list | 4/4 |
| Reference-call | 10/12 |
| call_user_func edges | 8/12 |
| Zend assignment | 23/32 |
| Recursive dump | 2/4 |
| Class declarations | 1/78 |
| Zend bug rows | 18/37 |
| Class-name scalar | 9/10 |
| Dynamic type blockers | 0/44 |
| Diagnostics/assertion | 0/47 |
| Non-array metadata | 0/74 |
| Resource-limit row | 0/1 |
| Magic/object | 20/69 |
| Magic methods | 0/69 |
| Standard-array map | 0/297 |
| Standard arrays | 243/296 |
| Array map/filter | 21/30 |
| Request/SAPI | 1/41 |
| Unsupported language | 0/288 |
| Attribute blockers | 0/141 |
| Attribute metadata | 0/204 |
| Heredoc/nowdoc array | 14/70 |
| Standard-array tdei | 61/71 |
| array_rand slice | 6/7 |
| Zend op/control | 15/26 |
| Binary key row | 1/1 |
| COW gate | 26/26 |
| PHPT 1k baseline | 265/1000 |

## Rules

- Update `PROGRESS.md`, then run `tools/update-progress-mirrors.sh`.
- Keep mirrors compact and numeric.
- Never claim broad PHP compatibility from row-specific patches.
