# PTN Progress

Refresh: 2026-06-13T12:12Z
Measured: `ptn-je09` integration on current `origin/master` `710be66`;
focused statement-form void-cast verification passed.

Recent RC slices cover constants, inline HTML, includes/once guards,
closures, `stdClass`, properties/destructors, reflection, array helpers,
`json_encode()`, `printf()`, `basename()`, `pathinfo()`, `dirname()` levels,
search/count/string internals, scalar `str_replace()`, `chr()` diagnostics,
`crc32()`, scalar conversion internals, PHP metadata constants, standard
streams, foreach destructuring, namespace/import forms, `global` bindings,
dynamic-variable writes/unsets, stream metadata, locale support, type
predicates including `is_iterable()`, PHPT runner ini values, statement-form
`(void)` casts, and offset compound/coalescing.

Recent movers include statement-form `(void)` expression discard,
`display_errors`/`zend.assertions` runner ini plumbing, bracketed namespace
blocks, `is_iterable()` for arrays in the current non-`Traversable` subset,
grouped namespace imports, scalar conversions, `global`, PHP metadata
constants, `dirname()` levels, `pathinfo()`, `LC_*`/`setlocale()`, search/count
internals, persistent standard streams, `property_exists()`, PHPT manifests,
`crc32()`, `str_replace()` counts, and integer validation for `chr()`,
`intdiv()`, and file offsets.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 618 | 618 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 272 | 272 | 0 |
| PHPT Zend rows | 86 | 86 | 0 |
| PHPT ext/standard rows | 131 | 131 | 0 |
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
internals, COW/reference slices, functions, closures, `stdClass`, class/object
shells/constants, properties, destructors, reflection, assertions,
namespace/import forms, streams, file reads/writes, array/string/numeric
helpers through `array_udiff*()`, `json_encode()`, `printf()`, `fdiv()`,
`explode()`, string/path/search helpers, scalar conversions, `is_iterable()`,
inline HTML, locale support, PHP metadata constants, PHPT runner ini values,
statement-form `(void)`, array mutators, inc/dec, `global`, dynamic variables,
and offset compound/null coalescing.

## Remaining Bounded Exclusions

- None among the 272 runnable rows in the current bounded manifest.
- Callback manifest has 3 runnable passing rows and 2 unsupported-extension
  exclusions.

## Verification

Current branch verification for `ptn-je09`: diff, format, focused void-cast
tests 4/4, and focused PHPT row 1/1 passed.

Follow-ups remain visibility/exception/reference/global edges, typed/promoted
properties, interfaces/traits, magic methods, first-class callables, dynamic
includes, unsupported internals, scalar offset-lvalues, assertions,
binary-safe array keys, `Traversable` objects, embedded-NUL internals, object
IDs, host-locale parity, `str_replace()` array forms, exact scalar conversion
diagnostics, and object/reference targets.
