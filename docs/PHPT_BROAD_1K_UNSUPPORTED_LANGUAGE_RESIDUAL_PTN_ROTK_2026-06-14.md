# PHPT Broad 1k Unsupported-Language Residual: 2026-06-14 ptn-rotk

Issue: `ptn-rotk`

This slice refreshed the broad 1k PHPT classifier on `origin/master` at
`68259103345c` and recorded the post-attribute-split `unsupported-language`
residual. It is a blocker map, not a runtime support claim.

The earlier unsupported-language map had 288 rows because PHP attribute syntax
was still grouped under `unsupported-language`. After the explicit
`unsupported-attribute-metadata` split, the broad 1k residual language bucket is
147 rows. Those rows still require generic parser, AST/IR, symbol-table,
class-metadata, and call-lowering work before they should become runnable.
`ptn-18tp` subsequently split this same 147-row residual into semantic
classifier buckets; this manifest remains the pre-split residual source set.

## Broad 1k Evidence

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-rotk-baseline-1k
```

Generated broad manifest:

```text
.runtime/ptn-rotk-baseline-1k/20260614T092949Z/phpt-baseline-1000.txt
```

Classifier artifacts:

```text
.runtime/phpt-progress/summary-20260614T092949Z.txt
.runtime/phpt-progress/classification-20260614T092949Z.tsv
.runtime/phpt-progress/runnable-20260614T092949Z.txt
.runtime/phpt-progress/excluded-20260614T092949Z.tsv
```

State:

```text
PTN: 68259103345c
php-src PHPT corpus revision: 8c63ec400ce8e07c57a8d9499317b96a8beafb8b
```

Broad result:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 1,000 | 424 | 576 |

Relevant classifier buckets:

| Classification | Rows |
| --- | ---: |
| `unsupported-attribute-metadata` | 149 |
| `unsupported-language` | 147 |
| `unsupported-class-metadata` | 135 |
| `unsupported-request-input-ini` | 28 |
| `unsupported-extension` | 20 |
| `unsupported-diagnostics-runtime` | 17 |
| `unsupported-assertion-ini` | 17 |

## Focused Residual Manifest

The 147-row residual manifest is committed at:

```text
tools/phpt-unsupported-language-residual-ptn-rotk-manifest.txt
```

It was copied from:

```text
.runtime/phpt-progress/excluded-20260614T092949Z/unsupported-language.txt
```

Focused verification:

```sh
PHPT_PROGRESS_DIR=.runtime/ptn-rotk-unsupported-language-focused \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-unsupported-language-residual-ptn-rotk-manifest.txt
```

Focused artifact:

```text
.runtime/ptn-rotk-unsupported-language-focused/classification-20260614T093615Z.tsv
```

Focused result before `ptn-18tp`:

| Selected | Runnable | Excluded | Bucket |
| ---: | ---: | ---: | --- |
| 147 | 0 | 147 | `unsupported-language` |

## Residual Split

Reason split from the then-current broad classifier:

| Generic blocker | Rows |
| --- | ---: |
| Call-site or array unpacking (`...`) | 34 |
| Trait declarations | 25 |
| Interface declarations | 23 |
| Interface implementation checks | 15 |
| Anonymous class syntax (`new class`) | 15 |
| Nullable type-hint metadata and coercion (`?T`) | 14 |
| Static local variables | 11 |
| Variable variables and runtime symbol-table lookup/mutation | 8 |
| Named-argument binding for modeled array internals | 1 |
| Generator/yield lowering | 1 |
| Total | 147 |

Path split:

| Path group | Rows |
| --- | ---: |
| `Zend/tests/attributes/` | 46 |
| `Zend/tests/anon/` | 22 |
| `Zend/tests/arg_unpack/` | 14 |
| `Zend/tests/array_unpack/` | 13 |
| `ext/standard/tests/array/` | 11 |
| `Zend/tests/ArrayAccess/` | 10 |
| `Zend/tests/backtrace/` | 5 |
| `Zend/tests/arrow_functions/` | 5 |
| `Zend/tests/autoload/` | 2 |
| `Zend/tests/assert/` | 2 |
| `tests/basic/` | 1 |
| Root-level `Zend/tests/*.phpt` singles | 16 |

## Implementation Boundary

No row-specific patch should open these tests. The largest remaining groups are
separate generic systems:

- unpacking requires source-order argument expansion, named/positional merge
  rules, reference checks, and array spread lowering;
- traits and interfaces require declaration metadata, composition,
  implementation checks, conflict diagnostics, constants, and method tables;
- anonymous classes require class declaration synthesis, constructor dispatch,
  lexical scope handling, and metadata naming;
- nullable type hints and static locals belong to the shared function/type
  model;
- variable variables require explicit runtime symbol-table fallback.

The productive path is to implement one of those generic surfaces and then
rerun this focused manifest, rather than treating the 147 rows as a single
runnable cluster.

## Verification

```sh
cargo fmt --check
cargo test --test phpt_classifier
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-rotk-baseline-1k
PHPT_PROGRESS_DIR=.runtime/ptn-rotk-unsupported-language-focused \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-unsupported-language-residual-ptn-rotk-manifest.txt
```
