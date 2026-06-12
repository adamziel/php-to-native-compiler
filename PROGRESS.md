# PTN Progress

Refresh: 2026-06-12T08:42Z
Measured: `ptn-h0ig` rebased after `origin/master` `7f28328c`.

Recent RC slices cover property/static-property inc/dec
statements/expressions, scalar/string inc/dec value semantics, dynamic-variable
array/string-offset writes and unsets, array-offset inc/dec
statements/expressions, variable-root array/append compound assignment
expressions, bounded private instance-property access, protected/private dump
metadata, full/short lazy ternaries, PHP-style object `var_export()`, bounded
`get_class()` metadata, property/static-property `isset()`/`empty()`/`??` quiet
probes, direct array mutators including default
`sort()`/`asort()`/`arsort()`/`ksort()`/`krsort()`/`rsort()`/`natsort()`, sort
flag diagnostics, set operations, `array_udiff*()`, exact string/lang rows,
highlight output paths, `join()`/`implode()`, scalar `sprintf()`,
`array_product()`, `array_keys()`, key-boundary helpers, `array_search()`,
catchable `intdiv()` exceptions, ASCII case string internals, and `strrev()`.

Recent movers include dynamic-root array/string-offset writes/unsets, object
`var_export()`, `get_class()` metadata, property/static quiet probes, default
sort/asort/arsort/ksort/krsort/rsort/natsort, sort flag diagnostics, offset
compound assignments, property/static inc/dec, `array_product()`,
`array_keys()` filtering, key-boundary helpers, `array_search()` key lookup,
exact `intdiv()` exceptions, ASCII `strtolower()`/`strtoupper()`,
key-preserving natural `natsort()`, and binary-safe `strrev()`.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 530 | 530 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 204 | 204 | 0 |
| PHPT Zend rows | 76 | 76 | 0 |
| PHPT ext/standard rows | 81 | 81 | 0 |
| PHPT tests/basic+func+lang | 45 | 45 | 0 |
| PHPT other rows | 2 | 2 | 0 |
| PHPT COW manifest | 29 | 29 | 0 |
| Post-merge COW gate | 25 | 25 | 0 |
| PHPT callback manifest | 2 | 2 | 0 |

## RC Surface

Parser/IR/C backend, boxed values, variables/constants, strings, scalar
operators, ternary expressions, ordered arrays, `foreach`, branch/loop/switch,
compile-time includes, selected internals, COW/reference slices, user
functions, call-frame introspection, scalar type hints with literal-array
defaults, bounded closures/callables, `stdClass`, public class/object shells,
declared properties, quiet probes, metadata intrinsics, `is_callable()`,
assertions, interpolation, streams, `pow()`, `array_merge()`, `strrev()`,
`array_search()`, `call_user_func_array()`, highlight output paths,
`var_export()`, direct array mutators, set operations, inc/dec
statements/expressions, and dynamic-variable array/string-offset writes.

## Remaining Bounded Failures

- None in the current 204-row bounded manifest.

## Verification

Verification: `ptn-uvy0` passed fmt, build, focused `array_search()` coverage,
focused `array_search1.phpt`, and `cargo test` with native/compiler 529/529
plus COW/doc-tests. `ptn-h0ig` adds adjacent `array_search()` coverage. Final
gates are fmt, build, focused coverage, and `cargo test` with native/compiler
530/530 plus COW/doc-tests.

Follow-ups remain full visibility/inheritance metadata, typed/promoted
properties, interfaces/traits, namespaces, reflection, remaining magic methods,
first-class callables, destructors, dynamic includes, unsupported internals,
scalar offset-lvalues, assertion configuration, binary-safe array keys,
remaining inc/dec Unicode/reference/COW/diagnostic edges, object
metadata/IDs/visibility edges, and broader foreach destructuring or reference
targets.
