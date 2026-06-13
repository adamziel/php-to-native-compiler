# PTN Progress

Refresh: 2026-06-13T20:29Z
Measured: `ptn-flje` broad COW/reference frontier after `ptn-550s.8`.

Recent RC slices cover constants, includes, closures/arrow functions, object
callables, PHPT blockers, streams, filesystem/path helpers, strings,
asymmetric property set visibility, COW maps, quiet string-offset diagnostics,
`array_walk()` userdata separation, and broad COW blocker mapping.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 670 | 670 | 0 |
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
| PHPT array-internal COW frontier | 72 | 0 | 72 |
| PHPT COW foreach/reference frontier | 103 | 31 | 72 |
| Post-merge COW gate | 26 | 26 | 0 |
| PHPT callback manifest | 5 | 5 | 0 |
| PHPT include manifest | 2 | 2 | 0 |
| PHPT formatted string rows | 75 | 25 | 50 |
| PHPT string/scalar alias rows | 35 | 23 | 12 |
| PHPT broad 1k baseline | 1000 | 265 | 735 |

## Remaining Exclusions

- No known failures among the 485 accepted bounded rows. Classify-only reports
  459 runnable rows and 26 excluded rows outside the string slice.
- Array-internal COW frontier: 72 selected, 0 runnable, 72 excluded. COW
  foreach/reference frontier: 103 selected, 51 runnable, 31 passing.
- Zend arrow rows: 001-004 pass; remaining rows need reflection closure
  binding metadata, nullable types, generator `yield`, or `assert.exception`
  ini.

## Verification

`ptn-flje` adds `tools/phpt-cow-broad-frontier-manifest.txt` and
`docs/COW_BROAD_FRONTIER_BLOCKERS_2026-06-13.md`: current classify-only is
46 selected, 31 runnable, 15 excluded. Historical worker execution before
newer classifier blockers ran 42 rows, passed 11, and failed 31. Follow-ups
are `ptn-begn`, `ptn-8d2u`, `ptn-1om3`, `ptn-vwyp`, and `ptn-f0rp`.

`ptn-550s.8` reruns seven residual non-recursive `array_walk()` rows: six pass
and `bug39576` is classified as class-metadata before the userdata path.

Recent COW gates remain: `ptn-550s.5` string/scalar alias PHPT 23/23,
`ptn-550s.4` COW manifest 54/54, and `ptn-550s.3` array-internal COW frontier
72/0/72. Follow-ups remain typed properties, traits, magic methods, attributes,
heredoc/nowdoc, userland `throw`, readonly metadata, nullable types, generator
`yield`, first-class callables, dynamic includes, unsupported internals,
scalar-offset lvalues, `Traversable`, formatter/callback parity, process
boundaries, and classifier scan batching.
