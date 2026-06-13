# PTN Progress

Refresh: 2026-06-13T19:49Z
Measured: `ptn-550s.2` COW foreach/reference frontier after `ptn-550s.5`.

Recent RC slices cover constants, includes, closures, object callables,
PHPT blockers, streams, filesystem/path helpers, strings,
asymmetric property set visibility, COW maps, function-boundary PHPT, quiet
string-offset diagnostics, and foreach/reference COW classification.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 652 | 652 | 0 |
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

## Remaining Bounded Exclusions

- No known failures among the 485 accepted bounded rows. Classify-only reports
  459 runnable rows and 26 excluded rows outside the string slice.
- Callback frontier is 5/5; filesystem/path/process remains 13/46 with
  harness-cleanup and process-boundary exclusions.
- The array-internal COW frontier classifies 72 selected rows before
  execution: 58 `unsupported-internal`, 9 `unsupported-language`, and
  5 `unsupported-class-metadata`.

## Verification

`ptn-550s.2` adds `tools/phpt-cow-foreach-reference-manifest.txt` and
`docs/PHPT_COW_FOREACH_REFERENCE_FRONTIER_2026-06-13.md`. Final classify-only
selected 103 rows, kept 51 runnable, and excluded 52 blockers:
13 class-metadata, 18 language, 1 ini, and 20 internal. Final bounded run
selected 103, ran 51, passed 31, failed 20, and skipped/warned 0.

`ptn-550s.5` adds string-offset diagnostics. Its committed manifest
selected 35, ran 23, excluded 12, and passed 23/23. `ptn-550s.4` expanded
focused COW function-boundary rows to 54/54. `ptn-550s.1` adds the broad COW
risk map: 1k classify-only 431 runnable / 569 excluded; 5k 2,564 runnable /
2,436 excluded against php-src `8c63ec400ce8e07c57a8d9499317b96a8beafb8b`.
`ptn-550s.3` classifies the array-internal COW frontier at 72 selected,
0 runnable, and 72 excluded.

Follow-ups remain typed properties, traits, magic methods, attributes,
heredoc/nowdoc, userland `throw`, readonly metadata, first-class callables,
dynamic includes, unsupported internals, scalar-offset lvalues, `Traversable`,
embedded-NUL internals, formatter/callback parity, process boundaries, and
classifier scan batching.
