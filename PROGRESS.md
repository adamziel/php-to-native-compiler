# PTN Progress

Refresh: 2026-06-13T11:32Z
Measured: `ptn-7tqe` rebased on current `origin/master` `0fd3ca1cc`;
focused verification green.

Recent RC slices cover constants, embedded-NUL `var_export()`, includes/once
guards, closures, `stdClass`, properties/destructors, inherited static
dispatch, `property_exists()` metadata, array helpers, `json_encode()`,
`printf()`, `basename()`, `pathinfo()`, `dirname()` levels, `strcasecmp()`,
search/count internals, scalar `str_replace()`, `chr()` diagnostics, `crc32()`,
PHP version/build/platform metadata constants, standard streams, foreach
destructuring, `global` bindings, dynamic-variable writes/unsets, stream
metadata, locale constants and `setlocale()`, catchable arithmetic/operator
errors, alternate `<>` parsing, offset compound/coalescing, and length-aware
`str_split()` chunks.

Recent movers include `global` function-local binding to root globals, PHP
version/build/platform metadata constants, `dirname()` positive-level
traversal, `pathinfo()`, modeled `LC_*` constants, C/POSIX `setlocale()`,
search/count internals, PHP 8.4 array warning/overflow behavior, persistent
standard streams, `property_exists()`, PHPT manifests, `crc32()`,
`str_replace()` counts, `str_split()` promotion, and integer validation for
`chr()`, `intdiv()`, and file offsets.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 589 | 589 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 272 | 270 | 2 |
| PHPT Zend rows | 85 | 84 | 1 |
| PHPT ext/standard rows | 132 | 131 | 1 |
| PHPT focused stream rows | 2 | 2 | 0 |
| PHPT tests/basic+func+lang | 50 | 50 | 0 |
| PHPT other rows | 5 | 5 | 0 |
| PHPT COW manifest | 29 | 29 | 0 |
| Post-merge COW gate | 26 | 26 | 0 |
| PHPT callback manifest | 5 | 3 | 2 |
| PHPT include manifest | 2 | 2 | 0 |

## RC Surface

Parser/IR/C backend, boxed values, variables/constants, strings, scalar
operators, arrays, `foreach`, control flow, includes/once guards, selected
internals, COW/reference slices, functions, closures, `stdClass`,
class/object shells/constants, properties, destructors, reflection, assertions,
namespaces/imports, streams, file reads/writes, array/string/numeric helpers
through `array_udiff*()`, `json_encode()`, `printf()`, `fdiv()`, `explode()`,
`str_split()`, `str_replace()`, `strcasecmp()`, `strncmp()`, `strrchr()`,
`pathinfo()`, `dirname()` levels, `crc32()`, `basename()`, locale support, PHP
version/build/platform metadata constants, `var_export()`, array mutators,
inc/dec, `global` bindings, dynamic-variable writes/unsets, and offset
compound/null coalescing assignments.

## Remaining Bounded Exclusions

- None among the 270 runnable rows in the current 272-row bounded manifest.
  Two selected rows are classified out for unsupported ini requirements.
- Callback manifest has 3 runnable passing rows and 2 unsupported-extension
  exclusions.

## Verification

Current branch verification for `ptn-7tqe`: post-rebase `git diff --check`,
`cargo fmt --check`, focused native `str_split()` reducer 1/1, parser
modeled-internal guard 1/1, string-internal fast-path reducer 1/1, and focused
`str_split_basic.phpt` 1/1. PHPT COW passed 29/29 on the immediately prior
rebased base. Baseline evidence before the promotion: bounded PHPT on
`527a6eb99` passed 268/268 with 2 exclusions, and `str_split_basic.phpt`
failed because `str_split()` was undefined.

Follow-ups remain visibility/exception/reference/global edges, typed/promoted
properties, interfaces/traits, magic methods, first-class callables, dynamic
includes, unsupported internals, scalar offset-lvalues, assertions,
binary-safe array keys, embedded-NUL internals, object IDs, host-locale parity,
`str_replace()` array forms, and object/reference targets.
