# PTN Progress

Refresh: 2026-06-12T17:19Z
Measured: `ptn-z31e` rebased on current `origin/master` `4a0de706c`.

Recent RC slices cover public class constants, embedded-NUL `var_export()`,
`explode()`, namespaced internal fallback, static-property `??=`, include-once
guards, property/static inc/dec, dynamic-variable writes/unsets,
array/string-offset compound assignments, private properties, inherited
parent-private property slots, public `__destruct()` lifecycle dispatch, quiet
probes, array mutators, sort flags, set operations, `array_udiff*()`,
`join()`/`implode()`, bounded `sprintf()`/`printf()`, bounded `json_encode()`,
`array_is_list()`, `array_search()`, `array_slice()`, `array_pad()`,
`count()`/`sizeof()`, `str_pad()`, `chunk_split()`,
`abs()`/`sqrt()`/`fdiv()` TypeErrors, ASCII case/trim, PHP/CLI/Zend metadata,
`ReflectionFunction`, namespaces, foreach list destructuring, dynamic
include/require dispatch, and return-only `void` declarations.

Recent movers include the bounded `explode()` reducer for binary separators,
positive/zero/negative limits, empty-separator `ValueError`, and parser
protection against redeclaring `explode()`, plus the prior inherited
private/public property and destructor rows.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 572 | 572 | 0 |
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
introspection, scalar plus `void` return hints, closures, `stdClass`,
class/object shells/constants, declared/static properties including inherited
parent-private slots, public destructor dispatch, reflection,
callability/countability, assertions, namespaces/imports, streams, metadata,
array/string/numeric helpers through `array_udiff*()`, `array_is_list()`,
`count()`/`sizeof()`, `json_encode()`, `printf()`, `chunk_split()`, `fdiv()`,
`explode()`, highlight paths, `var_export()`, array mutators, inc/dec, foreach
destructuring, dynamic-variable writes/unsets, and array/string-offset compound
assignments.

## Remaining Bounded Failures

- None in the current 222-row bounded manifest.

## Verification

Current slice verification: `cargo fmt --check`; `cargo build --bin phpc`;
native smoke matrix 6/6; focused
`cargo test compile_explode_internal_function_to_native_binary` 1/1; focused
PHPT rows `ext/standard/tests/strings/explode.phpt` and
`ext/standard/tests/general_functions/array_is_list.phpt` 2/2; PHPT COW
manifest 29/29; and post-merge COW gate 17/17 oracle, 3/3 notice, 6/6
diagnostics.

Follow-ups remain destructor visibility/exception/reference/global edges,
typed/promoted properties, interfaces/traits, bracketed/grouped namespaces,
broader fallback/reflection, magic methods, first-class callables, dynamic
includes, unsupported internals, scalar offset-lvalues, assertion config,
binary-safe array keys, class-constant edges, dynamic-variable
null-coalescing/by-reference lvalues, remaining embedded-NUL internals, inc/dec
Unicode/reference/COW diagnostics, object IDs, broader `chr()`/`abs()` edges,
`sqrt()` non-finite edges, and object/reference targets.
