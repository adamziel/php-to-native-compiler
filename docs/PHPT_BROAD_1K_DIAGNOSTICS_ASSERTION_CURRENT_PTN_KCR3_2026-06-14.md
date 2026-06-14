# PHPT Broad 1k Diagnostics/Assertion Current Frontier: 2026-06-14 ptn-kcr3

Issue: `ptn-kcr3`

This slice refreshes the broad 1k diagnostics/assertion blocker map on current
`master`. It is a blocker map, not a runtime behavior change: these rows need
shared runtime services for stack-frame diagnostics, user handler state,
`ErrorException` metadata, assertion modes, and diagnostic INI channels.

## Broad 1k Evidence

Command:

```sh
PHPT_PROGRESS_DIR=.runtime/ptn-kcr3-1k-progress \
  tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-kcr3-1k
```

Generated broad manifest:

```text
.runtime/ptn-kcr3-1k/20260614T121538Z/phpt-baseline-1000.txt
```

Classifier artifacts:

```text
.runtime/ptn-kcr3-1k-progress/classification-20260614T121538Z.tsv
.runtime/ptn-kcr3-1k-progress/runnable-20260614T121538Z.txt
.runtime/ptn-kcr3-1k-progress/excluded-20260614T121538Z.tsv
```

State:

```text
PTN: dfa23a6f856e
php-src PHPT corpus revision: 8c63ec400ce8e07c57a8d9499317b96a8beafb8b
```

Broad result:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 1,000 | 424 | 576 |

## Focused Manifest

Committed manifest:

```text
tools/phpt-diagnostics-assertion-current-ptn-kcr3-manifest.txt
```

Selection from the broad classifier:

```sh
awk -F'\t' '$2 ~ /^(unsupported-diagnostics-runtime|unsupported-assertion-ini|unsupported-assertion-runtime|unsupported-diagnostics-ini)$/ {print $1}'
```

Focused replay:

```sh
PHPT_PROGRESS_DIR=.runtime/ptn-kcr3-diagnostics-assertion-focused \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-diagnostics-assertion-current-ptn-kcr3-manifest.txt
```

Focused artifacts:

```text
.runtime/ptn-kcr3-diagnostics-assertion-focused-final/classification-20260614T122618Z.tsv
.runtime/ptn-kcr3-diagnostics-assertion-focused-final/excluded-20260614T122618Z.tsv
```

Focused result:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 48 | 0 | 48 |

## Category Split

| Classification | Rows |
| --- | ---: |
| `unsupported-diagnostics-runtime` | 17 |
| `unsupported-assertion-ini` | 17 |
| `unsupported-assertion-runtime` | 9 |
| `unsupported-diagnostics-ini` | 5 |

Reason split:

| Runtime blocker | Rows |
| --- | ---: |
| Configurable `assert.exception` assertion mode | 17 |
| `debug_backtrace()`/`debug_print_backtrace()` stack-frame snapshots | 9 |
| User error/exception handler state and fallback dispatch | 6 |
| Runtime `zend.assertions` mode switching | 3 |
| Engine diagnostic/logging mode `fatal_error_backtraces` | 3 |
| Namespace-aware assertion resolution and diagnostics | 2 |
| `assert_options()` mode/callback state | 2 |
| `ErrorException` severity and trace metadata | 2 |
| Engine diagnostic/logging mode `error_log` | 1 |
| Engine diagnostic/logging mode `report_memleaks` | 1 |
| Assertion expression lvalue mode interaction | 1 |
| Assertion AST pretty-printing for closure expressions | 1 |

Path split:

| Path group | Rows |
| --- | ---: |
| `Zend/tests/assert` | 24 |
| `Zend/tests/backtrace` | 12 |
| Root `Zend/tests` | 8 |
| `tests/basic` | 2 |
| `Zend/tests/asymmetric_visibility` | 1 |
| `Zend/tests/arrow_functions` | 1 |

## Relation To Existing Maps

This is the current focused diagnostics/assertion set after the later
class-declaration, class-metadata, and attribute-metadata classifier splits.
It supersets `tools/phpt-diagnostics-assertion-frontier-manifest.txt` by one
row: `Zend/tests/ErrorException_getSeverity.phpt`. The current classifier
still keeps both `ErrorException` constructor/severity rows in
`unsupported-diagnostics-runtime`, and the focused replay confirms all 48 rows
remain classified on current `master`.

It remains a diagnostics/assertion subset of the broader 144-row runtime
boundary map in `tools/phpt-runtime-boundary-current-ptn-2qq9-manifest.txt`.

## Blocker Boundary

The 48 rows are above the broad-slice threshold, but they do not form a single
credible small implementation patch. Generic support needs shared runtime
architecture:

- stack frame storage for backtrace functions, includes, `$this`, arguments,
  limits, and trace string formatting;
- user error and exception handler state, including fallback and handler
  exception behavior;
- `ErrorException` metadata for severity, previous, file, line, code, and
  trace APIs;
- assertion runtime state for `assert.exception`, `zend.assertions`,
  `assert_options()`, callbacks, namespace resolution, and diagnostic
  rendering;
- process-global diagnostic INI channels for `fatal_error_backtraces`,
  `error_log`, and `report_memleaks`.

Until those services exist, keeping these rows classified prevents broad PHPT
telemetry from mixing unsupported diagnostics and assertion modes into ordinary
PHP semantic failures.

## Verification

```sh
PHPT_PROGRESS_DIR=.runtime/ptn-kcr3-1k-progress \
  tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-kcr3-1k
PHPT_PROGRESS_DIR=.runtime/ptn-kcr3-diagnostics-assertion-focused \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-diagnostics-assertion-current-ptn-kcr3-manifest.txt
cargo fmt --check
cargo test --test phpt_classifier
```
