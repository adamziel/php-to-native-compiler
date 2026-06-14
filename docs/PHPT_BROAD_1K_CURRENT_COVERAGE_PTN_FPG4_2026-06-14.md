# PHPT Broad 1k Current Coverage Map: 2026-06-14 ptn-fpg4

Issue: `ptn-fpg4`

This slice refreshes the broad 1k PHPT classifier on current `origin/master`
and reconciles the runnable rows against the focused manifests already
committed in-tree. It is a blocker map and coverage check, not a runtime
implementation claim.

The current broad 1k set has no unmapped runnable cluster large enough for a
credible one-patch semantic implementation. The remaining large row groups are
excluded by generic compiler/runtime boundaries such as class metadata,
attribute metadata, request/SAPI state, diagnostics state, and unsupported
language constructs.

## Broad 1k Evidence

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-b35n-baseline-post
```

Generated broad manifest:

```text
.runtime/ptn-b35n-baseline-post/20260614T090208Z/phpt-baseline-1000.txt
```

Artifacts:

```text
.runtime/phpt-progress/classification-20260614T090208Z.tsv
.runtime/phpt-progress/runnable-20260614T090208Z.txt
.runtime/phpt-progress/excluded-20260614T090208Z.tsv
.runtime/phpt-progress/summary-20260614T090208Z.txt
```

State:

```text
PTN artifact commit: ee63f3764a8a
php-src PHPT corpus: /home/claude/php-src-phpt
corpus revision: 8c63ec400ce8e07c57a8d9499317b96a8beafb8b
```

Classifier result:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 1,000 | 424 | 576 |

Excluded buckets:

| Classification | Rows |
| --- | ---: |
| `unsupported-attribute-metadata` | 149 |
| `unsupported-language` | 147 |
| `unsupported-class-metadata` | 135 |
| `unsupported-request-input-ini` | 28 |
| `unsupported-extension` | 20 |
| `unsupported-diagnostics-runtime` | 17 |
| `unsupported-assertion-ini` | 17 |
| `unsupported-resource-limit-ini` | 15 |
| `sapi-behavior` | 13 |
| `unsupported-assertion-runtime` | 9 |
| `unsupported-diagnostics-ini` | 5 |
| `harness-cleanup` | 4 |
| `process-boundary` | 3 |
| `skipif-precondition` | 2 |
| `unsupported-function-disable-ini` | 2 |
| `unsupported-host-path-ini` | 2 |
| `unsupported-opcache-ini` | 2 |
| `unsupported-scalar-format-ini` | 2 |
| `environment-assumption` | 1 |
| `external-service` | 1 |
| `unsupported-internal` | 1 |
| `unsupported-resource-limit` | 1 |

`Zend/tests/ErrorException_getSeverity.phpt`, previously called out as a
residual runnable diagnostics row in an older map, now classifies as:

```text
unsupported-diagnostics-runtime
requires ErrorException severity and trace metadata, outside PTN modeled built-in exception values
```

## Runnable Coverage Check

The current broad-runnable rows were compared with committed focused manifests:

```sh
tmpdir=.runtime/ptn-fpg4-analysis-current-check-090208
mkdir -p "$tmpdir"
awk 'NF && $1 !~ /^#/ {print $1}' tools/phpt-*-manifest.txt \
  | LC_ALL=C sort -u > "$tmpdir/committed-focused-rows.txt"
LC_ALL=C sort -u .runtime/phpt-progress/runnable-20260614T090208Z.txt \
  > "$tmpdir/current-runnable.txt"
comm -23 "$tmpdir/current-runnable.txt" "$tmpdir/committed-focused-rows.txt" \
  > "$tmpdir/unmatched-runnable.txt"
wc -l "$tmpdir/current-runnable.txt" \
  "$tmpdir/committed-focused-rows.txt" \
  "$tmpdir/unmatched-runnable.txt"
```

Result:

```text
  424 .runtime/ptn-fpg4-analysis-current-check-090208/current-runnable.txt
 1606 .runtime/ptn-fpg4-analysis-current-check-090208/committed-focused-rows.txt
    0 .runtime/ptn-fpg4-analysis-current-check-090208/unmatched-runnable.txt
```

The broad 1k runnable rows split by source family as:

| Family | Rows | Existing focused coverage |
| --- | ---: | --- |
| `ext/standard/tests/array/*` | 294 | Standard-array, array callback, array key/value, array set-operation, and array residual manifests. |
| root `Zend/tests/*.phpt` | 81 | Zend assignment/reference, historical bug-regression, diagnostics, recursive dump, and operator/control manifests. |
| `Zend/tests/asymmetric_visibility/*` | 22 | Asymmetric visibility frontier manifest. |
| `tests/basic/*` | 16 | Core/basic operator, runtime configuration, environment, and request/SAPI maps. |
| `Zend/tests/ast/*` | 4 | Zend operator/control frontier manifest. |
| `Zend/tests/arrow_functions/*` | 3 | Diagnostics/assertion and language/runtime maps. |
| `Zend/tests/assert/*` | 2 | Assertion runtime frontier manifest. |
| `Zend/tests/access_modifiers/*` | 1 | Class/object metadata frontier maps. |
| `Zend/tests/attributes/*` | 1 | Attribute metadata frontier maps. |

There are zero current broad-runnable rows outside committed focused manifests.

## Large Blockers

The current `unsupported-language` bucket is already smaller after the
attribute split. Its remaining reasons are still generic compiler features:

| Blocker | Rows |
| --- | ---: |
| Call-site or array unpacking (`...`) | 34 |
| Trait declarations | 25 |
| Interface declarations | 23 |
| Interface implementation checks | 15 |
| Anonymous class syntax (`new class`) | 15 |
| Nullable type-hint metadata and coercion (`?T`) | 14 |
| Static local variables | 11 |
| Variable variables and runtime symbol-table mutation | 8 |
| Named-argument binding for modeled array internals | 1 |
| Generator/yield lowering | 1 |

The current `unsupported-class-metadata` rows are also split across distinct
metadata systems:

| Blocker | Rows |
| --- | ---: |
| Magic method dispatch/reflection metadata | 69 |
| Non-public property visibility metadata | 19 |
| Typed property metadata | 12 |
| Runtime class autoload symbol-table mutation | 9 |
| Non-public method visibility dispatch and diagnostics | 7 |
| Indirect readonly property mutation diagnostics | 7 |
| Abstract class/method contract metadata | 5 |
| Final class/method override metadata | 4 |
| Complete internal arginfo/class registry reflection | 3 |

The explicit attribute bucket covers 149 rows:

| Blocker | Rows |
| --- | ---: |
| PHP attribute syntax (`#[...]`) and reflection metadata | 141 |
| Internal attribute/reflection metadata | 8 |

The request/SAPI bucket is a separate runtime boundary:

| Blocker | Rows |
| --- | ---: |
| `enable_post_data_reading` request state | 7 |
| `register_argc_argv` request state | 6 |
| `file_uploads` request state | 6 |
| `max_input_vars` request state | 4 |
| `variables_order` request state | 2 |
| `always_populate_raw_post_data` request state | 1 |
| `max_input_nesting_level` request state | 1 |
| `post_max_size` request state | 1 |

The next productive implementation slice should therefore use one of the
existing focused manifests as the starting point instead of reopening the broad
1k selection as a new ad hoc cluster. Standard-array residuals, class/object
metadata, runtime diagnostics, and request/SAPI state are all meaningful
frontiers, but each requires a dedicated generic design rather than a row-level
patch.

## Verification

```sh
cargo fmt --check
cargo test --test phpt_classifier
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-b35n-baseline-post
```
