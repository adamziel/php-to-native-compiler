# PTN Progress

Refresh: 2026-06-13T18:49Z
Measured: `ptn-4tfb` broad 1k blocker map rebased after `ptn-qsmv.9`.

Recent RC slices cover constants, includes, closures, object callables/cloning,
reflection metadata, helper internals, PHPT blocker classification,
environment/include-path, filesystem/path helpers, streams, `function_exists()`,
`get_parent_class()`, 50 additional ext/standard string rows, broad 1k array
and blocker maps, and asymmetric private(set)/protected(set) property metadata
and write checks.

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
filesystem/path helpers, dynamic variables, offset/property compounds,
expanded string rows, and asymmetric property set-visibility metadata.

## Remaining Bounded Exclusions

- No known failures among the 485 accepted rows in the bounded manifest.
  Classify-only reports 459 runnable rows and 26 pre-existing excluded rows
  outside the added string slice.
- Callback frontier is 5/5; filesystem/path/process remains 13/46 with
  harness-cleanup and process-boundary exclusions.

## Verification

`ptn-4tfb` evidence: broad 1k run selected 1,000 rows, excluded 549, ran 451,
passed 265, and failed 186: Zend 45/139 pass, ext/standard 212/295 pass, core
8/17 pass. Largest mapped blockers are standard array comparison/casting
23 rows, standard callback dispatch/diagnostics 21 rows, and Zend
class/object/property dispatch 24 rows.

`ptn-qsmv.9` rebase verification passed `cargo check`, `cargo fmt --check`,
focused asymmetric/property native tests, `phpt_classifier`, and full
`cargo test`. `ptn-qsmv.3` string manifest passed 50/50. `ptn-o7kg` maps
274 runnable array rows by family.

Follow-ups remain typed properties, interfaces/traits, magic methods,
attributes/reflection, arrow functions, heredoc/nowdoc parsing, userland
`throw`, readonly property metadata, first-class callables, dynamic includes,
unsupported internals, scalar-offset lvalues, `Traversable`, embedded-NUL
internals, conversion diagnostics, formatter/callback parity,
filesystem/process boundaries, and process execution.
