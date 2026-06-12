# PTN Progress

Refresh: 2026-06-13T00:59Z
Measured: `ptn-98d8.8` rebased on current `origin/master` `1bad86067`.

Recent RC slices cover class constants, embedded-NUL `var_export()`,
`explode()`, `strncmp()`, `strrchr()`, namespaces/imports, includes/once
guards, returns, closures, `stdClass`, declared/static properties, public
destructors, reflection, quiet probes, array mutators/set/sort/udiff/list
helpers, `count()`/`sizeof()`, `json_encode()`, `printf()`/`sprintf()`,
`basename()`, `file_get_contents()`, `strcasecmp()`, `chr()` diagnostics,
string-internal diagnostics, PHP metadata, `php_uname()`, foreach list
destructuring, dynamic-variable writes/unsets and `??=`, stream metadata, and
array/string-offset compound/null coalescing assignments.

Recent movers include `chr()` integer diagnostics, `strncmp()`/`strrchr()`,
`basename()`, `file_get_contents()`, `strcasecmp()`, streams, and dynamic-root
`??=` reducers. PHPT manifest runners now preclassify broad rows into runnable
versus unsupported extension, unsupported ini/runtime, SAPI/request,
external-service, and intentional non-goal exclusions so broad campaign
failures remain PTN semantic signal rather than environment noise.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 579 | 579 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 232 | 230 | 2 |
| PHPT Zend rows | 80 | 80 | 0 |
| PHPT ext/standard rows | 102 | 102 | 0 |
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
`json_encode()`, `printf()`, `fdiv()`, `explode()`, `strcasecmp()`,
`strncmp()`, `strrchr()`, `basename()`, `chr()` diagnostics, `var_export()`,
array mutators, inc/dec, foreach destructuring, dynamic-variable writes/unsets,
and array/string-offset compound/null coalescing assignments.

## Remaining Bounded Failures

- None among the 230 runnable rows in the current 232-row bounded manifest.
  Two selected rows are classified out for unsupported ini requirements.

## Verification

Current slice verification: `git diff --check`; `cargo fmt --check`;
focused `chr` 7/7 and `intdiv` 4/4; full `cargo test` 579/579 plus
auxiliary/doc tests; bounded PHPT selected 232, runnable 230, excluded 2, and
passed 230/230; PHPT COW 29/29; post-merge COW 17/17 oracle, 3/3 notice, 6/6
diagnostics. PHPT classification smokes:
`run-phpt-manifest.sh -` selected 3, runnable 1, excluded 2 and passed the
runnable `strlen` row; `run-bounded-phpt.sh` with the same synthetic bucket
selected 3, runnable 1, excluded 2 and passed the runnable row.

Follow-ups remain visibility/exception/reference/global edges, typed/promoted
properties, interfaces/traits, bracketed/grouped namespaces, broader
fallback/reflection, magic methods, first-class callables, dynamic includes,
unsupported internals, scalar offset-lvalues, assertion config, binary-safe
array keys, append-form `??=`, embedded-NUL internals, object IDs,
locale constants/`setlocale()`, and object/reference targets.
