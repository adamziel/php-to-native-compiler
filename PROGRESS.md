# PTN Progress

Refresh: 2026-06-13T16:27Z
Measured: `ptn-nid9` key/callback array set-operation expansion on
`origin/master` `d4e5e187`; native reducer and focused array PHPT gates
passed.

Recent RC slices cover constants, inline HTML, includes/once guards, closures,
`stdClass`, properties/destructors, ReflectionFunction metadata, array helpers,
key-aware `array_diff*()`/`array_intersect*()` and user-comparator
`array_uintersect*()` helpers, `json_encode()`, formatted output,
`basename()`, `pathinfo()`, `dirname()` levels, search/count/string internals,
scalar/array type hints, ordered-array `str_replace()`, `str_split()`,
`nl2br()`, `strncasecmp()`, `strncmp()`, `strcmp()`, trim-family aliases,
`strpbrk()`, `array_unique()`, broad array/scalar/operator PHPT coverage,
unsupported-language PHPT preclassification, inherited invokable object
callables, callable-name output, static `__invoke` rejection, object clone
expressions, magic-method visibility warnings, `php_uname()` mode validation,
environment/include-path helpers, `getcwd()`/`chdir()`, PHPT runner ini
values, statement-form `(void)` casts, offset compound/coalescing, and
property/static-property compounds.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 645 | 645 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 400 | 400 | 0 |
| PHPT Zend rows | 94 | 94 | 0 |
| PHPT ext/standard rows | 223 | 223 | 0 |
| PHPT focused array key/callback set rows | 75 | 38 | 37 |
| PHPT focused stream rows | 2 | 2 | 0 |
| PHPT focused cwd rows | 2 | 2 | 0 |
| PHPT tests/basic+func+lang | 78 | 78 | 0 |
| PHPT other rows | 5 | 5 | 0 |
| PHPT COW manifest | 29 | 29 | 0 |
| Post-merge COW gate | 26 | 26 | 0 |
| PHPT callback manifest | 5 | 5 | 0 |
| PHPT include manifest | 2 | 2 | 0 |
| PHPT formatted string rows | 75 | 25 | 50 |

## RC Surface

Parser/IR/C backend, boxed values, variables/constants, strings, scalar
operators, arrays, `foreach`, control flow, includes/once guards, selected
internals, COW/reference slices, functions, closures, `stdClass`, class/object
shells/constants, properties, destructors, reflection, assertions,
namespace/import forms, streams, file reads/writes, array/string/numeric
helpers through key-aware and callback-aware set operations, expanded
array/string/operator PHPT variations, PHP metadata constants, scalar
conversions, `is_iterable()`, locale support, invokable object callables,
callable-name output, static `__invoke` validation, SPL object identity
intrinsics, object clone expressions, non-recursive `array_replace()`, runner
ini/include-path state, `getcwd()`/`chdir()`, formatted stream/output helpers,
statement-form `(void)`, array mutators, inc/dec, `global`, dynamic variables,
offset compound/null coalescing, and direct property/static-property compounds.

## Remaining Bounded Exclusions

- None among the 400 runnable rows in the current bounded manifest.
- None among the 5 callback/callable frontier rows in the current callback
  manifest.

## Verification

`ptn-nid9` verification: native
`compile_array_key_and_callback_set_operations_to_native_binary`,
`cargo fmt --check`, and focused array key/callback PHPT passed. The PHPT
selected 75 rows, ran 73, passed 38, failed 35, and excluded 2
unsupported-language.

Follow-ups remain typed properties, interfaces/traits, magic methods,
first-class callables, dynamic includes, unsupported internals,
scalar-offset lvalues, `Traversable`, remaining embedded-NUL internals,
host-locale parity, conversion diagnostics, exact formatter edge parity,
array callback diagnostic parity, and process execution.
