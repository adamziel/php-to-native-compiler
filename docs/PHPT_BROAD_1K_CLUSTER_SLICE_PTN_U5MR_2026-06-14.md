# PHPT Broad 1k Cluster Slice: 2026-06-14 ptn-u5mr

Issue: `ptn-u5mr`

This slice refreshes the broad PHPT 1k classifier on current `origin/master`
and checks whether a new high-yield broad cluster remains available outside the
focused frontier manifests already committed today. It is a blocker map, not a
runtime implementation claim.

The broad runnable surface is already fully covered by committed focused
manifests. A new generic implementation patch should therefore start from one
of those focused frontiers rather than trying to infer a new 25-row broad
cluster from the wrapper tier.

## Broad 1k Evidence

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-u5mr-baseline
cargo fmt --check
cargo test --test phpt_classifier
```

Generated broad manifest:

```text
.runtime/ptn-u5mr-baseline/20260614T091439Z/phpt-baseline-1000.txt
```

Artifacts:

```text
.runtime/phpt-progress/classification-20260614T091439Z.tsv
.runtime/phpt-progress/runnable-20260614T091439Z.txt
.runtime/phpt-progress/excluded-20260614T091439Z.tsv
.runtime/phpt-progress/summary-20260614T091439Z.txt
```

State:

```text
PTN: bb6b6ec428c0
origin/master: bb6b6ec428c0
php-src PHPT corpus: /home/claude/php-src-phpt
corpus revision: 8c63ec400ce8e07c57a8d9499317b96a8beafb8b
```

Classifier result:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 1,000 | 424 | 576 |

Classifier buckets:

| Classification | Rows |
| --- | ---: |
| `runnable` | 424 |
| `unsupported-attribute-metadata` | 149 |
| `unsupported-language` | 147 |
| `unsupported-class-metadata` | 135 |
| `unsupported-request-input-ini` | 28 |
| `unsupported-extension` | 20 |
| `unsupported-assertion-ini` | 17 |
| `unsupported-diagnostics-runtime` | 17 |
| `unsupported-resource-limit-ini` | 15 |
| `sapi-behavior` | 13 |
| `unsupported-assertion-runtime` | 9 |
| `unsupported-diagnostics-ini` | 5 |
| `harness-cleanup` | 4 |
| `process-boundary` | 3 |
| `skipif-precondition` | 2 |
| `unsupported-function-disable-ini` | 2 |
| `unsupported-host-path-ini` | 2 |
| `unsupported-opcache-ini` | 2 |
| `unsupported-scalar-format-ini` | 2 |
| `environment-assumption` | 1 |
| `external-service` | 1 |
| `unsupported-internal` | 1 |
| `unsupported-resource-limit` | 1 |

## Runnable Family Map

The 424 runnable rows partition by broad semantic family as follows:

| Family | Rows | Existing focused frontier |
| --- | ---: | --- |
| `ext/standard/tests/array/*` | 294 | `tools/phpt-broad-standard-array-frontier-manifest.txt`; `tools/phpt-standard-array-current-ptn-ke94-manifest.txt` |
| Root-level Zend historical bug rows | 35 | `tools/phpt-zend-bug-regression-frontier-manifest.txt` |
| Zend assignment/reference/array rows | 31 | `tools/phpt-zend-assignment-reference-frontier-manifest.txt`; COW manifests |
| `Zend/tests/asymmetric_visibility/*` | 22 | `tools/phpt-asymmetric-visibility-frontier-manifest.txt` |
| `tests/basic/*` | 16 | `tools/phpt-core-basic-operator-frontier-manifest.txt`; runtime-boundary/config maps |
| Other root-level Zend rows | 13 | Zend operator/control, bounded, and scalar manifests |
| `Zend/tests/ast/*` | 4 | `tools/phpt-zend-operator-control-frontier-manifest.txt` |
| Zend break diagnostics | 4 | `tools/phpt-zend-operator-control-frontier-manifest.txt` |
| `Zend/tests/arrow_functions/*` | 3 | Bounded and diagnostics/runtime maps |
| `Zend/tests/assert/*` | 2 | `tools/phpt-assertion-runtime-frontier-manifest.txt` |

The largest standard-array row families inside the broad runnable set are still
array-helper edge surfaces, but their blockers are not a single primitive:

| Standard-array family | Rows |
| --- | ---: |
| `array_chunk_variation*` | 29 |
| `array_map_variation*` | 15 |
| `array_filter_variation*` | 8 |
| `array_slice_variation*` | 8 |
| `array_diff_assoc_variation*` | 7 |
| `array_diff_uassoc_variation*` | 7 |
| `array_diff_variation*` | 7 |
| `array_merge_variation*` | 7 |
| `array_sum_variation*` | 7 |
| `array_intersect_assoc_variation*` | 6 |
| `array_intersect_variation*` | 6 |
| `array_key_exists_variation*` | 6 |
| `array_merge_recursive_variation*` | 6 |
| `array_shift_variation*` | 6 |

`array_chunk()` is already covered by committed green focused evidence, so the
largest apparent named broad family is not a new implementation target.

## Focused Manifest Reconciliation

Command:

```sh
comm -23 \
  <(sort .runtime/phpt-progress/runnable-20260614T091439Z.txt) \
  <(awk 'NF && $1 !~ /^#/' tools/phpt-*-manifest.txt |
    sed 's#^tools/[^:]*:##' | sort -u)
```

Result:

```text
0 broad runnable rows remain outside committed focused manifests.
```

Non-partitioned intersections with committed focused manifests:

| Rows | Manifest |
| ---: | --- |
| 294 | `tools/phpt-broad-standard-array-frontier-manifest.txt` |
| 294 | `tools/phpt-standard-array-current-ptn-ke94-manifest.txt` |
| 127 | `tools/phpt-bounded-manifest.txt` |
| 65 | `tools/phpt-array-callback-validation-manifest.txt` |
| 36 | `tools/phpt-array-key-value-frontier-manifest.txt` |
| 35 | `tools/phpt-zend-bug-regression-frontier-manifest.txt` |
| 34 | `tools/phpt-core-basic-operator-frontier-manifest.txt` |
| 32 | `tools/phpt-array-chunk-broad-1k-manifest.txt` |
| 32 | `tools/phpt-zend-assignment-reference-frontier-manifest.txt` |
| 25 | `tools/phpt-zend-operator-control-frontier-manifest.txt` |
| 22 | `tools/phpt-asymmetric-visibility-frontier-manifest.txt` |
| 21 | `tools/phpt-heredoc-nowdoc-frontier-manifest.txt` |
| 19 | `tools/phpt-cow-manifest.txt` |
| 11 | `tools/phpt-cow-broad-frontier-manifest.txt` |

## Blocker Boundary

There is no uncovered broad runnable cluster where a credible single generic
change can newly move at least 25 rows without selecting an existing focused
frontier first.

The next productive implementation splits are:

1. Standard-array residuals: key/value conversion, callback diagnostics,
   ordered-array mutation/reference behavior, random key selection, and
   recursive merge semantics.
2. Zend root rows: assignment/lvalue semantics, historical engine regression
   rows, operator/control diagnostics, and object/reference dispatch.
3. Diagnostics/assertion rows: stack traces, user handlers, `ErrorException`
   metadata, assertion runtime state, and diagnostics INI modes.
4. Class/object metadata rows: attributes, interfaces/traits, asymmetric
   visibility, magic methods, reflection metadata, and unsupported class-like
   declarations.
5. Runtime-boundary rows: request/SAPI setup, PHPT harness cleanup and
   environment sections, child-process execution, and process-global INI state.

## Verification

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-u5mr-baseline
```
