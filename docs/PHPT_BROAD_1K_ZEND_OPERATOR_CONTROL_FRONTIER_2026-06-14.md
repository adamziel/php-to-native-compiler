# PHPT Broad 1k Zend Operator/Control Frontier: 2026-06-14

Issue: `ptn-ubqw`

This slice refreshed the broad 1k baseline and split out the remaining
non-assignment Zend runnable rows that are large enough to matter but not
credible as one implementation patch. A narrow generic implementation was made
for PHP double-quoted `\e` escapes and binary-safe array string keys; the
remaining 26-row operator/control/AST/assertion cluster is recorded as a
blocker map.

## Broad 1k Baseline

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-ubqw-baseline-before
```

Generated manifest:
`.runtime/ptn-ubqw-baseline-before/20260614T042048Z/phpt-baseline-1000.txt`

Classifier artifact:
`.runtime/phpt-progress/summary-20260614T042048Z.txt`

Corpus revision: `8c63ec400ce8e07c57a8d9499317b96a8beafb8b`

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 1,000 | 429 | 571 |

Top classifier buckets:

| Bucket | Rows |
| --- | ---: |
| `unsupported-language` | 281 |
| `unsupported-class-metadata` | 144 |
| `unsupported-request-input-ini` | 28 |
| `unsupported-extension` | 20 |
| `unsupported-diagnostics-runtime` | 18 |
| `unsupported-assertion-ini` | 17 |
| `unsupported-resource-limit-ini` | 15 |
| `sapi-behavior` | 13 |
| `unsupported-assertion-runtime` | 9 |

Runnable rows by broad source:

| Source | Runnable |
| --- | ---: |
| `ext/standard/tests` | 296 |
| `Zend/tests` | 117 |
| `tests/basic` | 16 |

The `Zend/tests` runnable set now splits as:

| Zend cluster | Rows | Notes |
| --- | ---: | --- |
| Assignment/reference lvalues | 32 | Already mapped by `PHPT_BROAD_1K_ZEND_ASSIGNMENT_FRONTIER_2026-06-14.md`. |
| Historical bug regressions | 37 | Mixed engine regressions; needs separate source-level triage. |
| Asymmetric visibility | 22 | Property visibility/set-scope diagnostics and references. |
| Operator/control/AST/assertion frontier | 26 | Focused by this slice. |

## Implemented Movement

The implementation fixes two generic runtime/parser gaps:

- double-quoted string escape `\e` now lowers to byte `0x1b`;
- array string-key transforms/renderers use stored key lengths rather than
  C-string termination for `array_change_key_case()`, `var_export()`, and
  `json_encode()`.

Focused PHPT evidence:

| Run | Row | Passed | Failed |
| --- | --- | ---: | ---: |
| Before final escape fix | `ext/standard/tests/array/array_change_key_case_variation8.phpt` | 0 | 1 |
| After | `ext/standard/tests/array/array_change_key_case_variation8.phpt` | 1 | 0 |

Artifacts:

- Before/failing focused artifact:
  `.runtime/phpt-progress/run-20260614T043744Z-manifest.log`
- After/passing focused artifact:
  `.runtime/phpt-progress/run-20260614T045117Z-manifest.log`

The first broad standard-array partial run also observed the same row failing
before this fix at `.runtime/phpt-progress/run-20260614T042619Z-manifest.log`.

## Focused Zend Manifest

Committed manifest:
`tools/phpt-zend-operator-control-frontier-manifest.txt`

Run:

```sh
tools/run-bounded-phpt.sh tools/phpt-zend-operator-control-frontier-manifest.txt
```

Result artifact:
`.runtime/phpt-progress/run-20260614T060540Z-manifest.log`

| Selected | Runnable | Passed | Failed | Excluded | Skipped | Warned |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 26 | 25 | 15 | 10 | 1 | 0 | 0 |

## Row Outcomes

| Row group | Rows | Outcome | Boundary |
| --- | ---: | --- | --- |
| `add_001` through `add_007` | 7 | pass | Array/object/string addition and numeric conversion are covered for these shapes. |
| `and_001` | 1 | pass | Bitwise string AND covered for this row. |
| `arrow_functions/001`, `002`, and `004` | 3 | pass | Basic arrow capture/autoglobal rows covered. |
| `arrow_functions/003` | 1 | excluded | Current classifier marks the variable-variable row `unsupported-language`. |
| `assert/*` | 2 | pass | These assertion crash/empty-statement rows stay green under current assertion subset. |
| `67468` and `ast/zend-pow-assign` | 2 | pass | Highlight string and pow-assign paths are covered. |
| `access_modifiers/access_modifiers_006` | 1 | fail | Multiple static access-modifier diagnostics need parser/class-member validation parity. |
| `ast/ast_serialize_backtick_literal`, `ast/ast_serialize_floats`, `ast/gh21072` | 3 | fail | AST serialization and constant-expression `(unset)` rejection need a real AST metadata surface. |
| `attributes/nodiscard/005` | 1 | fail | Internal `NoDiscard` native-method metadata remains outside the modeled attribute surface. |
| `binary` | 1 | fail | Binary literal/cast diagnostic parity remains incomplete. |
| `break_error_001` through `004` | 4 | fail | `break` level expression validation and loop-context diagnostics need parser/control-flow parity. |

## Recommended Splits

1. Keep the passing arithmetic, arrow-function, and assertion rows as regression
   coverage; do not spend another broad slice remapping them.
2. Split `break_error_*` into a parser/control-flow diagnostic patch. It is
   the largest coherent failing group in this manifest.
3. Split AST serialization separately from runtime operators. The failing AST
   rows need metadata/serialization work, not operator runtime patches.
4. Treat `NoDiscard` native-method metadata as classifier or metadata work
   alongside the broader internal-attribute frontier.

## Verification

```sh
cargo fmt --check
cargo test lexer_preserves_unknown_string_escape_backslashes
cargo test compile_double_quoted_byte_escapes_to_native_binary
cargo test compile_array_change_key_case_to_native_binary
cargo test compile_json_encode_and_printf_to_native_binary
cargo test compile_var_export_embedded_nul_strings_to_native_binary
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-ubqw-baseline-before
tools/run-bounded-phpt.sh .runtime/ptn-ubqw-array-change-key-case-manifest.txt
tools/run-bounded-phpt.sh tools/phpt-zend-operator-control-frontier-manifest.txt
```
