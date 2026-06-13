# PTN Progress

Refresh: 2026-06-13T19:07Z
Measured: `ptn-550s.3` array-internal COW frontier rebased after
`ptn-qsmv.8`.

Recent RC slices cover constants, includes, closures, object callables,
reflection metadata, helper internals, PHPT blocker classification with opt-in
SKIPIF preconditions, environment/include-path, filesystem/path helpers,
streams, `function_exists()`, `get_parent_class()`, expanded string rows,
broad 1k maps, asymmetric property set visibility, and array-internal COW
blocker classification for unmodeled mutating helpers.

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
- The broad array-internal COW frontier now classifies all 72 selected
  `ext/standard/tests/array` rows before execution: 58 `unsupported-internal`,
  9 `unsupported-language`, and 5 `unsupported-class-metadata`.

## Verification

`ptn-550s.3` adds
`tools/phpt-array-internal-cow-frontier-manifest.txt` for unmodeled mutating
helpers including `array_splice()`, `array_walk_recursive()`,
`array_multisort()`, `usort()`, `uasort()`, and `uksort()`. Worker evidence
before the classifier change was 72 selected, 58 runnable, and 14 excluded;
current classify-only output is 72 selected, 0 runnable, and 72 excluded.

`ptn-qsmv.8` kept the PHPT shell checks, `phpt_classifier`, runner `--help`,
and `cargo build --bin phpc` green. With `--classify-harness-programs`, broad
1k kept 424 runnable / 576 excluded, moving 6 rows to `harness-skipif`; broad
5k kept 2,254 runnable / 2,746 excluded, moving 310 rows.

Follow-ups remain typed properties, traits, magic methods, attributes,
arrow functions, heredoc/nowdoc parsing, userland `throw`, readonly metadata,
first-class callables, dynamic includes, unsupported internals, scalar-offset
lvalues, `Traversable`, embedded-NUL internals, diagnostics, formatter/callback
parity, process boundaries, SKIPIF modeling under `ptn-awta`, and classifier
scan batching under `ptn-qwby`.
