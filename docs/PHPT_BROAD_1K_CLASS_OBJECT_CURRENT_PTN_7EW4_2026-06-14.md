# PHPT Broad 1k Class/Object Current Map: 2026-06-14 ptn-7ew4

Issue: `ptn-7ew4`

This slice refreshes the broad 1k PHPT classifier on the rebased branch and
records the current class/object declaration and metadata cluster. It is a
blocker map, not a runtime support claim: the rows below need shared parser,
symbol-table, class metadata, magic dispatch, visibility, reflection, and
runtime mutation semantics before they should be admitted as executable PHPT
signal.

## Broad 1k Evidence

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-7ew4-baseline-rebased
```

Generated broad manifest:

```text
.runtime/ptn-7ew4-baseline-rebased/20260614T121155Z/phpt-baseline-1000.txt
```

Artifacts:

```text
.runtime/phpt-progress/classification-20260614T121155Z.tsv
.runtime/phpt-progress/runnable-20260614T121155Z.txt
.runtime/phpt-progress/excluded-20260614T121155Z.tsv
.runtime/phpt-progress/summary-20260614T121155Z.txt
```

Historical branch state:

```text
PTN: 5aa87dad0d09
php-src PHPT corpus: /home/claude/php-src-phpt
corpus revision: 8c63ec400ce8e07c57a8d9499317b96a8beafb8b
```

Result:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 1,000 | 424 | 576 |

Top historical broad classifier buckets:

| Bucket | Rows |
| --- | ---: |
| `unsupported-attribute-syntax-metadata` | 141 |
| `unsupported-magic-method-metadata` | 69 |
| `unsupported-call-unpacking` | 34 |
| `unsupported-request-input-ini` | 28 |
| `unsupported-trait-declaration` | 25 |
| `unsupported-interface-declaration` | 23 |
| `unsupported-extension` | 20 |
| `unsupported-property-visibility-metadata` | 19 |
| `unsupported-diagnostics-runtime` | 17 |
| `unsupported-assertion-ini` | 17 |
| `unsupported-resource-limit-ini` | 15 |
| `unsupported-interface-implementation` | 15 |
| `unsupported-anonymous-class` | 15 |

## Focused Cluster

Committed focused manifest:

```text
tools/phpt-class-object-metadata-current-ptn-7ew4-manifest.txt
```

It was selected from the broad 1k rows classified into class/object declaration
and metadata buckets on the ptn-7ew4 branch:

```sh
awk -F'\t' '$2 ~ /^(unsupported-(anonymous-class|interface-implementation|interface-declaration|trait-declaration|attribute-syntax-metadata|internal-attribute-metadata|internal-reflection-metadata|magic-method-metadata|property-visibility-metadata|typed-property-metadata|readonly-property-metadata|method-visibility-metadata|class-contract-metadata|autoload-metadata))$/ {print $1}' \
  .runtime/phpt-progress/classification-20260614T121155Z.tsv
```

Focused replay:

```sh
PHPT_PROGRESS_DIR=.runtime/ptn-7ew4-class-object-current-integrated \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-class-object-metadata-current-ptn-7ew4-manifest.txt
```

Artifact:

```text
.runtime/ptn-7ew4-class-object-current-integrated/classification-20260614T134036Z.tsv
```

Focused result:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 362 | 0 | 362 |

## Bucket Map

| Bucket | Rows | Blocking runtime/compiler surface |
| --- | ---: | --- |
| `unsupported-attribute-syntax-metadata` | 141 | Attribute parser nodes, target validation, declaration metadata, internal attribute classes, and reflection APIs. |
| `unsupported-object-string-conversion-metadata` | 61 | Object string conversion hooks, `__toString()` availability, conversion diagnostics, callback dispatch, and reflection/backtrace metadata. |
| `unsupported-trait-declaration` | 25 | Trait declarations, composition, conflict resolution, aliasing, precedence, and trait method/property metadata. |
| `unsupported-interface-declaration` | 23 | Interface declarations, inherited contracts, constants, method compatibility, and reflection metadata. |
| `unsupported-property-visibility-metadata` | 19 | Protected/private property slots, inherited-private slot separation, visibility diagnostics, and property-table reflection. |
| `unsupported-interface-implementation` | 15 | `implements` validation, interface contract checks, ArrayAccess-style object array access, and diagnostics. |
| `unsupported-anonymous-class` | 15 | Anonymous-class parser/lowering, generated metadata names, constructor dispatch, and reflection naming. |
| `unsupported-typed-property-metadata` | 12 | Typed property declaration metadata, initialization state, assignment coercion, and read diagnostics. |
| `unsupported-class-contract-metadata` | 9 | Abstract/final class and method contract metadata plus override validation. |
| `unsupported-autoload-metadata` | 9 | Runtime class symbol-table mutation and autoload boundaries. |
| `unsupported-internal-attribute-metadata` | 8 | Internal attribute classes and reflection metadata such as `Attribute` and `Deprecated`. |
| `unsupported-magic-method-metadata` | 8 | Magic method visibility/staticness/signature checks and method metadata. |
| `unsupported-readonly-property-metadata` | 7 | Readonly static and indirect readonly mutation diagnostics. |
| `unsupported-method-visibility-metadata` | 7 | Non-public method dispatch and diagnostics. |
| `unsupported-internal-reflection-metadata` | 3 | Complete internal arginfo/class registry reflection metadata. |

## Path Concentration

| Path family | Rows |
| --- | ---: |
| `Zend/tests/attributes/*` | 183 |
| `ext/standard/tests/array/*` | 71 |
| Other/root `Zend/tests/*` | 39 |
| `Zend/tests/anon/*` | 20 |
| `Zend/tests/asymmetric_visibility/*` | 16 |
| `Zend/tests/access_modifiers/*` | 12 |
| `Zend/tests/autoload/*` | 10 |
| `Zend/tests/ArrayAccess/*` | 10 |
| Other | 1 |

## Blocker Boundary

This 362-row cluster is high yield, but not a credible one-patch runtime
implementation target. The rows cross multiple generic PHP systems:

1. Parser and semantic model work for attributes, anonymous classes,
   interfaces, and traits.
2. Class metadata storage for visibility, typed/readonly properties, method
   contracts, and internal reflection.
3. Runtime dispatch for magic methods, property hooks, ArrayAccess-like object
   interactions, and autoload-driven symbol mutation.
4. Reflection/backtrace/diagnostic surfaces that must share the same metadata
   rather than hard-code PHPT output.

The focused manifest keeps the broad 1k class/object surface explicit while
preserving the runnable PHPT signal for already-modeled native semantics.

## Verification

```sh
cargo fmt --check
bash -n tools/phpt-classifier.sh
cargo test --test phpt_classifier
PHPT_PROGRESS_DIR=.runtime/ptn-7ew4-class-object-current-integrated \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-class-object-metadata-current-ptn-7ew4-manifest.txt
```

Results:

- Historical `tools/run-phpt-baseline.sh --tier 1000 --classify-only`: passed;
  1,000 selected, 424 runnable, 576 classified out.
- `cargo fmt --check`, `bash -n tools/phpt-classifier.sh`, and
  `cargo test --test phpt_classifier` passed on the integrated replay.
- Focused class/object replay: passed; 362 selected, 0 runnable, 362 classified
  out across the mapped buckets above.
