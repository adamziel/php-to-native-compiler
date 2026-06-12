# PTN Progress

Refresh: 2026-06-13T03:09Z
Measured: `ptn-98d8.5` rebased on current `origin/master` `d21b77f37`.

Recent RC slices cover class constants, embedded-NUL `var_export()`,
`explode()`, `strncmp()`, `strrchr()`, namespaces/imports, includes/once
guards, closures, `stdClass`, declared/static properties, public destructors,
reflection, inherited static method dispatch, `property_exists()` metadata,
array mutators/set/sort/udiff/list helpers, `json_encode()`,
`printf()`/`sprintf()`, `basename()`, `pathinfo()`, `file_get_contents()`,
`strcasecmp()`, scalar `str_replace()` count outputs and TypeErrors, `chr()`
diagnostics, length-aware `crc32()`, metadata, foreach list destructuring,
dynamic-variable writes/unsets and `??=`, stream metadata, and
array/string-offset compound/null coalescing.

Recent movers include binary-safe `pathinfo()` component extraction with
`PATHINFO_*` flags, declared/static `property_exists()` metadata, inherited
static method dispatch, broad PHPT baseline manifests, length-aware scalar
`crc32()` checksums, scalar `str_replace()` count out-parameters, `chr()`
integer diagnostics, `strncmp()`/`strrchr()`, `basename()`,
`file_get_contents()`, `strcasecmp()`, streams, and PHPT preclassification.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 582 | 582 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 242 | 240 | 2 |
| PHPT Zend rows | 82 | 82 | 0 |
| PHPT ext/standard rows | 110 | 110 | 0 |
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
namespaces/imports, streams, file reads/writes, array/string/numeric helpers
through `array_udiff*()`, `json_encode()`, `printf()`, `fdiv()`, `explode()`,
`str_replace()`, `strcasecmp()`, `strncmp()`, `strrchr()`, `pathinfo()`,
`crc32()`, `basename()`, `chr()` diagnostics, `var_export()`, array mutators,
inc/dec, foreach destructuring, dynamic-variable writes/unsets, and
array/string-offset compound/null coalescing assignments.

## Remaining Bounded Failures

- None among the 240 runnable rows in the current 242-row bounded manifest.
  Two selected rows are classified out for unsupported ini requirements.

## Verification

2026-06-13T03:42Z: passed diff check, `cargo fmt`, focused native
`pathinfo()` 1/1, focused PHPT `pathinfo` 5/5, `compile_native` 582/582,
auxiliary COW tests 13/13, bounded PHPT 240/240 with 2 excluded, PHPT COW
29/29, and post-merge COW 26/26.

Follow-ups remain visibility/exception/reference/global edges, typed/promoted
properties, interfaces/traits, bracketed/grouped namespaces, broader
fallback/reflection, magic methods, first-class callables, dynamic includes,
unsupported internals, scalar offset-lvalues, assertion config, binary-safe
array keys, append-form `??=`, embedded-NUL internals, object IDs,
locale constants/`setlocale()`, `str_replace()` array forms, and
object/reference targets.
