# PTN Progress

Refresh: 2026-06-13T16:51Z
Measured: `ptn-oy8c` scalar/operator bounded-manifest expansion on
`origin/master` `be870adb9`; the focused 30-row scalar/operator PHPT gate
passed 30/30.

Recent RC slices cover constants, includes, closures, `stdClass`,
properties/destructors, ReflectionFunction metadata, array helpers, key-aware
and callback-aware set operations, formatted output, path/search/count/string
internals, scalar/array type hints, scalar/operator PHPT coverage,
unsupported-language and harness PHPT preclassification, invokable object
callables, object cloning, magic-method visibility warnings, `php_uname()`,
environment/include-path helpers, `getcwd()`/`chdir()`, runner ini values,
statement-form `(void)` casts, offset compound/coalescing, and property/static
property compounds.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 645 | 645 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 410 | 410 | 0 |
| PHPT Zend rows | 104 | 104 | 0 |
| PHPT ext/standard rows | 223 | 223 | 0 |
| PHPT focused array key/callback set rows | 75 | 38 | 37 |
| PHPT focused stream rows | 2 | 2 | 0 |
| PHPT focused cwd rows | 2 | 2 | 0 |
| PHPT tests/basic+func+lang | 78 | 78 | 0 |
| PHPT other rows | 5 | 5 | 0 |
| PHPT COW manifest | 29 | 29 | 0 |
| Post-merge COW gate | 26 | 26 | 0 |
| PHPT callback manifest | 5 | 5 | 0 |
| PHPT include manifest | 2 | 2 | 0 |
| PHPT formatted string rows | 75 | 25 | 50 |

## RC Surface

Parser/IR/C backend, boxed values, variables/constants, strings, scalar
operators, arrays, `foreach`, control flow, includes/once guards, selected
internals, COW/reference slices, functions, closures, `stdClass`, class/object
shells/constants, properties, destructors, reflection, assertions,
namespace/import forms, streams, file reads/writes, array/string/numeric
helpers through key-aware/callback-aware set operations, expanded
array/string/operator PHPT variations, metadata constants, scalar conversions,
locale/callable diagnostics, SPL object identity, object cloning, runner
ini/include-path state, formatted stream/output helpers, PHPT blocker
categories for cleanup/env/noisy/SAPI harness sections, statement-form
`(void)`, array mutators, inc/dec, `global`, dynamic variables, offset
compound/null coalescing, and direct property/static property compounds.

## Remaining Bounded Exclusions

- None among the 410 runnable rows in the current bounded manifest.
- None among the 5 callback/callable frontier rows in the current callback
  manifest.

## Verification

`ptn-oy8c` verification: focused scalar/operator PHPT manifest selected 30
rows, ran 30, passed 30, failed 0, skipped 0, warned 0. The merged bounded
manifest now has 410 rows: Zend 104, ext/standard 223, tests/basic+func+lang
78, and other 5. `ptn-iwit` classify-only broad 1k remains selected 1,000,
runnable 742, excluded 258 with blocker evidence.

Follow-ups remain typed properties, interfaces/traits, magic methods,
first-class callables, dynamic includes, unsupported internals,
scalar-offset lvalues, `Traversable`, remaining embedded-NUL internals,
host-locale parity, conversion diagnostics, exact formatter edge parity,
array callback diagnostic parity, and process execution.
