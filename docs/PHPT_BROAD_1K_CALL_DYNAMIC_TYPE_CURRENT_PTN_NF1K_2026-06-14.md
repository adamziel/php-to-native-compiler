# PHPT Broad 1k Call/Dynamic Type Current Map: 2026-06-14 ptn-nf1k

Issue: `ptn-nf1k`

This slice records the 69-row call/dynamic/type language boundary from the
broad PHPT 1k classifier. It is a blocker map, not a runtime support claim. The
selected rows are the union of the `unsupported-call-unpacking` category and
the function/type/dynamic categories that already have separate focused maps.

The combined frontier is useful because these rows share adjacent
implementation boundaries: PHP-aware argument expansion and call binding,
function-local state, type metadata, generator suspension, and explicit dynamic
symbol-table fallback. Landing any one piece should re-run this union manifest
and then remove only the classifier branch whose semantics are actually
implemented.

## Broad 1k Evidence

Command:

```sh
PHPT_PROGRESS_DIR=.runtime/ptn-nf1k-1k-progress \
  tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-nf1k-1k
```

Generated broad manifest:

```text
.runtime/ptn-nf1k-1k/20260614T113800Z/phpt-baseline-1000.txt
```

Classifier artifacts:

```text
.runtime/ptn-nf1k-1k-progress/classification-20260614T113800Z.tsv
.runtime/ptn-nf1k-1k-progress/runnable-20260614T113800Z.txt
.runtime/ptn-nf1k-1k-progress/excluded-20260614T113800Z.tsv
```

State:

```text
PTN: 4f2bcbaefaae
php-src PHPT corpus revision: 8c63ec400ce8e07c57a8d9499317b96a8beafb8b
```

Broad classifier result:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 1,000 | 424 | 576 |

## Focused Manifest

Committed manifest:

```text
tools/phpt-call-dynamic-type-current-ptn-nf1k-manifest.txt
```

Selection from `classification-20260614T113800Z.tsv`:

```sh
awk -F'\t' '$2 ~ /^(unsupported-call-unpacking|unsupported-type-hint|unsupported-function-state|unsupported-dynamic-symbol|unsupported-generator-runtime|unsupported-internal-call-binding)$/ {print $1}'
```

Focused classify-only replay:

```sh
PHPT_PROGRESS_DIR=.runtime/ptn-nf1k-call-dynamic-type-current \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-call-dynamic-type-current-ptn-nf1k-manifest.txt
```

Focused artifact:

```text
.runtime/ptn-nf1k-call-dynamic-type-current/classification-20260614T115300Z.tsv
```

Focused result:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 69 | 0 | 69 |

## Category Split

| Classification | Rows |
| --- | ---: |
| `unsupported-call-unpacking` | 34 |
| `unsupported-type-hint` | 14 |
| `unsupported-function-state` | 11 |
| `unsupported-dynamic-symbol` | 8 |
| `unsupported-generator-runtime` | 1 |
| `unsupported-internal-call-binding` | 1 |

Path split:

| Path group | Rows |
| --- | ---: |
| Root and miscellaneous `Zend/tests` | 16 |
| `Zend/tests/array_unpack` | 13 |
| `Zend/tests/arg_unpack` | 13 |
| `Zend/tests/attributes` | 12 |
| `ext/standard/tests/array` | 10 |
| `Zend/tests/arrow_functions` | 5 |

## Relation To Existing Maps

This manifest is the current union of two committed focused surfaces:

| Existing focus | Rows in this manifest |
| --- | ---: |
| `tools/phpt-call-unpacking-frontier-ptn-wd68-manifest.txt` | 34 |
| `tools/phpt-function-dynamic-current-ptn-rlzz-manifest.txt` | 35 |

The union remains fully classified on current `master`: all 69 rows are
excluded, and no row is runnable.

## Blocker Boundary

These rows are above the broad-slice threshold, but one credible patch cannot
open them all. Generic support needs separate compiler/runtime layers:

- parser, AST, and IR representation for call-site and array unpacking;
- ordered argument expansion, named/positional merge rules, duplicate-key
  diagnostics, by-reference binding, and internal-call parameter metadata;
- array spread semantics for integer and string keys, invalid operands,
  traversables, references, and COW behavior;
- nullable and `never` type metadata, coercion, and return/control-flow
  validation;
- function-local `static` storage with initialization timing, recursion,
  references, and shutdown behavior;
- variable-variable lookup/mutation through an explicit dynamic symbol-table
  boundary;
- generator/yield suspension, return propagation, and by-reference rejection.

The next implementation slice should choose one category from this manifest,
land the generic semantics, and then re-run this focused union plus the broad
1k classifier.

## Verification

```sh
PHPT_PROGRESS_DIR=.runtime/ptn-nf1k-call-dynamic-type-current \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-call-dynamic-type-current-ptn-nf1k-manifest.txt
cargo fmt --check
cargo test --test phpt_classifier
```
