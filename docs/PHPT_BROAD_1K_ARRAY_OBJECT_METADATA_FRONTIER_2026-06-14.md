# PHPT Broad 1k Array Object/Metadata Frontier: 2026-06-14

Issue: `ptn-h8f7`

This slice starts from the broad 1k PHPT baseline and narrows a standard-array
cluster whose failures are not array algorithm bugs. These rows exercise array
helpers with objects, non-public properties or methods, magic accessors, magic
conversion, or reflection-visible metadata. PTN's current runtime has a bounded
public class/property model, so treating these rows as ordinary array-helper
failures would hide the class/object metadata dependency.

This is a blocker map, not a support claim. Reopening these rows requires a
generic object metadata implementation shared by array helpers, callback
dispatch, property access, comparison/stringification, and reflection.

## Broad 1k Evidence

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only --out-dir .runtime/ptn-h8f7-baseline-final
```

Generated manifest:
`.runtime/ptn-h8f7-baseline-final/20260614T031319Z/phpt-baseline-1000.txt`

Artifacts:

```text
.runtime/phpt-progress/classification-20260614T031319Z.tsv
.runtime/phpt-progress/runnable-20260614T031319Z.txt
.runtime/phpt-progress/summary-20260614T031319Z.txt
```

PTN state: post-rebase `ptn-h8f7` branch.

Corpus revision: `8c63ec400ce8e07c57a8d9499317b96a8beafb8b`

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 1000 | 429 | 571 |

Top broad classifier buckets:

| Bucket | Rows |
| --- | ---: |
| `unsupported-language` | 281 |
| `unsupported-class-metadata` | 144 |
| `unsupported-request-input-ini` | 28 |
| `unsupported-extension` | 20 |
| `unsupported-diagnostics-runtime` | 18 |
| `unsupported-assertion-ini` | 17 |
| `unsupported-resource-limit-ini` | 15 |
| `sapi-behavior` | 13 |

## Focused Frontier

Committed manifest:
`tools/phpt-array-object-metadata-frontier-manifest.txt`

Selection:

```sh
awk -F'\t' '$1 ~ /^ext\/standard\/tests\/array\// && $2 == "unsupported-class-metadata" {print $1}' \
  .runtime/phpt-progress/classification-20260614T031319Z.tsv
```

Focused classifier result:

```sh
tools/run-bounded-phpt.sh --classify-only tools/phpt-array-object-metadata-frontier-manifest.txt
```

Result at `.runtime/phpt-progress/summary-20260614T032009Z.txt`:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 70 | 0 | 70 |

Reason split:

| Blocker | Rows |
| --- | ---: |
| Unsupported magic method dispatch/reflection metadata | 60 |
| Non-public property visibility metadata | 9 |
| Non-public method visibility dispatch and diagnostics | 1 |

## Array Helper Concentration

The 70 rows span value/key comparison helpers, callback set operations, merge
and reverse helpers, and object-column extraction:

| Helper family | Rows |
| --- | ---: |
| `array_intersect*()` | 17 |
| `array_udiff*()` / `array_uintersect*()` | 21 |
| `array_diff*()` | 7 |
| `array_merge*()` / `array_reverse()` | 10 |
| `array_column*()` | 4 |
| `array_map*()` | 4 |
| Other array helpers | 7 |

Representative rows:

```text
ext/standard/tests/array/array_column_property_visibility.phpt
ext/standard/tests/array/array_column_variant_objects.phpt
ext/standard/tests/array/array_diff_key_variation1.phpt
ext/standard/tests/array/array_intersect_assoc_variation1.phpt
ext/standard/tests/array/array_map_object3.phpt
ext/standard/tests/array/array_merge_recursive_variation1.phpt
ext/standard/tests/array/array_udiff_assoc_basic.phpt
ext/standard/tests/array/array_uintersect_uassoc_basic.phpt
```

## Why This Is A Blocker

The shared dependency is object metadata, not the individual array helpers.
Generic support needs:

- magic method dispatch for `__get`, `__isset`, `__toString`, `__debugInfo`,
  and overload-sensitive array helper access;
- non-public property and method visibility checks with PHP diagnostic parity;
- typed, readonly, asymmetric, and uninitialized property metadata visible to
  helper reads, writes, comparisons, and callbacks;
- reflection metadata for methods/properties and object conversion;
- comparator/stringification behavior that routes through the same object
  hooks as ordinary PHP expressions.

Until those semantics exist, these rows are better tracked as a 70-row object
metadata frontier than as scattered array-helper failures.

## Verification

```sh
cargo fmt --check
tools/run-phpt-baseline.sh --tier 1000 --classify-only --out-dir .runtime/ptn-h8f7-baseline-before
tools/run-bounded-phpt.sh --classify-only tools/phpt-array-object-metadata-frontier-manifest.txt
```
