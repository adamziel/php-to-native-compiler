# PTN Progress

Refresh: 2026-06-12T16:06Z
Measured: `ptn-tca0` rebased on current `origin/master`.

Recent RC slices cover public class constants, embedded-NUL `var_export()`,
namespaced internal fallback, static-property `??=`, include-once guards,
property/static inc/dec, dynamic-variable writes/unsets and array/string-offset
compound assignments, array/append compound assignments, private properties,
object `var_export()`, `get_class()`, quiet probes, array mutators, sort flags,
set operations, `array_udiff*()`, string/lang rows, highlight paths,
`join()`/`implode()`, bounded `sprintf()`/`printf()`, bounded `json_encode()`,
`array_product()`, key helpers, `array_is_list()`, `array_search()`,
`array_slice()`, `array_pad()`, `count()`/`sizeof()` modes, `str_pad()`,
catchable `intdiv()`, `chunk_split()`, `abs()`/`sqrt()`/`fdiv()` TypeErrors,
ASCII case/trim, PHP/CLI/Zend metadata, PHPT runner probes,
`ceil()`/`floor()`, `chr()` diagnostics, `is_countable()`,
`ReflectionFunction`, namespaces, foreach list destructuring, bounded dynamic
include/require dispatch, and return-only `void` declarations.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 568 | 568 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 219 | 219 | 0 |
| PHPT Zend rows | 80 | 80 | 0 |
| PHPT ext/standard rows | 92 | 92 | 0 |
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
introspection, scalar plus `void` return type hints, closures, `stdClass`,
public class/object shells/constants, declared/static properties, quiet
probes, reflection, callability/countability, assertions, namespaces/imports,
streams, metadata, array/string/numeric helpers through `array_udiff*()`,
`array_is_list()`, `count()`/`sizeof()`, `json_encode()`, `printf()`,
`chunk_split()`, `fdiv()`, highlight paths, `var_export()`, array mutators,
inc/dec, foreach destructuring, dynamic-variable writes/unsets and
array/string-offset compound assignments.

## Remaining Bounded Failures

- None in the current 219-row bounded manifest.

## Verification

Current slice verification: `cargo fmt --check`; `cargo test void`;
`cargo test compile_array_is_list_to_native_binary`;
`cargo test compile_json_encode_and_printf_to_native_binary`;
focused `ext/standard/tests/general_functions/array_is_list.phpt` 1/1;
`tools/run-phpt-manifest.sh tools/phpt-bounded-manifest.txt` 219/219; and
post-merge COW gate 17/17 oracle, 3/3 notice, 6/6 diagnostics.

Follow-ups remain visibility/inheritance metadata, typed/promoted properties,
interfaces/traits, bracketed/grouped namespaces, broader fallback/reflection,
magic methods, first-class callables, destructors, dynamic includes,
unsupported internals, scalar offset-lvalues, assertion config, binary-safe
array keys, class-constant edges, dynamic-variable null-coalescing/by-reference
lvalues, remaining embedded-NUL internals, inc/dec Unicode/reference/COW
diagnostics, object IDs, broader `chr()`/`abs()` edges, `sqrt()` non-finite
edges, and object/reference targets.
