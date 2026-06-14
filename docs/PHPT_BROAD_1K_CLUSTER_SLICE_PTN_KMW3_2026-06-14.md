# PHPT Broad 1k Cluster Slice: 2026-06-14 ptn-kmw3

Issue: `ptn-kmw3`

This slice refreshes the broad PHPT 1k classifier from the `ptn-kmw3` branch
and checks for one high-yield semantic cluster that can credibly move at least
25 broad rows. It is a blocker map, not a runtime support claim. The current
broad runnable set is already fully covered by committed focused manifests, and
the excluded rows above the 25-row threshold are split across generic compiler
and runtime subsystems that should not be reopened as a row-shaped patch.

The branch evidence was collected at PTN `141c879c8f43`. When integrated on
current `master`, the class-declaration aggregate had already been split into
trait, interface, implementation-check, and anonymous-class buckets. The table
below is reconciled to those current bucket names without changing the row
totals or the blocker conclusion.

## Broad 1k Evidence

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-kmw3-baseline-rebased
```

Generated broad manifest:

```text
.runtime/ptn-kmw3-baseline-rebased/20260614T104145Z/phpt-baseline-1000.txt
```

Classifier artifacts:

```text
.runtime/phpt-progress/summary-20260614T104145Z.txt
.runtime/phpt-progress/classification-20260614T104145Z.tsv
.runtime/phpt-progress/runnable-20260614T104145Z.txt
.runtime/phpt-progress/excluded-20260614T104145Z.tsv
```

State:

```text
PTN: 141c879c8f43
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
| `unsupported-attribute-metadata` | 149 |
| `unsupported-magic-method-metadata` | 69 |
| `unsupported-call-unpacking` | 34 |
| `unsupported-request-input-ini` | 28 |
| `unsupported-trait-declaration` | 25 |
| `unsupported-interface-declaration` | 23 |
| `unsupported-extension` | 20 |
| `unsupported-property-visibility-metadata` | 19 |
| `unsupported-diagnostics-runtime` | 17 |
| `unsupported-assertion-ini` | 17 |
| `unsupported-interface-implementation` | 15 |
| `unsupported-anonymous-class` | 15 |
| `unsupported-resource-limit-ini` | 15 |
| `unsupported-type-hint` | 14 |
| `sapi-behavior` | 13 |
| `unsupported-typed-property-metadata` | 12 |
| `unsupported-function-state` | 11 |
| `unsupported-class-contract-metadata` | 9 |
| `unsupported-autoload-metadata` | 9 |
| `unsupported-assertion-runtime` | 9 |
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
| Attribute metadata | 149 | PHP `#[...]` parser metadata, target validation, reflection, repeatability, and internal attribute metadata. |
| Class/object metadata | 213 | Class declarations, magic methods, property and method visibility, typed and readonly slots, contracts, autoload, and internal reflection metadata. |
| Language and dynamic call surfaces | 68 | Call-site/array unpacking, nullable type hints, function static state, variable variables, and generator lowering. |
| Diagnostics/assertion state | 48 | ErrorException/trace metadata, assertion runtime modes, assertion INI behavior, and diagnostic INI surfaces. |
| Runtime boundary and environment | 98 | Request/SAPI state, unavailable extensions, resource limits, harness cleanup/preconditions, process boundaries, and host/environment assumptions. |

The large groups above are real compatibility targets, but each crosses several
compiler/runtime contracts. Treating any of them as a single broad-slice patch
would either overclaim support or mix unrelated semantics.

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
| `array_diff*` | 39 | Mostly covered by set-operation manifests; residuals are comparator arity, nested value warnings, includes, and heredoc/string edges. |
| `array_chunk()` | 32 | Already green in focused broad evidence; not a new implementation target. |
| `array_intersect*` | 30 | Same set-operation frontier shape as `array_diff*`. |
| Other array helpers | 21 | Mixed helper cases below the threshold once split by primitive. |
| `array_map()` | 19 | Callback dispatch, arity diagnostics, object callables, reference behavior, and zip semantics. |
| `array_key*` | 19 | Key coercion, resource/object diagnostics, and warning parity. |
| `array_merge*` | 18 | Recursive merge, references, reindexing, and ordered-array mutation. |
| `array_sum()` | 12 | Already mostly green in focused evidence; residuals are not a broad patch. |
| `array_slice()` | 10 | Ordered-array slicing and key preservation edges. |
| `array_filter()` | 10 | Callback mode and key/value argument shape. |
| `array_fill*` | 8 | Allocation/resource-limit and key/value conversion edges. |
| `array_rand()` | 7 | Coherent helper target, but below the broad threshold alone. |

## Focused Manifest Reconciliation

Command:

```sh
awk 'NF && $1 !~ /^#/ {print $1}' tools/phpt-*-manifest.txt \
  | LC_ALL=C sort -u > .runtime/ptn-kmw3-committed-focused-rows-rebased.txt
LC_ALL=C sort -u .runtime/phpt-progress/runnable-20260614T104145Z.txt \
  > .runtime/ptn-kmw3-current-runnable-rebased.txt
comm -23 .runtime/ptn-kmw3-current-runnable-rebased.txt \
  .runtime/ptn-kmw3-committed-focused-rows-rebased.txt \
  > .runtime/ptn-kmw3-unmatched-runnable-rebased.txt
wc -l .runtime/ptn-kmw3-current-runnable-rebased.txt \
  .runtime/ptn-kmw3-committed-focused-rows-rebased.txt \
  .runtime/ptn-kmw3-unmatched-runnable-rebased.txt
```

Result:

```text
  424 .runtime/ptn-kmw3-current-runnable-rebased.txt
 1655 .runtime/ptn-kmw3-committed-focused-rows-rebased.txt
    0 .runtime/ptn-kmw3-unmatched-runnable-rebased.txt
 2079 total
```

There are zero current broad-runnable rows outside committed focused manifests.

## Blocker Boundary

No fresh broad cluster in this 1k slice has a credible single generic change
that newly moves at least 25 rows:

1. The runnable 294-row standard-array surface is already represented by
   focused manifests. Its residual failures split across key/value conversion,
   callback dispatch, ordered mutation/reference behavior, random key
   selection, and comparator diagnostics.
2. The only named standard-array family above 25 rows that is not a mixed
   set-operation surface, `array_chunk()`, is already green in focused
   evidence.
3. Class/object and attribute metadata are high-yield but require class-table,
   parser, reflection, visibility, magic dispatch, and autoload work. Those
   should be sequenced as focused metadata features, not one broad patch.
4. Runtime-boundary rows are request/SAPI, extension, process, resource-limit,
   and environment surfaces; they should stay classified until PTN has native
   runtime boundaries for them.

The next productive implementation work should start from one of the committed
focused frontiers: standard-array residuals, class/object metadata, diagnostics
and assertion state, Zend operator/control, or runtime-boundary state.

## Verification

```sh
cargo fmt --check
cargo test --test phpt_classifier
```

The broad classify-only artifacts listed above are branch-collected evidence.
This integration is documentation-only and was checked on current `master` with
`cargo fmt --check`, `cargo test --test phpt_classifier`, and repository diff
sanity checks.
