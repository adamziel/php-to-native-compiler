# PTN Progress

Refresh: 2026-06-13T15:41Z
Measured: `ptn-qsmv.4` broad ext/standard array PHPT expansion on
`origin/master` `310c965c`; focused PHPT and formatting gates passed after
conflict resolution.

Recent RC slices cover constants, inline HTML, includes/once guards, closures,
`stdClass`, properties/destructors, ReflectionFunction metadata, array helpers,
`json_encode()`, `printf()`, `basename()`, `pathinfo()`, `dirname()` levels,
search/count/string internals, scalar/array type hints, ordered-array
`str_replace()`, `str_split()`, `nl2br()`, `strncasecmp()`, `strpbrk()`,
`chop()`, `array_unique()`, broad `array_chunk()`/`array_filter()`/
`array_diff*()`/`array_intersect*()` PHPT coverage, inherited invokable object
callables, callable-name output for `is_callable()`, static `__invoke`
rejection, object clone expressions with public `__clone`, magic-method
visibility warnings, `php_uname()` mode validation, environment/include-path
helpers, `getcwd()`/`chdir()`, PHPT runner ini values, statement-form `(void)`
casts, offset compound/coalescing, and property/static-property compounds.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 641 | 641 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 343 | 343 | 0 |
| PHPT Zend rows | 89 | 89 | 0 |
| PHPT ext/standard rows | 199 | 199 | 0 |
| PHPT focused stream rows | 2 | 2 | 0 |
| PHPT focused cwd rows | 2 | 2 | 0 |
| PHPT tests/basic+func+lang | 50 | 50 | 0 |
| PHPT other rows | 5 | 5 | 0 |
| PHPT COW manifest | 29 | 29 | 0 |
| Post-merge COW gate | 26 | 26 | 0 |
| PHPT callback manifest | 5 | 5 | 0 |
| PHPT include manifest | 2 | 2 | 0 |

## RC Surface

Parser/IR/C backend, boxed values, variables/constants, strings, scalar
operators, arrays, `foreach`, control flow, includes/once guards, selected
internals, COW/reference slices, functions, closures, `stdClass`, class/object
shells/constants, properties, destructors, reflection, assertions,
namespace/import forms, streams, file reads/writes, array/string/numeric
helpers through `array_udiff*()`, `array_unique()`, expanded
`array_chunk()`/`array_filter()`/diff/intersect PHPT variations,
`json_encode()`, `printf()`, `fdiv()`, `explode()`, trim-family aliases
including `chop()`, `strpbrk()`, PHP metadata constants with `php_uname()`
mode validation, string/path/search helpers, scalar conversions,
`is_iterable()`, locale support, invokable object callables, callable-name
output, static `__invoke` validation, SPL object identity intrinsics, object
clone expressions, non-recursive `array_replace()`, runner ini/include-path
state, `getcwd()`/`chdir()`, statement-form `(void)`, array mutators, inc/dec,
`global`, dynamic variables, offset compound/null coalescing, and direct
property/static-property compounds.

## Remaining Bounded Exclusions

- None among the 343 runnable rows in the current bounded manifest.
- None among the 5 callback/callable frontier rows in the current callback
  manifest.

## Verification

Current branch verification for `ptn-qsmv.4`: the 59 newly added bounded
manifest rows, targeted array native reducers, and `cargo fmt --check` passed
after conflict resolution.

Follow-ups remain typed properties, interfaces/traits, magic methods,
first-class callables, dynamic includes, unsupported internals,
scalar-offset lvalues, `Traversable`, remaining embedded-NUL internals,
host-locale parity, conversion diagnostics, process execution, and callables.
