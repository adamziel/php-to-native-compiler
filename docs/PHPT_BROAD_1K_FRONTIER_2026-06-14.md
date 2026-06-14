# PHPT Broad 1k Frontier: 2026-06-14

Issue: `ptn-2juv`

Broad baseline source:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only
tools/run-bounded-phpt.sh --classify-harness-programs \
  .runtime/phpt-baseline/20260613T233737Z/phpt-baseline-1000.txt
```

The generated 1k broad manifest was
`.runtime/phpt-baseline/20260613T235955Z/phpt-baseline-1000.txt`, using
php-src revision `8c63ec400ce8e07c57a8d9499317b96a8beafb8b`.

The rebased classifier run was collected after rebasing across `ptn-feps`. The
classifier artifacts are:

- classify-only classification:
  `.runtime/phpt-progress/classification-20260613T235955Z.tsv`
- runnable manifest:
  `.runtime/phpt-progress/runnable-20260613T235955Z.txt`
- earlier partial full-run log:
  `.runtime/phpt-progress/run-20260613T234455Z-zend.log`

## Classifier Summary

The refreshed broad 1k classifier selected 1,000 rows, left 447 runnable, and
excluded 553 rows:

| Classification | Rows |
| --- | ---: |
| `runnable` | 447 |
| `unsupported-language` | 351 |
| `unsupported-class-metadata` | 84 |
| `unsupported-request-input-ini` | 28 |
| `unsupported-extension` | 20 |
| `unsupported-assertion-ini` | 17 |
| `unsupported-resource-limit-ini` | 15 |
| `sapi-behavior` | 13 |
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

Runnable rows by bucket:

| Bucket | Runnable |
| --- | ---: |
| `ext/standard/tests` | 276 |
| `Zend/tests` | 155 |
| `tests/basic` | 16 |

## Runnable Array Frontier

The current broad standard bucket is dominated by array helpers. The largest
runnable families are:

| Family | Rows | Current blocker shape |
| --- | ---: | --- |
| Diff/intersect helpers | 68 | Multiple shared gaps: PHP string comparison/stringification, callback validation, catchable argument diagnostics, and binary literal/string parity. Prior focused evidence showed 47/62 passing and 15 failing, so one small generic change is unlikely to clear 25 rows alone. |
| `array_chunk()` | 32 | Focused native run now passes 32/32 from the broad 1k manifest; this is covered evidence, not the next implementation target. |
| Other array helpers | 24 | Mixed new and edge helpers including `array_all`, `array_any`, `array_column`, `array_find`, and legacy numeric rows. |
| Sum/product | 19 | Numeric aggregation, overflow, reference, and warning parity. |
| `array_map()` | 19 | Callback invocation and callback diagnostic parity. |
| Key helpers | 18 | `array_key_exists()`, `array_key_first()`, `array_key_last()`, and `array_keys()` edge cases. |
| Stack/queue mutators | 13 | Mutation, reindexing, reference visibility, and overflow parity. |
| `array_filter()` | 10 | Callback mode and truthiness/callback diagnostics. |
| `array_slice()` | 9 | Offset/length/key-preservation edge parity. |
| `array_merge()` | 9 | Reindexing and reference/COW visibility. |
| `array_merge_recursive()` | 7 | Recursive merge semantics and references. |
| `array_change_key_case()` | 7 | Key coercion and binary-safe key handling. |
| `array_rand()` | 6 | Missing/partial random key helper semantics. |
| `array_search()` | 6 | Shared loose/strict comparison edge cases. |
| `array_fill()` / `array_fill_keys()` | 7 | Size guards, key conversion, and warning parity. |

The only single named family above the 25-row target is `array_chunk()`, and a
focused recheck on `ptn-kj3c` confirms the generic helper is already green. The
higher-yield 68-row diff/intersect group crosses several runtime boundaries and
is not credible as a one-step implementation slice without first narrowing
comparison, diagnostic, callback, and binary-string work.

## Focused `array_chunk()` Recheck

This follow-up uses the current broad 1k source shape from:

```text
.runtime/phpt-baseline/20260614T071321Z/phpt-baseline-1000.txt
```

The corpus revision is still:

```text
8c63ec400ce8e07c57a8d9499317b96a8beafb8b
```

Committed focused manifest:

```text
tools/phpt-array-chunk-broad-1k-manifest.txt
```

Run:

```sh
tools/run-bounded-phpt.sh tools/phpt-array-chunk-broad-1k-manifest.txt
```

Result artifact:

```text
.runtime/phpt-progress/summary-20260614T071814Z.txt
```

| Selected | Runnable | Excluded | Passed | Failed | Skipped | Warned |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 32 | 32 | 0 | 32 | 0 | 0 | 0 |

The selected rows cover default and preserved-key chunking, nested arrays,
references, sparse/mixed keys, invalid size diagnostics, and the broad
variation rows. No code change is needed for this family on the current branch.

## Runnable Zend Frontier

The current runnable Zend rows group as:

| Family | Rows | Blocker shape |
| --- | ---: | --- |
| Historical bug regressions | 43 | Mixed engine edge cases, often involving old object/reference/control-flow behavior. |
| Array/reference/object writes | 32 | Array lvalues, object/property writes, global/static state, references, and assignment diagnostics. |
| Asymmetric visibility | 22 | Property visibility diagnostics, reference behavior, unset, and virtual accessors. |
| Other Zend rows | 16 | Mixed operators and engine surface. |
| Assertion semantics | 11 | Assertion runtime state and diagnostics. |
| Backtrace diagnostics | 10 | Backtrace frame/argument/include metadata. |
| Attributes/internal metadata | 9 | Internal class/reflection/attribute metadata. |
| AST serialization | 4 | AST literal and operator serialization. |
| Arrow functions | 4 | Closure/arrow edge semantics. |
| Break diagnostics | 4 | Control-flow diagnostic parity. |

## Full-Run Blocker

An earlier full run on commit `1880d7e49eaf` built `target/debug/phpc` and
started the Zend bucket. It reached:

```text
PASS Zend/tests/67468.phpt
FAIL Zend/tests/ErrorException_construct.phpt
TEST Zend/tests/ErrorException_getSeverity.phpt
```

`Zend/tests/ErrorException_getSeverity.phpt` then left a long-lived local
`phpc` process and the run did not progress to the standard array bucket. The
run was stopped to avoid leaving a background process. No full pass/fail run was
completed after rebasing because the current deliverable is a blocker map. This
makes a broad before/after pass-count run unreliable until either:

1. the ErrorException severity row is fixed or classified, or
2. the broad evidence is split into focused bucket manifests that skip the
   Zend run blocker when measuring array helper work.

## Recommended Next Split

1. For implementation, split diff/intersect into narrower generic primitives:
   value stringification/comparison, catchable argument diagnostics, and
   callback validation. Do not attempt the full 68-row group as one patch.
2. Keep `array_chunk()` in regression evidence, but do not use it as the next
   broad implementation target unless this 32-row focused manifest regresses.
3. Use focused manifests for broad array helper work so Zend diagnostics or
   wrapper-level issues cannot hide standard-bucket pass/fail evidence.
