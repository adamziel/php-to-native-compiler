# PHPT Broad 1k Array Frontier: 2026-06-13

Issue: `ptn-o7kg`

Broad baseline source:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only
```
The current generated 1k broad manifest was
`.runtime/phpt-baseline/20260613T181254Z/phpt-baseline-1000.txt`, using
php-src revision `8c63ec400ce8e07c57a8d9499317b96a8beafb8b`.

This map starts from `origin/master` `c02a591df72c`, after the recent
attribute, heredoc, unsupported syntax, asymmetric visibility, and
class-metadata classifier slices. It records the remaining runnable broad 1k
frontier; the rows are mostly real array/runtime semantics, not obvious
classifier misses.

## Current Broad 1k Map

The classify-only run selected 1,000 broad PHPT rows:

```text
runnable: 430
unsupported-language: 404
unsupported-class-metadata: 51
unsupported-ini: 73
unsupported-extension: 20
harness-cleanup: 4
sapi-behavior: 13
process-boundary: 3
external-service: 1
environment-assumption: 1
```

Runnable rows by top-level bucket:

```text
274 ext/standard/tests/array
139 Zend/tests
17 tests/basic
```

## Array Frontier

The array bucket is the next high-yield semantic cluster. The 274 runnable
`ext/standard/tests/array` rows group as follows:

| Rows | Family |
| ---: | --- |
| 75 | set/diff/intersect helpers, including user-comparator variants |
| 32 | `array_chunk()` |
| 18 | key/existence helpers |
| 18 | `array_map()` |
| 17 | other array helpers and edge rows |
| 16 | `array_merge()` / recursive merge |
| 15 | `array_sum()` / `array_product()` numeric aggregates |
| 13 | stack/queue mutators: `array_push()`, `array_pop()`, `array_shift()` |
| 10 | `array_filter()` |
| 9 | `array_slice()` |
| 7 | `array_fill()` / `array_fill_keys()` |
| 7 | `array_change_key_case()` |
| 6 | `array_search()` |
| 6 | `array_rand()` |
| 5 | `array_reduce()` |
| 5 | `array_pad()` |
| 4 | `array_splice()` |
| 4 | `array_flip()` |
| 4 | `array_combine()` |
| 3 | `array_reverse()` |

This is an implementation frontier rather than a single syntax blocker. The
rows require broader ordered-array parity around callback invocation, mutation
and COW visibility, key normalization, error/exception diagnostics, recursive
merge semantics, overflow handling, and PHP's exact comparison/coercion rules.

Recommended next implementation split:

1. Pick one function family with a clear helper boundary, such as
   `array_chunk()` or `array_map()`, and run a focused manifest for only that
   family.
2. Add generic native/runtime tests for the shared primitive first, especially
   key preservation, warnings, callback arity, and COW/reference visibility.
3. Re-run the focused manifest and then the broad 1k classify-only/run evidence.

## Zend Runnable Frontier

The remaining 139 runnable `Zend/tests` rows are mixed semantic edge cases:

| Rows | Family |
| ---: | --- |
| 52 | historical bug regressions |
| 20 | assignment/reference/object writes |
| 13 | Zend array edge cases |
| 13 | other Zend rows |
| 11 | assertion semantics |
| 9 | backtrace diagnostics |
| 9 | attribute-adjacent internals |
| 8 | operator additions |
| 4 | break diagnostics |

These are less coherent than the array bucket and should be split after the
array frontier or after a focused diagnostics/backtrace slice.

## Core Basic Rows

The remaining 17 runnable core `tests/basic` rows cover startup metadata,
headers, ini quantity parsing, encoding, float string casts, and array
deprecation diagnostics. These are small targeted slices, not the highest-yield
broad cluster.

## Verification

```sh
cargo test --test phpt_classifier
tools/run-phpt-baseline.sh --tier 1000 --classify-only
```
