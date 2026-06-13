# PTN Progress

Refresh: 2026-06-13T19:34Z
Measured: `ptn-550s.5` string/scalar alias PHPT slice after `ptn-550s.4`.

Recent RC slices cover constants, includes, closures, object callables,
helper internals, PHPT blockers, streams, filesystem/path helpers, strings,
asymmetric property set visibility, array-internal COW, broad COW maps,
function-boundary PHPT, and quiet string-offset diagnostics.

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
- The array-internal COW frontier classifies all 72 selected
  `ext/standard/tests/array` rows before execution: 58 `unsupported-internal`,
  9 `unsupported-language`, and 5 `unsupported-class-metadata`.

## Verification

`ptn-550s.5` adds quiet string-offset `isset()`/`empty()` float/resource
diagnostics and `docs/PHPT_STRING_SCALAR_ALIAS_2026-06-13.md`. Broad candidate
evidence selected 44 rows, ran 36, excluded 8, passed 11, and failed 25. The
committed manifest selected 35, ran 23, excluded 12, and passed 23/23; blockers
are heredoc/nowdoc, ini settings, typed property metadata, and `zend_test`.

`ptn-550s.4` expanded focused COW function-boundary rows to 54/54, including
25 new by-reference/call-frame/callable/`array_reduce()` rows. `ptn-550s.1`
adds the broad COW risk map: 1k classify-only 431 runnable / 569 excluded; 5k
2,564 runnable / 2,436 excluded against php-src
`8c63ec400ce8e07c57a8d9499317b96a8beafb8b`. `ptn-550s.3` classifies the
array-internal COW frontier at 72 selected, 0 runnable, and 72 excluded.

Follow-ups remain typed properties, traits, magic methods, attributes,
heredoc/nowdoc, userland `throw`, readonly metadata, first-class callables,
dynamic includes, unsupported internals, remaining scalar-offset lvalues,
`Traversable`, embedded-NUL internals, formatter/callback parity, process
boundaries, and classifier scan batching.
