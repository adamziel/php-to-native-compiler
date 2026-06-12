# PTN Progress

Refresh: 2026-06-12T15:23Z
Measured: `ptn-n1tv` rebased after `origin/master` `d17d760bd`.

Recent RC slices cover direct static-property null coalescing assignment,
include-once guards for compiled include helpers, property/static-property
inc/dec, dynamic-variable array/string-offset writes and unsets, array/append
compound assignments, bounded private properties, object `var_export()`,
`get_class()`, quiet property/static probes, direct array mutators including
the sort family, explicit regular sort flags, set operations, `array_udiff*()`,
exact string/lang rows, highlight output paths, `join()`/`implode()`,
`sprintf()`, `array_product()`, key helpers, `array_search()`,
`array_slice()`, `array_pad()`, `count()`/`sizeof()` normal and recursive
modes, `str_pad()`, catchable `intdiv()` plus unsupported-operand TypeErrors,
catchable `chunk_split()` length `ValueError`, ASCII case/trim reducers,
length-aware `chunk_split()` empty-input endings, modeled PHP/CLI/Zend
metadata, PHPT runner environment probes, `ceil()`/`floor()` diagnostics,
`is_countable()`, `chr()` finite float/float-string precision diagnostics,
first `ReflectionFunction` metadata backed by generated user-function metadata
plus the internal registry, unbracketed namespaces, foreach list destructuring
with reference elements, and bounded dynamic include/require path dispatch.

Recent movers include compiled `include_once`/`require_once` reducers,
static-property null coalescing assignment, dynamic-root offset writes/unsets,
property/static quiet probes and inc/dec, direct sort-family mutators and flag
diagnostics, array key/search/slice/pad helpers, shared `count()`/`sizeof()`
optional-mode dispatch, `COUNT_NORMAL`/`COUNT_RECURSIVE` constants, recursive
array counting with recursion warnings, invalid-mode `ValueError`s,
byte-length `str_pad()` modes and `STR_PAD_*` constants, catchable `intdiv()`
unsupported-operand TypeErrors, catchable `chunk_split()` length diagnostics,
ASCII case and trim reducers, `chunk_split()` empty-input binary endings,
`phpversion()`, `php_sapi_name()`, `zend_version()`, `PHP_VERSION`, `PHP_SAPI`,
`PHP_OS`, `PHP_SHLIB_SUFFIX`, `get_loaded_extensions()`, `extension_loaded()`,
`ini_get()`, `get_cfg_var()`, `php_ini_scanned_files()`, `php_uname()`,
`realpath()`, `scandir()`, `preg_match()`, `str_replace()`, `chr()` call-site
precision deprecations, `ceil()`/`floor()` `TypeError` parity, namespace PHPT
rows `ns_001`, `ns_002`, `ns_003`, `ns_014`, foreach destructuring paths,
include PHPT rows, `ext/standard/tests/array/count_basic`,
`ext/standard/tests/array/sizeof_basic2`, `ext/standard/tests/strings/str_pad`,
`ext/standard/tests/strings/chunk_split_variation7`, ReflectionFunction names,
namespace/short-name probes, internal/user flags, parameter counts, variadic
status, and manifest tooling that accepts `-` stdin plus readable fd-backed
process-substitution inputs.

## Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 557 | 557 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 218 | 218 | 0 |
| PHPT Zend rows | 80 | 80 | 0 |
| PHPT ext/standard rows | 91 | 91 | 0 |
| PHPT tests/basic+func+lang | 45 | 45 | 0 |
| PHPT other rows | 2 | 2 | 0 |
| PHPT COW manifest | 29 | 29 | 0 |
| Post-merge COW gate | 26 | 26 | 0 |
| PHPT callback manifest | 4 | 4 | 0 |
| PHPT include manifest | 2 | 2 | 0 |

## RC Surface

Parser/IR/C backend, boxed values, variables/constants, strings, scalar
operators, ternary expressions, ordered arrays, `foreach`, branch/loop/switch,
compile-time includes, bounded include/require dispatch, selected internals,
COW/reference slices, user functions, call-frame introspection, scalar type
hints, bounded closures, `stdClass`, public class/object shells, declared
properties, quiet probes, metadata intrinsics, `ReflectionFunction` metadata
probes, `is_callable()`, `is_countable()`, assertions, interpolation,
unbracketed namespaces, simple imports, streams, modeled PHP/CLI/Zend version
metadata, PHPT runner environment probes, `get_loaded_extensions()`, `pow()`,
`array_merge()`, `array_pad()`, `count()`/`sizeof()`, `str_pad()`,
`array_slice()`, `strrev()`, first-byte case helpers, trim-family internals,
length-aware `chunk_split()` empty-input endings, catchable `chunk_split()`
length errors, `chr()` call-site precision deprecations, `array_search()`,
`call_user_func_array()`, catchable `intdiv()` integer-operand TypeErrors,
highlight output paths, `var_export()`, direct array mutators including
`natcasesort()`, explicit regular sort flags, set operations, inc/dec,
foreach destructuring, dynamic-variable array/string-offset writes, direct
static-property null coalescing assignment, include helpers sharing caller file
scope, return values, and once guards, and CLI/extension metadata probes used
by the PHPT runner.

## Remaining Bounded Failures

- None in the current 218-row bounded manifest.

## Verification

Verification: recent slices added focused `include_once`/`require_once` native
reducer coverage, focused static-property `??=` parser/native coverage,
`array_search()`, `natcasesort()`, `ceil()`/`floor()` diagnostics,
`is_countable()`, `ucfirst()`, `lcfirst()`, `array_pad()`, `array_slice()`,
catchable `chunk_split()` length `ValueError`, explicit regular sort flags,
trim-family byte charlists, namespace parser/resolver PHPT rows 4/4, foreach
list destructuring/reference-element coverage, include PHPT manifest coverage
2/2, modeled `str_pad()` plus `ext/standard/tests/strings/str_pad.phpt`,
`chunk_split("", ..., "|".chr(0))` plus
`ext/standard/tests/strings/chunk_split_variation7.phpt`, focused `intdiv()`
unsupported-operand `TypeError` coverage, callback/reflection manifest
coverage 4/4, stdin/process-substitution manifest smoke checks, focused
PHP/CLI/Zend metadata registry and redeclaration coverage, focused `chr()`
precision-deprecation call-site coverage, focused `count()`/`sizeof()`
mode/alias coverage, and native runner-probe coverage for INI metadata,
extensions, directory scans, and regex captures.

Follow-ups remain visibility/inheritance metadata, typed/promoted properties,
interfaces/traits, bracketed/grouped namespace forms, namespace fallback
parity, broader reflection classes/parameters/methods, magic methods,
first-class callables, destructors, dynamic includes outside bounded path sets,
unsupported internals, scalar offset-lvalues, assertion config, binary-safe
array keys, inc/dec Unicode/reference/COW/diagnostics, object IDs/visibility,
broader `chr()` unsupported-type diagnostics, and broader foreach/object/
reference targets.
