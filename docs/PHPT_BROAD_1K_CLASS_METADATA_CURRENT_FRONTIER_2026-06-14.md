# PHPT Broad 1k Current Class Metadata Frontier: 2026-06-14

Issue: `ptn-tnmt`

This slice refreshes the current broad 1k `unsupported-class-metadata`
frontier on `origin/master` and records a committed blocker map. It is not a
support claim and does not add runtime behavior: the rows share class/object
metadata dependencies that need generic compiler and runtime work before they
can become useful PHPT execution signal.

## Broad 1k Evidence

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only --out-dir .runtime/ptn-tnmt-baseline-rebased
```

Generated manifest:
`.runtime/ptn-tnmt-baseline-rebased/20260614T072055Z/phpt-baseline-1000.txt`

Artifacts:

```text
.runtime/phpt-progress/classification-20260614T072055Z.tsv
.runtime/phpt-progress/runnable-20260614T072055Z.txt
.runtime/phpt-progress/excluded-20260614T072055Z.tsv
```

PTN measurement commit: `75c9165c966b`

Corpus revision: `8c63ec400ce8e07c57a8d9499317b96a8beafb8b`

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 1000 | 425 | 575 |

Top broad classifier buckets:

| Bucket | Rows |
| --- | ---: |
| `unsupported-language` | 288 |
| `unsupported-class-metadata` | 143 |
| `unsupported-request-input-ini` | 28 |
| `unsupported-extension` | 20 |
| `unsupported-assertion-ini` | 17 |
| `unsupported-diagnostics-runtime` | 16 |
| `unsupported-resource-limit-ini` | 15 |
| `sapi-behavior` | 13 |

## Focused Frontier

Committed manifest:
`tools/phpt-broad-class-metadata-current-frontier-manifest.txt`

Selection command:

```sh
awk -F'\t' '$2=="unsupported-class-metadata" {print $1}' \
  .runtime/phpt-progress/classification-20260614T072055Z.tsv
```

Focused replay:

```sh
tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-broad-class-metadata-current-frontier-manifest.txt
```

Focused artifacts:

```text
.runtime/phpt-progress/classification-20260614T072616Z.tsv
.runtime/phpt-progress/excluded-20260614T072616Z.tsv
.runtime/phpt-progress/excluded-20260614T072616Z/unsupported-class-metadata.txt
```

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 143 | 0 | 143 |

All 143 focused rows remain excluded as `unsupported-class-metadata`.

## Reason Split

| Blocker | Rows |
| --- | ---: |
| Unsupported magic method dispatch/reflection metadata | 69 |
| Non-public property visibility metadata | 19 |
| Typed property metadata | 12 |
| Runtime class autoload symbol-table mutation | 9 |
| Internal attribute/reflection metadata | 8 |
| Non-public method visibility dispatch and diagnostics | 7 |
| Indirect readonly property mutation diagnostics | 7 |
| Abstract class/method contract metadata | 5 |
| Final class/method override metadata | 4 |
| Complete internal arginfo/class registry reflection | 3 |

## Path Concentration

| Path family | Rows |
| --- | ---: |
| `ext/standard/tests/array` | 70 |
| `Zend/tests/asymmetric_visibility` | 16 |
| `Zend/tests/access_modifiers` | 12 |
| `Zend/tests/autoload` | 9 |
| `Zend/tests/attributes` | 8 |
| `Zend/tests/backtrace` | 2 |
| Other top-level `Zend/tests` rows | 26 |

The standard-array half is mostly object/magic metadata entering array helper
semantics:

| Standard-array blocker | Rows |
| --- | ---: |
| Unsupported magic method dispatch/reflection metadata | 60 |
| Non-public property visibility metadata | 9 |
| Non-public method visibility dispatch and diagnostics | 1 |

The non-array Zend half is the generic class model surface:

| Zend blocker | Rows |
| --- | ---: |
| Typed property metadata | 12 |
| Non-public property visibility metadata | 10 |
| Unsupported magic method dispatch/reflection metadata | 9 |
| Runtime class autoload symbol-table mutation | 9 |
| Internal attribute/reflection metadata | 8 |
| Indirect readonly property mutation diagnostics | 7 |
| Non-public method visibility dispatch and diagnostics | 6 |
| Abstract class/method contract metadata | 5 |
| Final class/method override metadata | 4 |
| Complete internal arginfo/class registry reflection | 3 |

Representative rows:

```text
Zend/tests/access_modifiers/access_modifiers_008.phpt
Zend/tests/asymmetric_visibility/static_props.phpt
Zend/tests/attributes/029_reflect_internal_symbols.phpt
Zend/tests/autoload/bug61011.phpt
Zend/tests/backtrace/debug_backtrace_options.phpt
Zend/tests/bug38779.phpt
ext/standard/tests/array/array_column_property_visibility.phpt
ext/standard/tests/array/array_map_object3.phpt
ext/standard/tests/array/array_reverse_variation3.phpt
ext/standard/tests/array/array_uintersect_assoc_basic.phpt
```

## Why This Is A Blocker

The broad cluster is not one missing helper or one parser production. Moving it
credibly requires shared class/object metadata that all compiler paths can use:

- property metadata for typed, readonly, asymmetric, protected, and private
  slots, including inherited private slots and indirect writes;
- method metadata for visibility checks, abstract/final contracts, magic
  dispatch, callback boundaries, and diagnostics;
- reflection metadata for internal symbols, attributes, arginfo, closure
  binding, properties, and methods;
- autoload/runtime symbol-table mutation boundaries that can change class
  availability after compile-time collection;
- array helper object handling that observes visibility, magic conversion,
  property reads, callback dispatch, and reflection consistently.

Treating these rows as runnable today would turn absent generic metadata layers
into noisy runtime/parser failures. Keeping the 143-row current frontier
explicitly mapped preserves the broad 1k signal until the class model grows
those semantics.

## Verification

```sh
cargo fmt --check
cargo test --test phpt_classifier
tools/run-phpt-baseline.sh --tier 1000 --classify-only --out-dir .runtime/ptn-tnmt-baseline-rebased
tools/run-bounded-phpt.sh --classify-only --classify-harness-programs tools/phpt-broad-class-metadata-current-frontier-manifest.txt
```

Results:

- `cargo fmt --check`: passed.
- `cargo test --test phpt_classifier`: passed, 31 tests.
- Broad classify-only: passed, 1000 selected, 425 runnable, 575 excluded.
- Focused classify-only: passed, 143 selected, 0 runnable, 143 excluded.
