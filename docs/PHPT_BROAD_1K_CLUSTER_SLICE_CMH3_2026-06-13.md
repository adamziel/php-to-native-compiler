# PHPT Broad 1k Cluster Slice: 2026-06-13 cmh3

Issue: `ptn-cmh3`

This slice used the broad PHPT baseline tooling against `origin/master` on
commit `1880d7e49eaf`. The php-src corpus was
`/home/claude/php-src-phpt` at revision
`8c63ec400ce8e07c57a8d9499317b96a8beafb8b`.

The selected broad manifest was:

```text
.runtime/phpt-baseline/20260613T233641Z/phpt-baseline-1000.txt
```

## Baseline Command

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only
```

The classify-only run selected 1,000 PHPT rows:

| Bucket | Selected | Runnable |
| --- | ---: | ---: |
| `Zend/tests` | 530 | 152 |
| `ext/standard/tests` | 384 | 275 |
| `tests` | 86 | 16 |
| Total | 1,000 | 443 |

Classifier exclusions:

| Classification | Rows |
| --- | ---: |
| `unsupported-language` | 351 |
| `unsupported-class-metadata` | 84 |
| `unsupported-request-input-ini` | 28 |
| `unsupported-extension` | 20 |
| `unsupported-assertion-ini` | 17 |
| `unsupported-resource-limit-ini` | 15 |
| `sapi-behavior` | 13 |
| `harness-skipif` | 6 |
| `unsupported-diagnostics-ini` | 5 |
| `harness-cleanup` | 4 |
| `process-boundary` | 3 |
| `unsupported-scalar-format-ini` | 2 |
| `unsupported-opcache-ini` | 2 |
| `unsupported-host-path-ini` | 2 |
| `unsupported-function-disable-ini` | 2 |
| `unsupported-internal` | 1 |
| `external-service` | 1 |
| `environment-assumption` | 1 |

## Executable Run Status

The executable 1k run was started with:

```sh
tools/run-phpt-baseline.sh --tier 1000
```

It generated the same 1,000-row shape under
`.runtime/phpt-baseline/20260613T234401Z/` and classified the same
443 runnable rows. Under the current shared worker load, the `Zend/tests`
bucket was still compiling row 6 after several minutes, so the run was stopped
before producing a full pass/fail summary. Partial evidence from
`.runtime/phpt-progress/run-20260613T234401Z-zend.log`:

| Row | Result |
| --- | --- |
| `Zend/tests/67468.phpt` | PASS |
| `Zend/tests/ErrorException_construct.phpt` | FAIL |
| `Zend/tests/ErrorException_getSeverity.phpt` | FAIL |
| `Zend/tests/access_modifiers/access_modifiers_006.phpt` | FAIL |
| `Zend/tests/add_001.phpt` | PASS |

Because the executable 1k run was not credible to finish in this environment,
this slice records a blocker map rather than claiming newly passing rows.

## Runnable Array Frontier

The 275 runnable `ext/standard/tests/array` rows are still the clearest broad
1k semantic frontier. Grouped by basename, they split into these focused
families:

| Rows | Family |
| ---: | --- |
| 31 | `array_chunk` |
| 17 | `array_map` |
| 9 | `array_slice` |
| 9 | `array_filter` |
| 9 | `array_diff_assoc` |
| 9 | `array_diff` |
| 8 | `array_sum` |
| 8 | `array_merge` |
| 7 | `array_merge_recursive` |
| 7 | `array_key_exists` |
| 7 | `array_diff_uassoc` |
| 6 | `array_shift` |
| 6 | `array_rand` |
| 6 | `array_keys` |
| 6 | `array_intersect_ukey` |
| 6 | `array_change_key_case` |
| 5 | `array_search` |
| 5 | `array_product` |
| 5 | `array_intersect_uassoc` |
| 5 | `array_intersect_key` |
| 5 | `array_diff_ukey` |
| 5 | `array_diff_key` |
| 4 | `array_splice` |
| 4 | `array_push` |
| 4 | `array_pad` |
| 4 | `array_flip` |
| 4 | `array_all` / `array_any` / `array_find` / `array_find_key` |
| 3 | `array_reverse` |
| 3 | `array_reduce` |
| 3 | `array_intersect_assoc` |
| 3 | `array_intersect` |
| 3 | `array_fill` |
| 3 | `array_combine` |

The remaining runnable array rows are mostly one- or two-row edge cases around
suffix-specific rows such as `array_chunk2`, `array_map_error`,
`array_map_object2`, `array_pop`, `array_key_first`, `array_key_last`,
`array_fill_keys`, `array_count_values`, replacement helpers, aggregate
overflow, and error diagnostics.

## Blocker Split

The next credible implementation slices are:

1. Set/diff/intersect helper parity: roughly 70 runnable rows across
   `array_diff*`, `array_intersect*`, `array_udiff*`, and `array_uintersect*`.
   This needs shared value comparison, key comparison, callback comparison,
   and diagnostic semantics rather than per-row output fixes.
2. Callback-driven array helpers: roughly 33 runnable rows across `array_map*`,
   `array_filter*`, `array_reduce*`, and the newer
   `array_all`/`array_any`/`array_find`/`array_find_key` helpers. This needs
   generic callable validation, internal callback invocation, arity handling,
   key/value argument modes, and callback result propagation.
3. Reindexing and ordered-array copy helpers: roughly 31 runnable rows across
   `array_slice*`, `array_merge*`, `array_merge_recursive*`,
   `array_reverse*`, and `array_pad*`. This needs consistent integer-key
   preservation, next-key overflow behavior, recursive merge semantics, and
   COW/reference-safe copying.
4. Key/value lookup helpers: roughly 28 runnable rows across
   `array_key_exists*`, `array_keys*`, `array_search*`, `array_flip*`,
   `array_count_values*`, and `array_combine*`. This needs shared array-key
   coercion, binary-safe keys, strict/loose comparison parity, and warning
   emission for invalid value/key operands.
5. Array mutators and COW/reference visibility: roughly 18 runnable rows across
   `array_shift*`, `array_push*`, `array_pop*`, `array_splice*`, and
   replacement helpers. This needs reference-preserving mutation, temporary
   by-reference diagnostics, reindexing, and destructor-reentrant safety.

The `array_chunk` basename group is the largest single runnable family at
31 rows, with one additional suffix-specific `array_chunk2` row. It is not a
good implementation target for this slice: the bounded manifest already
contains the broad `array_chunk*` rows and the current status dashboard
records the bounded manifest as 485/485 passing. A focused run can still be
used as regression evidence, but it is unlikely to move the broad frontier.

## Recommendation

Use the callback-driven array helper slice next if the goal is a compact
implementation change that can move multiple broad rows while staying generic.
It is smaller than the set/diff/intersect frontier, but it has clearer helper
boundaries and can share callable validation with existing `call_user_func*()`
and `array_walk*()` behavior.
