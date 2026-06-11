# PTN Progress

Refresh: 2026-06-11T16:30Z
Measured: `ptn-qhla` rebased on `origin/master` at `4cc4c178b` after the
`ptn-lrty.1` release dashboard refresh. `array_column()` internal support is
integrated, including null/string/int column and index keys, numeric-string key
canonicalization, missing-column skips, append fallback for missing index keys,
and cloned result values for COW safety.

## Test Dashboard

| Format / source | Ported | Passing | Needs work |
| --- | ---: | ---: | ---: |
| Source unit tests | 3 | 3 | 0 |
| Native/compiler Rust suite | 451 | 451 | 0 |
| Native smoke matrix | 6 | 6 | 0 |
| PHPT bounded manifest | 200 | 174 | 26 |
| PHPT Zend rows | 76 | 69 | 7 |
| PHPT ext/standard rows | 77 | 67 | 10 |
| PHPT tests/basic+func+lang | 45 | 36 | 9 |
| PHPT other rows | 2 | 2 | 0 |
| PHPT COW manifest | 29 | 29 | 0 |
| Post-merge COW gate | 25 | 25 | 0 |
| PHPT callback manifest | 2 | 2 | 0 |

## Frozen Failure Clusters

- `ptn-lrty.5`: 13 numeric/operator/scalar-offset rows: mixed `+` coercions,
  `ZEND_POW_ASSIGN`, nested scalar offset compound write, and 64-bit add/bitwise
  exactness.
- `ptn-lrty.3`: 6 array-internal rows: `array_merge`/`array_walk`,
  `array_shift`, `array_diff`, `array_intersect`, and `array_key_exists`
  variants. The `array_column` row is now covered by `ptn-qhla`.
- `ptn-lrty.4`: 4 string/output rows: `shuffle`/`str_shuffle`,
  `highlight_string`, `highlight_file`, and `strlen` parity.
- `ptn-lrty.6` plus `ptn-r52`: 3 control-flow/foreach/lang rows:
  `tests/lang/024.phpt`, `foreachLoop.003`, and `foreachLoop.004`.

## Verification

Release dashboard commands run by `ptn-lrty.1`: `cargo fmt --check`;
`cargo build --bin phpc`; `cargo test`; `tools/run-native-smoke-matrix.sh`;
`tools/run-post-merge-cow-gate.sh`;
`tools/run-bounded-phpt.sh tools/phpt-cow-manifest.txt`;
`tools/run-bounded-phpt.sh tools/phpt-bounded-manifest.txt`;
`tools/run-bounded-phpt.sh tools/phpt-callback-manifest.txt`.

Evidence logs from `ptn-lrty.1`: bounded `summary-20260611T161121Z.txt`
(173/200), COW PHPT `summary-20260611T160936Z.txt` (29/29), callback
`summary-20260611T161926Z.txt` (2/2). `ptn-qhla` adds the
`array_column_numeric_string_key.phpt` pass, bringing the bounded dashboard to
174/200. Rebase verification for `ptn-qhla`: `cargo fmt --check`;
`git diff --check origin/master..HEAD`; `cargo test`;
`tools/run-native-smoke-matrix.sh`; `tools/run-post-merge-cow-gate.sh`.
