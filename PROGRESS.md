# PTN Progress

Refresh: 2026-06-11T21:46Z
Measured: `ptn-z8jv` rebased on `origin/master` at `87b40b509`.

Recent RC slices cover array/key canonicalization, foreach targets, catchable
arithmetic/assertion boundaries, public `__call`/`__toString`, precision-driven
float output, scalar `var_dump()` spelling, inline HTML, string-offset
diagnostics, minimal resources, `array_key_exists()` null/resource-key parity,
PHP-style float exponent spelling, direct `ksort()`/`shuffle()`,
variable-root array-path cursor mutation, one-argument `array_pop()`/
`array_shift()`, literal-array defaults, `pow()`, `array_merge()`, bounded
`highlight_string()` output buffers, and `phpc -d error_reporting=N`. This
slice adds `call_user_func_array()` through shared callable dispatch.

Exact `strlen.phpt` gets object `__toString()` length right; it still fails on
ordering/source-path parity for the `${str}` deprecation versus the undefined
`$strS` warning.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 475 | 475 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 200 | 190 | 10 |
| PHPT Zend rows | 76 | 76 | 0 |
| PHPT ext/standard rows | 77 | 72 | 5 |
| PHPT tests/basic+func+lang | 45 | 40 | 5 |
| PHPT other rows | 2 | 2 | 0 |
| PHPT COW manifest | 29 | 29 | 0 |
| Post-merge COW gate | 25 | 25 | 0 |

## RC Surface

Parser/IR/C backend, boxed values, variables/constants, strings, scalar
operators, ordered arrays, `foreach`, branch/loop/switch, compile-time includes,
selected internals, COW/reference slices, top-level functions, call-frame
introspection, scalar type hints including literal-array defaults, bounded
closures/callables, `stdClass`, public class/object shells, public static
properties, public property writes/`??=`, inherited methods/constructors,
public `__call`, public `__toString`, `is_callable()`, assertion errors,
heredoc/nowdoc literals, string interpolation slices, variable-root array-path
cursor/single-pop/shift mutation, `pow()`, `array_merge()`,
`call_user_func_array()`, CLI `error_reporting` ini wiring, bounded
`highlight_string()`/empty output-buffer reads, and direct array mutators
through `shuffle`.

## Remaining Bounded Failures

- `ptn-lrty.3`, `ptn-xery`, and `ptn-k95f`: 2 broad array-internal rows remain:
  `007` and `008`. `001`, `005`, `array_column()`, and `array_key_exists()`
  variants are covered.
- `ptn-lrty.4`, `ptn-loyg`, and `ptn-qm7v`: 3 string/output rows remain. `004`
  reaches array-element inc/dec; `005` is covered; `006` and `strlen`
  diagnostic parity remain.
- `ptn-lrty.6` plus `ptn-r52`: `tests/lang/024.phpt` remains at the
  dynamic-variable array-offset lvalue blocker.
- `ptn-lrty.5`: 4 64-bit bitwise operator rows remain after object/array add
  diagnostics, `add_variationStr`, and `add_basiclong_64bit` are covered.

## Verification

Evidence: exact `array_key_exists*` PHPT rows pass 3/3 for `ptn-lrty.3`; exact
`tests/lang/operators/add_basiclong_64bit.phpt` passes for `ptn-icd9`; exact
`strings/004.phpt` advances for `ptn-loyg`; exact `array/005.phpt` passes for
`ptn-xery`; exact `array/001.phpt` passes for `ptn-k95f`; exact
`strings/005.phpt` passes for `ptn-qm7v`; exact
`call_user_func_array_variation_001.phpt` passes for this slice. COW PHPT
remains 29/29; callback PHPT remains 2/2. Focused native coverage checks
`call_user_func_array()` user, internal, and by-reference callbacks.
`cargo fmt --check`, `git diff --check`, focused PHPT, and full `cargo test`
pass locally.

Follow-ups remain broad visibility/inheritance, typed/non-public/promoted
properties, interfaces/traits, namespaces, reflection, remaining magic methods,
first-class callables, destructors, broader resources/exceptions, dynamic
includes, heredoc interpolation, full unsupported-internal coverage, exact
64-bit bitwise operator parity, scalar offset-lvalues, assertion configuration,
non-direct-variable or non-numeric inc/dec, and broader foreach destructuring/
reference targets.
