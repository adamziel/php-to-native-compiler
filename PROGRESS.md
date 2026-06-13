# PTN Progress

Refresh: 2026-06-13T06:08Z
Measured: `ptn-y7rf` rebased on current `origin/master` `eaa43afc9`.

Recent RC slices cover class constants, embedded-NUL `var_export()`,
`explode()`, `strncmp()`, `strrchr()`, namespaces/imports, includes/once
guards, closures, `stdClass`, properties/destructors/reflection, inherited
static dispatch, `property_exists()` metadata, array helpers, `json_encode()`,
`printf()`/`sprintf()`, `basename()`, `pathinfo()`, `file_get_contents()`,
`strcasecmp()`, string search/slice/count internals, `str_replace()` scalar
and bounded array operand counts/TypeErrors, `chr()` diagnostics, `crc32()`,
standard streams, foreach destructuring, dynamic-variable writes/unsets and
`??=`, stream metadata, keyword boolean tails after direct assignments, and
offset compound/coalescing.

Recent movers include binary-safe search/count internals with PHP offset
bounds, PHP 8.4 `array_sum()`/`array_product()` warnings and overflow
promotion, persistent `STDIN`/`STDOUT`/`STDERR`, binary-safe `pathinfo()`,
`property_exists()` metadata, PHPT manifests, keyword boolean `and`/`or`/`xor`
tails after direct assignment statements, length-aware `crc32()`, scalar and
bounded array `str_replace()` counts, and stream preclassification.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 584 | 584 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 262 | 260 | 2 |
| PHPT Zend rows | 82 | 82 | 0 |
| PHPT ext/standard rows | 130 | 130 | 0 |
| PHPT focused stream rows | 2 | 2 | 0 |
| PHPT tests/basic+func+lang | 45 | 45 | 0 |
| PHPT other rows | 5 | 5 | 0 |
| PHPT COW manifest | 29 | 29 | 0 |
| Post-merge COW gate | 26 | 26 | 0 |
| PHPT callback manifest | 4 | 4 | 0 |
| PHPT include manifest | 2 | 2 | 0 |

## Broad PHPT Baseline

`tools/run-phpt-baseline.sh` generates deterministic 1k/5k/10k broad manifests
from `Zend/tests`, `ext/standard/tests`, and core `tests`, records the corpus
revision, and treats pass/fail/skip/warn totals as measurement signal.

## RC Surface

Parser/IR/C backend, boxed values, variables/constants, strings, scalar
operators, ternary, ordered arrays, `foreach`, control flow, includes/once
guards, selected internals, COW/reference slices, user functions, call-frame
introspection, scalar plus `void` return hints, closures, `stdClass`,
class/object shells/constants, declared/static properties,
`property_exists()` metadata, inherited static method dispatch, public
destructors, reflection, assertions, namespaces/imports, streams and standard
stream constants, file reads/writes, array/string/numeric helpers through
`array_udiff*()`, `array_sum()`, `array_product()`, `json_encode()`,
`printf()`, `fdiv()`, `explode()`, `str_replace()`, `strcasecmp()`,
`strncmp()`, `strrchr()`, string search/slice/count internals, `pathinfo()`,
`crc32()`, `basename()`, `chr()` diagnostics, `var_export()`, array mutators,
inc/dec, foreach destructuring, dynamic-variable writes/unsets, direct
assignment statement keyword boolean tails, and array/string-offset
compound/null coalescing assignments.

## Remaining Bounded Failures

- None among the 260 runnable rows in the current 262-row bounded manifest.
  Two selected rows are classified out for unsupported ini requirements.

## Verification

2026-06-13T06:07Z: `ptn-9uzn` passed diff check, `cargo fmt`, focused
keyword-boolean assignment-tail parser/native tests 2/2, `cargo test` 584/584
plus COW/doc tests, bounded PHPT 260/260 with 2 excluded, PHPT COW 29/29, and
post-merge COW 26/26.

2026-06-13T06:08Z: bounded PHPT 260/260 with 2 excluded unsupported-ini rows
and PHPT COW 29/29 passed on `507a48131` before the `ptn-y7rf` code change.
After the `str_replace()` array operand change, `cargo fmt --check`, focused
native reducer 1/1, focused `str_replace_basic.phpt` 1/1, and full
`cargo test` 584/584 plus COW/doc tails passed. After rebasing onto
`eaa43afc9`, `cargo fmt --check`, focused native reducer 1/1, and focused
`str_replace_basic.phpt` 1/1 were rechecked.

Follow-ups remain visibility/exception/reference/global edges, typed/promoted
properties, interfaces/traits, bracketed/grouped namespaces, broader
fallback/reflection, magic methods, first-class callables, dynamic includes,
unsupported internals, scalar offset-lvalues, assertion config, binary-safe
array keys, append-form `??=`, embedded-NUL internals, object IDs,
locale constants/`setlocale()`, remaining `str_replace()` nested
object/reference array-element parity, and object/reference targets.
