# PTN Progress

Refresh: 2026-06-13T11:50Z
Measured: `ptn-ud8s` integration on current `origin/master` `b356fbf4f`;
focused `is_iterable()` verification green.

Recent RC slices cover constants, embedded-NUL `var_export()`, inline HTML
output, includes/once guards, closures, `stdClass`, properties/destructors,
inherited static dispatch, `property_exists()` metadata, array helpers,
`json_encode()`, `printf()`, `basename()`, `pathinfo()`, `dirname()` levels,
`strcasecmp()`, search/count internals, scalar `str_replace()`, `chr()`
diagnostics, `crc32()`, `boolval()`/`floatval()`/`doubleval()`, PHP
version/build/platform metadata constants, standard streams, foreach
destructuring, unbracketed namespaces with simple/grouped imports, `global`
bindings, dynamic-variable writes/unsets, stream metadata, locale constants and
`setlocale()`, catchable arithmetic/operator errors, type predicates including
`is_iterable()`, alternate `<>` parsing, and offset compound/coalescing.

Recent movers include `is_iterable()` for arrays in the current
non-`Traversable` object subset, grouped namespace imports, scalar conversion
internals for `boolval()`/`floatval()`/`doubleval()`, `global` function-local
binding to root globals, PHP version/build/platform metadata constants,
`dirname()` positive-level traversal, `pathinfo()`, modeled `LC_*` constants,
C/POSIX `setlocale()`, search/count internals, persistent standard streams,
`property_exists()`, PHPT manifests, `crc32()`, `str_replace()` counts, and
integer validation for `chr()`, `intdiv()`, and file offsets.

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
namespaces/imports including grouped use forms, streams, file reads/writes,
array/string/numeric helpers through `array_udiff*()`, `json_encode()`,
`printf()`, `fdiv()`, `explode()`, `str_replace()`, `strcasecmp()`,
`strncmp()`, `strrchr()`, `pathinfo()`, `dirname()` levels, `crc32()`,
`basename()`, `boolval()`/`floatval()`/`doubleval()`, `is_iterable()`,
inline HTML output, locale support, PHP version/build/platform metadata
constants, `var_export()`, array mutators, inc/dec, `global` bindings,
dynamic-variable writes/unsets, and offset compound/null coalescing
assignments.

## Remaining Bounded Exclusions

- None among the 269 runnable rows in the current 271-row bounded manifest.
  Two selected rows are classified out for unsupported ini requirements.
- Callback manifest has 3 runnable passing rows and 2 unsupported-extension
  exclusions.

## Verification

Current branch verification for `ptn-ud8s`: diff check, `cargo fmt`, focused
`is_iterable()` native test, and modeled-internal redeclaration parser test.

Follow-ups remain visibility/exception/reference/global edges, typed/promoted
properties, interfaces/traits, bracketed namespace blocks, magic methods,
first-class callables, dynamic includes, unsupported internals, scalar
offset-lvalues, assertions, binary-safe array keys, `Traversable` objects,
embedded-NUL internals, object IDs, host-locale parity, `str_replace()` array
forms, exact scalar conversion diagnostics for edge values, and
object/reference targets.
