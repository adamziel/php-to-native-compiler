# PTN Progress

Refresh: 2026-06-14T01:45Z
Measured: `ptn-gwlo` after `ptn-x6x5`.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 713 | 713 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 485 | 485 | 0 |
| PHPT Zend rows | 119 | 119 | 0 |
| PHPT ext/standard rows | 280 | 280 | 0 |
| PHPT focused array key/callback set rows | 75 | 38 | 37 |
| PHPT focused array callback validation rows | 65 | 46 | 19 |
| PHPT focused stream rows | 2 | 2 | 0 |
| PHPT focused cwd rows | 2 | 2 | 0 |
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
| PHPT generator/fiber COW boundary bucket | 12 | 0 | 12 |
| PHPT broad 1k attribute blocker bucket | 141 | 0 | 141 |
| Post-merge COW gate | 26 | 26 | 0 |
| PHPT callback manifest | 5 | 5 | 0 |
| PHPT include manifest | 2 | 2 | 0 |
| PHPT formatted string rows | 75 | 25 | 50 |
| PHPT focused array predicate/find rows | 4 | 4 | 0 |
| PHPT broad 1k baseline | 1000 | 265 | 735 |

## Remaining Exclusions

- Bounded: 485/485; classify-only: 459 runnable/26 excluded.
- COW/reference: array-internal 72/17/55, foreach/reference 103/31/72,
  reference-call 12 selected, 11 runnable, 9 pass.
- Zend frontier: 32 runnable, 22 pass; residuals cover append lvalues,
  next-key overflow, object/member lvalues, compound TypeErrors, `$this`.
- Broad 1k: 422 runnable/578 excluded; attributes 141; generator/fiber COW
  boundary 12 selected/0 runnable/12 excluded.

## Verification

`ptn-gwlo` maps a 32-row Zend assignment/reference/object-write frontier:
`classification-20260614T011318Z.tsv`,
`tools/phpt-zend-assignment-reference-frontier-manifest.txt`, and focused PHPT
`run-20260614T012211Z-manifest.log` at 22/32.

`ptn-x6x5` validates array-helper callbacks; native validation passes, and
callback validation PHPT is 65 selected, 46 pass, 19 mapped failures.

`ptn-oz24`: 141 broad 1k attribute exclusions. `ptn-lrlt`: diagnostic and
assertion buckets; broad 1k classify-only 421/579. `ptn-yl7i`: class-name
hints.

`ptn-kgqa`: array predicate/find callbacks, PHPT 4/4. `ptn-550s.10`:
foreach-list 4/4 and COW gate 26/26.

`ptn-vwyp`: generator/fiber COW 46/32/14. `ptn-2juv`: broad 1k 1,000/447/553.
