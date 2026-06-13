# PTN Progress

Refresh: 2026-06-13T14:00Z
Measured: `ptn-2yg2` property/static-property compounds on `origin/master`
`42c352a9f`; focused parser/native gates passed.

Recent RC slices cover constants, inline HTML, includes/once guards,
closures, `stdClass`, properties/destructors, reflection, array helpers,
`json_encode()`, `printf()`, `basename()`, `pathinfo()`, `dirname()` levels,
search/count/string internals, scalar/array type hints, ordered-array
`str_replace()`, `str_split()`, `nl2br()`, `strncasecmp()`, `chr()`, `crc32()`, PHP
metadata constants, standard streams, foreach destructuring,
namespace/import forms, `global`, dynamic-variable writes/unsets, stream
metadata, locale constants plus `localeconv()`, predicates including
`is_iterable()`, invokable callables, SPL object identity intrinsics,
non-recursive `array_replace()`, `getcwd()`/`chdir()`, PHPT runner ini values,
statement-form `(void)` casts, offset compound/coalescing, and
property/static-property compounds.

Recent movers include `display_errors`/`zend.assertions` runner ini plumbing,
bracketed namespace blocks, `is_iterable()` for arrays in the current subset,
grouped imports, scalar conversions, `global`, PHP metadata constants,
`dirname()` levels, `pathinfo()`, `LC_*`/`setlocale()`/`localeconv()`,
invokable object callables, search/count internals, persistent standard
streams, `property_exists()`, PHPT manifests, `crc32()`, `str_replace()` array
operands/reference entries/counts, `str_split()`, `nl2br()`, `strncasecmp()`, array
parameter/return hints, object IDs, cwd helpers, property/static-property
compounds, and integer validation for `chr()`/`intdiv()`/file offsets.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 631 | 631 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 280 | 280 | 0 |
| PHPT Zend rows | 87 | 87 | 0 |
| PHPT ext/standard rows | 138 | 138 | 0 |
| PHPT focused stream rows | 2 | 2 | 0 |
| PHPT focused cwd rows | 2 | 2 | 0 |
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
inline HTML, locale support including `localeconv()`, invokable object
callables, scalar and ordered-array `str_replace()`, `str_split()`, `nl2br()`,
PHP metadata constants, `strncasecmp()`, scalar/array type hints, SPL object
identity intrinsics, non-recursive `array_replace()`, PHPT runner ini values,
`getcwd()`/`chdir()`, statement-form `(void)`, array mutators, inc/dec,
`global`, dynamic variables, offset compound/null coalescing, and direct
property/static-property compounds.

## Remaining Bounded Exclusions

- None among the 280 runnable rows in the current bounded manifest.
- Callback manifest has 3 runnable passing rows and 2 unsupported-extension
  exclusions.

## Verification

Current branch verification for `ptn-2yg2`: parser accepts
property/static-property compound assignments, and the native
property/static-property compound reducer passed.

Follow-ups remain typed properties, interfaces/traits, magic methods,
first-class callables, dynamic includes, unsupported internals,
scalar-offset lvalues, `Traversable`, embedded-NUL internals, host-locale
parity, conversion diagnostics, and callables.
