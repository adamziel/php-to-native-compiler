# PTN Progress

Refresh: 2026-06-12T17:42Z
Measured: `ptn-ausc` rebased on current `origin/master` `c9b748f11`.

Recent RC slices cover public class constants, embedded-NUL `var_export()`,
`explode()`, namespaced internal fallback, static-property `??=`, once guards,
property/static inc/dec, dynamic-variable writes/unsets, array/string-offset
compound assignments, private properties, inherited parent-private slots,
public `__destruct()` lifecycle dispatch, quiet probes, array mutators, sort
flags, set operations, `array_udiff*()`, `join()`/`implode()`, bounded
`sprintf()`/`printf()`, bounded `json_encode()`, `array_is_list()`,
`array_search()`, `array_slice()`, `array_pad()`, `array_reverse()`,
`count()`/`sizeof()`, `str_pad()`, `str_shuffle()`, `strtr()`,
`chunk_split()`, string-internal object/closure given-type diagnostics,
`abs()`/`sqrt()`/`fdiv()` TypeErrors, ASCII case/trim, PHP/CLI/Zend metadata,
`php_uname()`, `ReflectionFunction`, namespaces, foreach list destructuring,
dynamic include/require dispatch, and return-only `void` declarations.

Recent movers include shared string-internal TypeError naming for `stdClass`,
exceptions, and `Closure`, plus bounded PHPT coverage for `array_reverse()`,
`php_uname()`, `str_shuffle()`, and `strtr()`.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 572 | 572 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 226 | 226 | 0 |
| PHPT Zend rows | 80 | 80 | 0 |
| PHPT ext/standard rows | 96 | 96 | 0 |
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
`explode()`, shared string-internal diagnostics, highlight paths,
`var_export()`, array mutators, inc/dec, foreach destructuring,
dynamic-variable writes/unsets, and array/string-offset compound assignments.

## Remaining Bounded Failures

- None in the current 226-row bounded manifest.

## Verification

Current slice verification: `cargo fmt --check`; focused
`cargo test compile_string_internals_reject_non_string_values_to_native_binary`;
affected PHPT rows `strlen.phpt`, `ord_basic.phpt`, and `dirname_basic.phpt`
3/3; bounded PHPT manifest 226/226; PHPT COW manifest 29/29; and post-merge
COW gate 17/17 oracle, 3/3 notice, 6/6 diagnostics.

Follow-ups remain destructor visibility/exception/reference/global edges,
typed/promoted properties, interfaces/traits, bracketed/grouped namespaces,
broader fallback/reflection, magic methods, first-class callables, dynamic
includes, unsupported internals, scalar offset-lvalues, assertion config,
binary-safe array keys, class-constant edges, dynamic-variable
null-coalescing/by-reference lvalues, remaining embedded-NUL internals, inc/dec
Unicode/reference/COW diagnostics, object IDs, broader `chr()`/`abs()` edges,
`sqrt()` non-finite edges, and object/reference targets.
