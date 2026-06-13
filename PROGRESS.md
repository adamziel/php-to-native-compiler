# PTN Progress

Refresh: 2026-06-13T17:16Z
Measured: `ptn-qsmv.5` object/class/callable metadata expansion rebased after
`ptn-6kt2` filesystem/path support.

Recent RC slices cover constants, includes, closures, `stdClass`,
properties/destructors, reflection metadata, array helpers, formatted output,
string/path/count internals, scalar/operator PHPT rows, PHPT classification
including PHP attributes, invokable object callables, object cloning,
environment/include-path helpers, `getcwd()`/`chdir()`, compound
offsets/properties, filesystem metadata/path helpers, `chmod()`/`touch()`,
`fwrite()`/`fputs()` stream writes, `function_exists()` global/static-method
separation, and bounded `get_parent_class()`.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 645 | 645 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 435 | 435 | 0 |
| PHPT Zend rows | 119 | 119 | 0 |
| PHPT ext/standard rows | 230 | 230 | 0 |
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
COW/reference slices, functions, closures, `stdClass`, class/object shells,
properties, destructors, reflection, assertions, namespaces, streams, file
reads/writes, array/string/numeric helpers, scalar conversions, runner
ini/include-path state, PHPT blocker categories, filesystem metadata/path
helpers, statement-form `(void)`, array mutators, inc/dec, `global`, dynamic
variables, offset compound/null coalescing, property compounds,
`function_exists()` global/static-method separation, and bounded
`get_parent_class()` metadata.

## Remaining Bounded Exclusions

- None among the 435 runnable rows in the current bounded manifest.
- None among the 5 callback/callable frontier rows in the current callback
  manifest.
- The focused filesystem/path/process manifest classifies 8 harness-cleanup
  rows and 25 process-boundary rows.

## Verification

`ptn-qsmv.5` verification: targeted native metadata gates passed, `cargo fmt
--check` passed, and the 25 newly added object/class/callable PHPT rows passed
with 25 selected, 25 runnable, 25 passes, and no exclusions.

Previous `ptn-6kt2` verification remains: focused native compile tests and
`tools/run-phpt-manifest.sh tools/phpt-filesystem-path-process-manifest.txt`
passed with 46 selected rows, 13 runnable rows, 13 passes, 8 harness-cleanup
exclusions, and 25 process-boundary exclusions.

Follow-ups remain typed properties, interfaces/traits, magic methods, PHP
attributes/reflection metadata, first-class callables, dynamic includes,
unsupported internals, scalar-offset lvalues, `Traversable`, embedded-NUL
internals, host-locale parity, conversion diagnostics, formatter edge parity,
array callback diagnostic parity, and process execution.
