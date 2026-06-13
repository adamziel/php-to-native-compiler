# PTN Progress

Refresh: 2026-06-13T10:30Z
Measured: `ptn-cijm` rebased on current `origin/master` `527a6eb99`;
verification green.

Recent RC slices cover constants, embedded-NUL `var_export()`, includes/once
guards, closures, `stdClass`, properties/destructors, inherited static
dispatch, `property_exists()` metadata, array helpers, `json_encode()`,
`printf()`, `basename()`, `pathinfo()`, `strcasecmp()`, search/count
internals, scalar `str_replace()`, `chr()` diagnostics, `crc32()`, standard
streams, foreach destructuring, dynamic-variable writes/unsets, stream
metadata, keyword boolean tails after direct assignments, locale constants and
`setlocale()` current/C/POSIX queries including `null`, catchable
divide/modulo/shift operator errors, alternate `<>` not-equal parsing, offset
compound/coalescing, and non-finite float TypeErrors through shared
integer-internal validation, plus modeled `display_errors` and
`zend.assertions` ini keys.

Recent movers include search/count internals, PHP 8.4 array warning/overflow
behavior, persistent standard streams, `pathinfo()`,
`property_exists()`, PHPT manifests, modeled `LC_*` constants, C/POSIX
`setlocale()`, catchable operator exceptions, tests/lang operator rows,
`crc32()`, `str_replace()` counts, and integer validation for `chr()`,
`intdiv()`, and file offsets, plus unsupported-ini PHPT promotion for
`display_errors` and `zend.assertions`.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 590 | 590 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 270 | 270 | 0 |
| PHPT Zend rows | 89 | 89 | 0 |
| PHPT ext/standard rows | 131 | 131 | 0 |
| PHPT focused stream rows | 2 | 2 | 0 |
| PHPT tests/basic+func+lang | 47 | 47 | 0 |
| PHPT other rows | 5 | 5 | 0 |
| PHPT COW manifest | 29 | 29 | 0 |
| Post-merge COW gate | 26 | 26 | 0 |
| PHPT callback manifest | 4 | 4 | 0 |
| PHPT include manifest | 2 | 2 | 0 |

## RC Surface

Parser/IR/C backend, boxed values, variables/constants, strings, scalar
operators, ternary, arrays, `foreach`, control flow, includes/once guards,
selected internals, COW/reference slices, user functions, scalar plus `void`
return hints, closures, `stdClass`, class/object shells/constants, declared and
static properties, `property_exists()` metadata, inherited static dispatch,
public destructors, reflection, assertions, namespaces/imports, streams, file
reads/writes, array/string/numeric helpers through `array_udiff*()`,
`array_sum()`, `array_product()`, `json_encode()`, `printf()`, `fdiv()`,
`explode()`, `str_replace()`, `strcasecmp()`, `strncmp()`, `strrchr()`, string
search/slice/count internals, `pathinfo()`, `crc32()`, `basename()`, `chr()`
diagnostics, locale constants and `setlocale()`, `var_export()`, array
mutators, inc/dec, foreach destructuring, dynamic-variable writes/unsets,
catchable operator arithmetic exceptions, alternate not-equal parsing, keyword
boolean tails, array/string-offset compound/null coalescing assignments, and
the current `display_errors`/`zend.assertions` ini surface.

## Remaining Bounded Failures

- None among the current 270-row bounded manifest.

## Verification

Current branch verification for `ptn-cijm`: `cargo fmt --check`, focused ini
native tests 2/2, direct former unsupported-ini PHPT rows 2/2, `cargo test`
with `compile_native` 590/590 plus source/COW/doc tests, bounded PHPT 270/270
with 0 classified exclusions, PHPT COW 29/29, and prior post-merge COW 26/26.

Follow-ups remain visibility/exception/reference/global edges, typed/promoted
properties, interfaces/traits, namespaces, fallback/reflection, magic methods,
first-class callables, dynamic includes, unsupported internals, scalar
offset-lvalues, assertions, binary-safe array keys, append-form `??=`,
embedded-NUL internals, object IDs, host-locale parity, `str_replace()` array
forms, and object/reference targets.
