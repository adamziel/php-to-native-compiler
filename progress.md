# PTN Progress Mirror

Last refresh: 2026-06-14T22:54Z.
Source: `PROGRESS.md`
Measured: `ptn-c284`; `ptn-xymv`; `ptn-j8b8/b35n/18tp/gkvr`; `ptn-gt7b`; `ptn-30ji`; `ptn-h0qa` broad 1k classify-only 424 runnable / 576 classified; `ptn-dkcs` call-unpacking 34 selected / 0 runnable / 34 classified; `ptn-ei36` unpacking split 20 call / 14 array classified; `ptn-550s.12` broad 1k array-helper row pack 0/10 -> 10/10; `ptn-qsmv.14` class/interface row pack; `ptn-qsmv.16` array callback/map-filter row pack; `ptn-qsmv.17` assertion/runtime-config +10 broad rows and broad 1k classify-only 440/560; `ptn-s80e` broad 1k array/reference row pack 10/20 -> 20/20; `ptn-j6gv` broad 1k string/runtime row pack 15/25 -> 25/25; `ptn-55u0` broad 1k unpack row pack 2/34 raw baseline -> 10/10 runnable after split; `ptn-tiqh` COW/reference row pack 21/21 on submitted base; `ptn-ouhx` object-string array-helper row pack 0/34 -> 34/34, object-string source bucket 19/61 -> 53/61, broad 1k 285 -> 419 passing (501 runnable / 499 classified after, stitched from timed broad run plus remaining slice); `ptn-lxw1` array COW/reference row pack 9/9 focused, 2/2 candidates, 19/20 mixed control; `ptn-xcmz` broad 1k property/object metadata row pack 0/19 current-base focused baseline -> 12/12 runnable; `ptn-s8cn` call-unpacking row pack 0/20 classified -> 11/11 runnable passed and broad 1k classify-only 472/528 -> 545/455; `ptn-1d60` array_map null-reference row 65/66 -> 66/66 and broad 1k classify-only 558/442 with 370 current standard-array runnable rows and 0 standard-strings rows; `ptn-qg7b` asymmetric-visibility row pack 4/23 current-base focused baseline -> 14/23 final branch, with completed broad 1k run 463/533 before final master fast-forward; `ptn-g7ta` object-string array row pack +24 broad 1k rows and focused 23/23; `ptn-mqvk` broad 1k classify-only 546/454 before -> 532/468 after and broad COW/reference row pack 22/28 -> 27/27 runnable plus 1 classified; COW 69/103 passed; `ptn-601n` anonymous-class/object metadata row pack 0/21 raw focused baseline -> 10/10 runnable after split and final broad 1k classify-only 558/442; `ptn-c5ar` object/class metadata runtime diagnostics row pack 9 passed / 7 failed / 4 classified -> 20/20 passed and current-target broad 1k classify-only 532/468 -> 563/437; `ptn-zhup` object/callback/merge row pack 13/18 -> 17/18 runnable plus 2 classified, merge slice 21/25 -> 24/25, and final broad 1k classify-only 558/442 on submitted base; `ptn-buig` break/continue diagnostics moved the broad 1k Zend bucket 137/198 -> 141/198 and focused Zend operator/control frontier 21/26; `ptn-u4el` string runtime row pack 1/10 -> 10/10 and broad 1k 477/558 -> 479/558; `ptn-jlzj` control-flow/call-unpack broad 1k 453 -> 482 passing.

Compact signal: Units 3/3; Native 732/732; Bounded 479/486; Zend 119/119; ext/standard-PHPT 274/281; Array-key/cb 38/75.

| Format / source | Passing |
| --- | ---: |
| Units | 3/3 |
| Native | 732/732 |
| Bounded | 479/486 |
| Zend | 119/119 |
| ext/standard-PHPT | 274/281 |
| Array-key/cb | 38/75 |
| Array-cb-valid | 66/66 |
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
| Asym-vis | 14/23 |
| Dynamic-type | 0/44 |
| Diagnostics | 0/47 |
| Non-array-meta | 0/74 |
| Class/object-meta | 0/221 |
| Class/object-prop-pack | 12/19 |
| Class/object-c5ar | 20/20 |
| Class/object-meta-granular | 0/135 |
| Class-metadata-split | 0/143 |
| Core/basic-op | 18/34 |
| Runtime-INI | 0/73 |
| Resource-limit | 0/1 |
| Magic/object | 20/69 |
| Magic-methods | 0/8 |
| Object-string-meta | 53/61 |
| Object-string-array-pack | 34/34 |
| Object-callback-merge-zhup | 17/20 |
| Std-array-map | 0/297 |
| Std-arrays | 263/296 |
| Map/filter | 25/30 |
| Request/SAPI | 1/41 |
| Anon-class | 0/15 |
| Interface-decl | 0/23 |
| Interface-impl | 0/15 |
| Trait-decl | 0/25 |
| Call-unpack | 11/20 |
| Array-unpack | 10/14 |
| Type-hint | 0/14 |
| Function-state | 0/11 |
| Dynamic-symbol | 3/8 |
| Generator-runtime | 0/1 |
| Internal-call-bind | 1/1 |
| Attribute-syntax | 0/141 |
| Attribute-internal | 0/8 |
| Attribute-meta | 0/204 |
| Heredoc-array | 20/70 |
| Anon-class-pack | 10/21 |
| Std-array-row-pack | 10/10 |
| Std-array-s80e | 20/20 |
| Std-array-tdei | 61/71 |
| Std-array-lxw1 | 19/20 |
| array_rand | 6/7 |
| Zend-op/control | 21/26 |
| Binary-key | 1/1 |
| Runtime-config | 10/54 |
| COW-gate | 26/26 |
| COW-reference-tiqh | 21/21 |
| COW-reference-mqvk | 27/28 |
| 1k-baseline | 479/1000 |

Canonical dashboard: `PROGRESS.md`. Regenerate with
`tools/update-progress-mirrors.sh` after changing canonical progress.
