# PHPT Broad 1k Runtime Config/Boundary Focused Subset: 2026-06-14 ptn-3qj5

Issue: `ptn-3qj5`

This slice records a broad 1k PHPT classifier run and rolls up the rows blocked
on runtime configuration, request/SAPI state, assertion runtime modes,
process/harness boundaries, host preconditions, and resource-limit behavior. It
is a focused 103-row subset of the broader current `ptn-2qq9` 144-row
runtime/configuration/diagnostics boundary map, not a support claim: these rows
need shared runtime architecture rather than one local parser, array, or output
formatting patch.

## Broad 1k Evidence

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-3qj5-baseline-1k
```

Generated broad manifests:

```text
.runtime/ptn-3qj5-baseline-1k/20260614T100200Z/phpt-baseline-1000.txt
.runtime/ptn-3qj5-baseline-1k/20260614T100200Z/phpt-baseline-5000.txt
.runtime/ptn-3qj5-baseline-1k/20260614T100200Z/phpt-baseline-10000.txt
```

Classifier artifacts:

```text
.runtime/phpt-progress/classification-20260614T100200Z.tsv
.runtime/phpt-progress/runnable-20260614T100200Z.txt
.runtime/phpt-progress/excluded-20260614T100200Z.tsv
```

Evidence state:

```text
PTN: 7d6f6b2c6af8
php-src PHPT corpus: /home/claude/php-src-phpt
corpus revision: 8c63ec400ce8e07c57a8d9499317b96a8beafb8b
```

The branch was then rebased over docs/manifest-only broad PHPT maps
`ca37037df`, `019c42477`, and `80ca9dfc3`; no classifier/runtime code changed
in this slice. Later classifier refinements split class metadata and class
declaration buckets, but the runtime/config focused selection below is
unchanged.

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 1,000 | 424 | 576 |

Top recorded broad classifier buckets:

| Classification | Rows |
| --- | ---: |
| `unsupported-attribute-metadata` | 149 |
| `unsupported-class-metadata` | 135 |
| `unsupported-class-declaration` (later split by `ptn-gkvr`) | 78 |
| `unsupported-call-unpacking` | 34 |
| `unsupported-request-input-ini` | 28 |
| `unsupported-extension` | 20 |
| `unsupported-diagnostics-runtime` | 17 |
| `unsupported-assertion-ini` | 17 |
| `unsupported-resource-limit-ini` | 15 |
| `unsupported-type-hint` | 14 |
| `sapi-behavior` | 13 |
| `unsupported-function-state` | 11 |
| `unsupported-assertion-runtime` | 9 |

## Focused Frontier

Committed manifest:

```text
tools/phpt-runtime-config-boundary-ptn-3qj5-manifest.txt
```

Selection from the recorded broad classifier:

```sh
awk -F'\t' '$2 ~ /ini$/ ||
  $2 == "unsupported-assertion-runtime" ||
  $2 == "sapi-behavior" ||
  $2 == "process-boundary" ||
  $2 == "external-service" ||
  $2 == "environment-assumption" ||
  $2 == "skipif-precondition" ||
  $2 == "unsupported-resource-limit" {print $1}' \
  .runtime/phpt-progress/classification-20260614T100200Z.tsv
```

Focused classify-only command:

```sh
PHPT_PROGRESS_DIR=.runtime/ptn-3qj5-runtime-config-boundary-focused-rebased \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-runtime-config-boundary-ptn-3qj5-manifest.txt
```

Focused artifacts:

```text
.runtime/ptn-3qj5-runtime-config-boundary-focused-rebased/classification-20260614T101218Z.tsv
.runtime/ptn-3qj5-runtime-config-boundary-focused-rebased/runnable-20260614T101218Z.txt
.runtime/ptn-3qj5-runtime-config-boundary-focused-rebased/excluded-20260614T101218Z.tsv
```

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 103 | 0 | 103 |

The same committed manifest was replayed after the current `ptn-gkvr`
class-declaration split and remained fully classified: 103 selected, 0
runnable, 103 excluded.

## Blocker Split

| Runtime boundary | Rows | Classifier buckets |
| --- | ---: | --- |
| Request/SAPI input state | 41 | `unsupported-request-input-ini` 28, `sapi-behavior` 13 |
| Assertion configuration/runtime modes | 26 | `unsupported-assertion-ini` 17, `unsupported-assertion-runtime` 9 |
| Memory and allocation resource limits | 16 | `unsupported-resource-limit-ini` 15, `unsupported-resource-limit` 1 |
| Diagnostic/logging configuration | 5 | `unsupported-diagnostics-ini` 5 |
| Process/runtime INI registry gaps | 8 | `unsupported-function-disable-ini`, `unsupported-host-path-ini`, `unsupported-opcache-ini`, and `unsupported-scalar-format-ini`, 2 each |
| Harness, process, service, and host preconditions | 7 | `process-boundary` 3, `skipif-precondition` 2, `external-service` 1, `environment-assumption` 1 |

Source split:

| Source family | Rows |
| --- | ---: |
| `tests/basic` | 65 |
| `Zend/tests` | 35 |
| `ext/standard` | 3 |

Representative rows:

```text
Zend/tests/assert/expect_001.phpt
Zend/tests/assert/gh16293_001.phpt
Zend/tests/backtrace/fatal_error_backtraces_001.phpt
ext/standard/tests/array/array_fill_error2.phpt
tests/basic/011.phpt
tests/basic/GHSA-9pqp-7h25-4f32.phpt
tests/basic/enable_post_data_reading_01.phpt
tests/basic/gh17951_runtime_change_6.phpt
tests/basic/gh7896.phpt
tests/basic/req60524-win.phpt
```

## Why This Is A Blocker

The focused rows are high-yield, but they are not one credible implementation
patch. Generic support needs several shared boundaries:

- a request context and SAPI adapter for request-body parsing, uploads,
  `php://input`, and superglobal population;
- a runtime INI/configuration table shared by `-d`, `ini_get()`, `ini_set()`,
  `ini_restore()`, assertions, diagnostics, scalar formatting, host paths, and
  function registry mutation;
- assertion mode state for `zend.assertions`, `assert.exception`,
  `assert_options()`, callbacks, namespace resolution, expression rendering,
  and disabled-assertion behavior;
- memory-accounting and resource-limit enforcement before allocation-heavy
  rows can execute safely;
- PHPT harness support for `--ENV--`, host preconditions, external-service
  setup, cleanup, and child-process execution outside measured program output.

Reopening these rows today would convert explicit blockers into unrelated
native CLI request-state, assertion-mode, diagnostics, process-boundary, and
resource-limit failures. The next implementation work should split along those
runtime contracts rather than by PHPT filename family.

## Verification

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-3qj5-baseline-1k
PHPT_PROGRESS_DIR=.runtime/ptn-3qj5-runtime-config-boundary-focused-rebased \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-runtime-config-boundary-ptn-3qj5-manifest.txt
cargo fmt --check
cargo test --test phpt_classifier
```
