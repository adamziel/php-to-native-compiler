# PTN Progress

Refresh: 2026-06-12T20:04Z
Measured: `ptn-ocf5` rebased on current `origin/master` `d787cc16c`.

Recent RC slices cover public class constants, embedded-NUL `var_export()`,
`explode()`, `strrchr()`, namespaced internal fallback, static-property `??=`,
once guards, property/static inc/dec, dynamic-variable writes/unsets and `??=`,
array and string-offset compound/null coalescing assignments, private
properties, inherited parent-private slots, public `__destruct()` dispatch,
quiet probes, array mutators, sort flags, set operations, `array_udiff*()`,
`join()`/`implode()`, bounded `sprintf()`/`printf()`, `json_encode()`,
`array_is_list()`, `array_search()`, `array_slice()`, `array_pad()`,
`array_reverse()`, `count()`/`sizeof()`, `str_pad()`, `str_shuffle()`,
`strtr()`, `chunk_split()`, `file_get_contents()`, string-internal
object/closure given-type diagnostics, `abs()`/`sqrt()`/`fdiv()` TypeErrors,
ASCII case/trim, PHP/CLI/Zend metadata, `php_uname()`, `ReflectionFunction`,
namespaces, foreach list destructuring, dynamic include/require dispatch,
return-only `void` declarations, and file-stream `stream_get_meta_data()`
metadata.

Recent movers include bounded binary-safe `file_get_contents()` reads with
offset/length handling, length-aware scalar `strrchr()` support, file-stream
metadata arrays, and dynamic-root `??=` reducers.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 577 | 577 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 229 | 229 | 0 |
| PHPT Zend rows | 80 | 80 | 0 |
| PHPT ext/standard rows | 99 | 99 | 0 |
| PHPT focused stream rows | 2 | 2 | 0 |
| PHPT tests/basic+func+lang | 45 | 45 | 0 |
| PHPT other rows | 5 | 5 | 0 |
| PHPT COW manifest | 29 | 29 | 0 |
| Post-merge COW gate | 26 | 26 | 0 |
| PHPT callback manifest | 4 | 4 | 0 |
| PHPT include manifest | 2 | 2 | 0 |

## RC Surface

Parser/IR/C backend, boxed values, variables/constants, strings, scalar
operators, ternary, ordered arrays, `foreach`, control flow, includes/once
guards, selected internals, COW/reference slices, user functions, call-frame
introspection, scalar plus `void` return hints, closures, `stdClass`,
class/object shells/constants, declared/static properties including inherited
parent-private slots, public destructor dispatch, reflection,
callability/countability, assertions, namespaces/imports, streams and
file-stream metadata, file reads/writes, array/string/numeric helpers through
`array_udiff*()`, `array_is_list()`, `count()`/`sizeof()`, `json_encode()`,
`printf()`, `chunk_split()`, `fdiv()`, `explode()`, `strrchr()`, shared
string-internal diagnostics, highlight paths, `var_export()`, array mutators,
inc/dec, foreach destructuring, dynamic-variable writes/unsets, and
array/string-offset compound/null coalescing assignments.

## Remaining Bounded Failures

- None in the current 229-row bounded manifest.

## Verification

Current slice verification: `cargo fmt --check`; focused `file_get_contents()`
native test 1/1; focused file PHPT rows 2/2; full `cargo test` 577/577;
bounded PHPT manifest 229/229; PHPT COW manifest 29/29; post-merge COW gate
17/17 oracle, 3/3 notice, 6/6 diagnostics.

Follow-ups remain destructor visibility/exception/reference/global edges,
typed/promoted properties, interfaces/traits, bracketed/grouped namespaces,
broader fallback/reflection, magic methods, first-class callables, dynamic
includes, unsupported internals, scalar offset-lvalues, assertion config,
binary-safe array keys, class-constant edges, dynamic-variable by-reference
lvalues, append-form `??=`, remaining embedded-NUL internals, inc/dec
Unicode/reference/COW diagnostics, object IDs, broader `chr()`/`abs()` edges,
`sqrt()` non-finite edges, and object/reference targets.
