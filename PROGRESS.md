# PTN Progress

Refresh: 2026-06-12T15:58Z
Measured: `ptn-dnkw` rebased after `origin/master` `ba4345528`.

Recent RC slices cover public class constants, embedded-NUL `var_export()`,
namespaced internal fallback, static-property `??=`, include-once guards,
property/static inc/dec, dynamic-variable writes/unsets, array/append compound
assignments, private properties, object `var_export()`, `get_class()`, quiet
probes, array mutators, sort flags, set operations, `array_udiff*()`,
string/lang rows, highlight paths, `join()`/`implode()`, `sprintf()`,
`array_product()`, key helpers, `array_search()`, `array_slice()`,
`array_pad()`, `count()`/`sizeof()` modes, `str_pad()`, catchable `intdiv()`,
`chunk_split()`, and `abs()`/`sqrt()`/`fdiv()` TypeErrors, ASCII case/trim,
PHP/CLI/Zend metadata, PHPT runner probes, `ceil()`/`floor()`, `chr()`
diagnostics, `is_countable()`, `ReflectionFunction`, namespaces, foreach list
destructuring, and bounded dynamic include/require dispatch.

Recent movers include class-constant reducers, the `explode.phpt` embedded-NUL
`var_export()` md5 boundary, namespace fallback, once-include reducers,
static-property coalesce assignment, dynamic-root writes/unsets,
property/static probes, sort diagnostics, array helpers, count/sizeof modes,
`str_pad()` constants, numeric-internal operand diagnostics,
`chunk_split()` length diagnostics, ASCII case/trim, runner probes, namespace
and include PHPT rows, `str_pad`, `chunk_split_variation7`, `count_basic`,
`sizeof_basic2`, ReflectionFunction metadata, and fd-backed manifest tooling.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 562 | 562 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 218 | 218 | 0 |
| PHPT Zend rows | 80 | 80 | 0 |
| PHPT ext/standard rows | 91 | 91 | 0 |
| PHPT tests/basic+func+lang | 45 | 45 | 0 |
| PHPT other rows | 2 | 2 | 0 |
| PHPT COW manifest | 29 | 29 | 0 |
| Post-merge COW gate | 26 | 26 | 0 |
| PHPT callback manifest | 4 | 4 | 0 |
| PHPT include manifest | 2 | 2 | 0 |

## RC Surface

Parser/IR/C backend, boxed values, variables/constants, strings, scalar
operators, ternary, ordered arrays, `foreach`, control flow, includes/once
guards, selected internals, COW/reference slices, user functions, call-frame
introspection, type hints, closures, `stdClass`, public class/object
shells/constants, declared/static properties, quiet probes, reflection,
callability/countability, assertions, namespaces/imports, streams, metadata,
array/string/numeric helpers through `array_udiff*()`, `chunk_split()`,
`sizeof()`, and `fdiv()`, highlight paths, `var_export()`, array mutators,
inc/dec, foreach destructuring, and dynamic-variable writes.

## Remaining Bounded Failures

- None in the current 218-row bounded manifest.

## Verification

Verification: recent slices added class constants, embedded-NUL `var_export()`
coverage matching `explode.phpt` md5 evidence, registered-internal fallback,
once-includes, static-property `??=`, namespace PHPT rows 4/4, include PHPT
manifest 2/2, `str_pad.phpt`, `chunk_split_variation7.phpt`, focused
`intdiv()` TypeErrors, catchable `chunk_split()` length errors,
callback/reflection manifest 4/4, streamed-manifest smokes, runner probes,
`chr()` diagnostics, count/sizeof coverage, and focused
`abs()`/`sqrt()`/`fdiv()` operand coverage.

Follow-ups remain visibility/inheritance metadata, typed/promoted properties,
interfaces/traits, bracketed/grouped namespaces, broader fallback/reflection,
magic methods, first-class callables, destructors, dynamic includes,
unsupported internals, scalar offset-lvalues, assertion config, binary-safe
array keys, class-constant edges, remaining embedded-NUL internals, inc/dec
Unicode/reference/COW diagnostics, object IDs, broader `chr()`/`abs()` edges,
`sqrt()` non-finite edges, and object/reference targets.
