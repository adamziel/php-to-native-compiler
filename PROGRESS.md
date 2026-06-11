# PTN Progress

Refresh: 2026-06-11T16:19Z
Measured: `48da1fca0dc9` on `polecat/rc-01/ptn-lrty.1-rc`, rebased on
`origin/master` at `7d642e99b`.

## Test Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 450 | 450 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 200 | 173 | 27 |
| PHPT Zend rows | 76 | 69 | 7 |
| PHPT ext/standard rows | 77 | 66 | 11 |
| PHPT tests/basic+func+lang | 45 | 36 | 9 |
| PHPT other rows | 2 | 2 | 0 |
| PHPT COW manifest | 29 | 29 | 0 |
| Post-merge COW gate | 25 | 25 | 0 |
| PHPT callback manifest | 2 | 2 | 0 |

## Frozen Failure Clusters

- `ptn-lrty.5`: 13 numeric/operator/scalar-offset rows: mixed `+` coercions,
  `ZEND_POW_ASSIGN`, nested scalar offset compound write, and 64-bit add/bitwise
  exactness.
- `ptn-lrty.3`: 7 array-internal rows: `array_merge`/`array_walk`,
  `array_shift`, `array_diff`, `array_intersect`, `array_column`, and
  `array_key_exists` variants.
- `ptn-lrty.4`: 4 string/output rows: `shuffle`/`str_shuffle`,
  `highlight_string`, `highlight_file`, and `strlen` parity.
- `ptn-lrty.6` plus `ptn-r52`: 3 control-flow/foreach/lang rows:
  `tests/lang/024.phpt`, `foreachLoop.003`, and `foreachLoop.004`.

## Verification

Commands run: `cargo fmt --check`; `cargo build --bin phpc`; `cargo test`;
`tools/run-native-smoke-matrix.sh`; `tools/run-post-merge-cow-gate.sh`;
`tools/run-bounded-phpt.sh tools/phpt-cow-manifest.txt`;
`tools/run-bounded-phpt.sh tools/phpt-bounded-manifest.txt`;
`tools/run-bounded-phpt.sh tools/phpt-callback-manifest.txt`.

Evidence logs: bounded `summary-20260611T161121Z.txt` (173/200), COW PHPT
`summary-20260611T160936Z.txt` (29/29), callback
`summary-20260611T161926Z.txt` (2/2). `cargo fmt`, `cargo build`, `cargo test`,
native smoke, and post-merge COW gate passed.
