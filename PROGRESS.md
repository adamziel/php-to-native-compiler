# PTN Progress

Refresh: 2026-06-12T03:27Z
Measured: `ptn-ir7c` rebased after `origin/master` `54564d870`.

Recent RC slices cover dynamic-variable array/string-offset writes and unsets,
array-offset inc/dec statements and expressions including dynamic roots,
bounded private instance-property access from declaring-class methods,
protected/private property metadata for initialization and dump labels,
full/short ternary expressions with lazy selected-arm evaluation, PHP-style
object `var_export()`, array set operations, `array_udiff*()`, exact
`strings/004`, `strings/006`, and `tests/lang/024`, highlight output paths,
`join()` / `implode()`, and scalar `sprintf()`.

Recent PHPT movers: `ptn-dcyl` exact `strings/006`, `ptn-e3zm` focused
`array_udiff*()`, `ptn-bhp6` exact `strings/004`, `ptn-e3ha`
`tests/lang/024` via `${expr}[key] = value`, `ptn-y5na` dynamic-root
array/string-offset unsets, and `ptn-ir7c` exact `array/007` via non-public
property metadata, ternary comparator expressions, and PHP-style object
`var_export()`.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 521 | 521 | 0 |
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
declared instance-property defaults and metadata, public constructors,
`is_callable()`, assertions, heredoc/nowdoc, interpolation, streams, `pow()`,
`array_merge()`, `join()`/`implode()`, scalar `sprintf()`,
`call_user_func_array()`, CLI/error-reporting wiring, highlight output paths,
scalar/array/object `var_export()`, direct array mutators, set operations,
array-offset inc/dec statements and expressions, dynamic inc/dec expressions,
and dynamic-variable array/string-offset writes and unsets.

## Remaining Bounded Failures

- None in the current 200-row bounded manifest.

## Verification

Final-base checks: `cargo fmt --check`, `cargo test` 521/521, focused
`cargo test non_public_property`, focused `cargo test ternary`, and exact
`array/007.phpt` green. This slice also ran bounded PHPT 200/200, COW PHPT
29/29, and post-merge COW gate 25/25 during rebased verification.

Follow-ups remain full visibility/inheritance metadata, typed/promoted
properties, interfaces/traits, namespaces, reflection, remaining magic methods,
first-class callables, destructors, dynamic includes, unsupported internals,
scalar offset-lvalues, assertion configuration, non-numeric/property/static
inc/dec, and broader foreach destructuring/reference targets.
