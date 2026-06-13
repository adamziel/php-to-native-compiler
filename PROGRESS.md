# PTN Progress

Refresh: 2026-06-13T05:42Z
Measured: `ptn-1mts` rebased on current `origin/master` `507a48131`.

Recent RC slices cover class constants, embedded-NUL `var_export()`,
`explode()`, `strncmp()`, `strrchr()`, namespaces/imports, includes/once
guards, closures, `stdClass`, properties/destructors/reflection, inherited
static dispatch, `property_exists()` metadata, array mutators/set/sort/udiff
helpers, `array_sum()`/`array_product()` warnings and overflow, `json_encode()`,
`printf()`/`sprintf()`, `basename()`, `pathinfo()`, `file_get_contents()`,
`strcasecmp()`, string search/slice/count internals, scalar `str_replace()`
counts/TypeErrors, `chr()` diagnostics, `crc32()`, standard streams, foreach
list destructuring, dynamic-variable writes/unsets and `??=`, stream metadata,
offset compound/coalescing, plus locale constants and libc-backed
`setlocale()` fallbacks.

Recent movers include binary-safe `strpos()`/`stripos()`,
`strrpos()`/`strripos()`, `strstr()`/`stristr()`, and `substr_count()` with PHP
offset bounds, PHP 8.4 `array_sum()`/`array_product()` unsupported-type
warnings and overflow promotion, persistent `STDIN`/`STDOUT`/`STDERR`,
binary-safe `pathinfo()` with `PATHINFO_*` flags, locale metadata and
`setlocale()` fallbacks, `property_exists()` metadata, broad PHPT manifests,
length-aware `crc32()`, scalar `str_replace()` counts,
`strncmp()`/`strrchr()`, `basename()`, and stream preclassification.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 584 | 584 | 0 |
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

## Broad PHPT Baseline

`tools/run-phpt-baseline.sh` generates deterministic 1k/5k/10k broad manifests
from `Zend/tests`, `ext/standard/tests`, and core `tests`, records the corpus
revision, and treats pass/fail/skip/warn totals as measurement signal.

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
`printf()`, `fdiv()`, `explode()`, `str_replace()`, `strcasecmp()`,
`strncmp()`, `strrchr()`, string search/slice/count internals, `pathinfo()`,
`crc32()`, `basename()`, `chr()` diagnostics, `var_export()`, array mutators,
inc/dec, foreach destructuring, dynamic-variable writes/unsets,
array/string-offset compound/null coalescing assignments, plus `LC_*` locale
constants and `setlocale()`.

## Remaining Bounded Failures

- None among the 260 runnable rows in the current 262-row bounded manifest.
  Two selected rows are classified out for unsupported ini requirements.

## Verification

2026-06-13T05:42Z: passed `git diff --check`, `cargo fmt --check`,
focused native locale 1/1, focused native string-internal 1/1, focused search
PHPT 5/5, `cargo test` 584/584 plus COW/doc tests, PHPT COW 29/29, and
post-merge COW 26/26. Bounded PHPT was rerun before the final string-search
rebase at 257 selected/255 runnable/2 excluded, 255/255 passed; the current
262 selected/260 runnable bounded count is from the upstream base and was not
rerun end-to-end in this slice.

Follow-ups remain visibility/exception/reference/global edges, typed/promoted
properties, interfaces/traits, bracketed/grouped namespaces, broader
fallback/reflection, magic methods, first-class callables, dynamic includes,
unsupported internals, scalar offset-lvalues, assertion config, binary-safe
array keys, append-form `??=`, embedded-NUL internals, object IDs,
`str_replace()` array forms, and object/reference targets.
