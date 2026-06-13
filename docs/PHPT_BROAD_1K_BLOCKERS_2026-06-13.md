# PHPT Broad 1k Blocker Map: 2026-06-13

Issue: `ptn-4tfb`

Broad baseline source:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only
tools/run-phpt-baseline.sh --tier 1000
```

The generated 1k broad manifest was
`.runtime/phpt-baseline/20260613T174202Z/phpt-baseline-1000.txt`, using
php-src revision `8c63ec400ce8e07c57a8d9499317b96a8beafb8b`.

The run was collected on `origin/master` `607a810d2c0e`, after the `ptn-n1q2`
class/object metadata classifier slice. The submitted branch was later rebased
across the `ptn-qsmv.3` bounded-manifest expansion and the `ptn-6fbw` blocker
map; neither changed the compiler/runtime path exercised by this broad run.
The run selected 1,000 broad PHPT rows from `Zend/tests`,
`ext/standard/tests`, and `tests`.

## Summary

The classifier excluded 549 rows and left 451 runnable rows. The runnable
rows passed 265 and failed 186:

| Bucket | Selected | Runnable | Passed | Failed |
| --- | ---: | ---: | ---: | ---: |
| `Zend/tests` | 530 | 139 | 45 | 94 |
| `ext/standard/tests` | 384 | 295 | 212 | 83 |
| `tests` | 86 | 17 | 8 | 9 |
| Total | 1,000 | 451 | 265 | 186 |

Classifier exclusions from the same run:

| Classification | Rows |
| --- | ---: |
| `unsupported-language` | 334 |
| `unsupported-class-metadata` | 100 |
| `unsupported-ini` | 73 |
| `unsupported-extension` | 20 |
| `sapi-behavior` | 13 |
| `harness-cleanup` | 4 |
| `process-boundary` | 3 |
| `environment-assumption` | 1 |
| `external-service` | 1 |

No single remaining failure cluster is a credible small implementation slice
that moves at least 25 broad 1k rows without crossing architectural boundaries.
The largest near-term implementation clusters are just below that threshold and
need shared runtime/compiler work rather than row-specific patches.

## Standard Array Blockers

`ext/standard/tests/array` supplied the clearest runnable cluster: 295 rows
ran, 212 passed, and 83 failed. The 83 failures map as follows:

| Blocker | Rows | Generic gap |
| --- | ---: | --- |
| Array value/key comparison and casting | 23 | Shared zval comparison, key coercion, object/resource/array stringification, and nested array comparison parity across array helpers. |
| Callback dispatch and diagnostic parity | 21 | Callback helpers need unified callable validation, internal-function callback dispatch, catchable callback arity diagnostics, and invalid callback TypeErrors. |
| Missing or partial array internals | 13 | `array_rand()`, `array_splice()`, `array_replace()`, and `array_first()`/`array_last()` edge helpers are absent or incomplete. |
| Ordered-array mutation, reference, and overflow semantics | 10 | Mutating helpers need next-key overflow, temporary-by-reference diagnostics, recursive mutation, and reference-preserving paths. |
| Binary string literal and embedded-NUL handling | 9 | `b"..."`/`b'...'` source strings and binary-safe key/value handling are incomplete in parser/runtime helper paths. |
| Argument validation and diagnostics | 7 | Several helpers still need PHP-style strict-mode coercion, option validation, and argument-count/TypeError wording. |

Representative rows:

```text
ext/standard/tests/array/array_diff_variation3.phpt
ext/standard/tests/array/array_intersect_variation4.phpt
ext/standard/tests/array/array_search_variation1.phpt
ext/standard/tests/array/array_map_variation12.phpt
ext/standard/tests/array/array_filter_variation9.phpt
ext/standard/tests/array/array_rand_basic1.phpt
ext/standard/tests/array/array_splice_basic.phpt
ext/standard/tests/array/array_diff_variation10.phpt
ext/standard/tests/array/array_change_key_case_flag_error.phpt
```

## Zend Blockers

`Zend/tests` ran 139 rows, passed 45, and failed 94. The failures are broader
engine semantics rather than a single low-risk slice:

| Blocker | Rows | Generic gap |
| --- | ---: | --- |
| Class/object/property and dynamic dispatch semantics | 24 | Dynamic static calls, method/property visibility, object assignment/reference behavior, destructor timing, and class-local state need deeper class-table/runtime work. |
| Array/reference lvalue and global-state semantics | 18 | Array append/read lvalues, assignment error precedence, global/static variable aliasing, and reference-preserving array/object paths remain incomplete. |
| Parser, diagnostics, literals, control, and include edges | 17 | Binary integer literals, break diagnostics, AST serialization, locale-independent names, error suppression, nested declarations, comments/inline output, and include error paths are incomplete. |
| Stack trace and user error handler state | 13 | `debug_backtrace()`, `debug_print_backtrace()`, trace argument snapshots, and user error handler fallback behavior are not modeled generically. |
| Internal class, arginfo, and attribute metadata | 13 | Internal classes, reflected attributes, native arginfo, readonly attribute objects, and related diagnostics need common metadata storage. |
| Assertion runtime and assertion diagnostics | 9 | `assert()` still lacks full runtime enable/disable state, namespace diagnostics, AST pretty-printing, and assertion callback exception behavior. |

Representative rows:

```text
Zend/tests/bug26802.phpt
Zend/tests/bug27669.phpt
Zend/tests/bug31525.phpt
Zend/tests/array_append_by_reference.phpt
Zend/tests/assign_ref_error_var_handling.phpt
Zend/tests/binary.phpt
Zend/tests/break_error_004.phpt
Zend/tests/backtrace/debug_backtrace_limit.phpt
Zend/tests/attributes/034_target_values.phpt
Zend/tests/assert/expect_020.phpt
```

## Core/Basic Blockers

`tests` ran 17 rows, passed 8, and failed 9:

| Blocker | Rows | Representative rows |
| --- | ---: | --- |
| Missing internal helpers | 4 | `ini_parse_quantity_*`, `header_register_callback*` |
| Runtime metadata constants/path state | 3 | `bug54514.phpt`, `build_date.phpt`, `encoding.phpt` |
| Filesystem diagnostics | 1 | `bug45986.phpt` |
| Float string/cast parity | 1 | `consistent_float_string_casts.phpt` |

## Next Candidate Slices

1. Array value/key comparison and casting: start with a shared runtime helper
   used by `array_diff`, `array_intersect`, `array_search`, `array_flip`,
   `array_keys`, and `array_count_values`. This is the largest standard-array
   cluster at 23 rows and could exceed 25 if paired with the binary string
   literal path.
2. Callback dispatch and diagnostics: centralize callback validation and
   internal-function callback calls for `array_map`, `array_filter`, and
   `array_u*` helpers. The measured broad 1k cluster is 21 rows.
3. Class/object dynamic dispatch: implement dynamic static calls, method
   visibility metadata, and object/reference assignment paths together. This is
   24 Zend rows in the 1k tier but touches deeper class runtime semantics.

These should be implemented as generic compiler/runtime features and then
verified by rerunning the broad 1k tier plus targeted manifests.
