# PTN Progress

Refresh: 2026-06-13T00:16Z
Measured: `ptn-jbqd` rebased on current `origin/master` `490a8c41e`.

Recent RC slices cover public class constants, embedded-NUL `var_export()`,
`explode()`, `strncmp()`, `strrchr()`, namespaced internal fallback,
static-property `??=`, once guards, property/static inc/dec, dynamic-variable
writes/unsets and `??=`, array and string-offset compound/null coalescing
assignments, private properties, inherited parent-private slots, public
`__destruct()` dispatch, quiet probes, array mutators, sort flags, set
operations, `array_udiff*()`, `join()`/`implode()`, bounded
`sprintf()`/`printf()`, `json_encode()`, `array_is_list()`, `array_search()`,
`array_slice()`, `array_pad()`, `array_reverse()`, `count()`/`sizeof()`,
`basename()`, `str_pad()`, `str_shuffle()`, `strtr()`, `chunk_split()`,
`file_get_contents()`, `strcasecmp()`, `chr()` null/non-finite/unsupported
operand diagnostics, string-internal object/closure given-type diagnostics,
`abs()`/`sqrt()`/`fdiv()` TypeErrors, ASCII case/trim, PHP/CLI/Zend metadata,
`php_uname()`, `ReflectionFunction`, namespaces, foreach list destructuring,
dynamic include/require dispatch, return-only `void` declarations, and
file-stream `stream_get_meta_data()` metadata.

Recent movers include `chr()` integer diagnostics, `strncmp()`/`strrchr()`,
`basename()`, `file_get_contents()`, `strcasecmp()`, streams, and dynamic-root
`??=` reducers.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 579 | 579 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 232 | 232 | 0 |
| PHPT Zend rows | 80 | 80 | 0 |
| PHPT ext/standard rows | 102 | 102 | 0 |
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
`printf()`, `chunk_split()`, `fdiv()`, `explode()`, `strcasecmp()`,
`strncmp()`, `strrchr()`, `basename()`, `chr()` integer-argument diagnostics,
shared string-internal diagnostics, highlight paths, `var_export()`, array
mutators, inc/dec, foreach destructuring, dynamic-variable writes/unsets, and
array/string-offset compound/null coalescing assignments.

## Remaining Bounded Failures

- None in the current 232-row bounded manifest.

## Verification

Current slice verification: `git diff --check`; `cargo fmt --check`;
focused `chr` 7/7 and `intdiv` 4/4; full `cargo test` 579/579 plus
auxiliary/doc tests; bounded PHPT 232/232; PHPT COW 29/29; post-merge COW
17/17 oracle, 3/3 notice, 6/6 diagnostics.

Follow-ups remain destructor visibility/exception/reference/global edges,
typed/promoted properties, interfaces/traits, bracketed/grouped namespaces,
broader fallback/reflection, magic methods, first-class callables, dynamic
includes, unsupported internals, scalar offset-lvalues, assertion config,
binary-safe array keys, class-constant edges, dynamic-variable by-reference
lvalues, append-form `??=`, remaining embedded-NUL internals, inc/dec
Unicode/reference/COW diagnostics, object IDs, strict/reference `chr()` edges,
broader numeric/string edges, locale constants/`setlocale()`, and
object/reference targets.
