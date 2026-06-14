# PHPT Broad 1k Cluster Slice: 2026-06-14 ptn-aaoj

Issue: `ptn-aaoj`

This slice refreshes the broad PHPT 1k classifier on current `master` and
checks whether a new high-yield broad cluster is available. It records a
blocker and coverage map rather than a runtime behavior change.

The current broad runnable surface is already fully covered by committed
focused manifests. The remaining large buckets are separate compiler/runtime
boundaries, so this slice does not claim a credible one-patch implementation
target that can move at least 25 broad rows without entering an existing
focused frontier.

## Broad 1k Evidence

Command:

```sh
PHPT_PROGRESS_DIR=.runtime/ptn-aaoj-baseline-current-progress \
  tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-aaoj-baseline-current
```

Generated broad manifest:

```text
.runtime/ptn-aaoj-baseline-current/20260614T115609Z/phpt-baseline-1000.txt
```

Artifacts:

```text
.runtime/ptn-aaoj-baseline-current-progress/classification-20260614T115609Z.tsv
.runtime/ptn-aaoj-baseline-current-progress/runnable-20260614T115609Z.txt
.runtime/ptn-aaoj-baseline-current-progress/excluded-20260614T115609Z.tsv
.runtime/ptn-aaoj-baseline-current-progress/summary-20260614T115609Z.txt
```

State:

```text
PTN evidence run: ce822ad7e3f1
php-src PHPT corpus: /home/claude/php-src-phpt
corpus revision: 8c63ec400ce8e07c57a8d9499317b96a8beafb8b
```

Classifier result:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 1,000 | 424 | 576 |

Top classifier buckets:

| Classification | Rows |
| --- | ---: |
| `runnable` | 424 |
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
| `unsupported-type-hint` | 14 |
| `sapi-behavior` | 13 |
| `unsupported-typed-property-metadata` | 12 |
| `unsupported-function-state` | 11 |
| `unsupported-class-contract-metadata` | 9 |
| `unsupported-autoload-metadata` | 9 |
| `unsupported-assertion-runtime` | 9 |
| `unsupported-internal-attribute-metadata` | 8 |
| `unsupported-dynamic-symbol` | 8 |

## Runnable Family Map

The 424 runnable rows split by source family as follows:

| Family | Rows |
| --- | ---: |
| `ext/standard/tests` | 294 |
| `Zend/tests` root | 81 |
| `Zend/tests/asymmetric_visibility` | 22 |
| `tests/basic` | 16 |
| `Zend/tests/ast` | 4 |
| `Zend/tests/arrow_functions` | 3 |
| `Zend/tests/assert` | 2 |
| `Zend/tests/attributes` | 1 |
| `Zend/tests/access_modifiers` | 1 |

The largest runnable group remains standard-array helper behavior, but the
committed standard-array residual map already splits its failures across
key/value conversion, callback diagnostics, ordered-array mutation/reference
semantics, `array_rand()`, and user-comparator set operations. Those are
separate generic primitives, not one broad implementation patch.

## Focused Manifest Reconciliation

Command:

```sh
tmpdir=.runtime/ptn-aaoj-analysis-current
mkdir -p "$tmpdir"
awk 'NF && $1 !~ /^#/' tools/phpt-*-manifest.txt |
  LC_ALL=C sort -u > "$tmpdir/committed-focused-rows.txt"
LC_ALL=C sort -u .runtime/ptn-aaoj-baseline-current-progress/runnable-20260614T115609Z.txt \
  > "$tmpdir/current-runnable.txt"
comm -23 "$tmpdir/current-runnable.txt" "$tmpdir/committed-focused-rows.txt" \
  > "$tmpdir/unmatched-runnable.txt"
wc -l "$tmpdir/current-runnable.txt" \
  "$tmpdir/committed-focused-rows.txt" \
  "$tmpdir/unmatched-runnable.txt"
```

Result:

```text
  424 .runtime/ptn-aaoj-analysis-current/current-runnable.txt
 1656 .runtime/ptn-aaoj-analysis-current/committed-focused-rows.txt
    0 .runtime/ptn-aaoj-analysis-current/unmatched-runnable.txt
```

Largest intersections with committed focused manifests:

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

## Blocker Boundary

There are zero broad runnable rows outside committed focused manifests. A new
generic implementation should start from one of those focused frontiers rather
than rediscovering the same broad row families.

The largest excluded categories are also distinct architectural surfaces:

1. Attribute syntax, declaration attachment, and reflection metadata: 141 rows.
2. Internal attribute/reflection metadata: 8 rows.
3. Magic method metadata and dispatch/reflection parity: 69 rows.
4. Call-site and array unpacking: 34 rows.
5. Request/SAPI input and INI state: 28 rows.
6. Trait declarations: 25 rows.
7. Interface declarations and implementation checks: 38 rows total.

Those are credible future implementation themes, but each needs a dedicated
semantic design and focused test path. Treating them as one broad cluster would
mix parser, class metadata, call lowering, request runtime, and reflection
work in one branch.

## Verification

```sh
PHPT_PROGRESS_DIR=.runtime/ptn-aaoj-baseline-current-progress \
  tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-aaoj-baseline-current
cargo fmt --check
cargo test --test phpt_classifier
```
