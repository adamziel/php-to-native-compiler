# PTN Progress

Refresh: 2026-06-13T02:23Z
Measured: `ptn-88qe` rebased on current `origin/master` `bb6eef149`.

Recent RC slices cover class constants, embedded-NUL `var_export()`,
`explode()`, `strncmp()`, `strrchr()`, namespaces/imports, includes/once
guards, closures, `stdClass`, declared/static properties, public destructors,
reflection, array mutators/set/sort/udiff/list helpers, `json_encode()`,
`printf()`/`sprintf()`, `basename()`, `dirname()` levels,
`file_get_contents()`, `strcasecmp()`,
scalar `str_replace()` count outputs and TypeErrors, `chr()` diagnostics,
metadata, foreach list destructuring, dynamic-variable writes/unsets and
`??=`, stream metadata, and array/string-offset compound/null coalescing.

Recent movers include broad PHPT baseline manifests, scalar `str_replace()`
count out-parameters, `chr()` integer diagnostics, `strncmp()`/`strrchr()`,
`basename()`, `dirname()` levels, `file_get_contents()`, `strcasecmp()`,
streams, and PHPT preclassification of unsupported rows.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 581 | 581 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 234 | 232 | 2 |
| PHPT Zend rows | 80 | 80 | 0 |
| PHPT ext/standard rows | 104 | 104 | 0 |
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
class/object shells/constants, declared/static properties, public destructors,
reflection, callability/countability, assertions, namespaces/imports, streams,
file reads/writes, array/string/numeric helpers through `array_udiff*()`,
`json_encode()`, `printf()`, `fdiv()`, `explode()`, `dirname()` levels,
`str_replace()`,
`strcasecmp()`, `strncmp()`, `strrchr()`, `basename()`, `chr()` diagnostics,
`var_export()`, array mutators, inc/dec, foreach destructuring,
dynamic-variable writes/unsets, and array/string-offset compound/null
coalescing assignments.

## Remaining Bounded Failures

- None among the 232 runnable rows in the current 234-row bounded manifest.
  Two selected rows are classified out for unsupported ini requirements.

## Verification

Current slice verification: `cargo fmt --check`; `git diff --check`; `bash -n`
for PHPT scripts; `cargo test compile_dirname_levels_phpt_shape_to_native_binary`;
`run-phpt-manifest.sh -` for `ext/standard/tests/strings/dirname_multi.phpt`
selected 1/runnable 1/passed 1 with corpus revision telemetry; full bounded
check before adding `dirname_multi.phpt` to the manifest selected 233/runnable
231/excluded 2 and passed 231; COW manifest selected 29/runnable 29/passed 29.

Follow-ups remain visibility/exception/reference/global edges, typed/promoted
properties, interfaces/traits, bracketed/grouped namespaces, broader
fallback/reflection, magic methods, first-class callables, dynamic includes,
unsupported internals, scalar offset-lvalues, assertion config, binary-safe
array keys, append-form `??=`, embedded-NUL internals, object IDs,
locale constants/`setlocale()`, `str_replace()` array forms, and
object/reference targets.
