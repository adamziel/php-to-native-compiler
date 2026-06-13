# PTN Progress

Refresh: 2026-06-13T17:06Z
Measured: `ptn-cm8x` overlapping PHP-attribute PHPT classification coverage on
`origin/master` `962a430f`; this keeps the stronger merged classifier and adds
class-attribute regression coverage.

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

`ptn-cm8x` verification: broad 1k raw `#[` scan remains 203 rows: 153
PHP-attribute blockers, 44 earlier extension/language/ini exclusions, and 6
title-only rows runnable. The focused attribute manifest selected 25, ran 0,
and excluded 25 as unsupported-language; classifier tests passed 3/3,
`cargo build --bin phpc`, `cargo fmt --check`, and classifier `bash -n` passed.

Follow-ups remain typed properties, interfaces/traits, magic methods, PHP
attributes/reflection metadata, first-class callables, dynamic includes,
unsupported internals, scalar-offset lvalues, `Traversable`, embedded-NUL
internals, host-locale parity, conversion diagnostics, formatter edge parity,
array callback diagnostic parity, and process execution.
