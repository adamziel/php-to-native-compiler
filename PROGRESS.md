# PTN Progress

Refresh: 2026-06-12T09:28Z
Measured: `ptn-4ngk` rebased after `origin/master` `86e0e60e1`.

Recent RC slices cover property/static-property inc/dec, dynamic-variable
array/string-offset writes and unsets, array-offset inc/dec, array/append
compound assignments, bounded private properties, object `var_export()`,
`get_class()`, property/static quiet probes, direct array mutators including
`sort()`/`asort()`/`arsort()`/`ksort()`/`krsort()`/`rsort()`/`natsort()`/
`natcasesort()`, set operations, `array_udiff*()`, exact string/lang rows,
highlight output paths, `join()`/`implode()`, `sprintf()`, `array_product()`,
`array_keys()`, key-boundary helpers, `array_search()`, catchable `intdiv()`,
ASCII case string internals, `strrev()`, PHP-style `ceil()`/`floor()`
numeric-argument diagnostics, and `is_countable()` over current boxed arrays.

Recent movers include dynamic-root array/string-offset writes/unsets, object
`var_export()`, property/static quiet probes, default sort-family mutators,
sort flag diagnostics, offset compounds, property/static inc/dec,
`array_product()`, `array_keys()` filtering, key-boundary helpers,
`array_search()` key lookup, `intdiv()` exceptions, ASCII case conversion,
key-preserving natural `natsort()`/`natcasesort()`, binary-safe `strrev()`,
`ceil()`/`floor()` invalid numeric-string/unsupported-operand `TypeError`
parity, and the current boxed-array `is_countable()` predicate.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 533 | 533 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 205 | 205 | 0 |
| PHPT Zend rows | 76 | 76 | 0 |
| PHPT ext/standard rows | 82 | 82 | 0 |
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
interpolation, streams, `pow()`, `array_merge()`, `strrev()`, `array_search()`,
`call_user_func_array()`, highlight output paths, `var_export()`, direct array
mutators including `natcasesort()`, set operations, inc/dec
statements/expressions, and dynamic-variable array/string-offset writes.

## Remaining Bounded Failures

- None in the current 205-row bounded manifest.

## Verification

Verification: `ptn-uvy0` passed fmt, build, focused `array_search()` coverage,
focused `array_search1.phpt`, and `cargo test` with native/compiler 529/529
plus COW/doc-tests. `ptn-h0ig` adds adjacent `array_search()` coverage.
`ptn-t6qd` adds `natcasesort()` plus
`ext/standard/tests/array/sort/natcasesort_basic.phpt`. Final gates passed:
fmt, build, focused parser/native/PHPT `natcasesort()` coverage, and
`cargo test` with native/compiler 531/531 plus COW/doc-tests. `ptn-ah6f`
adds `ceil()`/`floor()` numeric-argument diagnostics and targeted
`ext/standard/tests/math/floorceil.phpt` coverage after rebase. `ptn-4ngk`
adds current boxed-array `is_countable()` coverage. Focused parser/native
reducers passed, `cargo test` passed with native/compiler 533/533 plus
COW/doc-tests, bounded PHPT passed 205/205, COW PHPT passed 29/29, and the
post-merge COW gate passed 25/25.

Follow-ups remain full visibility/inheritance metadata, typed/promoted
properties, interfaces/traits, namespaces, reflection, remaining magic methods,
first-class callables, destructors, dynamic includes, unsupported internals,
scalar offset-lvalues, assertion configuration, binary-safe array keys,
remaining inc/dec Unicode/reference/COW/diagnostic edges, object
metadata/IDs/visibility edges, and broader foreach destructuring or reference
targets.
