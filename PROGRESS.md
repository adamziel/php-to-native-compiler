# PTN Progress

Refresh: 2026-06-12T13:58Z
Measured: `ptn-ss5c` rebased after `origin/master` `be3938576`.

Recent RC slices cover property/static-property inc/dec, dynamic-variable
array/string-offset writes and unsets, array/append compound assignments,
bounded private properties, object `var_export()`, `get_class()`, quiet
property/static probes, direct array mutators including the sort family,
explicit regular sort flags, set operations, `array_udiff*()`, exact
string/lang rows, highlight output paths, `join()`/`implode()`, `sprintf()`,
`array_product()`, key helpers, `array_search()`, `array_slice()`,
`array_pad()`, `str_pad()`, catchable `intdiv()`, ASCII case/trim reducers,
length-aware `chunk_split()` empty-input endings, `ceil()`/`floor()`
diagnostics, `is_countable()`, unbracketed namespaces, foreach list
destructuring with reference elements, and bounded dynamic include/require path
dispatch.

Recent movers include dynamic-root offset writes/unsets, property/static quiet
probes and inc/dec, direct sort-family mutators and flag diagnostics, array
key/search/slice/pad helpers, byte-length `str_pad()` modes and `STR_PAD_*`
constants, catchable `intdiv()`, ASCII case and trim reducers, `chunk_split()`
empty-input binary endings, `ceil()`/`floor()` `TypeError` parity, namespace
PHPT rows `ns_001`, `ns_002`, `ns_003`, `ns_014`, foreach destructuring paths,
include PHPT rows, `ext/standard/tests/strings/str_pad`, and
`ext/standard/tests/strings/chunk_split_variation7`.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 548 | 548 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 216 | 216 | 0 |
| PHPT Zend rows | 80 | 80 | 0 |
| PHPT ext/standard rows | 89 | 89 | 0 |
| PHPT tests/basic+func+lang | 45 | 45 | 0 |
| PHPT other rows | 2 | 2 | 0 |
| PHPT COW manifest | 29 | 29 | 0 |
| Post-merge COW gate | 26 | 26 | 0 |
| PHPT callback manifest | 2 | 2 | 0 |
| PHPT include manifest | 2 | 2 | 0 |

## RC Surface

Parser/IR/C backend, boxed values, variables/constants, strings, scalar
operators, ternary expressions, ordered arrays, `foreach`, branch/loop/switch,
compile-time includes, bounded dynamic include/require path dispatch, selected
internals, COW/reference slices, user functions, call-frame introspection,
scalar type hints, bounded closures, `stdClass`, public class/object shells,
declared properties, quiet probes, metadata intrinsics, `is_callable()`,
`is_countable()`, assertions, interpolation, unbracketed namespaces, simple
imports, streams, `pow()`, `array_merge()`, `array_pad()`, `str_pad()`,
`array_slice()`, `strrev()`, first-byte case helpers, trim-family internals,
length-aware `chunk_split()` empty-input endings, `array_search()`,
`call_user_func_array()`, highlight output paths, `var_export()`, direct array
mutators including `natcasesort()`, explicit regular sort flags, set
operations, inc/dec, foreach destructuring, dynamic-variable
array/string-offset writes, and include helpers sharing caller file scope and
return values.

## Remaining Bounded Failures

- None in the current 216-row bounded manifest.

## Verification

Verification: recent slices added `array_search()`, `natcasesort()`,
`ceil()`/`floor()` diagnostics, `is_countable()`, `ucfirst()`, `lcfirst()`,
`array_pad()`, `array_slice()`, explicit regular sort flags, trim-family byte
charlists, namespace parser/resolver PHPT rows 4/4, foreach list
destructuring/reference-element coverage, include PHPT manifest coverage 2/2,
modeled `str_pad()` plus `ext/standard/tests/strings/str_pad.phpt`, and
`chunk_split("", ..., "|".chr(0))` plus
`ext/standard/tests/strings/chunk_split_variation7.phpt`.

Follow-ups remain visibility/inheritance metadata, typed/promoted properties,
interfaces/traits, bracketed/grouped namespace forms, namespace fallback
parity, reflection, remaining magic methods, first-class callables, destructors,
fully dynamic includes outside bounded path sets, `include_once`/`require_once`,
unsupported internals, scalar offset-lvalues, assertion configuration,
binary-safe array keys, inc/dec Unicode/reference/COW/diagnostic edges, object
metadata/IDs/visibility edges, and broader foreach/object/reference targets.
