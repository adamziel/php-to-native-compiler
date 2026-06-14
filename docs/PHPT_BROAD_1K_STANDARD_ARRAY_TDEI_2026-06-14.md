# PHPT Broad 1k Standard Array Slice: 2026-06-14 tdei

Issue: `ptn-tdei`

This slice used the broad PHPT baseline tooling on `origin/master` and focused
the standard-array rows that dominate the current runnable 1k frontier. The
php-src corpus was `/home/claude/php-src-phpt` at revision
`8c63ec400ce8e07c57a8d9499317b96a8beafb8b`.

## Broad 1k Classifier

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only
```

Artifacts:

```text
.runtime/phpt-baseline/20260614T033006Z/phpt-baseline-1000.txt
.runtime/phpt-progress/classification-20260614T033007Z.tsv
.runtime/phpt-progress/runnable-20260614T033007Z.txt
.runtime/phpt-progress/summary-20260614T033007Z.txt
```

The classifier selected 1,000 broad rows, left 430 runnable, and excluded 570:

| Classification | Rows |
| --- | ---: |
| `runnable` | 430 |
| `unsupported-language` | 281 |
| `unsupported-class-metadata` | 144 |
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

## Standard Array Shape

The broad standard-array bucket contains 383 selected rows. The classifier
leaves 296 runnable rows and excludes 87:

| Classification | Rows |
| --- | ---: |
| `runnable` | 296 |
| `unsupported-class-metadata` | 70 |
| `unsupported-language` | 9 |
| `unsupported-extension` | 5 |
| `harness-cleanup` | 1 |
| `unsupported-resource-limit` | 1 |
| `unsupported-resource-limit-ini` | 1 |

Runnable standard-array families:

| Family | Runnable rows | Current status |
| --- | ---: | --- |
| `array_diff*` | 39 | Set-operation frontier; crosses loose value comparison, key comparison, object/resource/string conversion, custom comparator ordering, and diagnostics. |
| `array_chunk` | 32 | Focused run is 32/32 passing; not an implementation target. |
| `array_intersect*` | 30 | Same set-operation frontier as `array_diff*`. |
| `array_map` | 19 | Mixed callback arity, object-callable diagnostics, references, binary strings, and null-zip offset diagnostics. |
| `array_key*` | 19 | Key coercion, lookup, and result-shape edge cases. |
| `array_sum` / `array_product` | 19 | Numeric aggregation, overflow, reference, and warning parity. |
| `array_filter` | 11 | Mode validation and key/value callback arity behavior. |
| `array_slice` / `array_merge*` | 28 | Reindexing, recursive merge, copy/reference visibility, and next-key behavior. |
| Other mutators/helpers | 99 | Smaller groups around fill, pad, shift/push/pop, rand, search, column, flip, combine, reduce, and replacement helpers. |

The largest runnable implementation frontier is the 76-row set-operation group
(`array_diff*`, `array_intersect*`, `array_udiff*`, and `array_uintersect*`).
It is not credible as a single patch because the failures span multiple
generic runtime boundaries rather than one missing helper.

## Focused Evidence

### `array_chunk`

Manifest:

```text
.runtime/phpt-baseline/20260614T033006Z/array-chunk-runnable.txt
```

Run:

```sh
tools/run-bounded-phpt.sh .runtime/phpt-baseline/20260614T033006Z/array-chunk-runnable.txt
```

Result:

```text
.runtime/phpt-progress/run-20260614T034828Z-manifest.log
32 selected, 32 runnable, 32 passed, 0 failed
```

This exceeds the 25-row cluster size, but it is already green on current
`origin/master`, so it is evidence that the largest single named family is not
the next implementation target.

### Callback Helpers

Manifest:

```text
.runtime/phpt-baseline/20260614T033006Z/callback-array-runnable.txt
```

Run:

```sh
tools/run-bounded-phpt.sh .runtime/phpt-baseline/20260614T033006Z/callback-array-runnable.txt
```

Result:

```text
.runtime/phpt-progress/run-20260614T034235Z-manifest.log
39 selected, 39 runnable, 29 passed, 10 failed
```

The ten residual rows split as:

| Blocker | Rows | Representative rows |
| --- | ---: | --- |
| Catchable internal-callback arity errors | 4 | `array_map_error.phpt`, `array_map_variation10.phpt`, `array_map_variation9.phpt`, `array_reduce_variation1.phpt` |
| Key/value callback mode and arity interaction | 1 | `array_filter_variation10.phpt` |
| Object/static callable diagnostics | 1 | `array_map_object2.phpt` |
| Built-in callback diagnostic wording | 1 | `array_map_variation12.phpt` |
| Recursive/reference array mapping parity | 1 | `array_map_variation2.phpt` |
| Null offset deprecation parity for zipping arrays of different sizes | 1 | `array_map_variation7.phpt` |
| Dynamic class-name constant surface in catch output | 1 | `array_filter_invalid_mode.phpt` |

The next credible callback implementation split is catchable user-callback
arity errors when internal helpers invoke user functions or closures. It should
be shared by `array_map()`, `array_filter()`, `array_reduce()`, and
`call_user_func*()` paths instead of being patched row by row.

## Recommendation

Do not spend the next broad slice on `array_chunk`; it is already 32/32. The
highest-yield implementation target remains set-operation parity, but it should
be split into smaller generic primitives:

1. shared loose value/key comparison and stringification for array set helpers;
2. custom comparator ordering and duplicate handling for `array_udiff*()` and
   `array_uintersect*()`;
3. catchable callback arity/diagnostic behavior shared across internal
   callback helpers;
4. binary-safe key/value handling and recursive/reference array dump parity.

If those primitives are not implemented together, the standard-array broad 1k
frontier is better treated as a blocker map than as a single implementation
patch.
