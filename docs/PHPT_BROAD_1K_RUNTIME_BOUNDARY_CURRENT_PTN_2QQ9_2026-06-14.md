# PHPT Broad 1k Runtime Boundary Current Map: 2026-06-14 ptn-2qq9

Issue: `ptn-2qq9`

This slice refreshes the broad 1k PHPT classifier on `origin/master` after the
language and class-metadata bucket splits, then records the
runtime/configuration/diagnostics boundary as a focused 144-row manifest. It is
a blocker map, not a runtime implementation claim. The class-declaration bucket
names below have been reconciled with the later `ptn-gkvr` split.

The selected rows are excluded for process-global or host/runtime boundaries:
request/SAPI state, extension availability, assertion and diagnostic runtime
state, INI-controlled behavior, resource limits, process execution,
environment assumptions, and PHPT harness cleanup. These do not form one
credible small implementation patch; reopening them safely needs generic
runtime state and diagnostics architecture.

## Broad 1k Evidence

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-2qq9-postrebase-baseline-1k
```

Generated broad manifest:

```text
.runtime/ptn-2qq9-postrebase-baseline-1k/20260614T103801Z/phpt-baseline-1000.txt
```

Classifier artifacts:

```text
.runtime/phpt-progress/summary-20260614T103801Z.txt
.runtime/phpt-progress/classification-20260614T103801Z.tsv
.runtime/phpt-progress/runnable-20260614T103801Z.txt
.runtime/phpt-progress/excluded-20260614T103801Z.tsv
```

State:

```text
PTN: f2a73c767658
php-src PHPT corpus revision: 8c63ec400ce8e07c57a8d9499317b96a8beafb8b
```

Broad result:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 1,000 | 424 | 576 |

Largest current excluded buckets:

| Classification | Rows |
| --- | ---: |
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
| `unsupported-resource-limit-ini` | 15 |
| `unsupported-interface-implementation` | 15 |
| `unsupported-anonymous-class` | 15 |

## Focused Runtime Boundary Manifest

Committed manifest:

```text
tools/phpt-runtime-boundary-current-ptn-2qq9-manifest.txt
```

It was selected from:

```text
.runtime/phpt-progress/classification-20260614T103801Z.tsv
```

Focused verification:

```sh
PHPT_PROGRESS_DIR=.runtime/ptn-2qq9-runtime-boundary-focused \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-runtime-boundary-current-ptn-2qq9-manifest.txt
```

Focused artifact:

```text
.runtime/ptn-2qq9-runtime-boundary-focused/classification-20260614T104355Z.tsv
```

Focused result:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 144 | 0 | 144 |

## Category Split

Focused classifier split:

| Classification | Rows |
| --- | ---: |
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
| `unsupported-function-disable-ini` | 2 |
| `unsupported-host-path-ini` | 2 |
| `unsupported-opcache-ini` | 2 |
| `unsupported-scalar-format-ini` | 2 |
| `skipif-precondition` | 2 |
| `environment-assumption` | 1 |
| `external-service` | 1 |
| `unsupported-resource-limit` | 1 |

Path split:

| Path group | Rows |
| --- | ---: |
| `tests/basic/` | 69 |
| `Zend/tests/assert/` | 25 |
| Root `Zend/tests/` | 19 |
| `Zend/tests/backtrace/` | 13 |
| `Zend/tests/attributes/` | 8 |
| `ext/standard/tests/array/` | 8 |
| `Zend/tests/asymmetric_visibility/` | 1 |
| Root `ext/standard/tests/` | 1 |

## Implementation Boundary

These rows should stay classified until the relevant generic runtime systems
exist:

- request and SAPI setup state, including argv, POST data, upload, and
  variables-order initialization;
- process-global INI state for assertion, diagnostics, resource limits,
  disabled functions, opcache flags, precision, and host-path behavior;
- stack-frame diagnostics for `debug_backtrace()`, `debug_print_backtrace()`,
  fatal backtraces, user handler state, and `ErrorException` metadata;
- assertion runtime state for `zend.assertions`, `assert_options()`,
  callbacks, bail modes, namespace resolution, and AST rendering;
- extension and host availability boundaries, including opcache/FFI and
  external-service assumptions;
- child-process and cleanup harness behavior that PTN native CLI does not yet
  model as reusable runtime semantics.

The current broad runnable set remains at 424 rows and is already covered by
committed focused manifests. This 144-row runtime boundary is above the 25-row
slice threshold, but it is intentionally a blocker map: the work spans several
runtime subsystems and should be split into dedicated generic implementation
tasks.

## Verification

```sh
cargo fmt --check
cargo test --test phpt_classifier
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-2qq9-postrebase-baseline-1k
PHPT_PROGRESS_DIR=.runtime/ptn-2qq9-runtime-boundary-focused \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-runtime-boundary-current-ptn-2qq9-manifest.txt
```
