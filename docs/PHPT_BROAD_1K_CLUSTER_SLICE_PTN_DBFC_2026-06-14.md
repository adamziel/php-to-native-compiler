# PHPT Broad 1k Cluster Slice: ptn-dbfc

Issue: `ptn-dbfc`

This slice refreshed the broad PHPT 1k classifier on current `origin/master`
and looked for one high-yield semantic cluster that could credibly move at
least 25 rows. The current frontier does not have such a narrow implementation
shape: the rows above that threshold are either already green in focused
evidence or split across multiple generic compiler/runtime primitives.

This is therefore a blocker map with current counts, not a behavior change.

## Broad 1k Evidence

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only
```

Generated broad manifests:

```text
.runtime/ptn-b35n-baseline-post/20260614T090208Z/phpt-baseline-1000.txt
.runtime/ptn-b35n-baseline-post/20260614T090208Z/phpt-baseline-5000.txt
.runtime/ptn-b35n-baseline-post/20260614T090208Z/phpt-baseline-10000.txt
```

Classifier artifacts:

```text
.runtime/phpt-progress/summary-20260614T090208Z.txt
.runtime/phpt-progress/classification-20260614T090208Z.tsv
.runtime/phpt-progress/runnable-20260614T090208Z.txt
.runtime/phpt-progress/excluded-20260614T090208Z.tsv
```

PTN evidence commit: `ee63f3764a8a`

php-src PHPT corpus revision:
`8c63ec400ce8e07c57a8d9499317b96a8beafb8b`

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 1,000 | 424 | 576 |

Classifier split:

| Classification | Rows |
| --- | ---: |
| `runnable` | 424 |
| `unsupported-attribute-metadata` | 149 |
| `unsupported-language` | 147 |
| `unsupported-class-metadata` | 135 |
| `unsupported-request-input-ini` | 28 |
| `unsupported-extension` | 20 |
| `unsupported-diagnostics-runtime` | 17 |
| `unsupported-assertion-ini` | 17 |
| `unsupported-resource-limit-ini` | 15 |
| `sapi-behavior` | 13 |
| `unsupported-assertion-runtime` | 9 |
| `unsupported-diagnostics-ini` | 5 |
| `harness-cleanup` | 4 |
| `process-boundary` | 3 |
| `unsupported-function-disable-ini` | 2 |
| `unsupported-host-path-ini` | 2 |
| `unsupported-opcache-ini` | 2 |
| `unsupported-scalar-format-ini` | 2 |
| `skipif-precondition` | 2 |
| `environment-assumption` | 1 |
| `external-service` | 1 |
| `unsupported-internal` | 1 |
| `unsupported-resource-limit` | 1 |

Runnable rows by source:

| Source | Rows |
| --- | ---: |
| `ext/standard/tests/array` | 294 |
| `Zend/tests` root | 81 |
| `Zend/tests/asymmetric_visibility` | 22 |
| `tests/basic` | 16 |
| `Zend/tests/ast` | 4 |
| `Zend/tests/arrow_functions` | 3 |
| `Zend/tests/assert` | 2 |
| `Zend/tests/access_modifiers` | 1 |
| `Zend/tests/attributes` | 1 |

## Standard Array Shape

The only runnable source above the 25-row target is
`ext/standard/tests/array`, with 294 runnable rows. Its broad family shape is:

| Family | Runnable rows | Current implementation boundary |
| --- | ---: | --- |
| Other array helpers | 84 | Mixed key/value, mutation, column, combine, flip, reduce, replacement, stack/queue, and scalar-helper edges. |
| `array_diff*` / `array_udiff*` | 43 | Mostly green in focused set-operation evidence; residual rows split across nested array warnings, helper includes, and comparator arity. |
| `array_intersect*` / `array_uintersect*` | 33 | Same set-operation frontier as diff helpers. |
| `array_chunk()` | 32 | Already green in focused broad evidence; not an implementation target. |
| `array_merge()` / `array_slice()` | 28 | Recursion, reference visibility, reindexing, and ordered-array mutation semantics. |
| `array_sum()` / `array_product()` | 19 | Current focused docs report these rows green except unrelated residual helpers. |
| `array_map()` | 19 | Callback arity, object-callable diagnostics, references, binary strings, and zip semantics. |
| `array_key*` | 19 | Key coercion, resource/object diagnostics, and precision-loss warning parity. |
| `array_filter()` | 10 | Callback mode and key/value callback argument shape. |
| `array_rand()` | 7 | Bounded helper semantics and validation only; below the broad target alone. |

Existing focused evidence on the current 2026-06-14 frontier shows why this is
not one implementation patch:

| Focused frontier | Current result | Residual blocker shape |
| --- | ---: | --- |
| Standard array residual | 294 runnable, 244 passed, 50 failed | Five subclusters: 15 key/value conversion rows, 9 callback rows, 12 mutation/reference rows, 7 `array_rand()` rows, and 7 comparator rows. |
| Set operations | 119 selected, 76 runnable, 64 passed, 12 failed | Residuals are 5 comparator arity rows, 3 nested-array warning rows, 2 heredoc escape rows, and 2 include-declaration rows. |
| Callback/set-operation frontier | 66 runnable, 49 passed, 17 failed | Split across filter modes, map diagnostics/zip/reference parity, reduce accumulator parity, and comparator ordering. |
| Key/value helpers | 42 selected, 37 runnable, 28 passed, 9 failed | Split across `range()` string endpoints, binary escapes, directory resources, object diagnostics, and float-key deprecations. |
| `array_chunk()` | 32 runnable, 32 passed | Already green. |

A local standard-array run was started from
`tools/phpt-broad-standard-array-frontier-manifest.txt` and reached
`array_chunk_variation22.phpt` with 29 completed passing rows before it was
stopped. That partial run confirmed the already-green `array_chunk()` region
but was intentionally not used as a full pass/fail measurement because the
current committed focused maps already cover the residual split.

## Zend and Core Shape

The non-array runnable frontier is also below the target after splitting by
semantic ownership:

| Frontier | Rows | Current blocker shape |
| --- | ---: | --- |
| `Zend/tests` root | 81 | Historical bug, assignment/reference, operator/control, object lifecycle, quiet diagnostics, and callable diagnostics are independent primitives. |
| Asymmetric visibility | 22 | Constructor promotion, declaration validation, reference/unset guards, typed property initialization, and property hooks. |
| `tests/basic` | 16 | Core filesystem/build metadata, INI helpers, and header callback SAPI state. |
| AST/arrow/assert/access/attribute slices | 11 | Source AST preservation, closure edges, assertion runtime, access modifiers, and attribute metadata are separate features. |

The existing focused maps for Zend bug regressions, asymmetric visibility, and
core/basic operator-control rows each land below the 25-row implementation
threshold once split by generic primitive.

## Implementation Boundary

No single credible generic patch in this refreshed 1k slice reaches the target:

1. The only single named standard-array family over 25 rows,
   `array_chunk()`, is already green.
2. The broader standard-array bucket is high-yield as a regression target, but
   the remaining failures are five different runtime surfaces: conversion,
   callback dispatch, ordered mutation/reference behavior, random selection,
   and comparator matching.
3. Zend and core rows are broad engine semantics; their coherent subclusters
   are below 25 rows and already have current maps.
4. Class/object metadata and unsupported-language exclusions are large, but
   reopening them requires broad class metadata, trait/interface, attribute,
   unpacking, generator, heredoc, and variable-variable work rather than a
   narrow PHPT patch.

## Next Credible Splits

The best follow-up implementation order is:

1. Shared array key/value conversion and nested-value warning parity across
   key helpers, search/flip/count helpers, and value-based set operations.
2. Shared catchable callback invocation diagnostics for internal helpers,
   including `array_map()`, `array_filter()`, `array_reduce()`, and user
   comparator set operations.
3. Ordered-array mutation/reference semantics for merge, replace, push, shift,
   pointer helpers, and recursive helper cases.
4. Parser/control-flow diagnostics for `break` levels and large binary integer
   literals as a separate Zend/core slice.
5. Constructor-promoted asymmetric properties before reopening the larger
   typed-property and property-hook frontier.

## Verification

Completed:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only
```

Stopped after confirming the already-green leading rows and before a full
summary:

```sh
tools/run-bounded-phpt.sh tools/phpt-broad-standard-array-frontier-manifest.txt
```

Required final checks for this docs-only blocker map:

```sh
cargo fmt --check
```
