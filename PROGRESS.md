# PTN Progress

Refresh: 2026-06-12T01:56Z
Measured: `ptn-uqzn` rebased after `origin/master` `9ce948357`.

Recent RC slices cover array/key canonicalization, foreach targets, catchable
arithmetic/assertion boundaries, public `__call`/`__toString`, scalar
`var_dump()`, inline HTML, string-offset diagnostics, boxed streams,
`array_key_exists()` parity, PHP-style float output, `ksort()`/`shuffle()`,
array cursor/pop/shift mutation, literal-array defaults, `pow()`,
`array_merge()`, `call_user_func_array()`, `phpc -d error_reporting=N`,
filtered bitwise diagnostics, legacy `${var}` deprecations, scalar/array
`var_export()`, `array_diff*()`/`array_intersect*()`/`array_udiff*()`,
bounded `highlight_string()`/`highlight_file()`, array-offset `++`/`--`,
`join()`/`implode()`, scalar `sprintf()`, dynamic-variable
array/string-offset writes, and this slice's expression-form array-offset
inc/dec plus bounded private instance-property access from declaring-class
methods.

Recent PHPT movers: `ptn-dcyl` exact `strings/006`, `ptn-e3zm` focused
`array_udiff*()`, `ptn-bhp6` exact `strings/004`, `ptn-e3ha`
`tests/lang/024` via `${expr}[key] = value`, `ptn-9dn7` nested dynamic-root
appends/string offsets, `ptn-e02q` expression-form array-offset inc/dec, and
`ptn-uqzn` private-property access moving `array/007` to the next `? :`
expression blocker.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 492 | 492 | 0 |
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
operators, ordered arrays, `foreach`, branch/loop/switch, compile-time
includes, selected internals, COW/reference slices, user functions, call-frame
introspection, scalar type hints with literal-array defaults, bounded
closures/callables, `stdClass`, public class/object shells, public properties,
public constructors, `is_callable()`, assertions, heredoc/nowdoc,
interpolation, streams, `pow()`, `array_merge()`, `join()`/`implode()`,
scalar `sprintf()`, `call_user_func_array()`, CLI/error-reporting wiring,
highlight output paths, scalar/array `var_export()`, direct array mutators,
set operations, array-offset inc/dec statements and expressions, and simple
dynamic-variable array/string-offset writes.
Declared private instance properties are initialized, read/written from the
declaring class, denied externally, and labeled in `var_dump()`.

## Remaining Bounded Failures

- `ext/standard/tests/array/007.phpt`: covered diff/intersect/udiff reducers
  now pass the private-member class setup and stop at unsupported ternary
  conditional expressions in object comparator functions.

## Verification

Post-rebase baseline from `ptn-e02q`: `cargo fmt --check && cargo test`,
native smoke 6/6, bounded PHPT 199/200, COW PHPT 29/29, post-merge COW 25/25.
This slice adds focused `cargo test private_instance_properties --test
compile_native`, public-property regression, exact `array/007.phpt` failing
later at ternary parse syntax, and changed-branch COW PHPT 29/29.

Follow-ups remain ternary expressions, visibility/inheritance,
typed/promoted properties, interfaces/traits, namespaces, reflection,
remaining magic methods,
first-class callables, destructors, dynamic includes, unsupported internals,
scalar offset-lvalues, assertion configuration, non-numeric and
non-variable-root inc/dec, and broader foreach destructuring/reference targets.
