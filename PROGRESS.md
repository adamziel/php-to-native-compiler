# PTN Progress

Refresh: 2026-06-13T01:54Z
Measured: `ptn-98d8.1` rebased on current `origin/master` `3b520ef7c`.

Recent RC slices cover class constants, embedded-NUL `var_export()`,
`explode()`, `strncmp()`, `strrchr()`, namespaces/imports, includes/once
guards, closures, `stdClass`, declared/static properties, public destructors,
reflection, array mutators/set/sort/udiff/list helpers, `json_encode()`,
`printf()`/`sprintf()`, `basename()`, `file_get_contents()`, `strcasecmp()`,
scalar `str_replace()` count outputs and TypeErrors, `chr()` diagnostics,
metadata, foreach list destructuring, dynamic-variable writes/unsets and
`??=`, stream metadata, array/string-offset compound/null coalescing, locale
category constants, and bounded `setlocale()` state/query support.

Recent movers include broad PHPT baseline manifests, scalar `str_replace()`
count out-parameters, `chr()` integer diagnostics, `strncmp()`/`strrchr()`,
`basename()`, `file_get_contents()`, `strcasecmp()`, streams,
native-process locale state, and PHPT preclassification of unsupported rows.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 581 | 581 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 233 | 231 | 2 |
| PHPT Zend rows | 80 | 80 | 0 |
| PHPT ext/standard rows | 103 | 103 | 0 |
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
`json_encode()`, `printf()`, `fdiv()`, `explode()`, `str_replace()`,
`strcasecmp()`, `strncmp()`, `strrchr()`, `basename()`, `chr()` diagnostics,
`var_export()`, array mutators, inc/dec, foreach destructuring,
dynamic-variable writes/unsets, and array/string-offset compound/null
coalescing assignments.

## Remaining Bounded Failures

- None among the 231 runnable rows in the current 233-row bounded manifest.
  Two selected rows are classified out for unsupported ini requirements.

## Verification

Current slice verification: `git diff --check`; `cargo fmt --check`; `bash -n`
for PHPT scripts; `run-phpt-manifest.sh -` selected 1/runnable 1/passed 1 with
corpus revision telemetry; `run-phpt-baseline.sh --generate-only` emitted
1k/5k/10k manifests; tier-5 baseline smoke selected 5/runnable 4/excluded 1
and recorded 3 passed, 1 failed as broad measurement signal; focused locale
native reducer; full `cargo test` 581/581 plus auxiliary/doc tests.

Follow-ups remain visibility/exception/reference/global edges, typed/promoted
properties, interfaces/traits, bracketed/grouped namespaces, broader
fallback/reflection, magic methods, first-class callables, dynamic includes,
unsupported internals, scalar offset-lvalues, assertion config, binary-safe
array keys, append-form `??=`, embedded-NUL internals, object IDs,
`str_replace()` array forms, object/reference targets, and locale-sensitive
behavior beyond native-process `setlocale()` state.
