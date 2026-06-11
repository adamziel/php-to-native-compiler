# PTN Progress

Refresh: 2026-06-11T21:19Z
Measured: `ptn-k95f` rebased on `origin/master` at `eb820e5fd`.

Recent RC slices cover array/key canonicalization, foreach assignment targets,
catchable arithmetic/assertion boundaries, public `__call`, public
`__toString` string conversion, precision-driven float output, scalar
`var_dump()` spelling, inline HTML, and string-offset diagnostics.
`ptn-lrty.3` adds minimal resource values plus `fopen()`, `fclose()`,
`is_resource()`, and `array_key_exists()` parity for `null` deprecation and
resource-key integer casting. `ptn-icd9` adds PHP-style scalar float exponent
spelling for echo/casts/concatenation/string internals, so
`tests/lang/operators/add_basiclong_64bit.phpt` passes. `ptn-loyg` adds
`ksort()` and `shuffle()` direct array mutation plus `str_shuffle()` byte
shuffling; `ext/standard/tests/strings/004.phpt` now reaches array-element
inc/dec after sort/shuffle support. `ptn-xery` adds variable-root array-path
support for cursor-moving internals and one-argument `array_pop()`/
`array_shift()`, clearing `ext/standard/tests/array/005.phpt`. `ptn-k95f` adds
literal-array function defaults, modeled `pow()`, and plain `array_merge()`
integer-key append/reindex plus string-key overwrite semantics, clearing
`ext/standard/tests/array/001.phpt`.

Exact `strlen.phpt` gets the object `__toString()` length right; it still
fails on ordering/source-path parity for the `${str}` deprecation versus the
undefined `$strS` warning.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 471 | 471 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 200 | 189 | 11 |
| PHPT Zend rows | 76 | 76 | 0 |
| PHPT ext/standard rows | 77 | 71 | 6 |
| PHPT tests/basic+func+lang | 45 | 40 | 5 |
| PHPT other rows | 2 | 2 | 0 |
| PHPT COW manifest | 29 | 29 | 0 |
| Post-merge COW gate | 25 | 25 | 0 |

## RC Surface

Parser/IR/C backend, boxed values, variables/constants, strings, scalar
operators, ordered arrays, `foreach`, branch/loop/switch, compile-time
includes, selected internals, COW/reference slices, top-level functions,
call-frame introspection, scalar type hints including literal-array defaults,
bounded closures/callables, `stdClass`, public class/object shells, direct
public static properties, public property writes/`??=`, inherited public
methods, public constructors, public `__call`, public `__toString`,
`is_callable()`, assertion errors, heredoc/nowdoc literals, string
interpolation slices, variable-root array-path cursor and single-pop/shift
mutation, modeled `pow()`, plain `array_merge()`, and direct array mutators
including `array_pop`, `array_push`, `array_shift`, `array_unshift`, `ksort`,
and `shuffle`.

## Remaining Bounded Failures

- `ptn-lrty.3`, `ptn-xery`, and `ptn-k95f`: 2 broad array-internal rows remain:
  `007` and `008`. `001`, `005`, `array_column()`, and `array_key_exists()`
  variants are covered.
- `ptn-lrty.4` plus `ptn-loyg`: 4 string/output rows remain. `004` now reaches
  array-element inc/dec after sort/shuffle support; `005`, `006`, and `strlen`
  diagnostic-order/source-path parity remain.
- `ptn-lrty.6` plus `ptn-r52`: `tests/lang/024.phpt` remains at the
  dynamic-variable array-offset lvalue blocker.
- `ptn-lrty.5`: 4 64-bit bitwise operator rows remain after object/array add
  diagnostics, `add_variationStr`, and `add_basiclong_64bit` are covered.

## Verification

Evidence: exact target PHPT rows `array_key_exists.phpt`,
`array_key_exists_variation1.phpt`, and
`array_key_exists_null_deprecation.phpt` pass 3/3 for `ptn-lrty.3`; exact
`tests/lang/operators/add_basiclong_64bit.phpt` passes for `ptn-icd9`; exact
`ext/standard/tests/strings/004.phpt` advances to array-element inc/dec for
`ptn-loyg`; exact `ext/standard/tests/array/005.phpt` passes for `ptn-xery`;
exact `ext/standard/tests/array/001.phpt` passes for `ptn-k95f`. `ptn-lrty.3`
bounded PHPT `summary-20260611T201728Z.txt` is 186/200, and `ptn-icd9` bounded
PHPT `summary-20260611T202054Z.txt` is 186/200 on its pre-merge base, covering
the complementary operator row. `ptn-xery` bounded PHPT
`summary-20260611T202534Z.txt` is 186/200 on its pre-merge base. `ptn-k95f`
bounded PHPT `summary-20260611T203030Z.txt` is 186/200 on its pre-merge base.
COW PHPT `summary-20260611T202912Z.txt`, `summary-20260611T203045Z.txt`, and
`summary-20260611T203453Z.txt` are 29/29; `ptn-k95f` COW PHPT
`run-20260611T204003Z.log` is 29/29; callback PHPT
`summary-20260611T203116Z.txt` is 2/2. Focused `ptn-k95f` coverage includes
array function parameter defaults, `compile_user_function_array_default_arguments_to_native_binary`,
`compile_pow_function_to_native_binary`, and `compile_array_merge_to_native_binary`.

Follow-ups remain broad visibility/inheritance, typed/non-public/promoted
properties, interfaces/traits, namespaces, reflection, remaining magic methods,
first-class callables, destructors, broader resources/exceptions, dynamic
includes, heredoc interpolation, full unsupported-internal coverage, exact
64-bit bitwise operator parity, scalar offset-lvalues, assertion configuration,
non-direct-variable or non-numeric inc/dec, and broader foreach destructuring/
reference targets.
