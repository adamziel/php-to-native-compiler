# Broad PHPT 1k Magic Metadata Blockers: 2026-06-14

Issue: `ptn-knrm`

This slice refreshes broad 1k evidence on current `origin/master` after
`ptn-igxz`, `ptn-1f0f`, and `ptn-4fd3`, then maps the largest remaining
class-metadata blocker: unsupported magic method dispatch and reflection
metadata.

## Evidence

Source state:

- PTN: `0e868d3b731a`
- php-src PHPT corpus: `8c63ec400ce8e07c57a8d9499317b96a8beafb8b`

Commands:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-knrm-baseline
tools/run-bounded-phpt.sh --classify-only \
  tools/phpt-broad-magic-metadata-manifest.txt
```

Generated broad manifest:

```text
.runtime/ptn-knrm-baseline/20260614T023307Z/phpt-baseline-1000.txt
```

Broad 1k classify-only result:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 1000 | 431 | 569 |

Top current exclusion buckets:

| Classification | Rows |
| --- | ---: |
| `unsupported-language` | 281 |
| `unsupported-class-metadata` | 144 |
| `unsupported-request-input-ini` | 28 |
| `unsupported-extension` | 20 |
| `unsupported-diagnostics-runtime` | 17 |
| `unsupported-assertion-ini` | 17 |

## Magic Metadata Cluster

`unsupported-class-metadata` now contains 144 broad rows. The largest reason is:

```text
requires unsupported magic method dispatch/reflection metadata
```

That reason accounts for 69 rows:

| Source area | Rows |
| --- | ---: |
| `ext/standard/tests/array` | 60 |
| `Zend/tests/asymmetric_visibility` | 4 |
| Zend object/backtrace bug rows | 5 |

Focused manifest:

```text
tools/phpt-broad-magic-metadata-manifest.txt
```

The focused classify-only run keeps all 69 rows excluded as
`unsupported-class-metadata`; this is intentional. Running them as executable
coverage would turn missing generic object/magic metadata into noisy array
helper failures.

## Why This Is A Blocker

This cluster crosses the object model and array/internal helper surfaces:

- Objects need generic public magic method metadata, especially `__toString`,
  `__get`, `__set`, `__isset`, `__unset`, and `__debugInfo`, with visibility and
  signature validation.
- Runtime value conversion must route object-to-string and object-to-array
  observations through magic dispatch where PHP does.
- Array helpers such as `array_diff*`, `array_intersect*`, `array_map()`,
  `array_column()`, merge/reverse/push/shift helpers, and callback comparator
  variants need to share that object conversion path instead of handling object
  values ad hoc.
- Reflection and diagnostic paths need to expose magic method metadata in stack
  frames, debug output, property access, and class metadata queries.

Until those semantics exist, the broad baseline should keep these rows in a
precise class-metadata bucket rather than reopening them as unrelated array
frontier failures.

## Representative Rows

```text
Zend/tests/asymmetric_visibility/__set.phpt
Zend/tests/asymmetric_visibility/__unset.phpt
Zend/tests/backtrace/bug39445.phpt
ext/standard/tests/array/array_column_object_cast.phpt
ext/standard/tests/array/array_diff_variation8.phpt
ext/standard/tests/array/array_intersect_variation8.phpt
ext/standard/tests/array/array_map_variation17.phpt
ext/standard/tests/array/array_merge_recursive_variation5.phpt
ext/standard/tests/array/array_udiff_variation2.phpt
ext/standard/tests/array/array_uintersect_variation1.phpt
```

## Next Implementation Split

1. Store declared magic method flags/signatures in class metadata and validate
   public visibility/signature constraints during class registration.
2. Add a shared object string-conversion helper that dispatches public
   `__toString()` and reports PHP-style failures.
3. Route array helper loose comparison, key/value conversion, and callback
   diagnostics through that shared conversion helper.
4. Extend reflection/debug metadata once conversion dispatch is generic.
