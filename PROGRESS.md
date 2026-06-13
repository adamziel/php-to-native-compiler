# PTN Progress

Refresh: 2026-06-13T16:43Z
Measured: `ptn-iwit` PHPT classify-only blocker maps on `origin/master`
`9e8306383`; broad 1k classification selected 1,000 rows, kept 742 runnable,
and excluded 258 with blocker evidence.

Recent RC slices cover constants, includes, closures, `stdClass`,
properties/destructors, ReflectionFunction metadata, array helpers, key-aware
and callback-aware set operations, formatted output, path/search/count/string
internals, scalar/array type hints, broad array/scalar/operator PHPT coverage,
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
| PHPT bounded manifest | 400 | 400 | 0 |
| PHPT Zend rows | 94 | 94 | 0 |
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

- None among the 400 runnable rows in the current bounded manifest.
- None among the 5 callback/callable frontier rows in the current callback
  manifest.

## Verification

`ptn-iwit` verification: classify-only broad 1k selected 1,000 rows, with 742
runnable and 258 exclusions: unsupported-language 143, unsupported-ini 73,
unsupported-extension 20, harness-cleanup 4, SAPI behavior 13,
process-boundary 3, external-service 1, and environment-assumption 1. A 40-row
5k cleanup slice classified 40/40 excluded rows, including 33
`harness-cleanup`; `cargo fmt --check` and shell syntax checks passed.

Follow-ups remain typed properties, interfaces/traits, magic methods,
first-class callables, dynamic includes, unsupported internals,
scalar-offset lvalues, `Traversable`, remaining embedded-NUL internals,
host-locale parity, conversion diagnostics, exact formatter edge parity,
array callback diagnostic parity, and process execution.
