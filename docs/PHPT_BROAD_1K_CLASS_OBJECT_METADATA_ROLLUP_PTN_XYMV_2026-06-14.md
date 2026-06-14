# PHPT Broad 1k Class/Object Metadata Rollup: 2026-06-14

Issue: `ptn-xymv`

This slice refreshes the broad 1k PHPT classifier on current `origin/master`
and rolls up the class-like parser and object metadata rows into one current
blocker map. It is not an implementation claim. The selected rows all need the
same generic class-table, object metadata, reflection, visibility, magic
dispatch, trait/interface, anonymous-class, and autoload layers before they can
produce useful executable PHPT signal.

## Broad 1k Evidence

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-xymv-baseline-before
```

Generated manifest:
`.runtime/ptn-xymv-baseline-final/20260614T075345Z/phpt-baseline-1000.txt`

Classifier artifact:
`.runtime/phpt-progress/classification-20260614T075345Z.tsv`

PTN commit: `1e3b9015c2c7`

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

## Focused Rollup

Committed manifest:
`tools/phpt-class-object-metadata-rollup-manifest.txt`

Selection from `classification-20260614T075345Z.tsv`:

```sh
awk -F'\t' '$2=="unsupported-class-metadata" ||
  ($2=="unsupported-language" &&
   $3 ~ /(interface declarations|interface implementation checks|trait declarations|anonymous class syntax)/) {
     print $1
   }'
```

Focused classify-only verification:

```sh
tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-class-object-metadata-rollup-manifest.txt
```

Result at `.runtime/phpt-progress/summary-20260614T075906Z.txt`:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 221 | 0 | 221 |

Classifier split:

| Bucket | Rows |
| --- | ---: |
| `unsupported-class-metadata` | 143 |
| `unsupported-language` class-like declarations | 78 |

## Blocker Split

| Generic blocker | Rows |
| --- | ---: |
| Unsupported magic method dispatch/reflection metadata | 69 |
| Trait declarations and composition | 25 |
| Interface declarations | 23 |
| Non-public property visibility metadata | 19 |
| Interface implementation checks | 15 |
| Anonymous class syntax and metadata | 15 |
| Typed property metadata | 12 |
| Runtime class autoload symbol-table mutation | 9 |
| Internal attribute/reflection metadata | 8 |
| Non-public method visibility dispatch and diagnostics | 7 |
| Indirect readonly property mutation diagnostics | 7 |
| Abstract class/method contract metadata | 5 |
| Final class/method override metadata | 4 |
| Complete internal arginfo/class registry reflection | 3 |

Path concentration:

| Path family | Rows |
| --- | ---: |
| `ext/standard/tests/array` | 71 |
| `Zend/tests/attributes` | 42 |
| `Zend/tests/other` | 33 |
| `Zend/tests/anon` | 20 |
| `Zend/tests/asymmetric_visibility` | 16 |
| `Zend/tests/access_modifiers` | 12 |
| `Zend/tests/autoload` | 10 |
| `Zend/tests/ArrayAccess` | 10 |
| `Zend/tests/backtrace` | 6 |
| `tests/basic` | 1 |

Representative rows:

```text
Zend/tests/ArrayAccess/ArrayAccess_indirect_append.phpt
Zend/tests/anon/001.phpt
Zend/tests/attributes/override/001.phpt
Zend/tests/autoload/bug61011.phpt
Zend/tests/backtrace/debug_backtrace_options.phpt
Zend/tests/bug32427.phpt
Zend/tests/access_modifiers/access_modifiers_011.phpt
Zend/tests/asymmetric_visibility/static_props.phpt
ext/standard/tests/array/array_column_property_visibility.phpt
ext/standard/tests/array/array_map_object3.phpt
tests/basic/bug73969.phpt
```

## Why This Is A Blocker

The rows are high yield, but they are not credible as one narrow patch. Generic
support needs:

- interface and trait declarations in parser, AST, class-table metadata, and
  method/property conflict validation;
- anonymous class expression lowering, naming, constructor dispatch, source
  metadata, reflection, and closure binding;
- full property and method visibility, including typed, readonly, asymmetric,
  inherited, and uninitialized slots;
- magic method dispatch for property access, object conversion, comparison,
  callback dispatch, and array helper internals;
- runtime autoload and symbol-table mutation boundaries that can change class
  availability after native compilation starts;
- reflection metadata for internal and userland classes, methods, properties,
  attributes, arginfo, and closure binding.

Reclassifying these rows as runnable before those layers land would turn a
coherent class/object metadata frontier into scattered parser, runtime, and
diagnostic failures. The next useful implementation split should land one
generic layer, then re-run this manifest and remove only the classifier branch
whose semantics are actually supported.

## Verification

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-xymv-baseline-final
tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-class-object-metadata-rollup-manifest.txt
cargo fmt --check
cargo test --test phpt_classifier
```
