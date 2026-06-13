# PTN Progress

Refresh: 2026-06-13T11:44Z
Measured: `ptn-ooqj` integration on current `origin/master` `2f1950ea6`;
verification green.

Recent RC slices cover constants, embedded-NUL `var_export()`, inline HTML
output, includes/once guards, closures, `stdClass`, properties/destructors,
inherited static dispatch, `property_exists()` metadata, array helpers,
`json_encode()`, `printf()`, `basename()`, `pathinfo()`, `dirname()` levels,
`strcasecmp()`, search/count internals, scalar `str_replace()`, `chr()`
diagnostics, `crc32()`, `boolval()`/`floatval()`/`doubleval()`, PHP
version/build/platform metadata constants, standard streams, foreach
destructuring, `global` bindings, dynamic-variable writes/unsets, stream
metadata, locale constants and `setlocale()`, catchable arithmetic/operator
errors, alternate `<>` parsing, offset compound/coalescing, and public
property/static-property compound assignments.

Recent movers include scalar conversion internals for
`boolval()`/`floatval()`/`doubleval()`, `global` function-local binding to root
globals, PHP version/build/platform metadata constants, `dirname()`
positive-level traversal, `pathinfo()`, modeled `LC_*` constants, C/POSIX
`setlocale()`, search/count internals, PHP 8.4 array warning/overflow
behavior, persistent standard streams, `property_exists()`, PHPT manifests,
`crc32()`, `str_replace()` counts, and integer validation for `chr()`,
`intdiv()`, file offsets, and direct property/static compound lvalues.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 592 | 592 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 271 | 269 | 2 |
| PHPT Zend rows | 85 | 84 | 1 |
| PHPT ext/standard rows | 131 | 130 | 1 |
| PHPT focused stream rows | 2 | 2 | 0 |
| PHPT tests/basic+func+lang | 50 | 50 | 0 |
| PHPT other rows | 5 | 5 | 0 |
| PHPT COW manifest | 29 | 29 | 0 |
| Post-merge COW gate | 26 | 26 | 0 |
| PHPT callback manifest | 5 | 3 | 2 |
| PHPT include manifest | 2 | 2 | 0 |

## RC Surface

Parser/IR/C backend, boxed values, variables/constants, strings, scalar
operators, arrays, `foreach`, control flow, includes/once guards, selected
internals, COW/reference slices, functions, closures, `stdClass`,
class/object shells/constants, properties, destructors, reflection, assertions,
namespaces/imports, streams, file reads/writes, array/string/numeric helpers
through `array_udiff*()`, `json_encode()`, `printf()`, `fdiv()`, `explode()`,
`str_replace()`, `strcasecmp()`, `strncmp()`, `strrchr()`, `pathinfo()`,
`dirname()` levels, `crc32()`, `basename()`,
`boolval()`/`floatval()`/`doubleval()`, inline HTML output, locale support,
PHP version/build/platform metadata constants, `var_export()`, array mutators,
inc/dec, `global` bindings, dynamic-variable writes/unsets, and offset
compound/null coalescing assignments plus direct public property/static-property
compound assignments.

## Remaining Bounded Exclusions

- None among the 269 runnable rows in the current 271-row bounded manifest.
  Two selected rows are classified out for unsupported ini requirements.
- Callback manifest has 3 runnable passing rows and 2 unsupported-extension
  exclusions.

## Verification

Current branch verification for `ptn-ooqj`: `cargo fmt --check`, focused
`cargo test compound_assign` 13/13, full `cargo test` with source unit 3/3,
native/compiler 592/592, and COW integration 13/13, isolated bounded PHPT
269/269 plus 2 unsupported-ini exclusions, and PHPT COW 29/29.

Follow-ups remain visibility/exception/reference/global edges, typed/promoted
properties, interfaces/traits, magic methods, first-class callables, dynamic
includes, unsupported internals, scalar offset-lvalues, assertions,
binary-safe array keys, embedded-NUL internals, object IDs, host-locale parity,
`str_replace()` array forms, exact scalar conversion diagnostics for edge
values, and remaining object/reference targets including non-public,
nested/dynamic, and by-reference property lvalue edges.
