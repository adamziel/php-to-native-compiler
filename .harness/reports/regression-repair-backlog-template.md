# Regression Repair Lane Backlog Template

Lane: 31, developer-83

Scope: read-only manager-support template for future repair lanes from the
blocked `221205Z` PHPT gate. This document uses the current `PLAN.md` rules and
the normalized regression artifacts, but it does not claim support for any
feature and does not assign implementation work. No compiler/runtime source
edits were made.

## Evidence

- Candidate gate:
  `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377`
- Regression list:
  `regressions-from-latest-published-passes.txt`
- Candidate status artifacts:
  `current-status.normalized.tsv`, `all-results.txt`,
  `pass-regression-summary.tsv`
- Current planning rules:
  `/home/claude/php-to-native-compiler/PLAN.md`
- Supporting local reports:
  `.harness/reports/221205Z-source-diff-risk.md`,
  `.harness/reports/standard-array-replay-selector.md`,
  `.harness/reports/221205Z-standard-strings-replace-replay.md`,
  `.harness/reports/221205Z-standard-scalar-misc.md`,
  `.harness/reports/221205Z-secondary-ext.md`,
  `.harness/reports/221205Z-late-priority-overlap.md`

Authoritative public score stays at `7873 / 20294` on accepted commit
`0b917f67`. The `221205Z` candidate is blocked at `7197 / 20294`, with `1166`
latest-public PASS regressions. It cannot move public score.

Current normalized regression status shape:

| Status in candidate artifacts | Rows |
| --- | ---: |
| `ABSENT` | 1136 |
| `FAILED` | 27 |
| `BORKED` | 3 |
| **Total** | **1166** |

Planning-compatible late-priority overlap is only `5 / 1166` rows: four clear
`eval` rows and one lexical variable-variable-pattern row. Future repair lanes
should avoid `eval` and variable-variable implementation unless a manager
explicitly opens M3 work.

## Lane Readiness Classes

Use these classes before assigning implementation:

| Class | When to use | Required next action |
| --- | --- | --- |
| `M0-replay` | Regression rows are absent from candidate status/result artifacts. | Replay accepted vs candidate with exact PHPT paths and classify harness/control-plane versus semantic failure. |
| `M0-direct` | Candidate artifacts preserve `FAILED` or `BORKED` rows. | Open a narrow diagnostic or metadata repair lane with preserved failure text and focused tests. |
| `M1-control-plane` | Score files, row normalization, wrapper setup, or scheduler commands disagree. | Patch only harness/control-plane configuration and prove with low-CPU deterministic checks. |
| `M2-repair` | Replay proves a semantic compiler/runtime failure and the fix surface is narrow. | Implement with code, Rust tests, CLI exercise path, docs, progress update, and named unsupported edges. |

Do not convert an `ABSENT` cluster directly into implementation work. The
first lane for such a cluster must be replay/control-plane classification.

## Required Lane Template

Every future repair backlog item should use this schema:

| Field | Required content |
| --- | --- |
| Title | One narrow behavior or control-plane outcome. Avoid broad labels like "fix standard library". |
| Readiness class | One of `M0-replay`, `M0-direct`, `M1-control-plane`, `M2-repair`. |
| Owner module | Concrete subsystem: PHPT harness, parser, interpreter builtin family, runtime object model, SPL runtime, reflection metadata, extension constants, docs/progress. |
| Evidence source | Exact artifact/report path and the reason this is a candidate lane. |
| Precheck rows | Exact PHPT paths and current accepted/candidate status. Include at least one minimal row and one adjacent guard row when possible. |
| Postcheck rows | Exact rows that must improve or remain non-regressed after the change. Usually the precheck rows plus one adjacent guard row outside the direct fix. |
| Focused Rust tests | Existing test target or new focused test names. Use `cargo test` filters, not a full suite, unless manager/integrator requests it. |
| CLI exercise path | A `phpc run` or focused PHPT replay path that executes the changed surface. For `compile`, prove unsupported lowering is rejected rather than emitted incorrectly. |
| Docs/progress | Behavior changes require docs and `docs/PROGRESS.md`; report-only lanes do not. |
| Unsupported-edge note | Name what remains unsupported, including late-priority `eval` and variable-variable boundaries when relevant. |
| Full-gate policy | Focused rows guide the lane only. Public score movement requires a pinned full PHPT gate with zero unadjudicated PASS regressions. |

## Starter Backlog

These rows are backlog templates, not active assignments.

### P0: Replay Absent Regression Stratified Sample

| Field | Value |
| --- | --- |
| Readiness class | `M0-replay` |
| Owner module | PHPT gate/replay harness |
| Evidence source | `1136 / 1166` regressions are absent from candidate status/result artifacts. |
| Precheck rows | `php-src/ext/standard/tests/array/array_chunk2.phpt`; `php-src/ext/standard/tests/strings/str_replace_basic.phpt`; `php-src/ext/standard/tests/math/round_RoundingMode.phpt`; `php-src/ext/spl/tests/SplFileObject/SplFileObject_fgetcsv_basic.phpt`; `php-src/ext/reflection/tests/ReflectionClass_getConstants_basic.phpt`; `php-src/ext/uri/tests/rfc3986/parsing/basic_success_all.phpt`. |
| Postcheck rows | Same rows, replayed against accepted `0b917f67` and candidate `56fe9377`, with explicit classification: accepted pass, candidate pass/fail/borked/absent, and artifact path. |
| Focused Rust tests | None unless replay exposes a compiler/runtime defect. Verification is artifact parsing and command logging. |
| CLI exercise path | Record the exact `run_gate.sh`/wrapper or focused PHPT replay command used; do not run a full gate for this lane. |
| Docs/progress | Write a replay report and SQLite events only. Do not edit product docs. |
| Unsupported-edge note | If a row depends on `eval` or variable variables, tag it late-priority rather than using it as a first repair target. |

### P1: Direct Readonly/Internal Property Diagnostic Repair

| Field | Value |
| --- | --- |
| Readiness class | `M0-direct` moving to `M2-repair` after focused reproduction |
| Owner module | Runtime object model plus interpreter diagnostic mapping |
| Evidence source | Preserved `FAILED` rows in `current-status.normalized.tsv` and source-diff risk audit. |
| Precheck rows | `php-src/ext/standard/tests/directory/DirectoryClass_readonly_handle.phpt`; `php-src/ext/standard/tests/directory/DirectoryClass_readonly_path.phpt`; `php-src/ext/date/tests/DatePeriod_modify_readonly_property.phpt`; `php-src/ext/xmlreader/tests/014.phpt`; `php-src/ext/bcmath/tests/number/properties_write_error.phpt`. |
| Postcheck rows | Same rows plus `php-src/ext/bcmath/tests/number/properties_unset.phpt` and `php-src/Zend/tests/readonly_classes/readonly_class_property1.phpt` as an adjacent guard. |
| Focused Rust tests | Add or update object/property diagnostic tests in `compiler/tests/object_model.rs` and runtime property visibility tests in `runtime/src/lib.rs`; run filtered `cargo test` commands for those names. |
| CLI exercise path | Minimal `phpc run` scripts that attempt write/unset on internal readonly properties and check PHP-shaped diagnostics, plus focused PHPT replay rows. |
| Docs/progress | Update `docs/SUPPORT.md` and `docs/PROGRESS.md` only after behavior changes are implemented and tested. |
| Unsupported-edge note | Do not broaden this lane into full property-hook/asymmetric-visibility support unless exact PHPT rows and docs are added. |

### P2: Extension `SKIPIF` Constant Metadata Repair

| Field | Value |
| --- | --- |
| Readiness class | `M0-direct` |
| Owner module | Interpreter core constants / extension metadata |
| Evidence source | Preserved `BORKED` rows from skip scripts with undefined constants. |
| Precheck rows | `php-src/ext/intl/tests/rangeformatter/rangeformatter_icu63_compatibility.phpt`; `php-src/ext/openssl/tests/openssl_libctx_without_zts_argon.phpt`; `php-src/ext/pcre/tests/grep2.phpt`. |
| Postcheck rows | Same rows should stop being `BORKED`; add one simple constant probe for each extension if a fixture exists. |
| Focused Rust tests | Add focused constant lookup tests in the interpreter constant table area; run filtered `cargo test` for those tests. |
| CLI exercise path | `phpc run` snippets for `defined()`/constant reads matching `INTL_ICU_VERSION`, `ZEND_THREAD_SAFE`, and `PCRE_JIT_SUPPORT`; focused PHPT replay after. |
| Docs/progress | Document newly exposed constants and remaining extension-surface limits. |
| Unsupported-edge note | Passing `SKIPIF` does not imply support for full intl/openssl/pcre body semantics. |

### P3: Standard Array Semantic Repair Tranche

| Field | Value |
| --- | --- |
| Readiness class | `M0-replay` first, then `M2-repair` only if semantic failures reproduce |
| Owner module | Interpreter standard array builtins and runtime array value semantics |
| Evidence source | `249` standard array regressions, all absent from candidate artifacts; selector report chose no-`SKIPIF` rows. |
| Precheck rows | `php-src/ext/standard/tests/array/array_chunk2.phpt`; `php-src/ext/standard/tests/array/array_count_values.phpt`; `php-src/ext/standard/tests/array/array_diff_single_array.phpt`; `php-src/ext/standard/tests/array/array_filter_basic.phpt`; `php-src/ext/standard/tests/array/array_map_basic.phpt`; `php-src/ext/standard/tests/array/array_merge.phpt`; `php-src/ext/standard/tests/array/array_walk/array_walk_basic1.phpt`; `php-src/ext/standard/tests/array/sort/array_multisort_basic1.phpt`. |
| Postcheck rows | Same rows plus one guard for a related currently passing array row if replay identifies a semantic fix surface. |
| Focused Rust tests | Add focused interpreter/runtime tests for the exact builtin family changed; use filtered `cargo test`, not full suite. |
| CLI exercise path | Minimal `phpc run` scripts for the fixed builtin plus focused PHPT replay. |
| Docs/progress | Update array support rows in docs only when implementation and tests prove behavior. |
| Unsupported-edge note | Keep references/copy-on-write and unsupported callback edge cases named if they remain outside the fix. |

### P4: Standard String Replacement Repair Tranche

| Field | Value |
| --- | --- |
| Readiness class | `M0-replay` first |
| Owner module | Interpreter string builtins |
| Evidence source | `197` standard strings regressions are absent; replacement selector found strict `str_replace()` / `str_ireplace()` rows. |
| Precheck rows | `php-src/ext/standard/tests/strings/str_replace_basic.phpt`; `php-src/ext/standard/tests/strings/str_replace_array_refs.phpt`; `php-src/ext/standard/tests/strings/bug27675.phpt`. |
| Postcheck rows | Same rows, then expand to `php-src/ext/standard/tests/strings/bug71188.phpt` and `php-src/ext/standard/tests/strings/bug33076.phpt` only if the first replay proves semantic failure. |
| Focused Rust tests | Add targeted string replacement tests for count output, array replacements, case-insensitive shrink behavior, and argument diagnostics. |
| CLI exercise path | `phpc run` snippets for `str_replace()`/`str_ireplace()` plus focused PHPT replay. |
| Docs/progress | Update docs only after behavior is implemented and proven. |
| Unsupported-edge note | Reference-backed arrays and by-reference count output must be named if not fully repaired. |

### P5: SPL Object/Autoload Repair Tranche

| Field | Value |
| --- | --- |
| Readiness class | `M0-replay` first |
| Owner module | SPL runtime objects, iterator behavior, autoload/class lookup |
| Evidence source | `137` SPL regressions, all absent from candidate artifacts; source-diff risk includes `SplTempFileObject` and class-order autoload changes. |
| Precheck rows | `php-src/ext/spl/tests/SplFileObject/SplFileObject_fgetcsv_basic.phpt`; `php-src/ext/spl/tests/SplFileObject/SplFileObject_key_basic.phpt`; `php-src/ext/spl/tests/ArrayObject/arrayObject___construct_basic1.phpt`; `php-src/ext/spl/tests/SplObjectStorage/SplObjectStorage_current_empty_storage.phpt`; `php-src/ext/spl/tests/autoloading/spl_autoload_call_basic.phpt`. |
| Postcheck rows | Same rows; include `php-src/ext/spl/tests/autoloading/bug74372.phpt` only as late-priority-tagged context because it uses `eval`. |
| Focused Rust tests | Add targeted SPL class/object tests only after replay shows semantic failure. |
| CLI exercise path | `phpc run` snippets for the affected SPL object method plus focused PHPT replay. |
| Docs/progress | Update SPL support docs with exact supported methods and unsupported methods. |
| Unsupported-edge note | Autoload rows using `eval` remain late-priority unless explicitly opened as M3. |

### P6: Reflection Metadata Repair Tranche

| Field | Value |
| --- | --- |
| Readiness class | `M0-replay` first |
| Owner module | Reflection metadata and class/function/method/property descriptors |
| Evidence source | `110` reflection regressions, all absent from candidate artifacts; source range includes reflection parameter, enum, method, and constant metadata changes. |
| Precheck rows | `php-src/ext/reflection/tests/ReflectionClass_getConstants_basic.phpt`; `php-src/ext/reflection/tests/ReflectionClass_getProperties_001.phpt`; `php-src/ext/reflection/tests/ReflectionParameter_001.phpt`; `php-src/ext/reflection/tests/ReflectionMethod_constructor_basic.phpt`; `php-src/ext/reflection/tests/ReflectionProperty_getModifiers.001.phpt`. |
| Postcheck rows | Same rows plus one internal-parameter default row if the fix touches internal function metadata. |
| Focused Rust tests | Add/update `compiler/tests/reflection_metadata.rs` for the exact descriptor changed. |
| CLI exercise path | `phpc run` snippets that instantiate the reflection API and print the specific metadata, plus focused PHPT replay. |
| Docs/progress | Update reflection support notes and unsupported reflection APIs after proof. |
| Unsupported-edge note | `php-src/ext/reflection/tests/bug64936.phpt` uses `eval` and should remain late-priority. |

### P7: Secondary Extension Replay/Repair Tranche

| Field | Value |
| --- | --- |
| Readiness class | Mixed: `M0-replay` for absent rows, `M0-direct` for direct failures |
| Owner module | Extension-specific builtins and metadata (`uri`, `posix`, `tokenizer`, `xmlreader`, `session`, `random`) |
| Evidence source | Secondary extension shard: `103` rows, with `94` absent, `6` failed, and `3` borked. |
| Precheck rows | Absent sample: `php-src/ext/uri/tests/rfc3986/parsing/basic_success_all.phpt`; `php-src/ext/posix/tests/posix_uname_basic.phpt`; `php-src/ext/tokenizer/tests/token_get_all_variation19.phpt`; `php-src/ext/session/tests/session_cache_limiter_basic.phpt`; `php-src/ext/random/tests/01_functions/rand_basic.phpt`. Direct sample: `php-src/ext/xmlreader/tests/014.phpt`. |
| Postcheck rows | Same rows, split into separate lanes after replay classification. |
| Focused Rust tests | Extension-specific tests only after semantic failure is proven. |
| CLI exercise path | One minimal `phpc run` script per changed extension function plus focused PHPT replay. |
| Docs/progress | Document exact extension subset and skipped/unsupported body semantics. |
| Unsupported-edge note | Tokenizer row `token_get_all_variation19.phpt` uses `eval`; use it for control-plane/late-tag context, not first implementation. |

## Completion Checklist

Before a future implementation lane can be marked complete:

- Implementation code exists and is scoped to the owner module.
- Focused Rust tests pass for the changed behavior.
- A CLI path exercises the behavior through `phpc run` or proves `compile`
  rejection for unsupported lowering.
- Focused PHPT precheck rows become postcheck evidence, not just anecdotal
  examples.
- Docs and `docs/PROGRESS.md` are updated for behavior changes.
- Unsupported edges are explicitly named.
- No public score movement is claimed until a pinned full PHPT gate has zero
  unadjudicated PASS regressions.

## Commands Run

```sh
python3 - <<'PY'
from pathlib import Path
from collections import Counter

root = Path('/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377')
rows = [line.strip() for line in (root / 'regressions-from-latest-published-passes.txt').read_text().splitlines() if line.strip()]
status = {}
for line in (root / 'current-status.normalized.tsv').read_text().splitlines():
    state, row = line.split('\t', 1)
    status[row] = state
print(len(rows), Counter(status.get(row, 'ABSENT') for row in rows))
PY
```

```sh
sed -n '1,260p' /home/claude/php-to-native-compiler/PLAN.md
sed -n '1,220p' .harness/reports/221205Z-source-diff-risk.md
sed -n '1,220p' .harness/reports/standard-array-replay-selector.md
sed -n '1,180p' .harness/reports/221205Z-standard-strings-replace-replay.md
sed -n '1,180p' .harness/reports/221205Z-secondary-ext.md
```
