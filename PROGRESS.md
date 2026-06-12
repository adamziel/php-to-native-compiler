# PTN Progress

Refresh: 2026-06-13T04:35Z
Measured: `ptn-98d8.4` rebased on current `origin/master` `60d3576f3`.

Recent RC slices cover class constants, embedded-NUL `var_export()`,
`explode()`, `strncmp()`, `strrchr()`, namespaces/imports, includes/once
guards, closures, `stdClass`, properties/destructors/reflection, inherited
static dispatch, `property_exists()` metadata, array mutators/set/sort/udiff
helpers, `array_sum()`/`array_product()` warnings and overflow, `json_encode()`,
`printf()`/`sprintf()`, `basename()`, `pathinfo()`, `file_get_contents()`,
`strcasecmp()`, scalar `str_replace()` counts/TypeErrors, `chr()` diagnostics,
`crc32()`, standard streams, foreach list destructuring, dynamic-variable
writes/unsets and `??=`, stream metadata, and offset compound/coalescing.

Recent movers include PHP 8.4 `array_sum()`/`array_product()` unsupported-type
warnings and overflow promotion, persistent `STDIN`/`STDOUT`/`STDERR`,
binary-safe `pathinfo()` with `PATHINFO_*` flags, `property_exists()` metadata,
broad PHPT baseline manifests, length-aware scalar `crc32()` checksums, scalar
`str_replace()` count out-parameters, `chr()` diagnostics,
`strncmp()`/`strrchr()`, `basename()`, `file_get_contents()`, `strcasecmp()`,
streams, and PHPT preclassification.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 583 | 583 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 257 | 255 | 2 |
| PHPT Zend rows | 82 | 82 | 0 |
| PHPT ext/standard rows | 125 | 125 | 0 |
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
destructors, reflection, callability/countability, assertions,
namespaces/imports, streams and standard stream constants, file reads/writes,
array/string/numeric helpers through `array_udiff*()`, `array_sum()`,
`array_product()`, `json_encode()`, `printf()`, `fdiv()`, `explode()`,
`str_replace()`, `strcasecmp()`, `strncmp()`, `strrchr()`, `pathinfo()`,
`crc32()`, `basename()`, `chr()` diagnostics, `var_export()`, array mutators,
inc/dec, foreach destructuring, dynamic-variable writes/unsets, and
array/string-offset compound/null coalescing assignments.

## Remaining Bounded Failures

- None among the 255 runnable rows in the current 257-row bounded manifest.
  Two selected rows are classified out for unsupported ini requirements.

## Verification

2026-06-13T04:35Z: passed diff check, `cargo fmt`, focused native
sum/product 1/1, focused PHPT sum/product/key 15/15, `cargo test` 583/583
plus COW/doc tests, bounded PHPT 255/255 with 2 excluded, PHPT COW 29/29, and
post-merge COW 26/26.

Follow-ups remain visibility/exception/reference/global edges, typed/promoted
properties, interfaces/traits, bracketed/grouped namespaces, broader
fallback/reflection, magic methods, first-class callables, dynamic includes,
unsupported internals, scalar offset-lvalues, assertion config, binary-safe
array keys, append-form `??=`, embedded-NUL internals, object IDs,
locale constants/`setlocale()`, `str_replace()` array forms, and
object/reference targets.
