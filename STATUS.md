# PTN Status

Last refresh: 2026-06-14T07:58Z.
Measured: `ptn-c284`; COW-foreach 103 selected, 69 passed.

## Operating Goal

Hold the RC line to generic PHP semantics while expanding toward broad
php-src PHPT coverage. Report numbers, not compatibility claims.

## Current Signal

Units 3/3; Native 729/729; Bounded 479/486; Zend 119/119; ext/standard-PHPT 274/281; Array-key/cb 38/75.

## Active Buckets

| Bucket | Count |
| --- | ---: |
| Units | 3/3 |
| Native | 729/729 |
| Bounded | 479/486 |
| Zend | 119/119 |
| ext/standard-PHPT | 274/281 |
| Array-key/cb | 38/75 |
| Array-cb-valid | 49/66 |
| Array-diff | 58/61 |
| Diff-cmp | 64/76 |
| Array-setops | 64/119 |
| Fill/pad | 11/12 |
| Array-set/cb | 86/106 |
| Array-cb-slice | 28/38 |
| FS/process | 13/46 |
| First-class-callable | 10/12 |
| basic+func+lang | 78/78 |
| COW-manifest | 54/54 |
| Nested-foreach | 2/3 |
| Array-COW | 17/72 |
| COW-foreach | 69/103 |
| Foreach-list | 4/4 |
| Ref-call | 10/12 |
| CUF-edges | 8/12 |
| Zend-assign | 23/32 |
| Recursive-dump | 2/4 |
| Classes | 1/78 |
| Zend-bugs | 18/37 |
| Class-name | 9/10 |
| Dynamic-type | 0/44 |
| Diagnostics | 0/47 |
| Non-array-meta | 0/74 |
| Core/basic-op | 18/34 |
| Runtime-INI | 0/73 |
| Resource-limit | 0/1 |
| Magic/object | 20/69 |
| Magic-methods | 0/69 |
| Std-array-map | 0/297 |
| Std-arrays | 243/296 |
| Map/filter | 21/30 |
| Request/SAPI | 1/41 |
| Unsupported-lang | 0/288 |
| Attribute-blockers | 0/141 |
| Attribute-meta | 0/204 |
| Heredoc-array | 14/70 |
| Std-array-tdei | 61/71 |
| array_rand | 6/7 |
| Zend-op/control | 15/26 |
| Binary-key | 1/1 |
| Runtime-config | 0/54 |
| COW-gate | 26/26 |
| 1k-baseline | 265/1000 |

## Rules

- Update `PROGRESS.md`, then run `tools/update-progress-mirrors.sh`.
- Keep mirrors compact and numeric.
- Never claim broad PHP compatibility from row-specific patches.
