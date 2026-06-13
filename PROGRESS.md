# PTN Progress

Refresh: 2026-06-13T13:06Z
Measured: `ptn-5fqu` integration on current `origin/master` `f55d262`;
focused `strncasecmp()` native/parser reducers and PHPT rows passed.

Recent RC slices cover constants, inline HTML, includes/once guards,
closures, `stdClass`, properties/destructors, reflection, array helpers,
`json_encode()`, `printf()`, `basename()`, `pathinfo()`, `dirname()` levels,
search/count/string internals, scalar and ordered-array `str_replace()`,
`str_split()` byte chunking, `strncasecmp()` byte-prefix comparison, `chr()`
diagnostics, `crc32()`, scalar conversion internals, PHP metadata constants,
standard streams, foreach destructuring, namespace/import forms, `global`
bindings, dynamic-variable writes/unsets, stream metadata, stable locale
constants plus `localeconv()`, type predicates including `is_iterable()`,
invokable object callables via
`__invoke`, PHPT runner ini values, statement-form `(void)` casts, and offset
compound/coalescing.

Recent movers include `display_errors`/`zend.assertions` runner ini plumbing,
bracketed namespace blocks, `is_iterable()` for arrays in the current subset,
grouped namespace imports, scalar conversions, `global`, PHP metadata
constants, `dirname()` levels, `pathinfo()`, `LC_*`/`setlocale()`/`localeconv()`,
including PHP-to-libc category mapping, invokable object callables,
search/count internals, persistent standard streams, `property_exists()`, PHPT
manifests, `crc32()`, `str_replace()` array operands/reference entries/counts,
`str_split()`, `strncasecmp()`, and integer validation for `chr()`, `intdiv()`,
and file offsets.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 622 | 622 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 276 | 276 | 0 |
| PHPT Zend rows | 86 | 86 | 0 |
| PHPT ext/standard rows | 135 | 135 | 0 |
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
inline HTML, locale support including `localeconv()`, invokable object
callables, scalar and ordered-array `str_replace()`, `str_split()`, PHP
metadata constants, `strncasecmp()`, PHPT runner ini values, statement-form
`(void)`, array mutators, inc/dec, `global`, dynamic variables, and offset
compound/null coalescing.

## Remaining Bounded Exclusions

- None among the 276 runnable rows in the current bounded manifest.
- Callback manifest has 3 runnable passing rows and 2 unsupported-extension
  exclusions.

## Verification

Current branch verification for `ptn-5fqu`:
`compile_string_internals_use_direct_string_operand_fast_paths_to_native_binary`,
`parser_rejects_user_function_redeclaring_modeled_internal`, focused PHPT
`strncasecmp_basic.phpt` and `strncasecmp_error.phpt` 2/2, plus 622-test
inventory.

Follow-ups remain visibility/exception/reference edges, typed properties,
interfaces/traits, magic methods, first-class callables, dynamic
includes, unsupported internals, scalar offset-lvalues, assertions,
binary-safe array keys, `Traversable` objects, embedded-NUL internals, object
IDs, host-locale parity, `str_replace()` object/diagnostic edges, exact scalar
conversion diagnostics, broad callable syntax, and object/reference targets.
