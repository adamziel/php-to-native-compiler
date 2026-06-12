# PTN Progress

Refresh: 2026-06-12T04:56Z
Measured: `ptn-9x8x` rebased after `origin/master` `cd21152f6`.

Recent RC slices cover dynamic-variable array/string-offset writes and unsets,
array-offset inc/dec statements/expressions, bounded private instance-property
access from declaring-class methods, protected/private dump metadata,
full/short lazy ternaries, PHP-style object `var_export()`, bounded
`get_class()` metadata, read-side property `isset()`/`empty()`/`??` quiet
probes, direct array mutators including default `sort()`, array set operations,
`array_udiff*()`, exact `strings/004`, `strings/006`, and `tests/lang/024`,
highlight output paths, `join()`/`implode()`, and scalar `sprintf()`.

Recent PHPT movers: `ptn-dcyl` exact `strings/006`, `ptn-e3zm`
`array_udiff*()`, `ptn-bhp6` exact `strings/004`, `ptn-e3ha`
`tests/lang/024` via `${expr}[key] = value`, `ptn-y5na` dynamic-root
array/string-offset unsets, `ptn-ir7c` exact `array/007`, and `ptn-juzx`
refined object `var_export()` for declared objects, `stdClass`, and nested
object arrays; `ptn-wrom` adds focused coverage for declared-object property
arrays nested inside `var_export()` output, and `ptn-5xx7` pins same-class
static private-property/protected export parity; `ptn-geav` adds `get_class()`
metadata coverage, and `ptn-if1w` adds property `isset()`/`empty()`/`??`
quiet-probe coverage. `ptn-9x8x` adds a focused direct-variable `sort()` COW
reducer.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 510 | 510 | 0 |
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
CLI/error-reporting wiring, highlight output paths,
scalar/array/current-object `var_export()`, direct array mutators including
`sort()`, set operations, array-offset inc/dec statements and expressions,
dynamic inc/dec expressions, and dynamic-variable array/string-offset writes
and unsets.

## Remaining Bounded Failures

- None in the current 200-row bounded manifest.

## Verification

Recent baseline: `cargo fmt --check`, full `cargo test` with native/compiler
509/509, exact `array/007.phpt`, bounded PHPT 200/200, COW PHPT 29/29, and
post-merge COW gate 25/25. `ptn-geav` adds `get_class()` coverage, and
`ptn-if1w` adds focused property `isset()`/`empty()`/`??` coverage. `ptn-9x8x`
final checks include `cargo fmt --check`, `cargo build --bin phpc`, focused
`cargo test sort`, focused parser mutation-target coverage, inline `sort()`
reducer coverage, and full `cargo test` with native/compiler 510/510.

Follow-ups remain full visibility/inheritance metadata, typed/promoted
properties, interfaces/traits, namespaces, reflection, remaining magic methods,
first-class callables, destructors, dynamic includes, unsupported internals,
scalar offset-lvalues, assertion configuration, non-numeric/property/static
inc/dec, and broader foreach destructuring/reference targets.
