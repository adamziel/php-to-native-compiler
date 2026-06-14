# PHPT Broad 1k Request/SAPI Frontier: 2026-06-14

Issue: `ptn-ri9o`

This slice maps the broad 1k PHPT rows blocked on request input, upload, and
SAPI request-boundary semantics. The rows are all under `tests/basic` and cover
POST/COOKIE parsing, `$_GET`/`$_POST`/`$_FILES`/`$_SERVER` population,
`php://input`, `variables_order`, `register_argc_argv`, upload limits, and
request-size or nesting limits.

This is a blocker map, not a support claim. PTN currently compiles native CLI
programs and has no CGI/request boundary, upload parser, request superglobal
population pipeline, or per-request INI state. Reopening these rows today
mostly skips under `run-tests.php` because CGI is unavailable, and the adjacent
CLI request-state rows fail.

## Broad 1k Evidence

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only --out-dir .runtime/ptn-ri9o-baseline
```

Generated manifest:
`.runtime/ptn-ri9o-baseline/20260614T034052Z/phpt-baseline-1000.txt`

Classification artifact:
`.runtime/phpt-progress/classification-20260614T034053Z.tsv`

Evidence command reported PTN commit: `37dd64d2261b`

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
| interface implementation checks | 15 |
| anonymous class syntax | 15 |
| `memory_limit` parsing/enforcement | 15 |

## Focused Frontier

Committed manifest:
`tools/phpt-request-sapi-frontier-manifest.txt`

Selection from `classification-20260614T034053Z.tsv`:

```sh
awk -F'\t' '$2 == "unsupported-request-input-ini" || $2 == "sapi-behavior" {print $1}'
```

Classified result:
`.runtime/phpt-progress/run-20260614T035836Z-manifest.log`

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 41 | 0 | 41 |

Classified split:

| Bucket | Rows |
| --- | ---: |
| `unsupported-request-input-ini` | 28 |
| `sapi-behavior` (`--POST--`) | 13 |

Request/input INI sub-buckets:

| Request input surface | Rows |
| --- | ---: |
| `enable_post_data_reading` | 7 |
| `register_argc_argv` | 6 |
| `file_uploads` | 6 |
| `max_input_vars` | 4 |
| `variables_order` | 2 |
| `post_max_size` | 1 |
| `max_input_nesting_level` | 1 |
| `always_populate_raw_post_data` | 1 |

Raw execution with classification disabled:

```sh
PTN_PHPT_CLASSIFY=0 tools/run-bounded-phpt.sh tools/phpt-request-sapi-frontier-manifest.txt
```

Result:
`.runtime/phpt-progress/run-20260614T035849Z-manifest.log`

| Selected | Runnable | Passed | Failed | Skipped | Excluded |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 41 | 41 | 1 | 3 | 37 | 0 |

Raw failures:

```text
tests/basic/012.phpt
tests/basic/012_register_argc_argv_disabled.phpt
tests/basic/bug29971.phpt
```

The single raw pass is `tests/basic/gh15905.phpt`; it does not make the cluster
implementation-ready because the surrounding request and CGI behavior is still
absent.

## Why This Is A Blocker

The rows share a runtime boundary rather than a local language feature:

- request superglobal population for `$_GET`, `$_POST`, `$_COOKIE`,
  `$_FILES`, `$_REQUEST`, and `$_SERVER`;
- CGI/HTTP request parsing for query strings, POST bodies, cookies, multipart
  form uploads, duplicate keys, malformed names, and upload metadata;
- `php://input` stream state with `enable_post_data_reading` and large body
  behavior;
- per-request INI controls such as `variables_order`, `register_argc_argv`,
  `file_uploads`, `post_max_size`, `max_input_vars`, and nesting limits;
- a SAPI execution harness that can run PHPT `--POST--`, cookie, header, and
  request sections against the native binary instead of plain CLI invocation.

Treating these as runnable in the current native CLI harness would mostly
produce harness skips or request-state failures. The next generic architecture
step is a request context object and SAPI adapter that can initialize
superglobals and request streams before user code runs.

## Representative Rows

```text
tests/basic/002.phpt
tests/basic/011.phpt
tests/basic/012.phpt
tests/basic/bug29971.phpt
tests/basic/bug55500.phpt
tests/basic/enable_post_data_reading_01.phpt
tests/basic/enable_post_data_reading_07.phpt
tests/basic/gh15905.phpt
```

## Verification

```sh
cargo fmt --check
cargo test phpt_classifier
tools/run-phpt-baseline.sh --tier 1000 --classify-only --out-dir .runtime/ptn-ri9o-baseline
tools/run-bounded-phpt.sh tools/phpt-request-sapi-frontier-manifest.txt
PTN_PHPT_CLASSIFY=0 tools/run-bounded-phpt.sh tools/phpt-request-sapi-frontier-manifest.txt
```
