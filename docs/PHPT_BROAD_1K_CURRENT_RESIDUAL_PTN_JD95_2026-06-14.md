# PHPT Broad 1k Current Residual Map: 2026-06-14 ptn-jd95

Issue: `ptn-jd95`

This slice refreshes the broad 1k PHPT classifier on current `origin/master`
and reconciles the remaining runnable rows against the focused frontier
manifests already committed today. It is a blocker map and telemetry cleanup,
not a runtime implementation claim. The current runnable surface is already
covered by focused maps; forcing a new implementation here would either
duplicate prior work or cross several unrelated generic PHP runtime boundaries.

## Broad 1k Evidence

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-jd95-baseline
```

Generated broad manifest:

```text
.runtime/ptn-jd95-baseline/20260614T075645Z/phpt-baseline-1000.txt
```

Artifacts:

```text
.runtime/phpt-progress/classification-20260614T075645Z.tsv
.runtime/phpt-progress/runnable-20260614T075645Z.txt
.runtime/phpt-progress/excluded-20260614T075645Z.tsv
.runtime/phpt-progress/summary-20260614T075645Z.txt
```

State:

```text
PTN: b4e0f62f98d1
php-src PHPT corpus: /home/claude/php-src-phpt
corpus revision: 8c63ec400ce8e07c57a8d9499317b96a8beafb8b
```

Classifier result as written by the broad wrapper:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 1,000 | 425 | 575 |

Top classifier buckets:

| Classification | Rows |
| --- | ---: |
| `unsupported-language` | 288 |
| `unsupported-class-metadata` | 143 |
| `unsupported-request-input-ini` | 28 |
| `unsupported-extension` | 20 |
| `unsupported-assertion-ini` | 17 |
| `unsupported-diagnostics-runtime` | 16 |
| `unsupported-resource-limit-ini` | 15 |
| `sapi-behavior` | 13 |
| `unsupported-assertion-runtime` | 9 |

## Focused Residual

The broad runnable file left one diagnostics row runnable:

```text
Zend/tests/ErrorException_getSeverity.phpt
```

Directly invoking the classifier function on that row reports
`unsupported-diagnostics-runtime`, and a one-row bounded manifest does the same:

```sh
printf 'Zend/tests/ErrorException_getSeverity.phpt\n' \
  > .runtime/ptn-jd95-errorexception-row.txt
tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  .runtime/ptn-jd95-errorexception-row.txt
```

Artifact:

```text
.runtime/phpt-progress/classification-20260614T080929Z.tsv
```

Focused result:

| Selected | Runnable | Excluded | Classification |
| ---: | ---: | ---: | --- |
| 1 | 0 | 1 | `unsupported-diagnostics-runtime` |

The existing diagnostics/assertion manifest intentionally leaves this row out,
because adding it to that larger focused manifest still leaves the bounded
runner with one runnable row after rebase:

```sh
PHPT_PROGRESS_DIR=.runtime/ptn-jd95-diagnostics-serial \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-diagnostics-assertion-frontier-manifest.txt
```

For that validation, `tools/phpt-diagnostics-assertion-frontier-manifest.txt`
temporarily included `Zend/tests/ErrorException_getSeverity.phpt`.

Artifact:

```text
.runtime/ptn-jd95-diagnostics-serial/classification-20260614T081036Z.tsv
```

| Selected | Runnable | Excluded | `unsupported-diagnostics-runtime` |
| ---: | ---: | ---: | ---: |
| 48 | 1 | 47 | 16 |

The committed diagnostics/assertion manifest therefore remains the existing
47-row excluded set. `Zend/tests/ErrorException_getSeverity.phpt` is recorded
here as the one current broad runnable row outside committed focused manifests,
and it should be handled either by an `ErrorException` metadata implementation
or by a separate classifier-runner consistency fix.

```sh
tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-diagnostics-assertion-frontier-manifest.txt
```

Artifact:

```text
.runtime/phpt-progress/classification-20260614T063850Z.tsv
```

| Selected | Runnable | Excluded | `unsupported-diagnostics-runtime` |
| ---: | ---: | ---: | ---: |
| 47 | 0 | 47 | 16 |

## Runnable Family Map

The broad wrapper's 425 runnable rows split by source family as:

| Family | Rows | Current focused map |
| --- | ---: | --- |
| `ext/standard/tests/array/*` | 294 | `tools/phpt-broad-standard-array-frontier-manifest.txt` plus the current standard-array docs. |
| root-level `Zend/tests/*.phpt` | 82 | Covered by the Zend assignment/reference, historical bug-regression, operator/control, diagnostics, and related focused manifests. |
| `Zend/tests/asymmetric_visibility/*` | 22 | `tools/phpt-asymmetric-visibility-frontier-manifest.txt`. |
| `tests/basic/*` | 16 | `tools/phpt-core-basic-operator-frontier-manifest.txt` and runtime-config/environment maps. |
| `Zend/tests/ast/*` | 4 | `tools/phpt-zend-operator-control-frontier-manifest.txt`. |
| `Zend/tests/arrow_functions/*` | 3 | Diagnostics/assertion and language/runtime maps. |
| `Zend/tests/assert/*` | 2 | `tools/phpt-assertion-runtime-frontier-manifest.txt`. |
| `Zend/tests/attributes/*` | 1 | Attribute metadata frontier maps. |
| `Zend/tests/access_modifiers/*` | 1 | Class/object metadata frontier maps. |

After subtracting committed `tools/phpt-*-manifest.txt` rows from the broad
runnable file, the only unmatched row is
`Zend/tests/ErrorException_getSeverity.phpt`. There is no remaining unmapped
broad runnable cluster with a credible one-patch 25-row movement.

## Blocker Boundary

The next productive implementation work should choose one existing focused
frontier rather than invent a new broad cluster:

1. Standard-array residuals: key/value conversion, callback diagnostics,
   ordered-array mutation/reference behavior, `array_rand()`, and user
   comparator errors remain separate runtime primitives.
2. Zend root rows: assignment/lvalue, historical bug-regression, operator/
   control-flow, shutdown callbacks, diagnostics, and object dispatch are
   already split into smaller focused maps.
3. Diagnostics/assertion rows: stack traces, user handlers, `ErrorException`
   severity/trace fields, assertion runtime state, and diagnostics INI modes
   need shared runtime services.
4. Class/object metadata rows: asymmetric visibility, access modifiers,
   attributes, magic methods, and reflection metadata should stay classified
   until those generic metadata systems exist.

## Verification

```sh
cargo fmt --check
cargo test --test phpt_classifier
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-jd95-baseline
printf 'Zend/tests/ErrorException_getSeverity.phpt\n' \
  > .runtime/ptn-jd95-errorexception-row.txt
tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  .runtime/ptn-jd95-errorexception-row.txt
tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-diagnostics-assertion-frontier-manifest.txt
```
