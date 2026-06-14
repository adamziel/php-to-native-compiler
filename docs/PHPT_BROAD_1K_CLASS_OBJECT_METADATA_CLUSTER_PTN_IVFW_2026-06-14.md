# PHPT Broad 1k Class/Object Metadata Cluster: 2026-06-14

Issue: `ptn-ivfw`

This slice refreshes the broad 1k PHPT classifier on the current
compiler/classifier state and maps the class/object metadata cluster. It is a
blocker map, not a support claim. The 362 selected rows span parser syntax,
class-table construction, property/method visibility, magic dispatch, runtime
autoload, attributes, reflection metadata, anonymous classes, traits,
interfaces, and object behavior inside standard-array helpers. A single
implementation patch is not credible as a generic 25-row move without crossing
multiple runtime/compiler contracts.

## Broad 1k Evidence

Source state:

- PTN evidence state: `b4e0f62f98d1` (later rebased across docs-only
  `0d87dbff9`)
- php-src PHPT corpus: `8c63ec400ce8e07c57a8d9499317b96a8beafb8b`

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-ivfw-before
```

Generated broad manifest:

```text
.runtime/ptn-ivfw-before/20260614T075827Z/phpt-baseline-1000.txt
```

Classifier artifacts:

```text
.runtime/phpt-progress/classification-20260614T075828Z.tsv
.runtime/phpt-progress/runnable-20260614T075828Z.txt
.runtime/phpt-progress/excluded-20260614T075828Z.tsv
```

Classifier result:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 1,000 | 425 | 575 |

The broad excluded set is still dominated by class/object metadata surfaces:

| Broad blocker | Rows |
| --- | ---: |
| PHP attributes and reflection metadata | 141 |
| Magic method dispatch/reflection metadata | 69 |
| Trait declarations | 25 |
| Interface declarations | 23 |
| Non-public property visibility metadata | 19 |
| Interface implementation checks | 15 |
| Anonymous class syntax | 15 |
| Typed property metadata | 12 |
| Runtime class autoload symbol-table mutation | 9 |
| Internal attribute/reflection metadata | 8 |
| Non-public method visibility dispatch and diagnostics | 7 |
| Indirect readonly property mutation diagnostics | 7 |
| Abstract class/method contract metadata | 5 |
| Final class/method override metadata | 4 |
| Internal arginfo/class registry reflection | 3 |

## Focused Cluster

Focused manifest:

```text
.runtime/ptn-ivfw/class-object-metadata-cluster.txt
```

Selection from the current broad classification:

```sh
awk -F '\t' '$2=="unsupported-class-metadata" ||
  ($2=="unsupported-language" &&
    ($3 ~ /attribute syntax/ ||
     $3 ~ /trait declarations/ ||
     $3 ~ /interface declarations/ ||
     $3 ~ /interface implementation checks/ ||
     $3 ~ /anonymous class syntax/)) {print $1}' \
  .runtime/phpt-progress/classification-20260614T075828Z.tsv \
  > .runtime/ptn-ivfw/class-object-metadata-cluster.txt
```

Focused command:

```sh
tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  .runtime/ptn-ivfw/class-object-metadata-cluster.txt
```

Focused artifacts:

```text
.runtime/phpt-progress/classification-20260614T080451Z.tsv
.runtime/phpt-progress/excluded-20260614T080451Z.tsv
.runtime/phpt-progress/runnable-20260614T080451Z.txt
```

Focused result:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 362 | 0 | 362 |

Classifier split:

| Category | Rows |
| --- | ---: |
| `unsupported-language` | 219 |
| `unsupported-class-metadata` | 143 |

## Source Shape

The focused cluster is not only Zend parser/class rows. It also blocks
standard-array object cases that should eventually share the same object and
metadata runtime:

| Source family | Rows |
| --- | ---: |
| `Zend/tests/attributes/*` and nested attribute rows | 183 |
| `ext/standard/tests/array/*` object/class rows | 71 |
| `Zend/tests/anon/*` | 20 |
| `Zend/tests/asymmetric_visibility/*` excluded rows | 16 |
| `Zend/tests/access_modifiers/*` | 12 |
| `Zend/tests/autoload/*` | 10 |
| `Zend/tests/ArrayAccess/*` | 10 |
| `Zend/tests/backtrace/*` class/metadata rows | 6 |
| Other Zend class/object regressions | 33 |
| `tests/basic/bug73969.phpt` | 1 |

## Blocker Map

| Rows | Generic blocker | Representative rows |
| ---: | --- | --- |
| 141 | Attribute syntax and metadata need parser support for attribute groups/arguments/targets, validation, repeatability, class constants, reflection APIs, and internal attribute metadata. | `Zend/tests/attributes/001_placement.phpt`, `Zend/tests/attributes/029_reflect_internal_symbols.phpt`, `Zend/tests/attributes/constants/multiple_attributes_grouped.phpt` |
| 69 | Magic method dispatch/reflection metadata needs complete magic hook registration, object dump/debug metadata, visibility interaction, `ArrayAccess` hooks, and reflection-visible method metadata. | `Zend/tests/ArrayAccess/bug30346.phpt`, `Zend/tests/__debugInfo_reference.phpt`, `ext/standard/tests/array/array_map_object3.phpt` |
| 63 | Traits, interfaces, interface implementation checks, and anonymous classes need class-table entries with inheritance/implementation graphs, trait composition, anonymous class naming, and diagnostics. | `Zend/tests/anon/001.phpt`, `Zend/tests/attributes/Attribute/Attribute_on_interface.phpt`, `Zend/tests/attributes/Attribute/Attribute_on_trait.phpt` |
| 45 | Property metadata is incomplete for non-public visibility, typed slots, readonly indirect mutation, asymmetric excluded rows, and object helper visibility checks. | `Zend/tests/asymmetric_visibility/__set.phpt`, `Zend/tests/assign_typed_ref_result.phpt`, `ext/standard/tests/array/array_column_property_visibility.phpt` |
| 23 | Method/class declaration metadata is incomplete for non-public methods, abstract/final contracts, access modifier validation, and internal arginfo/class reflection. | `Zend/tests/access_modifiers/access_modifiers_001.phpt`, `Zend/tests/abstract_method_optional_params.phpt`, `Zend/tests/arginfo_zpp_mismatch.phpt` |
| 9 | Runtime autoload needs symbol-table mutation, loader invocation ordering, failure handling, and class-table invalidation around native compilation boundaries. | `Zend/tests/autoload/bug*.phpt` |
| 12 | Standard-array object rows are blocked by the same class/object metadata surface, not by array helper shape alone: object keys, object values, visibility, magic dispatch, and comparator callback metadata all depend on class semantics. | `ext/standard/tests/array/array_fill_object.phpt`, `ext/standard/tests/array/array_diff_key_variation1.phpt`, `ext/standard/tests/array/array_uintersect_basic.phpt` |

The blocker counts above intentionally overlap by implementation surface rather
than by PHPT path family. The focused classifier count remains the authoritative
row count: 362 selected, 0 runnable, 362 excluded.

## Next Implementation Splits

1. Attribute parser/AST metadata: parse `#[...]` groups and preserve target,
   argument, and source metadata without enabling reflection support yet.
2. Class-table graph: add interface declarations, implementation checks, and
   trait composition before reopening rows that depend on inheritance metadata.
3. Anonymous class lowering: assign stable generated class metadata and object
   construction semantics through the same class table as named classes.
4. Property/method metadata: separate read, write, unset, by-reference, and
   visibility modes for declared properties and methods, including typed and
   readonly slots.
5. Magic/ArrayAccess dispatch: route magic hooks and `ArrayAccess` through
   shared callable dispatch with PHP-compatible diagnostics and reflection
   metadata.
6. Autoload/runtime symbol mutation: define the dynamic class lookup boundary
   before turning autoload rows back into runnable native tests.

## Verification

```sh
cargo fmt --check
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-ivfw-before
tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  .runtime/ptn-ivfw/class-object-metadata-cluster.txt
```
