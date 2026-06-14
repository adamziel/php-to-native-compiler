# PHPT Broad 1k Runtime INI Frontier: 2026-06-14

Issue: `ptn-jn15`

This slice refreshes the current broad 1k runtime INI/configuration frontier.
It is a blocker map, not a support claim: the selected rows need runtime
configuration state, request/SAPI boundaries, assertion mode switches, memory
accounting, diagnostic logging channels, function-table mutation, host path
configuration, OPcache state, and scalar formatting defaults.

## Broad 1k Evidence

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only --out-dir .runtime/ptn-jn15-baseline-current
```

Generated manifest:
`.runtime/ptn-jn15-baseline-current/20260614T055818Z/phpt-baseline-1000.txt`

Artifacts:

```text
.runtime/phpt-progress/classification-20260614T055818Z.tsv
.runtime/phpt-progress/runnable-20260614T055818Z.txt
.runtime/phpt-progress/summary-20260614T055818Z.txt
```

PTN state: current `ptn-jn15` branch from `origin/master`.

Corpus revision: `8c63ec400ce8e07c57a8d9499317b96a8beafb8b`

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 1000 | 425 | 575 |

Top broad classifier buckets:

| Bucket | Rows |
| --- | ---: |
| `unsupported-language` | 288 |
| `unsupported-class-metadata` | 143 |
| `unsupported-request-input-ini` | 28 |
| `unsupported-extension` | 20 |
| `unsupported-assertion-ini` | 17 |
| `unsupported-diagnostics-runtime` | 16 |
| `unsupported-resource-limit-ini` | 15 |
| `sapi-behavior` | 13 |

## Focused Frontier

Committed manifest:
`tools/phpt-runtime-ini-frontier-manifest.txt`

Selection from the classifier:

```sh
awk -F'\t' '$2 ~ /-ini$/ {print $1}' \
  .runtime/phpt-progress/classification-20260614T055818Z.tsv
```

Focused classifier result:

```sh
tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-runtime-ini-frontier-manifest.txt
```

Result at `.runtime/phpt-progress/summary-20260614T060524Z.txt`:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 73 | 0 | 73 |

All 73 focused rows remain classified into runtime INI/config buckets.

## Bucket Split

| Runtime configuration surface | Rows |
| --- | ---: |
| Request/input/upload SAPI INI state | 28 |
| Assertion `assert.exception` mode | 17 |
| PHP `memory_limit` parsing/enforcement | 15 |
| Diagnostic/logging INI channels | 5 |
| Runtime `disable_functions` mutation | 2 |
| Zend OPcache configuration | 2 |
| Scalar formatting defaults | 2 |
| Host-path INI values | 2 |

Path concentration:

| Path family | Rows |
| --- | ---: |
| `tests/basic` | 46 |
| `Zend/tests` | 25 |
| `ext/standard` | 2 |

Representative rows:

```text
Zend/tests/assert/expect_001.phpt
Zend/tests/backtrace/fatal_error_backtraces_001.phpt
Zend/tests/bug36568.phpt
ext/standard/tests/GHSA-96wq-48vp-hh57.phpt
tests/basic/011.phpt
tests/basic/bug67988.phpt
tests/basic/enable_post_data_reading_01.phpt
tests/basic/gh17951_runtime_change_6.phpt
tests/basic/req60524-win.phpt
```

## Why This Is A Blocker

These rows share a runtime configuration boundary rather than one local helper
bug. Generic support needs:

- one process/request configuration table shared by `-d`, `ini_get()`,
  `ini_set()`, `ini_restore()`, and internal subsystems;
- request context and SAPI adapters for input, uploads, `php://input`, and
  superglobal population controlled by INI values;
- assertion runtime mode state, including disabled assertions, exception mode,
  callbacks, bail behavior, namespace resolution, and diagnostic rendering;
- memory-accounting and allocation-failure boundaries before `memory_limit`
  rows can execute safely;
- diagnostic channels for fatal backtraces, error logs, and memory leak
  reporting;
- runtime function registry mutation for `disable_functions`;
- host-path and OPcache configuration boundaries that currently do not exist
  in the native runtime.

Treating these rows as runnable today would convert missing runtime-state
architecture into noisy PHPT failures. Keeping the 73-row frontier explicit
makes the broad 1k dashboard actionable while those generic subsystems are
designed.

## Verification

```sh
cargo fmt --check
cargo test --test phpt_classifier
tools/run-phpt-baseline.sh --tier 1000 --classify-only --out-dir .runtime/ptn-jn15-baseline-current
tools/run-bounded-phpt.sh --classify-only --classify-harness-programs tools/phpt-runtime-ini-frontier-manifest.txt
```
