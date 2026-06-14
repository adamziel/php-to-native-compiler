# PTN Status

Last refresh: 2026-06-14T16:48Z.
Measured: `ptn-c284`; `ptn-xymv`; `ptn-j8b8/b35n/18tp/gkvr`; `ptn-gt7b`; `ptn-30ji`; `ptn-h0qa` broad 1k classify-only 424 runnable / 576 classified; `ptn-dkcs` call-unpacking 34 selected / 0 runnable / 34 classified; `ptn-ei36` unpacking split 20 call / 14 array classified; `ptn-550s.12` broad 1k array-helper row pack 0/10 -> 10/10; `ptn-qsmv.16` array callback/map-filter row pack; `ptn-qsmv.17` assertion/runtime-config +10 broad rows and broad 1k classify-only 440/560; `ptn-s80e` broad 1k array/reference row pack 10/20 -> 20/20; `ptn-qsmv.13` broad 1k run-tests 366/440 runnable passed, 560 classified; COW 69/103 passed.

## Operating Goal

Hold the RC line to generic PHP semantics while expanding toward broad
php-src PHPT coverage. Report numbers, not compatibility claims.

## Current Signal

Units 3/3; Native 730/730; Bounded 479/486; Zend 119/119; ext/standard-PHPT 274/281; Array-key/cb 38/75.

## Active Buckets

| Bucket | Count |
| --- | ---: |
| Units | 3/3 |
| Native | 730/730 |
| Bounded | 479/486 |
| Zend | 119/119 |
| ext/standard-PHPT | 274/281 |
| Array-key/cb | 38/75 |
| Array-cb-valid | 64/66 |
| Array-diff | 59/61 |
| Diff-cmp | 67/76 |
| Array-setops | 67/119 |
| Fill/pad | 12/12 |
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
| Class/object-meta | 0/221 |
| Class/object-meta-granular | 0/135 |
| Class-metadata-split | 0/143 |
| Core/basic-op | 18/34 |
| Runtime-INI | 0/73 |
| Resource-limit | 0/1 |
| Magic/object | 20/69 |
| Magic-methods | 0/8 |
| Object-string-meta | 0/61 |
| Std-array-map | 0/297 |
| Std-arrays | 263/296 |
| Map/filter | 25/30 |
| Request/SAPI | 1/41 |
| Anon-class | 0/15 |
| Interface-decl | 0/23 |
| Interface-impl | 0/15 |
| Trait-decl | 0/25 |
| Call-unpack | 0/20 |
| Array-unpack | 0/14 |
| Type-hint | 0/14 |
| Function-state | 0/11 |
| Dynamic-symbol | 0/8 |
| Generator-runtime | 0/1 |
| Internal-call-bind | 0/1 |
| Attribute-syntax | 0/141 |
| Attribute-internal | 0/8 |
| Attribute-meta | 0/204 |
| Heredoc-array | 20/70 |
| Std-array-row-pack | 10/10 |
| Std-array-s80e | 20/20 |
| Std-array-tdei | 61/71 |
| array_rand | 6/7 |
| Zend-op/control | 15/26 |
| Binary-key | 1/1 |
| Runtime-config | 10/54 |
| COW-gate | 26/26 |
| 1k-baseline | 366/1000 |

## Rules

- Update `PROGRESS.md`, then run `tools/update-progress-mirrors.sh`.
- Keep mirrors compact and numeric.
- Never claim broad PHP compatibility from row-specific patches.
