# PTN Progress

Refresh: 2026-06-14T01:00Z
Measured: `ptn-lrlt` runtime diagnostics classifier after `ptn-yl7i`.

Slices cover callable/object, COW, PHPT blockers, arrays, and diagnostics.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 712 | 712 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 485 | 485 | 0 |
| PHPT Zend rows | 119 | 119 | 0 |
| PHPT ext/standard rows | 280 | 280 | 0 |
| PHPT focused array key/callback set rows | 75 | 38 | 37 |
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
- Broad 1k classify-only is now 421 runnable/579 excluded.
- Runtime buckets add 17 diagnostics rows and 9 assertion-runtime rows.
- Generator/fiber COW bucket is 12 selected/0 runnable/12 excluded.

## Verification

`ptn-lrlt` adds buckets for unsupported diagnostic APIs and assertion modes
while keeping `assert()` and diagnostic names in strings/comments
runnable. Broad 1k classify-only is 421 runnable/579 excluded.

`ptn-yl7i` adds class-name hints and generated checks; target
`call_user_func_by_ref` passes in the reference bucket.

`ptn-kgqa` adds array predicate/find callback scanning. Focused PHPT is 4/4.
`ptn-550s.10` keeps foreach-list PHPT 4/4 and COW gate 26/26.

`ptn-vwyp` classifies COW generator/fiber rows: 46 selected, 32 runnable, 14
excluded. `ptn-2juv` records broad 1k: 1,000 selected, 447 runnable, 553
classified.

Follow-ups remain typed properties, traits, magic methods, attributes,
Exception traces, nullable types, generators/Fibers, includes, callback
arrays, `Traversable`, and INI modes.
