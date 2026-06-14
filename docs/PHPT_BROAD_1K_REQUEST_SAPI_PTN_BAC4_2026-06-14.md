# PHPT Broad 1k Request/SAPI Frontier: 2026-06-14 ptn-bac4

Issue: `ptn-bac4`

This slice refreshes the broad 1k request/SAPI boundary. It is a blocker map,
not a runtime implementation claim. The cluster is coherent and above the
25-row target, but it needs a generic request runtime and SAPI harness instead
of row-local patches. The broad 1k snapshot below is historical branch evidence;
the focused request/SAPI replay was rerun on the integrated queue head.

## Broad 1k Evidence

Command:

```sh
PHPT_PROGRESS_DIR=.runtime/ptn-bac4-baseline-progress \
  tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-bac4-baseline
```

Generated broad manifests:

```text
.runtime/ptn-bac4-baseline/20260614T122123Z/phpt-baseline-1000.txt
.runtime/ptn-bac4-baseline/20260614T122123Z/phpt-baseline-5000.txt
.runtime/ptn-bac4-baseline/20260614T122123Z/phpt-baseline-10000.txt
```

Artifacts:

```text
.runtime/ptn-bac4-baseline-progress/classification-20260614T122123Z.tsv
.runtime/ptn-bac4-baseline-progress/runnable-20260614T122123Z.txt
.runtime/ptn-bac4-baseline-progress/excluded-20260614T122123Z.tsv
.runtime/ptn-bac4-baseline-progress/summary-20260614T122123Z.txt
```

State:

```text
Historical PTN: ee4439bf00c7
php-src PHPT corpus: /home/claude/php-src-phpt
corpus revision: 8c63ec400ce8e07c57a8d9499317b96a8beafb8b
```

Later integration added classifier bucket splits outside the request/SAPI
surface, notably object-string conversion and call-vs-array unpacking. The
request/SAPI counts are unchanged by those splits.

Classifier result:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 1,000 | 424 | 576 |

Top historical broad classifier buckets:

| Classification | Rows |
| --- | ---: |
| `runnable` | 424 |
| `unsupported-attribute-syntax-metadata` | 141 |
| `unsupported-magic-method-metadata` | 69 |
| `unsupported-call-unpacking` | 34 |
| `unsupported-request-input-ini` | 28 |
| `unsupported-trait-declaration` | 25 |
| `unsupported-interface-declaration` | 23 |
| `unsupported-extension` | 20 |
| `unsupported-property-visibility-metadata` | 19 |
| `unsupported-diagnostics-runtime` | 17 |
| `unsupported-assertion-ini` | 17 |
| `unsupported-resource-limit-ini` | 15 |
| `unsupported-interface-implementation` | 15 |
| `unsupported-anonymous-class` | 15 |
| `unsupported-type-hint` | 14 |
| `sapi-behavior` | 13 |

The request/SAPI slice is the 41-row union of
`unsupported-request-input-ini` and `sapi-behavior`.

Current focused replay below confirms the request/SAPI union is still 28 + 13
after the later classifier splits for object-string conversion and unpacking.

## Focused Request/SAPI Classification

Manifest:

```text
tools/phpt-request-sapi-frontier-manifest.txt
```

Command:

```sh
PHPT_PROGRESS_DIR=.runtime/ptn-bac4-request-classify-current \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-request-sapi-frontier-manifest.txt
```

Artifacts:

```text
.runtime/ptn-bac4-request-classify-current/classification-20260614T135358Z.tsv
.runtime/ptn-bac4-request-classify-current/excluded-20260614T135358Z.tsv
.runtime/ptn-bac4-request-classify-current/summary-20260614T135358Z.txt
```

Focused classifier result:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 41 | 0 | 41 |

Classifier split:

| Classification | Rows |
| --- | ---: |
| `unsupported-request-input-ini` | 28 |
| `sapi-behavior` | 13 |

Request/input INI sub-buckets:

| INI/request surface | Rows |
| --- | ---: |
| `enable_post_data_reading` | 7 |
| `register_argc_argv` | 6 |
| `file_uploads` | 6 |
| `max_input_vars` | 4 |
| `variables_order` | 2 |
| `post_max_size` | 1 |
| `max_input_nesting_level` | 1 |
| `always_populate_raw_post_data` | 1 |

The 13 `sapi-behavior` rows are PHPT `--POST--`/CGI request rows:

```text
tests/basic/002.phpt
tests/basic/003.phpt
tests/basic/004.phpt
tests/basic/005.phpt
tests/basic/013.phpt
tests/basic/014.phpt
tests/basic/015.phpt
tests/basic/016.phpt
tests/basic/017.phpt
tests/basic/018.phpt
tests/basic/019.phpt
tests/basic/020.phpt
tests/basic/bug78236.phpt
```

The 28 request-input INI rows include CLI argv/request-state rows, upload and
cookie rows, `php://input`, and request parsing limit rows:

```text
tests/basic/011.phpt
tests/basic/011_empty_query.phpt
tests/basic/011_register_argc_argv_disabled.phpt
tests/basic/011_windows.phpt
tests/basic/012.phpt
tests/basic/012_register_argc_argv_disabled.phpt
tests/basic/021.phpt
tests/basic/022.phpt
tests/basic/023.phpt
tests/basic/025.phpt
tests/basic/028.phpt
tests/basic/030.phpt
tests/basic/031.phpt
tests/basic/032.phpt
tests/basic/bug29971.phpt
tests/basic/bug53180.phpt
tests/basic/bug55500.phpt
tests/basic/bug61000.phpt
tests/basic/bug78929.phpt
tests/basic/bug79699.phpt
tests/basic/enable_post_data_reading_01.phpt
tests/basic/enable_post_data_reading_02.phpt
tests/basic/enable_post_data_reading_03.phpt
tests/basic/enable_post_data_reading_04.phpt
tests/basic/enable_post_data_reading_05.phpt
tests/basic/enable_post_data_reading_06.phpt
tests/basic/enable_post_data_reading_07.phpt
tests/basic/gh15905.phpt
```

## Raw Pass-Through Check

Command:

```sh
PHPT_PROGRESS_DIR=.runtime/ptn-bac4-request-raw-current \
  PTN_PHPT_CLASSIFY=0 \
  tools/run-bounded-phpt.sh tools/phpt-request-sapi-frontier-manifest.txt
```

Results:

- `cargo fmt --check` passed.
- Current focused request/SAPI classify-only replay: 41 selected, 0 runnable,
  41 classified out; run-tests-exit 0.
- Current raw pass-through with classification disabled: 41 selected, 41
  runnable, 1 passed, 3 failed, 37 skipped, 0 warned; run-tests-exit 1.

Artifacts:

```text
.runtime/ptn-bac4-request-raw-current/classification-20260614T135558Z.tsv
.runtime/ptn-bac4-request-raw-current/run-20260614T135558Z-manifest.log
.runtime/ptn-bac4-request-raw-current/summary-20260614T135558Z.txt
```

Raw result with classification disabled:

| Selected | Runnable | Passed | Failed | Skipped | Warned |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 41 | 41 | 1 | 3 | 37 | 0 |

The 37 skipped rows all report CGI unavailable in the current native CLI PHPT
harness. The three raw failures are:

```text
tests/basic/012.phpt
tests/basic/012_register_argc_argv_disabled.phpt
tests/basic/bug29971.phpt
```

The single raw pass is:

```text
tests/basic/gh15905.phpt
```

## Blocker Boundary

The cluster needs a shared request runtime boundary:

1. SAPI request context creation before user code runs.
2. Query-string, POST body, cookie, and multipart upload parsing.
3. Population of `$_GET`, `$_POST`, `$_COOKIE`, `$_FILES`, `$_REQUEST`,
   `$_SERVER`, `$argc`, and `$argv` according to request mode and INI state.
4. `php://input` stream state, including `enable_post_data_reading` behavior.
5. Request limits and diagnostics for `post_max_size`, `max_input_vars`, and
   `max_input_nesting_level`.
6. A PHPT runner/SAPI adapter that can execute `--POST--`, cookie, header, and
   request sections against the native binary instead of plain CLI invocation.

Until that architecture exists, keeping these rows classified is more accurate
than letting broad pass counts depend on unavailable CGI harness behavior or
partial CLI superglobal state.

## Verification

```sh
cargo fmt --check
PHPT_PROGRESS_DIR=.runtime/ptn-bac4-baseline-progress \
  tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-bac4-baseline
PHPT_PROGRESS_DIR=.runtime/ptn-bac4-request-classify-current \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-request-sapi-frontier-manifest.txt
PHPT_PROGRESS_DIR=.runtime/ptn-bac4-request-raw-current \
  PTN_PHPT_CLASSIFY=0 \
  tools/run-bounded-phpt.sh tools/phpt-request-sapi-frontier-manifest.txt
```
