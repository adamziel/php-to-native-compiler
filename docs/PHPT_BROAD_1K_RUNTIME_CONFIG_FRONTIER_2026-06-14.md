# PHPT Broad 1k Runtime Configuration Frontier: 2026-06-14

Issue: `ptn-9isx`

This slice refreshes the broad 1k PHPT classifier on `origin/master` and maps
the non-request runtime-configuration rows that are still outside PTN's modeled
native CLI runtime. This is a blocker map, not a support claim: the rows share
process/runtime configuration state rather than one local parser or array
helper bug.

Request input and SAPI-boundary INI rows stay in
`docs/PHPT_BROAD_1K_REQUEST_SAPI_FRONTIER_2026-06-14.md`; this map covers the
remaining INI/configuration surfaces.

## Broad 1k Evidence

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-9isx-rebased-baseline-current
```

Generated manifest:
`.runtime/ptn-9isx-rebased-baseline-current/20260614T053306Z/phpt-baseline-1000.txt`

Classification artifact:
`.runtime/phpt-progress/classification-20260614T053306Z.tsv`

Evidence command reported PTN commit: `752fa595e424`

Corpus revision: `8c63ec400ce8e07c57a8d9499317b96a8beafb8b`

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 1000 | 429 | 571 |

Top blocker counts:

| Bucket | Rows |
| --- | ---: |
| PHP attributes | 141 |
| magic method dispatch/reflection metadata | 69 |
| call-site/array unpacking | 34 |
| trait declarations | 25 |
| interface declarations | 23 |
| non-public property visibility metadata | 19 |
| configurable `assert.exception` assertion mode | 17 |
| `memory_limit` parsing/enforcement | 15 |
| interface implementation checks | 15 |
| anonymous class syntax | 15 |

## Focused Runtime-Config Frontier

Committed manifest:
`tools/phpt-runtime-config-frontier-manifest.txt`

Selection from `classification-20260614T053306Z.tsv`:

```sh
awk -F'\t' '($2 ~ /^unsupported-.*-ini$/ &&
  $2 != "unsupported-request-input-ini") ||
  $2 == "unsupported-assertion-runtime" {print $1}'
```

Focused classify-only result:

```text
.runtime/phpt-progress/summary-20260614T054248Z.txt
```

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 54 | 0 | 54 |

Classified split:

| Bucket | Rows |
| --- | ---: |
| `unsupported-assertion-ini` | 17 |
| `unsupported-resource-limit-ini` | 15 |
| `unsupported-assertion-runtime` | 9 |
| `unsupported-diagnostics-ini` | 5 |
| `unsupported-function-disable-ini` | 2 |
| `unsupported-host-path-ini` | 2 |
| `unsupported-opcache-ini` | 2 |
| `unsupported-scalar-format-ini` | 2 |

Source split:

| Source bucket | Rows |
| --- | ---: |
| `Zend/tests` | 34 |
| `tests/basic` | 18 |
| `ext/standard` | 2 |

## Blocked Rows

```text
Zend/tests/ArrayAccess/bug63217.phpt
Zend/tests/arrow_functions/gh7900.phpt
Zend/tests/assert/bug70528.phpt
Zend/tests/assert/bug71922.phpt
Zend/tests/assert/disable_assert_function.phpt
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
Zend/tests/assert/expect_016.phpt
Zend/tests/assert/expect_017.phpt
Zend/tests/assert/expect_018.phpt
Zend/tests/assert/expect_019.phpt
Zend/tests/assert/expect_020.phpt
Zend/tests/assert/gh11580.phpt
Zend/tests/assert/gh16293_001.phpt
Zend/tests/assert/gh16293_002.phpt
Zend/tests/asymmetric_visibility/ast_printing.phpt
Zend/tests/backtrace/fatal_error_backtraces_001.phpt
Zend/tests/backtrace/fatal_error_backtraces_002.phpt
Zend/tests/backtrace/fatal_error_backtraces_003.phpt
Zend/tests/bug30820.phpt
Zend/tests/bug36568.phpt
Zend/tests/bug39438.phpt
ext/standard/tests/GHSA-96wq-48vp-hh57.phpt
ext/standard/tests/array/array_sum.phpt
tests/basic/bug31875.phpt
tests/basic/bug67988.phpt
tests/basic/errorlog_permission.phpt
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
tests/basic/gh20858.phpt
tests/basic/ini_directive_deprecated_report_memleaks.phpt
tests/basic/precision.phpt
tests/basic/req60524-win.phpt
```

## Why This Is A Blocker

Generic support needs process-level runtime state that PTN does not model yet:

- `assert.exception`, `zend.assertions`, `assert_options()`, assertion
  callbacks, assertion expression rendering, and namespace-aware assertion
  resolution while preserving the existing direct `AssertionError` path.
- Zend memory manager/resource-limit state for `memory_limit`,
  `max_memory_limit`, runtime `ini_set()`, and allocation-failure diagnostics.
- Engine diagnostic/logging switches such as `fatal_error_backtraces`,
  `error_log`, and `report_memleaks`.
- Runtime function table mutation from `disable_functions`.
- OPcache configuration, which does not correspond to a PTN native runtime
  layer today.
- Scalar formatting and host path defaults such as `default_charset`,
  `serialize_precision`, `sendmail_path`, and `sys_temp_dir`.

Reopening these rows before those shared runtime configuration layers exist
would mostly convert explicit blockers into unrelated assertion, diagnostics,
memory-limit, and process-global failures.

## Verification

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-9isx-baseline-current
tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-runtime-config-frontier-manifest.txt
cargo fmt --check
cargo test --test phpt_classifier
```
