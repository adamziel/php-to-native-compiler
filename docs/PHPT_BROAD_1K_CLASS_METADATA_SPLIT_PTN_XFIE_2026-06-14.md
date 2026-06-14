# PHPT Broad 1k Class Metadata Classifier Split: 2026-06-14

Issue: `ptn-xfie`

This slice uses the broad PHPT baseline tooling on `origin/master` and splits
the previous broad `unsupported-class-metadata` bucket into semantic
class/object metadata categories. This is not a runtime support claim: all
affected rows remain excluded, but 135 broad 1k rows are now classified with a
more precise blocker category.

## Evidence

Before command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only
```

Before artifacts:

```text
.runtime/phpt-progress/classification-20260614T095743Z.tsv
.runtime/phpt-progress/excluded-20260614T095743Z.tsv
.runtime/phpt-progress/excluded-20260614T095743Z/unsupported-class-metadata.txt
```

Before broad result:

| Selected | Runnable | Excluded | `unsupported-class-metadata` |
| ---: | ---: | ---: | ---: |
| 1000 | 424 | 576 | 135 |

After command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-xfie-broad-current
```

After artifacts:

```text
.runtime/ptn-xfie-broad-current/20260614T111502Z/phpt-baseline-1000.txt
.runtime/phpt-progress/classification-20260614T111502Z.tsv
.runtime/phpt-progress/excluded-20260614T111502Z.tsv
```

After broad result:

| Selected | Runnable | Excluded | `unsupported-class-metadata` |
| ---: | ---: | ---: | ---: |
| 1000 | 424 | 576 | 0 |

The selected corpus revision was
`8c63ec400ce8e07c57a8d9499317b96a8beafb8b` for both runs.

## Focused Replay

Committed focused manifest:
`tools/phpt-broad-1k-class-metadata-split-ptn-xfie-manifest.txt`

The manifest is the exact 135-row set that previously classified as
`unsupported-class-metadata`.

```sh
PHPT_PROGRESS_DIR=.runtime/ptn-xfie-class-metadata-focused-final \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-broad-1k-class-metadata-split-ptn-xfie-manifest.txt
```

Focused artifacts:

```text
.runtime/ptn-xfie-class-metadata-focused-final/classification-20260614T112909Z.tsv
.runtime/ptn-xfie-class-metadata-focused-final/excluded-20260614T112909Z.tsv
```

Focused result:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 135 | 0 | 135 |

Current replay after the later object-string classifier split wrote:

```text
.runtime/ptn-xfie-class-metadata-current/classification-20260614T124927Z.tsv
```

with the same 135 selected, 0 runnable, and 135 excluded result.

## New Buckets

| Bucket | Rows |
| --- | ---: |
| `unsupported-object-string-conversion-metadata` | 61 |
| `unsupported-property-visibility-metadata` | 19 |
| `unsupported-typed-property-metadata` | 12 |
| `unsupported-autoload-metadata` | 9 |
| `unsupported-class-contract-metadata` | 9 |
| `unsupported-magic-method-metadata` | 8 |
| `unsupported-method-visibility-metadata` | 7 |
| `unsupported-readonly-property-metadata` | 7 |
| `unsupported-internal-reflection-metadata` | 3 |

## Blocker Shape

The split keeps the previous safety boundary but makes the next implementation
frontier clearer:

- `unsupported-object-string-conversion-metadata`: `__toString()` conversion
  metadata used by comparisons, array helpers, diagnostics, and reflection.
- `unsupported-magic-method-metadata`: residual magic method dispatch,
  reflection names, dump behavior, and callback boundaries.
- `unsupported-property-visibility-metadata`: protected/private instance or
  static property visibility metadata, including array helper object access.
- `unsupported-typed-property-metadata`: typed instance/static property
  declaration metadata and enforcement.
- `unsupported-autoload-metadata`: runtime class symbol-table mutation through
  autoload callbacks.
- `unsupported-class-contract-metadata`: abstract/final class and method
  contract or override checks.
- `unsupported-method-visibility-metadata`: protected/private method dispatch
  and diagnostics.
- `unsupported-readonly-property-metadata`: readonly static property and
  indirect mutation diagnostics outside the current runnable subset.
- `unsupported-internal-reflection-metadata`: internal arginfo/class registry and
  reflection metadata not yet modeled.

Representative rows:

```text
Zend/tests/access_modifiers/access_modifiers_008.phpt
Zend/tests/asymmetric_visibility/static_props.phpt
Zend/tests/autoload/bug61011.phpt
Zend/tests/backtrace/debug_backtrace_options.phpt
ext/standard/tests/array/array_column_property_visibility.phpt
ext/standard/tests/array/array_map_object3.phpt
ext/standard/tests/array/array_reverse_variation3.phpt
ext/standard/tests/array/array_uintersect_assoc_basic.phpt
```

## Verification

```sh
cargo fmt --check
cargo test --test phpt_classifier
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-xfie-broad-current
PHPT_PROGRESS_DIR=.runtime/ptn-xfie-class-metadata-focused-final \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-broad-1k-class-metadata-split-ptn-xfie-manifest.txt
PHPT_PROGRESS_DIR=.runtime/ptn-xfie-class-metadata-current \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-broad-1k-class-metadata-split-ptn-xfie-manifest.txt
```

Results:

- `cargo fmt --check`: passed.
- `cargo test --test phpt_classifier`: passed, 35 tests.
- Broad classify-only after split: passed, 1000 selected, 424 runnable, 576
  excluded.
- Focused classify-only: passed, 135 selected, 0 runnable, 135 excluded.
