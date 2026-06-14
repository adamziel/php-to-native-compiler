# PHPT Broad 1k Cluster Slice: 2026-06-14 ptn-cgfh

Issue: `ptn-cgfh`

This slice refreshes the broad PHPT 1k classifier on current `origin/master`
and checks for one high-yield semantic cluster that can credibly move at least
25 broad rows. It is a blocker map, not a behavior change: the current broad
runnable set is already fully covered by committed focused manifests, while the
excluded high-count buckets require multi-surface compiler/runtime work.

## Broad 1k Evidence

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-h0qa-current-baseline
```

Generated broad manifests:

```text
.runtime/ptn-h0qa-current-baseline/20260614T125733Z/phpt-baseline-1000.txt
.runtime/ptn-h0qa-current-baseline/20260614T125733Z/phpt-baseline-5000.txt
.runtime/ptn-h0qa-current-baseline/20260614T125733Z/phpt-baseline-10000.txt
```

Classifier artifacts:

```text
.runtime/ptn-h0qa-current-progress/summary-20260614T125733Z.txt
.runtime/ptn-h0qa-current-progress/classification-20260614T125733Z.tsv
.runtime/ptn-h0qa-current-progress/runnable-20260614T125733Z.txt
.runtime/ptn-h0qa-current-progress/excluded-20260614T125733Z.tsv
```

State:

```text
PTN: 24318afd2014
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
| `unsupported-magic-method-metadata` | 8 |
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
| Attribute metadata | 149 | PHP `#[...]` syntax plus declaration/reflection metadata and internal attribute objects. |
| Class/object metadata | 213 | Object-string conversion, residual magic methods, trait/interface/class declarations, visibility, typed/readonly slots, contracts, autoload, and internal reflection metadata. |
| Language and dynamic call surfaces | 69 | Call/array unpacking, nullable/never types, function static state, variable variables, generator runtime, and internal named-argument binding. |
| Diagnostics/assertion state | 48 | `ErrorException`/trace metadata, assertion runtime modes, assertion INI behavior, and diagnostic INI channels. |
| Runtime, environment, and array-internal boundaries | 97 | Request/SAPI state, unavailable extensions, resource limits, process and harness boundaries, host/environment assumptions, and one array-internal runtime blocker. |

The largest single high-yield bucket is attribute syntax metadata at 141 rows,
but implementing it generically requires parser support, declaration metadata,
target validation, reflection metadata, repeatability semantics, and internal
attribute class behavior. The combined 69-row object-string and residual
magic-method metadata surface similarly crosses method lookup, visibility,
object storage, reflection, and dispatch semantics.

The narrowest excluded bucket at the target threshold is
`unsupported-trait-declaration` with exactly 25 rows. It is not a credible
single patch because the rows cover trait declarations, composition, aliases,
precedence, conflict diagnostics, attribute override metadata, backtrace class
metadata, and one basic engine regression. Treating it as one PHPT row patch
would overclaim trait support.

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

The standard-array surface remains the only runnable family above the 25-row
target:

| Standard-array family | Rows | Current boundary |
| --- | ---: | --- |
| Other array helpers | 83 | Mixed key/value, search, flip, combine, column, stack/queue, reduce, replace, reverse, product, pad, splice, and scalar-helper edges. |
| `array_diff*` / `array_udiff*` | 43 | Mostly represented by set-operation manifests; residuals are comparator arity, nested value warnings, includes, and string/parser edges. |
| `array_intersect*` / `array_uintersect*` | 33 | Same set-operation frontier shape as diff helpers. |
| `array_chunk()` | 32 | Already green in focused broad evidence; not a new implementation target. |
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
  | LC_ALL=C sort -u > .runtime/ptn-cgfh-focused-rows.txt
LC_ALL=C sort -u .runtime/ptn-h0qa-current-progress/runnable-20260614T125733Z.txt \
  > .runtime/ptn-cgfh-runnable-rows.txt
comm -23 .runtime/ptn-cgfh-runnable-rows.txt \
  .runtime/ptn-cgfh-focused-rows.txt \
  > .runtime/ptn-cgfh-unmatched-runnable.txt
wc -l .runtime/ptn-cgfh-runnable-rows.txt \
  .runtime/ptn-cgfh-focused-rows.txt \
  .runtime/ptn-cgfh-unmatched-runnable.txt
```

Result:

```text
  424 .runtime/ptn-cgfh-runnable-rows.txt
 1656 .runtime/ptn-cgfh-focused-rows.txt
    0 .runtime/ptn-cgfh-unmatched-runnable.txt
 2080 total
```

There are zero current broad-runnable rows outside committed focused manifests.

## Blocker Boundary

No fresh broad cluster in this 1k slice has a credible single generic change
that newly moves at least 25 rows:

1. The broad runnable frontier is already covered by committed focused
   manifests, so the high-count standard-array source family is a regression
   target rather than a new untracked implementation cluster.
2. The standard-array residuals are split across key/value conversion,
   callback dispatch, ordered mutation/reference behavior, random key
   selection, and comparator diagnostics.
3. The excluded buckets above or near 25 rows are real compatibility areas but
   cross parser, metadata, runtime, reflection, diagnostics, and SAPI
   boundaries.
4. Runtime-boundary rows should remain classified until PTN has native request,
   process, resource-limit, extension, and environment state boundaries.

The next credible implementation work should start from one focused frontier:
standard-array residuals, call/array unpacking lowering, trait/interface
metadata, magic-method dispatch metadata, diagnostics/assertion state, or
request/SAPI runtime state. Each should carry its own focused manifest and
generic semantic tests before broad rows are reclassified as runnable.

## Verification

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-h0qa-current-baseline
cargo fmt --check
cargo test --test phpt_classifier
```
