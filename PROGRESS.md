# PTN Progress

Refresh: 2026-06-11T19:25Z
Measured: `ptn-ttud` rebased on `origin/master` at `13494af73`.
`array_column()` support is integrated. `array_filter()` rejects unknown mode
values with the modeled PHP `ValueError`. Foreach non-array diagnostics spell
boolean operands as `false given` or `true given`. Shared deprecation
diagnostics emit the PHP-style leading blank-line separator. Declared public
non-static `__construct` methods now run during `new Class(...)` after property
defaults, using inherited public method lookup, ordinary `$this` binding,
positional/default arguments, and return cleanup. Arithmetic on non-numeric
strings or mixed array operands now raises catchable `TypeError` diagnostics
through shared runtime helper paths, and unsupported arithmetic operand
diagnostics use concrete object/exception/closure class names. Inline HTML
before, between, and after PHP blocks tokenizes/parses as output and lowers
through the shared echo path. Common string/byte internals share modeled
string-argument TypeErrors for array/object/closure/exception operands while
preserving scalar, null, and embedded-NUL paths. Nested string-offset assign-op
diagnostics distinguish single-offset assign-op from nested
string-offset-as-array access, and single-offset string assign-op now runs the
shared string-offset key diagnostic path before throwing.
`Zend/tests/offset_assign.phpt`, `Zend/tests/add_002.phpt`, and
`Zend/tests/add_003.phpt` pass. Public `__call` fallback is wired for missing
direct object methods and supported object callable dispatch, and
`is_callable()` validates the current string, closure, static-array, and
object-array callable subset. `phpc -d precision=N` drives scalar float
stringification for generated native execution, and `var_dump()` now uses
PHP-style fixed spelling for integer-valued finite floats below `1e17` while
preserving exponent spelling for large and non-integer exponent cases.
`assert()` is modeled as an internal that returns `true` for truthy assertions
and throws catchable `AssertionError` with compiler-generated default assertion
text for one-argument direct calls. Foreach key/value bindings now use generic
assignment-target storage for direct variables and array dimensions, so
`tests/lang/foreachLoop.004.phpt` passes. Exact `strlen.phpt` still fails on
object `__toString` and interpolation diagnostic ordering.

## RC Surface

The release-candidate compiler/runtime path covers parser/IR/C backend, boxed
values, variables/constants, strings, scalar operators, ordered arrays,
`foreach`, branch/loop/switch control flow, compile-time-resolved includes,
selected standard internals, COW/reference slices, top-level functions,
call-frame introspection, scalar type hints, bounded closures/callables,
`stdClass`, public class/object shells, direct public static properties,
public property writes/`??=`, inherited public methods, public constructor
dispatch, diagnostic filtering, catchable arithmetic `TypeError` boundaries
with object-class operand names, string-internal argument `TypeError`
boundaries, inline HTML output across PHP blocks, string-offset assign-op
diagnostics, foreach assignment-target bindings, public `__call` fallback for
object calls/callables, `is_callable()` subset validation,
`phpc -d precision=N` scalar float stringification, current scalar
`var_dump()` float spelling, `assert()`/`AssertionError`, plain heredoc/nowdoc
literals, and string interpolation slices.

The RC demo corpus exercises scalar control flow, string internals, arrays plus
`array_combine`, `array_filter`, and `array_chunk`, top-level functions, public
class/object shells, direct static properties, and public property `??=`.

## Test Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 463 | 463 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 200 | 185 | 15 |
| PHPT Zend rows | 76 | 76 | 0 |
| PHPT ext/standard rows | 77 | 68 | 9 |
| PHPT tests/basic+func+lang | 45 | 39 | 6 |
| PHPT other rows | 2 | 2 | 0 |
| PHPT COW manifest | 29 | 29 | 0 |
| Post-merge COW gate | 25 | 25 | 0 |
| PHPT callback manifest | 2 | 2 | 0 |

## Frozen Failure Clusters

- `ptn-lrty.5`: 5 numeric/operator rows remain after `zend-pow-assign`,
  `offset_assign`, `add_variationStr`, and object/array add diagnostics are
  covered.
- `ptn-lrty.3`: 5 array-internal rows remain: `001`, `005`, `007`, `008`, and
  `array_key_exists_variation1`.
- `ptn-lrty.4`: 4 string/output rows remain: `004`, `005`, `006`, and
  `strlen`.
- `ptn-lrty.6` plus `ptn-r52`: 1 control-flow/lang row remains after
  `foreachLoop.003.phpt` and `foreachLoop.004.phpt` are covered;
  `tests/lang/024.phpt` now reaches the dynamic-variable array-offset lvalue
  blocker after inline HTML.

## Post-RC Architecture

Explicit follow-up work: full visibility/inheritance semantics, typed or
non-public/promoted properties, interfaces/traits, namespaces, class constants,
reflection, remaining magic methods (`__invoke`, `__callStatic`, property
hooks, destructors), first-class callable syntax, old-style constructors,
visibility-aware callable metadata beyond the public-only slice, object
destructuring/`Traversable`, broader static-property semantics, property
compound/static lvalues beyond public property `??=`, exceptions, resources,
dynamic include/include_once behavior, heredoc interpolation/flexible
indentation, unsupported internals, exact 64-bit operator/diagnostic parity,
remaining scalar offset-lvalue parity, non-direct-variable or non-numeric
inc/dec parity, remaining PHP float formatting edge cases outside the current
scalar `var_dump()` slice, advanced assertion configuration side effects
beyond the current direct-call expression text, and broader foreach
destructuring/reference target coverage.

## Verification

Evidence: bounded `summary-20260611T185418Z.txt` (182/200) plus exact
`tests/lang/operators/add_variationStr.phpt`,
`Zend/tests/ast/zend-pow-assign.phpt`, `Zend/tests/add_002.phpt`,
`Zend/tests/add_003.phpt`, and `tests/lang/foreachLoop.004.phpt`; COW PHPT
`manifest-20260611T191235Z.txt` (29/29), callback
`summary-20260611T161926Z.txt` (2/2), native smoke, and post-merge COW gate.
Earlier `ptn-lu3y` frontier evidence, bounded `summary-20260611T173724Z.txt`
(179/200) and COW `run-20260611T174736Z.log` (29/29), is superseded by the
current dashboard after subsequent RC slices. `ptn-qhla`, `ptn-en6v`,
`ptn-dzgg`, `ptn-p0y1`, `ptn-lrty.8`, `ptn-29og`, `ptn-lrty.6`,
`ptn-lrty.4`, `ptn-lrty.5`, `ptn-lrty.9`, `ptn-wk0a`, `ptn-3i3q`,
`ptn-v8dv`, `ptn-h1je`, `ptn-2sq2`, and `ptn-ttud` add the current array,
foreach, deprecation, constructor, arithmetic, inline-HTML, string-diagnostic,
string-offset diagnostic, magic-callable, `is_callable()`, float-precision,
`var_dump()` float, `assert()`, foreach assignment-target, and arithmetic
object-class diagnostic rows. Focused `ptn-ttud` verification covers
`compile_arithmetic_rejects_non_numeric_operands_to_native_binary` plus exact
`Zend/tests/add_002.phpt` and `Zend/tests/add_003.phpt`; final gates cover
`cargo fmt --check`, `git diff --check`, `cargo test`, native smoke matrix,
and post-merge COW gate.
