# PHPT Broad 1k Array Callback Frontier: 2026-06-13

Issue: `ptn-d7t0`

This slice refreshed broad 1k evidence on `origin/master` and then narrowed the
largest runnable array-helper clusters. The goal was to find one generic
implementation slice capable of moving at least 25 broad PHPT rows. Current
evidence shows the nearby rows are real implementation work, but the remaining
failures split across multiple runtime surfaces rather than one small
semantics change.

## Broad 1k Baseline

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only --out-dir .runtime/ptn-d7t0-baseline-before
```

Generated manifest:
`.runtime/ptn-d7t0-baseline-before/20260613T231940Z/phpt-baseline-1000.txt`

Corpus revision: `8c63ec400ce8e07c57a8d9499317b96a8beafb8b`

Classifier result:

| Bucket | Rows |
| --- | ---: |
| Selected | 1,000 |
| Runnable | 436 |
| Excluded | 564 |
| `unsupported-language` | 351 |
| `unsupported-class-metadata` | 91 |
| `unsupported-request-input-ini` | 28 |
| `unsupported-extension` | 20 |
| `unsupported-assertion-ini` | 17 |
| `unsupported-resource-limit-ini` | 15 |
| `sapi-behavior` | 13 |
| Other classifier buckets | 29 |

Runnable broad rows by top-level source:

| Source | Rows |
| --- | ---: |
| `ext/standard/tests/array` | 275 |
| `Zend/tests` | 145 |
| `tests` | 16 |

## Focused Evidence

`array_chunk()` was the largest obvious runnable family, but it is already
covered:

```sh
tools/run-bounded-phpt.sh .runtime/ptn-d7t0-array-chunk-before.txt
```

Result: 32 selected, 32 runnable, 32 passed, 0 failed.

The set-operation and callback-helper clusters carry the remaining broad array
frontier:

| Focused set | Manifest rows | Passed | Failed | Log |
| --- | ---: | ---: | ---: | --- |
| Set operations: `array_(diff|intersect|udiff|uintersect)*` | 68 | 50 | 18 | `.runtime/phpt-progress/run-20260613T233521Z-manifest.log` |
| Callback helpers: `array_(map|filter|reduce|find|any|all)*` | 38 | 20 | 18 | `.runtime/phpt-progress/run-20260613T234918Z-manifest.log` |
| Combined frontier | 106 | 70 | 36 | |

## Failure Buckets

The 36 failing rows are not one narrow helper bug. They group into these
generic blockers:

| Rows | Blocker | Representative rows |
| ---: | --- | --- |
| 4 | Missing predicate/find helpers | `array_all_basic.phpt`, `array_any_basic.phpt`, `array_find_basic.phpt`, `array_find_key_basic.phpt` |
| 1 | Constant/parser surface for array-filter modes | `array_filter_invalid_mode.phpt` currently trips the class-constant diagnostic path for `ARRAY_FILTER_USE_*` usage |
| 12 | Callable validation and dispatch diagnostics | missing/invalid function callbacks, object-method callbacks, failing built-ins, and comparator arity rows across `array_map()`, `array_filter()`, `array_diff_ukey()`, `array_intersect_ukey()`, and `array_u*()` helpers |
| 5 | Callback argument and reference semantics | key/value mode argument shape, anonymous callback behavior, reference preservation, uneven `array_map()` zipping, and `array_reduce()` accumulator behavior |
| 8 | Set-operation value conversion, catchable argument handling, and nested array stringification | `array_diff_1.phpt`, `array_diff_single_array.phpt`, `array_diff_variation3.phpt`, `array_diff_variation4.phpt`, `array_diff_variation9.phpt`, `array_intersect_*variation9.phpt` |
| 6 | User-comparator set-operation matching/order semantics | `array_udiff_*` and `array_uintersect_*` rows where comparator results, duplicate handling, or callback arity still diverge |

Representative diffs:

- `array_all_basic.phpt`: `array_all()` is not registered as an internal.
- `array_map_variation14.phpt`: null callback handling works for the first
  case, then an invalid empty-string callback falls through to a fatal dynamic
  function call instead of a catchable callback diagnostic.
- `array_diff_ukey_variation10.phpt`: missing callback names reach
  `ptn_call_callable()` and fatal, rather than the array helper reporting the
  argument-specific callback error.
- `array_diff_variation9.phpt`: nested arrays require repeated
  `Array to string conversion` warnings while the helper continues comparing.
- `array_filter_invalid_mode.phpt`: mode constants hit the unsupported
  class-constant fetch diagnostic before the helper can raise its `ValueError`.

## Implementation Boundary

Relevant runtime/compiler boundaries:

- `src/backend/runtime/internals_internal_functions.c`
  - `ptn_internal_expect_callback_arg()`
  - `ptn_array_intersect_or_diff()`
  - `ptn_array_custom_set_operation()`
  - `ptn_internal_array_map()`
  - `ptn_internal_array_reduce()`
- `src/backend.rs`
  - generated `ptn_callable_is_valid()`
  - generated `ptn_call_callable()`
  - `internal_call_may_invoke_callable()`
- `src/parser.rs`
  - internal-function reservation/dispatch list
  - constant and class-constant expression handling around `ARRAY_FILTER_USE_*`

The largest coherent near-term implementation is a shared callback argument
validation and dispatch path for internal array helpers. That can retire part
of the 12-row validation bucket and some user-comparator diagnostics, but it
will not by itself move 25 rows because the remaining failures need distinct
work: new helper registration, parser/constant handling, nested array
stringification warnings, `array_map()` null/zipping/reference semantics, and
set-operation comparator matching.

## Recommended Next Slice

Start with the internal callback validation layer:

1. Validate callback operands before helper iteration for every callback-taking
   array helper, including the callback position after variable array
   arguments.
2. Make invalid callback failures catchable where PHP reports
   `TypeError`/`ArgumentCountError`, rather than dispatching to
   `ptn_call_callable()` and fataling.
3. Share the same validation path between `array_map()`, `array_filter()`,
   `array_reduce()`, `array_diff_u*()`, `array_intersect_u*()`,
   `array_udiff*()`, and `array_uintersect*()`.
4. Only then split follow-ups for `array_all()`/`array_any()`/`array_find*()`,
   `ARRAY_FILTER_USE_*` constant parsing, nested array stringification warnings,
   and `array_map()` null/zipping/reference semantics.

Verification used for this blocker map:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only --out-dir .runtime/ptn-d7t0-baseline-before
tools/run-bounded-phpt.sh .runtime/ptn-d7t0-array-chunk-before.txt
tools/run-bounded-phpt.sh .runtime/ptn-d7t0-array-set-before.txt
tools/run-bounded-phpt.sh .runtime/ptn-d7t0-array-callback-before.txt
```
