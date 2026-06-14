# PHPT Broad 1k Array Named/Callback Frontier: 2026-06-14

Issue: `ptn-sdcx`

This slice starts from the broad PHPT 1k baseline and narrows the standard-array
callback frontier. A broad implementation move of 25 rows is not credible as
one patch: the remaining failures cross named internal-call lowering,
catchable callable arity diagnostics, object-method callback diagnostics,
recursive references, and `array_map()` zip/binary-string parity.

## Broad 1k Evidence

Generated manifest:

```text
.runtime/ptn-sdcx-before/20260614T040037Z/phpt-baseline-1000.txt
```

Corpus revision:

```text
8c63ec400ce8e07c57a8d9499317b96a8beafb8b
```

Before:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-sdcx-before
```

Artifacts:

```text
.runtime/phpt-progress/classification-20260614T040037Z.tsv
.runtime/phpt-progress/runnable-20260614T040037Z.txt
.runtime/phpt-progress/summary-20260614T040037Z.txt
```

After, using the same generated manifest:

```sh
tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  .runtime/ptn-sdcx-before/20260614T040037Z/phpt-baseline-1000.txt
```

Artifacts:

```text
.runtime/phpt-progress/classification-20260614T042654Z.tsv
.runtime/phpt-progress/runnable-20260614T042654Z.txt
.runtime/phpt-progress/summary-20260614T042654Z.txt
```

## Classifier Movement

The classifier now identifies named arguments to modeled array internal
functions as an unsupported internal-call lowering surface. This is generic:
user-defined named calls remain runnable, and `Class::method()` expressions
inside array helper calls are not treated as named arguments.

| Run | Selected | Runnable | Excluded | `unsupported-language` | `unsupported-class-metadata` |
| --- | ---: | ---: | ---: | ---: | ---: |
| Before | 1,000 | 429 | 571 | 281 | 144 |
| After | 1,000 | 428 | 572 | 282 | 144 |

Newly classified runnable row:

```text
ext/standard/tests/array/array_filter_invalid_mode.phpt
```

This row previously executed and failed at the backend-level fatal
`named arguments currently support user-defined functions`. It is now an
explicit blocker until internal functions carry enough parameter-name/default
metadata to bind named arguments without changing PHP defaults for skipped
optional parameters.

## Focused Callback Evidence

Focused manifest:

```text
.runtime/ptn-sdcx/array-map-filter-manifest.txt
```

Generated from the broad runnable manifest with:

```sh
awk '/^ext\/standard\/tests\/array\/array_(map|filter)/ {print}' \
  .runtime/phpt-progress/runnable-20260614T040037Z.txt \
  > .runtime/ptn-sdcx/array-map-filter-manifest.txt
tools/run-bounded-phpt.sh .runtime/ptn-sdcx/array-map-filter-manifest.txt
```

Run log:

```text
.runtime/phpt-progress/run-20260614T040553Z-manifest.log
```

| Focused set | Selected | Runnable | Passed | Failed |
| --- | ---: | ---: | ---: | ---: |
| `array_map*` / `array_filter*` | 30 | 30 | 21 | 9 |

Failing rows:

```text
ext/standard/tests/array/array_filter_invalid_mode.phpt
ext/standard/tests/array/array_filter_variation10.phpt
ext/standard/tests/array/array_map_error.phpt
ext/standard/tests/array/array_map_object2.phpt
ext/standard/tests/array/array_map_variation10.phpt
ext/standard/tests/array/array_map_variation12.phpt
ext/standard/tests/array/array_map_variation2.phpt
ext/standard/tests/array/array_map_variation7.phpt
ext/standard/tests/array/array_map_variation9.phpt
```

## Blocker Map

| Rows | Blocker | Representative rows |
| ---: | --- | --- |
| 1 | Named arguments for modeled array internals need internal parameter-name/default metadata and named binding before dispatch. | `array_filter_invalid_mode.phpt` |
| 4 | Internal callback invocation needs catchable `ArgumentCountError` behavior instead of fatal arity exits when callbacks receive too few arguments. | `array_filter_variation10.phpt`, `array_map_error.phpt`, `array_map_variation10.phpt`, `array_map_variation9.phpt` |
| 2 | Callable diagnostics need PHP-specific class-method and built-in arity text. | `array_map_object2.phpt`, `array_map_variation12.phpt` |
| 1 | Recursive array references remain outside current boxed array dump/reference support. | `array_map_variation2.phpt` |
| 1 | `array_map()` zip semantics still need exact null/deprecation and key behavior for uneven arrays. | `array_map_variation7.phpt` |

Next credible implementation splits:

1. Add internal function parameter metadata and named-argument binding with a
   missing-argument/default representation instead of filling skipped optional
   slots with `null`.
2. Make callback arity failures thrown/catchable across
   `array_map()`, `array_filter()`, `array_reduce()`, `array_walk*()`, and
   `call_user_func*()` through shared callable dispatch.
3. Split `array_map()` reference/recursive-array and uneven-array behavior from
   callback validation; those are separate array/reference semantics.

## Verification

```sh
bash -n tools/phpt-classifier.sh
cargo test --test phpt_classifier
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-sdcx-before
tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  .runtime/ptn-sdcx-before/20260614T040037Z/phpt-baseline-1000.txt
tools/run-bounded-phpt.sh .runtime/ptn-sdcx/array-map-filter-manifest.txt
```
