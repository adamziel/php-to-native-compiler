# PTN Progress

Refresh: 2026-06-13T07:59Z
Measured: `ptn-yiif` rebased on current `origin/master` `217ccada8`.

Recent RC slices cover constants, embedded-NUL `var_export()`, `explode()`,
`strncmp()`, `strrchr()`, namespaces/imports, includes/once guards, closures,
`stdClass`, properties/destructors, inherited static dispatch,
`property_exists()` metadata, array helpers, `json_encode()`, `printf()`,
`basename()`, optional-level `dirname()`, `pathinfo()`,
`strcasecmp()`, string search/slice/count internals, scalar `str_replace()`,
`chr()` diagnostics, `crc32()`, standard streams, foreach destructuring,
dynamic-variable writes/unsets, stream metadata, keyword boolean
tails after direct assignments, locale constants/`setlocale()`, catchable
divide/modulo/shift operator errors, alternate `<>` not-equal parsing, and
offset compound/coalescing.

Recent movers include binary-safe search/count internals, PHP 8.4 array
warning/overflow behavior, persistent standard streams, optional-level
`dirname()`, `pathinfo()`,
`property_exists()`, PHPT manifests, keyword boolean tails, modeled `LC_*`
constants with C/POSIX `setlocale()`, catchable operator exceptions,
tests/lang 64-bit operator rows, `crc32()`, `str_replace()` counts, and stream
preclassification.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 587 | 587 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 270 | 268 | 2 |
| PHPT Zend rows | 88 | 88 | 0 |
| PHPT ext/standard rows | 130 | 130 | 0 |
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
selected internals, COW/reference slices, user functions, call-frame
introspection, scalar plus `void` return hints, closures, `stdClass`,
class/object shells/constants, declared/static properties,
`property_exists()` metadata, inherited static method dispatch, public
destructors, reflection, assertions, namespaces/imports, streams, file
reads/writes, array/string/numeric helpers through
`array_udiff*()`, `array_sum()`, `array_product()`, `json_encode()`,
`printf()`, `fdiv()`, `explode()`, `str_replace()`, `strcasecmp()`,
`strncmp()`, `strrchr()`, string search/slice/count internals,
optional-level `dirname()`, `pathinfo()`, `crc32()`, `basename()`,
`chr()` diagnostics, locale constants and
`setlocale()`, `var_export()`, array mutators, inc/dec, foreach destructuring,
dynamic-variable writes/unsets, catchable operator arithmetic exceptions,
alternate not-equal parsing, direct assignment statement keyword boolean tails,
and array/string-offset compound/null coalescing assignments.

## Remaining Bounded Failures

- None among the 268 runnable rows in the current 270-row bounded manifest.
  Two selected rows are classified out for unsupported ini requirements.

## Verification

Current slice `ptn-yiif` is green on `cargo fmt --check`, focused
`dirname()` native tests 2/2, and `cargo test` 587/587 plus COW/doc tests.
The pre-slice frontier check used to choose this reducer reran the bounded
manifest at 260/260 with 2 classified exclusions and PHPT COW at 29/29; the
current dashboard baseline remains bounded PHPT 268/268 with 2 classified
exclusions.

Follow-ups remain visibility/exception/reference/global edges, typed/promoted
properties, interfaces/traits, namespaces, fallback/reflection, magic methods,
first-class callables, dynamic includes,
unsupported internals, scalar offset-lvalues, assertions, binary-safe array
keys, append-form `??=`, embedded-NUL internals, object IDs,
host-locale parity beyond the bounded C-locale slice, `str_replace()` array
forms, and object/reference targets.
