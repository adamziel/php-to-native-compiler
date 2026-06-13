# PTN Progress

Refresh: 2026-06-13T04:08Z
Measured: `ptn-2n8w` rebased on current `origin/master` `60d3576f3` plus
dirname levels support.

Recent RC slices cover class constants, embedded-NUL `var_export()`,
`explode()`, `strncmp()`, `strrchr()`, namespaces/imports, includes/once
guards, returns, closures, `stdClass`, declared/static properties, public
destructors, reflection, inherited static method dispatch, quiet probes,
`property_exists()` metadata, array mutators/set/sort/udiff/list helpers,
`count()`/`sizeof()`, `json_encode()`, `printf()`/`sprintf()`, `basename()`,
`dirname()` levels, `pathinfo()`, `file_get_contents()`, `strcasecmp()`,
scalar `str_replace()` count outputs and TypeErrors, `chr()` diagnostics,
length-aware `crc32()`, string-internal diagnostics, PHP metadata,
`php_uname()`, foreach list destructuring, dynamic-variable writes/unsets and
`??=`, stream metadata, and array/string-offset compound/null coalescing
assignments.

Recent movers include binary-safe `pathinfo()` component extraction with
`PATHINFO_*` flags, declared/static `property_exists()` metadata, inherited
static method dispatch, broad PHPT baseline manifests, length-aware scalar
`crc32()` checksums, scalar `str_replace()` count out-parameters,
`dirname()` multi-level path reduction, `chr()` integer diagnostics,
`strncmp()`/`strrchr()`, `basename()`, `file_get_contents()`, `strcasecmp()`,
streams, and PHPT preclassification.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 583 | 583 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 243 | 241 | 2 |
| PHPT Zend rows | 82 | 82 | 0 |
| PHPT ext/standard rows | 111 | 111 | 0 |
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
`str_replace()`, `strcasecmp()`, `strncmp()`, `strrchr()`, `crc32()`,
`dirname()` levels, `pathinfo()`, `basename()`, `chr()` diagnostics,
`var_export()`, array mutators, inc/dec, foreach destructuring,
dynamic-variable writes/unsets, and array/string-offset compound/null
coalescing assignments.

## Remaining Bounded Failures

- None among the 241 runnable rows in the current 243-row bounded manifest.
  Two selected rows are classified out for unsupported ini requirements.

## Verification

Current final-rebase verification: `git diff --check`; `cargo fmt --check`;
focused native dirname 2/2, `property_exists()` 1/1, string-internal 2/2,
and `pathinfo()` 1/1; focused PHPT property/pathinfo/crc32/dirname 11/11.
Filtered native runs confirm 583 compile-native tests on the merged tree.
Before the pathinfo rebase, full `cargo test` passed 582/582 plus
auxiliary/doc and COW integration tests, bounded PHPT passed selected 238,
runnable 236, excluded 2, passed 236/236, and PHPT COW passed 29/29. The
rebased `pathinfo()` base records bounded PHPT 240/240 with 2 excluded, PHPT
COW 29/29, and post-merge COW 26/26.

Follow-ups remain visibility/exception/reference/global edges, typed/promoted
properties, interfaces/traits, bracketed/grouped namespaces, broader
fallback/reflection, magic methods, first-class callables, dynamic includes,
unsupported internals, scalar offset-lvalues, assertion config, binary-safe
array keys, append-form `??=`, embedded-NUL internals, object IDs,
locale constants/`setlocale()`, `str_replace()` array forms, and
object/reference targets.
