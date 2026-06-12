# PTN Progress

Refresh: 2026-06-12T16:53Z
Measured: `ptn-jh2q` rebased on current `origin/master` `2f5532b41`.

Recent RC slices cover public class constants, embedded-NUL `var_export()`,
namespaced internal fallback, static-property `??=`, include-once guards,
property/static inc/dec, dynamic-variable writes/unsets and array/string-offset
compound assignments, array/append compound assignments, private properties,
inherited parent-private property slots distinct from child public
redeclarations, public `__destruct()` lifecycle dispatch, object
`var_export()`, `get_class()`, quiet probes, array mutators, sort flags, set
operations, `array_udiff*()`, string/lang rows, highlight paths,
`join()`/`implode()`, bounded `sprintf()`/`printf()`, bounded `json_encode()`,
`array_product()`, key helpers, `array_is_list()`, `array_search()`,
`array_slice()`, `array_pad()`, `count()`/`sizeof()` modes, `str_pad()`,
catchable `intdiv()`, `chunk_split()`, `abs()`/`sqrt()`/`fdiv()` TypeErrors,
ASCII case/trim, PHP/CLI/Zend metadata, PHPT runner probes,
`ceil()`/`floor()`, `chr()` diagnostics, `is_countable()`,
`ReflectionFunction`, namespaces, foreach list destructuring, bounded dynamic
include/require dispatch, and return-only `void` declarations.

Recent movers include dynamic-root offset writes/unsets, property/static quiet
probes and inc/dec, direct sort-family mutators and flag diagnostics, array
key/search/pad helpers, catchable `intdiv()`, ASCII case and trim reducers,
`ceil()`/`floor()` `TypeError` parity, `is_countable()`, namespace PHPT rows
`ns_001`, `ns_002`, `ns_003`, and `ns_014`, `array_is_list()` PHPT coverage,
void return declarations, inherited private/public property redeclarations,
and destructor rows `destructor_and_echo` and `destructor_inheritance`.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 571 | 571 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 222 | 222 | 0 |
| PHPT Zend rows | 80 | 80 | 0 |
| PHPT ext/standard rows | 92 | 92 | 0 |
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
introspection, scalar plus `void` return type hints, closures, `stdClass`,
public class/object shells/constants, declared/static properties including
inherited parent-private slots, public destructor lifecycle dispatch, quiet
probes, reflection, callability/countability, assertions, namespaces/imports,
streams, metadata, array/string/numeric helpers through `array_udiff*()`,
`array_is_list()`, `count()`/`sizeof()`, `json_encode()`, `printf()`,
`chunk_split()`, `fdiv()`, highlight paths, `var_export()`, array mutators,
inc/dec, foreach destructuring, dynamic-variable writes/unsets and
array/string-offset compound assignments.

## Remaining Bounded Failures

- None in the current 222-row bounded manifest.

## Verification

Current slice verification: `cargo fmt --check`; `cargo build --bin phpc`;
destructor native reducers
`compile_declared_class_destructor_runs_at_shutdown_to_native_binary` and
`compile_inherited_class_destructor_runs_on_unset_to_native_binary`; focused
PHPT rows `tests/classes/destructor_and_echo.phpt` and
`tests/classes/destructor_inheritance.phpt` 2/2; bounded PHPT manifest
222/222; and post-merge COW gate 17/17 oracle, 3/3 notice, 6/6 diagnostics.

Follow-ups remain destructor visibility/exception/reference/global edges,
typed/promoted properties, interfaces/traits, bracketed/grouped namespaces,
broader fallback/reflection, magic methods, first-class callables, dynamic
includes, unsupported internals, scalar offset-lvalues, assertion config,
binary-safe array keys, class-constant edges, dynamic-variable
null-coalescing/by-reference lvalues, remaining embedded-NUL internals, inc/dec
Unicode/reference/COW diagnostics, object IDs, broader `chr()`/`abs()` edges,
`sqrt()` non-finite edges, and object/reference targets.
