# PHPT Broad 1k Runtime Boundary Frontier: 2026-06-14

Issue: `ptn-ypmu`

This slice uses the broad PHPT baseline tooling on current `origin/master` and
maps the rows blocked on native runtime boundary semantics rather than local
parser, scalar, array, or object helper behavior. The cluster covers request
SAPI input, PHPT setup/cleanup/environment sections, child-process execution,
external php-src server harnesses, and process-global host-path INI state.

This is a blocker map, not a support claim. Reopening these rows as normal
native CLI tests today converts explicit boundary classifications into CGI
skips plus unrelated request-state, cleanup, process, and environment failures.

## Broad 1k Evidence

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-ypmu-baseline-rebased
```

Generated broad manifest:

```text
.runtime/ptn-ypmu-baseline-rebased/20260614T073049Z/phpt-baseline-1000.txt
```

Artifacts:

```text
.runtime/phpt-progress/classification-20260614T073049Z.tsv
.runtime/phpt-progress/runnable-20260614T073049Z.txt
.runtime/phpt-progress/excluded-20260614T073049Z.tsv
```

PTN state for this broad baseline: `42173d2f1033`

Corpus revision: `8c63ec400ce8e07c57a8d9499317b96a8beafb8b`

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 1,000 | 424 | 576 |

Top classifier buckets:

| Classification | Rows |
| --- | ---: |
| `unsupported-language` | 288 |
| `unsupported-class-metadata` | 143 |
| `unsupported-request-input-ini` | 28 |
| `unsupported-extension` | 20 |
| `unsupported-assertion-ini` | 17 |
| `unsupported-diagnostics-runtime` | 17 |
| `unsupported-resource-limit-ini` | 15 |
| `sapi-behavior` | 13 |
| `unsupported-assertion-runtime` | 9 |

## Focused Boundary Evidence

Committed manifest:

```text
tools/phpt-runtime-boundary-frontier-manifest.txt
```

Selection from `classification-20260614T073049Z.tsv`:

```sh
awk -F'\t' '$2 == "unsupported-request-input-ini" ||
  $2 == "sapi-behavior" ||
  $2 == "process-boundary" ||
  $2 == "external-service" ||
  $2 == "environment-assumption" ||
  $2 == "unsupported-host-path-ini" ||
  $2 == "harness-cleanup" { print $1 }'
```

Focused classify-only command:

```sh
tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-runtime-boundary-frontier-manifest.txt
```

Artifacts:

```text
.runtime/phpt-progress/classification-20260614T073620Z.tsv
.runtime/phpt-progress/runnable-20260614T073620Z.txt
.runtime/phpt-progress/excluded-20260614T073620Z.tsv
```

Classify-only result:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 52 | 0 | 52 |

Focused split:

| Bucket | Rows | Shared blocker |
| --- | ---: | --- |
| `unsupported-request-input-ini` | 28 | Request/input/upload SAPI state controlled by `register_argc_argv`, `file_uploads`, `max_input_vars`, `enable_post_data_reading`, `variables_order`, `post_max_size`, `max_input_nesting_level`, and legacy raw-post controls. |
| `sapi-behavior` | 13 | PHPT `--POST--` request sections need a SAPI adapter and request-body parser before native execution. |
| `harness-cleanup` | 4 | PHPT `--CLEAN--` setup/cleanup sections need to run outside measured program output. |
| `process-boundary` | 3 | Native child-process execution, extension-directory subprocess checks, and pipe/control semantics are not modeled by PTN's CLI runtime boundary. |
| `unsupported-host-path-ini` | 2 | Process-global host-path INI state such as `sendmail_path` and `sys_temp_dir` is not modeled. |
| `external-service` | 1 | The row needs an external service or php-src server harness. |
| `environment-assumption` | 1 | PHPT `--ENV--` setup is outside current script semantics. |

Source split: 49 `tests/basic` rows, 2 `Zend/tests` rows, and 1
`ext/standard` row.

## Raw Native CLI Evidence

Raw command with classification disabled:

```sh
PTN_PHPT_CLASSIFY=0 tools/run-bounded-phpt.sh \
  tools/phpt-runtime-boundary-frontier-manifest.txt
```

Raw run artifact:

```text
.runtime/phpt-progress/run-20260614T072505Z-manifest.log
```

Raw result:

| Selected | Runnable | Passed | Failed | Skipped | Warned |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 52 | 52 | 2 | 13 | 37 | 0 |

The 37 skips are mostly request rows requiring CGI. The 13 failures are the
expected plain-CLI failures for cleanup/autoload stream-wrapper rows, CLI
`$argc`/`$argv` request state, process-boundary checks, `php://input`,
`variables_order`, environment setup, and host-path INI behavior.

## Blocked Rows

Request/input INI and POST SAPI rows:

```text
tests/basic/002.phpt
tests/basic/003.phpt
tests/basic/004.phpt
tests/basic/005.phpt
tests/basic/011.phpt
tests/basic/011_empty_query.phpt
tests/basic/011_register_argc_argv_disabled.phpt
tests/basic/011_windows.phpt
tests/basic/012.phpt
tests/basic/012_register_argc_argv_disabled.phpt
tests/basic/013.phpt
tests/basic/014.phpt
tests/basic/015.phpt
tests/basic/016.phpt
tests/basic/017.phpt
tests/basic/018.phpt
tests/basic/019.phpt
tests/basic/020.phpt
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
tests/basic/bug78236.phpt
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

Harness, process, external-service, environment, and host-path rows:

```text
Zend/tests/autoload/bug63741.phpt
Zend/tests/bug38779_1.phpt
ext/standard/tests/array/array_count_values_variation.phpt
tests/basic/GHSA-9pqp-7h25-4f32.phpt
tests/basic/bug67198.phpt
tests/basic/bug71273.phpt
tests/basic/bug80384.phpt
tests/basic/gh16998.phpt
tests/basic/gh20858.phpt
tests/basic/gh7896.phpt
tests/basic/req60524-win.phpt
```

## Next Implementation Split

1. Add a request context and SAPI adapter that can initialize request
   superglobals, `php://input`, cookies, upload metadata, and per-request INI
   controls before user code runs.
2. Extend PHPT execution to run `--ENV--`, setup, and `--CLEAN--` sections
   outside the measured native program output.
3. Add a native child-process/resource boundary shared by `proc_open()`,
   process-control helpers, extension-directory subprocess checks, and future
   `exec()`/`system()`/`popen()` support.
4. Model process-global host path configuration such as `sendmail_path` and
   `sys_temp_dir` as runtime INI state, not ad hoc PHPT behavior.

## Verification

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-ypmu-baseline-rebased
tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-runtime-boundary-frontier-manifest.txt
PTN_PHPT_CLASSIFY=0 tools/run-bounded-phpt.sh \
  tools/phpt-runtime-boundary-frontier-manifest.txt
```
