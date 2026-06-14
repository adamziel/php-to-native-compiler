# PHPT Broad Array Helper Slice: 2026-06-14 ndkl

Issue: `ptn-ndkl`

This slice used the broad PHPT baseline tooling on `origin/master` and then
focused the standard-array callback/helper frontier.

The php-src corpus was `/home/claude/php-src-phpt` at revision
`8c63ec400ce8e07c57a8d9499317b96a8beafb8b`.

## Broad 1k Classifier

Current implementation commit:

```text
64aef5958e42 feat: support array first last helpers
```

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only
```

Artifacts:

```text
.runtime/phpt-baseline/20260614T011614Z/phpt-baseline-1000.txt
.runtime/phpt-progress/classification-20260614T011615Z.tsv
.runtime/phpt-progress/runnable-20260614T011615Z.txt
```

The broad 1k classifier selected 1,000 rows, left 447 runnable, and excluded
553 rows:

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

## Focused Callback/Helper Evidence

The 38-row callback/predicate manifest was derived from the broad 1k runnable
manifest:

```sh
rg '^ext/standard/tests/array/(array_(map|filter|reduce|all|any|find|find_key))' \
  .runtime/phpt-progress/runnable-20260614T003943Z.txt \
  > .runtime/ptn-ndkl/array-callback-predicate-before.txt
```

Before evidence, collected before rebasing across the predicate/find helper
work:

```text
.runtime/phpt-progress/run-20260614T004459Z-manifest.log
38 selected, 38 runnable, 20 passed, 18 failed
```

Current evidence on `64aef5958e42`:

```text
.runtime/phpt-progress/run-20260614T010711Z-manifest.log
38 selected, 38 runnable, 24 passed, 14 failed
```

The separate helper slice includes the predicate/find rows plus the two
`array_first()`/`array_last()` broad rows added here:

```text
.runtime/ptn-ndkl/new-array-helpers-after.txt
.runtime/phpt-progress/run-20260614T011430Z-manifest.log
6 selected, 6 runnable, 6 passed, 0 failed
```

Rows:

```text
ext/standard/tests/array/array_all_basic.phpt
ext/standard/tests/array/array_any_basic.phpt
ext/standard/tests/array/array_find_basic.phpt
ext/standard/tests/array/array_find_key_basic.phpt
ext/standard/tests/array/array_first_last.phpt
ext/standard/tests/array/array_first_last_errors.phpt
```

## Current 38-Row Residuals

The remaining 14 failures are not one implementation primitive:

| Blocker | Rows | Representative rows |
| --- | ---: | --- |
| Catchable callback arity errors | 5 | `array_map_error.phpt`, `array_map_variation10.phpt`, `array_map_variation9.phpt`, `array_reduce_variation1.phpt`, `array_filter_variation10.phpt` |
| Invalid callback and language-construct diagnostics | 6 | `array_filter_variation9.phpt`, `array_map_object2.phpt`, `array_map_variation12.phpt`, `array_map_variation14.phpt`, `array_map_variation15.phpt`, `array_map_variation16.phpt` |
| Recursive/reference array value parity | 1 | `array_map_variation2.phpt` |
| Null offset deprecation parity for callback return arrays | 1 | `array_map_variation7.phpt` |
| Class-constant diagnostic surface in catch blocks | 1 | `array_filter_invalid_mode.phpt` |

The next credible implementation split is catchable callable invocation errors
for internal callback helpers. It should be shared by `array_map()`,
`array_filter()`, `array_reduce()`, `call_user_func*()`, and callback
validation paths rather than patched row-by-row.
