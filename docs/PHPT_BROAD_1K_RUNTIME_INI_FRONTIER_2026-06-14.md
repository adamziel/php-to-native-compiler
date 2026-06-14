# PHPT Broad 1k Runtime/System INI Frontier: 2026-06-14

Issue: `ptn-7uli`

This slice maps current broad 1k rows blocked on runtime configuration rather
than ordinary PHP syntax or helper behavior. The rows cover assertion INI
modes, memory/resource limits, engine diagnostic/logging INI, disabled
functions, scalar formatting defaults, opcache controls, and host-path
configuration.

This is a blocker map, not a support claim. PTN currently accepts a bounded
set of CLI `-d` options and native runtime environment knobs, but it does not
have a Zend-compatible INI registry, memory manager/resource-limit boundary,
mutable function table, opcache subsystem, diagnostic logging channel, or
process-global host-path configuration.

## Broad 1k Evidence

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only
```

Generated manifest:

```text
.runtime/phpt-baseline/20260614T060358Z/phpt-baseline-1000.txt
```

Classification artifacts:

```text
.runtime/phpt-progress/classification-20260614T060358Z.tsv
.runtime/phpt-progress/runnable-20260614T060358Z.txt
.runtime/phpt-progress/excluded-20260614T060358Z.tsv
```

Evidence command reported PTN commit: `5de799cfcd6b`

Corpus revision: `8c63ec400ce8e07c57a8d9499317b96a8beafb8b`

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 1,000 | 424 | 576 |

The runtime/system INI slice accounts for 46 current broad 1k excluded rows.

## Focused Evidence

Committed manifest:

```text
tools/phpt-runtime-ini-frontier-manifest.txt
```

Selection from `classification-20260614T060358Z.tsv`:

```sh
awk -F'\t' '$2=="unsupported-assertion-ini" ||
  $2=="unsupported-resource-limit-ini" ||
  $2=="unsupported-resource-limit" ||
  $2=="unsupported-diagnostics-ini" ||
  $2=="unsupported-function-disable-ini" ||
  $2=="unsupported-scalar-format-ini" ||
  $2=="unsupported-opcache-ini" ||
  $2=="unsupported-host-path-ini" {print $1}'
```

Classified result:

```text
.runtime/phpt-progress/summary-20260614T061458Z.txt
.runtime/phpt-progress/classification-20260614T061458Z.tsv
```

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 46 | 0 | 46 |

Classified split:

| Bucket | Rows |
| --- | ---: |
| `unsupported-assertion-ini` | 17 |
| `unsupported-resource-limit-ini` | 15 |
| `unsupported-resource-limit` | 1 |
| `unsupported-diagnostics-ini` | 5 |
| `unsupported-function-disable-ini` | 2 |
| `unsupported-scalar-format-ini` | 2 |
| `unsupported-opcache-ini` | 2 |
| `unsupported-host-path-ini` | 2 |

## Rows By Bucket

Assertion INI modes:

```text
Zend/tests/arrow_functions/gh7900.phpt
Zend/tests/assert/bug71922.phpt
Zend/tests/assert/expect_001.phpt
Zend/tests/assert/expect_002.phpt
Zend/tests/assert/expect_003.phpt
Zend/tests/assert/expect_004.phpt
Zend/tests/assert/expect_005.phpt
Zend/tests/assert/expect_006.phpt
Zend/tests/assert/expect_007.phpt
Zend/tests/assert/expect_008.phpt
Zend/tests/assert/expect_009.phpt
Zend/tests/assert/expect_010.phpt
Zend/tests/assert/expect_011.phpt
Zend/tests/assert/expect_012.phpt
Zend/tests/assert/expect_013.phpt
Zend/tests/assert/expect_014.phpt
Zend/tests/asymmetric_visibility/ast_printing.phpt
```

Resource limits:

```text
Zend/tests/bug36568.phpt
Zend/tests/bug39438.phpt
ext/standard/tests/GHSA-96wq-48vp-hh57.phpt
ext/standard/tests/array/array_fill_error2.phpt
ext/standard/tests/array/array_sum.phpt
tests/basic/gh17951_ini_parse_1.phpt
tests/basic/gh17951_ini_parse_2.phpt
tests/basic/gh17951_ini_parse_3.phpt
tests/basic/gh17951_ini_parse_4.phpt
tests/basic/gh17951_ini_parse_5.phpt
tests/basic/gh17951_runtime_change_1.phpt
tests/basic/gh17951_runtime_change_2.phpt
tests/basic/gh17951_runtime_change_3.phpt
tests/basic/gh17951_runtime_change_4.phpt
tests/basic/gh17951_runtime_change_5.phpt
tests/basic/gh17951_runtime_change_6.phpt
```

Diagnostics, disabled functions, scalar formatting, opcache, and host paths:

```text
Zend/tests/ArrayAccess/bug63217.phpt
Zend/tests/assert/disable_assert_function.phpt
Zend/tests/backtrace/fatal_error_backtraces_001.phpt
Zend/tests/backtrace/fatal_error_backtraces_002.phpt
Zend/tests/backtrace/fatal_error_backtraces_003.phpt
Zend/tests/bug30820.phpt
tests/basic/bug31875.phpt
tests/basic/bug67988.phpt
tests/basic/errorlog_permission.phpt
tests/basic/gh20858.phpt
tests/basic/ini_directive_deprecated_report_memleaks.phpt
tests/basic/precision.phpt
tests/basic/req60524-win.phpt
```

## Why This Is A Blocker

These rows are not credible as a small local implementation slice because they
share process-wide engine state:

- assertion rows need `assert.exception`, `zend.assertions`, and related
  runtime mode switching to feed the existing catchable `AssertionError` path;
- resource-limit rows need PHP quantity parsing plus allocation accounting and
  fatal diagnostics at the Zend memory-manager boundary;
- diagnostics rows need runtime channels for fatal backtraces, `error_log`, and
  report-memleaks behavior;
- disabled-function rows require mutable function-table policy instead of
  PTN's current fixed internal/userland registry;
- scalar formatting rows need default charset and `serialize_precision`
  integration beyond the current bounded `precision` support;
- opcache and host-path rows depend on process-global configuration surfaces
  that do not map to native CLI execution yet.

The next implementation split should be a generic INI registry with typed
directives and per-directive capability flags. After that, assertion and scalar
formatting modes are plausible early users. Memory limits and opcache should
stay classified until PTN has runtime allocation accounting and an opcache
model.

## Verification

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only
tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-runtime-ini-frontier-manifest.txt
cargo fmt --check
```
