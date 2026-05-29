# PHP Native Compiler Progress

Updated: 2026-05-29 02:43 CEST
Primary branch: `master`
Latest source head: `2911bd1f fix: add standard file metadata builtins`

## Progress Score

This file is the public progress report for the project. AO workers and the
supervisor must update this file before claiming public progress.

Progress is the pinned php-src PHPT full-suite pass rate:

`passed runnable PHPTs / total runnable PHPTs`

Current score: **1618 / 20294 pinned runnable PHPTs = 7.97%**.

No other percentage is public project progress. Focused PHPT passes, source
checkpoints, PRs, and docs/status edits are evidence for the next batch, but
they do not move this score unless a pinned full-suite run is completed,
parsed, regression-checked, and published here.

The first full-suite baseline was recorded for Batch 001 stack10 on php-src
`f97ff597429a2fe633665a7e02d97c8077f9f90f`, run
`phpt-full-batch001-20260528T010422Z-php-src-f97ff59-base-3e702be4-stack10`.
Counts: 1118 passed, 19156 failed, 964 skipped, 20 xfailed, 0 borked;
`run-tests.php` exited 1. Evidence lives under
`/home/claude/supervised-php-compiler/state/logs/phpt-full-batch001-20260528T010422Z-php-src-f97ff59-base-3e702be4-stack10`.

The Batch002 full-suite result was on the same php-src pin, run
`phpt-full-batch002-20260528T100640Z-php-src-f97ff59-public-bc0ed214-source-e72efe27-stack11`.
Counts: 1193 passed, 19081 failed, 964 skipped, 20 xfailed, 0 borked;
`run-tests.php` exited 1. Regressions from the Batch001 PASS set: 0. Evidence
lives under
`/home/claude/supervised-php-compiler/state/logs/phpt-full-batch002-20260528T100640Z-php-src-f97ff59-public-bc0ed214-source-e72efe27-stack11`.

The Batch003 full-suite result was on the same php-src pin, run
`phpt-full-batch003-20260528T154907Z-php-src-f97ff59-public-20f3be4c-source-202dd1ec-stack21`.
Counts: 1311 passed, 17129 failed, 967 skipped, 12 xfailed, 1839 borked;
`run-tests.php` exited 1. Regressions from the Batch002 PASS set: 0. Evidence
lives under
`/home/claude/supervised-php-compiler/state/logs/phpt-full-batch003-20260528T154907Z-php-src-f97ff59-public-20f3be4c-source-202dd1ec-stack21`.

Batch003 produced BORKED rows from PHPT harness/setup paths, so the public
score uses the stable pinned denominator and does not use the raw runner
`1311 / 18452 = 7.10%` calculation that excludes borked rows.

The Batch004 checkpoint8 regression-repair sharded gate on the same php-src pin
recorded 1369 / 20294 pinned runnable PHPTs = 6.75% with zero regressions from
the Batch003 PASS set. Evidence lives under
`/home/claude/supervised-php-compiler/state/logs/phpt-full-batch004-regression-repair-sharded-20260528T192018Z-php-src-f97ff59-public-3c86fc6a-source-b75047df-stack8`.

The Batch004 checkpoint10 sharded publication gate was on the same php-src pin,
run
`phpt-full-batch004-checkpoint10-sharded-20260528T195852Z-php-src-f97ff59-public-37941f23-source-241b8411-stack10`.
Counts: 1413 passed, 17771 failed, 2109 skipped, 16 xfailed, 1022 borked;
all 12 shards exited nonzero because failing PHPTs remain. Regressions from the
checkpoint8 PASS set: 0. Evidence lives under
`/home/claude/supervised-php-compiler/state/logs/phpt-full-batch004-checkpoint10-sharded-20260528T195852Z-php-src-f97ff59-public-37941f23-source-241b8411-stack10`.

Batch004 checkpoint10 still has BORKED rows, so the public score uses the
stable pinned denominator and does not use the raw runner
`1413 / 19200 = 7.36%` calculation.

The latest full-suite result is the Batch005 checkpoint10 sharded publication
gate on the same php-src pin, run
`phpt-full-batch005-checkpoint10-sharded-20260528T224229Z-php-src-f97ff59-public-fd74fba9-source-1c4da4c5-stack10`.
Counts: 1618 passed, 17025 failed, 2490 skipped, 15 xfailed, 1183 borked;
all 12 shards exited nonzero because failing PHPTs remain. The PASS-set
comparison reported three Batch004 PASS rows as SKIPPED; a focused rerun showed
all three are Windows-only PHPTs (`bug78220.phpt`,
`dirname_no_path_normalization-win32.phpt`, and `bug69115.phpt`), so this is
recorded as a platform-skip guard rather than a semantic regression. Evidence
lives under
`/home/claude/supervised-php-compiler/state/logs/phpt-full-batch005-checkpoint10-sharded-20260528T224229Z-php-src-f97ff59-public-fd74fba9-source-1c4da4c5-stack10`.

Batch005 checkpoint10 still has BORKED rows, so the public score uses the
stable pinned denominator and does not use the raw runner
`1618 / 18658 = 8.67%` calculation.

## PHPT Harness

| Item | State | Evidence |
| --- | --- | --- |
| php-src pin | Done | `/home/claude/php-src-phpt` at `f97ff597429a2fe633665a7e02d97c8077f9f90f` |
| Static inventory | Done | 21,827 PHPT files; 12,777 static runnable candidates |
| `phpc` PHPT wrapper | Done | `/home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper` |
| Skip/xfail ledger | Started | `/home/claude/supervised-php-compiler/state/php-core-suite-skip-ledger.tsv` |
| First full-suite baseline | Done | 1118 / 20294 runnable PHPTs passed (5.51%); run id `phpt-full-batch001-20260528T010422Z-php-src-f97ff59-base-3e702be4-stack10` |
| Batch002 full-suite gate | Done | 1193 / 20294 runnable PHPTs passed (5.88%); 0 regressions from Batch001 PASS set; run id `phpt-full-batch002-20260528T100640Z-php-src-f97ff59-public-bc0ed214-source-e72efe27-stack11` |
| Batch003 full-suite gate | Done | 1311 / 20294 pinned runnable PHPTs passed (6.46%); 0 regressions from Batch002 PASS set; run id `phpt-full-batch003-20260528T154907Z-php-src-f97ff59-public-20f3be4c-source-202dd1ec-stack21` |
| Batch004 checkpoint8 regression-repair sharded gate | Done | 1369 / 20294 pinned runnable PHPTs passed (6.75%); 0 regressions from Batch003 PASS set; run id `phpt-full-batch004-regression-repair-sharded-20260528T192018Z-php-src-f97ff59-public-3c86fc6a-source-b75047df-stack8` |
| Batch004 checkpoint10 sharded gate | Done | 1413 / 20294 pinned runnable PHPTs passed (6.96%); 0 regressions from checkpoint8 PASS set; run id `phpt-full-batch004-checkpoint10-sharded-20260528T195852Z-php-src-f97ff59-public-37941f23-source-241b8411-stack10` |
| Batch005 checkpoint10 sharded gate | Done | 1618 / 20294 pinned runnable PHPTs passed (7.97%); 3 PASS-to-SKIP platform guards from Windows-only PHPTs; run id `phpt-full-batch005-checkpoint10-sharded-20260528T224229Z-php-src-f97ff59-public-fd74fba9-source-1c4da4c5-stack10` |
| Batch004 source batch | Complete | Batch004 source checkpoints accepted: 10 / 10; checkpoint10 sharded gate published |
| Batch005 source batch | Complete | Batch005 source checkpoints accepted: 10 / 10; checkpoint10 sharded gate published |
| Batch006 source batch | Started | Source checkpoints accepted: 3 / 10 gate threshold; checkpoint3 integrates standard `fileinode()` / `fileowner()` / `filegroup()` / `filetype()` metadata builtins |

Focused PHPT history is tracked separately in
`/home/claude/supervised-php-compiler/state/php-core-suite-focused-history.tsv`.
Focused passes prove candidate direction; they do not define project percent.

## Current Integration

Batch006 source checkpoint 3 is primary-integrated under AO supervision. This
is a source checkpoint with focused proof, not a percentage change. The public
PHPT score remains **1618 / 20294 pinned runnable PHPTs = 7.97%** until the
next pinned full-suite or supervisor-approved sharded publication gate is
completed, regression-checked, and published here.

- primary source head:
  `2911bd1f fix: add standard file metadata builtins`
- reviewed and integration patch:
  `/home/claude/supervised-php-compiler/state/patches/ao-coder-standard-file-metadata-successor3-split-current-aa1b289e-20260529.patch`
- reviewed and integration patch SHA256:
  `8dec37f4eda2e204088cc55d69d31aa10ec213e1d49b065157e7f35418a96643`
- reviewer gate: phpc-18 recorded `FINAL GO-CANDIDATE /
  FOCUSED-GATES-PASS`; artifacts:
  `/home/claude/supervised-php-compiler/state/workers/batch006-review-standard-file-metadata-successor3-split-aa1b289e-20260529.{status.md,report.md,gates.log}`
- critic gate: phpc-33 recorded `SAFE-FOR-INTEGRATION /
  CURRENT-PUBLIC-AA1B289E`; artifacts:
  `/home/claude/supervised-php-compiler/state/workers/batch006-critic-standard-file-metadata-successor3-split-aa1b289e-20260529.{status.md,report.md}`
- integration handoff: phpc-38 recorded the exact current-public GO+SAFE pair
  as routeable under:
  `/home/claude/supervised-php-compiler/state/workers/ao-p28-replacement-handoff-watch-aa1b289e-20260529.{status.md,report.md,watch.log}`
- supervisor focused gates: PASS for `git apply --check`,
  `git diff --check`, `cargo fmt --all -- --check`, focused Rust
  `standard_file_metadata_builtins`, `cargo build -p phpc`, and the focused
  four-file PHPT metadata split
- full PHPT suite: not run for this single source checkpoint; Batch006 broad
  gate is held until 10 accepted source checkpoints, or until the supervisor
  explicitly opens a regression/publication gate

This checkpoint generalizes local filesystem metadata builtins for
`fileinode()`, `fileowner()`, `filegroup()`, and `filetype()` through shared
path/stat helpers and native known-function registration. It deliberately
splits out the larger metadata family rows that still need `touch()`,
`tempnam()`, and warning-diagnostic support. This is not keyed to a PHPT
filename, expected-output fixture, batch marker, public hash, or test-name
branch.

Previous Batch006 source checkpoint 2 was `func_get*` call-frame semantics:

Batch006 source checkpoint 2 is primary-integrated under AO supervision. This
is a source checkpoint with focused proof, not a percentage change. The public
PHPT score remains **1618 / 20294 pinned runnable PHPTs = 7.97%** until the
next pinned full-suite or supervisor-approved sharded publication gate is
completed, regression-checked, and published here.

- primary source head:
  `0f844622 fix: add func_get call-frame semantics`
- reviewed and integration patch:
  `/home/claude/supervised-php-compiler/state/patches/ao-integration-p51-func-get-successor2-5bf8ccd3-20260529.patch`
- reviewed and integration patch SHA256:
  `d40babd8f5b3d7c2fe1ec98d7b1707ddb4398882ea5f2ce549db77fce5177d3e`
- staged integration diff:
  `/home/claude/supervised-php-compiler/state/patches/ao-integration-p51-func-get-callframe-successor2-5bf8ccd3-20260529.staged.patch`
- staged integration diff SHA256:
  `fc69927875edd1e377ffd8ad871ede43f9357f98393fb22f09168f328b79d19c`
- reviewer gate: p7 recorded `FINAL GO-CANDIDATE /
  FOCUSED-GATES-PASS / PHPT-PASS`; artifacts:
  `/home/claude/supervised-php-compiler/state/workers/batch006-review-dynamic-reference-func-get-callframe-successor2-5bf8ccd3-20260529.{status.md,report.md,gates.log}`
- critic gate: phpc-33 recorded `SAFE-FOR-INTEGRATION`; artifacts:
  `/home/claude/supervised-php-compiler/state/workers/batch006-critic-dynamic-reference-func-get-callframe-successor2-5bf8ccd3-20260529.{status.md,report.md}`
- integration handoff: phpc-38 recorded `FINAL-HANDOFF /
  READY-FOR-SUPERVISOR-APPLY`; artifacts:
  `/home/claude/supervised-php-compiler/state/workers/ao-integration-p51-func-get-successor2-5bf8ccd3-20260529.{status.md,report.md,gates.log}`
- supervisor focused gates: PASS for `git apply --check`,
  `git diff --check`, `cargo fmt --all -- --check`, focused Rust
  `dynamic_features func_get`, focused Rust
  `forbidden_scope_introspection_builtins_report_dynamic_call_error`,
  `cargo build -p phpc`, and the focused 20-file PHPT `func_get*` /
  `func_num_args()` cluster
- full PHPT suite: not run for this single source checkpoint; Batch006 broad
  gate is held until 10 accepted source checkpoints, or until the supervisor
  explicitly opens a regression/publication gate

This checkpoint generalizes active user call-frame argument visibility for
`func_get_arg()`, `func_get_args()`, and `func_num_args()`, including extra
positional arguments, global-scope and invalid-position PHP-shaped diagnostics,
and fully-qualified global constant reads through the existing constant path.
The generator/yield row remains a separate runtime/source blocker. This is not
keyed to a PHPT filename, expected-output fixture, batch marker, public hash, or
test-name branch.

Previous Batch006 source checkpoint 1 was C/POSIX locale support:

Batch006 source checkpoint 1 is primary-integrated under AO supervision. This
is a source checkpoint with focused proof, not a percentage change. The public
PHPT score remained **1618 / 20294 pinned runnable PHPTs = 7.97%** until the
next pinned full-suite or supervisor-approved sharded publication gate is
completed, regression-checked, and published here.

- primary source head:
  `94a9cfd9 fix: add C locale setlocale and strcoll`
- reviewed patch:
  `/home/claude/supervised-php-compiler/state/patches/ao-coder-ext-standard-setlocale-strcoll-c-locale-capability-phpc42-fd74fba9-20260529.patch`
- reviewed and integration patch SHA256:
  `3d0d552b7c40a35e654c69c37f009575e18cccb3480e2faf0a4b2fae07709812`
- current-public integration patch:
  `/home/claude/supervised-php-compiler/state/patches/ao-integration-p42-setlocale-strcoll-7088dd77-20260529.patch`
- reviewer gate: p7 recorded `FINAL GO-CANDIDATE /
  FOCUSED-GATES-PASS / CORRECTED-PHPT-EVIDENCE-PASS`; artifacts:
  `/home/claude/supervised-php-compiler/state/workers/batch006-review-ext-standard-setlocale-strcoll-fd74fba9-20260529.{status.md,report.md,gates.log}`
- critic gate: phpc-33 recorded `SAFE-FOR-INTEGRATION`; artifacts:
  `/home/claude/supervised-php-compiler/state/workers/batch006-critic-ext-standard-setlocale-strcoll-fd74fba9-20260529.{status.md,report.md}`
- integration handoff: p28 recorded `FINAL-HANDOFF /
  READY-FOR-SUPERVISOR-APPLY`; artifacts:
  `/home/claude/supervised-php-compiler/state/workers/ao-integration-p42-setlocale-strcoll-7088dd77-20260529.{status.md,report.md,gates.log}`
- supervisor focused gates: PASS for patch SHA, `git apply --check`,
  `git diff --check`, docs/PROGRESS/examples exclusion, production
  exact-shape marker audit, `cargo fmt --all -- --check`, focused Rust
  `setlocale_capability_builtin` (4/4), `cargo build -p phpc`, and focused
  PHPT `ext/standard/tests/strings/strcoll.phpt` plus custom C/POSIX
  `setlocale()` coverage (2/2)
- full PHPT suite: not run for this single source checkpoint; Batch006 broad
  gate is held until 10 accepted source checkpoints, or until the supervisor
  explicitly opens a regression/publication gate

This checkpoint generalizes bounded C/POSIX locale support: `LC_*` constants,
`setlocale()` metadata and argument handling for the supported subset, and
C-locale byte collation for `strcoll()`. It is not keyed to a PHPT filename,
expected-output fixture, batch marker, public hash, or test-name branch.

Previous Batch005 source checkpoint 10 was standard array behavior, and the
Batch005 checkpoint10 sharded publication gate is recorded above as the latest
full-suite score publication.

Previous Batch005 source checkpoint 9 was date/timezone scalar behavior:

Batch005 source checkpoint 9 is primary-integrated under AO supervision. This
is a source checkpoint with focused proof, not a percentage change. The public
PHPT score remains **1413 / 20294 pinned runnable PHPTs = 6.96%** until the
next pinned full-suite or supervisor-approved sharded publication gate is
completed, regression-checked, and published here.

- primary source head:
  `6d0ab7d4 fix: add core date and timezone builtins`
- reviewed patch:
  `/home/claude/supervised-php-compiler/state/patches/ao-coder-date-timezone-current-37941f23-20260528.patch`
- reviewed patch SHA256:
  `98e1320df09e0d7f0a9692b1d0b42cefa619eb886e54e52e8184b4323f1659db`
- reviewer gate: p7 recorded `FINAL GO-CANDIDATE / FOCUSED-GATES-PASS /
  PHPT-PASS`; artifacts:
  `/home/claude/supervised-php-compiler/state/workers/batch005-review-date-timezone-current-4ab71273-20260529.{status.md,report.md}`
- critic gate: phpc-33 recorded `SAFE-FOR-INTEGRATION`; artifacts:
  `/home/claude/supervised-php-compiler/state/workers/batch005-critic-date-timezone-current-4ab71273-20260529.{status.md,report.md}`
- integration handoff: the supervisor applied the exact p7+phpc-33 SAFE patch
  directly because it applied cleanly after checkpoint8
- supervisor focused gates: PASS for patch SHA, `git apply --check`,
  `git diff --check`, production exact-shape marker audit,
  `cargo fmt --all -- --check`, focused Rust
  `date_time_builtin`, `cargo build -p phpc`, and focused PHPT cluster
  `ext/date/tests/003.phpt`, `004.phpt`, `006.phpt`, `007.phpt`, and
  `008.phpt`
- full PHPT suite: not run for this single source checkpoint; next broad gate
  is due after 10 accepted Batch005 source checkpoints or explicit regression
  repair

This checkpoint generalizes core date/timezone scalar behavior for
`time()`, `mktime()`, `gmmktime()`, `date()`, `gmdate()`, `idate()`,
`checkdate()`, `getdate()`, `localtime()`,
`date_default_timezone_get()`, and `date_default_timezone_set()`. It adds
bounded timezone state and deterministic request-time handling instead of
hard-coding individual PHPT outputs.

Previous Batch005 source checkpoint 8 was call-user-function argument order:

Batch005 source checkpoint 8 is primary-integrated under AO supervision. This
is a source checkpoint with focused proof, not a percentage change. The public
PHPT score remains **1413 / 20294 pinned runnable PHPTs = 6.96%** until the
next pinned full-suite or supervisor-approved sharded publication gate is
completed, regression-checked, and published here.

- primary source head:
  `bb3852a8 fix: preserve call_user_func spread source order`
- reviewed patch:
  `/home/claude/supervised-php-compiler/state/patches/ao-coder-call-argument-unpack-source-order-call-user-func-7c5484a6-20260528.patch`
- reviewed patch SHA256:
  `5e5b1410c57bb752add4eb0f26315d185276dedf38e0e49092de421c4ba4f383`
- reviewer gate: p7 recorded `FINAL GO-CANDIDATE /
  CURRENT-HEAD-EQUIVALENT / BASELINE-DIRECT-PHPT-BLOCKER-PROVEN /
  CUSTOM-PHPT-PASS / FOCUSED-GATES-PASS` with `FINAL_FAIL=0`; artifacts:
  `/home/claude/supervised-php-compiler/state/workers/batch005-review-call-argument-unpack-source-order-call-user-func-7c5484a6-20260528.{status.md,report.md,gates.log}`
- critic gate: phpc-33 recorded `SAFE-FOR-INTEGRATION`; artifacts:
  `/home/claude/supervised-php-compiler/state/workers/batch005-critic-call-argument-unpack-source-order-call-user-func-7c5484a6-20260528.{status.md,report.md}`
- integration handoff: the supervisor applied the exact p7+phpc-33 SAFE patch
  directly because it applied cleanly after checkpoint7
- supervisor focused gates: PASS for patch SHA, `git apply --check`,
  `git diff --check`, docs/PROGRESS/examples exclusion, added-line production
  exact-shape marker audit, `cargo fmt --all -- --check`, focused Rust
  `dynamic_features call_user_func_argument_unpacking_reuses_source_order_named_binding`,
  `cargo build -p phpc`, and custom focused PHPT
  `ao-coder-call-argument-unpack-source-order-call-user-func-7c5484a6-20260528.phpt`
- supervisor primary gate log:
  `/home/claude/supervised-php-compiler/state/workers/supervisor-primary-p31-source-order-batch005-checkpoint8-20260529.gates.log`
- full PHPT suite: not run for this single source checkpoint; next broad gate
  is due after 10 accepted Batch005 source checkpoints or explicit regression
  repair

This checkpoint generalizes `call_user_func()` spread/named argument handling
for user-function dispatch. Spread arguments now reuse the source-order array
spread evaluation path and carry argument keys through call-frame binding, so
named and variadic keys are preserved without hard-coding a PHPT row. The
pinned php-src `call_user_func.phpt` row is still baseline-blocked by an
unsupported anonymous-class parser case, proven against an unpatched binary,
so the custom focused PHPT isolates the source-order behavior.

Previous Batch005 source checkpoint 7 was stream-wrapper origin metadata:

Batch005 source checkpoint 7 is primary-integrated under AO supervision. This
is a source checkpoint with focused proof, not a percentage change. The public
PHPT score remains **1413 / 20294 pinned runnable PHPTs = 6.96%** until the
next pinned full-suite or supervisor-approved sharded publication gate is
completed, regression-checked, and published here.

- primary source head:
  `8b415d7e fix: track stream wrapper origins`
- reviewed patch:
  `/home/claude/supervised-php-compiler/state/patches/ao-coder-ext-standard-stream-wrapper-registry-origin-metadata-successor2-phpc42-7c5484a6-20260528.patch`
- reviewed patch SHA256:
  `9c92e2aedeedc51570e4524ce9dc803a93c15e379bb8482cfb4534aaee5f1bc6`
- reviewer gate: reviewer-B/phpc-48 recorded `FINAL GO-CANDIDATE` with
  current-public focused proof; artifacts:
  `/home/claude/supervised-php-compiler/state/workers/batch005-review-stream-wrapper-origin-metadata-7c5484a6-20260528.{status.md,report.md,gates.log}`
- critic gate: phpc-49 recorded `SAFE-FOR-INTEGRATION /
  REVIEWER-B-FINAL-GO-VERIFIED / READ-ONLY`; artifacts:
  `/home/claude/supervised-php-compiler/state/workers/batch005-critic-stream-wrapper-origin-metadata-7c5484a6-20260528.{status.md,report.md}`
- integration handoff: the supervisor applied the exact reviewer-B+phpc-49
  SAFE patch directly to avoid parking a safe checkpoint behind handoff churn
- supervisor focused gates: PASS for patch SHA, `git apply --check`,
  `git diff --check`, docs/PROGRESS/examples exclusion, added-line production
  exact-shape marker audit, `cargo fmt --all -- --check`, focused Rust
  `stream_resource_builtin stream_wrapper`, `cargo build -p phpc`, and
  focused PHPT `ext/standard/tests/streams/stream_get_wrappers.phpt`
- supervisor primary gate log:
  `/home/claude/supervised-php-compiler/state/workers/supervisor-primary-p42-stream-wrapper-origin-batch005-checkpoint7-20260529.gates.log`
- full PHPT suite: not run for this single source checkpoint; next broad gate
  is due after 10 accepted Batch005 source checkpoints or explicit regression
  repair

This checkpoint generalizes stream wrapper registry state by storing
builtin/user origin metadata plus user wrapper class metadata. User
registration, unregister, restore, and wrapper listing now distinguish
built-in protocols from user replacements instead of treating every protocol
as a bare string. True user stream callback I/O remains a separate unsupported
split. Production code is not keyed to a PHPT filename, expected-output
fixture, batch marker, public hash, or test-name branch.

Previous Batch005 source checkpoint 6 was local filesystem and stream-adjacent
behavior:

Batch005 source checkpoint 6 is primary-integrated under AO supervision. This
is a source checkpoint with focused proof, not a percentage change. The public
PHPT score remains **1413 / 20294 pinned runnable PHPTs = 6.96%** until the
next pinned full-suite or supervisor-approved sharded publication gate is
completed, regression-checked, and published here.

- primary source head:
  `2beac3ba fix: cover local filesystem file operations`
- reviewed patch:
  `/home/claude/supervised-php-compiler/state/patches/ao-coder-standard-file-stream-successor2-current-e3c73bab-20260528.patch`
- reviewed patch SHA256:
  `6d7f7e918473e3b05954b8f89099ac444494c162c077ad091f3c5d61fe983218`
- reviewer gate: p7 recorded `FINAL GO-CANDIDATE /
  FOCUSED-GATES-PASS / PHPT-PASS` with `FINAL_FAIL=0`; artifacts:
  `/home/claude/supervised-php-compiler/state/workers/batch005-review-standard-file-stream-successor2-da8720d0-20260528.{status.md,report.md,gates.log}`
- critic gate: phpc-33 recorded `SAFE-FOR-INTEGRATION /
  REVIEWER-FINAL-GO-VERIFIED / READ-ONLY`; artifacts:
  `/home/claude/supervised-php-compiler/state/workers/batch005-critic-standard-file-stream-successor2-da8720d0-20260528.{status.md,report.md}`
- integration handoff: p28 was routed but the supervisor applied the exact
  p7+phpc-33 reviewed patch directly to avoid leaving a safe checkpoint idle
- supervisor focused gates: PASS for patch SHA, `git diff --check`,
  docs/PROGRESS/examples exclusion, corrected added-line production
  exact-shape marker audit, `cargo fmt --all -- --check`, focused Rust
  `standard_filesystem_mutation_builtins`, `cargo build -p phpc`, and a
  corrected clean focused PHPT file/stream cluster with 10/10 PASS rows
- supervisor primary gate log:
  `/home/claude/supervised-php-compiler/state/workers/supervisor-primary-p43-standard-file-stream-batch005-checkpoint6-20260528.gates.log`
- full PHPT suite: not run for this single source checkpoint; next broad gate
  is due after 10 accepted Batch005 source checkpoints or explicit regression
  repair

This checkpoint generalizes local filesystem and stream-adjacent behavior for
the supported native runtime subset. It registers and implements common file
and directory builtins, bounded local path resolution, stat cache handling,
file mutation/read helpers, `mkdir()`/`rmdir()`/`copy()`/`rename()`/`chdir()`/
`scandir()` behavior, file mode `x` support, stream resource boundary checks,
and related string/format helpers needed by the file/dir PHPT cluster.
Production code is not keyed to a PHPT filename, expected-output fixture,
batch marker, public hash, or test-name branch.

Previous Batch005 source checkpoint 5 was reference-taking builtin named-array
behavior:

Batch005 source checkpoint 5 is primary-integrated under AO supervision. This
is a source checkpoint with focused proof, not a percentage change. The public
PHPT score remains **1413 / 20294 pinned runnable PHPTs = 6.96%** until the
next pinned full-suite or supervisor-approved sharded publication gate is
completed, regression-checked, and published here.

- primary source head:
  `60d5ff6f fix: bind named arrays for reference builtins`
- reviewed patch:
  `/home/claude/supervised-php-compiler/state/patches/ao-coder-call-argument-unpack-reference-builtin-named-array-e3c73bab-20260528.patch`
- reviewed patch SHA256:
  `6f1c640d048b44a5f933f10219e1d2260ef3a3f1c1cabfb10eb5d30f5b0d20c1`
- reviewer gate: reviewer-B/phpc-48 recorded `FINAL GO-CANDIDATE /
  FOCUSED-GATES-PASS / PHPT-PASS` with `FINAL_FAIL=0`; artifacts:
  `/home/claude/supervised-php-compiler/state/workers/batch005-review-call-argument-unpack-reference-builtin-named-array-e3c73bab-20260528.{status.md,report.md,gates.log}`
- critic gate: phpc-49 recorded `SAFE-FOR-INTEGRATION /
  REVIEWER-B-FINAL-GO-VERIFIED / READ-ONLY`; artifacts:
  `/home/claude/supervised-php-compiler/state/workers/batch005-critic-call-argument-unpack-reference-builtin-named-array-e3c73bab-20260528.{status.md,report.md}`
- integration handoff: phpc-28 recorded `FINAL-HANDOFF /
  READY-FOR-SUPERVISOR-APPLY` after scratch gates; artifacts:
  `/home/claude/supervised-php-compiler/state/workers/batch005-integration-call-argument-unpack-reference-builtin-named-array-e3c73bab-20260528.{status.md,report.md,gates.log}`
- supervisor focused gates: PASS for patch SHA, `git apply --check`,
  `git diff --check`, docs/PROGRESS/examples exclusion, production
  exact-shape marker audit, `cargo fmt --all -- --check`, focused Rust
  `call_user_func_array_binds_named_reference_arguments_for_builtin_callbacks`,
  `cargo build -p phpc`, and focused custom PHPT
  `call_user_func_array` reference-builtin named-array row
- supervisor primary gate log:
  `/home/claude/supervised-php-compiler/state/workers/supervisor-primary-p31-reference-builtin-batch005-checkpoint5-20260528.gates.log`
- full PHPT suite: not run for this single source checkpoint; next broad gate
  is due after 10 accepted Batch005 source checkpoints or explicit regression
  repair

This checkpoint generalizes `call_user_func_array()` for reference-taking
builtin callbacks with string-keyed argument arrays. Named arguments now bind
through reflection parameter metadata for reference-capable builtins such as
`array_pop()`, `array_unshift()`, `ksort()`, and `next()`, preserving the first
by-reference argument cell when available and falling back through the shared
builtin callback path otherwise. Production code is not keyed to a PHPT
filename, expected-output fixture, batch marker, public hash, or test-name
branch.

Previous Batch005 source checkpoint 4 was metadata-backed builtin named-array
behavior:

Batch005 source checkpoint 4 is primary-integrated under AO supervision. This
is a source checkpoint with focused proof, not a percentage change. The public
PHPT score remains **1413 / 20294 pinned runnable PHPTs = 6.96%** until the
next pinned full-suite or supervisor-approved sharded publication gate is
completed, regression-checked, and published here.

- primary source head:
  `251d8c0e fix: bind named arrays for builtin callbacks`
- reviewed patch:
  `/home/claude/supervised-php-compiler/state/patches/ao-coder-call-argument-unpack-builtin-named-array-afe8dfae-20260528.patch`
- reviewed patch SHA256:
  `9f74d6c87a269a0abec27b4f5255c8789e78a57d17ec92336d3d05ed6b0a2883`
- reviewer gate: p7 recorded `FINAL GO-CANDIDATE /
  FOCUSED-GATES-PASS / PHPT-PASS` with `FINAL_FAIL=0`; artifacts:
  `/home/claude/supervised-php-compiler/state/workers/batch005-review-call-argument-unpack-builtin-named-array-afe8dfae-20260528.{status.md,report.md,gates.log}`
- critic gate: phpc-33 recorded `SAFE-FOR-INTEGRATION /
  P7-FINAL-GO-VERIFIED / READ-ONLY`; artifacts:
  `/home/claude/supervised-php-compiler/state/workers/batch005-critic-call-argument-unpack-builtin-named-array-afe8dfae-20260528.{status.md,report.md}`
- integration handoff: phpc-28 recorded a ready no-cargo handoff because the
  strict cargo cap was occupied; supervisor applied the exact p7+phpc-33
  reviewed patch to avoid stalling checkpoint4
- supervisor focused gates: PASS for patch SHA, `git apply --check`,
  `git diff --check`, docs/PROGRESS/examples exclusion, production
  exact-shape marker audit, `cargo fmt --all -- --check`, focused Rust
  `call_user_func_array_binds_named_arguments_for_metadata_backed_builtins`,
  `cargo build -p phpc`, and focused custom PHPT
  `call_user_func_array` metadata-backed builtin named-array row
- supervisor primary gate log:
  `/home/claude/supervised-php-compiler/state/workers/supervisor-primary-p31-builtin-named-array-batch005-checkpoint4-20260528.gates.log`
- full PHPT suite: not run for this single source checkpoint; next broad gate
  is due after 10 accepted Batch005 source checkpoints or explicit regression
  repair

This checkpoint generalizes `call_user_func_array()` for metadata-backed
builtin callbacks. String-keyed argument arrays now bind through shared
reflection parameter metadata, including duplicate and unknown named argument
diagnostics, positional-after-named checks, default handling, required arity,
and explicit unsupported boundaries for reference-taking or metadata-missing
builtins. Production code is not keyed to a PHPT filename, expected-output
fixture, batch marker, public hash, or test-name branch.

Previous Batch005 source checkpoint 3 was class-like metadata existence
behavior:

Batch005 source checkpoint 3 is primary-integrated under AO supervision. This
is a source checkpoint with focused proof, not a percentage change. The public
PHPT score remains **1413 / 20294 pinned runnable PHPTs = 6.96%** until the
next pinned full-suite or supervisor-approved sharded publication gate is
completed, regression-checked, and published here.

- primary source head:
  `268ef6ea fix: generalize class metadata existence checks`
- reviewed patch:
  `/home/claude/supervised-php-compiler/state/patches/ao-coder-zend-type-class-metadata-namespace-current-8eb41150-20260528.patch`
- reviewed patch SHA256:
  `459f26dbb79cb6121130ce98483ffbb7f4322a7b88c99274f5d6c1e48f1978be`
- reviewer gate: p7 recorded `FINAL GO-CANDIDATE /
  CURRENT-PUBLIC-9109ED01-SOURCE-EQUIVALENT / FOCUSED-GATES-PASS /
  PHPT-PASS` with `FINAL_FAIL=0`; artifacts:
  `/home/claude/supervised-php-compiler/state/workers/batch005-review-zend-type-class-metadata-namespace-8eb41150-20260528.{status.md,report.md,gates.log}`
- critic gate: phpc-33 recorded
  `SAFE-FOR-INTEGRATION-ON-9109ED01 /
  CURRENT-PUBLIC-SOURCE-EQUIVALENCE-VERIFIED / READ-ONLY`; artifacts:
  `/home/claude/supervised-php-compiler/state/workers/batch005-critic-zend-type-class-metadata-namespace-8eb41150-20260528.{status.md,report.md}`
- integration handoff: p28 reached the handoff route but was blocked on stale
  cargo-slot state; supervisor applied the exact p7+phpc-33 reviewed patch to
  avoid stalling checkpoint3
- supervisor focused gates: PASS for patch SHA, `git apply --check`,
  `git diff --check`, docs/PROGRESS/examples exclusion, production
  exact-shape marker audit, `cargo fmt --all -- --check`, focused Rust
  `magic_namespace_constant_uses_current_namespace_context`, focused Rust
  `metadata_exists_coerces_scalar_names_and_rejects_non_scalars`, `cargo build
  -p phpc`, focused PHPT `class_exists_001.phpt` and `class_exists_002.phpt`,
  and focused PHPT `interface_exists_001.phpt` plus
  `traits/trait_exists_001.phpt`
- supervisor primary gate log:
  `/home/claude/supervised-php-compiler/state/workers/supervisor-primary-p38-zend-type-class-metadata-batch005-checkpoint3-20260528.gates.log`
- full PHPT suite: not run for this single source checkpoint; next broad gate
  is due after 10 accepted Batch005 source checkpoints or explicit regression
  repair

This checkpoint generalizes class-like metadata existence behavior. The parser
now lowers `__NAMESPACE__` from namespace context, and shared runtime handling
for `class_exists()`, `interface_exists()`, `trait_exists()`, and
`enum_exists()` coerces string-compatible scalar names while rejecting
non-scalars. The direct `enum_exists.phpt` row remains blocked by a separate
nested-enum parser gap. Production code is not keyed to a PHPT filename,
expected-output fixture, batch marker, public hash, or test-name branch.

Previous Batch005 source checkpoint 2 was truthful stream wrapper capability
reporting:

Batch005 source checkpoint 2 is primary-integrated under AO supervision. This
is a source checkpoint with focused proof, not a percentage change. The public
PHPT score remains **1413 / 20294 pinned runnable PHPTs = 6.96%** until the
next pinned full-suite or supervisor-approved sharded publication gate is
completed, regression-checked, and published here.

- primary source head:
  `fd8be20d fix: report supported stream wrappers`
- reviewed patch:
  `/home/claude/supervised-php-compiler/state/patches/ao-coder-standard-stream-get-wrappers-capability-source-candidate-successor2-stack-after-p31-8eb41150-20260528.patch`
- reviewed patch SHA256:
  `c649759528b06ed84f7741213c545071d26338377a3a59906bae307623f4b434`
- reviewer gate: p7 recorded `FINAL GO-CANDIDATE /
  FOCUSED-GATES-PASS / PHPT-PASS` with `FINAL_FAIL=0` and canonical
  stack-after-p31 equivalence; artifacts:
  `/home/claude/supervised-php-compiler/state/workers/batch005-review-standard-stream-get-wrappers-successor2-stack-after-p31-8eb41150-20260528.{status.md,report.md,gates.log}`
- critic gate: phpc-33 recorded `SAFE-FOR-INTEGRATION /
  P7-FINAL-GO-VERIFIED / CANONICAL-EQUIVALENCE-PASS / READ-ONLY`;
  artifacts:
  `/home/claude/supervised-php-compiler/state/workers/batch005-critic-standard-stream-get-wrappers-successor2-stack-after-p31-8eb41150-20260528.{status.md,report.md}`
- integration handoff: phpc-28 acknowledged the exact p7+phpc-33 handoff but
  stayed `WAITING-FOR-CARGO-SLOT` on a stale cargo sample; supervisor applied
  the exact reviewed patch to avoid stalling the accepted checkpoint
- supervisor focused gates: PASS for patch SHA, `git apply --check`,
  `git diff --check`, docs/PROGRESS/examples exclusion, production
  exact-shape marker audit, `cargo fmt --all -- --check`, focused Rust
  `stream_get_wrappers_reports_truthful_current_stream_subset`, focused Rust
  `emit_ir_folds_stream_metadata_but_rejects_direct_calls`, `cargo build -p
  phpc`, focused custom `stream_get_wrappers` PHPT via `run-tests.php`, and
  direct `opendir-001.phpt` skipif proof via `run-tests.php`
- supervisor primary gate log:
  `/home/claude/supervised-php-compiler/state/workers/supervisor-primary-p34-stream-get-wrappers-batch005-checkpoint2-20260528.gates.log`
- full PHPT suite: not run for this single source checkpoint; next broad gate
  is due after 10 accepted Batch005 source checkpoints or explicit regression
  repair

This checkpoint generalizes stream wrapper capability reporting.
`stream_get_wrappers()` is now advertised in builtin/native metadata and
returns the currently implemented `file` and `php` wrappers, while unsupported
wrappers remain absent so PHP core skipifs can make truthful decisions. The
direct `stream_get_wrappers.phpt` row still needs generalized
`stream_wrapper_register()` mutation support, tracked as a separate blocker.
Production code is not keyed to a PHPT filename, expected-output fixture,
batch marker, public hash, or test-name branch.

Previous Batch005 source checkpoint 1 was `call_user_func_array()` named and
variadic argument handling:

Batch005 source checkpoint 1 is primary-integrated under AO supervision. This
is a source checkpoint with focused proof, not a percentage change. The public
PHPT score remains **1413 / 20294 pinned runnable PHPTs = 6.96%** until the
next pinned full-suite or supervisor-approved sharded publication gate is
completed, regression-checked, and published here.

- primary source head:
  `4e129363 fix: preserve named variadic keys in call_user_func_array`
- exported integration patch:
  `/home/claude/supervised-php-compiler/state/patches/ao-integration-p31-call-user-func-array-named-variadic-d97da76e-20260528.patch`
- exported integration patch SHA256:
  `9fd36c727642483393657496e942dfc913145da92bb9bafcee2c5448f519ef02`
- reviewer gate: p7 recorded `FINAL GO-CANDIDATE / FOCUSED-GATES-PASS /
  PHPT-PASS / DIRECT-ROW-BASELINE-BLOCKED` with `FINAL_FAIL=0`; artifacts:
  `/home/claude/supervised-php-compiler/state/workers/batch004-review-call-user-func-array-named-variadic-37941f23-20260528.{status.md,report.md,gates.log}`
- critic gate: phpc-33 recorded `SAFE-FOR-INTEGRATION /
  P7-FINAL-GO-NORMALIZED / BASELINE-BLOCKER-ACCEPTED / READ-ONLY`;
  artifacts:
  `/home/claude/supervised-php-compiler/state/workers/batch004-critic-call-user-func-array-named-variadic-37941f23-20260528.{status.md,report.md}`
- integration gate: phpc-28 recorded
  `READY-FOR-SUPERVISOR-APPLY-ON-D97DA76E`; artifacts:
  `/home/claude/supervised-php-compiler/state/workers/ao-integration-p31-call-user-func-array-named-variadic-d97da76e-20260528.{status.md,report.md,gates.log}`
- supervisor focused gates: PASS for `git apply --check`, patch SHA,
  `git diff --check`, docs/PROGRESS/examples exclusion, production exact-shape
  audit, `cargo fmt --all -- --check`, focused Rust
  `call_user_func_array_preserves_named_variadic_argument_keys`,
  `cargo build -p phpc`, and focused wrapper PHPT custom
  `call_user_func_array` named/variadic row
- supervisor primary gate log:
  `/home/claude/supervised-php-compiler/state/workers/supervisor-primary-p31-call-user-func-array-named-variadic-batch005-checkpoint1-20260528.gates.log`
- full PHPT suite: not run for this single source checkpoint; next broad gate
  is due after 10 accepted Batch005 source checkpoints or explicit regression
  repair

This checkpoint generalizes `call_user_func_array()` user-function argument
handling. User-function frames now preserve supplied argument keys through
named/variadic evaluation, including duplicate/unknown/subset error paths and
keyed variadic rest storage. Production code is not keyed to a PHPT filename,
expected-output fixture, batch marker, public hash, or test-name branch.

Previous Batch004 source checkpoint 10 was standard byte-string builtins:

Batch004 source checkpoint 10 is primary-integrated under AO supervision. The
checkpoint10 sharded publication gate is now published with **1413 / 20294
pinned runnable PHPTs = 6.96%** and zero regressions from the checkpoint8 PASS
set.

- primary source head:
  `241b8411 fix: preserve byte strings in standard builtins`
- exported integration patch:
  `/home/claude/supervised-php-compiler/state/patches/ao-integration-p34-standard-string-byte-builtins-successor4-a12390b9-20260528.patch`
- exported integration patch SHA256:
  `03f7e13d4544de670f012035ff48f2f7caa9d0d62d882245f82a946d70b720b1`
- reviewer gate: p7 recorded `FINAL GO-CANDIDATE / FOCUSED-GATES-PASS /
  PHPT-PASS / SOURCE-EQUIVALENT-CHECKPOINT9` with `FINAL_FAIL=0`; artifacts:
  `/home/claude/supervised-php-compiler/state/workers/batch004-review-standard-string-byte-builtins-successor4-stack-after-p31-8dafddc4-20260528.{status.md,report.md,gates.log}`
- critic gate: phpc-33 recorded `SAFE-FOR-INTEGRATION /
  P7-FINAL-GO-VERIFIED / SOURCE-EQUIVALENT-CHECKPOINT9 / READ-ONLY`;
  artifacts:
  `/home/claude/supervised-php-compiler/state/workers/batch004-critic-standard-string-byte-builtins-successor4-stack-after-p31-8dafddc4-20260528.{status.md,report.md}`
- integration gate: phpc-28 recorded `FINAL-HANDOFF /
  READY-FOR-SUPERVISOR-APPLY`; artifacts:
  `/home/claude/supervised-php-compiler/state/workers/ao-integration-p34-standard-string-byte-builtins-successor4-a12390b9-20260528.{status.md,report.md,gates.log}`
- supervisor focused gates: PASS after restoring the reviewed
  `compiler/tests/str_repeat_builtin.rs` file that the p28 exported patch had
  omitted; PASS for `git diff --check`, docs/PROGRESS/examples exclusion,
  production exact-shape audit, `cargo fmt --all -- --check`, focused Rust
  `cargo test -p phpc --test str_repeat_builtin -- --test-threads=1` (6/6),
  `cargo build -p phpc`, and the focused wrapper PHPT strings cluster (4/4)
- supervisor primary gate log:
  `/home/claude/supervised-php-compiler/state/workers/supervisor-primary-p34-standard-string-byte-successor4-checkpoint10-20260528.gates.log`
- full PHPT suite: supervisor-approved checkpoint10 sharded publication gate
  completed as
  `phpt-full-batch004-checkpoint10-sharded-20260528T195852Z-php-src-f97ff59-public-37941f23-source-241b8411-stack10`;
  counts were 1413 passed, 17771 failed, 2109 skipped, 16 xfailed, 1022
  borked; checkpoint8 PASS regressions were 0

This checkpoint generalizes byte-string handling for standard builtins. It
adds shared `chr`, `bin2hex`, and `str_repeat` builtin metadata and runtime
semantics, preserves binary bytes through echo/concat/string helper paths, and
adds `ValueError` metadata used by negative `str_repeat()` behavior. Production
code is not keyed to a PHPT filename, expected-output fixture, batch marker, or
test-name branch.

Previous Batch004 source checkpoint 9 was builtin callback unpacking:

Batch004 source checkpoint 9 is primary-integrated under AO supervision. This
is a source checkpoint with focused proof, not a percentage change. The public
PHPT score remains **1369 / 20294 pinned runnable PHPTs = 6.75%** until the
next pinned full-suite or supervisor-approved sharded full-suite gate is
completed, regression-checked, and published here.

- primary source head:
  `7ae57dc2 fix: expand unpacked arguments for builtin callbacks`
- exported integration patch:
  `/home/claude/supervised-php-compiler/state/patches/ao-integration-p31-builtin-callback-successor2-8dafddc4-20260528.patch`
- exported integration patch SHA256:
  `1e879023185ee515ceca688999bfb91b9b172cb8c1caba68e3f510811fcffa77`
- reviewer gate: p7 recorded `FINAL GO-CANDIDATE / FOCUSED-GATES-PASS /
  PHPT-PASS` with `FINAL_FAIL=0` on public `8dafddc4`; artifacts:
  `/home/claude/supervised-php-compiler/state/workers/batch004-review-call-argument-unpack-builtin-callback-successor2-current-8dafddc4-20260528.{status.md,report.md,gates.log}`
- critic gate: phpc-33 recorded `SAFE-FOR-INTEGRATION /
  P7-FINAL-GO-VERIFIED / READ-ONLY`; artifacts:
  `/home/claude/supervised-php-compiler/state/workers/batch004-critic-call-argument-unpack-builtin-callback-successor2-current-8dafddc4-20260528.{status.md,report.md}`
- integration gate: phpc-28 recorded `FINAL-HANDOFF /
  READY-FOR-SUPERVISOR-APPLY`; artifacts:
  `/home/claude/supervised-php-compiler/state/workers/ao-integration-p31-builtin-callback-successor2-8dafddc4-20260528.{status.md,report.md,gates.log}`
- supervisor focused gates: PASS for `git diff --check`,
  docs/PROGRESS/examples exclusion, production exact-shape audit,
  `cargo fmt --all -- --check`, focused Rust
  `call_user_func_builtin_callback_expands_integer_unpacked_arguments`,
  `cargo build -p phpc`, and focused wrapper PHPT
  `/home/claude/supervised-php-compiler/state/workers/ao-coder-call-argument-unpack-builtin-callback-current-3c86fc6a-20260528.phpt`
- supervisor primary gate log:
  `/home/claude/supervised-php-compiler/state/workers/supervisor-primary-p31-builtin-callback-successor2-checkpoint9-20260528.gates.log`
- full PHPT suite: not run for this single source checkpoint; due after
  Batch004 reaches checkpoint10 or an explicit sharded/regression-repair
  publication gate

This checkpoint generalizes builtin callback argument evaluation. Builtin
callbacks invoked through `call_user_func()` now reuse the same builtin
value-call argument evaluator as direct builtin calls, so integer-keyed
unpacked arguments expand consistently. Production code is not keyed to a PHPT
filename, expected-output fixture, batch marker, or test-name branch.

Previous Batch004 source checkpoint 8 was the fatal-output regression repair:

Batch004 source checkpoint 8 is primary-integrated under AO supervision as a
regression repair for the checkpoint7 sharded gate. The checkpoint8
regression-repair sharded gate is now published with **1369 / 20294 pinned
runnable PHPTs = 6.75%** and zero regressions from the Batch003 PASS set.

- primary source head:
  `b75047df fix: preserve fatal output separator after inline output`
- source candidate patch:
  `/home/claude/supervised-php-compiler/state/patches/ao-coder-lsb-bug47699-regression-fix-20260528.patch`
- source candidate patch SHA256:
  `d6f2feca7306349c7d057122c67a36bbce342b6141a460520de015b21208ecbe`
- reviewer gate: p7 recorded `REVIEW-SCOPE-CORRECTED / FINAL
  GO-CANDIDATE / FOCUSED-GATES-PASS / PHPT-BUG47699-PASS` with
  `FINAL_FAIL=0`; artifacts:
  `/home/claude/supervised-php-compiler/state/workers/batch004-review-lsb-bug47699-regression-fix-2fe12551-20260528.{status.md,report.md,gates.log}`
- critic gate: supervisor audit recorded `SAFE-FOR-SUPERVISOR-APPLY`
  after phpc-33 followed a stale contradictory route; artifact:
  `/home/claude/supervised-php-compiler/state/workers/supervisor-critic-lsb-bug47699-regression-fix-2fe12551-20260528.status.md`
- focused gates: PASS for `git diff --check`, docs/PROGRESS exclusion,
  production exact-shape audit, `cargo fmt --all -- --check`, focused Rust
  `autoload_builtins::lsb_autoload_class_not_found_fatal_uses_single_separator_after_inline_output`,
  `cargo build -p phpc`, wrapper PHPT `Zend/tests/lsb/bug47699.phpt`, and
  wrapper PHPT `Zend/tests/type_declarations/typed_properties_055.phpt`
- supervisor primary gate log:
  `/home/claude/supervised-php-compiler/state/workers/supervisor-primary-lsb-bug47699-regression-fix-20260528.gates.log`
- full PHPT suite: supervisor-approved regression-repair sharded gate completed
  as
  `phpt-full-batch004-regression-repair-sharded-20260528T192018Z-php-src-f97ff59-public-3c86fc6a-source-b75047df-stack8`;
  counts were 1369 passed, 17815 failed, 2109 skipped, 16 xfailed, 1022 borked;
  Batch003 PASS regressions were 0

This checkpoint fixes a general fatal-output separator rule. Fatal output after
inline output without a trailing newline now receives exactly one separator
newline, while fatal output after newline-terminated multi-line output keeps the
blank-line separator needed by typed-property fatal diagnostics. Production
code is not keyed to a PHPT filename, batch marker, run-tests state, class name,
or expected-output fixture.

Previous Batch004 source checkpoint 7 was PHP_EOL plus RuntimeException
multicatch:

Batch004 source checkpoint 7 is primary-integrated under AO supervision.
This is a source checkpoint with focused proof, not a percentage change. The
public PHPT score remains **1311 / 20294 pinned runnable PHPTs = 6.46%** until
the next pinned full-suite run after 10 accepted Batch004 source checkpoints
or an explicit sharded publication gate.

- primary source head:
  `5240b156 fix: add PHP_EOL and runtime multicatch metadata`
- exported integration patch:
  `/home/claude/supervised-php-compiler/state/patches/ao-integration-php-eol-plus-bug74444-stack-24026a54-20260528.patch`
- exported integration patch SHA256:
  `a9c93ea6f704bcc978f93b628a6b2fcafbfc0aea4498325fd969ea152c693274`
- source candidate patch SHA256s:
  `307dd349d809af80dd2f405594e06b0cd511cecb2dd063362e82d26c5174c50a`
  (`PHP_EOL` constant support) and
  `0e7d39cb70ec6a8ae685b58c99a1de2c2cedc038aa72832cd135f81b5afcb837`
  (bug74444 RuntimeException multicatch successor3)
- reviewer gate: p7 recorded `FINAL GO-CANDIDATE /
  STACK SOURCE REVIEW ACCEPT / FOCUSED-GATES-PASS / PHPT-PASS` with
  `FINAL_FAIL=0` on `24026a54`; artifacts:
  `/home/claude/supervised-php-compiler/state/workers/batch004-review-php-eol-plus-bug74444-stack-24026a54-20260528.{status.md,report.md,gates.log}`
- critic gate: phpc-33 recorded `SAFE-FOR-INTEGRATION` after phpc-26 hit a
  context-limit exit; artifacts:
  `/home/claude/supervised-php-compiler/state/workers/batch004-critic-php-eol-plus-bug74444-stack-24026a54-20260528.{status.md,report.md}`
- integration gate: p28 recorded `FINAL-HANDOFF / FOCUSED-GATES-PASS /
  READY-FOR-SUPERVISOR-APPLY`, then the supervisor applied the exact reviewed
  two-patch stack; artifacts:
  `/home/claude/supervised-php-compiler/state/workers/ao-integration-php-eol-plus-bug74444-stack-24026a54-20260528.{status.md,report.md}`
- focused gates: PASS for `git diff --check`, docs/PROGRESS exclusion,
  production exact-shape audit, `cargo fmt --all -- --check`, focused Rust
  constant and exception/class-table tests, `cargo build -p phpc`, and wrapper
  PHPT cluster `Zend/tests/try/bug74444.phpt` plus
  `Zend/tests/try/try_multicatch_001.phpt` through
  `Zend/tests/try/try_multicatch_007.phpt`
- supervisor primary gate log:
  `/home/claude/supervised-php-compiler/state/workers/supervisor-primary-php-eol-plus-bug74444-stack-checkpoint7-20260528.gates.log`
- full PHPT suite: not run for this single source checkpoint; due after
  Batch004 reaches 10 accepted source checkpoints or an explicit sharded
  publication gate

This checkpoint adds `PHP_EOL` to the shared builtin constant tables and adds
runtime `RuntimeException` class metadata/catch compatibility needed for broad
multicatch behavior. It is not keyed to a PHPT filename or exact output shape:
the `PHP_EOL` path is a general builtin constant path, and the multicatch work
is covered by runtime class-table and exception-focused Rust gates.

Previous Batch004 source checkpoint 6 was dynamic static-method diagnostics:

Batch004 source checkpoint 6 is now primary-integrated under AO supervision.
This is a source checkpoint with focused proof, not a percentage change. The
public PHPT score remains **1311 / 20294 pinned runnable PHPTs = 6.46%** until
the next pinned full-suite run after 10 accepted Batch004 source checkpoints.

- primary source head:
  `0e7f0cd6 fix: validate dynamic static method calls`
- exported integration patch:
  `/home/claude/supervised-php-compiler/state/patches/ao-integration-p19-dynamic-static-method-diagnostics-c12fb41a-20260528.patch`
- exported integration patch SHA256:
  `7387aa6c3d05d080f93388f86959bab27d363d216400b9a1f19abc64fae5c8d9`
- source candidate patch SHA256:
  `7387aa6c3d05d080f93388f86959bab27d363d216400b9a1f19abc64fae5c8d9`
- reviewer gate: p7 recorded `FINAL GO-CANDIDATE /
  FOCUSED-GATES-PASS / PHPT-PASS` with `FINAL_FAIL=0` on `c12fb41a`
- critic gate: phpc-26 evidence-shape hold was resolved by p7 normalization;
  supervisor finalized `SAFE-FOR-INTEGRATION` for the exact normalized
  c12fb41a artifact after p26 lagged
- integration gate: supervisor applied the exact p7/p26-approved patch after
  p28 remained blocked on stale contradictory status
- focused gates: PASS for `git diff --check`, docs/PROGRESS exclusion,
  production exact-shape audit, `cargo fmt --all -- --check`, `cargo test -p
  phpc --test dynamic_features
  dynamic_static_method_call_names_validate_receiver_then_method_name --
  --exact --test-threads=1`, `cargo build -p phpc`, and wrapper PHPT cluster
  `Zend/tests/dynamic_call/dynamic_call_002.phpt`,
  `Zend/tests/dynamic_call/dynamic_call_003.phpt`,
  `Zend/tests/dynamic_call/dynamic_call_004.phpt`,
  `Zend/tests/dynamic_call/dynamic_call_non_static.phpt`,
  `Zend/tests/dynamic_call/dynamic_call_freeing.phpt`, and
  `Zend/tests/dynamic_call/bug46246.phpt`
- supervisor primary gate log:
  `/home/claude/supervised-php-compiler/state/workers/supervisor-primary-p19-dynamic-static-method-diagnostics-checkpoint6-20260528.gates.log`
- full PHPT suite: not run for this single source checkpoint; due after
  Batch004 reaches 10 accepted source checkpoints

This checkpoint generalizes dynamic static-method call handling for
static-property-shaped callees. It evaluates and validates receiver and method
name operands in PHP order, reports non-string method names and invalid
receivers through the runtime diagnostic path, and routes valid calls through
the existing named, `self`, `parent`, late-static, object, and class-string
static method helpers. It is not keyed to PHPT filenames, fixture names,
expected output rows, or one exact source shape.

Previous Batch004 source checkpoint 5 was builtin argument unpacking:

Batch004 source checkpoint 5 is primary-integrated under AO supervision.
This is a source checkpoint with focused proof, not a percentage change. The
public PHPT score remains **1311 / 20294 pinned runnable PHPTs = 6.46%** until
the next pinned full-suite run after 10 accepted Batch004 source checkpoints.

- primary source head:
  `1ae1c80d fix: expand builtin call argument unpacking`
- p28 final handoff head:
  `42e6b845f78b656d6ce876c45b1b47a45542ae46`
- exported integration patch:
  `/home/claude/supervised-php-compiler/state/patches/ao-integration-p31-builtin-spread-successor2-ebcccce6-20260528.patch`
- exported integration patch SHA256:
  `48a7f9f39154ed6f492c690787a49e4b66b868094bf4eec5a53141b96110760b`
- source candidate patch SHA256:
  `46b8ba5bd21a13769c7f4719ee3ce6de0d24eeb14b32eb78851ae13a3d3b6de7`
- reviewer gate: p7 recorded `FINAL GO-CANDIDATE /
  FOCUSED-GATES-PASS / PHPT-PASS` with `FINAL_FAIL=0` on `ebcccce6`
- critic gate: phpc-26 recorded `SAFE-FOR-INTEGRATION` for the exact
  successor2 patch on `ebcccce6`
- integration gate: p28 recorded `FINAL-HANDOFF / FOCUSED-GATES-PASS /
  READY-FOR-SUPERVISOR-APPLY`
- focused gates: PASS for `git diff --check`, docs/PROGRESS exclusion,
  production exact-shape audit, `cargo fmt --all -- --check`, `cargo test -p
  phpc --test array_map
  array_map_null_callback_accepts_unpacked_array_argument_list -- --exact
  --test-threads=1`, `cargo build -p phpc`, and wrapper PHPT
  `Zend/tests/arg_unpack/internal.phpt`
- supervisor primary gate log:
  `/home/claude/supervised-php-compiler/state/workers/supervisor-primary-p31-builtin-spread-checkpoint5-20260528.gates.log`
- full PHPT suite: not run for this single source checkpoint; due after
  Batch004 reaches 10 accepted source checkpoints

This checkpoint generalizes builtin call argument handling so builtin fallback
calls evaluate arguments in source order, expand integer-keyed array unpacking,
and reject unsupported named, string-keyed, or post-spread builtin arguments
with regular runtime errors. It is not keyed to PHPT filenames, fixture names,
expected output rows, or one exact source shape.

Previous Batch004 source checkpoint 4 was dynamic callable visibility:

Batch004 source checkpoint 4 is now primary-integrated under AO supervision.
This is a source checkpoint with focused proof, not a percentage change. The
public PHPT score remains **1311 / 20294 pinned runnable PHPTs = 6.46%** until
the next pinned full-suite run after 10 accepted Batch004 source checkpoints.

- primary source head:
  `84bb8c5b fix: honor scoped array callable visibility`
- p28 final handoff head:
  `e3dd25571e6da646e7e72569396e3198c2338b66`
- exported integration patch:
  `/home/claude/supervised-php-compiler/state/patches/ao-integration-p19-bug46246-array-callable-visibility-successor3-32acd4d3-20260528.patch`
- exported integration patch SHA256:
  `78e02efef625846949864234d953f435330fe01784a101f5414af6ba1dbdb616`
- source candidate patch SHA256:
  `55adf7387aa05d82ec100c436a5d9ace2ead7b683a9b378eb2bfebbcdb3205b1`
- reviewer gate: p7 recorded `FINAL GO-CANDIDATE /
  FOCUSED-GATES-PASS / PHPT-PASS` with `FINAL_FAIL=0` on `32acd4d3`
- critic gate: phpc-26 recorded `SAFE-FOR-INTEGRATION` for the exact
  successor3 patch on `32acd4d3`
- integration gate: p28 recorded `FINAL-HANDOFF / FOCUSED-GATES-PASS /
  READY-FOR-SUPERVISOR-APPLY`
- focused gates: PASS for `git diff --check`, docs/PROGRESS exclusion,
  production exact-shape audit, `cargo fmt --all -- --check`, `cargo test -p
  phpc --test dynamic_features array_object_callables_use_calling_scope_visibility
  -- --exact --test-threads=1`, `cargo build -p phpc`, and wrapper PHPT
  `Zend/tests/dynamic_call/bug46246.phpt`
- supervisor primary gate log:
  `/home/claude/supervised-php-compiler/state/workers/supervisor-primary-p19-bug46246-checkpoint4-20260528.gates.log`
- full PHPT suite: not run for this single source checkpoint; due after
  Batch004 reaches 10 accepted source checkpoints

This checkpoint generalizes dynamic object-call and object array-callable
method resolution so current-scope private methods are honored before receiver
hierarchy fallback. Direct `$this->$method()`, `call_user_func([$this,
$method])`, and `call_user_func_array([$this, $method], [])` now share the
scoped resolver for object callbacks. It is not keyed to PHPT filenames,
fixture names, expected output rows, or one exact source shape.

Previous Batch004 source checkpoint 3 was try/finally recursive_previous
uncaught-fatal handling:

Batch004 source checkpoint 3 is primary-integrated under AO supervision.
This is a source checkpoint with focused proof, not a percentage change. The
public PHPT score remains **1311 / 20294 pinned runnable PHPTs = 6.46%** until
the next pinned full-suite run after 10 accepted Batch004 source checkpoints.

- primary source head:
  `0f98ed84 fix: preserve uncaught throwing finally state`
- p28 final handoff head:
  `b24feac856f4d21d1d2e04aba0fd9ccf89c2c524`
- exported integration patch:
  `/home/claude/supervised-php-compiler/state/patches/ao-integration-p18-try-finally-8ff12ea3-20260528.patch`
- exported integration patch SHA256:
  `6b669b39d6b2188168dbfc73e28eb8c34ba3c13a13cf0df8e4259de1f48942f5`
- source candidate patch SHA256:
  `7946a88fc3eed3046c3c287de0a6b6876e32c45be5cd450e2bf4d8acbbf77280`
- prerequisite patch SHA256:
  `46f4519dcb9e3e9914aa419261c1a6ea6783d4acab28d764561c65b9419605d1`
- reviewer gate: p7 recorded `FINAL GO-CANDIDATE /
  FOCUSED-GATES-PASS / PHPT-PASS` with `FINAL_FAIL=0` on `8ff12ea3`
- critic gate: phpc-26 recorded `SAFE-FOR-INTEGRATION` for the exact
  prerequisite plus successor2 stack on `8ff12ea3`
- integration gate: p28 recorded `FINAL-HANDOFF / FOCUSED-GATES-PASS /
  READY-FOR-SUPERVISOR-APPLY`
- focused gates: PASS for `git diff --check`, docs/PROGRESS exclusion,
  production exact-shape audit, `cargo fmt --all -- --check`, `cargo test -p
  phpc --test object_model uncaught_exception_fatal_renders_message_and_throw_site
  -- --exact --test-threads=1`, `cargo test -p phpc --test object_model
  throwing_finally_overrides_pending_exception_for_uncaught_fatal -- --exact
  --test-threads=1`, `cargo build -p phpc`, and wrapper PHPT
  `Zend/tests/try/try_finally_recursive_previous.phpt`
- supervisor primary gate log:
  `/home/claude/supervised-php-compiler/state/workers/supervisor-primary-p18-try-finally-checkpoint3-20260528.gates.log`
- full PHPT suite: not run for this single source checkpoint; due after
  Batch004 reaches 10 accepted source checkpoints

This checkpoint generalizes bounded Exception/Throwable constructor state and
uncaught fatal emission for throwing `finally` paths. Core exception subclasses
now initialize protected `message`, `code`, and `previous` state, uncaught
throws use the shared execution shutdown/fatal path, and throwing `finally`
overrides pending exceptions without recursive `previous` output. It is not
keyed to PHPT filenames, fixture names, expected output rows, or one exact
source shape.

Previous Batch004 source checkpoint 2 was getenv runtime builtin support:

Batch004 source checkpoint 2 is primary-integrated under AO supervision.
This is a source checkpoint with focused proof, not a percentage change. The
public PHPT score remains **1311 / 20294 pinned runnable PHPTs = 6.46%** until
the next pinned full-suite run after 10 accepted Batch004 source checkpoints.

- primary source head:
  `ac3ea46c fix: add getenv runtime builtin`
- p28 final handoff head:
  `7e92e4ca588f093e6afd934224cc6677a1d0c6c6`
- exported integration patch:
  `/home/claude/supervised-php-compiler/state/patches/ao-integration-getenv-borked-skipif-source-candidate-20260528.patch`
- exported integration patch SHA256:
  `45031061695aabd5f6b0c8387b4c1a6c16acf2d6563bc61b4618e88d70ffcb44`
- source candidate patch SHA256:
  `8ef899fd5e6bd496c9734adb01b4b6c254f96063b0a01e3be918fe20c620d856`
- reviewer gate: p7 current-head gates recorded `FINAL GO-CANDIDATE /
  FOCUSED-GATES-PASS / PHPT-SKIPIF-NON-BORKED-PASS` with `FINAL_FAIL=0`
  on `c1b1a5ca`
- critic gate: `SAFE-FOR-INTEGRATION / GETENV-C1B1A5CA`; supervisor
  finalized the critic status from p7/p26 durable artifacts after both TUI
  sessions stalled before their final status rewrite
- focused gates: PASS for `git diff --check`, docs/PROGRESS exclusion,
  production exact-shape audit, `cargo fmt --all -- --check`, `cargo test -p
  phpc --test getenv_builtin -- --test-threads=1`, `cargo build -p phpc`, and
  wrapper PHPTs `tests/basic/GHSA-9pqp-7h25-4f32.phpt`,
  `tests/basic/gh16998.phpt`, and `ext/mbstring/tests/cp936_encoding.phpt`
  as SKIP/non-BORKED checks
- supervisor primary gate log:
  `/home/claude/supervised-php-compiler/state/workers/supervisor-primary-getenv-borked-checkpoint2-20260528.gates.log`
- full PHPT suite: not run for this single source checkpoint; due after
  Batch004 reaches 10 accepted source checkpoints

This checkpoint generalizes `getenv()` runtime/interpreter builtin support for
environment-based PHPT SKIPIF/runtime evaluation. It adds callable metadata and
interpreter behavior for direct and dynamic string calls, present/missing/empty
environment values, no-arg/null environment snapshots, and `local_only`
validation while preserving direct native `getenv()` lowering rejection. It is
not keyed to PHPT filenames, skipif source text, expected output rows, or one
exact source shape.

Previous Batch004 source checkpoint 1 was p15 typed-properties reference-held
TypeError:

Batch004 source checkpoint 1 is primary-integrated under AO supervision.
This is a source checkpoint with focused proof, not a percentage change. The
public PHPT score remains **1311 / 20294 pinned runnable PHPTs = 6.46%** until
the next pinned full-suite run after 10 accepted Batch004 source checkpoints.

- primary source head:
  `15ea036b fix: preserve typed property reference type errors`
- p28 final handoff head:
  `476912f52c7ad032e66b5edfc5808b3ed9ac3ca1`
- exported integration patch:
  `/home/claude/supervised-php-compiler/state/patches/ao-integration-p15-typed-properties-reference-held-typeerror-successor5-20260528.patch`
- exported integration patch SHA256:
  `d614d437986d41e527741d83181cfc31f0d8af424ee53dbc9665b8f6c650841a`
- source candidate patch SHA256:
  `93a2b65509ab3e1e145a52d6b706258cdbfc0899350cb8f069976196a9c96077`
- reviewer gate: p7 recorded `FINAL GO-CANDIDATE /
  FOCUSED-GATES-PASS` with `FINAL_FAIL=0`
- critic gate: phpc-26 recorded `SAFE-FOR-INTEGRATION` and audited
  `ace4177d..480e32ab` as `PROGRESS.md`-only/source-equivalent for this
  candidate
- focused gates: PASS for `git diff --check`, docs/PROGRESS exclusion,
  production exact-shape audit, `cargo fmt --all -- --check`, `cargo test -p
  phpc --test typed_properties_reference_held_type_errors -- --test-threads=1`,
  `cargo build -p phpc`, and wrapper PHPT
  `Zend/tests/type_declarations/typed_properties_055.phpt`
- full PHPT suite: not run for this single source checkpoint; due after
  Batch004 reaches 10 accepted source checkpoints

This checkpoint generalizes typed-property reference TypeError handling for
by-reference calls. The interpreter now preserves pending user/closure call
frames for uncaught catchable PHP errors, converts reference-held typed
property assignment failures into catchable `TypeError`, formats the uncaught
trace through the shared fatal path, and supports nested visible object
property reference arguments. It is not keyed to PHPT filenames, fixture class
names, property names, output rows, or one exact source shape.

Previous post-Batch002 source checkpoint 21 was p15 disallowed property types:

Post-Batch002 source checkpoint 21 is now primary-integrated under AO
supervision. This is a source checkpoint with focused proof, not a percentage
change. Batch003 has now completed and moved the public score to **1311 /
20294 pinned runnable PHPTs = 6.46%**, with 0 regressions from the Batch002
PASS set.

- primary source head:
  `202dd1ec fix: reject disallowed property types`
- p28 final handoff head:
  `fa71d4d44524b5e7e13f6b46e0f3fdd247d6ed50`
- exported integration patch:
  `/home/claude/supervised-php-compiler/state/patches/ao-integration-p15-typed-properties-disallowed-callable-type-successor3-20260528.patch`
- exported integration patch SHA256:
  `8f72a01d654d181282fd1de333f7c9f13ae85918f8b91da0e9bea41f2cdb2eec`
- source candidate patch SHA256:
  `3350b25c52d096c62441bd3b797c6c5578b050a1c05a89b3d08d80906a143fd8`
- reviewer gate: p7 recorded `FINAL GO-CANDIDATE /
  FOCUSED-GATES-PASS` with `FINAL_FAIL=0`
- critic gate: phpc-26 recorded `SAFE-FOR-INTEGRATION`
- focused gates: PASS for `git diff --check`, docs/PROGRESS exclusion,
  production exact-shape audit, `cargo fmt --all -- --check`, `cargo test -p
  phpc --test typed_properties_disallowed_property_types -- --test-threads=1`,
  `cargo build -p phpc`, and wrapper PHPTs
  `Zend/tests/type_declarations/typed_properties_053.phpt` and
  `Zend/tests/type_declarations/typed_properties_054.phpt`
- full PHPT suite: Batch003 complete; 1311 passed on the pinned public
  denominator, 0 regressions from the Batch002 PASS set

This checkpoint generalizes startup validation for disallowed property type
names. Class, trait, interface, and promoted-property declarations now reject
`callable` and nullable `?callable` property types through the existing fatal
startup diagnostic path. It is not keyed to PHPT filenames, fixture class names,
property names, expected output rows, or one exact source shape.

Previous post-Batch002 source checkpoint 20 was p19 dynamic-call bug63173:

Post-Batch002 source checkpoint 20 is now primary-integrated under AO
supervision. This is a source checkpoint with focused proof, not a percentage
change. It was later included in Batch003, whose current public score is
recorded at the top of this file.

- primary source head:
  `32b8b637 fix: validate dynamic array callback shape`
- p28 final handoff head:
  `9c1c5f1ccc54fce7ffd67e467ff8177d2bbb484f`
- exported integration patch:
  `/home/claude/supervised-php-compiler/state/patches/ao-integration-p19-bug63173-successor2-source-20260528.patch`
- exported integration patch SHA256:
  `fe7cf9e2657475ea7f457f14fff2bb94ace298d3cef3f92c3ea7010eb038427c`
- source candidate patch SHA256:
  `e47baaa9976949160728edf8d1e0650104926911c1cda3360175a4a73ed4d03b`
- reviewer gate: p7 recorded `FINAL GO-CANDIDATE /
  FOCUSED-GATES-PASS` with `FINAL_FAIL=0`
- critic gate: phpc-26 recorded `SAFE-FOR-INTEGRATION /
  P19-DYNAMIC-CALL-BUG63173-SUCCESSOR2`
- focused gates: PASS for `git diff --check`, docs/PROGRESS exclusion,
  production exact-shape audit, `cargo fmt --all -- --check`, `cargo test -p
  phpc --test dynamic_features
  invalid_array_callbacks_with_missing_zero_one_indices_emit_uncaught_error --
  --exact --test-threads=1`, `cargo build -p phpc`, and wrapper PHPTs
  `Zend/tests/dynamic_call/bug63173.phpt`,
  `Zend/tests/dynamic_call/dynamic_call_freeing.phpt`,
  `Zend/tests/dynamic_call/bug77877.phpt`, and
  `Zend/tests/dynamic_call/dynamic_fully_qualified_call.phpt`
- full PHPT suite: not run for this single source checkpoint

This checkpoint generalizes dynamic array callback shape validation and
uncaught PHP `Error` fatal emission. Array callbacks that do not contain
integer indices `0` and `1` now use PHP's array-callback diagnostic, while
escaped catchable runtime errors are emitted through the generic uncaught
PHP-error fatal path with exit code 255. It is not keyed to PHPT filenames,
fixture callback values, Rust test names, expected output rows, or one exact
source shape.

Previous post-Batch002 source checkpoint 19 was p15 assignment expression:

Post-Batch002 source checkpoint 19 is now primary-integrated under AO
supervision. This is a source checkpoint with focused proof, not a percentage
change. It was later included in Batch003, whose current public score is
recorded at the top of this file.

- primary source head:
  `dfd7ff09 fix: return coerced typed property assignment values`
- p28 final handoff head:
  `6cd6cf358a82e1bf4282f12797bb81e8d2364fe8`
- exported integration patch:
  `/home/claude/supervised-php-compiler/state/patches/ao-integration-p15-typed-properties-assignment-expression-result-20260528.patch`
- exported integration patch SHA256:
  `6c34485e6c6fadf93ff61e2c8866e3adcd47c0b4db6315dfebc5c8a60591cf02`
- source candidate patch SHA256:
  `38f0ef54da9f0df773b6c4d4bdd3508bec98ca9db7db1d756031c90c8d2f77db`
- reviewer gate: p7 recorded `FINAL GO-CANDIDATE /
  FOCUSED-GATES-PASS` with `FINAL_FAIL=0`
- critic gate: phpc-26 recorded `SAFE-FOR-INTEGRATION /
  P15-ASSIGNMENT-EXPRESSION`
- focused gates: PASS for `git diff --check`, docs/PROGRESS exclusion,
  production exact-shape audit, `cargo fmt --all -- --check`, `cargo test -p
  phpc --test assignment_expression
  typed_object_property_assignment_expression_returns_stored_coerced_value --
  --exact --test-threads=1`, `cargo test -p phpc --test
  assignment_expression
  dynamic_typed_object_property_assignment_expression_returns_stored_coerced_value
  -- --exact --test-threads=1`, `cargo build -p phpc`, and wrapper PHPT
  `Zend/tests/type_declarations/typed_properties_077.phpt`
- full PHPT suite: not run for this single source checkpoint

This checkpoint generalizes typed object property assignment-expression
results. Static and dynamic typed-property writes now return the stored/coerced
value from the runtime typed-property storage boundary, while fallback dynamic
public-property writes preserve the original assignment value. It is not keyed
to PHPT filenames, Rust test names, fixture class names, expected output rows,
or one exact source shape.

Previous post-Batch002 source checkpoint 18 was p31 argument unpack successor:

Post-Batch002 source checkpoint 18 is now primary-integrated under AO
supervision. This is a source checkpoint with focused proof, not a percentage
change. It was later included in Batch003, whose current public score is
recorded at the top of this file.

- primary source head:
  `97e80bcc fix: support string-keyed argument unpacking`
- p28 final handoff head:
  `2ca746d91fba23d9c782efc774e5176802fdd546`
- exported integration patch:
  `/home/claude/supervised-php-compiler/state/patches/ao-integration-p31-call-argument-unpack-successor3-20260528.patch`
- exported integration patch SHA256:
  `3cfacd229b26665d76685fb7212a0c23ce11e689f78965105279172e267debdd`
- source candidate patch SHA256:
  `14e92e0e615934164fa76b2d3919c705c1388c7c480254cf675dfdbbb5675fd7`
- reviewer gate: p7 recorded `FINAL GO-CANDIDATE /
  FOCUSED-GATES-PASS / PHPT-NON-REGRESSION` with `FINAL_FAIL=0`
- critic gate: phpc-26 recorded `SAFE-FOR-INTEGRATION /
  P31-CALL-ARGUMENT-UNPACK-SUCCESSOR3`
- focused gates: PASS for `git diff --check`, docs/PROGRESS exclusion,
  production exact-shape audit, `cargo fmt --all -- --check`, `cargo test -p
  phpc --test functions_and_scopes
  string_keyed_argument_unpacking_binds_declared_parameter_names -- --exact
  --test-threads=1`, `cargo test -p phpc --test functions_and_scopes
  named_arguments_after_unpack_feed_variadic_rest_with_string_keys -- --exact
  --test-threads=1`, `cargo test -p phpc --test dynamic_features
  closure_argument_unpacking_accepts_string_keys_as_named_arguments -- --exact
  --test-threads=1`, `cargo build -p phpc`, and wrapper PHPT
  `Zend/tests/named_params/unpack_and_named_1.phpt`
- full PHPT suite: not run for this single source checkpoint

This checkpoint generalizes userland argument binding for string-keyed unpacked
arguments. The interpreter now carries unpacked argument keys through call-frame
binding, maps string keys to declared parameters, preserves unmatched string
keys in variadic rest arrays, and reports duplicate named-parameter conflicts
through the same runtime path. It is not keyed to PHPT filenames, fixture
function names, expected output rows, public commit hashes, or one exact source
shape.

Previous post-Batch002 source checkpoint 17 was p17 attributes next-source:

Post-Batch002 source checkpoint 17 is now primary-integrated under AO
supervision. This is a source checkpoint with focused proof, not a percentage
change. It was later included in Batch003, whose current public score is
recorded at the top of this file.

- primary source head:
  `f508515b fix: allow abstract trait override targets`
- p28 final handoff head:
  `72dda13203750f569189f4a9f84bbc4b6d48cbc7`
- exported integration patch:
  `/home/claude/supervised-php-compiler/state/patches/ao-integration-p17-attributes-next-source-20260528.patch`
- exported integration patch SHA256:
  `ae96e39c4792bf581f3b72df8565903e0e9998eb745640e7294f8bf3d754a80b`
- source candidate patch SHA256:
  `6da60add13f1c5be68041f5990ed6e35835c6d97bba8d4c4e9184e6e8910cea4`
- reviewer gate: p7 recorded `FINAL GO-CANDIDATE /
  FOCUSED-GATES-PASS` with `FINAL_FAIL=0`
- critic gate: phpc-26 recorded `SAFE-FOR-INTEGRATION /
  P17-ATTRIBUTES-NEXT-SOURCE`
- focused gates: PASS for `git diff --check`, docs/PROGRESS exclusion,
  production exact-shape audit, `cargo fmt --all -- --check`, `cargo test -p
  phpc --test object_model
  abstract_trait_methods_can_satisfy_method_override_attributes -- --exact
  --test-threads=1`, `cargo build -p phpc`, and wrapper PHPTs
  `Zend/tests/attributes/override/016.phpt`,
  `Zend/tests/attributes/override/gh12189_6.phpt`, and
  `Zend/tests/attributes/override/021.phpt`
- full PHPT suite: not run for this single source checkpoint

This checkpoint generalizes abstract trait method support for method
`#[\Override]` validation. Trait parser support now accepts public abstract
method signatures with semicolon bodies, abstract trait requirements are kept
out of executable method runtime tables, and startup override validation can
match methods against directly used and nested composed abstract trait
requirements. It is not keyed to PHPT filenames, fixture trait/class names,
method names, source hashes, or exact output rows.

Previous post-Batch002 source checkpoint 16 was p19 dynamic call:

Post-Batch002 source checkpoint 16 is now primary-integrated under AO
supervision. This is a source checkpoint with focused proof, not a percentage
change. It was later included in Batch003, whose current public score is
recorded at the top of this file.

- primary source head:
  `8f4e7996 fix: support static object array callbacks`
- p28 final handoff head:
  `9cd7a209ff7f46bbc219438a42d7db13ecf8622a`
- exported integration patch:
  `/home/claude/supervised-php-compiler/state/patches/ao-integration-p19-bug77877-successor3-source-20260528.patch`
- exported integration patch SHA256:
  `14ae3ba2706f7a13964952437258c1a535f7fa3dbbc62dcbe07073a81811ca67`
- source candidate patch SHA256:
  `597847ef8f27e7f77a91c36cd784935c516eca2cee7b89c422035316b814af98`
- reviewer gate: p7 recorded `FINAL GO-CANDIDATE /
  FOCUSED-GATES-PASS` with `FINAL_FAIL=0`
- critic gate: phpc-26 recorded `SAFE-FOR-INTEGRATION /
  P19-DYNAMIC-CALL-BUG77877-SUCCESSOR3`
- focused gates: PASS for `git diff --check`, production exact-shape audit,
  `cargo fmt --all -- --check`, `cargo test -p phpc --test dynamic_features
  static_object_array_callbacks_do_not_bind_this -- --exact --test-threads=1`,
  `cargo build -p phpc`, and wrapper PHPTs
  `Zend/tests/dynamic_call/bug77877.phpt`,
  `Zend/tests/dynamic_call/bug68475.phpt`,
  `Zend/tests/dynamic_call/dynamic_call_non_static.phpt`, and
  `Zend/tests/dynamic_call/dynamic_call_freeing.phpt`
- full PHPT suite: not run for this single source checkpoint

This checkpoint generalizes dynamic object-array callback dispatch and
`array_map()` userland callback arity handling. Object-array callables that
resolve to public static methods invoke without binding `$this`, ordinary
`call_user_func()` and `call_user_func_array()` keep strict userland arity, and
`array_map()` enforces required parameters while allowing extra mapped values.
It is not keyed to PHPT filenames, fixture class names, method names, callback
values, or exact output rows.

Previous post-Batch002 source checkpoint 15 was p17 method override:

Post-Batch002 source checkpoint 15 is now primary-integrated under AO
supervision. This is a source checkpoint with focused proof, not a percentage
change. It was later included in Batch003, whose current public score is
recorded at the top of this file.

- primary source head:
  `8d4f1971 fix: validate method override attributes`
- p28 final handoff head:
  `549fd3d8fc62efd87d8f85af89d71cdfb909a482 fix: validate attributes override 021`
- exported integration patch:
  `/home/claude/supervised-php-compiler/state/patches/ao-integration-p17-override-021-source-20260528.patch`
- exported integration patch SHA256:
  `775f6afd15942a28e8d2ac56fbe2c6b269f1ba5296bd714f2ced6d811e938805`
- source candidate patch SHA256:
  `3cac2c0bd27415529f4e0a294b3a5590a444cb24357cfe4bb9319ec69142ebd6`
- reviewer gate: p7 recorded `FINAL GO-CANDIDATE /
  FOCUSED-GATES-PASS` with `FINAL_FAIL=0`
- critic gate: phpc-26 recorded `SAFE-FOR-INTEGRATION /
  P17-ATTRIBUTES-OVERRIDE-021`
- focused gates: PASS for `git diff --check`, `cargo fmt --all --
  --check`, `cargo test -p phpc --test object_model
  method_override_attribute_validates_class_interface_trait_and_constructor_methods
  -- --exact --test-threads=1`, `cargo build -p phpc`, and wrapper PHPTs
  `Zend/tests/attributes/override/001.phpt`,
  `Zend/tests/attributes/override/003.phpt`,
  `Zend/tests/attributes/override/006.phpt`,
  `Zend/tests/attributes/override/008.phpt`,
  `Zend/tests/attributes/override/017.phpt`,
  `Zend/tests/attributes/override/018.phpt`,
  `Zend/tests/attributes/override/021.phpt`,
  `Zend/tests/attributes/override/022.phpt`,
  `Zend/tests/attributes/override/023.phpt`, and
  `Zend/tests/attributes/override/024.phpt`
- full PHPT suite: not run for this single source checkpoint

This checkpoint generalizes method-level `#[\Override]` metadata and startup
validation across parent class methods, implemented interface methods, parent
interface methods, trait-provided methods, and constructor-specific method
relationships. It preserves parser/AST method attributes and emits startup
fatal diagnostics through the existing attribute validation path. It is not
keyed to PHPT filenames, fixture class names, method names, constructor names,
or exact output rows.

Previous post-Batch002 source checkpoint 14 was p31 call-argument unpack:

Post-Batch002 source checkpoint 14 is primary-integrated under AO
supervision. This is a source checkpoint with focused proof, not a percentage
change. It was later included in Batch003, whose current public score is
recorded at the top of this file.

- primary source head:
  `30e8e734 fix: expand argument unpacking for calls`
- source candidate patch:
  `/home/claude/supervised-php-compiler/state/patches/ao-coder-call-argument-unpack-source-candidate-20260528.patch`
- source candidate patch SHA256:
  `262c711cbe59ae51d5a89c49050538b0a156f8232f78818f507233f4f8dde530`
- reviewer gate: p7 recorded `FINAL GO-CANDIDATE /
  FOCUSED-GATES-PASS` with `FINAL_FAIL=0`
- critic gate: phpc-26 recorded `SAFE-FOR-INTEGRATION /
  P31-CALL-ARGUMENT-UNPACK`
- focused gates: PASS for `git diff --check`, `cargo fmt --all --
  --check`, `cargo test -p phpc --test dynamic_features
  dynamic_callable_array_and_closure_spread_arguments_are_expanded -- --exact
  --test-threads=1`, `cargo test -p phpc --test functions_and_scopes
  variadic_argument_unpacking_expands_array_values_for_user_functions --
  --exact --test-threads=1`, `cargo build -p phpc`, and wrapper PHPTs
  `Zend/tests/arg_unpack/dynamic.phpt`, `Zend/tests/arg_unpack/method.phpt`,
  and `Zend/tests/arg_unpack/new.phpt`
- full PHPT suite: not run for this single source checkpoint

This checkpoint generalizes positional array spread expansion for user
functions, method calls, dynamic callables, and `new` expression call frames. It
delays arity checks when spread arguments are present, expands numeric array
entries into positional call values, and keeps unsupported named/string-keyed
and by-reference spread shapes behind explicit diagnostics. It is not keyed to
PHPT filenames, fixture class names, method names, argument values, or exact
output rows.

Previous post-Batch002 source checkpoint 13 was p15 scalar constant-default
TypeError:

Post-Batch002 source checkpoint 13 is primary-integrated under AO
supervision. This is a source checkpoint with focused proof, not a percentage
change. It was later included in Batch003, whose current public score is
recorded at the top of this file.

- primary source head:
  `ed7fbc6b fix: throw type errors for constant defaults`
- source candidate patch:
  `/home/claude/supervised-php-compiler/state/patches/ao-coder-scalar-constant-defaults-error-typeerror-20260528.patch`
- source candidate patch SHA256:
  `aa2f1fb770ecb26ee9aaac10425397baabfdf9b32361577ef79abe0e749049b6`
- reviewer gate: p7 recorded `FINAL GO-CANDIDATE /
  NON-REGRESSION-ANCHOR / FOCUSED-GATES-PASS` with `FINAL_FAIL=0`
- critic gate: phpc-26 recorded `SAFE-FOR-INTEGRATION /
  P15-SCALAR-CONSTANT-DEFAULT-TYPEERROR / NON-REGRESSION-ANCHOR`
- focused gates: PASS for `git diff --check`, `cargo fmt --all --
  --check`, `cargo test -p phpc --test scalar_constant_default_type_errors --
  --test-threads=1`, `cargo test -p phpc --test default_parameter_constants
  -- --test-threads=1`, `cargo test -p phpc --test functions_and_scopes
  reference_parameter_literal_argument_reports_php_fatal_without_calling_body
  -- --exact --test-threads=1`, `cargo build -p phpc`, and wrapper PHPT
  `Zend/tests/type_declarations/scalar_constant_defaults_error.phpt`
- non-regression anchor: `scalar_constant_defaults.phpt` still fails on
  missing `PHP_EOL`, but p7 proved the same failure on clean public
  `4574c6fc` without this candidate; it is a separate pre-existing dependency,
  not a candidate-specific regression
- full PHPT suite: not run for this single source checkpoint

This checkpoint generalizes user-function call argument type mismatch handling
for default and constant-backed scalar parameters. It routes mismatches through
catchable `TypeError`, registers `TypeError` under the existing `Error`/Throwable
model, preserves catchability for existing dynamic-call `Error` paths, and emits
PHP-shaped uncaught call-argument diagnostics. It is not keyed to PHPT filenames,
fixture function names, constant names, expected output rows, or `PHP_EOL`.

Previous post-Batch002 source checkpoint 12 was p17 promoted-property override:

Post-Batch002 source checkpoint 12 is primary-integrated under AO. This is a
source checkpoint with focused proof, not a percentage change. It was later
included in Batch003, whose current public score is recorded at the top of this
file.

- primary source head:
  `0dafd34e fix: validate promoted property override attributes`
- p28 final handoff head:
  `286b80809b369c480ba6717a716eca19209da262 fix: validate promoted property override attributes`
- exported integration patch:
  `/home/claude/supervised-php-compiler/state/patches/ao-integration-p17-promoted-property-override-source-20260528.patch`
- exported integration patch SHA256:
  `a21ed82a834d6d16c2a2d523b324b6a49cd0590d4836bc8894df2a9596ece6a5`
- source candidate patch SHA256:
  `3f0d273aadd46fab7ff5e29bb755f97e2ab32257b766e2475a99e84640206f53`
- reviewer gate: p7 recorded `FINAL GO-CANDIDATE /
  FOCUSED-GATES-PASS` with `FINAL_FAIL=0`
- critic gate: phpc-26 recorded `SAFE-FOR-INTEGRATION` for the exact SHA on
  public head `4985692f`
- focused gates: PASS for `git diff --check`, `cargo fmt --all --
  --check`, `cargo test -p phpc --test object_model
  property_override_attribute_validates_promoted_constructor_properties --
  --exact --test-threads=1`, `cargo test -p phpc --test syntax_boundaries
  promoted_property -- --test-threads=1`, `cargo build -p phpc`, and wrapper
  PHPTs `Zend/tests/attributes/override/properties_19.phpt` and
  `Zend/tests/attributes/override/properties_20.phpt`
- full PHPT suite: not run for this single source checkpoint

This checkpoint generalizes constructor promoted-property metadata into the
existing property `#[\Override]` validation path. It captures promotion
visibility and pending attributes during parsing, materializes promoted
constructor parameters as property metadata, and keeps constructor promotion
initialization/native lowering conservative with generalized diagnostics. It is
not keyed to PHPT filenames, exact output, class names, property names, or
fixture shape.

Previous post-Batch002 source checkpoint 11 was p19 dynamic static-method
strings:

- primary source head:
  `229c38cf test: cover dynamic static method strings`
- primary implementation head:
  `fbeed70d fix: dispatch dynamic static method strings`
- p28 final handoff head:
  `f6b59ce0d504f116d1263b7d3cc83d7489643277 fix: dispatch dynamic static method strings`
- exported integration patch:
  `/home/claude/supervised-php-compiler/state/patches/ao-integration-p19-dynamic-call-static-method-strings-20260528.patch`
- exported integration patch SHA256:
  `4be43e1bbaf1cff03a497fad824ab06d9aba3fe82af9a824345f96c8bac650c9`
- source candidate patch SHA256:
  `54b416fcffd52bfd82834185fe0c10734c09844c72d565282652e32f19980613`
- reviewer gate: p7 recorded `FINAL GO-CANDIDATE /
  FOCUSED-GATES-PASS` with `FINAL_FAIL=0`
- critic gate: phpc-26 recorded `SAFE-FOR-INTEGRATION /
  P19-DYNAMIC-STATIC-METHOD-STRINGS-REPAIR2`
- focused gates: PASS for `git diff --check`, `cargo fmt --all --
  --check`, `cargo test -p phpc --test dynamic_features
  dynamic_static_method_string_calls_are_dispatched -- --exact
  --test-threads=1`, `cargo build -p phpc`, and wrapper PHPTs
  `Zend/tests/dynamic_call/bug68475.phpt`,
  `Zend/tests/dynamic_call/dynamic_call_non_static.phpt`, and
  `Zend/tests/dynamic_call/dynamic_call_freeing.phpt`
- full PHPT suite: not run for this single source checkpoint
- supervisor alignment: p28's final handoff export superseded the earlier
  source-only export by adding the focused Rust test hunk. Primary now includes
  that test coverage, and the supervisor reran `git diff --check`,
  `cargo fmt --all -- --check`, and exact Rust test
  `dynamic_static_method_string_calls_are_dispatched` with PASS.

This checkpoint generalizes dynamic string callables of the `Class::method`
shape through the existing static callable dispatch path, adds PHP-shaped
dynamic-call diagnostics for non-static static calls and malformed callbacks,
and adds compact `printf` builtin support through the existing bounded `sprintf`
formatting path. It is not keyed to PHPT filenames, fixture class names, method
names, or exact output strings.

Previous post-Batch002 source checkpoint 10 was p18 bug72216 array-offset
reference return:

- primary source head:
  `55b2bea2 fix: preserve try array-offset reference returns`
- p28 final handoff head:
  `5c35b2d1a8f35b021df7855745bd4c80553c1fd0 fix: preserve try array-offset reference returns`
- exported integration patch:
  `/home/claude/supervised-php-compiler/state/patches/ao-integration-p18-try-bug72216-array-offset-ref-source-20260528.patch`
- exported integration patch SHA256:
  `adab76c5a6c958398fadc2031589b78270ed2e6fc83b28ea9926cb9f515f13a7`
- source candidate patch SHA256:
  `4b2ec8e2eaefd47e5300d0b0ae63b8db7d49e145a9aaf02e1b47b8b293c55913`
- reviewer gate: p7 recorded `FINAL GO-CANDIDATE /
  FOCUSED-GATES-PASS` with `FINAL_FAIL=0`
- critic gate: phpc-26 recorded `SAFE-FOR-INTEGRATION /
  P18-TRY-BUG72216-ARRAY-OFFSET-REF-SOURCE`
- focused gates: PASS for `git diff --check`, `cargo fmt --all --
  --check`, `cargo build -p phpc`, and wrapper PHPT
  `Zend/tests/try/bug72216.phpt`
- full PHPT suite: not run for this single source checkpoint

This checkpoint generalizes array-offset reference-return preservation across
try/finally frame cleanup by falling back to the existing
`reference_return_alias_cell` path when a local static array-offset alias cell
is not present. It is not keyed to `bug72216`, exact output, variable names, or
fixture shape.

Previous post-Batch002 source checkpoint 9 was p15 scalar default validation:

- primary source head:
  `279f066b fix: validate scalar default parameter types`
- p28 final handoff head:
  `afc8eda4070ad2d17c1d0f38d7b036911284a216 fix: validate scalar default parameter types`
- exported integration patch:
  `/home/claude/supervised-php-compiler/state/patches/ao-integration-p15-scalar-invalid-default-type-validation-20260528.patch`
- exported integration patch SHA256:
  `e6cb3cdb31bf20569e7b7dfe157342e8355519f81d890906576b0d1fffdbe389`
- source candidate patch SHA256:
  `55665d095241fd77c3962897ad465d25e038f4ba85a6077ebd2d931ecfe74046`
- reviewer gate: p7 recorded `FINAL GO-CANDIDATE /
  FOCUSED-GATES-PASS` with `FINAL_FAIL=0`
- critic gate: phpc-26 recorded `SAFE-FOR-INTEGRATION /
  P15-SCALAR-INVALID-DEFAULT-TYPE-VALIDATION`
- focused gates: PASS for `git diff --check`, `cargo fmt --all --
  --check`, `cargo test -p phpc --test scalar_default_type_validation --
  --test-threads=1`, `cargo build -p phpc`, and wrapper PHPTs
  `Zend/tests/type_declarations/scalar_float_with_invalid_default.phpt`,
  `Zend/tests/type_declarations/scalar_float_with_integer_default_weak.phpt`,
  and `Zend/tests/type_declarations/default_boolean_hint_values.phpt`
- full PHPT suite: not run for this single source checkpoint

This checkpoint generalizes startup/default-parameter validation for scalar
literal defaults across function and method declarations. It normalizes scalar
aliases, preserves PHP's weak-mode `int` default acceptance for `float`, and
routes invalid defaults through PHP/Zend fatal startup diagnostics. It is not
keyed to a PHPT filename, exact output, function name, argument name, or fixture
shape. `strict_types` remains split out because the compiler still rejects the
`declare(strict_types=1)` statement before this validator runs.

Previous post-Batch002 source checkpoint 8 was p17 property-override
validation:

- primary source head:
  `785f0147 fix: validate property override attributes`
- p28 final handoff head:
  `079da6b0a6ed14ce1f6bb2f75bf5e5d80f5753d2 fix: validate property override attributes`
- exported integration patch:
  `/home/claude/supervised-php-compiler/state/patches/ao-integration-p17-property-override-validation-20260528.patch`
- exported integration patch SHA256:
  `d6165ed78b4c6a5ba71fd3cbcf88e26bd5a591bab50ed1f9699b37200a3439b1`
- source candidate patch SHA256:
  `7778c68b1d76172446230ec6295c3e467dba109052c8a44967a69d9f2a54a324`
- reviewer gate: p7 recorded `FINAL GO-CANDIDATE /
  FOCUSED-GATES-PASS` with `FINAL_FAIL=0`
- critic gate: phpc-26 recorded `SAFE-FOR-INTEGRATION /
  P17-PROPERTY-OVERRIDE-SUCCESSOR-7778-CLEAN-P7`
- focused gates: PASS for `git diff --check`, `cargo fmt --all --
  --check`, `cargo test -p phpc --test object_model
  property_override_attribute -- --nocapture --test-threads=1`,
  `cargo test -p phpc --test syntax_boundaries
  interface_property_hook_declarations_parse_as_interface_metadata --
  --nocapture --test-threads=1`, `cargo build -p phpc`, and wrapper PHPTs
  `Zend/tests/attributes/override/properties_01.phpt`,
  `Zend/tests/attributes/override/properties_02.phpt`,
  `Zend/tests/attributes/override/properties_05.phpt`,
  `Zend/tests/attributes/override/properties_07.phpt`,
  `Zend/tests/attributes/override/properties_08.phpt`,
  `Zend/tests/attributes/override/properties_09.phpt`,
  `Zend/tests/attributes/override/properties_11.phpt`, and
  `Zend/tests/attributes/override/properties_18.phpt`
- full PHPT suite: not run for this single source checkpoint

This checkpoint generalizes property `#[\Override]` validation across parsed
class, interface, and trait property metadata and routes missing-match
diagnostics through PHP/Zend fatal startup output shape. It is not keyed to a
PHPT filename, exact output, class name, property name, or fixture shape.

Previous post-Batch002 source checkpoint 7 was p19 dynamic-call catchable
errors:

- primary source head:
  `7ba72ffd fix: make dynamic call undefined errors catchable`
- p28 final handoff head:
  `92a2be8aef15e62e609ac09431e9046ee75cb6bc fix: make dynamic call undefined errors catchable`
- exported integration patch:
  `/home/claude/supervised-php-compiler/state/patches/ao-integration-p19-dynamic-call-catchable-errors-20260528.patch`
- exported integration patch SHA256:
  `890a7cb898fc81986cc8b29e2743b688f9b10a6ab8101bda8db3051483114123`
- source candidate patch SHA256:
  `81c9ff108197f782ff699cddd36356b7da3137ab31ccb1039a0a2953c135766f`
- reviewer gate: p7 recorded a clean rerun `FINAL GO-CANDIDATE /
  FOCUSED-GATES-PASS` with `FINAL_FAIL=0` after the supervisor rejected an
  earlier malformed review artifact.
- critic gate: phpc-26 recorded `SAFE-FOR-INTEGRATION /
  P19-DYNAMIC-CALL-CATCHABLE-ERRORS-81C9-CLEAN-P7`
- focused gates: PASS for `git diff --check`, `cargo fmt --all --
  --check`, `cargo test -p phpc --test dynamic_features
  dynamic_undefined_function_calls_are_catchable_errors -- --exact
  --test-threads=1`, `cargo build -p phpc`, and wrapper PHPTs
  `Zend/tests/dynamic_call/dynamic_fully_qualified_call.phpt`,
  `Zend/tests/dynamic_call/dynamic_call_005.phpt`,
  `Zend/tests/dynamic_call/dynamic_call_006.phpt`,
  `Zend/tests/dynamic_call/dynamic_call_007.phpt`, and
  `Zend/tests/dynamic_call/dynamic_call_008.phpt`
- full PHPT suite: not run for this single source checkpoint

This checkpoint generalizes dynamic undefined-function diagnostics into
catchable PHP `Error` flow through the existing runtime diagnostic path. It is
not keyed to a PHPT filename, exact output, function name, namespace, or fixture
shape.

Previous post-Batch002 source checkpoint 6 was p15 typed-properties magic-set:

- primary source head: `219d2c41 fix: route unset typed properties through __set`
- p28 final handoff head:
  `cb2f342edc2f414e6f7094bbd03acb3a34832ef1 fix: route unset typed properties through __set`
- exported integration patch:
  `/home/claude/supervised-php-compiler/state/patches/ao-integration-p15-typed-properties-magic-set-source-20260528.patch`
- exported integration patch SHA256:
  `20fecfbf3d1e490a8571e5977581aa3225c31144aadcd2a889505bf81670cdc4`
- source candidate patch SHA256:
  `66b73b0bcb546f5188b5c0eaa2ccd7138c98299c3f6abe48fad1ada8d1f1ee3e`
- reviewer gate: p7 recorded `FINAL GO-CANDIDATE /
  FOCUSED-GATES-PASS` with `FINAL_FAIL=0`
- critic gate: phpc-26 recorded `SAFE-FOR-INTEGRATION /
  P15-TYPED-PROPERTIES-MAGIC-SET-SOURCE`
- focused gates: PASS for `git diff --check`, `cargo fmt --all --
  --check`, `cargo test -p phpc --test typed_properties_magic_set --
  --test-threads=1`, `cargo build -p phpc`, and wrapper PHPT
  `Zend/tests/type_declarations/typed_properties_magic_set.phpt`
- full PHPT suite: not run for this single source checkpoint

This checkpoint generalizes typed-property unset-state and magic property
semantics. It adds explicit declared-property unset tracking, routes assignment
to explicitly unset declared properties through `__set` when available, keeps
fresh uninitialized `isset()`/`empty()` behavior distinct, and turns
uninitialized typed-property read diagnostics into catchable PHP `Error` flow.
It is not keyed to a PHPT filename, exact output, class name, or property name.

Previous post-Batch002 source checkpoint 5 was p15 typed-properties
protected-inheritance diagnostic:

- primary source head: `a69386f5 fix: diagnose protected typed property inheritance`
- p28 final handoff head:
  `5158aa5d70545863cf7d033664aea5c4817434bd fix: diagnose protected typed property inheritance`
- exported integration patch:
  `/home/claude/supervised-php-compiler/state/patches/ao-integration-p15-typed-properties-protected-inheritance-diagnostic-20260528.patch`
- exported integration patch SHA256:
  `b94379a1f646e7bc282adb15e26f6eaa4b061cc7c67123295e777d1b337ef95b`
- source candidate patch SHA256:
  `2261bb12a2539602e4ba7702d4869fce5c150e2e262faa113896db435ace0206`
- reviewer gate: p7 recorded `FINAL GO-CANDIDATE /
  FOCUSED-GATES-PASS` with `FINAL_FAIL=0`
- critic gate: phpc-26 recorded `SAFE-FOR-INTEGRATION /
  P15-TYPED-PROPERTIES-PROTECTED-INHERITANCE-DIAGNOSTIC`
- focused gates: PASS for `git diff --check`, `cargo fmt --all --
  --check`, `cargo test -p phpc --test typed_properties_protected_inheritance
  -- --test-threads=1`, `cargo build -p phpc`, and wrapper PHPT
  `Zend/tests/type_declarations/typed_properties_protected_inheritance_mismatch.phpt`
- full PHPT suite: not run for this single source checkpoint

That checkpoint generalizes typed-property inheritance startup diagnostics by
walking declared class metadata and ancestor chains for inherited non-private
properties, checking staticness, declared type invariance, and visibility
compatibility. It is not keyed to a PHPT filename, exact output, class name, or
property name.

Previous post-Batch002 source checkpoint 4 was p15 typed-property reference
coercion:

- primary source head: `fba6bd86 fix: coerce typed property reference assignments`
- p28 final handoff head:
  `0b35a182bc237e7ef98ab12ccec3f792f6d15802 fix: coerce typed property reference assignments`
- exported integration patch:
  `/home/claude/supervised-php-compiler/state/patches/ao-integration-p15-typed-property-reference-coercion-20260528.patch`
- exported integration patch SHA256:
  `f6a7f82f4344a837e91c0bc4f66f64f6f6abf7dd09e30fe766a3a15989f940a9`
- source candidate patch SHA256:
  `7d7ceef7949a8d6fcb87a80ca2725e22455e0e1fe61d9f2ed452b9a28fb64275`
- reviewer gate: p7 recorded `FINAL GO-CANDIDATE /
  FOCUSED-GATES-PASS` with `FINAL_FAIL=0`
- critic gate: phpc-26 recorded `SAFE-FOR-INTEGRATION /
  P15-TYPED-PROPERTY-REFERENCE-COERCION-FINAL`
- focused gates: PASS for `git diff --check`, `cargo fmt --all -- --check`,
  `cargo test -p phpc --test typed_property_reference_coercion --
  --test-threads=1`, `cargo build -p phpc`, and wrapper PHPTs
  `Zend/tests/type_declarations/typed_properties_reference_coercion_leak.phpt`,
  `Zend/tests/type_declarations/typed_properties_011.phpt`, and
  `Zend/tests/type_declarations/typed_properties_023.phpt`
- full PHPT suite: not run for this single source checkpoint

That checkpoint generalizes typed-property reference assignment/coercion
through interpreter object-property reference semantics. It is not keyed to a
PHPT filename, exact output, or one fixture shape.

Previous post-Batch002 source checkpoint 3 was p17:

- primary source head: `0c23019b fix: parse interface property hooks`
- p28 final handoff head:
  `02238dfa1210ea6e91f2aaa6b33cf809ff1a452a fix: parse interface property hooks`
- exported integration patch:
  `/home/claude/supervised-php-compiler/state/patches/ao-integration-p17-property-hooks-parser-20260528.patch`
- exported integration patch SHA256:
  `4670dab66c27ef85676279cace2451124140e22b3068b887544dbdf6e2a3603f`
- source candidate patch SHA256:
  `cbebb748e738425310190de5e9cc68d36b3f2afe87cafa044fef4d9e451e2252`
- reviewer gate: p7 recorded `FINAL GO-CANDIDATE /
  PARSER-SCOPE-FOCUSED-GATES-PASS` with `FINAL_FAIL=0`
- critic gate: phpc-26 recorded `SAFE-FOR-INTEGRATION /
  P17-PROPERTY-HOOKS-PARSER-FINAL`
- focused gates: PASS for `git diff --check`, `cargo fmt --all -- --check`,
  `cargo test -p phpc --test syntax_boundaries property_hook -- --nocapture
  --test-threads=1`, `cargo build -p phpc`, and wrapper PHPT
  `Zend/tests/attributes/override/properties_01.phpt`
- full PHPT suite: not run for this single source checkpoint

That checkpoint adds parser and AST support for public interface property
hooks. It is not claiming `properties_02.phpt`: p7 recorded that downstream
override-validation failure as pre-existing against the reviewed public binary.

Previous post-Batch002 source checkpoint 2 was p19:

- primary source head: `57b32f28 fix: reject forbidden builtins in dynamic calls`
- p28 final handoff head:
  `c2796a1723ca7d199477018715914360db5f25e3 fix: reject forbidden builtins in dynamic calls`
- exported integration patch:
  `/home/claude/supervised-php-compiler/state/patches/ao-integration-p19-dynamic-call-forbidden-builtins-20260528.patch`
- exported integration patch SHA256:
  `8ba971193f06fb8430f994ba84cf5399aaf584bce3108b9366b6fc9aa127380e`
- source candidate patch SHA256:
  `0294c0e62b7858ed92999c4a9b35f2b15a4b215a0172a0c2789112c1555896e9`
- reviewer gate: p7 recorded `FINAL GO-CANDIDATE /
  FOCUSED-GATES-PASS` with `FINAL_FAIL=0`
- critic gate: phpc-26 recorded p19 `SAFE-FOR-INTEGRATION` for exact source
  patch SHA256
- focused gates: PASS for `git diff --check`, `cargo fmt --all -- --check`,
  Rust units
  `forbidden_scope_introspection_builtins_report_dynamic_call_error` and
  `forbidden_dynamic_builtins_are_rejected_through_callback_dispatchers`,
  `cargo build -p phpc`, and wrapper PHPTs
  `Zend/tests/dynamic_call/dynamic_call_005.phpt` through
  `Zend/tests/dynamic_call/dynamic_call_008.phpt`
- full PHPT suite: not run for this single source checkpoint

That checkpoint generalizes rejection of forbidden dynamic introspection
builtins through the shared dynamic callback dispatcher paths, including
`call_user_func` variants, before ordinary dynamic function lookup. It is not
keyed to dynamic-call PHPT filenames, expected output text, line numbers, or a
single source shape.

Previous post-Batch002 source checkpoint 1 was p18:

- primary source head: `0a18d1c5 fix: materialize undefined byref sources`
- p28 final handoff head:
  `f8a74c2299a90de7ccb50193aa3056b48b481f0c fix: materialize undefined by-reference sources`
- exported integration patch:
  `/home/claude/supervised-php-compiler/state/patches/ao-integration-p18-byref-undefined-20260528.patch`
- exported integration patch SHA256:
  `aaf34af8b9751b061a4da6f8deda2a8b659bae48f27049c4c624313949fb29d8`
- source candidate patch SHA256:
  `f35486eb1e43340cf1bf1841c4933ddeb1df55badafc841a221bddcc477d6387`
- reviewer gate: p7 recorded `FINAL GO-CANDIDATE /
  EXACT-SOURCE-GATE-PASS` with `FINAL_FAIL=0`
- critic gate: phpc-26 recorded `SAFE-FOR-INTEGRATION /
  P18-BYREF-UNDEFINED-SOURCE-FINAL` with `FINAL_FAIL=0`
- focused gates: PASS for `git diff --check`, `cargo fmt --all -- --check`,
  Rust unit
  `interpreter::tests::symbol_table_reference_aliases_materialize_undefined_source`,
  `cargo build -p phpc`, and wrapper PHPTs
  `Zend/tests/try/bug72215_2.phpt` and `Zend/tests/try/bug72215_3.phpt`
- full PHPT suite: not run for this single source checkpoint

That checkpoint generalizes by-reference static/global binding and local
reference-return direct-variable sources so missing direct-variable sources
materialize as shared null reference cells through the interpreter symbol-table
path. It is not keyed to `bug72215`, PHPT filenames, expected output text, line
numbers, or a single try/finally source shape.

The previous full-suite checkpoint remains Batch002 `STACK-CLEAN-11` at
`e72efe27 fix: reject reserved scalar special class names`, run id
`phpt-full-batch002-20260528T100640Z-php-src-f97ff59-public-bc0ed214-source-e72efe27-stack11`.

Next source integration resumes from queued generalized candidates after fresh
reviewer/critic gates and a clean p28 apply/gate cycle. Current watches include
p19 dynamic-call catchable-errors focused review, p17 property-override
validation repair after a `cargo fmt` NO-GO, p16 generator-yield blocker work,
and broader try/finally/Throwable source blockers. Public PASS-NO-PATCH probe
rows remain lower priority than source integration.

## Batch 001

Policy: stage 10 accepted generalized source PRs, run focused gates per PR, run
the full PHPT suite once after PR 10, repair regressions, then merge the whole
batch.

Current batch status: **10/10 accepted, not merged; full PHPT baseline recorded; regression/failure repair next**.
Independent reviewer `phpc-7` accepted r81 / PR #1 as Batch 001 PR 10
at 2026-05-28 02:59 CEST after accepted-stack apply, exact-shape audit,
focused Rust/compiler gates, and focused wrapper PHPT `Zend/tests/namespaces/ns_065.phpt`
passed (1/1).

Accepted for staging:

| # | Candidate | Main proof |
| ---: | --- | --- |
| 1 | Magic-method signature diagnostics | Current-head review, focused diagnostics gates |
| 2 | Symbol-table foreach owners | Current-head review, focused native-link gates |
| 3 | Exception/catch/finally propagation | Current-head review, focused exception gates |
| 4 | Generated-C return-reference sources | Current-head review, accepted-stack compatibility, focused reference-return gates |
| 5 | Closing-tag statement terminator | Focused `tests/basic/001.phpt`, invalid-syntax review, accepted-stack compatibility |
| 6 | Object lifecycle live roots | Caller-frame live-root review, accepted-stack compatibility, focused destructor gates |
| 7 | Grouped namespace class imports | Current-head review, accepted-stack compatibility, focused compiler gates, wrapper PHPT proof |
| 8 | By-reference foreach lingering slots | Accepted-stack review, focused `Zend/tests/foreach/foreach_reference.phpt`, slot-preserving array-copy gates |
| 9 | Magic method startup signature fatals | Accepted-stack review, focused `tests/classes/__call_002.phpt`, generalized magic contract gates |
| 10 | Multiple unbracketed namespace declarations | Independent accepted-stack review, focused `Zend/tests/namespaces/ns_065.phpt`, namespace parser/import gates |

Gate status and parked candidates:

| Item | State |
| --- | --- |
| Batch 001 full PHPT gate | Done in AO session `phpc-11`; first baseline recorded at 1118 / 20294 runnable PHPTs (5.51%) |
| Full-suite count guard | Done; `all-results.txt` used `PASSED/FAILED/SKIPPED/XFAILED`, the parser counted those statuses, and the verified row is in `state/php-core-suite-history.tsv` |
| PR #4 by-reference call expressions | Batch002 stack decision says PR #4 supersedes r82/PR #2; use PR #4 as the by-reference candidate because it covers `Zend/tests/bug39944.phpt` plus adjacent return/pass-by-reference PHPTs |
| PR #5 named by-reference arguments | GO-CANDIDATE after independent review on accepted stack10 + PR #4 + PR #5; focused Rust gates passed and wrapper PHPTs `Zend/tests/named_params/references.phpt`, `tests/lang/passByReference_007.phpt`, and `tests/lang/returnByReference.002.phpt` passed 3/3 |
| PR #6 foreach reference-backed `print_r()` | GO-CANDIDATE after independent review on accepted stack10; focused Rust/build gates passed and wrapper PHPT `tests/lang/foreach_with_references_001.phpt` plus foreach anchors passed after a generalized reference-backed array formatting fix |
| PR #7 magic `__call()` by-reference array args | GO-CANDIDATE after refreshed stack-safe independent review and p14 `SAFE-FOR-PROGRESS`; focused Rust/build/fixture gates passed and wrapper PHPTs `tests/classes/__call_003.phpt` plus `tests/classes/__call_001.phpt` passed 2/2; no full-suite run and no percent change |
| PR #7 follow-up: `__call_004` static-syntax fallback to current `__call()` | GO-CANDIDATE after independent review on accepted stack10 plus reviewed Batch002 through refreshed PR #7 and p14 `SAFE-FOR-PROGRESS`; focused Rust/build/fixture gates passed and wrapper PHPTs `tests/classes/__call_004.phpt`, `tests/classes/__call_003.phpt`, and `tests/classes/__call_001.phpt` passed 3/3; no full-suite run and no percent change |
| PR #8 `passByReference_002` real-stack refresh | GO-CANDIDATE after independent review on accepted stack10 plus reviewed Batch002 through refreshed PR #7 and p14 `SAFE-FOR-PROGRESS`; focused Rust/build gates passed and wrapper PHPTs `tests/lang/passByReference_002.phpt` plus `tests/lang/passByReference_004.phpt` passed 2/2; no full-suite run and no percent change |
| PR #13 `passByReference_012` / `array_shift()` by-reference builtin | GO-CANDIDATE after refreshed independent review on public base `49a44b0d` and p14 `SAFE-FOR-PROGRESS`; patch applies without a progress hunk, focused Rust `array_shift_builtin` tests passed 5/5, `cargo build -p phpc` passed, and wrapper PHPTs `tests/lang/passByReference_012.phpt`, `tests/lang/passByReference_008.phpt`, and `tests/lang/passByReference_009.phpt` passed 3/3; no full-suite run and no percent change |
| PR #9 `passByReference_004` real-stack PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent review and p14 `SAFE-FOR-PROGRESS`; the reviewed PR #8 stack already passes the target, the stale PR #9 patch is obsolete, and reviewer PHPT rerun `tests/lang/passByReference_004.phpt` plus `tests/lang/passByReference_002.phpt` passed 2/2; no source patch, no full-suite run, and no percent change |
| PR #16 `returnByReference.004` uppercase `Class` declaration keyword | GO-CANDIDATE after independent review and p14 `SAFE-FOR-PROGRESS`; focused `object_model` Rust gate, `cargo build -p phpc`, and wrapper PHPTs `tests/lang/returnByReference.004.phpt`, `tests/lang/returnByReference.002.phpt`, and `tests/lang/returnByReference.003.phpt` passed 3/3; no full-suite run and no percent change |
| PR #17 `returnByReference.006` dynamic-call return-by-reference fallback | GO-CANDIDATE after independent p7 review and p14 `SAFE-FOR-PROGRESS`; patch SHA `9cf5386634f214fb83e1517337ad4ea12f89662808f3c305143ebd3fcf1ec12e`; focused Rust gate, `cargo build -p phpc`, and wrapper PHPTs `tests/lang/returnByReference.006.phpt` plus `tests/lang/returnByReference.003.phpt` passed 2/2; no full-suite run and no percent change |
| PR #10 `passByReference_006` real-stack `var_dump()` reference visibility | GO-CANDIDATE after independent review and p14 `SAFE-FOR-PROGRESS`; patch SHA `3c5a5ef37747de2a52374eabe35f674e4732cbe588bd742a4bc7ae6e0ca4304b`; focused Rust gate, `cargo build -p phpc`, and wrapper PHPTs `tests/lang/passByReference_006.phpt`, `tests/lang/passByReference_004.phpt`, and `tests/lang/passByReference_002.phpt` passed 3/3; no full-suite run and no percent change |
| p19 `passByReference_005` repair2 missing-variable by-reference cells and non-referenceable argument fatal | GO-CANDIDATE after independent p7 review and p14 `SAFE-FOR-PROGRESS`; patch SHA `6c1fb034e7f598f214069728fc3c46bfd2e718742f7a5c6bcd8823f403a4a6ab`; `cargo fmt`, focused Rust `missing_variable_reads_warn_and_reference_arguments_materialize_null_cells`, `cargo build -p phpc`, and wrapper PHPTs `tests/lang/passByReference_005.phpt`, `_006.phpt`, `_004.phpt`, and `_002.phpt` passed 4/4; no full-suite run and no percent change |
| PR #19 `passByReference_003` undefined call-argument recovery | GO-CANDIDATE after independent review and p14 `SAFE-FOR-PROGRESS`; patch SHA `4f54ff6bd5517848b77f711e04373a1a571a6e8548dd7d2e31be9d7fab8a2ad6`; focused Rust gates, `cargo build -p phpc`, and wrapper PHPTs `tests/lang/passByReference_003.phpt`, `tests/lang/passByReference_001.phpt`, and `tests/lang/passByReference_007.phpt` passed 3/3; no full-suite run and no percent change |
| PR #18 `returnByReference.008` dynamic instance-method return-by-reference fallback | GO-CANDIDATE after independent review and p14 `SAFE-FOR-PROGRESS`; patch SHA `1c655efefb2e1aba956e912c4fe0a3c18f870497ff1b37f890cb53219f038d4f`; focused Rust gate, `cargo build -p phpc`, and wrapper PHPTs `tests/lang/returnByReference.008.phpt`, `tests/lang/returnByReference.004.phpt`, `tests/lang/returnByReference.003.phpt`, and `tests/lang/returnByReference.009.phpt` passed 4/4; no full-suite run and no percent change |
| p16 `call_static` magic static callable dispatch | GO-CANDIDATE after independent p7 review and p14 `SAFE-FOR-PROGRESS`; patch SHA `561b597f23a510902f02d9dd4cb25b23c46b15e279dea5d232f269b7b1639613`; focused Rust gate, `cargo build -p phpc`, and wrapper PHPT `Zend/tests/magic_methods/call_static.phpt` passed; nearby anchor `Zend/tests/magic_methods/call_static_002.phpt` failed in both candidate and reviewed-baseline runs and is recorded as pre-existing/non-regression; no full-suite run and no percent change |
| `Zend/tests/magic_methods/bug32429.phpt` `method_exists()` with `__call` PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing `method_exists()` behavior correctly returns false for an absent method even when `__call` exists, and wrapper PHPT `Zend/tests/magic_methods/bug32429.phpt` passed 1/1; no source patch, no cargo gate, no full-suite run, and no percent change |
| `Zend/tests/magic_methods/bug36006.phpt` destructor `$this` / parent-destructor cleanup PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing destructor cleanup behavior passes wrapper PHPT `Zend/tests/magic_methods/bug36006.phpt` 1/1; no source patch, no cargo gate, no full-suite run, and no percent change |
| `Zend/tests/magic_methods/bug37707.phpt` clone-new `__clone` PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing clone handling invokes `__clone` for unassigned `clone new` expressions and passes wrapper PHPT `Zend/tests/magic_methods/bug37707.phpt` 1/1 using the pass005 repair2 reviewed binary; no source patch, no cargo gate, no full-suite run, and no percent change |
| `Zend/tests/magic_methods/bug36759.phpt` shutdown destructor order PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing shutdown destructor order behavior passes wrapper PHPT `Zend/tests/magic_methods/bug36759.phpt` 1/1 using the pass005 repair2 reviewed binary; no source patch, no cargo gate, no full-suite run, and no percent change |
| `Zend/tests/magic_methods/bug38146.phpt` `__get` array-return foreach PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing runtime iterates over an array returned by `__get` in foreach/read context and passes wrapper PHPT `Zend/tests/magic_methods/bug38146.phpt` 1/1 using the pass005 repair2 reviewed binary; no source patch, no cargo gate, no full-suite run, and no percent change |
| `Zend/tests/magic_methods/bug44899_2.phpt` `__isset` / `empty()` / `__get` PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing magic-property behavior handles the `__isset`/`empty()`/`__get` interaction and passes wrapper PHPT `Zend/tests/magic_methods/bug44899_2.phpt` 1/1 using the pass005 repair2 reviewed binary; no source patch, no cargo gate, no full-suite run, and no percent change |
| `Zend/tests/magic_methods/bug47353.phpt` destructor object-allocation loop PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing destructor/object-allocation handling passes wrapper PHPT `Zend/tests/magic_methods/bug47353.phpt` 1/1 using the pass005 repair2 reviewed binary; no source patch, no cargo gate, no full-suite run, and no percent change |
| `Zend/tests/magic_methods/bug51822.phpt` static-property destructor ordering PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing static-property object lifetime/destructor ordering handles the target and passes wrapper PHPT `Zend/tests/magic_methods/bug51822.phpt` 1/1 using the pass005 repair2 reviewed binary; no source patch, no cargo gate, no full-suite run, and no percent change |
| `Zend/tests/magic_methods/bug54372.phpt` chained `__get` receiver method-call PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing chained magic property access preserves the object returned by `__get()` for the following method call, passing wrapper PHPT `Zend/tests/magic_methods/bug54372.phpt` 1/1 using the rebuilt reviewed public binary `/tmp/phpc-reviewed-public-687fcc41-20260528-target2/debug/phpc`; no source patch, no cargo gate beyond the authorized binary recovery, no full-suite run, and no percent change |
| `Zend/tests/magic_methods/bug68652.phpt` destructor/static-singleton recursion PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing destructor/static-singleton behavior handles the recursion target without a source patch, passing wrapper PHPT `Zend/tests/magic_methods/bug68652.phpt` 1/1 using the rebuilt reviewed public binary `/tmp/phpc-reviewed-public-687fcc41-20260528-target2/debug/phpc`; no source patch, no cargo gate beyond the authorized binary recovery, no full-suite run, and no percent change |
| `Zend/tests/magic_methods/bug69025.phpt` `__callStatic` missing-static dispatch PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing missing-static-method dispatch reaches public `__callStatic()` for this PHPT, passing wrapper PHPT `Zend/tests/magic_methods/bug69025.phpt` 1/1 using the rebuilt reviewed public binary `/tmp/phpc-reviewed-public-687fcc41-20260528-target2/debug/phpc`; no source patch, no cargo gate beyond the authorized binary recovery, no full-suite run, and no percent change |
| `Zend/tests/magic_methods/bug71818.phpt` destructor array mutation PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing destructor/array mutation handling passes wrapper PHPT `Zend/tests/magic_methods/bug71818.phpt` 1/1 using the replacement reviewed public binary `/tmp/phpc-reviewed-public-1b459adf-20260528-target/debug/phpc`; no source patch, no cargo gate beyond the reviewed binary recovery, no full-suite run, and no percent change |
| `Zend/tests/magic_methods/bug72177_2.phpt` ReflectionProperty/destructor scope PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing ReflectionProperty-set object/destructor scope behavior passes wrapper PHPT `Zend/tests/magic_methods/bug72177_2.phpt` 1/1 using the replacement reviewed public binary `/tmp/phpc-reviewed-public-1b459adf-20260528-target/debug/phpc`; no source patch, no cargo gate beyond the reviewed binary recovery, no full-suite run, and no percent change |
| `Zend/tests/magic_methods/bug75420.3.phpt` indirect magic argument modification PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing magic-method argument handling covers the indirect modification case, passing wrapper PHPT `Zend/tests/magic_methods/bug75420.3.phpt` 1/1 using the reviewed public binary `/tmp/phpc-reviewed-public-6d161522-20260528-target/debug/phpc`; no source patch, no cargo gate beyond reviewed binary recovery, no full-suite run, and no percent change |
| `Zend/tests/magic_methods/bug75420.9.phpt` indirect magic argument modification PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing magic-method argument handling covers the additional indirect modification case, passing wrapper PHPT `Zend/tests/magic_methods/bug75420.9.phpt` 1/1 using the reviewed public binary `/tmp/phpc-reviewed-public-6d161522-20260528-target/debug/phpc`; no source patch, no cargo gate beyond reviewed binary recovery, no full-suite run, and no percent change |
| `Zend/tests/magic_methods/bug75420.11.phpt` indirect magic argument modification PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing magic-method argument handling covers the additional indirect modification case, passing wrapper PHPT `Zend/tests/magic_methods/bug75420.11.phpt` 1/1 using the reviewed public binary `/tmp/phpc-reviewed-public-6d161522-20260528-target/debug/phpc`; no source patch, no cargo gate beyond reviewed binary recovery, no full-suite run, and no percent change |
| `Zend/tests/magic_methods/bug75420.13.phpt` indirect magic argument modification PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing magic-method argument handling covers the additional indirect modification case, passing wrapper PHPT `Zend/tests/magic_methods/bug75420.13.phpt` 1/1 using the reviewed public binary `/tmp/phpc-reviewed-public-6d161522-20260528-target/debug/phpc`; no source patch, no cargo gate beyond reviewed binary recovery, no full-suite run, and no percent change |
| `Zend/tests/magic_methods/bug75420.14.phpt` indirect magic argument modification PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing magic-method argument handling covers the additional indirect modification case, passing wrapper PHPT `Zend/tests/magic_methods/bug75420.14.phpt` 1/1 using the reviewed public binary `/tmp/phpc-reviewed-public-6d161522-20260528-target/debug/phpc`; no source patch, no cargo gate beyond reviewed binary recovery, no full-suite run, and no percent change |
| `Zend/tests/magic_methods/class_toString_concat_with_itself.phpt` `__toString()` self-concat PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing `__toString()` conversion handles concatenating the same class instance with itself, passing wrapper PHPT `Zend/tests/magic_methods/class_toString_concat_with_itself.phpt` 1/1 using the reviewed public binary `/tmp/phpc-reviewed-public-6d161522-20260528-target/debug/phpc`; no source patch, no cargo gate beyond reviewed binary recovery, no full-suite run, and no percent change |
| `Zend/tests/magic_methods/constructor_args.phpt` constructor argument arity PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing constructor argument binding/arity behavior handles the upstream magic-method target, passing wrapper PHPT `Zend/tests/magic_methods/constructor_args.phpt` 1/1 using the reviewed public binary `/tmp/phpc-reviewed-public-6d161522-20260528-target/debug/phpc`; no source patch, no cargo gate beyond reviewed binary recovery, no full-suite run, and no percent change |
| `Zend/tests/magic_methods/magic_get_destroy_object.phpt` `__get()` destruction ordering PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing magic `__get()` object lifetime/destruction behavior handles the upstream target, passing wrapper PHPT `Zend/tests/magic_methods/magic_get_destroy_object.phpt` 1/1 using the reviewed public binary `/tmp/phpc-reviewed-public-6d161522-20260528-target/debug/phpc`; no source patch, no cargo gate beyond reviewed binary recovery, no full-suite run, and no percent change |
| `Zend/tests/magic_methods/magic_methods_001.phpt` magic method behavior PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing magic-method dispatch/property behavior handles the upstream coverage target, passing wrapper PHPT `Zend/tests/magic_methods/magic_methods_001.phpt` 1/1 using the reviewed public binary `/tmp/phpc-reviewed-public-6d161522-20260528-target/debug/phpc`; no source patch, no cargo gate beyond reviewed binary recovery, no full-suite run, and no percent change |
| `Zend/tests/magic_methods/stringable_trait.phpt` trait-provided `__toString()` Stringable PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing trait method composition and `__toString()` handling make the class Stringable for this upstream target, passing wrapper PHPT `Zend/tests/magic_methods/stringable_trait.phpt` 1/1 using the reviewed public binary `/tmp/phpc-reviewed-public-6d161522-20260528-target/debug/phpc`; no source patch, no cargo gate beyond reviewed binary recovery, no full-suite run, and no percent change |
| `tests/classes/method_call_variation_001.phpt` dynamic method-call global-function-name variation PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing method-call/name-resolution behavior handles `$a->$b[Y]()` and `$a->X[Y]()` as global function-name calls for this upstream target, passing wrapper PHPT `tests/classes/method_call_variation_001.phpt` 1/1 using the reviewed public binary `/tmp/phpc-reviewed-public-6d161522-20260528-target/debug/phpc`; no source patch, no cargo gate beyond reviewed binary recovery, no full-suite run, and no percent change |
| `Zend/tests/dynamic_call/bug52940.phpt` dynamic-call array callback PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and phpc-25 `SAFE-FOR-PROGRESS`; existing dynamic-call handling covers the upstream callback target, passing wrapper PHPT `Zend/tests/dynamic_call/bug52940.phpt` 1/1 using the reviewed public binary `/tmp/phpc-reviewed-public-6d161522-20260528-target/debug/phpc`; no source patch, no cargo gate beyond reviewed binary recovery, no full-suite run, and no percent change |
| p15 `returnByReference.005` rebased object-receiver static reference-return dispatch | GO-CANDIDATE after independent p7 review and p14 `SAFE-FOR-PROGRESS`; patch SHA `9487557714a456f2b3f416af7db1ed9866c6428dd6072bd143afe6a86dd27895`; focused Rust gate, `cargo build -p phpc`, and wrapper PHPTs `tests/lang/returnByReference.005.phpt`, `tests/lang/returnByReference.004.phpt`, and `tests/lang/returnByReference.003.phpt` passed 3/3; no full-suite run and no percent change |
| `Zend/tests/dereference/dereference_005.phpt` array dereference PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing generalized ArrayAccess/object dereference handling passes wrapper PHPT `Zend/tests/dereference/dereference_005.phpt` 1/1; no source patch, no cargo gate, no full-suite run, and no percent change |
| `Zend/tests/dereference/dereference_008.phpt` dynamic-method array dereference PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing dynamic-method dereference/reference behavior passes wrapper PHPT `Zend/tests/dereference/dereference_008.phpt` 1/1; no source patch, no cargo gate, no full-suite run, and no percent change |
| `tests/lang/passByReference_008.phpt` / `tests/lang/passByReference_009.phpt` duplicate by-reference/by-value call-frame PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing call-frame argument binding/evaluation semantics pass wrapper PHPTs 2/2; no source patch, no cargo gate, no full-suite run, and no percent change |
| `Zend/tests/array_append_by_reference.phpt` append slot by-reference argument PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing array append/reference behavior allows an appended slot to be passed by reference in the upstream PHPT, passing wrapper PHPT `Zend/tests/array_append_by_reference.phpt` 1/1 using the rebuilt reviewed public binary `/tmp/phpc-reviewed-public-687fcc41-20260528-target2/debug/phpc`; no source patch, no cargo gate beyond the authorized binary recovery, no full-suite run, and no percent change |
| `Zend/tests/attributes/override/001.phpt` `Override` attribute baseline PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and phpc-24 `SAFE-FOR-PROGRESS`; existing attribute handling covers the upstream `Override` baseline target, passing wrapper PHPT `Zend/tests/attributes/override/001.phpt` 1/1 using the reviewed public binary `/tmp/phpc-reviewed-public-6d161522-20260528-target/debug/phpc`; no source patch, no cargo gate beyond reviewed binary recovery, no full-suite run, and no percent change |
| `Zend/tests/object_property_ref_incdec.phpt` object-property reference increment/decrement PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing object-property/reference behavior handles increment and decrement through an object property reference, passing wrapper PHPT `Zend/tests/object_property_ref_incdec.phpt` 1/1 using the reviewed public binary `/tmp/phpc-reviewed-public-6d161522-20260528-target/debug/phpc`; no source patch, no cargo gate beyond reviewed binary recovery, no full-suite run, and no percent change |
| `Zend/tests/assign_dim_op_same_var.phpt` assign-dim op same-variable PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing array-dimension assignment/update behavior handles using the same variable on both sides of the assignment op, passing wrapper PHPT `Zend/tests/assign_dim_op_same_var.phpt` 1/1 using the reviewed public binary `/tmp/phpc-reviewed-public-6d161522-20260528-target/debug/phpc`; no source patch, no cargo gate beyond reviewed binary recovery, no full-suite run, and no percent change |
| `Zend/tests/assign_to_obj_001.phpt` assignment to object expression PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing object-assignment expression behavior handles assigning to an object target in the upstream PHPT, passing wrapper PHPT `Zend/tests/assign_to_obj_001.phpt` 1/1 using the reviewed public binary `/tmp/phpc-reviewed-public-6d161522-20260528-target/debug/phpc`; no source patch, no cargo gate beyond reviewed binary recovery, no full-suite run, and no percent change |
| `Zend/tests/assign_to_var_001.phpt` assignment to variable expression PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing variable-assignment expression behavior handles assigning through the upstream target, passing wrapper PHPT `Zend/tests/assign_to_var_001.phpt` 1/1 using the reviewed public binary `/tmp/phpc-reviewed-public-6d161522-20260528-target/debug/phpc`; no source patch, no cargo gate beyond reviewed binary recovery, no full-suite run, and no percent change |
| `Zend/tests/assign_to_var_002.phpt` assignment to variable expression PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and critic `SAFE-FOR-PROGRESS`; existing variable-assignment expression behavior handles the follow-up upstream target, passing wrapper PHPT `Zend/tests/assign_to_var_002.phpt` 1/1 using the reviewed public binary `/tmp/phpc-reviewed-public-6d161522-20260528-target/debug/phpc`; no source patch, no cargo gate beyond reviewed binary recovery, no full-suite run, and no percent change |
| `Zend/tests/assign_to_var_004.phpt` assignment to variable expression PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and critic `SAFE-FOR-PROGRESS`; existing variable-assignment expression behavior handles the additional follow-up upstream target, passing wrapper PHPT `Zend/tests/assign_to_var_004.phpt` 1/1 using the reviewed public binary `/tmp/phpc-reviewed-public-6d161522-20260528-target/debug/phpc`; no source patch, no cargo gate beyond reviewed binary recovery, no full-suite run, and no percent change |
| `Zend/tests/jump/jump01.phpt` backward `goto` PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing goto label resolution and backward jump execution through a looped condition passes wrapper PHPT `Zend/tests/jump/jump01.phpt` 1/1 using the rebuilt reviewed public binary `/tmp/phpc-reviewed-public-687fcc41-20260528-target2/debug/phpc`; no source patch, no cargo gate beyond the authorized binary recovery, no full-suite run, and no percent change |
| `Zend/tests/jump/jump02.phpt` forward `goto` PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing goto label resolution and forward jump execution to a later label after a loop passes wrapper PHPT `Zend/tests/jump/jump02.phpt` 1/1 using the rebuilt reviewed public binary `/tmp/phpc-reviewed-public-687fcc41-20260528-target2/debug/phpc`; no source patch, no cargo gate beyond the authorized binary recovery, no full-suite run, and no percent change |
| `Zend/tests/jump/jump04.phpt` backward `goto` from loop PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing goto execution supports a backward jump from inside a loop to a prior label outside that loop, passing wrapper PHPT `Zend/tests/jump/jump04.phpt` 1/1 using the rebuilt reviewed public binary `/tmp/phpc-reviewed-public-687fcc41-20260528-target2/debug/phpc`; no source patch, no cargo gate beyond the authorized binary recovery, no full-suite run, and no percent change |
| `Zend/tests/jump/jump05.phpt` forward `goto` from nested loop/switch PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing goto execution supports a forward jump out of nested foreach/switch control flow to a later label, passing wrapper PHPT `Zend/tests/jump/jump05.phpt` 1/1 using the rebuilt reviewed public binary `/tmp/phpc-reviewed-public-687fcc41-20260528-target2/debug/phpc`; no source patch, no cargo gate beyond the authorized binary recovery, no full-suite run, and no percent change |
| `Zend/tests/jump/jump11.phpt` goto inside switch in constructor PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing goto label resolution and execution handles forward/backward jumps inside a switch in a constructor, passing wrapper PHPT `Zend/tests/jump/jump11.phpt` 1/1 using the replacement reviewed public binary `/tmp/phpc-reviewed-public-1b459adf-20260528-target/debug/phpc`; no source patch, no cargo gate beyond the reviewed binary recovery, no full-suite run, and no percent change |
| `Zend/tests/jump/jump15.phpt` forward `goto` from loop PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing goto label resolution and execution handles a forward jump out of a loop, passing wrapper PHPT `Zend/tests/jump/jump15.phpt` 1/1 using the replacement reviewed public binary `/tmp/phpc-reviewed-public-1b459adf-20260528-target/debug/phpc`; no source patch, no cargo gate beyond the reviewed binary recovery, no full-suite run, and no percent change |
| `Zend/tests/try/finally_goto_003.phpt` goto into finally block PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after binary-recovery p7 rerun and p14 `SAFE-FOR-PROGRESS`; existing try/finally/goto behavior handles the upstream finally-block jump case, passing wrapper PHPT `Zend/tests/try/finally_goto_003.phpt` 1/1 using the reviewed public binary `/tmp/phpc-reviewed-public-6d161522-20260528-target/debug/phpc`; prior missing-binary NO-GO was infrastructure-only and superseded; no source patch, no cargo gate beyond reviewed binary recovery, no full-suite run, and no percent change |
| `Zend/tests/try/finally_goto_005.phpt` goto/finally control-flow PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing try/finally/goto behavior handles the upstream control-flow case, passing wrapper PHPT `Zend/tests/try/finally_goto_005.phpt` 1/1 using the reviewed public binary `/tmp/phpc-reviewed-public-6d161522-20260528-target/debug/phpc`; prior temp-target mkdir failure was infrastructure-only and superseded; no source patch, no cargo gate beyond reviewed binary recovery, no full-suite run, and no percent change |
| `Zend/tests/try/try_finally_006.phpt` try/finally near-goto PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing try/finally control-flow behavior handles the upstream near-goto case, passing wrapper PHPT `Zend/tests/try/try_finally_006.phpt` 1/1 using the reviewed public binary `/tmp/phpc-reviewed-public-6d161522-20260528-target/debug/phpc`; no source patch, no cargo gate beyond reviewed binary recovery, no full-suite run, and no percent change |
| `Zend/tests/try/try_finally_013.phpt` try/finally return-in-loop PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing try/finally loop return behavior handles return in try and finally inside a loop, passing wrapper PHPT `Zend/tests/try/try_finally_013.phpt` 1/1 using the reviewed public binary `/tmp/phpc-reviewed-public-6d161522-20260528-target/debug/phpc`; no source patch, no cargo gate beyond reviewed binary recovery, no full-suite run, and no percent change |
| `Zend/tests/try/try_finally_014.phpt` nested-loop break/finally return PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing try/finally nested-loop control-flow behavior handles `break 2` in try with return in finally, passing wrapper PHPT `Zend/tests/try/try_finally_014.phpt` 1/1 using the reviewed public binary `/tmp/phpc-reviewed-public-6d161522-20260528-target/debug/phpc`; no source patch, no cargo gate beyond reviewed binary recovery, no full-suite run, and no percent change |
| `Zend/tests/try/try_finally_015.phpt` loop return ignored by finally PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing try/finally loop control-flow behavior handles the upstream ignored-return case, passing wrapper PHPT `Zend/tests/try/try_finally_015.phpt` 1/1 using the reviewed public binary `/tmp/phpc-reviewed-public-6d161522-20260528-target/debug/phpc`; no source patch, no cargo gate beyond reviewed binary recovery, no full-suite run, and no percent change |
| `Zend/tests/try/try_finally_021.phpt` nested try/finally control-flow PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing try/finally control-flow behavior handles the upstream nested finally case, passing wrapper PHPT `Zend/tests/try/try_finally_021.phpt` 1/1 using the reviewed public binary `/tmp/phpc-reviewed-public-6d161522-20260528-target/debug/phpc`; no source patch, no cargo gate beyond reviewed binary recovery, no full-suite run, and no percent change |
| `Zend/tests/try/try_catch_finally_004.phpt` catch rethrow/finally PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing try/catch/finally exception control-flow behavior handles the upstream rethrow-in-catch case, passing wrapper PHPT `Zend/tests/try/try_catch_finally_004.phpt` 1/1 using the reviewed public binary `/tmp/phpc-reviewed-public-6d161522-20260528-target/debug/phpc`; no source patch, no cargo gate beyond reviewed binary recovery, no full-suite run, and no percent change |
| `Zend/tests/try/try_catch_finally_006.phpt` try/catch/finally control-flow PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing try/catch/finally exception control-flow behavior handles the upstream case, passing wrapper PHPT `Zend/tests/try/try_catch_finally_006.phpt` 1/1 using the reviewed public binary `/tmp/phpc-reviewed-public-6d161522-20260528-target/debug/phpc`; no source patch, no cargo gate beyond reviewed binary recovery, no full-suite run, and no percent change |
| `Zend/tests/try/catch_finally_002.phpt` catch/finally return PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing try/catch/finally return control-flow behavior handles the upstream basic return case, passing wrapper PHPT `Zend/tests/try/catch_finally_002.phpt` 1/1 using the reviewed public binary `/tmp/phpc-reviewed-public-6d161522-20260528-target/debug/phpc`; no source patch, no cargo gate beyond reviewed binary recovery, no full-suite run, and no percent change |
| `Zend/tests/try/bug72215.phpt` finally-modified return PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing try/finally return-value behavior handles the upstream modified-in-finally case, passing wrapper PHPT `Zend/tests/try/bug72215.phpt` 1/1 using the reviewed public binary `/tmp/phpc-reviewed-public-6d161522-20260528-target/debug/phpc`; no source patch, no cargo gate beyond reviewed binary recovery, no full-suite run, and no percent change |
| `Zend/tests/try/bug72215_1.phpt` by-reference finally return PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing try/finally by-reference return-value behavior handles the upstream alias modified-in-finally case, passing wrapper PHPT `Zend/tests/try/bug72215_1.phpt` 1/1 using the reviewed public binary `/tmp/phpc-reviewed-public-6d161522-20260528-target/debug/phpc`; no source patch, no cargo gate beyond reviewed binary recovery, no full-suite run, and no percent change |
| `Zend/tests/type_declarations/typed_properties_011.phpt` typed-property array reference fetch PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing typed-property/reference behavior allows fetching a typed property by reference into an array and passes wrapper PHPT `Zend/tests/type_declarations/typed_properties_011.phpt` 1/1 using the pass005 repair2 reviewed binary; no source patch, no cargo gate, no full-suite run, and no percent change |
| `Zend/tests/type_declarations/typed_properties_023.phpt` typed static property PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing typed static-property behavior handles scalar coercion, increment/static reads and writes, and by-reference-return assignment into typed static properties, passing wrapper PHPT `Zend/tests/type_declarations/typed_properties_023.phpt` 1/1 using the rebuilt reviewed public binary `/tmp/phpc-reviewed-public-687fcc41-20260528-target2/debug/phpc`; no source patch, no cargo gate beyond the authorized binary recovery, no full-suite run, and no percent change |
| `Zend/tests/type_declarations/typed_properties_024.phpt` private typed-property inheritance isolation PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing typed-property inheritance behavior permits a subclass public typed property to reuse the name of a parent private typed property without conflict, passing wrapper PHPT `Zend/tests/type_declarations/typed_properties_024.phpt` 1/1 using the rebuilt reviewed public binary `/tmp/phpc-reviewed-public-687fcc41-20260528-target2/debug/phpc`; no source patch, no cargo gate beyond the authorized binary recovery, no full-suite run, and no percent change |
| `Zend/tests/type_declarations/typed_properties_027.phpt` float typed-property widening PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing typed-property assignment/read behavior widens an integer assignment into a float typed property, passing wrapper PHPT `Zend/tests/type_declarations/typed_properties_027.phpt` 1/1 using the rebuilt reviewed public binary `/tmp/phpc-reviewed-public-687fcc41-20260528-target2/debug/phpc`; no source patch, no cargo gate beyond the authorized binary recovery, no full-suite run, and no percent change |
| `Zend/tests/type_declarations/typed_properties_028.phpt` weak-mode typed-property coercion PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing typed-property assignment/read behavior coerces a string value into an int typed property when strict types are off, passing wrapper PHPT `Zend/tests/type_declarations/typed_properties_028.phpt` 1/1 using the rebuilt reviewed public binary `/tmp/phpc-reviewed-public-687fcc41-20260528-target2/debug/phpc`; no source patch, no cargo gate beyond the authorized binary recovery, no full-suite run, and no percent change |
| `Zend/tests/type_declarations/typed_properties_041.phpt` weak string conversion into typed property PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing typed-property assignment behavior converts the string `"10"` into the declared integer property value in weak mode, passing wrapper PHPT `Zend/tests/type_declarations/typed_properties_041.phpt` 1/1 using the replacement reviewed public binary `/tmp/phpc-reviewed-public-1b459adf-20260528-target/debug/phpc`; no source patch, no cargo gate beyond the reviewed binary recovery, no full-suite run, and no percent change |
| `Zend/tests/type_declarations/typed_properties_042.phpt` typed-property assignment source duplication PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing typed-property assignment behavior duplicates the source value correctly while assigning into the typed property, passing wrapper PHPT `Zend/tests/type_declarations/typed_properties_042.phpt` 1/1 using the replacement reviewed public binary `/tmp/phpc-reviewed-public-1b459adf-20260528-target/debug/phpc`; no source patch, no cargo gate beyond the reviewed binary recovery, no full-suite run, and no percent change |
| `Zend/tests/type_declarations/typed_properties_090.phpt` typed-property unset reference shadowing PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing typed-property unset/reference behavior respects shadowing when typed properties contain references, passing wrapper PHPT `Zend/tests/type_declarations/typed_properties_090.phpt` 1/1 using the reviewed public binary `/tmp/phpc-reviewed-public-6d161522-20260528-target/debug/phpc`; no source patch, no cargo gate beyond reviewed binary recovery, no full-suite run, and no percent change |
| `Zend/tests/type_declarations/typed_properties_098.phpt` uninitialized property by-reference initialization PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing typed-property/reference behavior initializes an unset untyped property to `null` when taken by reference, passing wrapper PHPT `Zend/tests/type_declarations/typed_properties_098.phpt` 1/1 using the reviewed public binary `/tmp/phpc-reviewed-public-6d161522-20260528-target/debug/phpc`; no source patch, no cargo gate beyond reviewed binary recovery, no full-suite run, and no percent change |
| `Zend/tests/type_declarations/typed_properties_100.phpt` invisible-property `__get` type-bypass PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing typed-property/magic `__get` behavior does not enforce the inaccessible property type on values returned through magic access, passing wrapper PHPT `Zend/tests/type_declarations/typed_properties_100.phpt` 1/1 using the reviewed public binary `/tmp/phpc-reviewed-public-6d161522-20260528-target/debug/phpc`; no source patch, no cargo gate beyond reviewed binary recovery, no full-suite run, and no percent change |
| `Zend/tests/list/list_004.phpt` `list()` assignment from array reference PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing list-assignment/reference behavior reads from an aliased array and passes wrapper PHPT `Zend/tests/list/list_004.phpt` 1/1 using the pass005 repair2 reviewed binary; no source patch, no cargo gate, no full-suite run, and no percent change |
| `Zend/tests/list/bug65969.phpt` chain assignment with `list()` PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing chain assignment behavior lets `list()` destructure the same array assigned to an object property, passing wrapper PHPT `Zend/tests/list/bug65969.phpt` 1/1 using the rebuilt reviewed public binary `/tmp/phpc-reviewed-public-687fcc41-20260528-target2/debug/phpc`; no source patch, no cargo gate beyond the authorized binary recovery, no full-suite run, and no percent change |
| `Zend/tests/list/bug72395.phpt` `list()` regression PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing list/foreach behavior covers the php-src regression target, passing wrapper PHPT `Zend/tests/list/bug72395.phpt` 1/1 using the rebuilt reviewed public binary `/tmp/phpc-reviewed-public-687fcc41-20260528-target2/debug/phpc`; no source patch, no cargo gate beyond the authorized binary recovery, no full-suite run, and no percent change |
| `Zend/tests/variadic/basic.phpt` basic variadic argument packing PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing variadic call binding packs surplus arguments and passes wrapper PHPT `Zend/tests/variadic/basic.phpt` 1/1 using the pass005 repair2 reviewed binary; no source patch, no cargo gate, no full-suite run, and no percent change |
| `Zend/tests/variadic/optional_params.phpt` optional-parameter-before-variadic PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and durable p14 `SAFE-FOR-PROGRESS`; existing variadic call binding preserves optional defaults before collecting surplus arguments and passes wrapper PHPT `Zend/tests/variadic/optional_params.phpt` 1/1 using the pass005 repair2 reviewed binary; no source patch, no cargo gate, no full-suite run, and no percent change |
| `Zend/tests/variadic/removing_parameter_error.phpt` remove required parameter before variadic PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing declaration compatibility accepts removing the required parameter before a variadic parameter, passing wrapper PHPT `Zend/tests/variadic/removing_parameter_error.phpt` 1/1 using the rebuilt reviewed public binary `/tmp/phpc-reviewed-public-687fcc41-20260528-target2/debug/phpc`; no source patch, no cargo gate beyond the authorized binary recovery, no full-suite run, and no percent change |
| `Zend/tests/variadic/variadic_implements_non_variadic.phpt` variadic implementation widening PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing method compatibility behavior accepts an implementation that widens a non-variadic interface method to a variadic child method, passing wrapper PHPT `Zend/tests/variadic/variadic_implements_non_variadic.phpt` 1/1 using the rebuilt reviewed public binary `/tmp/phpc-reviewed-public-687fcc41-20260528-target2/debug/phpc`; no source patch, no cargo gate beyond the authorized binary recovery, no full-suite run, and no percent change |
| `Zend/tests/variadic/bug67938.phpt` interface method variadic-tail implementation PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing method-compatibility and variadic dispatch behavior supports implementing interface methods by adding variadic tails and calling with or without extra variadic arguments, passing wrapper PHPT `Zend/tests/variadic/bug67938.phpt` 1/1 using the rebuilt reviewed public binary `/tmp/phpc-reviewed-public-687fcc41-20260528-target2/debug/phpc`; no source patch, no cargo gate beyond the authorized binary recovery, no full-suite run, and no percent change |
| `Zend/tests/restrict_globals/globals_in_globals.phpt` `$GLOBALS` self-key absence PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing `$GLOBALS` behavior keeps the copied globals table from containing a `GLOBALS` self-key, passing wrapper PHPT `Zend/tests/restrict_globals/globals_in_globals.phpt` 1/1 using the rebuilt reviewed public binary `/tmp/phpc-reviewed-public-687fcc41-20260528-target2/debug/phpc`; no source patch, no cargo gate beyond the authorized binary recovery, no full-suite run, and no percent change |
| `Zend/tests/bug39944.phpt` reference invocation | PR #2/r82 is parked/superseded for Batch002; do not stack it with PR #4 because both conflict in `compiler/src/interpreter.rs` and `compiler/tests/functions_and_scopes.rs` |
| Magic visibility warnings | PR #3 is `REBASE-NEEDED` for Batch 002 after r81/stack10 due docs conflict; production/test hunks replay |
| Foreach `$GLOBALS` lane | PASS-NO-PATCH accepted by reviewer; accepted stack10 passes `foreach_unset_globals`, `foreach_reference`, and `foreach_temp_array_expr_with_refs` |
| `Zend/tests/foreach/foreach_unset_globals.phpt` foreach over local array while unsetting `$GLOBALS` PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after corrected independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing foreach behavior traverses the local array while unsetting matching `$GLOBALS[$key]` entries without mutating the iterated array, passing wrapper PHPT `Zend/tests/foreach/foreach_unset_globals.phpt` 1/1 using the rebuilt reviewed public binary `/tmp/phpc-reviewed-public-687fcc41-20260528-target2/debug/phpc`; no source patch, no cargo gate beyond the authorized binary recovery, no full-suite run, and no percent change |
| Foreach object-property by-reference lane | GO-CANDIDATE after independent review; focused PHPT `Zend/tests/foreach/foreach_by_ref_to_property.phpt` plus foreach anchors passed 3/3, with PR #3/#4 stack compatibility checks |
| `Zend/tests/foreach/foreach_reference.phpt` by-reference foreach lingering alias PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing foreach/reference behavior preserves the last-element alias through `array_values()` and `array_reverse()` and passes wrapper PHPT `Zend/tests/foreach/foreach_reference.phpt` 1/1 using the pass005 repair2 reviewed binary; no source patch, no cargo gate, no full-suite run, and no percent change |
| `Zend/tests/foreach/foreach_temp_array_expr_with_refs.phpt` temporary array references foreach PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing foreach/reference behavior iterates temporary arrays containing references by reference and mutates the original variables, passing wrapper PHPT `Zend/tests/foreach/foreach_temp_array_expr_with_refs.phpt` 1/1 using the pass005 repair2 reviewed binary; no source patch, no cargo gate, no full-suite run, and no percent change |
| `Zend/tests/foreach/foreach_by_ref_repacking_insert.phpt` by-reference foreach packed-to-hash repacking PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing foreach/reference behavior handles packed-to-hash insert/repacking at the end of by-reference iteration and passes wrapper PHPT `Zend/tests/foreach/foreach_by_ref_repacking_insert.phpt` 1/1 using the pass005 repair2 reviewed binary; no source patch, no cargo gate, no full-suite run, and no percent change |
| `Zend/tests/foreach/goto_in_foreach.phpt` goto into foreach body PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing foreach/goto control flow enters the foreach body and continues after the loop, passing wrapper PHPT `Zend/tests/foreach/goto_in_foreach.phpt` 1/1 using the rebuilt reviewed public binary `/tmp/phpc-reviewed-public-687fcc41-20260528-target2/debug/phpc`; no source patch, no cargo gate beyond the authorized binary recovery, no full-suite run, and no percent change |
| `Zend/tests/foreach/bug37046.phpt` nested foreach static-scope PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing foreach/static-local behavior handles nested foreach loops over arrays returned by a function using a static local, passing wrapper PHPT `Zend/tests/foreach/bug37046.phpt` 1/1 using the rebuilt reviewed public binary `/tmp/phpc-reviewed-public-687fcc41-20260528-target2/debug/phpc`; no source patch, no cargo gate beyond the authorized binary recovery, no full-suite run, and no percent change |
| `Zend/tests/foreach/bug39990.phpt` foreach over overloaded `__get` array PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing magic-property/foreach behavior iterates the array returned by overloaded `__get()` property access, passing wrapper PHPT `Zend/tests/foreach/bug39990.phpt` 1/1 using the rebuilt reviewed public binary `/tmp/phpc-reviewed-public-687fcc41-20260528-target2/debug/phpc`; no source patch, no cargo gate beyond the authorized binary recovery, no full-suite run, and no percent change |
| `Zend/tests/foreach/bug76800.phpt` by-reference foreach sparse-key mutation PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing by-reference foreach mutation behavior handles insertion/unset without skipping the sparse original key in this PHPT, passing wrapper PHPT `Zend/tests/foreach/bug76800.phpt` 1/1 using the rebuilt reviewed public binary `/tmp/phpc-reviewed-public-687fcc41-20260528-target2/debug/phpc`; no source patch, no cargo gate beyond the authorized binary recovery, no full-suite run, and no percent change |
| `Zend/tests/foreach/gh11222.phpt` by-reference foreach rehash key-visit PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing by-reference foreach behavior handles unset/append mutations that rehash the iterated array without jumping over keys, passing wrapper PHPT `Zend/tests/foreach/gh11222.phpt` 1/1 using the rebuilt reviewed public binary `/tmp/phpc-reviewed-public-687fcc41-20260528-target2/debug/phpc`; no source patch, no cargo gate beyond the authorized binary recovery, no full-suite run, and no percent change |
| `Zend/tests/foreach/foreach_005.phpt` nested by-reference foreach PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing foreach/reference behavior passes wrapper PHPT `Zend/tests/foreach/foreach_005.phpt` 1/1 using the pass005 repair2 reviewed binary; no source patch, no cargo gate, no full-suite run, and no percent change |
| `Zend/tests/foreach/foreach_006.phpt` repeated by-reference foreach constant-array PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing foreach/reference behavior passes wrapper PHPT `Zend/tests/foreach/foreach_006.phpt` 1/1 using the pass005 repair2 reviewed binary; no source patch, no cargo gate, no full-suite run, and no percent change |
| `Zend/tests/foreach/foreach_007.phpt` by-reference foreach append-at-end PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing foreach/reference behavior visits the newly inserted element and passes wrapper PHPT `Zend/tests/foreach/foreach_007.phpt` 1/1 using the pass005 repair2 reviewed binary; no source patch, no cargo gate, no full-suite run, and no percent change |
| `Zend/tests/foreach/foreach_008.phpt` nested by-reference foreach unset PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing foreach/reference behavior handles nested by-reference foreach while elements are unset and passes wrapper PHPT `Zend/tests/foreach/foreach_008.phpt` 1/1 using the pass005 repair2 reviewed binary; no source patch, no cargo gate, no full-suite run, and no percent change |
| `Zend/tests/foreach/foreach_009.phpt` nested by-reference foreach sparsified-array PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing foreach/reference behavior handles nested by-reference foreach over a sparsified/resized array while inserting during inner iteration and passes wrapper PHPT `Zend/tests/foreach/foreach_009.phpt` 1/1 using the pass005 repair2 reviewed binary; no source patch, no cargo gate, no full-suite run, and no percent change |
| `Zend/tests/foreach/foreach_014.phpt` by-reference foreach `array_pop()` pointer PASS-NO-PATCH | ACCEPT-PASS-NO-PATCH after independent p7 review and p14 `SAFE-FOR-PROGRESS`; existing foreach/reference behavior handles by-reference foreach iterator pointer behavior when `array_pop()` removes the last element during iteration and passes wrapper PHPT `Zend/tests/foreach/foreach_014.phpt` 1/1 using the pass005 repair2 reviewed binary; no source patch, no cargo gate, no full-suite run, and no percent change |
| Anonymous-class dynamic-call blocker | AO scout classified this as NO-GO for Batch 001 PR 10; deferred as a broader parser/interpreter/native feature |
| PHPT focused queue | `tests/classes/__set__get_002.phpt` passes on the 9/10 stack; r85 queue now feeds additional coder lanes |
| Codex thread-store permissions | Fixed current session directory execute bit; smoke passed |
| Disk/data cleanup | Reclaimed Codex SQLite WAL; `/home` currently has 286G free |
| Agent Orchestrator migration | AO is installed, configured, polling this project, and persistent critic/reviewer/progress-reporter/coder roles are active |

## AO Control Plane

AO dashboard: `http://localhost:3000/projects/php-to-native-compiler`.

Required live roles:

| Role | Responsibility |
| --- | --- |
| Critic | Read-only audit for exact-shape lowering, shallow evidence, stale artifacts, and premature completion |
| Reviewer | Independent candidate apply/review/focused-gate proof before Batch 001 acceptance |
| Progress reporter | Keeps this `PROGRESS.md` file and durable supervisor state current after material AO events |
| Coders | Work disjoint focused PHPT lanes from the queue; each lane must produce a patch, PASS-NO-PATCH, or NO-GO artifact |

Current AO snapshot: `phpc-orchestrator` supervising; `phpc-26` critic;
`phpc-7` reviewer; `phpc-8` progress reporter; `phpc-28` Batch002 integration
worker; source-coder lanes `phpc-15`, `phpc-16`, `phpc-17`, `phpc-18`, and
`phpc-19`. Current watch targets are p28 candidate8 conflict resolution,
candidate9 compatibility after candidate8, new source patches exported by the
coder lanes, and the next authorized full-suite PHPT row. PASS-NO-PATCH row
publication is lower priority while reviewed source integration is active.

## Current Rules

- No exact-shape production lowering for individual PHPTs.
- No docs-only or tests-only progress.
- No full PHPT suite for every change.
- No batch merge before 10 accepted PRs, a full PHPT run, and regression repair.
- Legacy roadmap bars are retired; use PHPT pass rate as the only percent.

## Recent Source Anchors

| Commit | Capability | Gate log |
| --- | --- | --- |
| `0fa7b666` | Interpreter `enum_exists()` now uses SPL autoload callback/recheck for enum misses. | `state/logs/phpc-primary-enum-autoload-a5abdbb5-20260528.gates.log` |
| `2ef16e0d` | Request-scope `throw` inside active generated-C `finally` replays `finally` before the current unsupported-throw fatal boundary. | `state/logs/phpc-primary-throw-finally-fd52417e-20260528.gates.log` |
| `9c49c29b` | Generated-C comparison aborts now use cleanup-aware native error exits. | `state/logs/phpc-primary-comparison-abort-cleanup-4ed1624e-20260528.gates.log` |
| `d97a9fcf` | Dynamic runtime-registry missing required includes run active generated-C `finally` before fatal diagnostics. | `state/logs/phpc-primary-dynamic-include-finally-8a0a982f-20260528.gates.log` |

Detailed worker logs, PHPT inventory, batch review reports, and skip policy live
under `/home/claude/supervised-php-compiler/state/`.
