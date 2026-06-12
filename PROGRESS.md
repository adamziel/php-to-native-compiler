# PTN Progress

Refresh: 2026-06-13T01:46Z
Measured: `ptn-jzgh` rebased on current `origin/master` `52de5ad8a`.

Recent RC slices cover class constants, embedded-NUL `var_export()`,
`explode()`, `strncmp()`, `strrchr()`, namespaces/imports, includes/once
guards, returns, closures, `stdClass`, declared/static properties, public
destructors, reflection, quiet probes, array mutators/set/sort/udiff/list
helpers, `count()`/`sizeof()`, `json_encode()`, `printf()`/`sprintf()`,
`basename()`, `file_get_contents()`, `strcasecmp()`, scalar `str_replace()`
count outputs and invalid-operand TypeErrors, `chr()` diagnostics,
string-internal diagnostics, PHP metadata, `php_uname()`, foreach list
destructuring, dynamic-variable writes/unsets and `??=`, stream metadata, and
array/string-offset compound/null coalescing assignments.

Recent movers include scalar `str_replace()` count out-parameters plus
resource boundary TypeErrors, `chr()` integer diagnostics,
`strncmp()`/`strrchr()`, `basename()`, `file_get_contents()`, `strcasecmp()`,
streams, and PHPT manifest preclassification of unsupported extension,
unsupported ini/runtime, SAPI/request, external-service, and intentional
non-goal rows.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 580 | 580 | 0 |
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

Current slice verification: `git diff --check`; `cargo fmt --check`;
focused `str_replace()` native regression 1/1; focused
`str_replace_basic.phpt` selected 1/runnable 1/passed 1; full `cargo test`
580/580 plus auxiliary/doc tests; bounded PHPT selected 233/runnable 231/
excluded 2/passed 231; PHPT COW selected 29/runnable 29/passed 29.

Follow-ups remain visibility/exception/reference/global edges, typed/promoted
properties, interfaces/traits, bracketed/grouped namespaces, broader
fallback/reflection, magic methods, first-class callables, dynamic includes,
unsupported internals, scalar offset-lvalues, assertion config, binary-safe
array keys, append-form `??=`, embedded-NUL internals, object IDs,
locale constants/`setlocale()`, `str_replace()` array forms, and
object/reference targets.
