# PTN Progress

Refresh: 2026-06-12T07:10Z
Measured: `ptn-1gx7` rebased after `origin/master` `76fbd7d61`.

Recent RC slices cover property/static-property inc/dec
statements/expressions, scalar/string inc/dec value semantics, dynamic-variable
array/string-offset writes and unsets, array-offset inc/dec
statements/expressions, variable-root array/append compound assignment
expressions, bounded private instance-property access, protected/private dump
metadata, full/short lazy ternaries, PHP-style object `var_export()`, bounded
`get_class()` metadata, property/static-property `isset()`/`empty()`/`??` quiet
probes, direct array mutators including default `sort()`/`asort()`/`rsort()`,
`arsort()`, sort flag diagnostics, set operations, `array_udiff*()`, exact
`strings/004`, `strings/006`, and `tests/lang/024`, highlight output paths,
`join()`/`implode()`, scalar `sprintf()`, `array_product()`, and catchable
`intdiv()` exceptions.

Recent movers include exact string/lang/array rows, `array_udiff*()`,
dynamic-root array/string-offset writes and unsets, object `var_export()`,
`get_class()` metadata, property/static quiet probes, default
sort/asort/rsort/arsort, sort flag diagnostics, offset compound assignments,
and property/static inc/dec. `ptn-e69a` adds scalar/string inc/dec value
coverage; `ptn-n6db` adds `array_product()`; `ptn-rq8k` adds exact `intdiv()`
exceptions; `ptn-1gx7` adds key-preserving descending `arsort()`.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 522 | 522 | 0 |
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
`join()`/`implode()`, scalar `sprintf()`, `array_product()`,
`call_user_func_array()`,
CLI/error-reporting wiring, highlight output paths, scalar/array/current-object
`var_export()`, direct array mutators including `sort()`/`asort()`/`arsort()`/
`rsort()`, set operations, array-offset/property/static inc/dec
statements/expressions, scalar/string inc/dec value semantics, variable-root
array/append compound assignment expressions, dynamic inc/dec expressions, and
dynamic-variable
array/string-offset writes and unsets.

## Remaining Bounded Failures

- None in the current 200-row bounded manifest.

## Verification

Verification: `ptn-rq8k` passed fmt, build, focused `intdiv`, and `cargo test`
with native/compiler 521/521 plus COW/doc-tests. `ptn-1gx7` adds focused
parser/native `arsort()` coverage; final gates are fmt, build, focused
coverage, and `cargo test` with native/compiler 522/522.

Follow-ups remain full visibility/inheritance metadata, typed/promoted
properties, interfaces/traits, namespaces, reflection, remaining magic methods,
first-class callables, destructors, dynamic includes, unsupported internals,
scalar offset-lvalues, assertion configuration, remaining inc/dec
Unicode/reference/COW/diagnostic edges, object metadata/IDs/visibility edges,
and broader foreach destructuring or reference targets.
