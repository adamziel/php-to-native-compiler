# PHPT Filesystem/Path/Process Frontier: 2026-06-14

Issue: `ptn-d3vd`; refreshed by `ptn-dvu9`

This slice started from a fresh broad 1k classify-only baseline on current
`origin/master`, then focused the existing filesystem/path/process manifest.
The focused result shows that the executable path metadata surface is currently
green; the remaining high-count work is the generic native child-process
boundary plus PHPT cleanup harness execution, not individual file helper
semantics.

This is a blocker map, not a support claim for `proc_open()` or PHPT harness
cleanup. Reopening the 33 excluded rows requires generic runtime/harness
architecture.

## Broad 1k Evidence

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-d3vd-before
```

Generated manifest:
`.runtime/ptn-d3vd-before/20260614T060351Z/phpt-baseline-1000.txt`

Artifacts:

```text
.runtime/phpt-progress/classification-20260614T060351Z.tsv
.runtime/phpt-progress/runnable-20260614T060351Z.txt
.runtime/phpt-progress/summary-20260614T060351Z.txt
```

PTN state for this broad baseline: `5de799cfcd6b`

Corpus revision: `8c63ec400ce8e07c57a8d9499317b96a8beafb8b`

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 1000 | 424 | 576 |

Top classifier buckets:

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

Refresh evidence from `ptn-dvu9` reran the broad 1k classify-only baseline at
PTN state `a8856615c`:

```text
.runtime/ptn-dvu9-baseline-before/20260614T062734Z/phpt-baseline-1000.txt
.runtime/phpt-progress/classification-20260614T062734Z.tsv
.runtime/phpt-progress/runnable-20260614T062734Z.txt
.runtime/phpt-progress/excluded-20260614T062734Z.tsv
```

The refreshed broad result was 1,000 selected, 425 runnable, and 575 excluded.
The filesystem/process conclusion did not change: the current 1k tier contains
only a small direct process-boundary sample, so the focused 46-row manifest is
still the actionable frontier.

## Focused Filesystem/Path/Process Evidence

Manifest:
`tools/phpt-filesystem-path-process-manifest.txt`

Classify-only command:

```sh
tools/run-bounded-phpt.sh --classify-only \
  tools/phpt-filesystem-path-process-manifest.txt
```

Final focused PTN state after rebase: `d37652f2f8ce`

Classify-only artifacts:

```text
.runtime/phpt-progress/classification-20260614T061800Z.tsv
.runtime/phpt-progress/runnable-20260614T061800Z.txt
.runtime/phpt-progress/summary-20260614T061800Z.txt
```

Focused execution command:

```sh
tools/run-bounded-phpt.sh tools/phpt-filesystem-path-process-manifest.txt
```

Execution artifacts:

```text
.runtime/phpt-progress/classification-20260614T061819Z.tsv
.runtime/phpt-progress/runnable-20260614T061819Z.txt
.runtime/phpt-progress/run-20260614T061819Z-manifest.log
.runtime/phpt-progress/summary-20260614T061819Z.txt
```

Focused result:

| Selected | Runnable | Excluded | Passed | Failed | Skipped | Warned |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 46 | 13 | 33 | 13 | 0 | 0 | 0 |

Refresh evidence from `ptn-dvu9` reran the same focused manifest:

```text
.runtime/phpt-progress/classification-20260614T063307Z.tsv
.runtime/phpt-progress/runnable-20260614T063307Z.txt
.runtime/phpt-progress/excluded-20260614T063307Z.tsv
.runtime/phpt-progress/run-20260614T063307Z-manifest.log
```

The refreshed focused result was unchanged: 46 selected, 13 runnable, 33
excluded, 13 passed, 0 failed, 0 skipped, and 0 warned.

Classifier split:

| Classification | Rows |
| --- | ---: |
| `process-boundary` | 25 |
| `harness-cleanup` | 8 |

## Runnable Path Rows

All 13 runnable filesystem/path rows passed under native execution:

```text
ext/standard/tests/file/005_error.phpt
ext/standard/tests/file/005_variation2.phpt
ext/standard/tests/file/006_error.phpt
ext/standard/tests/file/file_exists_variation1.phpt
ext/standard/tests/file/filegroup_error.phpt
ext/standard/tests/file/fileinode_error.phpt
ext/standard/tests/file/fileowner_error.phpt
ext/standard/tests/file/fileperms_variation2.phpt
ext/standard/tests/file/filesize_error.phpt
ext/standard/tests/file/filestat.phpt
ext/standard/tests/file/filetype_basic.phpt
ext/standard/tests/file/filetype_error.phpt
ext/standard/tests/file/lstat_stat_error.phpt
```

These cover error paths and ordinary stat/file metadata behavior. The green
focused run means the next high-yield work should not patch these helpers
row-by-row.

## Process Boundary Blocker

The 25 `process-boundary` rows all exercise `proc_nice()` or `proc_open()`:

```text
ext/standard/tests/general_functions/proc_nice_basic-win.phpt
ext/standard/tests/general_functions/proc_nice_basic.phpt
ext/standard/tests/general_functions/proc_nice_variation2.phpt
ext/standard/tests/general_functions/proc_nice_variation5.phpt
ext/standard/tests/general_functions/proc_open-mb0.phpt
ext/standard/tests/general_functions/proc_open-mb1.phpt
ext/standard/tests/general_functions/proc_open.phpt
ext/standard/tests/general_functions/proc_open02.phpt
ext/standard/tests/general_functions/proc_open_array.phpt
ext/standard/tests/general_functions/proc_open_cmd.phpt
ext/standard/tests/general_functions/proc_open_cwd_null_bytes.phpt
ext/standard/tests/general_functions/proc_open_multiplex.phpt
ext/standard/tests/general_functions/proc_open_null.phpt
ext/standard/tests/general_functions/proc_open_pipes1.phpt
ext/standard/tests/general_functions/proc_open_pipes2.phpt
ext/standard/tests/general_functions/proc_open_pipes3.phpt
ext/standard/tests/general_functions/proc_open_redirect.phpt
ext/standard/tests/general_functions/proc_open_sockets1.phpt
ext/standard/tests/general_functions/proc_open_sockets2.phpt
ext/standard/tests/general_functions/proc_open_sockets3.phpt
ext/standard/tests/streams/proc_open_bug51800_right.phpt
ext/standard/tests/streams/proc_open_bug51800_right2.phpt
ext/standard/tests/streams/proc_open_bug60120.phpt
ext/standard/tests/streams/proc_open_bug64438.phpt
ext/standard/tests/streams/proc_open_bug69900.phpt
```

Generic support requires a native process runtime boundary, including command
execution, environment and current-working-directory handling, descriptor
specification parsing, pipes, multiplexing, sockets or unsupported descriptor
diagnostics, process status/resource objects, and platform-specific
`proc_nice()` behavior. Those semantics need to be shared by future
`exec()`/`system()`/`popen()` support rather than added as PHPT row patches.

## Harness Cleanup Blocker

The 8 `harness-cleanup` rows are ordinary file metadata basic cases with PHPT
`--CLEAN--` sections:

```text
ext/standard/tests/file/006_basic.phpt
ext/standard/tests/file/filegroup_basic.phpt
ext/standard/tests/file/fileinode_basic.phpt
ext/standard/tests/file/fileowner_basic.phpt
ext/standard/tests/file/fileperms_variation3.phpt
ext/standard/tests/file/filesize_basic.phpt
ext/standard/tests/file/is_dir_basic.phpt
ext/standard/tests/file/is_file_basic.phpt
```

These should reopen when the PHPT harness can execute cleanup/setup sections
outside measured program output. They are not evidence of missing
`filegroup()`, `fileinode()`, `fileowner()`, `fileperms()`, `filesize()`,
`is_dir()`, or `is_file()` runtime helpers by themselves.

## Next Implementation Split

1. Add an explicit native child-process/resource model for `proc_open()` and
   related helpers, including descriptor arrays, pipe resources, status, close,
   terminate, and current-working-directory handling.
2. Add platform-aware `proc_nice()` behavior and diagnostics through the same
   process boundary.
3. Extend the PHPT harness runner to model `--CLEAN--` and setup sections
   separately from measured output, then re-run the eight file metadata rows.

## Verification

```sh
cargo fmt --check
cargo test --test phpt_classifier
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-d3vd-before
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-dvu9-baseline-before
tools/run-bounded-phpt.sh --classify-only \
  tools/phpt-filesystem-path-process-manifest.txt
tools/run-bounded-phpt.sh tools/phpt-filesystem-path-process-manifest.txt
```
