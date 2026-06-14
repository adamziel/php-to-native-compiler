# PHPT Broad 1k Cluster Slice: 2026-06-14 ptn-73z5

Issue: `ptn-73z5`

This slice refreshes the broad PHPT 1k classifier on current `master` and
checks for one semantic cluster that can credibly move at least 25 broad rows.
It is a blocker map, not a runtime support claim. The refreshed data matches
the recent broad maps: every runnable broad row is already represented by a
committed focused manifest, while every 25+ excluded group is blocked by a
multi-contract parser/runtime feature.

## Broad 1k Evidence

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-h0qa-current-baseline
```

Generated broad manifest:

```text
.runtime/ptn-h0qa-current-baseline/20260614T125733Z/phpt-baseline-1000.txt
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

## Classifier Buckets

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
| `unsupported-internal-attribute-metadata` | 8 |
| `unsupported-readonly-property-metadata` | 7 |
| `unsupported-method-visibility-metadata` | 7 |
| `unsupported-diagnostics-ini` | 5 |
| `harness-cleanup` | 4 |
| `unsupported-internal-reflection-metadata` | 3 |
| `process-boundary` | 3 |
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

Grouped by implementation ownership, the excluded rows are:

| Owner | Rows | Boundary |
| --- | ---: | --- |
| Attribute metadata | 149 | PHP `#[...]` parser syntax, declaration metadata, internal attributes, target validation, repeatability, and reflection APIs. |
| Language and dynamic call surfaces | 147 | Call-site and array unpacking, traits, interfaces, anonymous classes, nullable/never type hints, static locals, variable variables, generators, and named internal-call binding. |
| Class/object metadata | 135 | Object-string conversion, residual magic methods, property/method visibility, typed and readonly slots, abstract/final contracts, autoload, and complete internal reflection metadata. |
| Diagnostics/assertion state | 48 | ErrorException/trace metadata, user handler state, debug backtrace APIs, assertion INI modes, and assertion runtime options. |
| Runtime boundary and environment | 97 | Request/SAPI state, unavailable extensions, resource limits, harness cleanup/preconditions, process boundaries, OPcache/function-disable/host INI, and external environment assumptions. |

## Runnable Coverage

The 424 runnable rows split by source family as:

| Family | Rows |
| --- | ---: |
| `ext/standard/tests/array/*` | 294 |
| Root-level `Zend/tests/*.phpt` | 81 |
| `Zend/tests/asymmetric_visibility/*` | 22 |
| `tests/basic/*` | 16 |
| `Zend/tests/ast/*` | 4 |
| `Zend/tests/arrow_functions/*` | 3 |
| `Zend/tests/assert/*` | 2 |
| `Zend/tests/access_modifiers/*` | 1 |
| `Zend/tests/attributes/*` | 1 |

The standard-array runnable family remains the only runnable source family over
the 25-row threshold:

| Standard-array family | Rows | Current boundary |
| --- | ---: | --- |
| Other array helpers | 90 | Already distributed across committed focused manifests; residuals are mixed helper semantics. |
| `array_diff*` | 39 | Covered by set-operation/diff-intersect manifests; remaining gaps are comparator, nested value, include, and string/parser edges. |
| `array_chunk()` | 32 | Already covered by focused green evidence, not a new broad implementation target. |
| `array_intersect*` | 30 | Same set-operation frontier as `array_diff*`. |
| `array_map()` | 19 | Below threshold once split by callback dispatch and arity/reference semantics. |
| `array_key*` | 19 | Below threshold once split by key coercion and invalid operand diagnostics. |
| `array_merge*` | 18 | Below threshold once split by recursive merge, references, reindexing, and ordered mutation. |
| `array_sum()` | 12 | Mostly covered by focused array arithmetic evidence. |
| `array_slice()` | 10 | Ordered slicing and key-preservation edges. |
| `array_filter()` | 10 | Callback mode and key/value argument shape. |
| `array_fill*` | 8 | Allocation/resource-limit and conversion edges. |
| `array_rand()` | 7 | Coherent helper target, but below the broad target alone. |

Focused-manifest reconciliation:

```sh
tmpdir=.runtime/ptn-73z5-analysis
mkdir -p "$tmpdir"
awk 'NF && $1 !~ /^#/ {print $1}' tools/phpt-*-manifest.txt \
  | LC_ALL=C sort -u > "$tmpdir/committed-focused-rows.txt"
LC_ALL=C sort -u .runtime/ptn-h0qa-current-progress/runnable-20260614T125733Z.txt \
  > "$tmpdir/current-runnable.txt"
comm -23 "$tmpdir/current-runnable.txt" \
  "$tmpdir/committed-focused-rows.txt" \
  > "$tmpdir/unmatched-runnable.txt"
wc -l "$tmpdir/current-runnable.txt" \
  "$tmpdir/committed-focused-rows.txt" \
  "$tmpdir/unmatched-runnable.txt"
```

Result:

```text
  424 .runtime/ptn-73z5-analysis/current-runnable.txt
 1656 .runtime/ptn-73z5-analysis/committed-focused-rows.txt
    0 .runtime/ptn-73z5-analysis/unmatched-runnable.txt
 2080 total
```

There are zero current broad-runnable rows outside committed focused manifests.

## Blocker Boundary

No fresh broad 1k cluster in this slice has a credible one-patch generic change
that newly moves at least 25 rows:

1. The runnable 294-row standard-array family is already covered by focused
   manifests. The residual families split below the threshold or across
   unrelated primitives.
2. The 34-row unpacking bucket spans function/method/new/internal call
   argument unpacking, array literal unpacking, destructuring, key validation,
   traversables, by-reference separation, and compile-time diagnostics. It is a
   real future feature, but not a safe broad-row patch.
3. The 25-row trait bucket reaches the threshold exactly, but generic support
   requires trait declarations, class-table registration, composition,
   alias/precedence rules, conflict diagnostics, and interaction with interface
   and visibility metadata.
4. Attribute, class/object metadata, diagnostics/assertion state, and
   runtime-boundary groups are all larger than 25 rows, but each crosses
   multiple compiler/runtime contracts and should be sequenced as focused
   metadata or runtime-boundary features.

The next productive implementation slices should start from focused manifests:
call/array unpacking, trait composition, magic method metadata, attribute
metadata, runtime diagnostics/assertion state, or request/SAPI state.

## Verification

```sh
cargo fmt --check
cargo test --test phpt_classifier
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-h0qa-current-baseline
```
