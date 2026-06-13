# PTN Progress

Refresh: 2026-06-13T09:39Z
Measured: `ptn-qzfa` atop current `origin/master` `ea501c3b5`;
focused `str_replace()` verification green.

Recent RC slices cover constants, embedded-NUL `var_export()`, includes/once
guards, closures, `stdClass`, properties/destructors, inherited static
dispatch, `property_exists()` metadata, array helpers, `json_encode()`,
`printf()`, `basename()`, `pathinfo()`, `strcasecmp()`, search/count
internals, scalar and ordered-array `str_replace()`, `chr()` diagnostics,
`crc32()`, standard streams, foreach destructuring, dynamic-variable
writes/unsets, stream metadata, keyword boolean tails after direct
assignments, locale constants/`setlocale()`, catchable divide/modulo/shift
operator errors, alternate `<>` not-equal parsing, offset compound/coalescing,
and non-finite float TypeErrors through shared integer-internal validation.

Recent movers include binary-safe search/count internals, PHP 8.4 array
warning/overflow behavior, persistent standard streams, `pathinfo()`,
`property_exists()`, PHPT manifests, modeled `LC_*` constants, C/POSIX
`setlocale()`, catchable operator exceptions, tests/lang operator rows,
`crc32()`, `str_replace()` counts and ordered-array operands, stream
preclassification, and integer internal argument validation for `chr()`,
`intdiv()`, and file offsets.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 589 | 589 | 0 |
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
selected internals, COW/reference slices, user functions, scalar plus `void`
return hints, closures, `stdClass`, class/object shells/constants, declared and
static properties, `property_exists()` metadata, inherited static dispatch,
public destructors, reflection, assertions, namespaces/imports, streams,
file reads/writes, array/string/numeric helpers through `array_udiff*()`,
`array_sum()`, `array_product()`, `json_encode()`, `printf()`, `fdiv()`,
`explode()`, scalar/array `str_replace()`, `strcasecmp()`, `strncmp()`,
`strrchr()`,
string search/slice/count internals, `pathinfo()`, `crc32()`, `basename()`,
`chr()` diagnostics, locale constants and `setlocale()`, `var_export()`,
array mutators, inc/dec, foreach destructuring, dynamic-variable writes/unsets,
catchable operator arithmetic exceptions, alternate not-equal parsing, keyword
boolean tails, and array/string-offset compound/null coalescing assignments.

## Remaining Bounded Failures

- None among the 268 runnable rows in the current 270-row bounded manifest.
  Two selected rows are classified out for unsupported ini requirements.

## Verification

Current branch verification for `ptn-qzfa`: `git diff --check`,
`cargo fmt --check`, `cargo test` native/compiler 589/589 plus ancillary COW
suites, focused PHPT `ext/standard/tests/strings/str_replace_array_refs.phpt`
1/1, bounded PHPT 268/268 with 2 classified exclusions, and baseline
post-merge COW gate 26/26.

Follow-ups remain visibility/exception/reference/global edges,
typed/promoted properties, interfaces/traits, namespaces, fallback/reflection,
magic methods, first-class callables, dynamic includes, unsupported internals,
scalar offset-lvalues, assertions, binary-safe array keys, append-form `??=`,
embedded-NUL internals, object IDs, host-locale parity, exact `str_replace()`
nested/object/reference edges, and object/reference targets.
