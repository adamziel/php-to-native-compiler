# PHPT Broad 1k Diagnostics/Assertion Frontier: 2026-06-14

Issue: `ptn-cuev`

This slice refreshes the broad 1k diagnostics and assertion runtime frontier on
current `origin/master`. It is a blocker map, not a behavior change: the
remaining excluded rows need shared runtime state for stack traces, error
handlers, `ErrorException`, assertion modes, and diagnostic INI channels rather
than row-local expected output handling.

## Broad 1k Evidence

Source state:

- PTN: `5de799cfcd6b`
- php-src PHPT corpus: `8c63ec400ce8e07c57a8d9499317b96a8beafb8b`

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-cuev-baseline
```

Generated broad manifest:

```text
.runtime/ptn-cuev-baseline/20260614T060742Z/phpt-baseline-1000.txt
```

Classifier artifacts:

```text
.runtime/phpt-progress/classification-20260614T060743Z.tsv
.runtime/phpt-progress/runnable-20260614T060743Z.txt
.runtime/phpt-progress/excluded-20260614T060743Z.tsv
```

Broad classifier result:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 1,000 | 424 | 576 |

Top broad classifier buckets:

| Classification | Rows |
| --- | ---: |
| `unsupported-language` | 288 |
| `unsupported-class-metadata` | 143 |
| `unsupported-request-input-ini` | 28 |
| `unsupported-extension` | 20 |
| `unsupported-diagnostics-runtime` | 17 |
| `unsupported-assertion-ini` | 17 |
| `unsupported-resource-limit-ini` | 15 |
| `sapi-behavior` | 13 |
| `unsupported-assertion-runtime` | 9 |
| `unsupported-diagnostics-ini` | 5 |

## Focused Frontier

Focused manifest:

```text
tools/phpt-diagnostics-assertion-frontier-manifest.txt
```

The broad extraction found 48 diagnostics/assertion candidates. The committed
focused blocker manifest keeps the 47 rows that current focused validation still
classifies as excluded. `Zend/tests/ErrorException_getSeverity.phpt` was
broad-classified as diagnostics runtime but is now runnable on the focused
manifest, so it is not counted as a blocker.

Validation command:

```sh
tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-diagnostics-assertion-frontier-manifest.txt
```

Artifacts:

```text
.runtime/phpt-progress/classification-20260614T063850Z.tsv
.runtime/phpt-progress/runnable-20260614T063850Z.txt
.runtime/phpt-progress/excluded-20260614T063850Z.tsv
```

Result:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 47 | 0 | 47 |

Focused classifier buckets:

| Classification | Rows |
| --- | ---: |
| `unsupported-diagnostics-runtime` | 16 |
| `unsupported-assertion-ini` | 17 |
| `unsupported-assertion-runtime` | 9 |
| `unsupported-diagnostics-ini` | 5 |

Reason split:

| Generic blocker | Rows |
| --- | ---: |
| Configurable `assert.exception` assertion mode | 17 |
| `debug_backtrace()`/`debug_print_backtrace()` stack-frame snapshots | 9 |
| User error/exception handler state and fallback dispatch | 6 |
| Runtime `zend.assertions` mode switching | 3 |
| Engine diagnostic/logging mode `fatal_error_backtraces` | 3 |
| Namespace-aware assertion resolution and diagnostics | 2 |
| `assert_options()` mode/callback state | 2 |
| Engine diagnostic/logging mode `error_log` | 1 |
| Engine diagnostic/logging mode `report_memleaks` | 1 |
| `ErrorException` severity and trace metadata | 1 |
| Assertion expression lvalue mode interaction | 1 |
| Assertion AST pretty-printing for closure expressions | 1 |

Representative rows:

```text
Zend/tests/ErrorException_construct.phpt
Zend/tests/assert/expect_001.phpt
Zend/tests/assert/expect_016.phpt
Zend/tests/assert/gh16293_001.phpt
Zend/tests/backtrace/bug_debug_backtrace.phpt
Zend/tests/backtrace/debug_backtrace_with_include_and_this.phpt
Zend/tests/backtrace/fatal_error_backtraces_001.phpt
Zend/tests/bug29890.phpt
tests/basic/errorlog_permission.phpt
```

## Implementation Boundary

These rows need shared runtime architecture:

- Stack frame storage with function/class/method type, include frames, `$this`,
  argument snapshots, limit handling, and `DEBUG_BACKTRACE_*` flags.
- User error and exception handler state, including fallback behavior and
  handler exceptions.
- `ErrorException` fields for severity, code, previous, file, line, and trace
  string APIs.
- Assertion runtime state for `assert.exception`, `zend.assertions`,
  `assert_options()`, `ASSERT_BAIL`, `ASSERT_CALLBACK`, namespace resolution,
  and diagnostic rendering.
- Diagnostic/logging channels for `fatal_error_backtraces`, `error_log`, and
  `report_memleaks`.

Until those pieces exist as generic runtime services, keeping these rows
classified prevents broad 1k telemetry from mixing unsupported diagnostic
channels into ordinary PHP semantic failures.

## Verification

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-cuev-baseline
tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-diagnostics-assertion-frontier-manifest.txt
```
