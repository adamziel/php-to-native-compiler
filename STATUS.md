# PTN Status

Last refresh: 2026-06-15T12:45Z.
Measured: `ptn-c284`; `ptn-xymv`; `ptn-j8b8/b35n/18tp/gkvr`; `ptn-gt7b`; `ptn-30ji`; `ptn-h0qa` broad 1k classify-only 424 runnable / 576 classified; `ptn-dkcs` call-unpacking 34 selected / 0 runnable / 34 classified; `ptn-ei36` unpacking split 20 call / 14 array classified; `ptn-550s.12` broad 1k array-helper row pack 0/10 -> 10/10; `ptn-qsmv.14` class/interface row pack; `ptn-qsmv.16` array callback/map-filter row pack; `ptn-qsmv.17` assertion/runtime-config +10 broad rows and broad 1k classify-only 440/560; `ptn-s80e` broad 1k array/reference row pack 10/20 -> 20/20; `ptn-j6gv` broad 1k string/runtime row pack 15/25 -> 25/25; `ptn-55u0` broad 1k unpack row pack 2/34 raw baseline -> 10/10 runnable after split; `ptn-tiqh` COW/reference row pack 21/21 on submitted base; `ptn-ouhx` object-string array-helper row pack 0/34 -> 34/34, object-string source bucket 19/61 -> 53/61, broad 1k 285 -> 419 passing (501 runnable / 499 classified after, stitched from timed broad run plus remaining slice); `ptn-lxw1` array COW/reference row pack 9/9 focused, 2/2 candidates, 19/20 mixed control; `ptn-xcmz` broad 1k property/object metadata row pack 0/19 current-base focused baseline -> 12/12 runnable; `ptn-s8cn` call-unpacking row pack 0/20 classified -> 11/11 runnable passed and broad 1k classify-only 472/528 -> 545/455; `ptn-1d60` array_map null-reference row 65/66 -> 66/66 and broad 1k classify-only 558/442 with 370 current standard-array runnable rows and 0 standard-strings rows; `ptn-qg7b` asymmetric-visibility row pack 4/23 current-base focused baseline -> 14/23 final branch, with completed broad 1k run 463/533 before final master fast-forward; `ptn-g7ta` object-string array row pack +24 broad 1k rows and focused 23/23; `ptn-mqvk` broad 1k classify-only 546/454 before -> 532/468 after and broad COW/reference row pack 22/28 -> 27/27 runnable plus 1 classified; COW 69/103 passed; `ptn-601n` anonymous-class/object metadata row pack 0/21 raw focused baseline -> 10/10 runnable after split and final broad 1k classify-only 558/442; `ptn-c5ar` object/class metadata runtime diagnostics row pack 9 passed / 7 failed / 4 classified -> 20/20 passed and current-target broad 1k classify-only 532/468 -> 563/437; `ptn-zhup` object/callback/merge row pack 13/18 -> 17/18 runnable plus 2 classified, merge slice 21/25 -> 24/25, and final broad 1k classify-only 558/442 on submitted base; `ptn-buig` break/continue diagnostics moved the broad 1k Zend bucket 137/198 -> 141/198 and focused Zend operator/control frontier 21/26; `ptn-u4el` string runtime row pack 1/10 -> 10/10 and broad 1k 477/558 -> 479/558; `ptn-jlzj` control-flow/call-unpack broad 1k 453 -> 482 passing; `ptn-wxno` object-string array-helper classifier unblock moved broad 1k classify-only 563/437 -> 588/412 and focused 25/25 passed; `ptn-iuhj` parser/method-visibility row pack 43 selected / 41 runnable / 22 passed / 2 classified and final broad 1k classify-only 566/434; `ptn-29k0` method metadata scope row pack added 4 newly passing broad rows over `ptn-wxno`, with the integrated hook-start broad classify-only moving 563/437 -> 591/409; `ptn-3ijs` literal/operator row pack 27/30 focused and broad Zend bucket 145/201 -> 151/201; `ptn-ck7w` static-property row pack selected 24 / 2 runnable / 2 passed / 22 classified, with broad 1k classify-only 591/409 -> 592/408; `ptn-vq7w` broad COW/reference row pack 24/24 and user-comparator sort pack 20/26; `ptn-d0lg` broad 1k COW/reference row pack 536/592 -> 546/592 runnable passed with standard stable at 371/371; `ptn-25s0` method-scope warning lock measured hook-start full broad 1k at 390/591 -> 527/591 passed, with 138 newly passing rows and one broad-run pass-to-fail splice row that reran green on the rebased branch; `ptn-ppri` standard string/array row pack 20 selected / 18 runnable / 18 passed / 2 classified on the final rebased branch, with hook-start broad 1k 414/591 -> 528/591 passed (+114, no pass-set regressions); `ptn-m8pk` ext-standard strings row pack 0/21 -> 21/21 focused, with deterministic tier-1000 containing 0 ext/standard string rows; `ptn-dgj9` reference-lvalue array path row pack broad 414/591 -> 418/591 passed (+4, no pass-set regressions), with final focused 22 selected / 21 runnable / 8 passed / 1 classified; `ptn-cfny` parser/control-flow row pack 34 selected / 34 runnable / 31 passed, with 10 confirmed newly passing broad Zend rows; `ptn-hgfn` object/class metadata trace row pack 20 selected / 11 runnable / 11 passed / 9 classified, with completed full broad 1k 499/563 -> 535/591 passed before later master fast-forwards and no pass-set regressions; `ptn-texo` magic object/class metadata row pack 0/6 raw -> 4/6 final focused, with 2 typed-property dump failures; `ptn-psbp` parser/control row pack 28/28 focused and broad 1k 563/609 -> 573/609 passed with 10 newly passing rows; `ptn-mucw` full-corpus match expression row pack 0/30 -> 26/30 runnable; `ptn-7wxg` simple trait row pack 1/50 raw -> 50/50 focused and full-corpus trait source bucket 21/349 -> 212/349 runnable; `ptn-w17z.4` parser/control row pack 56 selected / 39 runnable / 36 passed / 17 classified; `ptn-w17z.11` namespace/include row pack 26 selected / 26 runnable / 26 passed; `ptn-w17z.3` classic array sort flags row pack moved 17/76 -> 63/76 passing (+46, no regressions) from the full sort-family source bucket; `ptn-w17z.21` date/time/formatting row pack 0/40 -> 29/40 date scalar and 29/52 -> 41/52 formatting (+41 newly passing rows); `ptn-w17z.19` full-corpus 20k ENV/CLEAN slice 0/474 -> 201/474 runnable and focused cleanup 4/4; `ptn-w17z.7` ReflectionClass row pack 36/36; `ptn-w17z.20` full-corpus stream/path/include row pack 0/35 -> 35/35 runnable.

## Operating Goal

Hold the RC line to generic PHP semantics while expanding toward broad
php-src PHPT coverage. Report numbers, not compatibility claims.

## Current Signal

Units 3/3; Native 733/733; Bounded 479/486; Zend 119/119; ext/standard-PHPT 274/281; Array-key/cb 38/75.

## Active Buckets

| Bucket | Count |
| --- | ---: |
| Units | 3/3 |
| Native | 733/733 |
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
| Asym-vis | 22/23 |
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
| Magic-object-texo | 4/6 |
| Magic-methods | 0/8 |
| Method-visibility-iuhj | 6/9 |
| Object-string-meta | 53/61 |
| Object-string-array-pack | 34/34 |
| Object-string-wxno | 25/25 |
| Object-callback-merge-zhup | 17/20 |
| Object-method-29k0 | 32/35 |
| Object-method-hgfn | 11/20 |
| Static-property-ck7w | 2/24 |
| Std-array-map | 0/297 |
| Std-arrays | 263/296 |
| Map/filter | 25/30 |
| Request/SAPI | 1/41 |
| Anon-class | 0/15 |
| Interface-decl | 0/23 |
| Interface-impl | 0/15 |
| Trait-decl | 0/25 |
| Call-unpack | 13/20 |
| By-ref-call-unpack-dkyr | 2/3 |
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
| Std-string-array-ppri | 18/20 |
| User-comparator-sort-vq7w | 20/26 |
| Std-strings-m8pk | 21/21 |
| Std-strings-almd | 12/13 |
| array_rand | 6/7 |
| Zend-op/control | 21/26 |
| Zend-lit/op-3ijs | 27/30 |
| Parser-cfny | 31/34 |
| Parser-control-psbp | 28/28 |
| Binary-key | 1/1 |
| Runtime-config | 10/54 |
| COW-gate | 26/26 |
| COW-reference-tiqh | 21/21 |
| COW-reference-mqvk | 27/28 |
| COW-reference-vq7w | 24/24 |
| COW-reference-25s0 | 26/27 |
| COW-reference-dgj9 | 8/22 |
| COW-reference-d0lg | 23/48 |
| Broad-25s0-runnable | 527/591 |
| Broad-ppri-runnable | 528/591 |
| 1k-baseline | 573/1000 |
| Full-PHPT inventory | 21867/21867 |
| Full-PHPT 1k runnable | 383/1000 |

## Rules

- Update `PROGRESS.md`, then run `tools/update-progress-mirrors.sh`.
- Keep mirrors compact and numeric.
- Never claim broad PHP compatibility from row-specific patches.
