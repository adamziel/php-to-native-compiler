# PTN Progress

Refresh: 2026-06-13T19:18Z
Measured: `ptn-550s.1` broad COW risk inventory after `ptn-550s.3`.

Recent RC slices cover constants, includes, closures, object callables,
reflection metadata, helper internals, PHPT blocker classification, streams,
filesystem/path helpers, expanded string rows, asymmetric property set
visibility, array-internal COW classification, and a broad COW risk map.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 651 | 651 | 0 |
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
| PHPT COW manifest | 29 | 29 | 0 |
| PHPT array-internal COW frontier | 72 | 0 | 72 |
| Post-merge COW gate | 26 | 26 | 0 |
| PHPT callback manifest | 5 | 5 | 0 |
| PHPT include manifest | 2 | 2 | 0 |
| PHPT formatted string rows | 75 | 25 | 50 |
| PHPT broad 1k baseline | 1000 | 265 | 735 |

## Remaining Bounded Exclusions

- No known failures among the 485 accepted bounded rows. Classify-only reports
  459 runnable rows and 26 excluded rows outside the string slice.
- Callback frontier is 5/5; filesystem/path/process remains 13/46 with
  harness-cleanup and process-boundary exclusions.
- The array-internal COW frontier classifies all 72 selected
  `ext/standard/tests/array` rows before execution: 58 `unsupported-internal`,
  9 `unsupported-language`, and 5 `unsupported-class-metadata`.

## Verification

`ptn-550s.1` adds `docs/COW_BROAD_PHPT_RISK_MAP_2026-06-13.md`. 1k
classify-only selected 1,000 rows: 431 runnable, 569 excluded; broad 5k
selected 5,000 rows: 2,564 runnable, 2,436 excluded against php-src
`8c63ec400ce8e07c57a8d9499317b96a8beafb8b`. The focused COW manifest selected
29 rows and passed 29/29.

The COW map classifies 92 broad COW-sensitive rows across assignment/reference
cells, function boundaries, foreach, closures, array internals, object/property
references, generator/fiber boundaries, and string/scalar aliasing. Follow-ups
are `ptn-550s.2` through `ptn-550s.7`, plus prerequisites `ptn-0pys`,
`ptn-nya1`, `ptn-98lj`, and `ptn-qsmv.12`.

`ptn-550s.3` classifies the array-internal COW frontier at 72 selected,
0 runnable, and 72 excluded. With `--classify-harness-programs`, broad 1k kept
424 runnable / 576 excluded, moving 6 rows to `harness-skipif`; broad 5k kept
2,254 runnable / 2,746 excluded, moving 310 rows.

Follow-ups remain typed properties, traits, magic methods, attributes,
heredoc/nowdoc, userland `throw`, readonly metadata, first-class callables,
dynamic includes, unsupported internals, scalar-offset lvalues, `Traversable`,
embedded-NUL internals, formatter/callback parity, process boundaries, SKIPIF
modeling under `ptn-awta`, and classifier scan batching under `ptn-qwby`.
