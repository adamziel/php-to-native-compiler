# PTN Progress

Refresh: 2026-06-13T17:51Z
Measured: `ptn-7o62` attribute syntax blocker documentation/evidence rebased
after `ptn-c7iw` asymmetric visibility classification.

Recent RC slices cover constants, includes, closures, `stdClass`,
properties/destructors, reflection metadata, array helpers, formatted output,
scalar/operator PHPT rows, PHPT classification for attributes and other broad
syntax blockers, object callables/cloning, environment/include-path helpers,
filesystem metadata/path helpers, stream writes, `function_exists()`
static-method separation, and bounded `get_parent_class()`.

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
properties/destructors, reflection, namespaces, streams, file reads/writes,
array/string/numeric helpers, runner ini/include-path state, PHPT blocker
categories, filesystem metadata/path helpers, `(void)`, mutators, inc/dec,
dynamic variables, offset/property compounds, static-method-aware
`function_exists()`, and bounded `get_parent_class()` metadata.

## Remaining Bounded Exclusions

- None among the 435 runnable rows in the current bounded manifest.
- None among the 5 callback/callable frontier rows in the current callback
  manifest.
- The focused filesystem/path/process manifest classifies 8 harness-cleanup
  rows and 25 process-boundary rows.

## Verification

`ptn-7o62` evidence: focused attributes manifest selected 204 rows, classified
187 unsupported-language and 8 unsupported-extension rows, and left 9 runnable
attribute API rows failing. Before this slice, the same manifest had 137
runnable rows, 59 unsupported-language rows, and 8 unsupported-extension rows,
so 128 broad rows moved into blocker evidence. Current-branch classifier
syntax, classifier tests, and fmt passed.

Previous `ptn-c7iw` evidence moved 38 rows from runnable to
`unsupported-language` asymmetric property visibility blockers.

Follow-ups remain typed properties, interfaces/traits, magic methods, PHP
attributes/reflection metadata, arrow functions, heredoc/nowdoc parsing,
userland `throw`, readonly and asymmetric property metadata, first-class
callables, dynamic includes, unsupported internals, scalar-offset lvalues,
`Traversable`, embedded-NUL internals, host-locale parity, conversion
diagnostics, formatter edge parity, array callback diagnostic parity,
filesystem/process boundaries, and process execution.
