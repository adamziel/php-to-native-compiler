# Broad PHPT 1k Array Set And Float Slice: 2026-06-14

Issue: `ptn-odac`

This slice started from the broad 1k PHPT baseline and measured the largest
remaining runnable array-helper families. During integration it rebased over
`ptn-igxz`, which had already landed the generic semantics implicated by this
slice: leading-dot float tokenization, catchable set-operation TypeErrors,
single-source set-operation forms, callback prevalidation, and sorted registry
metadata. This report therefore records the integrated frontier instead of
claiming duplicate implementation ownership.

## Broad 1k Classifier Evidence

The initial `ptn-odac` classifier was run from source state `f11be7e50732`
against php-src PHPT corpus revision
`8c63ec400ce8e07c57a8d9499317b96a8beafb8b`.

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only --out-dir .runtime/ptn-odac-before-classify
```

Result:

| Measurement | Selected | Runnable | Excluded |
| --- | ---: | ---: | ---: |
| initial `ptn-odac` broad 1k classify-only | 1000 | 421 | 579 |
| current integrated broad 1k classify-only from `ptn-igxz` | 1000 | 443 | 557 |

Largest runnable families from the initial `ptn-odac` manifest:

| Family | Rows |
| --- | ---: |
| `ext/standard/tests/array/array_diff*` | 39 |
| `ext/standard/tests/array/array_chunk*` | 32 |
| `ext/standard/tests/array/array_intersect*` | 22 |
| `Zend/tests/asymmetric_visibility/*` | 22 |
| `ext/standard/tests/array/array_map*` | 19 |
| `ext/standard/tests/array/array_merge*` | 15 |

## Focused Evidence

| Focus | Command | Integrated result |
| --- | --- | ---: |
| Leading-dot float broad rows | `tools/run-bounded-phpt.sh .runtime/ptn-odac/leading-dot-float-runnable.txt` | 5/5 |
| `array_chunk*` broad rows | `tools/run-bounded-phpt.sh .runtime/ptn-odac/array-chunk-runnable.txt` | 32/32 |
| `array_diff*`/`array_intersect*` broad rows | `tools/run-bounded-phpt.sh .runtime/ptn-odac/array-diff-intersect-runnable.txt` | 58/61 |

Latest integrated logs:

```text
.runtime/phpt-progress/run-20260614T030203Z-manifest.log
.runtime/phpt-progress/run-20260614T030309Z-manifest.log
.runtime/phpt-progress/run-20260614T030801Z-manifest.log
```

The leading-dot float manifest has 5 runnable rows:

```text
ext/standard/tests/array/array_diff_variation3.phpt
ext/standard/tests/array/array_diff_variation4.phpt
ext/standard/tests/array/array_search_variation1.phpt
ext/standard/tests/array/array_search_variation2.phpt
ext/standard/tests/array/array_sum_variation3.phpt
```

Current lexer coverage accepts those PHP numeric forms while preserving token
precedence for `...`, `.=`, and bare `.`.

## Current Array Set-Operation Blocker Map

On the integrated branch, the 61-row broad `array_diff*` and
`array_intersect*` focused family has 58 passing rows and 3 failures.

| Blocker | Rows | Representative rows | Generic gap |
| --- | ---: | --- | --- |
| Array-to-string warning parity in value comparison | 3 | `array_diff_variation9.phpt`, `array_intersect_assoc_variation9.phpt`, `array_intersect_variation9.phpt` | The shared value-string comparison path compares nested array values as `"Array"` but still misses PHP's `Array to string conversion` warning emission with source-location/runtime diagnostic context. |

The integrated passing rows show that key-only set operations, binary-safe
string comparisons, ordinary value comparisons, references, one-source
set-operation forms, catchable array-argument TypeErrors, callback validation,
and user key-comparator paths are stable in this broad slice. The remaining
failures need a shared diagnostic fix in the value-to-string comparison path,
not row-specific expected-output handling.

## Verification

```sh
cargo fmt --check
cargo test lexer_accepts_leading_dot_float_literals
tools/run-phpt-baseline.sh --tier 1000 --classify-only --out-dir .runtime/ptn-odac-before-classify
tools/run-bounded-phpt.sh .runtime/ptn-odac/leading-dot-float-runnable.txt
tools/run-bounded-phpt.sh .runtime/ptn-odac/array-chunk-runnable.txt
tools/run-bounded-phpt.sh .runtime/ptn-odac/array-diff-intersect-runnable.txt
```
