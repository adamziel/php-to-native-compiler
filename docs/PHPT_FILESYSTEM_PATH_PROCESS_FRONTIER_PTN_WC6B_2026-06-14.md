# PHPT Filesystem/Path/Process Frontier: 2026-06-14 ptn-wc6b

Issue: `ptn-wc6b`

This slice refreshes the broad-derived filesystem/path/process frontier on a
rebased branch. It is a blocker map, not a runtime implementation change. The
current path metadata rows remain green; the high-yield residual cluster is the
generic native child-process boundary.

## Broad 1k Baseline Tooling

The branch was rebased on `origin/master` and the broad 1k tooling was started
with private output directories:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-wc6b-baseline
timeout 180s env PHPT_PROGRESS_DIR=.runtime/ptn-wc6b-progress \
  tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-wc6b-baseline-isolated
```

The first attempt generated:

```text
.runtime/ptn-wc6b-baseline/20260614T110920Z/phpt-baseline-1000.txt
```

The isolated attempt generated:

```text
.runtime/ptn-wc6b-baseline-isolated/20260614T111346Z/phpt-baseline-1000.txt
```

Both used php-src PHPT corpus revision:

```text
8c63ec400ce8e07c57a8d9499317b96a8beafb8b
```

Under the concurrent polecat load, classify-only did not complete inside the
bounded window. The isolated run timed out with status 124 after writing partial
artifacts through `.runtime/ptn-wc6b-progress/classification-20260614T111346Z.tsv`.
The generated 1k manifest was still useful for confirming that the existing
filesystem/path/process focused manifest remains the coherent cluster for this
slice.

## Focused Evidence

Manifest:

```text
tools/phpt-filesystem-path-process-manifest.txt
```

Command:

```sh
PHPT_PROGRESS_DIR=.runtime/ptn-wc6b-filesystem-progress-rebased \
  tools/run-bounded-phpt.sh tools/phpt-filesystem-path-process-manifest.txt
```

Artifacts:

```text
.runtime/ptn-wc6b-filesystem-progress-rebased/classification-20260614T112717Z.tsv
.runtime/ptn-wc6b-filesystem-progress-rebased/runnable-20260614T112717Z.txt
.runtime/ptn-wc6b-filesystem-progress-rebased/excluded-20260614T112717Z.tsv
.runtime/ptn-wc6b-filesystem-progress-rebased/run-20260614T112717Z-manifest.log
.runtime/ptn-wc6b-filesystem-progress-rebased/summary-20260614T112717Z.txt
```

State:

```text
PTN: 7eaac3d2ea6e
php-src PHPT corpus: /home/claude/php-src-phpt
corpus revision: 8c63ec400ce8e07c57a8d9499317b96a8beafb8b
```

The final branch was later rebased over a docs/progress-only class-metadata
commit; no runtime or PHPT runner files changed after this focused run.

Focused result:

| Selected | Runnable | Excluded | Passed | Failed | Skipped | Warned |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 46 | 13 | 33 | 13 | 0 | 0 | 0 |

Classifier split:

| Classification | Rows |
| --- | ---: |
| `process-boundary` | 25 |
| `harness-cleanup` | 8 |

## Runnable Path Rows

All 13 runnable filesystem/path rows passed:

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

These rows exercise stat/file metadata error handling, ordinary file type
metadata, and path existence behavior. They should stay in regression evidence;
they are not the next 25-row implementation target.

## Process Boundary Blocker

The 25 `process-boundary` rows are:

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

Generic support needs a native process runtime boundary shared by future
`proc_open()`, `proc_close()`, `proc_get_status()`, `proc_terminate()`,
`proc_nice()`, `exec()`, `system()`, `passthru()`, `shell_exec()`, and
`popen()` work. The missing semantics include descriptor array parsing, pipe
resources, process-resource lifetime, current-working-directory and environment
handling, multiplexing behavior, socket descriptor diagnostics, command
execution, status reporting, close/terminate behavior, and platform-aware
`proc_nice()` diagnostics.

## Harness Cleanup Blocker

The 8 `harness-cleanup` rows are:

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

These rows use PHPT `--CLEAN--` sections. Reopening them should happen through
runner support that executes cleanup/setup sections outside measured program
output. They are not evidence of missing `filegroup()`, `fileinode()`,
`fileowner()`, `fileperms()`, `filesize()`, `is_dir()`, or `is_file()` helpers
by themselves.

## Next Split

1. Implement a boxed process resource model for native child processes.
2. Add descriptor-spec parsing and pipe resource plumbing for `proc_open()`.
3. Share command execution, status, close, terminate, CWD, and environment
   handling with the future `exec()`/`system()`/`popen()` family.
4. Add platform-aware `proc_nice()` diagnostics through the same process
   boundary.
5. Extend the PHPT runner to execute cleanup/setup sections separately from
   measured output, then reclassify the 8 cleanup rows.

## Verification

```sh
cargo fmt --check
PHPT_PROGRESS_DIR=.runtime/ptn-wc6b-filesystem-progress-rebased \
  tools/run-bounded-phpt.sh tools/phpt-filesystem-path-process-manifest.txt
```
