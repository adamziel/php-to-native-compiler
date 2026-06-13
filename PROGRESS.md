# PTN Progress

Refresh: 2026-06-13T02:42Z
Measured: `ptn-liv5` rebased on current `origin/master` `05b4670fd`.

Recent RC slices cover class constants, embedded-NUL `var_export()`,
`explode()`, `strncmp()`, `strrchr()`, namespaces/grouped imports,
includes/once guards, closures, `stdClass`, declared/static properties, public
destructors, reflection, array mutators/set/sort/udiff/list helpers,
`json_encode()`, `printf()`/`sprintf()`, `basename()`, `file_get_contents()`,
`strcasecmp()`, scalar `str_replace()` count outputs and TypeErrors, `chr()`
diagnostics, length-aware `crc32()`, metadata, foreach list destructuring,
dynamic-variable writes/unsets and `??=`, stream metadata, and
array/string-offset compound/null coalescing.

Recent movers include broad PHPT baseline manifests, grouped namespace imports,
length-aware scalar `crc32()` checksums, scalar `str_replace()` count
out-parameters, `chr()` integer diagnostics, `strncmp()`/`strrchr()`,
`basename()`, `file_get_contents()`, `strcasecmp()`, streams, and PHPT
preclassification.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 581 | 581 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 235 | 233 | 2 |
| PHPT Zend rows | 80 | 80 | 0 |
| PHPT ext/standard rows | 105 | 105 | 0 |
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
reflection, callability/countability, assertions, namespaces/grouped imports,
streams, file reads/writes, array/string/numeric helpers through `array_udiff*()`,
`json_encode()`, `printf()`, `fdiv()`, `explode()`, `str_replace()`,
`strcasecmp()`, `strncmp()`, `strrchr()`, `crc32()`, `basename()`, `chr()`
diagnostics, `var_export()`, array mutators, inc/dec, foreach destructuring,
dynamic-variable writes/unsets, and array/string-offset compound/null
coalescing assignments.

## Remaining Bounded Failures

- None among the 233 runnable rows in the current 235-row bounded manifest.
  Two selected rows are classified out for unsupported ini requirements.

## Verification

Current slice verification: `git diff --check`; `cargo fmt --check`;
focused parser/native string-internal tests 2/2; focused `crc32` PHPT rows
2/2; full `cargo test` 580/580 plus auxiliary COW/doc tests; bounded PHPT
235 selected/233 runnable/2 excluded, 233/233 passed; PHPT COW 29/29; and
post-merge COW gate 17/17 oracle, 3/3 notice, 6/6 diagnostics. Grouped-import
slice verification additionally ran `tools/run-phpt-manifest.sh
tools/phpt-bounded-manifest.txt` before the crc32 rebase, selected
233/runnable 231/excluded 2 and passed 231; `tools/run-phpt-manifest.sh
tools/phpt-cow-manifest.txt` selected 29/runnable 29 and passed 29;
`cargo build --bin phpc`; `cargo test namespace --test compile_native` passed
5 namespace reducers including grouped imports.

Follow-ups remain visibility/exception/reference/global edges, typed/promoted
properties, interfaces/traits, bracketed namespace blocks, broader
fallback/reflection, magic methods, first-class callables, dynamic includes,
unsupported internals, scalar offset-lvalues, assertion config, binary-safe
array keys, append-form `??=`, embedded-NUL internals, object IDs,
locale constants/`setlocale()`, `str_replace()` array forms, and
object/reference targets.
