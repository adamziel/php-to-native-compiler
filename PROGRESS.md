# PTN Progress

Refresh: 2026-06-13T18:36Z
Measured: `ptn-qsmv.9` asymmetric property visibility support rebased after
`ptn-qsmv.3`, `ptn-6fbw`, and `ptn-o7kg`.

Recent RC slices cover constants, includes, closures, object callables/cloning,
reflection metadata, helper internals, PHPT blocker classification,
environment/include-path, filesystem/path helpers, streams, `function_exists()`,
`get_parent_class()`, 50 additional ext/standard string rows, the broad 1k
array-frontier map, and asymmetric private(set)/protected(set) property
metadata/write checks for declared instance and static properties.

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

## RC Surface

Parser/IR/C backend, boxed values, variables/constants, strings, scalar
operators, arrays, `foreach`, control flow, includes, selected internals,
COW/reference slices, functions/closures, class/object shells, reflection,
streams, file reads/writes, helper internals, runner state, PHPT blockers,
filesystem/path helpers, dynamic variables, offset/property compounds,
expanded string rows, and asymmetric property set-visibility metadata.

## Remaining Bounded Exclusions

- No known failures among the 485 accepted rows in the bounded manifest.
  Classify-only reports 459 runnable rows and 26 pre-existing excluded rows
  outside the added string slice.
- Callback frontier is 5/5; filesystem/path/process remains 13/46 with
  harness-cleanup and process-boundary exclusions.

## Verification

`ptn-qsmv.9` verification before rebase: `cargo check`, `cargo fmt --check`,
focused `cargo test asymmetric --test compile_native`, focused
`cargo test property --test compile_native`, `cargo test --test
phpt_classifier`, and full `cargo test` passed. Asymmetric visibility rows now
classify runnable instead of unsupported-language / unsupported-class-metadata.

`ptn-qsmv.3` verification: the rebased 50-row string manifest passed 50/50
with no skips or warnings. `ptn-o7kg` records broad 1k classify-only at 430
runnable / 570 classified and maps 274 runnable array rows by family.

Follow-ups remain typed properties, interfaces/traits, magic methods,
attributes/reflection, arrow functions, heredoc/nowdoc parsing, userland
`throw`, readonly property metadata, first-class callables, dynamic includes,
unsupported internals, scalar-offset lvalues, `Traversable`, embedded-NUL
internals, conversion diagnostics, formatter/callback parity,
filesystem/process boundaries, and process execution.
