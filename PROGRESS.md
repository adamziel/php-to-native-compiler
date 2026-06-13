# PTN Progress

Refresh: 2026-06-13T18:54Z
Measured: `ptn-qsmv.8` opt-in PHPT `--SKIPIF--` harness classification
rebased after `ptn-4tfb`.

Recent RC slices cover constants, includes, closures, object callables,
reflection metadata, helper internals, PHPT blocker classification with opt-in
SKIPIF preconditions, environment/include-path, filesystem/path helpers,
streams, `function_exists()`, `get_parent_class()`, 50 string rows, broad 1k
maps, and asymmetric property set visibility.

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
| Post-merge COW gate | 26 | 26 | 0 |
| PHPT callback manifest | 5 | 5 | 0 |
| PHPT include manifest | 2 | 2 | 0 |
| PHPT formatted string rows | 75 | 25 | 50 |
| PHPT broad 1k baseline | 1,000 | 265 | 735 |

## RC Surface

Parser/IR/C backend, boxed values, variables/constants, strings, scalar
operators, arrays, `foreach`, control flow, includes, selected internals,
COW/reference slices, functions/closures, class/object shells, reflection,
streams, file reads/writes, helper internals, runner state, PHPT blockers,
filesystem/path helpers, offset/property compounds, expanded string rows,
asymmetric metadata, and opt-in SKIPIF harness classification.

## Remaining Bounded Exclusions

- No known failures among the 485 accepted rows in the bounded manifest.
  Classify-only reports 459 runnable rows and 26 excluded rows outside the
  added string slice.
- Callback frontier is 5/5; filesystem/path/process remains 13/46 with
  harness-cleanup and process-boundary exclusions.

## Verification

`ptn-qsmv.8` verification: `cargo fmt --check`, PHPT shell syntax checks,
`cargo test --test phpt_classifier` 8/8, runner `--help`, and
`cargo build --bin phpc` passed. Bounded classify-only: 485 selected,
459 runnable, 26 excluded.

With `--classify-harness-programs`, broad 1k kept 424 runnable / 576 excluded;
6 rows moved to `harness-skipif`. Broad 5k kept 2,254 runnable / 2,746
excluded and moved 310 rows to `harness-skipif`.

`ptn-qsmv.9` passed `cargo check`, focused native tests,
`phpt_classifier`, and full `cargo test`. `ptn-4tfb` maps broad 1k
blockers: 265 pass / 186 fail among 451 runnable.

Follow-ups remain typed properties, traits, magic methods, attributes,
arrow functions, heredoc/nowdoc parsing, userland `throw`, readonly metadata,
first-class callables, dynamic includes, unsupported internals, scalar-offset
lvalues, `Traversable`, embedded-NUL internals, diagnostics, formatter/callback
parity, process boundaries, SKIPIF modeling under `ptn-awta`, and classifier
scan batching under `ptn-qwby`.
