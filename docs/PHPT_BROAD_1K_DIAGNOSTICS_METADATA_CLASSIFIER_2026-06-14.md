# PHPT Broad 1k Diagnostics/Metadata Classifier: 2026-06-14

Issue: `ptn-1f0f`

This slice keeps broad PHPT telemetry focused on semantics PTN currently
models. It was rebased over `ptn-lrlt`, which had already classified runtime
diagnostic and assertion-mode rows. The remaining broad 1k runnable set still
contained rows that require complete internal arginfo/class registries,
internal attribute metadata, or exception trace APIs. Those surfaces are not
implemented by PTN's runtime model yet, so the rows produced runnable noise
instead of useful native compatibility signal.

The classifier change is semantic rather than row-specific. It recognizes:

- internal attribute metadata: `Reflection*::getAttributes()`,
  `Attribute::*`, `Deprecated`, and `NoDiscard`;
- complete internal registry reflection through `get_defined_functions()`,
  `get_declared_classes()`, and
  `ReflectionClass::newInstanceWithoutConstructor()`;
- exception trace APIs such as `Exception::getTraceAsString()`.

Together with `ptn-lrlt`, the rebased classifier also recognizes stack trace
APIs, user error-handler state, `ErrorException` metadata, and runtime
assertion options.

## Broad 1k Evidence

The same generated manifest was used before and after:

```text
.runtime/ptn-1f0f-baseline-before/20260614T004957Z/phpt-baseline-1000.txt
```

Corpus revision:

```text
8c63ec400ce8e07c57a8d9499317b96a8beafb8b
```

Before:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-1f0f-baseline-before
```

Artifacts:

```text
.runtime/phpt-progress/classification-20260614T004958Z.tsv
.runtime/phpt-progress/runnable-20260614T004958Z.txt
.runtime/phpt-progress/summary-20260614T004958Z.txt
```

After:

```sh
tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  .runtime/ptn-1f0f-baseline-before/20260614T004957Z/phpt-baseline-1000.txt
```

Artifacts:

```text
.runtime/phpt-progress/classification-20260614T011616Z.tsv
.runtime/phpt-progress/runnable-20260614T011616Z.txt
.runtime/phpt-progress/summary-20260614T011616Z.txt
```

## Before/After Counts

| Run | Selected | Runnable | Excluded |
| --- | ---: | ---: | ---: |
| Original ptn-1f0f before | 1,000 | 447 | 553 |
| After upstream `ptn-lrlt` | 1,000 | 421 | 579 |
| After rebased `ptn-1f0f` | 1,000 | 409 | 591 |

Net movement from the original ptn-1f0f baseline: 38 broad 1k rows newly
classified. Incremental movement from this rebased branch on top of
`ptn-lrlt`: 12 rows.

Incremental `ptn-1f0f` buckets:

| Category | Newly classified rows |
| --- | ---: |
| `unsupported-class-metadata` | 11 |
| `unsupported-diagnostics-runtime` | 1 |

Final cumulative runtime buckets from the original ptn-1f0f baseline:

| Category | Newly classified rows |
| --- | ---: |
| `unsupported-diagnostics-runtime` | 18 |
| `unsupported-class-metadata` | 11 |
| `unsupported-assertion-runtime` | 9 |

## Incremental Rows

```text
Zend/tests/arginfo_zpp_mismatch.phpt
Zend/tests/arginfo_zpp_mismatch_strict.phpt
Zend/tests/attributes/007_self_reflect_attribute.phpt
Zend/tests/attributes/029_reflect_internal_symbols.phpt
Zend/tests/attributes/034_target_values.phpt
Zend/tests/attributes/deprecated/property_readonly_001.phpt
Zend/tests/attributes/deprecated/property_readonly_002.phpt
Zend/tests/attributes/deprecated/property_readonly_003.phpt
Zend/tests/attributes/nodiscard/property_readonly_001.phpt
Zend/tests/attributes/nodiscard/property_readonly_002.phpt
Zend/tests/backtrace/bug76047.phpt
Zend/tests/bug18556.phpt
```

## Implementation Boundary

These rows should be reopened by removing the classifier branches only after
the relevant generic semantics land:

1. Stack frame storage with function, class, method type, include, `$this`,
   argument snapshot, and limit/flag behavior for `debug_backtrace()` and
   `debug_print_backtrace()`.
2. User error-handler state and fallback behavior, including warnings promoted
   by handlers and handler exceptions.
3. Exception and `ErrorException` metadata for severity, file, line, previous,
   code, and trace string APIs.
4. Assertion runtime state for `zend.assertions`, `assert_options()`,
   `ASSERT_BAIL`, and `ASSERT_CALLBACK`, while preserving the current direct
   `AssertionError` path.
5. Internal function/class registry and arginfo reflection metadata.
6. Internal attribute classes/constants and Reflection attribute metadata.

Invalid `break` diagnostics and supported asymmetric-visibility rows remain
runnable; those are inside PTN's currently modeled parser/control-flow and
property visibility surfaces.

## Verification

```sh
bash -n tools/phpt-classifier.sh
cargo test --test phpt_classifier
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-1f0f-baseline-before
tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  .runtime/ptn-1f0f-baseline-before/20260614T004957Z/phpt-baseline-1000.txt
```
