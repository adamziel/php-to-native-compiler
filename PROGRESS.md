# PTN Progress

Refresh: 2026-06-13T07:23Z
Measured: `ptn-zta9` rebased on current `origin/master` `7bc467b8b`.

Recent RC slices cover constants, embedded-NUL `var_export()`, `explode()`,
`strncmp()`, `strrchr()`, namespaces/imports, includes/once guards, closures,
`stdClass`, properties/destructors/reflection, inherited static dispatch,
`property_exists()` metadata, array helpers, `json_encode()`,
`printf()`/`sprintf()`, `basename()`, `pathinfo()`, `file_get_contents()`,
`strcasecmp()`, string search/slice/count internals, `str_replace()` scalar
and top-level array forms,
`chr()` diagnostics, `crc32()`, standard streams, foreach destructuring,
dynamic-variable writes/unsets and `??=`, stream metadata, keyword boolean
tails after direct assignments, locale constants/`setlocale()`, and offset
compound/coalescing.

Recent movers include binary-safe search/count internals, PHP 8.4
`array_sum()`/`array_product()` warnings and overflow promotion, persistent
`STDIN`/`STDOUT`/`STDERR`, binary-safe `pathinfo()`, `property_exists()`,
PHPT manifests, keyword boolean assignment tails, modeled `LC_*` constants with
C/POSIX `setlocale()` dispatch, `crc32()`, scalar `str_replace()` counts,
top-level `str_replace()` array forms, and stream preclassification.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 585 | 585 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 262 | 260 | 2 |
| PHPT Zend rows | 82 | 82 | 0 |
| PHPT ext/standard rows | 130 | 130 | 0 |
| PHPT focused stream rows | 2 | 2 | 0 |
| PHPT tests/basic+func+lang | 45 | 45 | 0 |
| PHPT other rows | 5 | 5 | 0 |
| PHPT COW manifest | 29 | 29 | 0 |
| Post-merge COW gate | 26 | 26 | 0 |
| PHPT callback manifest | 4 | 4 | 0 |
| PHPT include manifest | 2 | 2 | 0 |

## RC Surface

Parser/IR/C backend, boxed values, variables/constants, strings, scalar
operators, ternary, ordered arrays, `foreach`, control flow, includes/once
guards, selected internals, COW/reference slices, user functions, call-frame
introspection, scalar plus `void` return hints, closures, `stdClass`,
class/object shells/constants, declared/static properties,
`property_exists()` metadata, inherited static method dispatch, public
destructors, reflection, assertions, namespaces/imports, streams and standard
stream constants, file reads/writes, array/string/numeric helpers through
`array_udiff*()`, `array_sum()`, `array_product()`, `json_encode()`,
`printf()`, `fdiv()`, `explode()`, `str_replace()` scalar/top-level array
forms, `strcasecmp()`,
`strncmp()`, `strrchr()`, string search/slice/count internals, `pathinfo()`,
`crc32()`, `basename()`, `chr()` diagnostics, locale constants and
`setlocale()`, `var_export()`, array mutators, inc/dec, foreach destructuring,
dynamic-variable writes/unsets, direct assignment statement keyword boolean
tails, and array/string-offset compound/null coalescing assignments.

## Remaining Bounded Failures

- None among the 260 runnable rows in the current 262-row bounded manifest.
  Two selected rows are classified out for unsupported ini requirements.

## Verification

2026-06-13T07:23Z: before the final rebase, passed `cargo fmt --check`,
focused `str_replace()` native reducers 2/2, `cargo test` 585/585 plus
COW/doc tests, bounded PHPT 260/260 with 2 excluded, PHPT COW 29/29, and
post-merge COW 26/26. After rebasing onto `origin/master` `7bc467b8b`, passed
`cargo fmt --check`, focused `str_replace()` native reducers 2/2, and
`str_replace_basic.phpt` 1/1.

Follow-ups remain visibility/exception/reference/global edges, typed/promoted
properties, interfaces/traits, bracketed/grouped namespaces, broader
fallback/reflection, magic methods, first-class callables, dynamic includes,
unsupported internals, scalar offset-lvalues, assertion config, binary-safe
array keys, append-form `??=`, embedded-NUL internals, object IDs,
host-locale parity beyond the bounded C-locale slice, nested `str_replace()`
diagnostics, and object/reference targets.
