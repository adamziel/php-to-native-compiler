# PHPT Broad 1k Cluster Slice: 2026-06-14 ptn-p2i7

Issue: `ptn-p2i7`

This slice refreshes the broad PHPT 1k classifier on the `origin/master`
compiler state available at measurement time and checks for one high-yield
semantic cluster that can credibly move at least 25 rows. It is a blocker map,
not a runtime behavior change.

The current broad runnable surface is fully reconciled with committed focused
manifests. The remaining 25+ row groups are already classified blocker
surfaces, not small runnable implementation patches.

## Broad 1k Evidence

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-p2i7-baseline-current
```

Generated broad manifest:

```text
.runtime/ptn-p2i7-baseline-current/20260614T112338Z/phpt-baseline-1000.txt
```

Artifacts:

```text
.runtime/phpt-progress/classification-20260614T112338Z.tsv
.runtime/phpt-progress/runnable-20260614T112338Z.txt
.runtime/phpt-progress/excluded-20260614T112338Z.tsv
.runtime/phpt-progress/summary-20260614T112338Z.txt
```

State:

```text
PTN evidence commit: 6afe0ba88847
origin/master at measurement: 6afe0ba88847
php-src PHPT corpus: /home/claude/php-src-phpt
corpus revision: 8c63ec400ce8e07c57a8d9499317b96a8beafb8b
```

Classifier result:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 1,000 | 424 | 576 |

Classifier buckets:

| Classification | Rows |
| --- | ---: |
| `runnable` | 424 |
| `unsupported-attribute-metadata` | 149 |
| `unsupported-magic-method-metadata` | 69 |
| `unsupported-call-unpacking` | 34 |
| `unsupported-request-input-ini` | 28 |
| `unsupported-trait-declaration` | 25 |
| `unsupported-interface-declaration` | 23 |
| `unsupported-extension` | 20 |
| `unsupported-property-visibility-metadata` | 19 |
| `unsupported-assertion-ini` | 17 |
| `unsupported-diagnostics-runtime` | 17 |
| `unsupported-anonymous-class` | 15 |
| `unsupported-interface-implementation` | 15 |
| `unsupported-resource-limit-ini` | 15 |
| `unsupported-type-hint` | 14 |
| `sapi-behavior` | 13 |
| `unsupported-typed-property-metadata` | 12 |
| `unsupported-function-state` | 11 |
| `unsupported-assertion-runtime` | 9 |
| `unsupported-autoload-metadata` | 9 |
| `unsupported-class-contract-metadata` | 9 |
| `unsupported-dynamic-symbol` | 8 |
| `unsupported-method-visibility-metadata` | 7 |
| `unsupported-readonly-property-metadata` | 7 |
| `unsupported-diagnostics-ini` | 5 |
| `harness-cleanup` | 4 |
| `process-boundary` | 3 |
| `unsupported-internal-reflection-metadata` | 3 |
| `skipif-precondition` | 2 |
| `unsupported-function-disable-ini` | 2 |
| `unsupported-host-path-ini` | 2 |
| `unsupported-opcache-ini` | 2 |
| `unsupported-scalar-format-ini` | 2 |
| `environment-assumption` | 1 |
| `external-service` | 1 |
| `unsupported-generator-runtime` | 1 |
| `unsupported-internal` | 1 |
| `unsupported-internal-call-binding` | 1 |
| `unsupported-resource-limit` | 1 |

After replaying onto the current integration branch, the broad 1k classify-only
pass on `2ef361944d26` preserved the same 1,000 selected, 424 runnable, and 576
excluded totals. The newer classifier split the former 149-row attribute bucket
into 141 `unsupported-attribute-syntax-metadata` rows and 8
`unsupported-internal-attribute-metadata` rows. The current artifact is:

```text
.runtime/ptn-p2i7-baseline-current-2-progress/classification-20260614T121918Z.tsv
```

## Runnable Surface

The 424 runnable rows split by source family as:

| Family | Rows |
| --- | ---: |
| `ext/standard/tests/array` | 294 |
| `Zend/tests` root | 81 |
| `Zend/tests/asymmetric_visibility` | 22 |
| `tests/basic` | 16 |
| `Zend/tests/ast` | 4 |
| `Zend/tests/arrow_functions` | 3 |
| `Zend/tests/assert` | 2 |
| `Zend/tests/access_modifiers` | 1 |
| `Zend/tests/attributes` | 1 |

Focused manifest reconciliation:

```text
  424 .runtime/ptn-p2i7-runnable-current.txt
 1656 .runtime/ptn-p2i7-focused-rows-current.txt
    0 .runtime/ptn-p2i7-unmatched-runnable-current.txt
```

The broad runnable rows all appear in committed focused manifests. Largest
non-partitioned focused-manifest intersections:

| Rows | Manifest |
| ---: | --- |
| 294 | `tools/phpt-standard-array-current-ptn-ke94-manifest.txt` |
| 294 | `tools/phpt-broad-standard-array-frontier-manifest.txt` |
| 127 | `tools/phpt-bounded-manifest.txt` |
| 81 | `tools/phpt-zend-root-current-ptn-xgk8-manifest.txt` |
| 65 | `tools/phpt-array-callback-validation-manifest.txt` |
| 36 | `tools/phpt-array-key-value-frontier-manifest.txt` |
| 35 | `tools/phpt-zend-bug-regression-frontier-manifest.txt` |
| 34 | `tools/phpt-core-basic-operator-frontier-manifest.txt` |
| 32 | `tools/phpt-zend-assignment-reference-frontier-manifest.txt` |
| 32 | `tools/phpt-array-chunk-broad-1k-manifest.txt` |
| 25 | `tools/phpt-zend-operator-control-frontier-manifest.txt` |
| 22 | `tools/phpt-asymmetric-visibility-frontier-manifest.txt` |
| 21 | `tools/phpt-heredoc-nowdoc-frontier-manifest.txt` |
| 19 | `tools/phpt-cow-manifest.txt` |
| 11 | `tools/phpt-cow-broad-frontier-manifest.txt` |

## Blocker Boundary

No new broad-runnable cluster remains outside committed focused evidence.
The 25+ row excluded groups are each broad compiler/runtime systems:

| Blocker | Rows | Boundary |
| --- | ---: | --- |
| Attribute syntax metadata | 141 | PHP `#[...]` syntax, parser metadata, and reflected userland attribute shapes. |
| Internal attribute metadata | 8 | Reflection/internal attribute metadata for built-in classes and functions. |
| Magic method metadata | 69 | Magic dispatch, object conversion, debug metadata, and reflection naming. |
| Call/array unpacking | 34 | Call-site unpacking, array literal unpacking, argument order, references, and diagnostics. |
| Request/input INI | 28 | Request, upload, argv, and input SAPI state rather than CLI script semantics. |
| Trait declarations | 25 | Trait composition, aliases, precedence, conflict diagnostics, and class metadata. |

These are credible implementation frontiers, but not single narrow fixes. The
next productive runtime/compiler work should start from the existing focused
manifests for standard-array residuals, Zend root semantics, diagnostics, or a
dedicated design for one excluded class/language boundary.

## Verification

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-p2i7-baseline-current
PHPT_PROGRESS_DIR=.runtime/ptn-p2i7-baseline-current-2-progress \
  tools/run-phpt-baseline.sh --tier 1000 --classify-only \
    --out-dir .runtime/ptn-p2i7-baseline-current-2
cargo fmt --check
cargo test --test phpt_classifier
```
