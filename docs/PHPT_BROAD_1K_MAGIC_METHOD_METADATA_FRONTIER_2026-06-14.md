# PHPT Broad 1k Magic Method Metadata Frontier: 2026-06-14

Issue: `ptn-f62z`

This slice maps the broad 1k `unsupported-class-metadata` rows that are blocked
by magic method dispatch or reflection metadata. It does not change runtime
behavior. The output is a reproducible blocker map for the next generic
object-runtime work.

## Broad Baseline

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only --out-dir .runtime/ptn-f62z-baseline-rebased
```

Artifacts:

```text
.runtime/ptn-f62z-baseline-rebased/20260614T025213Z/phpt-baseline-1000.txt
.runtime/phpt-progress/classification-20260614T025213Z.tsv
.runtime/phpt-progress/runnable-20260614T025213Z.txt
.runtime/phpt-progress/excluded-20260614T025213Z.tsv
```

The run used php-src PHPT corpus revision
`8c63ec400ce8e07c57a8d9499317b96a8beafb8b`.

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 1,000 | 430 | 570 |

Final broad 1k classifier buckets:

| Bucket | Rows |
| --- | ---: |
| `runnable` | 430 |
| `unsupported-language` | 281 |
| `unsupported-class-metadata` | 144 |
| `unsupported-request-input-ini` | 28 |
| `unsupported-extension` | 20 |
| `unsupported-diagnostics-runtime` | 18 |
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

## Focused Frontier

Committed manifest:

```text
tools/phpt-magic-method-metadata-frontier-manifest.txt
```

Selection from the classifier:

```sh
awk -F'\t' '$2=="unsupported-class-metadata" && $3 ~ /magic method/ {print $1}' \
  .runtime/phpt-progress/classification-20260614T025213Z.tsv
```

This selected 69 rows:

| Path family | Rows |
| --- | ---: |
| `ext/standard/tests/array` | 60 |
| `Zend/tests/asymmetric_visibility` | 4 |
| `Zend/tests` | 4 |
| `Zend/tests/backtrace` | 1 |

Focused classify-only verification:

```sh
tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-magic-method-metadata-frontier-manifest.txt
```

Artifact: `.runtime/phpt-progress/classification-20260614T025742Z.tsv`.

| Selected | Runnable | Excluded | Bucket |
| ---: | ---: | ---: | --- |
| 69 | 0 | 69 | `unsupported-class-metadata` |

Row-level method sets:

| Declared magic methods in row | Rows |
| --- | ---: |
| `__toString` | 61 |
| `__call` | 2 |
| `__set` | 1 |
| `__unset` | 1 |
| `__construct`, `__set` | 1 |
| `__construct`, `__unset` | 1 |
| `__construct`, `__get`, `__isset` | 1 |
| `__call`, `__construct`, `__destruct` | 1 |

Declaration occurrence counts across the 69 rows:

| Magic method | Occurrences |
| --- | ---: |
| `__toString` | 63 |
| `__construct` | 4 |
| `__call` | 3 |
| `__set` | 2 |
| `__unset` | 2 |
| `__destruct` | 1 |
| `__get` | 1 |
| `__isset` | 1 |

## Blocker Boundary

PTN already supports public `__construct`, public `__call` fallback for the
currently modeled direct/object-callback dispatch surface, public `__toString`
for current runtime string conversions, and public `__invoke` for supported
callable paths. These broad rows are still not safe to reopen as a syntactic
`__toString` exception because the rows exercise wider object metadata and
dispatch contexts:

- array helpers compare, key, merge, map, and reverse object values while PHP
  consults object-to-string conversion or property access hooks;
- asymmetric-visibility rows require `__set`/`__unset` hook behavior together
  with property visibility metadata;
- `array_column()` object cases require `__get`/`__isset` property access
  semantics over object inputs;
- Zend bug rows cover `__call`, destructor timing, and backtrace/reflection
  effects around magic dispatch.

The next implementation slice should add generic object metadata and dispatch
support instead of special-casing these PHPT names:

1. Store magic method availability, visibility, and staticness in the class
   metadata used by runtime dispatch and reflection.
2. Route object property reads, writes, `isset()`, and `unset()` through
   `__get`, `__set`, `__isset`, and `__unset` with PHP-compatible visibility
   and recursion guards.
3. Reuse the existing `__toString` conversion path from array helpers that
   compare or key values, preserving PHP warning and exception behavior.
4. Model destructor registration/timing before reopening rows that declare
   `__destruct`.
5. Re-run this manifest and remove only the classifier branches whose generic
   semantics have landed.

## Verification

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only --out-dir .runtime/ptn-f62z-baseline-rebased
tools/run-bounded-phpt.sh --classify-only --classify-harness-programs tools/phpt-magic-method-metadata-frontier-manifest.txt
cargo fmt --check
cargo test --test phpt_classifier
```
