# First Repair Backlog Refined

Developer: developer-82
Lane: 74
Generated: 2026-06-07T17:05Z
Worktree: `/home/claude/php-to-native-compiler/.harness/worktrees/developer-82`
Branch: `work/developer-82`

Scope: read-only backlog refinement from current evidence reports. No compiler,
runtime, harness source, php-src files, docs support claims, PHPT full gate, or
public score artifacts were changed. This report does not assign source work;
it names lanes that are safe to open only after the stated gates are met.

## Decision

Do not open broad PHP feature repair lanes from the `221205Z` absent clusters.
The current direct source candidate is:

1. Internal/core readonly property write and unset diagnostics for selected
   existing core objects, starting with `Directory` and `BcMath\Number`.

That candidate is still gated on replay-binary restoration and focused
precheck reproduction. The current first deterministic work remains:

1. Keep the PHPT control-plane path closed: nonzero test command selection,
   shard harness layout/completeness, and strict expected-row accounting.
2. Restore or rebuild durable accepted/candidate release `phpc` binaries for
   focused replay.
3. Replay the direct rows below. Promote only preserved semantic failures to
   source implementation.

`eval` and variable-variable rows remain late-priority and should not drive
first-wave repair work.

## Evidence Inputs

Current inputs used for this refinement:

- `.harness/reports/first-repair-lane-evidence-readiness-dev236.md`
- `.harness/reports/m0-first-direct-failed-borked-source-repair-dev447.md`
- `.harness/reports/221205Z-direct-failed-borked-triage.md`
- `.harness/reports/phpt-binary-wrapper-availability-recheck-dev305.md`
- `.harness/reports/221205Z-shard-abort-root-cause.md`
- `.harness/reports/221205Z-shard-rerun-smoke-dev116.md`
- `.harness/reports/zero-regression-gate-preflight-checklist-dev433.md`
- `.harness/reports/focused-replay-cookbook.md`
- `.harness/reports/regression-repair-backlog-template.md`

Relevant lane-state caveat: lane 68 is still marked `assigned/development` in
SQLite, but its requested artifact
`.harness/reports/221205Z-direct-failed-borked-triage.md` is present on current
`origin/master`. Treat the artifact as current evidence and clean up the stale
lane row separately.

## Priority Backlog

### P0: Replay And Gate Prerequisites

Readiness class: `M1-control-plane` plus `M0/M1 replay enabler`.

Owner module: PHPT gate harness, focused replay setup, and scheduler/test-loop
control plane.

Why first: product repair cannot be adjudicated reliably while the accepted and
candidate release binaries are missing and the previous candidate gate had
shard abort/completeness defects.

Required prechecks:

- `python -m unittest discover -s .harness/tests -v` remains nonzero and
  command discovery returns `tools/run-tests.sh`.
- Shard redirect smoke for:
  - `php-src/ext/pdo_mysql/tests/common.phpt`
  - `php-src/ext/pdo_pgsql/tests/common.phpt`
- Durable accepted replay binary for source
  `0b917f67a37d9ca9779d77f87173b628431c2425`.
- Durable candidate replay binary for source
  `56fe9377fb46be00db5fdd30c966fdba406dc581`.
- Wrapper executable:
  `/home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper`.
- php-src pin:
  `/home/claude/php-src-phpt` at
  `f97ff597429a2fe633665a7e02d97c8077f9f90f`.

Required postchecks:

- Each focused replay run writes `results.txt`, `run-tests.log`,
  `stdout.log`, `stderr.log`, row list, exit status, and binary manifest.
- Every full-gate shard must archive its `shard-XX.tests`, `results.txt`,
  `run-tests.log`, stdout/stderr, and exit status.
- Aggregation must fail if any expected PHPT path is absent from normalized
  status.

No Rust tests are required unless a harness source patch is made. If a harness
patch is made, use focused `.harness/tests` only. No full PHPT gate should be
run for this prerequisite lane.

### P1: Internal/Core Readonly Property Diagnostics

Readiness class: `M0-direct`, promote to `M2-repair` only after focused replay
reproduces preserved semantic failures.

Owner module: runtime object metadata and interpreter object-property
write/unset diagnostics.

Owned files for a future source lane:

- `runtime/src/lib.rs`
- `compiler/src/interpreter.rs`
- `compiler/tests/standard_directory_glob_builtins.rs`
- `compiler/tests/bcmath_builtin.rs`
- `compiler/tests/syntax_boundaries.rs`
- `docs/SUPPORT.md`
- `docs/PROGRESS.md`

Primary precheck PHPT rows:

- `php-src/ext/standard/tests/directory/DirectoryClass_readonly_handle.phpt`
- `php-src/ext/standard/tests/directory/DirectoryClass_readonly_path.phpt`
- `php-src/ext/bcmath/tests/number/properties_write_error.phpt`
- `php-src/ext/bcmath/tests/number/properties_unset.phpt`

Wider postcheck rows after the primary four are green:

- `php-src/ext/date/tests/DatePeriod_modify_readonly_property.phpt`
- `php-src/ext/date/tests/DatePeriod_properties2.phpt`
- `php-src/ext/xmlreader/tests/014.phpt`
- `php-src/Zend/tests/attributes/deprecated/property_readonly_001.phpt`
- `php-src/Zend/tests/attributes/deprecated/property_readonly_002.phpt`

Focused Rust tests expected from the implementation lane:

```sh
cargo test -p phpc --test standard_directory_glob_builtins directory_core_properties_reject_write_and_unset_as_readonly -- --test-threads=1
cargo test -p phpc --test bcmath_builtin bcmath_number_properties_reject_write_and_unset_as_readonly -- --test-threads=1
cargo test -p php_runtime core_class_table_marks_selected_internal_properties_readonly -- --test-threads=1
cargo test -p phpc --test syntax_boundaries unsupported_readonly_property_declarations_have_stable_parse_errors -- --test-threads=1
```

CLI exercise path expected from the implementation lane:

```sh
cargo run -p phpc -- test tests/fixtures/milestone-internal-readonly-properties
cargo run -p phpc -- test --compare-php tests/fixtures/milestone-internal-readonly-properties
cargo run -p phpc -- compile tests/fixtures/milestone-internal-readonly-properties/internal_readonly_property_diagnostics.php --emit-ir
```

The compile path should keep rejecting unsupported object/property native
lowering. Do not emit misleading IR or assembly for unsupported property forms.

Docs/progress requirement:

- Update `docs/SUPPORT.md` and `docs/PROGRESS.md` only after code, tests, CLI
  exercise, and focused replay prove the selected behavior.
- Claim only selected internal/core readonly diagnostics, not general readonly
  properties.

Unsupported edges to keep explicit:

- userland readonly property and readonly class declarations remain parser
  boundaries;
- property hooks and interface property satisfaction remain a separate lane;
- readonly class semantics, trait readonly compatibility, and asymmetric
  `private(set)` / `protected(set)` visibility remain out of scope;
- readonly references, indirect modification, append, nested array writes, and
  reflection metadata are not claimed;
- `DatePeriod`, `XMLReader`, and `Deprecated` rows are wider postchecks, not
  part of the initial support claim.

### P2: SKIPIF Extension/Core Constants

Readiness class: `M0-direct` for `BORKED` rows, but owner is wrapper or
constant metadata, not PHP body semantics.

Owner module: PHPT wrapper/environment and bounded constant exposure.

Precheck PHPT rows:

- `php-src/ext/intl/tests/rangeformatter/rangeformatter_icu63_compatibility.phpt`
- `php-src/ext/openssl/tests/openssl_libctx_without_zts_argon.phpt`
- `php-src/ext/pcre/tests/grep2.phpt`

Postcheck: the rows should stop being `BORKED`; body results must be recorded
separately and must not be counted as full intl/openssl/pcre support.

Focused tests for a future implementation lane:

- constant lookup tests for `INTL_ICU_VERSION`, `ZEND_THREAD_SAFE`, and
  `PCRE_JIT_SUPPORT`;
- `phpc run` probes for `defined()` and direct constant reads;
- focused PHPT replay with the three rows above.

Docs/progress requirement: document exposed constants and the remaining
extension limits. Passing SKIPIF is not a support claim for extension body
semantics.

### P3: Object Lifecycle, Destructor, Iterator, And GC Rows

Readiness class: `M0-direct` replay first. Do not start implementation until
focused replay proves a shared semantic root.

Owner module: interpreter object lifetime, destructor dispatch, iterator
lifetime, and GC cleanup.

Precheck PHPT rows:

- `php-src/tests/classes/ctor_dtor.phpt`
- `php-src/tests/classes/factory_and_singleton_002.phpt`
- `php-src/tests/classes/iterators_002.phpt`

Postcheck expansion:

- `php-src/tests/classes/destructor_and_echo.phpt`
- `php-src/Zend/tests/bug73989.phpt`
- `php-src/Zend/tests/gc/bug63635.phpt`

Expected tests after replay proves a single source fix:

- focused interpreter tests for destructor order and protected destructor
  diagnostics;
- `phpc run` fixture proving object lifetime order;
- focused PHPT replay of precheck rows.

Unsupported edges: closure-cycle GC and iterator destruction should remain
separate unless replay proves they share the same root as basic destructor
ordering.

### P4: Assertion And Throwable Formatting Rows

Readiness class: `M0-direct` replay first.

Owner module: assertion callback handling, throwable stringification,
serialization, and uncaught throwable formatting.

Precheck PHPT row:

- `php-src/Zend/tests/assert/expect_011.phpt`

Postcheck/replay rows:

- `php-src/Zend/tests/assert/expect_008.phpt`
- `php-src/Zend/tests/serialize/bug76502.phpt`
- `php-src/Zend/tests/uncaught_exception_error_supression.phpt`

Expected tests after replay proves a source defect:

- focused interpreter tests for assertion callback result and throwable
  message formatting;
- `phpc run` fixture for the specific exception path;
- focused PHPT replay of precheck and postcheck rows.

Unsupported edges: do not mix serialization chain fixes with assertion callback
behavior unless replay proves they share a root cause.

### P5: One-Row Diagnostic Or Opcache Rows

Readiness class: `M0-direct` replay/adjudication.

Precheck rows:

- `php-src/Zend/tests/type_declarations/variance/internal_parent/unresolvable_inheritance_check_param.phpt`
- `php-src/ext/opcache/tests/opt/sccp_037.phpt`

These are deliberately lower priority than P1-P4. Treat each as a one-row lane
unless focused replay finds a broader cluster.

### P6: Large Absent Clusters

Readiness class: `M0-replay` only. No implementation lane should start from
these clusters until focused replay converts specific rows into semantic
failures.

Clusters:

| Cluster | Regressions | Current interpretation |
| --- | ---: | --- |
| Standard arrays | 249 | Absent-result cluster; replay selector only. |
| Standard strings | 197 | Absent-result cluster; replacement/search selectors only. |
| Standard file/stream/directory | about 188 | Mixed absent/control-plane evidence; split after replay. |
| SPL | 137 | Absent-result cluster; replay by class family. |
| Reflection | 110 | Absent-result cluster; replay by metadata descriptor. |

If any replay row becomes semantic, open a separate narrow lane for that exact
builtin/class/descriptor family. If replay stays absent, route to PHPT gate
completeness/control-plane rather than product source.

## Deconflicted Opening Order

1. P0 replay/gate prerequisites.
2. P1 internal/core readonly diagnostics, but only after replay reproduction.
3. P2 SKIPIF constants if managers want to reduce `BORKED` rows before product
   body fixes.
4. P3 lifecycle/destructor rows.
5. P4 assertion/throwable rows.
6. P5 one-row diagnostics/opcache rows.
7. P6 absent clusters, replay only until semantic evidence exists.

Do not run multiple source lanes over object/property metadata at the same
time. P1 owns the selected readonly diagnostic surface; property hooks,
readonly classes, trait compatibility, `DatePeriod`, `XMLReader`, and
`Deprecated` rows are postchecks or separate future lanes.

## Commands And Queries

No full PHPT gate, Cargo command, merge, or product source edit was run.

Read-only evidence checks:

```sh
git status --short --branch
git cat-file -e origin/master:.harness/reports/first-repair-backlog-refined.md
git cat-file -e origin/work/developer-416:.harness/reports/first-repair-backlog-refined-dev416.md
sed -n '1,280p' .harness/reports/first-repair-lane-evidence-readiness-dev236.md
sed -n '1,320p' .harness/reports/m0-first-direct-failed-borked-source-repair-dev447.md
sed -n '1,320p' .harness/reports/221205Z-direct-failed-borked-triage.md
sed -n '1,260p' .harness/reports/phpt-binary-wrapper-availability-recheck-dev305.md
sed -n '1,260p' .harness/reports/zero-regression-gate-preflight-checklist-dev433.md
sed -n '1,240p' .harness/reports/focused-replay-cookbook.md
rg -n "BcMath|Directory|readonly|ObjectProperty|PhpPropertyMetadata|unset|assign" runtime/src/lib.rs compiler/src/interpreter.rs compiler/tests/bcmath_builtin.rs compiler/tests/standard_directory_glob_builtins.rs compiler/tests/syntax_boundaries.rs
```

MCP SQLite queries inspected worklane state for lanes 68, 69, 78, 117, 119,
122, 146, 151, and 155.
