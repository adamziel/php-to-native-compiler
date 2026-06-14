# PHPT Broad 1k Cluster Slice: 2026-06-14 ptn-666n

Issue: `ptn-666n`

This slice refreshes the broad PHPT 1k classifier from the `ptn-666n`
branch and checks whether a fresh high-yield semantic cluster can credibly
move at least 25 rows. It is a blocker map, not a runtime behavior claim.
The current broad runnable set is already fully covered by committed focused
manifests, while the large excluded buckets are separate compiler/runtime
surfaces that should be handled as focused semantic work.

## Broad 1k Evidence

Command:

```sh
PHPT_PROGRESS_DIR=.runtime/ptn-666n-current-progress \
  tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-666n-current-baseline
```

Generated broad manifest:

```text
.runtime/ptn-666n-current-baseline/20260614T134541Z/phpt-baseline-1000.txt
```

Classifier artifacts:

```text
.runtime/ptn-666n-current-progress/manifest-20260614T134541Z.txt
.runtime/ptn-666n-current-progress/classification-20260614T134541Z.tsv
.runtime/ptn-666n-current-progress/runnable-20260614T134541Z.txt
.runtime/ptn-666n-current-progress/excluded-20260614T134541Z.tsv
```

State:

```text
PTN evidence run: 62adccff7f75 plus ptn-666n docs replay
Rebased integration base: 62adccff7 (class/object blocker map refresh)
php-src PHPT corpus: /home/claude/php-src-phpt
corpus revision: 8c63ec400ce8e07c57a8d9499317b96a8beafb8b
```

Classifier result:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 1,000 | 424 | 576 |

## Current Classifier Buckets

| Classification | Rows |
| --- | ---: |
| `runnable` | 424 |
| `unsupported-attribute-syntax-metadata` | 141 |
| `unsupported-object-string-conversion-metadata` | 61 |
| `unsupported-request-input-ini` | 28 |
| `unsupported-trait-declaration` | 25 |
| `unsupported-interface-declaration` | 23 |
| `unsupported-call-unpacking` | 20 |
| `unsupported-extension` | 20 |
| `unsupported-property-visibility-metadata` | 19 |
| `unsupported-diagnostics-runtime` | 17 |
| `unsupported-assertion-ini` | 17 |
| `unsupported-resource-limit-ini` | 15 |
| `unsupported-interface-implementation` | 15 |
| `unsupported-anonymous-class` | 15 |
| `unsupported-array-unpacking` | 14 |
| `unsupported-type-hint` | 14 |
| `sapi-behavior` | 13 |
| `unsupported-typed-property-metadata` | 12 |
| `unsupported-function-state` | 11 |
| `unsupported-class-contract-metadata` | 9 |
| `unsupported-autoload-metadata` | 9 |
| `unsupported-assertion-runtime` | 9 |
| `unsupported-magic-method-metadata` | 8 |
| `unsupported-internal-attribute-metadata` | 8 |
| `unsupported-dynamic-symbol` | 8 |
| `unsupported-readonly-property-metadata` | 7 |
| `unsupported-method-visibility-metadata` | 7 |
| `unsupported-diagnostics-ini` | 5 |
| `harness-cleanup` | 4 |
| `unsupported-internal-reflection-metadata` | 3 |
| `process-boundary` | 3 |
| `unsupported-scalar-format-ini` | 2 |
| `unsupported-opcache-ini` | 2 |
| `unsupported-host-path-ini` | 2 |
| `unsupported-function-disable-ini` | 2 |
| `skipif-precondition` | 2 |
| `unsupported-resource-limit` | 1 |
| `unsupported-internal-call-binding` | 1 |
| `unsupported-internal` | 1 |
| `unsupported-generator-runtime` | 1 |
| `external-service` | 1 |
| `environment-assumption` | 1 |

Grouped by implementation ownership, the excluded rows are:

| Owner | Rows | Shape |
| --- | ---: | --- |
| Attribute metadata | 149 | PHP `#[...]` syntax attachment, attribute metadata, repeatability/target checks, and internal attribute metadata. |
| Class/object metadata | 213 | Trait/interface declarations, anonymous classes, object string conversion, magic methods, visibility, typed/readonly slots, contracts, autoload, and internal reflection metadata. |
| Language and dynamic call surfaces | 68 | Call-site and array unpacking, nullable type hints, function static state, variable variables, and generator lowering. |
| Diagnostics/assertion state | 48 | Diagnostic runtime metadata, assertion runtime modes, assertion INI behavior, and diagnostic INI surfaces. |
| Runtime boundary and environment | 98 | Request/SAPI state, unavailable extensions, resource limits, process/harness boundaries, host/environment assumptions, scalar-format INI, and internal call binding. |

The groups above are real compatibility targets, but each crosses several
generic contracts. They should not be reopened as row-shaped broad patches.

## Runnable Family Map

The 424 runnable rows split by source family as follows:

| Family | Rows |
| --- | ---: |
| `ext/standard/tests/array/*` | 294 |
| Root-level `Zend/tests/*.phpt` | 81 |
| `Zend/tests/asymmetric_visibility/*` | 22 |
| `tests/basic/*` | 16 |
| `Zend/tests/ast/*` | 4 |
| `Zend/tests/arrow_functions/*` | 3 |
| `Zend/tests/assert/*` | 2 |
| `Zend/tests/attributes/*` | 1 |
| `Zend/tests/access_modifiers/*` | 1 |

The standard-array surface is the only runnable family above the 25-row target:

| Standard-array family | Rows | Current boundary |
| --- | ---: | --- |
| Other array helpers | 77 | Mixed helpers below the threshold once split by primitive. |
| `array_diff*` | 39 | Existing set-operation frontier; residuals are comparator, include, nested-value warning, and string/heredoc edges. |
| `array_chunk()` | 32 | Already green in focused broad evidence. |
| `array_intersect*` | 30 | Same set-operation frontier shape as `array_diff*`. |
| `array_map()` | 19 | Callback dispatch, arity diagnostics, object callables, reference behavior, and zip semantics. |
| `array_key*` | 19 | Key coercion, resource/object diagnostics, and warning parity. |
| `array_merge*` | 18 | Recursive merge, references, reindexing, and ordered-array mutation. |
| `array_sum()` | 12 | Mostly green in focused evidence; residuals are not a broad patch. |
| `array_slice()` | 10 | Ordered-array slicing and key preservation edges. |
| `array_filter()` | 10 | Callback mode and key/value argument shape. |
| `array_fill*` | 8 | Allocation/resource-limit and key/value conversion edges. |
| `array_rand()` | 7 | Coherent helper target, but below the broad threshold alone. |
| `array_product()` | 7 | Numeric conversion and extension/resource diagnostic edges. |
| `array_push()` | 6 | Append/reference diagnostics and call-unpacking boundaries. |

## Focused Manifest Reconciliation

Command:

```sh
tmp=.runtime/ptn-666n-current-analysis
mkdir -p "$tmp"
awk 'NF && $1 !~ /^#/ {print $1}' tools/phpt-*-manifest.txt |
  LC_ALL=C sort -u > "$tmp/committed-focused-rows.txt"
LC_ALL=C sort -u \
  .runtime/ptn-666n-current-progress/runnable-20260614T134541Z.txt \
  > "$tmp/current-runnable.txt"
comm -23 "$tmp/current-runnable.txt" "$tmp/committed-focused-rows.txt" \
  > "$tmp/unmatched-runnable.txt"
wc -l "$tmp/current-runnable.txt" \
  "$tmp/committed-focused-rows.txt" \
  "$tmp/unmatched-runnable.txt"
```

Result:

```text
  424 .runtime/ptn-666n-current-analysis/current-runnable.txt
 1656 .runtime/ptn-666n-current-analysis/committed-focused-rows.txt
    0 .runtime/ptn-666n-current-analysis/unmatched-runnable.txt
 2080 total
```

Largest intersections with committed focused manifests:

| Rows | Manifest |
| ---: | --- |
| 294 | `tools/phpt-standard-array-current-ptn-ke94-manifest.txt` |
| 294 | `tools/phpt-broad-standard-array-frontier-manifest.txt` |
| 127 | `tools/phpt-bounded-manifest.txt` |
| 81 | `tools/phpt-zend-root-current-ptn-xgk8-manifest.txt` |
| 36 | `tools/phpt-array-key-value-frontier-manifest.txt` |
| 35 | `tools/phpt-zend-bug-regression-frontier-manifest.txt` |
| 34 | `tools/phpt-core-basic-operator-frontier-manifest.txt` |
| 32 | `tools/phpt-zend-assignment-reference-frontier-manifest.txt` |
| 32 | `tools/phpt-array-chunk-broad-1k-manifest.txt` |
| 25 | `tools/phpt-zend-operator-control-frontier-manifest.txt` |
| 22 | `tools/phpt-asymmetric-visibility-frontier-manifest.txt` |
| 21 | `tools/phpt-heredoc-nowdoc-frontier-manifest.txt` |

There are zero current broad-runnable rows outside committed focused manifests.

## Blocker Boundary

No fresh broad 1k cluster in this slice has a credible single generic change
that newly moves at least 25 rows:

1. The only runnable family above the threshold is the standard-array surface,
   and all 294 rows are already covered by committed focused manifests. The
   remaining failures split across callbacks, ordered-array mutation/reference
   behavior, key/value coercion, comparator diagnostics, and resource limits.
2. The excluded buckets above the 25-row target are not one implementation
   cluster. Attribute metadata, object string conversion, call/array unpacking,
   request/SAPI INI, and trait declarations require distinct parser,
   metadata, lowering, runtime, and reflection work.
3. The exact 25-row trait bucket is high-yield but not a low-risk broad patch:
   it needs trait declaration parsing, class-table composition, method
   conflict rules, visibility adaptation, and reflection metadata.

The next productive implementation work should start from an existing focused
frontier: standard-array residuals, class/object metadata, diagnostics and
assertion state, Zend operator/control, call unpacking, or trait/interface
declaration support.

## Verification

```sh
PHPT_PROGRESS_DIR=.runtime/ptn-666n-current-progress \
  tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-666n-current-baseline
cargo fmt --check
cargo test --test phpt_classifier
```

Results:

- Current broad 1k classify-only: 1,000 selected, 424 runnable, 576
  classified out; run-tests-exit 0.
- Focused-manifest reconciliation: 424 current runnable rows, 1,656 committed
  focused rows, 0 unmatched runnable rows.
- `cargo fmt --check` passed.
- `cargo test --test phpt_classifier` passed: 36 tests.
