# PHPT Broad 1k Attribute Syntax Current Frontier: 2026-06-14 ptn-lx5w

Issue: `ptn-lx5w`

This slice records the current broad 1k `unsupported-attribute-syntax-metadata`
category as a dedicated focused manifest. It is a blocker map, not PHP
attribute runtime support.

## Broad 1k Evidence

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-lx5w-baseline-1k
```

Generated broad manifest:

```text
.runtime/ptn-lx5w-baseline-1k/20260614T121543Z/phpt-baseline-1000.txt
```

Classifier artifacts:

```text
.runtime/phpt-progress/summary-20260614T121543Z.txt
.runtime/phpt-progress/classification-20260614T121543Z.tsv
.runtime/phpt-progress/runnable-20260614T121543Z.txt
.runtime/phpt-progress/excluded-20260614T121543Z.tsv
```

State:

```text
PTN: dfa23a6f856e
php-src PHPT corpus revision: 8c63ec400ce8e07c57a8d9499317b96a8beafb8b
```

Broad result:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 1,000 | 424 | 576 |

Largest current classifier buckets:

| Classification | Rows |
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
| `unsupported-anonymous-class` | 15 |
| `unsupported-interface-implementation` | 15 |
| `unsupported-resource-limit-ini` | 15 |

## Focused Attribute Syntax Manifest

Committed focused manifest:

```text
tools/phpt-attribute-syntax-current-ptn-lx5w-manifest.txt
```

Selection:

```sh
sort \
  .runtime/phpt-progress/excluded-20260614T121543Z/unsupported-attribute-syntax-metadata.txt \
  -o tools/phpt-attribute-syntax-current-ptn-lx5w-manifest.txt
```

Focused replay:

```sh
PHPT_PROGRESS_DIR=.runtime/ptn-lx5w-attribute-syntax-focused \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-attribute-syntax-current-ptn-lx5w-manifest.txt
```
Focused artifact:

```text
.runtime/ptn-lx5w-attribute-syntax-focused/classification-20260614T122118Z.tsv
```

Focused result:

| Selected | Runnable | Excluded | Bucket |
| ---: | ---: | ---: | --- |
| 141 | 0 | 141 | `unsupported-attribute-syntax-metadata` |

After rebasing over the docs-only filesystem blocker map at `ee4439bf0`, the
same focused replay on `2aaff7905643` wrote:

```text
.runtime/ptn-lx5w-attribute-syntax-focused-rebased/classification-20260614T122344Z.tsv
```

with the same 141 selected, 0 runnable, and 141 excluded result.

## Path Concentration

| Attribute test family | Rows |
| --- | ---: |
| Root `Zend/tests/attributes` rows | 35 |
| `deprecated` | 34 |
| `override` | 18 |
| `delayed_target_validation` | 18 |
| `constants` | 17 |
| `nodiscard` | 15 |
| `Attribute` | 4 |

## Relation To Existing Maps

Earlier work split PHP attribute rows out of the aggregate language bucket and
then split the broad attribute metadata category into syntax and internal
metadata buckets. This report is the current dedicated category replay for the
141 syntax rows under the post-split bucket name. The 8 internal attribute
metadata rows remain separate in the broader class/object/attribute metadata
maps.

## Blocker Boundary

These rows should stay classified until PTN has generic PHP attribute support:

1. Lexer/parser support for grouped `#[...]` attribute lists before every
   attachable declaration.
2. AST and semantic metadata for names, arguments, grouping, source spans, and
   declaration attachment targets.
3. Constant-expression evaluation for attribute arguments, including name
   resolution and strict-types interactions.
4. Target, repeatability, delayed validation, and built-in attribute semantics
   such as `Deprecated`, `NoDiscard`, `Override`, and `AllowDynamicProperties`.
5. Reflection integration through declaration metadata and
   `ReflectionAttribute`.

Opening these rows before that shared architecture exists would turn one known
parser/metadata frontier into scattered parse, validation, reflection, and
diagnostic failures.

## Verification

```sh
cargo fmt --check
cargo test --test phpt_classifier
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-lx5w-baseline-1k
PHPT_PROGRESS_DIR=.runtime/ptn-lx5w-attribute-syntax-focused \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-attribute-syntax-current-ptn-lx5w-manifest.txt
```
