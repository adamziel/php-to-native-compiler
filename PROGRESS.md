# PTN Progress

Refresh: 2026-06-13T18:17Z
Measured: `ptn-qsmv.3` broad ext/standard strings manifest expansion rebased
after `ptn-7o62` attribute syntax blocker evidence; the integrated 50-row
ext/standard string manifest passed 50/50.

Recent RC slices cover constants, includes, closures, `stdClass`, object
callables/cloning, reflection metadata, helper internals, formatted output,
PHPT blocker classification, environment/include-path, filesystem/path
helpers, streams, `function_exists()` static-method separation, bounded
`get_parent_class()`, and 50 additional ext/standard
string rows across cslashes, basename, bug regressions, chop/chunk-split,
explode, `str_split()`, `strncmp()`, and `strncasecmp()`.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 645 | 645 | 0 |
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
namespaces, streams, file reads/writes, helper internals, runner state, PHPT
blockers, filesystem/path helpers, dynamic variables, offset/property
compounds, `function_exists()`, `get_parent_class()`, and expanded string rows.

## Remaining Bounded Exclusions

- No known failures among the 485 accepted rows in the bounded manifest.
  Classify-only reports 459 runnable rows and 26
  pre-existing excluded rows outside the added string slice.
- Callback frontier is 5/5; filesystem/path/process remains 13/46 with
  harness-cleanup and process-boundary exclusions.

## Verification

`ptn-qsmv.3` verification: broad ext/standard strings classify-only selected
734 rows, runnable 677, excluded 57. Candidate runs found passing families:
120 rows yielded 30 passes, then 45 rows yielded 38 passes. The rebased
50-row string manifest selected 50, runnable 50, ran 50, passed 50, and had
0 failures/skips/warnings. The bounded manifest is now 485 rows: Zend 119,
ext/standard 280, tests/basic+func+lang 78, and other 8. Full bounded
classify-only selected 485 rows, runnable 459, excluded 26; none of the added
50 string rows were excluded.

Follow-ups remain typed properties, interfaces/traits, magic methods,
attributes/reflection, arrow functions, heredoc/nowdoc parsing, userland
`throw`, readonly/asymmetric metadata, first-class callables, dynamic includes,
unsupported internals, scalar-offset lvalues, `Traversable`, embedded-NUL
internals, conversion diagnostics, formatter/callback
parity, filesystem/process boundaries, and process execution.
