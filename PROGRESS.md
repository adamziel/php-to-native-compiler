# PTN Progress

Refresh: 2026-06-11T23:41Z
Measured: `ptn-dcyl` rebased on `origin/master` at `532cb5559`.

Recent RC slices cover array/key canonicalization, foreach targets, catchable
arithmetic/assertion boundaries, public `__call`/`__toString`,
precision-driven float output, scalar `var_dump()` spelling, inline HTML,
string-offset diagnostics, minimal boxed stream resources, `array_key_exists()`
null/resource-key parity, PHP-style float exponent spelling, direct
`ksort()`/`shuffle()`, variable-root array-path cursor mutation,
one-argument `array_pop()`/`array_shift()`, literal-array defaults, `pow()`,
`array_merge()`, bounded `highlight_string()` output buffers,
`phpc -d error_reporting=N`, `call_user_func_array()` through shared callable
dispatch, filtered bitwise integer-conversion diagnostics, legacy `${var}`
string interpolation deprecations, scalar/array `var_export()`, ordered-array
`array_diff*()`/`array_intersect*()` helpers, and this slice's
`highlight_file()` missing-file/output-buffer behavior plus PHP-style
highlight escaping.

`ptn-pjah` keeps streams open across `fopen()`, closes them through
`fclose()`, reports closed resources through `gettype()`, and makes
`is_resource()` false after close. `ptn-ndqj` covers scalar/array
`var_export()` and moves `ext/standard/tests/array/008.phpt` green. This slice
moves `ext/standard/tests/strings/006.phpt` green while retaining exact
`highlight_string()` escaping/output and no-buffer `ob_get_contents()` false.

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
`shuffle`, `array_diff*()`, and `array_intersect*()`.

## Remaining Bounded Failures

- `ext/standard/tests/array/007.phpt`: first diff reducers are covered, but the
  full row still reaches non-public class members and later `array_udiff*()`
  callback variants. `001`, `005`, `008`, `array_column()`, and
  `array_key_exists()` variants are covered.
- `ext/standard/tests/strings/004.phpt`: reaches array-element inc/dec after
  sort/shuffle support. `005`, `006`, and `strlen` are covered.
- `tests/lang/024.phpt`: dynamic-variable array-offset lvalue blocker.

## Verification

Post-rebase verification on `341f09d28c9f`: `cargo fmt --check`,
`git diff --check origin/master..HEAD`, and full `cargo test` pass, including
native/compiler 481/481 plus COW reducer/oracle suites. Bounded PHPT
`run-20260611T232729Z-manifest.log` is 197/200 with only `array/007`,
`strings/004`, and `tests/lang/024` failing. COW PHPT
`run-20260611T233905Z-*` is 29/29.

Follow-ups remain broad visibility/inheritance, typed/non-public/promoted
properties, interfaces/traits, namespaces, reflection, remaining magic
methods, first-class callables, destructors, broader resources/exceptions,
dynamic includes, heredoc interpolation, full unsupported-internal coverage,
remaining scalar offset-lvalues, assertion configuration, non-direct-variable
or non-numeric inc/dec, and broader foreach destructuring/reference targets.
