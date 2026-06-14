# PTN Progress

Refresh: 2026-06-14T01:16Z
Measured: `ptn-x6x5` array callback validation after `ptn-oz24`.

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
| PHPT generator/fiber COW boundary bucket | 12 | 0 | 12 |
| PHPT broad 1k attribute blocker bucket | 141 | 0 | 141 |
| Post-merge COW gate | 26 | 26 | 0 |
| PHPT callback manifest | 5 | 5 | 0 |
| PHPT include manifest | 2 | 2 | 0 |
| PHPT formatted string rows | 75 | 25 | 50 |
| PHPT focused array predicate/find rows | 4 | 4 | 0 |
| PHPT broad 1k baseline | 1000 | 265 | 735 |

## Remaining Exclusions

- Bounded rows are 485/485; classify-only is 459 runnable/26 excluded.
- Array-internal COW is 72/17/17; foreach/reference is 103/51/31.
- Reference-call bucket is 12 selected, 11 runnable, 1 excluded, 9 pass.
- Broad 1k classify-only is 421 runnable/579 excluded.
- Runtime buckets add 17 diagnostics rows and 9 assertion-runtime rows.
- Attribute map records 141 broad 1k exclusions.

## Verification

`ptn-x6x5` validates callback operands before array-helper iteration. Focused
native validation passes; array callback/set is 65 selected/runnable, 46
passing, 19 mapped failures.

`ptn-oz24` records 141 broad 1k attribute exclusions.

`ptn-lrlt` adds diagnostic/assertion buckets; broad 1k classify-only is
421/579. `ptn-yl7i` adds class-name hints.

`ptn-kgqa` adds array predicate/find callbacks, focused PHPT 4/4.
`ptn-550s.10` keeps foreach-list 4/4 and COW gate 26/26.

`ptn-vwyp` classifies generator/fiber COW: 46 selected, 32 runnable, 14
excluded. `ptn-2juv` records broad 1k: 1,000/447/553.

Follow-ups: properties, traits, attributes, traces, nullable,
generators/Fibers, includes, callbacks, `Traversable`, INI.
