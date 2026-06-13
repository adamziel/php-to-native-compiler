# PTN Progress

Refresh: 2026-06-13T08:10Z
Measured: `ptn-bf3f` rebased on current `origin/master` `217ccada8`.

Recent RC slices cover constants, embedded-NUL `var_export()`, `explode()`,
`strncmp()`, `strrchr()`, namespaces/imports, includes/once guards, closures,
`stdClass`, properties/destructors, inherited static dispatch,
`property_exists()` metadata, array helpers, `json_encode()`, `printf()`,
`basename()`, `pathinfo()`, `strcasecmp()`, string search/slice/count
internals, bounded `str_replace()` array forms, `chr()` diagnostics, `crc32()`,
standard streams, foreach destructuring, dynamic-variable writes/unsets, stream
metadata, keyword boolean tails after direct assignments, locale
constants/`setlocale()`, catchable divide/modulo/shift operator errors,
alternate `<>` not-equal parsing, and offset compound/coalescing.

Recent movers include binary-safe search/count internals, PHP 8.4 array
warning/overflow behavior, persistent standard streams, `pathinfo()`,
`property_exists()`, PHPT manifests, keyword boolean tails, modeled `LC_*`
constants with C/POSIX `setlocale()`, catchable operator exceptions,
tests/lang 64-bit operator rows, `crc32()`, scalar `str_replace()` counts,
bounded `str_replace()` array search/replacement/subject forms, and stream
preclassification.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 588 | 588 | 0 |
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
`strncmp()`, `strrchr()`, string search/slice/count internals, `pathinfo()`,
`crc32()`, `basename()`, `chr()` diagnostics, locale constants and
`setlocale()`, `var_export()`, array mutators, inc/dec, foreach destructuring,
dynamic-variable writes/unsets, catchable operator arithmetic exceptions,
alternate not-equal parsing, direct assignment statement keyword boolean tails,
and array/string-offset compound/null coalescing assignments.

## Remaining Bounded Failures

- None among the 268 runnable rows in the current 270-row bounded manifest.
  Two selected rows are classified out for unsupported ini requirements.

## Verification

2026-06-13T08:10Z: passed `cargo fmt --check`, focused native
`str_replace()` tests 2/2, focused PHPT
`ext/standard/tests/strings/str_replace_basic.phpt` 1/1, and post-rebase
`cargo test --test compile_native` 588/588 on current base `217ccada8`.
Prior full post-rebase `cargo test` on `7bc467b8b` covered compile-native
586/586 plus COW/doc tests. Earlier frontier evidence: bounded PHPT 260/260
with 2 unsupported-ini exclusions, PHPT COW 29/29, and post-merge COW 26/26.

Follow-ups remain visibility/exception/reference/global edges, typed/promoted
properties, interfaces/traits, namespaces, fallback/reflection, magic methods,
first-class callables, dynamic includes, unsupported internals, scalar
offset-lvalues, assertions, binary-safe array keys, append-form `??=`,
embedded-NUL internals, object IDs, host-locale parity beyond the bounded
C-locale slice, remaining `str_replace()` object/reference array-entry parity,
and object/reference targets.
