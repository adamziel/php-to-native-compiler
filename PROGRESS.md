# PTN Progress

Refresh: 2026-06-11T23:54Z
Measured: `ptn-e3zm` rebased on `origin/master` at `669568be7`.

Recent RC slices cover array/key canonicalization, foreach targets, catchable
arithmetic/assertion boundaries, public `__call`/`__toString`,
precision-driven float output, scalar `var_dump()` spelling, inline HTML,
string-offset diagnostics, boxed stream resources, `array_key_exists()` parity,
PHP-style float exponent spelling, `ksort()`/`shuffle()`, array cursor/pop/
shift mutation, literal-array defaults, `pow()`, `array_merge()`,
`highlight_string()`/`highlight_file()` output paths, `phpc -d
error_reporting=N`, `call_user_func_array()`, filtered bitwise diagnostics,
legacy `${var}` deprecations, scalar/array `var_export()`, ordered
`array_diff*()`/`array_intersect*()`, and this slice's `array_udiff()`,
`array_udiff_assoc()`, and `array_udiff_uassoc()`.

`ptn-pjah` keeps streams open across `fopen()`, closes them through
`fclose()`, reports closed resources through `gettype()`, and makes
`is_resource()` false after close. `ptn-ndqj` covers scalar/array
`var_export()` and moves `array/008.phpt` green.
`ptn-dcyl` moves `ext/standard/tests/strings/006.phpt` green while retaining
exact `highlight_string()` escaping/output and no-buffer `ob_get_contents()`
false. `ptn-e3zm` adds focused `array_udiff*()` callback reducers.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 481 | 481 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 200 | 197 | 3 |
| PHPT Zend rows | 76 | 76 | 0 |
| PHPT ext/standard rows | 77 | 75 | 2 |
| PHPT tests/basic+func+lang | 45 | 44 | 1 |
| PHPT other rows | 2 | 2 | 0 |
| PHPT COW manifest | 29 | 29 | 0 |
| Post-merge COW gate | 25 | 25 | 0 |
| PHPT callback manifest | 2 | 2 | 0 |

## RC Surface

Parser/IR/C backend, boxed values, variables/constants, strings, scalar
operators, ordered arrays, `foreach`, branch/loop/switch, compile-time
includes, selected internals, COW/reference slices, top-level functions,
call-frame introspection, scalar type hints including literal-array defaults,
bounded closures/callables, `stdClass`, public class/object shells, public
properties/static properties, public constructors, public `__call`/
`__toString`, `is_callable()`, assertion errors, heredoc/nowdoc,
interpolation slices, stream resources, `pow()`, `array_merge()`,
`call_user_func_array()`, CLI `error_reporting` ini wiring, bounded
`highlight_string()`/`highlight_file()`/empty output-buffer reads,
error-reporting-aware bitwise integer conversion diagnostics, scalar/array
`var_export()`, and direct array mutators and set operations including
`shuffle`, `array_diff*()`, `array_intersect*()`, and `array_udiff*()`.

## Remaining Bounded Failures

- `ext/standard/tests/array/007.phpt`: first diff reducers are covered, but the
  full row still reaches non-public class members before callbacks.
  `001`, `005`, `008`, `array_column()`, and `array_key_exists()` variants are
  covered.
- `ext/standard/tests/strings/004.phpt`: reaches array-element inc/dec after
  sort/shuffle support. `005`, `006`, and `strlen` are covered.
- `tests/lang/024.phpt`: dynamic-variable array-offset lvalue blocker.

## Verification

Post-rebase verification on `341f09d28c9f`: `cargo fmt --check`,
`git diff --check origin/master..HEAD`, and full `cargo test` pass, including
native/compiler 481/481 plus COW reducer/oracle suites. Bounded PHPT
`run-20260611T232729Z-manifest.log` is 197/200 with only `array/007`,
`strings/004`, and `tests/lang/024` failing. COW PHPT
`run-20260611T233905Z-*` is 29/29. `ptn-e3zm` was verified with
focused `compile_array_udiff_variants_to_native_binary`; exact generated
`array/007.php` still fails at line 51 with `non-public class members are
unsupported`.

Follow-ups remain broad visibility/inheritance, typed/promoted properties,
interfaces/traits, namespaces, reflection, remaining magic methods, first-class
callables, destructors, dynamic includes, full unsupported-internal coverage,
remaining scalar offset-lvalues, assertion configuration, non-direct-variable
or non-numeric inc/dec, and broader foreach destructuring/reference targets.
