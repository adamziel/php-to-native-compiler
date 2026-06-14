# PTN Progress

Refresh: 2026-06-14T02:22Z.
Measured: `ptn-4fd3` heredoc classifier frontier.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 715 | 715 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 485 | 485 | 0 |
| PHPT Zend rows | 119 | 119 | 0 |
| PHPT ext/standard rows | 280 | 280 | 0 |
| PHPT focused array key/callback set rows | 75 | 38 | 37 |
| PHPT focused array callback validation rows | 65 | 46 | 19 |
| PHPT focused array diff/intersect rows | 61 | 58 | 3 |
| PHPT focused filesystem/path/process rows | 46 | 13 | 33 |
| PHPT tests/basic+func+lang | 78 | 78 | 0 |
| PHPT other rows | 8 | 8 | 0 |
| PHPT COW manifest | 54 | 54 | 0 |
| PHPT nested foreach/reference rows | 3 | 2 | 1 |
| PHPT array-internal COW frontier | 72 | 17 | 55 |
| PHPT COW foreach/reference frontier | 103 | 31 | 72 |
| PHPT foreach list destructuring rows | 4 | 4 | 0 |
| PHPT broad reference-call bucket | 12 | 9 | 3 |
| PHPT broad Zend assignment/reference frontier | 32 | 22 | 10 |
| PHPT broad 1k attribute blocker bucket | 141 | 0 | 141 |
| PHPT broad heredoc/nowdoc array frontier | 70 | 14 | 56 |
| Post-merge COW gate | 26 | 26 | 0 |
| PHPT callback manifest | 5 | 5 | 0 |
| PHPT include manifest | 2 | 2 | 0 |
| PHPT formatted string rows | 75 | 25 | 50 |
| PHPT focused array predicate/find rows | 4 | 4 | 0 |
| PHPT broad 1k baseline | 1000 | 265 | 735 |

## Remaining Exclusions

- Bounded: 485/485; classifier 459 runnable/26 excluded.
- Broad 1k classify-only is 443 runnable/557 excluded after `ptn-4fd3`.
- Heredoc/nowdoc: 70 rows left the coarse blocker; 21 runnable, 49 magic
  method/reflection metadata.
- `ptn-1f0f` keeps attribute, arginfo/class registry, diagnostics, assertion,
  and exception trace rows in explicit buckets.
- Zend assignment/reference is 22/32; array diff/intersect is 58/61.
- COW/reference frontiers: array-internal 17/72, foreach/reference 31/103,
  reference-call 9/12.

## Verification

`ptn-4fd3` keeps plain heredoc and nowdoc PHPT rows runnable while classifying
interpolating heredoc bodies. Broad 1k classify-only moved from 422/578 to
443/557; focused `tools/phpt-heredoc-nowdoc-frontier-manifest.txt` selected 70
rows: 21 runnable, 14 pass, 7 fail, 49 excluded.

`ptn-1f0f` classifies broad diagnostics metadata; broad classify-only was
409/591 and classifier tests passed 22/22.

`ptn-igxz` makes array set-operation TypeErrors catchable, accepts one-source
forms, prevalidates callbacks, lexes `.5` floats, and keeps registry order
sorted. Native target tests pass; diff/intersect PHPT is 58/61.
