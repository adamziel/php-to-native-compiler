# PTN Progress

Refresh: 2026-06-12T11:42Z
Measured: `ptn-h1mb` rebased after `origin/master` `641ceea11`.

Recent RC slices cover property/static-property inc/dec, dynamic-variable
array/string-offset writes and unsets, array/append compound assignments,
bounded private properties, object `var_export()`, `get_class()`, quiet
property/static probes, direct array mutators including
`sort()`/`asort()`/`arsort()`/`ksort()`/`krsort()`/`rsort()`/`natsort()`/
`natcasesort()`, explicit regular sort flags, set operations,
`array_udiff*()`, exact string/lang rows, highlight output paths,
`join()`/`implode()`, `sprintf()`, `array_product()`, key helpers,
`array_search()`, `array_pad()`, catchable `intdiv()`, ASCII string
internals through `ucfirst()`/`lcfirst()` and trim-family reducers,
`ceil()`/`floor()` numeric diagnostics, `is_countable()`, and the first
unbracketed namespace/name-resolution slice for declarations, qualified names,
imports, and `__NAMESPACE__`.

Recent movers include dynamic-root offset writes/unsets, property/static quiet
probes and inc/dec, direct sort-family mutators and flag diagnostics, array
key/search/pad helpers, catchable `intdiv()`, ASCII case and trim reducers,
`ceil()`/`floor()` `TypeError` parity, `is_countable()`, and namespace PHPT
rows `ns_001`, `ns_002`, `ns_003`, and `ns_014`.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 541 | 541 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 213 | 213 | 0 |
| PHPT Zend rows | 80 | 80 | 0 |
| PHPT ext/standard rows | 86 | 86 | 0 |
| PHPT tests/basic+func+lang | 45 | 45 | 0 |
| PHPT other rows | 2 | 2 | 0 |
| PHPT COW manifest | 29 | 29 | 0 |
| Post-merge COW gate | 25 | 25 | 0 |
| PHPT callback manifest | 2 | 2 | 0 |

## RC Surface

Parser/IR/C backend, boxed values, variables/constants, strings, scalar
operators, ternary expressions, ordered arrays, `foreach`, branch/loop/switch,
compile-time includes, selected internals, COW/reference slices, user
functions, call-frame introspection, scalar type hints, bounded closures,
`stdClass`, public class/object shells, declared properties, quiet probes,
metadata intrinsics, `is_callable()`, `is_countable()`, assertions,
interpolation, unbracketed namespaces, simple imports, streams, `pow()`,
`array_merge()`, `array_pad()`, `strrev()`, `ucfirst()`, `lcfirst()`,
trim-family internals, `array_search()`, `call_user_func_array()`, highlight
output paths, `var_export()`, direct array mutators including `natcasesort()`,
explicit regular sort flags, set operations, inc/dec, and dynamic-variable
array/string-offset writes.

## Remaining Bounded Failures

- None in the current 213-row bounded manifest.

## Verification

Verification: recent slices added `array_search()`, `natcasesort()`,
`ceil()`/`floor()` diagnostics, `is_countable()`, `ucfirst()`, `lcfirst()`,
`array_pad()`, explicit regular sort flags, trim-family byte charlists, and
namespace parser/resolver coverage with focused namespace PHPT rows 4/4.

Follow-ups remain visibility/inheritance metadata, typed/promoted properties,
interfaces/traits, bracketed/grouped namespace forms, namespace fallback
parity, reflection, remaining magic methods, first-class callables, destructors,
dynamic includes, unsupported internals, scalar offset-lvalues, assertion
configuration, binary-safe array keys, inc/dec Unicode/reference/COW/
diagnostic edges, object metadata/IDs/visibility edges, and broader foreach
destructuring or reference targets.
