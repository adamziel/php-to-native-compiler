# PTN Progress

Refresh: 2026-06-13T17:00Z
Measured: `ptn-hu7e` PHPT attribute-syntax classification on `origin/master`
`875f567e`; classifier/build gates passed.

Recent RC slices cover constants, includes, closures, `stdClass`,
properties/destructors, ReflectionFunction metadata, array helpers, key-aware
and callback-aware set operations, formatted output, path/search/count/string
internals, scalar/array type hints, scalar/operator PHPT coverage,
unsupported-language and harness PHPT preclassification including PHP
attributes, invokable object callables, object cloning, magic-method
visibility warnings, `php_uname()`, environment/include-path helpers,
`getcwd()`/`chdir()`, runner ini values, statement-form `(void)` casts, offset
compound/coalescing, and property/static property compounds.

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
categories for cleanup/env/noisy/SAPI harness sections and broad unsupported
language forms, statement-form `(void)`, array mutators, inc/dec, `global`,
dynamic variables, offset compound/null coalescing, and direct property/static
property compounds.

## Remaining Bounded Exclusions

- None among the 410 runnable rows in the current bounded manifest.
- None among the 5 callback/callable frontier rows in the current callback
  manifest.

## Verification

`ptn-hu7e` verification: broad 1k raw `#[` scan found 203 rows; 153 `--FILE--`
rows classify with PHP attribute syntax evidence, 44 are excluded by earlier
extension/language/ini rules, and 6 title-only marker rows remain runnable.
`cargo test --test phpt_classifier` passed 3/3, `cargo build --bin phpc`,
`cargo fmt --check`, and `bash -n tools/phpt-classifier.sh` passed.

Follow-ups remain typed properties, interfaces/traits, magic methods, PHP
attributes/reflection metadata, first-class callables, dynamic includes,
unsupported internals, scalar-offset lvalues, `Traversable`, embedded-NUL
internals, host-locale parity, conversion diagnostics, formatter edge parity,
array callback diagnostic parity, and process execution.
