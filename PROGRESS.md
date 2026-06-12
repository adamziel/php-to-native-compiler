# PTN Progress

Refresh: 2026-06-12T02:59Z
Measured: `ptn-y5na` rebased after `origin/master` `de35b2068`.

Recent RC slices cover dynamic-variable array/string-offset writes and unsets,
array-offset inc/dec statements and expressions including dynamic roots,
bounded private instance-property access from declaring-class methods,
protected instance property parsing/initialization in current object storage,
and full/short ternary expressions with lazy selected-arm evaluation. Earlier
frontier movers include exact `strings/004`, `strings/006`,
`tests/lang/024`, array set operations, `array_udiff*()`, highlight output
paths, `join()`/`implode()`, and scalar `sprintf()`. This slice adds
dynamic-root array/string-offset unsets through the shared path-unset helper
for nested offsets, string-offset errors, and target-before-offset evaluation
order.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 502 | 502 | 0 |
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
operators, ternary expressions, ordered arrays, `foreach`, branch/loop/switch,
compile-time includes, selected internals, COW/reference slices, user
functions, call-frame introspection, scalar type hints with literal-array
defaults, bounded closures/callables, `stdClass`, public class/object shells,
declared instance-property defaults, public constructors, `is_callable()`,
assertions, heredoc/nowdoc, interpolation, streams, `pow()`, `array_merge()`,
`join()`/`implode()`, scalar `sprintf()`, `call_user_func_array()`,
CLI/error-reporting wiring, highlight output paths, scalar/array
`var_export()`, direct array mutators, set operations, array-offset inc/dec,
dynamic-variable array/string-offset writes and unsets, and dynamic inc/dec
expressions. Declared private instance properties are initialized, read/written
from the declaring class, denied externally, and labeled in `var_dump()`;
protected instance declarations are accepted and initialized for current
in-class use.

## Remaining Bounded Failures

- `ext/standard/tests/array/007.phpt`: now parses non-public instance
  properties and executes ternary comparator callbacks; the remaining diff is
  PHP-exact object output, notably `var_export()` object layout and broader
  non-public property metadata.

## Verification

Recent merged baseline: focused ternary/non-public tests, dynamic inc/dec
tests, `cargo fmt --check`, full `cargo test` with native/compiler 500/500,
bounded PHPT 199/200, and COW PHPT 29/29. This slice adds focused
`cargo test --test compile_native dynamic_variable_array_dimension` and
native/compiler 502/502 after rebase.

Follow-ups remain full visibility/inheritance metadata, typed/promoted
properties, interfaces/traits, namespaces, reflection, remaining magic methods,
first-class callables, destructors, dynamic includes, unsupported internals,
scalar offset-lvalues, assertion configuration, non-numeric/property/static
inc/dec, PHP-exact object dump/export metadata, and broader foreach
destructuring/reference targets.
