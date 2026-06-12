# PTN Progress

Refresh: 2026-06-12T11:15Z
Measured: `ptn-mznl` rebased after `origin/master` `a32153f14`.

Recent RC slices cover property/static-property inc/dec, dynamic-variable
array/string-offset writes and unsets, array/append compound assignments,
bounded private properties, object `var_export()`, `get_class()`, quiet
property/static probes, direct array mutators including
`sort()`/`asort()`/`arsort()`/`ksort()`/`krsort()`/`rsort()`/`natsort()`/
`natcasesort()`, explicit `SORT_REGULAR`/`0` flags for the regular direct
sort-family subset, set operations, `array_udiff*()`, exact string/lang rows,
highlight output paths, `join()`/`implode()`, `sprintf()`, `array_product()`,
`array_keys()`, key-boundary helpers, `array_search()`, `array_pad()`,
catchable `intdiv()`, ASCII case string internals, `strrev()`, `ucfirst()`,
`lcfirst()`, trim-family string internals, PHP-style `ceil()`/`floor()`
numeric-argument diagnostics, and `is_countable()` over current boxed arrays.

Recent movers include dynamic-root array/string-offset writes/unsets, object
`var_export()`, property/static quiet probes, default sort-family mutators,
sort flag diagnostics, offset compounds, property/static inc/dec,
`array_product()`, `array_keys()` filtering, key-boundary helpers,
`array_search()` key lookup, `array_pad()` ordered-map padding,
`intdiv()` exceptions, ASCII case conversion, key-preserving natural
`natsort()`/`natcasesort()`, binary-safe `strrev()`, first-byte
`ucfirst()`/`lcfirst()`, `ceil()`/`floor()` invalid numeric-string/
unsupported-operand `TypeError` parity, current boxed-array `is_countable()`,
and explicit regular direct sort flags plus trim-family byte charlists.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 538 | 538 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 209 | 209 | 0 |
| PHPT Zend rows | 76 | 76 | 0 |
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
interpolation, streams, `pow()`, `array_merge()`, `array_pad()`, `strrev()`,
`ucfirst()`, `lcfirst()`, trim-family string internals, `array_search()`,
`call_user_func_array()`, highlight output paths, `var_export()`, direct array
mutators including `natcasesort()`, explicit regular direct sort flags, set
operations, inc/dec statements/expressions, and dynamic-variable
array/string-offset writes.

## Remaining Bounded Failures

- None in the current 209-row bounded manifest.

## Verification

Verification: recent merged slices added `array_search()`, `natcasesort()`,
`ceil()`/`floor()` numeric diagnostics, current boxed-array `is_countable()`,
modeled `ucfirst()`, and modeled `array_pad()` with their focused PHPT/native
coverage. `ptn-qmtv` adds explicit `SORT_REGULAR`/`0` direct sort-family flags
for the regular mutator subset. `ptn-fvk9` adds `trim()`/`ltrim()`/`rtrim()`
with default bytes, ascending byte-range charlists, and three bounded
trim-family PHPT rows. `ptn-mznl` adds modeled `lcfirst()` reducer coverage
over the shared first-byte string case helper.

Follow-ups remain full visibility/inheritance metadata, typed/promoted
properties, interfaces/traits, namespaces, reflection, remaining magic methods,
first-class callables, destructors, dynamic includes, unsupported internals,
scalar offset-lvalues, assertion configuration, binary-safe array keys,
remaining inc/dec Unicode/reference/COW/diagnostic edges, object
metadata/IDs/visibility edges, and broader foreach destructuring or reference
targets.
