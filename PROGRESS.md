# PTN Progress

Refresh: 2026-06-13T05:47Z
Measured: `ptn-8bwi` rebased on current `origin/master` `507a48131`, with focused classifier evidence for bounded `display_errors`/`zend.assertions` ini rows.

Recent RC slices cover class constants, embedded-NUL `var_export()`,
`explode()`, `strncmp()`, `strrchr()`, namespaces/imports, includes/once
guards, closures, `stdClass`, properties/destructors/reflection, inherited
static dispatch, `property_exists()` metadata, array mutators/set/sort/udiff
helpers, `array_sum()`/`array_product()` warnings and overflow, `json_encode()`,
`printf()`/`sprintf()`, `basename()`, `pathinfo()`, `file_get_contents()`,
`strcasecmp()`, string search/slice/count internals, scalar `str_replace()`
counts/TypeErrors, `chr()` diagnostics, `crc32()`, standard streams, foreach
list destructuring, dynamic-variable writes/unsets and `??=`, stream metadata,
and offset compound/coalescing.

Recent movers include binary-safe `strpos()`/`stripos()`,
`strrpos()`/`strripos()`, `strstr()`/`stristr()`, and `substr_count()` with PHP
offset bounds, PHP 8.4 `array_sum()`/`array_product()` unsupported-type
warnings and overflow promotion, persistent `STDIN`/`STDOUT`/`STDERR`,
binary-safe `pathinfo()` with `PATHINFO_*` flags, `property_exists()` metadata,
broad PHPT manifests, length-aware `crc32()`, scalar `str_replace()` counts,
`strncmp()`/`strrchr()`, `basename()`, and stream/bounded-ini
preclassification.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 583 | 583 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 262 | 262 | 0 |
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
inc/dec, foreach destructuring, dynamic-variable writes/unsets, and
array/string-offset compound/null coalescing assignments.

## Remaining Bounded Failures

- None among the 262 runnable rows in the current bounded manifest.

## Verification

2026-06-13T05:19Z: passed diff check, `cargo fmt`, focused native
string-internal 1/1, focused search PHPT 5/5, `cargo test` 583/583 plus
COW/doc tests, bounded PHPT 260/260 with 2 excluded, PHPT COW 29/29, and
post-merge COW 26/26.

2026-06-13T05:47Z: rebased `ptn-8bwi` on `origin/master` `507a48131`;
`cargo fmt --check`, PHPT classifier shell syntax, and focused
`display_errors`/`zend.assertions` rows 2/2 passed with classification enabled.
Together with the upstream bounded PHPT 260/260 evidence, the current bounded
manifest has 262 selected runnable rows and 0 excluded. PHPT COW passed 29/29,
and post-merge COW gate passed 17 oracle, 3 notice, and 6 diagnostic cases.

Follow-ups remain visibility/exception/reference/global edges, typed/promoted
properties, interfaces/traits, bracketed/grouped namespaces, broader
fallback/reflection, magic methods, first-class callables, dynamic includes,
unsupported internals, scalar offset-lvalues, assertion config, binary-safe
array keys, append-form `??=`, embedded-NUL internals, object IDs,
locale constants/`setlocale()`, `str_replace()` array forms, and
object/reference targets.
