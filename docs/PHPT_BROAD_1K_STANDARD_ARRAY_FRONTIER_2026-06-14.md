# PHPT Broad 1k Standard Array Frontier: 2026-06-14

Issue: `ptn-51ey`

This slice uses the broad PHPT baseline tooling on the 1k tier and maps the
remaining `ext/standard/tests/array` frontier. It is a blocker map rather than
an implementation patch because the current runnable rows span several generic
runtime surfaces: ordered-array mutation, callback dispatch, loose comparison,
numeric aggregation, recursive merge/reference behavior, random selection, and
diagnostic precedence. No single credible helper patch explains 25 rows without
crossing those boundaries.

## Broad 1k Baseline

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only --out-dir .runtime/ptn-51ey-after-heredoc-rebase
```

Generated manifest after rebasing across the heredoc classifier frontier:
`.runtime/ptn-51ey-after-heredoc-rebase/20260614T023816Z/phpt-baseline-1000.txt`

Corpus revision: `8c63ec400ce8e07c57a8d9499317b96a8beafb8b`

Classifier artifact:
`.runtime/phpt-progress/summary-20260614T023816Z.txt`

The broad run selected 1,000 rows, kept 430 runnable, and excluded 570:

| Classification | Rows |
| --- | ---: |
| `runnable` | 430 |
| `unsupported-language` | 281 |
| `unsupported-class-metadata` | 144 |
| `unsupported-request-input-ini` | 28 |
| `unsupported-extension` | 20 |
| `unsupported-diagnostics-runtime` | 18 |
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

Runnable rows by broad bucket:

| Bucket | Runnable |
| --- | ---: |
| `ext/standard/tests` | 297 |
| `Zend/tests` | 117 |
| `tests/basic` | 16 |

## Focused Standard-Array Manifest

Focused manifest:
`.runtime/ptn-51ey-standard-array-runnable-after-heredoc-rebase.txt`

Generated from the broad runnable manifest with:

```sh
awk '/^ext\/standard\/tests\/array\// {print}' \
  .runtime/phpt-progress/runnable-20260614T023816Z.txt \
  > .runtime/ptn-51ey-standard-array-runnable-after-heredoc-rebase.txt
tools/run-bounded-phpt.sh --classify-only .runtime/ptn-51ey-standard-array-runnable-after-heredoc-rebase.txt
```

Focused classifier artifact:
`.runtime/phpt-progress/summary-20260614T024340Z.txt`

The focused standard-array run selected 297 rows, kept all 297 runnable, and
excluded 0. These rows are therefore real semantic/runtime frontier, not
remaining harness or syntax classification misses.

## Family Counts

| Family | Runnable rows | Main blocker shape |
| --- | ---: | --- |
| Diff/intersect set helpers | 76 | Loose comparison/stringification warning cadence, user-comparator ordering and duplicates, callback diagnostics, heredoc-reopened value rows, and nested-array comparison behavior. |
| `array_chunk()` | 32 | Already green in the focused `ptn-igxz` slice; retained in broad runnable rows as covered evidence, not a new blocker. |
| Key/existence helpers | 21 | Key coercion, quiet/object/resource diagnostics, `GLOBALS` handling, and first/last helper edge cases. |
| Merge/replace recursive helpers | 20 | Recursive collision rules, numeric-key reindexing/replacement, heredoc-reopened value rows, reference/COW preservation, and nested array alias visibility. |
| `array_sum()` / `array_product()` | 19 | Numeric conversion, leading-numeric warnings, overflow, reference visibility, and integration parity. |
| `array_map()` | 19 | Null callback zipping, callable/object diagnostics, callback result/reference handling, binary-safe strings, and uneven-array argument rows. |
| Stack/queue/cursor mutators | 17 | By-reference mutation visibility, cursor behavior, reindexing, and argument diagnostics. |
| `array_filter()` | 11 | Mode validation, key/value callback argument shapes, callback truthiness, and diagnostic order. |
| `array_slice()` | 10 | Offset/length coercion, key preservation, and reference/COW visibility. |
| `array_fill()` / `array_fill_keys()` | 9 | Size guards, key conversion, and value/reference propagation. |
| `array_rand()` | 7 | Random key selection semantics, argument bounds, and deterministic test-shape diagnostics. |
| `array_change_key_case()` | 7 | Key conversion, collision ordering, and binary-safe key handling. |
| `array_search()` | 6 | Loose/strict comparison edge cases and scalar/object coercion parity. |
| `array_pad()` | 6 | Signed size handling, allocation guard diagnostics, and key/index behavior. |
| `array_reduce()` | 5 | Accumulator initial value, refcount/reference behavior, and by-reference callback diagnostics. |
| Other array helpers | 5 | Mixed legacy rows that should be split only after higher-count families. |
| `array_flip()` | 5 | Invalid value warnings and key conversion. |
| `array_combine()` | 5 | Length validation, key conversion, and catchable diagnostics. |
| Predicate/find helpers | 4 | Callback invocation and early-exit semantics. |
| `array_splice()` | 4 | In-place mutation, replacement coercion, and destructor/reentrancy boundaries. |
| `array_column()` | 4 | Object/array row lookup, index-key conversion, and numeric-string key behavior. |
| `array_reverse()` | 3 | Key preservation and nested/reference visibility. |
| `array_count_values()` | 2 | Integer/string value counting and invalid value warnings. |

## Recommended Splits

1. Treat the 76 set-operation rows as multiple runtime primitives, not one
   helper patch. The next generic step is a shared comparison stringification
   path that emits PHP-compatible `Array to string conversion` warnings for
   nested arrays; prior focused evidence leaves three diff/intersect rows
   blocked there.
2. Split `array_map()` and `array_filter()` by callback argument shape and null
   callback zipping. Those rows share the callback runtime, but the remaining
   failures do not share one diagnostic or one result-shaping rule.
3. Split merge/replace recursive rows around ordered-map COW/reference
   preservation and numeric-key behavior. These helpers are implemented, so the
   next gains should come from alias/visibility semantics rather than registry
   plumbing.
4. Keep `array_chunk()` out of the next implementation target unless it
   regresses; the existing focused evidence records 32/32 passing.

## Verification

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only --out-dir .runtime/ptn-51ey-after-heredoc-rebase
tools/run-bounded-phpt.sh --classify-only .runtime/ptn-51ey-standard-array-runnable-after-heredoc-rebase.txt
```

This branch records a blocker map only; it intentionally does not claim newly
passing rows or newly classified unsupported rows.
