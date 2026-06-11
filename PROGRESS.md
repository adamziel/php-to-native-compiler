# PTN Progress

Refresh: 2026-06-11T16:47Z
Measured: `ptn-dzgg` rebased on `origin/master` at `ff7801200`.
`array_column()` support is integrated. `array_filter()` rejects unknown mode
values with the modeled PHP `ValueError`. Foreach non-array diagnostics now
spell boolean operands as `false given` or `true given` instead of `bool given`.

## RC Surface

The release-candidate compiler/runtime path covers parser/IR/C backend, boxed
values, variables/constants, strings, scalar operators, ordered arrays,
`foreach`, branch/loop/switch control flow, compile-time-resolved includes,
selected standard internals, COW/reference slices, top-level functions,
call-frame introspection, scalar type hints, bounded closures/callables,
`stdClass`, public class/object shells, direct public static properties,
public property writes/`??=`, inherited public methods, diagnostic filtering,
plain heredoc/nowdoc literals, and string interpolation slices.

The demo corpus exercises scalar control flow, string internals, arrays plus
`array_combine`, `array_filter`, and `array_chunk`, top-level functions, public
class/object shells, direct static properties, and public property `??=`.

## Test Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 451 | 451 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 200 | 174 | 26 |
| PHPT Zend rows | 76 | 69 | 7 |
| PHPT ext/standard rows | 77 | 67 | 10 |
| PHPT tests/basic+func+lang | 45 | 37 | 8 |
| PHPT other rows | 2 | 2 | 0 |
| PHPT COW manifest | 29 | 29 | 0 |
| Post-merge COW gate | 25 | 25 | 0 |
| PHPT callback manifest | 2 | 2 | 0 |

## Frozen Failure Clusters

- `ptn-lrty.5`: 13 numeric/operator/scalar-offset rows.
- `ptn-lrty.3`: 6 array-internal rows; `array_column` is now covered.
- `ptn-lrty.4`: 4 string/output rows.
- `ptn-lrty.6` plus `ptn-r52`: 2 remaining control-flow/foreach/lang rows
  after `foreachLoop.003.phpt` is covered by `ptn-dzgg`.

## Post-RC Architecture

Explicit follow-up work: full classes and inheritance, constructors/destructors,
typed or non-public properties, interfaces/traits, namespaces, class constants,
reflection, magic methods, broader static-property semantics, non-static
callables beyond direct object calls and bounded `[$object, "method"]`,
object destructuring, object `Traversable`, property compound lvalues beyond
public property `??=`, static-property compound/null-coalescing lvalues,
exceptions, resources, dynamic include/include_once behavior, heredoc
interpolation/flexible indentation, unsupported internals, exact 64-bit
operator/diagnostic parity, and remaining scalar offset-lvalue parity.

## Verification

Freeze evidence: bounded `summary-20260611T161121Z.txt` (173/200), COW PHPT
`summary-20260611T160936Z.txt` (29/29), callback
`summary-20260611T161926Z.txt` (2/2), native smoke, and post-merge COW gate.
`ptn-qhla` adds `array_column_numeric_string_key.phpt`, bringing bounded to
174/200. `ptn-lrty.7` reran `cargo fmt --check`, `cargo build --bin phpc`, the
documented README demo loop, and `cargo test`. `ptn-en6v` verification covered
`compile_array_filter_to_native_binary`, exact `array_filter_basic.phpt`,
`cargo test`, native smoke, and post-merge COW gate. Focused `ptn-dzgg`
verification covers `compile_foreach_non_array_diagnostics_include_source_path_to_native_binary`
and exact `foreachLoop.003.phpt`; the final rebase verification is
`cargo fmt --check`, `git diff --check origin/master..HEAD`, `cargo test`,
`tools/run-native-smoke-matrix.sh`, and `tools/run-post-merge-cow-gate.sh`.
