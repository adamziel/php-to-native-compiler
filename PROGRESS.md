# PTN Progress

Refresh: 2026-06-12T02:10Z
Measured: `ptn-vg78` rebased after `origin/master` `fdf2bf8af`.

Recent RC slices cover dynamic-variable array/string-offset writes,
array-offset inc/dec statements and expressions, bounded private
instance-property access from declaring-class methods, protected instance
property parsing/initialization in current object storage, and full/short
ternary expressions with lazy selected-arm evaluation. Earlier frontier movers
include exact `strings/004`, `strings/006`, `tests/lang/024`,
array set operations, `array_udiff*()`, highlight output paths, `join()` /
`implode()`, and scalar `sprintf()`.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 496 | 496 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 200 | 199 | 1 |
| PHPT Zend rows | 76 | 76 | 0 |
| PHPT ext/standard rows | 77 | 76 | 1 |
| PHPT tests/basic+func+lang | 45 | 45 | 0 |
| PHPT other rows | 2 | 2 | 0 |
| PHPT COW manifest | 29 | 29 | 0 |
| Post-merge COW gate | 25 | 25 | 0 |
| PHPT callback manifest | 2 | 2 | 0 |

## RC Surface

Parser/IR/C backend, boxed values, variables/constants, strings, scalar
operators, ternary expressions, ordered arrays, `foreach`,
branch/loop/switch, compile-time includes, selected internals, COW/reference
slices, user functions, call-frame introspection, scalar type hints with
literal-array defaults, bounded closures/callables, `stdClass`, public
class/object shells, declared instance-property defaults, public constructors,
`is_callable()`, assertions, heredoc/nowdoc, interpolation, streams, `pow()`,
`array_merge()`, `join()`/`implode()`, scalar `sprintf()`,
`call_user_func_array()`, CLI/error-reporting wiring, highlight output paths,
scalar/array `var_export()`, direct array mutators, set operations,
array-offset inc/dec statements and expressions, and dynamic-variable
array/string-offset writes.
Declared private instance properties are initialized, read/written from the
declaring class, denied externally, and labeled in `var_dump()`; protected
instance declarations are accepted and initialized for current in-class use.

## Remaining Bounded Failures

- `ext/standard/tests/array/007.phpt`: now parses non-public instance
  properties and executes ternary comparator callbacks; the remaining diff is
  PHP-exact object output, notably `var_export()` object layout and broader
  non-public property metadata.

## Verification

This slice: `cargo fmt --check`, `cargo check`, focused
`cargo test ternary --test compile_native`, focused
`cargo test non_public --test compile_native`, full
`cargo test --test compile_native` 496/496, `cargo test --lib` 3/3, COW PHPT
29/29, bounded PHPT baseline 199/200 with only `array/007`, and focused
`array/007.phpt` post-change failing only on object output parity.

Follow-ups remain full visibility/inheritance metadata, typed/promoted
properties, interfaces/traits, namespaces, reflection, remaining magic methods,
first-class callables, destructors, dynamic includes, unsupported internals,
scalar offset-lvalues, assertion configuration, remaining non-variable-root
inc/dec, PHP-exact object dump/export metadata, and broader foreach
destructuring/reference targets.
