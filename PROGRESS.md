# PTN Progress

Refresh: 2026-06-12T06:30Z
Measured: `ptn-e69a` rebased after `origin/master` `6657ca0af`.

Recent RC slices cover property/static-property inc/dec
statements/expressions, scalar/string inc/dec value semantics, dynamic-variable
array/string-offset writes and unsets, array-offset inc/dec
statements/expressions, variable-root array/append compound assignment
expressions, bounded private instance-property access, protected/private dump
metadata, full/short lazy ternaries, PHP-style object `var_export()`, bounded
`get_class()` metadata, property/static-property `isset()`/`empty()`/`??` quiet
probes, direct array mutators including default `sort()`/`asort()`/`rsort()`,
sort flag diagnostics, set operations, `array_udiff*()`, exact
`strings/004`, `strings/006`, and `tests/lang/024`, highlight output paths,
`join()`/`implode()`, and scalar `sprintf()`.

Recent movers include exact string/lang/array rows, `array_udiff*()`,
dynamic-root array/string-offset writes and unsets, object `var_export()`,
`get_class()` metadata, property/static quiet probes, default sort/asort/rsort,
sort flag diagnostics, offset compound assignments, and property/static inc/dec.
`ptn-e69a` adds null, boolean, numeric-string, alphanumeric-string,
dynamic-root, array-offset, property/static-property, and catchable non-scalar
inc/dec value coverage.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 519 | 519 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 200 | 200 | 0 |
| PHPT Zend rows | 76 | 76 | 0 |
| PHPT ext/standard rows | 77 | 77 | 0 |
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
declared instance-property defaults and metadata, property/static-property quiet
probes, public constructors, class/object metadata intrinsics, `is_callable()`,
assertions, heredoc/nowdoc, interpolation, streams, `pow()`, `array_merge()`,
`join()`/`implode()`, scalar `sprintf()`, `call_user_func_array()`,
CLI/error-reporting wiring, highlight output paths, scalar/array/current-object
`var_export()`, direct array mutators including `sort()`/`asort()`/`rsort()`, set
operations, array-offset/property/static inc/dec statements/expressions,
scalar/string inc/dec value semantics, variable-root array/append compound
assignment expressions, dynamic inc/dec expressions, and dynamic-variable
array/string-offset writes and unsets.

## Remaining Bounded Failures

- None in the current 200-row bounded manifest.

## Verification

Verification: `ptn-bfwv` passed fmt, build, focused `asort()`/assert, and
`cargo test` with native/compiler 515/515 plus COW/doc-tests. `ptn-1p1g` added
focused parser/native `rsort()` and `cargo test` 516/516. `ptn-9gfw` added
dynamic `sort()`/`asort()`/`rsort()` flag coverage and `cargo test` 517/517.
`ptn-rrfl` added static-property quiet probes and `cargo test` 518/518.
`ptn-e69a` adds scalar/string inc/dec reducer coverage; final gates are the
focused reducer and `cargo test` 519/519.

Follow-ups remain full visibility/inheritance metadata, typed/promoted
properties, interfaces/traits, namespaces, reflection, remaining magic methods,
first-class callables, destructors, dynamic includes, unsupported internals,
scalar offset-lvalues, assertion configuration, remaining inc/dec
Unicode/reference/COW/diagnostic edges, object metadata/IDs/visibility edges,
and broader foreach destructuring or reference targets.
