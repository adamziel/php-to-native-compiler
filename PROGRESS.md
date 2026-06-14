# PTN Progress

Refresh: 2026-06-14T01:55Z.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 714 | 714 | 0 |
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
- Zend frontier: 32 runnable, 22 pass; residuals: append lvalues, next-key
  overflow, object/member lvalues, compound TypeErrors, `$this`.
- `ptn-igxz`: broad 1,000 selected, 443 runnable, 557 excluded; diff/intersect
  leaves three nested-array warning rows.
- Broad 1k: 422 runnable/578 excluded; attributes 141; generator/fiber COW
  boundary 12 selected/0 runnable/12 excluded.

## Verification

`ptn-igxz` makes catchable set-operation TypeErrors, accepts one-source forms,
prevalidates callbacks, lexes `.5` floats, and keeps registry order sorted.
Evidence: native tests, inventory 3+714, PHPT
diff/intersect 58/61.

`ptn-gwlo` maps a 32-row Zend assignment/reference frontier:
`tools/phpt-zend-assignment-reference-frontier-manifest.txt` and focused PHPT
`run-20260614T012211Z-manifest.log` at 22/32.

`ptn-x6x5` validates callbacks; native passes, and callback PHPT is
65 selected, 46 pass, 19 mapped failures.
