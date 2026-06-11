# PTN Progress

Refresh: 2026-06-11T17:33Z
Measured: `ptn-29og` rebased on `origin/master` at `0fcbc60be`.
`array_column()` support is integrated. `array_filter()` rejects unknown mode
values with the modeled PHP `ValueError`. Foreach non-array diagnostics spell
boolean operands as `false given` or `true given`. Shared deprecation
diagnostics emit the PHP-style leading blank-line separator. Declared public
non-static `__construct` methods now run during `new Class(...)` after property
defaults, using inherited public method lookup, ordinary `$this` binding,
positional/default arguments, and return cleanup. Arithmetic on non-numeric
strings or mixed array operands now raises catchable `TypeError` diagnostics
through shared runtime helper paths.

## RC Surface

The release-candidate compiler/runtime path covers parser/IR/C backend, boxed
values, variables/constants, strings, scalar operators, ordered arrays,
`foreach`, branch/loop/switch control flow, compile-time-resolved includes,
selected standard internals, COW/reference slices, top-level functions,
call-frame introspection, scalar type hints, bounded closures/callables,
`stdClass`, public class/object shells, direct public static properties,
public property writes/`??=`, inherited public methods, public constructor
dispatch, diagnostic filtering, catchable arithmetic `TypeError` boundaries,
plain heredoc/nowdoc literals, and string interpolation slices.

The RC demo corpus exercises scalar control flow, string internals, arrays plus
`array_combine`, `array_filter`, and `array_chunk`, top-level functions, public
class/object shells, direct static properties, and public property `??=`.

## Test Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 454 | 454 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 200 | 179 | 21 |
| PHPT Zend rows | 76 | 72 | 4 |
| PHPT ext/standard rows | 77 | 68 | 9 |
| PHPT tests/basic+func+lang | 45 | 37 | 8 |
| PHPT other rows | 2 | 2 | 0 |
| PHPT COW manifest | 29 | 29 | 0 |
| Post-merge COW gate | 25 | 25 | 0 |
| PHPT callback manifest | 2 | 2 | 0 |

## Frozen Failure Clusters

- `ptn-lrty.5`: 10 numeric/operator/scalar-offset rows remain after the
  non-numeric arithmetic TypeError slice.
- `ptn-lrty.3`: 6 array-internal rows; `array_column` is now covered.
- `ptn-lrty.4`: 4 string/output rows.
- `ptn-lrty.6` plus `ptn-r52`: 2 control-flow/foreach/lang rows remain after
  `foreachLoop.003.phpt` is covered.

## Post-RC Architecture

Explicit follow-up work: full visibility/inheritance semantics, typed or
non-public/promoted properties, interfaces/traits, namespaces, class constants,
reflection, magic methods beyond public `__construct`, old-style constructors,
destructors, broader static-property semantics, non-static callables beyond
bounded dispatch, object destructuring/`Traversable`, property compound
lvalues beyond public property `??=`, exceptions, resources, dynamic includes,
heredoc interpolation/flexible indentation, unsupported internals, exact 64-bit
operator/diagnostic parity, remaining scalar offset-lvalue parity, and
non-direct-variable or non-numeric inc/dec parity.

## Verification

Freeze evidence: bounded `manifest-20260611T172355Z.txt` (179/200), COW PHPT
`summary-20260611T160936Z.txt` (29/29), callback
`summary-20260611T161926Z.txt` (2/2), native smoke, and post-merge COW gate.
`ptn-qhla`, `ptn-en6v`, `ptn-dzgg`, `ptn-p0y1`, `ptn-lrty.8`, and `ptn-29og`
add the current array/foreach/deprecation/constructor/arithmetic rows. Focused
`ptn-29og` verification covers
`compile_arithmetic_rejects_non_numeric_operands_to_native_binary`; exact PHPT
rows include `add_004.phpt`, `add_006.phpt`, and `add_007.phpt`; final rebase
verification is `cargo fmt --check`, `git diff --check origin/master..HEAD`,
`cargo test`, `tools/run-native-smoke-matrix.sh`, and
`tools/run-post-merge-cow-gate.sh`.
