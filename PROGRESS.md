# PTN Progress

Refresh: 2026-06-12T06:13Z
Measured: `ptn-9gfw` rebased after `origin/master` `8312706fd`.

Recent RC slices cover property/static-property inc/dec statements and
expressions, dynamic-variable array/string-offset writes and unsets,
array-offset inc/dec statements/expressions, variable-root array/append
compound assignment expressions, bounded private instance-property access,
protected/private dump metadata, full/short lazy ternaries, PHP-style object
`var_export()`, bounded `get_class()` metadata, read-side property
`isset()`/`empty()`/`??` quiet probes, direct array mutators including default
`sort()`/`asort()`/`rsort()`, sort flag diagnostics, array set operations,
`array_udiff*()`, exact `strings/004`, `strings/006`, and `tests/lang/024`,
highlight output paths, `join()`/`implode()`, and scalar `sprintf()`.

Recent PHPT movers: `ptn-dcyl` exact `strings/006`, `ptn-e3zm`
`array_udiff*()`, `ptn-bhp6` exact `strings/004`, `ptn-e3ha`
`tests/lang/024` via `${expr}[key] = value`, `ptn-y5na` dynamic-root
array/string-offset unsets, `ptn-ir7c` exact `array/007`, `ptn-juzx`
refined object `var_export()`, `ptn-6c76` property/static-property inc/dec,
`ptn-bfwv` default `asort()` preserving keys, `ptn-1p1g` default `rsort()`
reindexing values, and `ptn-9gfw` dynamic sort flag runtime boundaries.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 517 | 517 | 0 |
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
declared instance-property defaults and metadata, read-side property quiet
probes, public constructors, class/object metadata intrinsics, `is_callable()`,
assertions, heredoc/nowdoc, interpolation, streams, `pow()`, `array_merge()`,
`join()`/`implode()`, scalar `sprintf()`, `call_user_func_array()`,
CLI/error-reporting wiring, highlight output paths, scalar/array/current-object
`var_export()`, direct array mutators including `sort()`/`asort()`/`rsort()`, set
operations, array-offset inc/dec statements/expressions, variable-root
array/append compound assignment expressions, property/static-property inc/dec
statements/expressions, dynamic inc/dec expressions, and dynamic-variable
array/string-offset writes and unsets.

## Remaining Bounded Failures

- None in the current 200-row bounded manifest.

## Verification

Recent baseline: `ptn-bfwv` passed `cargo fmt --check`, `cargo build --bin
phpc`, focused `asort()`/assert coverage, and full `cargo test` with
native/compiler 515/515 plus COW/doc-tests. `ptn-1p1g` adds focused `rsort()`
reducer coverage; final checks include focused parser/native `rsort()`
coverage and full `cargo test` with native/compiler 516/516. `ptn-9gfw` adds
focused dynamic `sort()`/`asort()`/`rsort()` flag runtime coverage with
native/compiler 517/517.

Follow-ups remain full visibility/inheritance metadata, typed/promoted
properties, interfaces/traits, namespaces, reflection, remaining magic methods,
first-class callables, destructors, dynamic includes, unsupported internals,
scalar offset-lvalues, assertion configuration, non-numeric inc/dec edges,
object metadata/IDs/visibility edges, and broader foreach destructuring or
reference targets.
